#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from one_way_census import (  # noqa: E402
    Edge,
    belongs_to_test_only_module,
    effective_edges,
    external_module_candidates,
    mask_rust_literals_and_comments,
    partition_test_code,
    referenced_roots,
    test_only_module_roots,
    uses_dependency,
)


class RustPathCensusTests(unittest.TestCase):
    def references(self, source: str, parts: list[str] | None = None) -> set[str]:
        masked = mask_rust_literals_and_comments(source)
        roots, _ = referenced_roots(masked, parts or ["runtime", "input"])
        return roots

    def test_expands_grouped_crate_import_roots(self) -> None:
        roots = self.references(
            "use crate::{command::{self, Command}, geometry, text::layout};"
        )
        self.assertEqual(roots, {"command", "geometry", "text"})

    def test_resolves_super_paths_against_module_depth(self) -> None:
        roots = self.references(
            "use super::local; use super::super::command::Command;",
            ["runtime", "input", "text"],
        )
        self.assertEqual(roots, {"runtime"})
        root_level = self.references("use super::command::Command;", ["runtime"])
        self.assertEqual(root_level, {"command"})

    def test_expands_grouped_super_imports_at_the_resolved_level(self) -> None:
        root_level = self.references(
            "use super::{clipboard::Clipboard, layout, task};", ["context"]
        )
        self.assertEqual(root_level, {"clipboard", "layout", "task"})
        nested = self.references(
            "use super::super::{interaction, scene};", ["runtime", "input"]
        )
        self.assertEqual(nested, {"interaction", "scene"})
        local = self.references(
            "use super::{focus, key};", ["runtime", "input", "text"]
        )
        self.assertEqual(local, {"runtime"})

    def test_masks_comments_and_string_receipts(self) -> None:
        roots = self.references(
            '// crate::platform::Fake\nconst RECEIPT: &str = "crate::render::Fake";\n'
            "use crate::text::Text;"
        )
        self.assertEqual(roots, {"text"})

    def test_separates_cfg_test_items(self) -> None:
        source = """
use crate::text::Text;
#[cfg(test)]
mod tests {
    use crate::{platform, render};
}
"""
        masked = mask_rust_literals_and_comments(source)
        production, tests = partition_test_code(masked)
        production_roots, _ = referenced_roots(production, ["view"])
        test_roots, _ = referenced_roots(tests, ["view"])
        self.assertEqual(production_roots, {"text"})
        self.assertEqual(test_roots, {"platform", "render"})

    def test_resolves_external_cfg_test_modules_and_their_descendants(self) -> None:
        src = Path("workspace/src")
        lib = src / "lib.rs"
        live = src / "live.rs"
        tests = src / "checks.rs"
        child = src / "checks" / "child.rs"
        masked = {
            lib: mask_rust_literals_and_comments(
                "mod live;\n#[cfg(test)]\nmod checks;\n"
            ),
            live: "",
            tests: "",
            child: "",
        }

        roots = test_only_module_roots(src, masked)

        self.assertEqual(roots, {("checks",)})
        self.assertTrue(belongs_to_test_only_module(["checks"], roots))
        self.assertTrue(belongs_to_test_only_module(["checks", "child"], roots))
        self.assertFalse(belongs_to_test_only_module(["live"], roots))

    def test_external_module_candidates_follow_rust_file_housing(self) -> None:
        self.assertEqual(
            external_module_candidates(Path("src/lib.rs"), "tests"),
            (Path("src/tests.rs"), Path("src/tests/mod.rs")),
        )
        self.assertEqual(
            external_module_candidates(Path("src/render/filter.rs"), "tests"),
            (
                Path("src/render/filter/tests.rs"),
                Path("src/render/filter/tests/mod.rs"),
            ),
        )

    def test_external_dependency_does_not_match_a_nested_path_segment(self) -> None:
        self.assertTrue(uses_dependency("use windows::Win32;", "windows"))
        self.assertFalse(uses_dependency("use std::os::windows::ffi;", "windows"))

    def test_source_responsibilities_refine_only_their_own_receipts(self) -> None:
        edge = Edge(
            "view",
            "diagnostics",
            {"src/view/context.rs", "src/view/mod.rs"},
        )

        resolved = effective_edges(
            {("view", "diagnostics"): edge},
            {"view": "ui", "diagnostics": "diagnostics"},
            {"src/view/context.rs": "facade"},
        )

        self.assertEqual(
            [
                (
                    item.source_slot,
                    item.target_slot,
                    item.locations,
                )
                for item in resolved
            ],
            [
                ("facade", "diagnostics", {"src/view/context.rs"}),
                ("ui", "diagnostics", {"src/view/mod.rs"}),
            ],
        )


if __name__ == "__main__":
    unittest.main()
