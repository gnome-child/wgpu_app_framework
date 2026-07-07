#[test]
fn promoted_framework_has_no_scratch_or_legacy_root_surface() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let retired = [
        "scratch",
        "app",
        "ui",
        "native",
        "action.rs",
        "event.rs",
        "path.rs",
        "pointer.rs",
    ];

    for entry in retired {
        let path = src_dir.join(entry);
        assert!(
            !path.exists(),
            "{} should be absent after promoting scratch to the root framework",
            path.display()
        );
    }
}

#[test]
fn root_source_tree_has_no_empty_concept_buckets() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    for entry in std::fs::read_dir(&src_dir).expect("source directory should be readable") {
        let path = entry.expect("source entry should be readable").path();
        if !path.is_dir() {
            continue;
        }

        let mut entries = std::fs::read_dir(&path).unwrap_or_else(|error| {
            panic!("{} should be readable: {error}", path.display());
        });
        assert!(
            entries.next().is_some(),
            "{} is an empty root-level concept bucket",
            path.display()
        );
    }
}

#[test]
fn renderer_dependencies_stay_in_native_platform() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let allowed_roots = [
        src_dir.join("platform").join("native"),
        src_dir.join("render"),
        src_dir.join("paint"),
        src_dir.join("paint_geometry"),
    ];
    let renderer_modules = ["paint", "render"];

    assert_imports_only_under_any(&src_dir, &allowed_roots, &renderer_modules);
}

#[test]
fn renderer_paint_vocabulary_stays_private() {
    let lib = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("lib.rs"),
    )
    .expect("crate root should read");

    assert!(
        !lib.contains("pub mod paint;"),
        "renderer-facing paint vocabulary should not be public framework API"
    );
    assert!(
        !lib.contains("pub mod paint_geometry;"),
        "renderer-space geometry should not be public framework API"
    );
}

#[test]
fn renderer_file_modules_stay_private() {
    let render_mod = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("render")
            .join("mod.rs"),
    )
    .expect("render module should read");

    for module in [
        "canvas",
        "context",
        "frame",
        "primitive",
        "renderer",
        "surface",
    ] {
        assert!(
            !render_mod.contains(&format!("pub mod {module};")),
            "private renderer file module should not be part of the renderer facade: {module}"
        );
    }
}

#[test]
fn renderer_adapter_helpers_stay_crate_private() {
    let render_mod = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("render")
            .join("mod.rs"),
    )
    .expect("render module should read");

    for item in [
        "pub fn color_to_wgpu",
        "pub struct Scissor",
        "pub type Result",
    ] {
        assert!(
            !render_mod.contains(item),
            "renderer adapter helper should stay crate-private: {item}"
        );
    }
}

#[test]
fn paint_geometry_stays_below_text_and_native_rendering() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let allowed_roots = [
        src_dir.join("paint_geometry"),
        src_dir.join("paint"),
        src_dir.join("render"),
        src_dir.join("platform").join("native"),
        src_dir.join("text"),
    ];

    assert_imports_only_under_any(&src_dir, &allowed_roots, &["paint_geometry"]);
}

#[test]
fn paint_geometry_file_modules_stay_crate_private() {
    let paint_geometry_mod = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("paint_geometry")
            .join("mod.rs"),
    )
    .expect("paint geometry module should read");

    for module in ["area", "point", "rect"] {
        assert!(
            !paint_geometry_mod.contains(&format!("pub mod {module};")),
            "paint geometry file module should stay crate-private: {module}"
        );
    }
}

#[test]
fn geometry_file_modules_stay_private() {
    let geometry_mod = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("geometry")
            .join("mod.rs"),
    )
    .expect("geometry module should read");

    for module in ["area", "point", "rect", "size"] {
        assert!(
            !geometry_mod.contains(&format!("pub mod {module};")),
            "public geometry API should expose concepts through the facade, not file modules: {module}"
        );
    }
}

#[test]
fn text_buffer_mark_module_stays_private() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let buffer_mod = std::fs::read_to_string(src_dir.join("text").join("buffer").join("mod.rs"))
        .expect("text buffer module should read");

    assert!(
        !buffer_mod.contains("pub mod mark;"),
        "text buffer mark file module must stay private; re-export Mark, MarkRange, and MarkGravity"
    );
}

