#!/usr/bin/env python3
"""Generate the bounded deterministic SC-000 pairwise scroll manifest."""

from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import random
from collections import Counter
from pathlib import Path
from typing import Iterable


SCHEMA = "wgpu_l3.scroll_pairwise_manifest.v1"
GENERATOR_VERSION = 1
SEED = 20260716
CASE_CAP = 128
DEFAULT_OUTPUT = (
    Path(__file__).resolve().parents[1]
    / "docs"
    / "audits"
    / "fixtures"
    / "scroll-pairwise-manifest-v1.json"
)

DIMENSIONS = {
    "fixture": [
        "F01",
        "F02",
        "F03",
        "F04",
        "F05",
        "F06",
        "F07",
        "F08",
        "repeated-sibling-scopes",
        "empty-payload",
        "large-unrelated-property-scene",
        "pane-filter-surface",
    ],
    "input": [
        "pixel-wheel-trackpad",
        "line-wheel",
        "thumb-drag",
        "keyboard",
        "caret-reveal",
        "programmatic-absolute",
        "residency-request",
    ],
    "scale": ["1.0", "1.25", "1.5", "1.75", "2.0"],
    "tick": [
        "first-property",
        "coalesced-events",
        "no-op",
        "bound-saturation",
        "first-after-residency",
        "next-semantic-rebuild",
    ],
    "direction": ["forward", "reverse", "diagonal", "one-axis-saturated"],
}


def invalid_reason(case: tuple[str, ...], names: tuple[str, ...]) -> str | None:
    values = dict(zip(names, case, strict=True))
    if values["direction"] == "diagonal" and values["fixture"] != "F08":
        return "diagonal-requires-split-axis-fixture"
    return None


def pairs(case: tuple[str, ...]) -> frozenset[tuple[int, str, int, str]]:
    return frozenset(
        (left, case[left], right, case[right])
        for left, right in itertools.combinations(range(len(case)), 2)
    )


def valid_cases() -> tuple[tuple[str, ...], list[tuple[str, ...]], Counter[str]]:
    names = tuple(DIMENSIONS)
    valid: list[tuple[str, ...]] = []
    rejected: Counter[str] = Counter()
    for case in itertools.product(*(DIMENSIONS[name] for name in names)):
        reason = invalid_reason(case, names)
        if reason is None:
            valid.append(case)
        else:
            rejected[reason] += 1
    return names, valid, rejected


def select_pairwise(candidates: Iterable[tuple[str, ...]]) -> list[tuple[str, ...]]:
    candidates = list(candidates)
    random.Random(SEED).shuffle(candidates)
    coverage = [pairs(case) for case in candidates]
    uncovered = set().union(*coverage)
    selected: list[tuple[str, ...]] = []

    while uncovered:
        best_index = max(
            range(len(candidates)),
            key=lambda index: len(coverage[index] & uncovered),
        )
        gain = coverage[best_index] & uncovered
        if not gain:
            raise RuntimeError(f"unable to cover {len(uncovered)} valid pairs")
        selected.append(candidates[best_index])
        uncovered.difference_update(gain)
        candidates.pop(best_index)
        coverage.pop(best_index)

    return selected


def manifest() -> dict[str, object]:
    names, candidates, rejected = valid_cases()
    selected = select_pairwise(candidates)
    if len(selected) > CASE_CAP:
        raise RuntimeError(
            f"generated {len(selected)} cases, exceeding the campaign cap of {CASE_CAP}"
        )

    cases = [
        {"id": f"P{index:03}", **dict(zip(names, case, strict=True))}
        for index, case in enumerate(selected, start=1)
    ]
    canonical_cases = json.dumps(cases, sort_keys=True, separators=(",", ":")).encode()
    return {
        "schema": SCHEMA,
        "generator_version": GENERATOR_VERSION,
        "seed": SEED,
        "case_cap": CASE_CAP,
        "candidate_count": len(candidates),
        "rejected_combinations": dict(sorted(rejected.items())),
        "case_count": len(cases),
        "cases_sha256": hashlib.sha256(canonical_cases).hexdigest(),
        "dimensions": DIMENSIONS,
        "cases": cases,
    }


def encoded_manifest() -> str:
    return json.dumps(manifest(), indent=2) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    encoded = encoded_manifest()

    if args.check:
        if not args.output.exists():
            raise SystemExit(f"manifest is missing: {args.output}")
        if args.output.read_text(encoding="utf-8") != encoded:
            raise SystemExit(f"manifest is stale: {args.output}")
        data = json.loads(encoded)
        print(
            f"manifest_ok cases={data['case_count']} "
            f"sha256={data['cases_sha256']}"
        )
        return 0

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(encoded, encoding="utf-8", newline="\n")
    data = json.loads(encoded)
    print(
        f"manifest_written path={args.output} cases={data['case_count']} "
        f"sha256={data['cases_sha256']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
