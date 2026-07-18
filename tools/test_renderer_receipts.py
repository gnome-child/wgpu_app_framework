from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from check_renderer_receipts import (  # noqa: E402
    ReceiptError,
    _validate_common,
    parse_receipt,
    validate_pair,
)


def receipt_text(
    *,
    workload: str,
    guard_crossings: int,
    draw_p95: int = 12_000,
    device_type: str = "DiscreteGpu",
    refresh_millihertz: int = 240_000,
) -> str:
    present_submitted = 120
    values = {
        "schema": "wgpu_l3.renderer_receipt.v1",
        "presentation_receipt_schema": "wgpu_l3.presentation_compiler.v1",
        "presentation_receipt_complete": "true",
        "workload": workload,
        "os": "windows",
        "architecture": "x86_64",
        "adapter_name": "Test GPU",
        "adapter_backend": "Dx12",
        "adapter_device_type": device_type,
        "adapter_vendor": "1",
        "adapter_device": "2",
        "presentation_system": "DxgiFromVisual",
        "display_name": r"\\.\DISPLAY1",
        "display_refresh_millihertz": str(refresh_millihertz),
        "scale_factor_milli": "1000",
        "surface_format": "Bgra8UnormSrgb",
        "alpha_mode": "Auto",
        "present_mode": "Fifo",
        "desired_maximum_frame_latency": "2",
        "fallback_adapter_requested": "false",
        "fallback_selection_verified": "true",
        "frames_attempted": str(present_submitted),
        "frames_prepared": str(present_submitted),
        "frames_present_submitted": str(present_submitted),
        "frames_skipped": "0",
        "redraw_requests_issued": str(present_submitted),
        "redraw_deliveries": str(present_submitted),
        "redraw_no_progress": "0",
        "missed_refresh_opportunities": "0",
        "renderer_deadline_misses": "0" if draw_p95 < 16_000 else "1",
        "virtual_guard_crossings": str(guard_crossings),
        "replenishment_commits": str(guard_crossings),
        "frame_interval_us_sample_count": str(present_submitted - 1),
        "frame_interval_us_p95": "16667",
        "frame_interval_us_p99": "17000",
        "frame_interval_us_max": "18000",
        "draw_us_sample_count": str(present_submitted),
        "draw_us_p95": str(draw_p95),
        "draw_us_p99": str(draw_p95 + 500),
        "draw_us_max": str(draw_p95 + 1000),
        "replenishment_commit_us_sample_count": str(guard_crossings),
        "replenishment_commit_us_p95": "4000" if guard_crossings else "0",
        "scene_paint_calls": str(present_submitted),
        "inline_text_shape_calls_total": "20",
        "text_prepare_calls_total": "20",
        "quad_prepare_calls_total": "20",
        "content_upload_bytes_total": "1024",
        "property_upload_bytes": "0",
        "render_plan_rebuilds_total": str(present_submitted),
        "render_plan_reuses_total": "0",
        "full_surface_blits_total": str(present_submitted),
        "full_surface_blit_bytes_total": "4096",
        "acquire_successes": str(present_submitted),
        "presentation_frames_recorded": str(present_submitted),
        "primary_idle_frames": "0",
        "primary_property_frames": str(present_submitted - guard_crossings),
        "primary_residency_frames": str(guard_crossings),
        "primary_paint_frames": "0",
        "primary_layout_frames": "0",
        "primary_rebuild_frames": "0",
        "property_species_frames": str(present_submitted),
        "residency_species_frames": str(guard_crossings),
        "semantic_species_frames": "0",
        "device_species_frames": "0",
        "diagnostic_species_frames": "0",
        "mixed_property_residency_frames": str(guard_crossings),
        "materialization_calls": str(guard_crossings),
        "entering_rows": str(guard_crossings * 4),
        "departing_rows": str(guard_crossings * 4),
        "overlapping_rows": str(guard_crossings * 20),
        "revised_rows": "0",
        "moved_rows": "0",
        "membership_change_events": "0",
        "provider_binds": str(guard_crossings * 4),
        "view_nodes_cloned": str(guard_crossings * 24),
        "composition_reconciliations": str(guard_crossings),
        "composition_nodes_visited": str(guard_crossings * 24),
        "composition_nodes_reconstructed": str(guard_crossings * 24),
        "composition_identities_reused": str(guard_crossings * 20),
        "composition_nodes_added": str(guard_crossings * 4),
        "composition_nodes_changed": "0",
        "composition_nodes_removed": str(guard_crossings * 4),
        "layout_candidates": str(present_submitted),
        "layout_nodes_visited": str(present_submitted * 24),
        "layout_nodes_reused": str((present_submitted - guard_crossings) * 24),
        "layout_reused_candidates": str(present_submitted - guard_crossings),
        "scene_frames_scanned": str(present_submitted * 24),
        "scene_frames_painted": str(guard_crossings * 24),
        "scene_frames_reused": str((present_submitted - guard_crossings) * 24),
        "scene_row_fragments_spliced": str((present_submitted - guard_crossings) * 4),
        "scene_row_fragments_built": str(guard_crossings * 4),
        "scene_row_roots_visited": str(present_submitted * 20),
        "scene_commit_layout_frames_visited": str(present_submitted * 24),
        "scene_commit_nodes_registered": str(present_submitted * 24),
        "scene_commit_fragments_appended": str(present_submitted * 24),
        "scene_commit_draw_ops_lowered": str(present_submitted * 24),
        "scene_cache_entries_swept": str(present_submitted * 48),
        "scene_semantic_candidate_nodes_visited": str(guard_crossings * 24),
        "scene_semantic_candidate_draws_visited": str(guard_crossings * 24),
        "scene_residency_layout_frames_visited": str(guard_crossings * 24),
        "scene_residency_drawable_nodes_visited": str(guard_crossings * 48),
        "scene_residency_draw_ops_visited": str(guard_crossings * 24),
        "scene_residency_snapshot_nodes_built": str(guard_crossings * 24),
        "presentation_frame_total_p95_us": "900",
        "presentation_materialization_p95_us": "100" if guard_crossings else "0",
        "presentation_reconciliation_p95_us": "100" if guard_crossings else "0",
    }
    return "\n".join(f"{key}={value}" for key, value in values.items()) + "\n"


class RendererReceiptTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def write(self, name: str, contents: str) -> Path:
        path = self.root / name
        path.write_text(contents, encoding="utf-8")
        return path

    def pair(self, *, draw_p95: int = 12_000):
        in_window = parse_receipt(
            self.write(
                "in-window.txt",
                receipt_text(
                    workload="field-igpu-60hz-500px-in-window-scroll",
                    guard_crossings=0,
                    draw_p95=draw_p95,
                ),
            )
        )
        guard = parse_receipt(
            self.write(
                "guard.txt",
                receipt_text(
                    workload="field-igpu-60hz-800px-guard-replenishment",
                    guard_crossings=3,
                    draw_p95=draw_p95,
                ),
            )
        )
        return in_window, guard

    def test_valid_local_pair_reports_environment_topology_and_budget(self) -> None:
        summary = validate_pair(*self.pair())
        self.assertEqual(summary["environment"]["adapter_device_type"], "DiscreteGpu")
        self.assertEqual(summary["in_window"]["virtual_guard_crossings"], 0)
        self.assertEqual(summary["guard"]["virtual_guard_crossings"], 3)
        self.assertFalse(summary["in_window"]["renderer_budget_met"])

    def test_long_receipt_uses_the_bounded_timing_sample_population(self) -> None:
        receipt, _ = self.pair()
        for key in (
            "frames_attempted",
            "frames_prepared",
            "frames_present_submitted",
            "acquire_successes",
            "presentation_frames_recorded",
            "primary_property_frames",
            "property_species_frames",
        ):
            receipt.values[key] = "327"
        receipt.values["draw_us_sample_count"] = "128"
        receipt.values["frame_interval_us_sample_count"] = "128"
        errors: list[str] = []
        _validate_common(receipt, errors, require_field_igpu_60hz=False)
        self.assertEqual(errors, [])

    def test_optional_field_policy_requires_60_hz_integrated_gpu(self) -> None:
        with self.assertRaisesRegex(ReceiptError, "not IntegratedGpu"):
            validate_pair(*self.pair(), require_field_igpu_60hz=True)

        in_window, guard = self.pair()
        for receipt in (in_window, guard):
            receipt.values["adapter_device_type"] = "IntegratedGpu"
            receipt.values["display_refresh_millihertz"] = "59940"
        summary = validate_pair(
            in_window,
            guard,
            require_field_igpu_60hz=True,
        )
        self.assertTrue(summary["field_igpu_60hz_required"])

    def test_baseline_can_be_validated_without_pretending_to_meet_final_budget(self) -> None:
        pair = self.pair(draw_p95=20_000)
        summary = validate_pair(*pair)
        self.assertFalse(summary["in_window"]["renderer_budget_met"])
        with self.assertRaisesRegex(ReceiptError, "exceeds"):
            validate_pair(*pair, require_final_renderer_budget=True)

    def test_guard_receipt_must_witness_replenishment(self) -> None:
        in_window, _ = self.pair()
        bad_guard = parse_receipt(
            self.write(
                "bad-guard.txt",
                receipt_text(
                    workload="field-igpu-60hz-800px-guard-replenishment",
                    guard_crossings=0,
                ),
            )
        )
        with self.assertRaisesRegex(ReceiptError, "no guard crossing"):
            validate_pair(in_window, bad_guard)

    def test_duplicate_key_is_rejected(self) -> None:
        path = self.write(
            "duplicate.txt",
            receipt_text(
                workload="field-igpu-60hz-500px-in-window-scroll",
                guard_crossings=0,
            )
            + "schema=second\n",
        )
        with self.assertRaisesRegex(ReceiptError, "duplicate key"):
            parse_receipt(path)

    def test_incomplete_presentation_receipt_is_rejected(self) -> None:
        in_window, guard = self.pair()
        guard.values["presentation_receipt_complete"] = "false"
        with self.assertRaisesRegex(ReceiptError, "presentation receipt is incomplete"):
            validate_pair(in_window, guard)

    def test_erased_residency_species_is_rejected(self) -> None:
        in_window, guard = self.pair()
        guard.values["residency_species_frames"] = "0"
        with self.assertRaisesRegex(ReceiptError, "erased its residency species"):
            validate_pair(in_window, guard)

    def test_forged_second_primary_reason_is_rejected(self) -> None:
        in_window, guard = self.pair()
        guard.values["primary_layout_frames"] = "1"
        with self.assertRaisesRegex(ReceiptError, "not mutually exhaustive"):
            validate_pair(in_window, guard)


if __name__ == "__main__":
    unittest.main()
