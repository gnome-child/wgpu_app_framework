#!/usr/bin/env python3
"""Validate a paired set of instrumented renderer receipts."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


SCHEMA = "wgpu_l3.renderer_receipt.v1"
MIN_PRESENT_SUBMITTED_FRAMES = 60
MIN_REFRESH_MILLIHERTZ = 59_000
MAX_REFRESH_MILLIHERTZ = 61_000

ENVIRONMENT_KEYS = (
    "os",
    "architecture",
    "adapter_name",
    "adapter_backend",
    "adapter_device_type",
    "adapter_vendor",
    "adapter_device",
    "presentation_system",
    "display_name",
    "display_refresh_millihertz",
    "scale_factor_milli",
    "surface_format",
    "alpha_mode",
    "present_mode",
    "desired_maximum_frame_latency",
)

REQUIRED_TEXT_KEYS = (
    "schema",
    "workload",
    *ENVIRONMENT_KEYS,
    "fallback_adapter_requested",
    "fallback_selection_verified",
)

REQUIRED_INTEGER_KEYS = (
    "frames_attempted",
    "frames_present_submitted",
    "frames_skipped",
    "redraw_requests_issued",
    "redraw_deliveries",
    "redraw_no_progress",
    "missed_refresh_opportunities",
    "renderer_deadline_misses",
    "virtual_guard_crossings",
    "replenishment_commits",
    "frame_interval_us_sample_count",
    "frame_interval_us_p95",
    "frame_interval_us_p99",
    "frame_interval_us_max",
    "draw_us_sample_count",
    "draw_us_p95",
    "draw_us_p99",
    "draw_us_max",
    "replenishment_commit_us_sample_count",
    "replenishment_commit_us_p95",
    "scene_paint_calls",
    "inline_text_shape_calls_total",
    "text_prepare_calls_total",
    "quad_prepare_calls_total",
    "content_upload_bytes_total",
    "property_upload_bytes",
    "render_plan_rebuilds_total",
    "render_plan_reuses_total",
    "full_surface_blits_total",
    "full_surface_blit_bytes_total",
    "acquire_successes",
)


class ReceiptError(ValueError):
    pass


@dataclass(frozen=True)
class Receipt:
    path: Path
    values: dict[str, str]

    def text(self, key: str) -> str:
        try:
            return self.values[key]
        except KeyError as error:
            raise ReceiptError(f"{self.path}: missing {key}") from error

    def integer(self, key: str) -> int:
        value = self.text(key)
        try:
            parsed = int(value)
        except ValueError as error:
            raise ReceiptError(f"{self.path}: {key} is not an integer: {value!r}") from error
        if parsed < 0:
            raise ReceiptError(f"{self.path}: {key} must be non-negative")
        return parsed


def parse_receipt(path: Path) -> Receipt:
    values: dict[str, str] = {}
    in_adapter_info = False
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        raise ReceiptError(f"cannot read {path}: {error}") from error

    for line_number, raw_line in enumerate(lines, start=1):
        line = raw_line.strip()
        if line == "adapter_info_begin":
            in_adapter_info = True
            continue
        if line.endswith("adapter_info_end"):
            in_adapter_info = False
            continue
        if in_adapter_info or not line:
            continue
        if "=" not in line:
            raise ReceiptError(f"{path}:{line_number}: malformed receipt line: {raw_line!r}")
        key, value = line.split("=", 1)
        if not key or key in values:
            reason = "empty key" if not key else f"duplicate key {key!r}"
            raise ReceiptError(f"{path}:{line_number}: {reason}")
        values[key] = value

    receipt = Receipt(path=path, values=values)
    for key in REQUIRED_TEXT_KEYS:
        if not receipt.text(key):
            raise ReceiptError(f"{path}: {key} must not be empty")
    for key in REQUIRED_INTEGER_KEYS:
        receipt.integer(key)
    return receipt


def _require(errors: list[str], condition: bool, message: str) -> None:
    if not condition:
        errors.append(message)


def _validate_common(
    receipt: Receipt,
    errors: list[str],
    *,
    require_field_igpu_60hz: bool,
) -> None:
    label = str(receipt.path)
    _require(errors, receipt.text("schema") == SCHEMA, f"{label}: unsupported schema")
    _require(errors, receipt.text("os") == "windows", f"{label}: OS is not windows")
    _require(errors, receipt.text("adapter_backend") == "Dx12", f"{label}: backend is not Dx12")
    _require(
        errors,
        receipt.text("fallback_adapter_requested") == "false",
        f"{label}: fallback adapter was requested",
    )
    _require(
        errors,
        receipt.text("fallback_selection_verified") == "true",
        f"{label}: adapter selection is not verified",
    )

    refresh = receipt.integer("display_refresh_millihertz")
    _require(errors, refresh > 0, f"{label}: display refresh must be positive")
    if require_field_igpu_60hz:
        _require(
            errors,
            receipt.text("adapter_device_type") == "IntegratedGpu",
            f"{label}: adapter is not IntegratedGpu",
        )
        _require(
            errors,
            MIN_REFRESH_MILLIHERTZ <= refresh <= MAX_REFRESH_MILLIHERTZ,
            f"{label}: refresh {refresh} mHz is outside the optional 60 Hz field rail",
        )

    attempted = receipt.integer("frames_attempted")
    present_submitted = receipt.integer("frames_present_submitted")
    skipped = receipt.integer("frames_skipped")
    _require(
        errors,
        attempted >= present_submitted,
        f"{label}: present-submitted frames exceed attempts",
    )
    _require(
        errors,
        skipped == attempted - present_submitted,
        f"{label}: frames_skipped is inconsistent",
    )
    _require(
        errors,
        present_submitted >= MIN_PRESENT_SUBMITTED_FRAMES,
        f"{label}: only {present_submitted} frames reached present submission; need at least {MIN_PRESENT_SUBMITTED_FRAMES}",
    )
    _require(
        errors,
        receipt.integer("draw_us_sample_count") == present_submitted,
        f"{label}: draw sample count does not match present-submitted frames",
    )
    _require(
        errors,
        receipt.integer("acquire_successes") == present_submitted,
        f"{label}: acquire successes do not match present-submitted frames",
    )
    _require(
        errors,
        receipt.integer("frame_interval_us_sample_count") >= MIN_PRESENT_SUBMITTED_FRAMES - 1,
        f"{label}: too few frame-interval samples",
    )
    _require(
        errors,
        receipt.integer("redraw_no_progress") <= receipt.integer("redraw_deliveries"),
        f"{label}: no-progress redraws exceed delivered redraws",
    )


def validate_pair(
    in_window: Receipt,
    guard: Receipt,
    *,
    require_final_renderer_budget: bool = False,
    require_field_igpu_60hz: bool = False,
) -> dict[str, object]:
    errors: list[str] = []
    _validate_common(
        in_window,
        errors,
        require_field_igpu_60hz=require_field_igpu_60hz,
    )
    _validate_common(
        guard,
        errors,
        require_field_igpu_60hz=require_field_igpu_60hz,
    )

    for key in ENVIRONMENT_KEYS:
        _require(
            errors,
            in_window.text(key) == guard.text(key),
            f"receipts disagree on {key}",
        )

    _require(errors, in_window.text("workload") != guard.text("workload"), "workloads must differ")
    in_window_name = in_window.text("workload").lower()
    guard_name = guard.text("workload").lower()
    _require(errors, "500" in in_window_name and "in-window" in in_window_name, "in-window workload is mislabeled")
    _require(errors, "800" in guard_name and "guard" in guard_name, "guard workload is mislabeled")

    _require(
        errors,
        in_window.integer("virtual_guard_crossings") == 0,
        "in-window receipt crossed a virtual guard",
    )
    _require(
        errors,
        in_window.integer("replenishment_commits") == 0,
        "in-window receipt contains replenishment commits",
    )
    _require(
        errors,
        in_window.integer("replenishment_commit_us_sample_count") == 0,
        "in-window receipt contains replenishment timing samples",
    )

    guard_crossings = guard.integer("virtual_guard_crossings")
    replenishments = guard.integer("replenishment_commits")
    _require(errors, guard_crossings > 0, "guard receipt contains no guard crossing")
    _require(
        errors,
        replenishments == guard_crossings,
        "guard crossings and replenishment commits differ",
    )
    _require(
        errors,
        guard.integer("replenishment_commit_us_sample_count") == replenishments,
        "guard replenishment sample count is inconsistent",
    )

    refresh = in_window.integer("display_refresh_millihertz")
    refresh_budget_us = 1_000_000_000 / refresh

    def workload_summary(receipt: Receipt) -> dict[str, object]:
        draw_p95 = receipt.integer("draw_us_p95")
        renderer_deadline_misses = receipt.integer("renderer_deadline_misses")
        renderer_budget_met = draw_p95 <= refresh_budget_us and renderer_deadline_misses == 0
        if require_final_renderer_budget:
            _require(
                errors,
                draw_p95 <= refresh_budget_us,
                f"{receipt.path}: draw p95 {draw_p95} us exceeds {refresh_budget_us:.1f} us",
            )
            _require(
                errors,
                renderer_deadline_misses == 0,
                f"{receipt.path}: renderer missed {renderer_deadline_misses} refresh deadlines",
            )
        return {
            "workload": receipt.text("workload"),
            "frames_attempted": receipt.integer("frames_attempted"),
            "frames_present_submitted": receipt.integer("frames_present_submitted"),
            "frames_skipped": receipt.integer("frames_skipped"),
            "redraw_requests_issued": receipt.integer("redraw_requests_issued"),
            "redraw_deliveries": receipt.integer("redraw_deliveries"),
            "redraw_no_progress": receipt.integer("redraw_no_progress"),
            "frame_interval_us_p95": receipt.integer("frame_interval_us_p95"),
            "frame_interval_us_p99": receipt.integer("frame_interval_us_p99"),
            "frame_interval_us_max": receipt.integer("frame_interval_us_max"),
            "missed_refresh_opportunities": receipt.integer("missed_refresh_opportunities"),
            "draw_us_p95": draw_p95,
            "draw_us_p99": receipt.integer("draw_us_p99"),
            "draw_us_max": receipt.integer("draw_us_max"),
            "renderer_deadline_misses": renderer_deadline_misses,
            "renderer_budget_met": renderer_budget_met,
            "virtual_guard_crossings": receipt.integer("virtual_guard_crossings"),
            "replenishment_commit_us_p95": receipt.integer("replenishment_commit_us_p95"),
            "scene_paint_calls": receipt.integer("scene_paint_calls"),
            "inline_text_shape_calls_total": receipt.integer("inline_text_shape_calls_total"),
            "text_prepare_calls_total": receipt.integer("text_prepare_calls_total"),
            "quad_prepare_calls_total": receipt.integer("quad_prepare_calls_total"),
            "content_upload_bytes_total": receipt.integer("content_upload_bytes_total"),
            "property_upload_bytes": receipt.integer("property_upload_bytes"),
            "render_plan_rebuilds_total": receipt.integer("render_plan_rebuilds_total"),
            "render_plan_reuses_total": receipt.integer("render_plan_reuses_total"),
            "full_surface_blits_total": receipt.integer("full_surface_blits_total"),
            "full_surface_blit_bytes_total": receipt.integer("full_surface_blit_bytes_total"),
        }

    summary: dict[str, object] = {
        "schema": SCHEMA,
        "environment": {key: in_window.text(key) for key in ENVIRONMENT_KEYS},
        "refresh_budget_us": round(refresh_budget_us, 3),
        "in_window": workload_summary(in_window),
        "guard": workload_summary(guard),
        "final_renderer_budget_required": require_final_renderer_budget,
        "field_igpu_60hz_required": require_field_igpu_60hz,
    }
    if errors:
        raise ReceiptError("\n".join(errors))
    return summary


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--in-window", required=True, type=Path, help="500 px in-window receipt")
    parser.add_argument("--guard", required=True, type=Path, help="800 px guard-boundary receipt")
    parser.add_argument(
        "--require-final-renderer-budget",
        action="store_true",
        help="also require draw p95 inside one refresh and zero renderer deadline misses",
    )
    parser.add_argument(
        "--require-field-igpu-60hz",
        action="store_true",
        help="optionally require a 60 Hz integrated-GPU environment",
    )
    return parser


def main(argv: Iterable[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        summary = validate_pair(
            parse_receipt(args.in_window),
            parse_receipt(args.guard),
            require_final_renderer_budget=args.require_final_renderer_budget,
            require_field_igpu_60hz=args.require_field_igpu_60hz,
        )
    except ReceiptError as error:
        print(f"renderer receipt validation failed:\n{error}", file=sys.stderr)
        return 1
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