#[test]
fn text_edit_surface_module_stays_private() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let edit_mod = std::fs::read_to_string(src_dir.join("text").join("edit").join("mod.rs"))
        .expect("text edit module should read");

    assert!(
        !edit_mod.contains("pub mod surface;"),
        "text edit surface file module must stay private; re-export named surface concepts instead"
    );
}

#[test]
fn text_edit_implementation_modules_stay_private() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let edit_mod = std::fs::read_to_string(src_dir.join("text").join("edit").join("mod.rs"))
        .expect("text edit module should read");

    for module in ["outcome", "transaction"] {
        assert!(
            !edit_mod.contains(&format!("pub(crate) mod {module};"))
                && !edit_mod.contains(&format!("pub mod {module};")),
            "text edit implementation module must stay private behind named re-exports: {module}"
        );
    }
}

#[test]
fn text_unicode_helpers_stay_private_to_text_engine() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let text_mod =
        std::fs::read_to_string(src_dir.join("text").join("mod.rs")).expect("text module read");
    let unicode =
        std::fs::read_to_string(src_dir.join("text").join("unicode.rs")).expect("unicode read");

    assert!(
        !text_mod.contains("pub mod unicode;"),
        "text unicode helpers should stay private to the text engine"
    );
    assert!(
        !unicode.contains("pub(crate) fn"),
        "unicode helpers should not expose crate-wide helper functions"
    );
}

#[test]
fn text_layout_system_module_stays_private() {
    let layout_mod = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("text")
            .join("layout")
            .join("mod.rs"),
    )
    .expect("text layout module should read");

    assert!(
        !layout_mod.contains("pub(crate) mod system;") && !layout_mod.contains("pub mod system;"),
        "glyphon system adapters should stay behind the text layout facade"
    );
}

#[test]
fn draft_input_module_stays_private() {
    let draft_mod = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("draft")
            .join("mod.rs"),
    )
    .expect("draft module should read");

    assert!(
        !draft_mod.contains("pub(crate) mod input;") && !draft_mod.contains("pub mod input;"),
        "draft input file module must stay private; re-export Input and retention constants"
    );
}

#[test]
fn view_action_module_stays_private() {
    let view_mod = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("view")
            .join("mod.rs"),
    )
    .expect("view module should read");

    assert!(
        !view_mod.contains("pub mod action;"),
        "view action file module must stay private; expose the Action concept through view::Action"
    );
}

#[test]
fn demo_apps_do_not_leak_into_framework_source_or_public_api() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let examples_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let lib = std::fs::read_to_string(src_dir.join("lib.rs")).expect("crate root should read");

    for module in ["control_gallery", "glass_tuner", "text_editor"] {
        assert!(
            !src_dir.join(module).exists(),
            "demo app module {module} should live under examples, not src"
        );
        assert!(
            !lib.contains(&format!("pub mod {module};")),
            "demo app module {module} should not be public framework API"
        );

        let main = std::fs::read_to_string(examples_dir.join(module).join("main.rs"))
            .expect("example main should read");
        assert!(
            !main.contains("pub use wgpu_l3::*;"),
            "example {module} should import framework APIs explicitly, not re-export the crate"
        );
        assert_source_patterns_absent(
            &examples_dir.join(module).join("app"),
            &["super::super::".to_owned()],
        );
    }
}

#[test]
fn responder_chain_uses_service_responders_not_framework_fallbacks() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let forbidden = [
        format!("{}{}", "trait ", "Framework"),
        format!("{}{}", "with_", "framework"),
        format!("{}{}", "responder::", "Framework"),
        format!("{}{}", "mod ", "framework;"),
        format!("{}{}", "services/", "framework.rs"),
        format!("{}{}", "framework_", "command"),
        format!("{}{}", "Framework", "Runtime"),
        format!("{}{}", "framework_", "view"),
        format!("{}{}", "framework_", "shell"),
        format!("{}{}", "framework_", "icon"),
    ];

    assert_source_patterns_absent(&src_dir, &forbidden);
}

#[test]
fn runtime_services_use_typed_provider_targets() {
    let services_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("runtime")
        .join("services");
    let forbidden = [
        "AnyTarget::new::<".to_owned(),
        "target::args::<".to_owned(),
        "target::args_box::<".to_owned(),
        "text_target!(".to_owned(),
        "fn command_name(".to_owned(),
        format!("{}{}", "framework_", "command"),
    ];

    assert_source_patterns_absent(&services_dir, &forbidden);
}

