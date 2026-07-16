#!/usr/bin/env python3

import hashlib
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import generate_scroll_pairwise_manifest as generator  # noqa: E402


class ScrollPairwiseManifestTests(unittest.TestCase):
    def test_checked_in_manifest_is_current_and_bounded(self) -> None:
        checked_in = json.loads(generator.DEFAULT_OUTPUT.read_text(encoding="utf-8"))
        expected = generator.manifest()

        self.assertEqual(checked_in, expected)
        self.assertLessEqual(checked_in["case_count"], generator.CASE_CAP)
        self.assertEqual(
            [case["id"] for case in checked_in["cases"]],
            [f"P{index:03}" for index in range(1, checked_in["case_count"] + 1)],
        )

    def test_cases_cover_every_valid_pair(self) -> None:
        names, candidates, _ = generator.valid_cases()
        required_pairs = set().union(*(generator.pairs(case) for case in candidates))
        selected = [
            tuple(case[name] for name in names)
            for case in generator.manifest()["cases"]
        ]
        covered_pairs = set().union(*(generator.pairs(case) for case in selected))

        self.assertEqual(required_pairs - covered_pairs, set())
        for case in selected:
            self.assertIsNone(generator.invalid_reason(case, names))

    def test_case_hash_covers_canonical_case_payload(self) -> None:
        manifest = generator.manifest()
        encoded = json.dumps(
            manifest["cases"], sort_keys=True, separators=(",", ":")
        ).encode()

        self.assertEqual(
            manifest["cases_sha256"], hashlib.sha256(encoded).hexdigest()
        )


if __name__ == "__main__":
    unittest.main()