#[test]
fn focused_text_service_stays_behind_runtime_boundary() {
    let runtime_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("runtime");
    let services_mod = runtime_dir.join("services").join("mod.rs");
    let input_mod = runtime_dir.join("input").join("mod.rs");
    let services_source =
        std::fs::read_to_string(&services_mod).expect("runtime services module should read");
    let input_source =
        std::fs::read_to_string(&input_mod).expect("runtime input module should read");

    assert!(
        !services_source.contains("pub(in crate::runtime) mod text;"),
        "{} must keep focused text service private to runtime services",
        services_mod.display()
    );
    assert!(
        !input_source.contains("pub(in crate::runtime) mod text;"),
        "{} must keep text input internals private to runtime input",
        input_mod.display()
    );
    assert_source_patterns_absent(&runtime_dir, &[format!("{}{}", "services::", "text")]);
}

#[test]
fn composition_tree_owns_identity_not_behavior() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let lib = std::fs::read_to_string(src_dir.join("lib.rs")).expect("lib module should read");
    let composition_dir = src_dir.join("composition");
    let widget_dir = src_dir.join("widget");
    let composition_mod = std::fs::read_to_string(composition_dir.join("mod.rs"))
        .expect("composition mod should read");
    let composition_tree = std::fs::read_to_string(composition_dir.join("tree.rs"))
        .expect("composition tree should read");

    assert_source_patterns_absent(
        &composition_dir,
        &[
            "command::Registry".to_owned(),
            "runtime::".to_owned(),
            "platform::".to_owned(),
            "fn mount".to_owned(),
            "fn unmount".to_owned(),
        ],
    );
    for pattern in ["pub mod composition;", "pub use composition::Composition;"] {
        assert!(
            !lib.contains(pattern),
            "retained composition must not be public root API: {pattern}"
        );
    }
    for pattern in [
        "pub use tree::NodeId",
        "pub struct NodeId",
        "pub struct Composition",
        "pub use tree::{Changes",
        "pub use tree::{Node",
        "pub use tree::{Tree",
        "pub struct Changes {",
        "pub struct Tree {",
        "pub struct Node {",
        "pub fn tree(&self)",
    ] {
        assert!(
            !composition_mod.contains(pattern) && !composition_tree.contains(pattern),
            "retained composition tree internals must not be public API: {pattern}"
        );
    }

    let runtime_access = std::fs::read_to_string(src_dir.join("runtime").join("access.rs"))
        .expect("runtime access module should read");
    assert!(
        !runtime_access.contains("pub fn composition("),
        "runtime composition accessor must stay crate/test-visible"
    );

    let interaction_target = std::fs::read_to_string(src_dir.join("interaction").join("target.rs"))
        .expect("interaction target module should read");
    for pattern in [
        "pub fn command_node(",
        "pub fn text_area_node(",
        "pub fn scroll_node(",
        "pub fn scrollbar_node(",
        "pub fn floating_panel_node(",
        "pub fn label_node(",
        "pub fn menu_node(",
        "pub fn node_id(",
    ] {
        assert!(
            !interaction_target.contains(pattern),
            "retained node identity must not leak through public targets: {pattern}"
        );
    }

    let interaction_mod = std::fs::read_to_string(src_dir.join("interaction").join("mod.rs"))
        .expect("interaction mod should read");
    assert!(
        !interaction_mod.contains("pub mod target;"),
        "interaction target file module must stay private; re-export named target concepts instead"
    );

    let session_mod =
        std::fs::read_to_string(src_dir.join("session").join("mod.rs")).expect("session mod read");
    assert!(
        !session_mod.contains("pub mod focus;"),
        "session focus file module must stay private; re-export named focus concepts instead"
    );

    assert_source_patterns_absent(
        &widget_dir,
        &[
            "composition::NodeId".to_owned(),
            "crate::composition".to_owned(),
            "fn mount".to_owned(),
            "fn unmount".to_owned(),
        ],
    );
}

#[test]
fn retained_node_identity_replaces_structural_command_fallbacks() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    assert_source_patterns_absent(
        &src_dir,
        &[
            format!("{}{}", "Command", "Path"),
            format!("{}{}", "command", "_path"),
            format!("{}{}", "path_", "pointer_target"),
            format!("{}{}", "pointer_target_", "at_path"),
            format!("{}{}", "without_", "retained_id"),
        ],
    );
}

#[test]
fn structural_layout_paths_stay_internal() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let layout_mod = std::fs::read_to_string(src_dir.join("layout").join("mod.rs"))
        .expect("layout module should read");
    let frame = std::fs::read_to_string(src_dir.join("layout").join("frame.rs"))
        .expect("layout frame module should read");

    assert!(
        !layout_mod.contains("pub mod path;"),
        "layout structural paths must stay internal to layout/composition ancestry"
    );
    assert!(
        !layout_mod.contains("pub(crate) mod path;"),
        "layout structural path file module must stay private"
    );
    assert!(
        !frame.contains("pub(crate) fn path(&self)") && !frame.contains("pub fn path(&self)"),
        "layout frames must not expose structural paths as crate-wide identity"
    );
}

#[test]
fn layout_frame_and_hit_inspection_stays_internal() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let lib = std::fs::read_to_string(src_dir.join("lib.rs")).expect("crate root should read");
    let layout_dir = src_dir.join("layout");
    let layout_mod =
        std::fs::read_to_string(layout_dir.join("mod.rs")).expect("layout module should read");
    let runtime_presentation =
        std::fs::read_to_string(src_dir.join("runtime").join("presentation.rs"))
            .expect("runtime presentation module should read");

    for pattern in ["pub mod layout;", "pub use layout::Layout;"] {
        assert!(
            !lib.contains(pattern),
            "layout is derived runtime/presentation structure, not public root API: {pattern}"
        );
    }

    for pattern in [
        "pub struct Layout",
        "pub fn size(&self)",
        "pub mod engine;",
        "pub(crate) mod engine;",
        "pub mod frame;",
        "pub(crate) mod frame;",
        "pub mod hit;",
        "pub mod viewport;",
        "pub fn compose(",
        "pub fn compose_with_theme(",
        "pub fn frames(&self)",
        "pub fn viewport(&self)",
        "pub fn resolved_scroll(&self)",
        "pub fn hit_test(&self",
        "pub fn scroll_target_at(",
        "pub fn find_role(&self",
    ] {
        assert!(
            !layout_mod.contains(pattern),
            "layout inspection API must stay internal: {pattern}"
        );
    }

    let internal_layout_sources = ["frame.rs", "hit.rs", "viewport.rs", "text.rs"]
        .into_iter()
        .map(|file| {
            std::fs::read_to_string(layout_dir.join(file))
                .unwrap_or_else(|error| panic!("{file} should read: {error}"))
        })
        .collect::<Vec<_>>()
        .join("\n");

    for pattern in [
        "pub struct Frame",
        "pub struct Hit",
        "pub struct Viewport",
        "pub struct Area",
        "pub struct Field",
        "pub fn frame(&self)",
        "pub fn viewport(&self)",
        "pub fn resolved_scroll(&self)",
        "pub fn action_at(",
        "pub fn drag_action_at_with_engine(",
    ] {
        assert!(
            !internal_layout_sources.contains(pattern),
            "internal layout inspection item must stay crate-visible: {pattern}"
        );
    }

    assert!(
        !runtime_presentation.contains("pub fn hit_test("),
        "runtime hit testing exposes layout hit internals and must stay crate-visible"
    );
}

#[test]
fn scene_layout_painting_stays_internal() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let scene_mod = std::fs::read_to_string(src_dir.join("scene").join("mod.rs"))
        .expect("scene module should read");

    for pattern in [
        "pub fn paint(",
        "pub fn paint_with_theme(",
        "pub fn paint_with_clear(",
        "pub fn paint_with_clear_and_theme(",
    ] {
        assert!(
            !scene_mod.contains(pattern),
            "layout-to-scene painting must stay runtime/internal: {pattern}"
        );
    }
}

#[test]
fn view_tree_inspection_helpers_stay_internal() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let view_mod = std::fs::read_to_string(src_dir.join("view").join("mod.rs"))
        .expect("view module should read");

    for pattern in [
        "pub fn bindings(",
        "pub fn binding<",
        "pub fn text_areas(",
        "pub fn buttons(",
        "pub fn checkboxes(",
        "pub fn radios(",
        "pub fn sliders(",
        "pub fn text_boxes(",
        "pub fn menus(",
        "pub fn labels(",
        "pub fn floating_panels(",
    ] {
        assert!(
            !view_mod.contains(pattern),
            "view tree inspection helpers must stay internal: {pattern}"
        );
    }
}

#[test]
fn focus_traversal_goes_through_retained_composition() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let view_mod = std::fs::read_to_string(src_dir.join("view").join("mod.rs"))
        .expect("view module should read");
    let traversal = std::fs::read_to_string(src_dir.join("view").join("node").join("traversal.rs"))
        .expect("view node traversal module should read");

    for pattern in [
        "pub fn contains_enabled_focus(",
        "pub fn focus_order(",
        "pub fn next_focus(",
    ] {
        assert!(
            !view_mod.contains(pattern),
            "public view focus traversal must not bypass retained composition: {pattern}"
        );
    }

    for pattern in [
        "fn collect_focus_order(&self",
        "fn collect_floating_panel_focus_order(&self",
    ] {
        assert!(
            !traversal.contains(pattern),
            "view node traversal must not keep structural focus-order fallback: {pattern}"
        );
    }
}

#[test]
fn public_target_contract_uses_public_command_values() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let spec = std::fs::read_to_string(src_dir.join("command").join("spec.rs"))
        .expect("command spec module should read");
    let state = std::fs::read_to_string(src_dir.join("command").join("state.rs"))
        .expect("command state module should read");
    let response = std::fs::read_to_string(src_dir.join("response").join("mod.rs"))
        .expect("response module should read");

    for pattern in [
        "pub fn new(display_name: &'static str)",
        "pub fn shortcut(mut self",
        "pub fn key_chord(mut self",
    ] {
        assert!(
            spec.contains(pattern),
            "command registration Spec must remain constructible by app code: {pattern}"
        );
    }

    for pattern in [
        "pub fn enabled()",
        "pub fn disabled()",
        "pub fn hidden()",
        "pub fn checked(mut self",
    ] {
        assert!(
            state.contains(pattern),
            "command State must remain constructible by external Target implementations: {pattern}"
        );
    }

    for pattern in [
        "pub fn output(output: O)",
        "pub fn changed(output: O)",
        "pub fn failed(error: Error)",
        "pub fn into_result(self)",
    ] {
        assert!(
            response.contains(pattern),
            "Response must remain constructible/readable by external Target implementations: {pattern}"
        );
    }
}

fn assert_source_patterns_absent(path: &std::path::Path, patterns: &[String]) {
    for entry in std::fs::read_dir(path).expect("framework source directory should be readable") {
        let path = entry
            .expect("framework source entry should be readable")
            .path();
        if path.is_dir() {
            assert_source_patterns_absent(&path, patterns);
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("framework source file should read");
        for pattern in patterns {
            assert!(
                !source.contains(pattern),
                "{} must not contain stale routing concept {pattern}",
                path.display()
            );
        }
    }
}

fn assert_imports_only_under_any(
    path: &std::path::Path,
    allowed_roots: &[std::path::PathBuf],
    modules: &[&str],
) {
    for entry in std::fs::read_dir(path).expect("framework source directory should be readable") {
        let path = entry
            .expect("framework source entry should be readable")
            .path();
        if path.is_dir() {
            assert_imports_only_under_any(&path, allowed_roots, modules);
            continue;
        }

        if allowed_roots.iter().any(|root| path.starts_with(root))
            || path.extension().and_then(|extension| extension.to_str()) != Some("rs")
        {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("framework source file should read");
        for module in modules {
            assert!(
                !source_imports_crate_module(&source, module),
                "{} must import crate::{} only under one of {:?}",
                path.display(),
                module,
                allowed_roots
            );
        }
    }
}

fn source_imports_crate_module(source: &str, module: &str) -> bool {
    if source.contains(&format!("crate::{module}::")) {
        return true;
    }

    source.lines().any(|line| {
        let line = line.trim();
        if line == format!("use crate::{module};")
            || line.starts_with(&format!("use crate::{module}::"))
        {
            return true;
        }

        let Some(grouped) = line
            .strip_prefix("use crate::{")
            .and_then(|line| line.strip_suffix(';'))
        else {
            return false;
        };

        grouped.split(',').any(|segment| {
            let segment = segment.trim();
            let root = segment
                .split_once("::")
                .map_or(segment, |(root, _)| root.trim())
                .split_once(" as ")
                .map_or_else(|| segment, |(root, _)| root.trim());
            root == module
        })
    })
}
