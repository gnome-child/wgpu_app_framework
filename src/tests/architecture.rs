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
        src_dir.join("text"),
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
        !lib.contains(&format!("pub mod {};", old_paint_space_module())),
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
fn paint_stays_below_text_and_native_rendering() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let allowed_roots = [
        src_dir.join("paint"),
        src_dir.join("paint"),
        src_dir.join("render"),
        src_dir.join("platform").join("native"),
        src_dir.join("text"),
    ];

    assert_imports_only_under_any(&src_dir, &allowed_roots, &["paint"]);
}

#[test]
fn paint_file_modules_stay_private() {
    let paint_mod = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("paint")
            .join("mod.rs"),
    )
    .expect("paint geometry module should read");

    for module in ["area", "point"] {
        let pattern = format!("pub(crate) mod {module};");
        assert!(
            paint_mod.contains(&pattern),
            "unit-distinguished paint module should be crate-visible: {pattern}"
        );
        assert!(
            !paint_mod.contains(&format!("pub mod {module};")),
            "paint file module should not become public API: pub mod {module};"
        );
    }

    for module in ["grid", "rect"] {
        for visibility in ["pub mod", "pub(crate) mod"] {
            let pattern = format!("{visibility} {module};");
            assert!(
                !paint_mod.contains(&pattern),
                "single-concept paint module should stay behind root re-exports: {pattern}"
            );
        }
    }

    for alias in [
        "Area",
        "Point",
        "LogicalArea",
        "PhysicalArea",
        "LogicalPoint",
    ] {
        assert!(
            !paint_mod.contains(&format!("use area::{{{alias}"))
                && !paint_mod.contains(&format!("use point::{{{alias}"))
                && !paint_mod.contains(&format!(" as {alias}")),
            "paint should not reintroduce compound or unit-erasing root alias {alias}"
        );
    }
}

#[test]
fn old_paint_space_root_module_is_extinct() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let lib = std::fs::read_to_string(src_dir.join("lib.rs")).expect("crate root should read");
    let old_module = old_paint_space_module();

    assert!(
        !src_dir.join(old_module).exists(),
        "old paint-space root module should be folded into paint"
    );
    assert!(
        !lib.contains(&format!("mod {old_module};")),
        "crate root should not keep the old paint-space module"
    );
    assert_source_patterns_absent(
        &src_dir,
        &[format!("crate::{old_module}"), format!("{old_module}::")],
    );
}

fn old_paint_space_module() -> &'static str {
    concat!("paint_", "geometry")
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
    let lib = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("lib.rs"),
    )
    .expect("crate root should read");
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
    assert!(
        !lib.contains("pub mod draft;"),
        "draft is transient text-session state, not public root API"
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
fn view_style_module_stays_private() {
    let view_mod = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("view")
            .join("mod.rs"),
    )
    .expect("view module should read");

    for pattern in ["pub mod style;", "pub(crate) mod style;"] {
        assert!(
            !view_mod.contains(pattern),
            "view style file module should stay behind the facade: {pattern}"
        );
    }
}

#[test]
fn view_node_module_stays_private() {
    let view_mod = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("view")
            .join("mod.rs"),
    )
    .expect("view module should read");

    for pattern in ["pub mod node;", "pub(crate) mod node;"] {
        assert!(
            !view_mod.contains(pattern),
            "view node file module should stay behind the facade: {pattern}"
        );
    }
}

#[test]
fn view_control_module_stays_private() {
    let view_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("view");
    let view_mod =
        std::fs::read_to_string(view_dir.join("mod.rs")).expect("view module should read");
    let control_mod = std::fs::read_to_string(view_dir.join("control").join("mod.rs"))
        .expect("view control module should read");

    for pattern in ["pub mod control;", "pub(crate) mod control;"] {
        assert!(
            !view_mod.contains(pattern),
            "view control file module should stay behind the facade: {pattern}"
        );
    }

    assert!(
        !view_mod.contains("Control,"),
        "view Control enum is node storage; expose concrete control concepts instead"
    );
    assert!(
        !control_mod.contains("pub enum Control"),
        "view Control enum should not be public API"
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
fn fuzzy_matching_stays_with_palette_runtime() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let lib = std::fs::read_to_string(src_dir.join("lib.rs")).expect("crate root should read");

    assert!(
        !src_dir.join("fuzzy.rs").exists(),
        "fuzzy search is command-palette runtime support, not a root framework concept"
    );
    assert!(
        !lib.contains("mod fuzzy;") && !lib.contains("pub mod fuzzy;"),
        "fuzzy search should stay below runtime/palette ownership"
    );
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
    assert!(
        !interaction_mod.contains("pub use command_palette::CommandPalette;"),
        "command palette state is internal interaction/session state, not public interaction API"
    );
    assert!(
        !lib.contains("pub use interaction::Interaction;"),
        "interaction state storage is runtime/session state, not public root API"
    );
    assert!(
        !interaction_mod.contains("pub struct Interaction"),
        "interaction state storage should stay crate-internal"
    );
    assert!(
        !interaction_mod.contains("pub use pointer::{Capture, Pointer")
            && !interaction_mod.contains("pub use scroll::Scroll")
            && !interaction_mod.contains("pub use scroll::{Scroll,"),
        "interaction pointer/scroll storage should not be public API"
    );

    let session_window = std::fs::read_to_string(src_dir.join("session").join("window.rs"))
        .expect("session window should read");
    let session_interaction =
        std::fs::read_to_string(src_dir.join("session").join("interaction").join("mod.rs"))
            .expect("session interaction should read");
    assert!(
        !session_window.contains("pub fn interaction(&self)")
            && !session_interaction.contains("pub fn interaction(&self"),
        "session interaction accessors should stay crate-internal"
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
fn press_intent_stays_runtime_interaction_detail() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let interaction_mod = std::fs::read_to_string(src_dir.join("interaction").join("mod.rs"))
        .expect("interaction module should read");
    let pointer = std::fs::read_to_string(src_dir.join("interaction").join("pointer.rs"))
        .expect("interaction pointer should read");
    let input_mod =
        std::fs::read_to_string(src_dir.join("input").join("mod.rs")).expect("input module read");

    assert!(
        !interaction_mod.contains("pub use pointer::PressIntent"),
        "press intent is runtime/session press classification, not public interaction API"
    );
    assert!(
        !pointer.contains("pub enum PressIntent"),
        "press intent should stay crate-internal"
    );
    for pattern in [
        "pointer_down_with_intent",
        "intent: interaction::PressIntent",
        "PointerDown {",
    ] {
        assert!(
            !input_mod.contains(pattern),
            "public input should expose named pointer gestures, not internal press intent: {pattern}"
        );
    }
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
fn layout_reveal_stays_palette_agnostic() {
    let layout_mod = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("layout")
        .join("mod.rs");
    let source = std::fs::read_to_string(layout_mod).expect("layout module should read");

    assert!(
        !source.contains("Source::Palette"),
        "generic viewport reveal must not hardcode command-palette descendants"
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
        "pub mod chrome;",
        "pub(crate) mod chrome;",
        "pub mod control;",
        "pub(crate) mod control;",
        "pub mod engine;",
        "pub(crate) mod engine;",
        "pub mod flow;",
        "pub(crate) mod flow;",
        "pub mod frame;",
        "pub(crate) mod frame;",
        "pub mod hit;",
        "pub(crate) mod hit;",
        "pub mod text;",
        "pub(crate) mod text;",
        "pub mod typography;",
        "pub(crate) mod typography;",
        "pub mod viewport;",
        "pub(crate) mod viewport;",
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
    let scene_visual = std::fs::read_to_string(src_dir.join("scene").join("visual.rs"))
        .expect("scene visuals should read");

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
    assert!(
        !scene_mod.contains("pub use visual::Visuals")
            && !scene_visual.contains("pub struct Visuals"),
        "scene Visuals are runtime-derived paint input, not public scene API"
    );
}

#[test]
fn view_tree_inspection_helpers_stay_internal() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let view_mod = std::fs::read_to_string(src_dir.join("view").join("mod.rs"))
        .expect("view module should read");
    let view_presentation = std::fs::read_to_string(src_dir.join("view").join("presentation.rs"))
        .expect("view presentation should read");
    let view_access = std::fs::read_to_string(src_dir.join("view").join("node").join("access.rs"))
        .expect("view node access should read");
    let runtime_presentation =
        std::fs::read_to_string(src_dir.join("runtime").join("presentation.rs"))
            .expect("runtime presentation should read");

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
    assert!(
        !view_mod.contains("pub use presentation::Presentation;")
            && !view_presentation.contains("pub struct Presentation"),
        "view Presentation is an internal runtime checkpoint, not public view API"
    );
    assert!(
        !view_mod.contains("Node, Role") && !view_mod.contains("pub use node::Role"),
        "view Role is node storage vocabulary, not public view API"
    );
    assert!(
        !view_mod.contains("pub use action::Action"),
        "view Action is runtime routing vocabulary, not public view API"
    );
    for pattern in [
        "pub fn is_hovered(&self)",
        "pub fn is_pressed(&self)",
        "pub fn is_active(&self)",
    ] {
        assert!(
            !view_access.contains(pattern),
            "paint-only interaction state must not be public view-node inspection API: {pattern}"
        );
    }
    for pattern in [
        "pub fn drain(&mut self)",
        "pub fn drain_scenes(",
        "pub fn present(&mut self",
        "pub fn present_pending(",
    ] {
        assert!(
            !runtime_presentation.contains(pattern),
            "runtime pre-render presentation method should stay crate-internal: {pattern}"
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

#[test]
fn state_change_reasons_do_not_import_command_contracts() {
    let state_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("state");

    assert_source_patterns_absent(&state_dir, &["crate::command".to_owned()]);
}

#[test]
fn resting_geometry_snapping_has_no_primitive_mode_axis() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let scene_primitive = std::fs::read_to_string(src_dir.join("scene").join("primitive.rs"))
        .expect("scene primitive module should read");
    let scene_mod = std::fs::read_to_string(src_dir.join("scene").join("mod.rs"))
        .expect("scene mod should read");
    let paint_mod = std::fs::read_to_string(src_dir.join("paint").join("mod.rs"))
        .expect("paint mod should read");

    for source in [&scene_primitive, &scene_mod, &paint_mod] {
        for pattern in [
            "enum Snapping",
            "Snapping::",
            "pub use primitive::{ Snapping",
        ] {
            assert!(
                !source.contains(pattern),
                "quad snapping must be derived from motion/resting geometry, not a primitive mode: {pattern}"
            );
        }
    }
}

#[test]
fn text_origin_snapping_belongs_to_paint_grid() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let text_renderer = std::fs::read_to_string(src_dir.join("render").join("text_renderer.rs"))
        .expect("text renderer should read");
    let paint_grid = std::fs::read_to_string(src_dir.join("paint").join("grid.rs"))
        .expect("paint grid should read");

    assert!(
        !text_renderer.contains("fn snap_text_origin"),
        "text renderer must not keep a second text-origin snapper"
    );
    for pattern in ["fn snap_text_origin", "fn snap_centered_text_origin"] {
        assert!(
            paint_grid.contains(pattern),
            "paint Grid must own text-origin snapping helper: {pattern}"
        );
    }
}

#[test]
fn glyphon_viewports_are_owned_per_text_batch() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let text_renderer = std::fs::read_to_string(src_dir.join("render").join("text_renderer.rs"))
        .expect("text renderer should read");
    let render_start = text_renderer
        .find("fn render(")
        .expect("text renderer should expose render method");
    let trim_start = text_renderer[render_start..]
        .find("fn trim(")
        .map(|offset| render_start + offset)
        .expect("text renderer render method should be followed by trim");
    let render_body = &text_renderer[render_start..trim_start];

    assert!(
        text_renderer.contains("viewports: Vec<glyphon::Viewport>"),
        "glyphon viewport state must be parallel to per-batch text renderers"
    );
    assert!(
        !text_renderer.contains("viewport: glyphon::Viewport"),
        "text renderer must not keep one shared glyphon viewport uniform"
    );
    assert!(
        text_renderer.contains("self.update_viewport(render_context, renderer_index, viewport)"),
        "viewport writes should happen while preparing the owning text batch"
    );
    assert!(
        !render_body.contains("update_viewport"),
        "render must consume the prepared batch viewport, not write shared viewport state"
    );
}

#[test]
fn master_design_names_answer_patterns() {
    let master = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs")
            .join("master_design.md"),
    )
    .expect("master design should read");

    assert!(
        master.contains("## Answer Catalog"),
        "master design must name answer-patterns, not only smells"
    );
    for pattern in [
        "One Truth, One Owner",
        "Witness Demotion",
        "Axis Splitting",
        "Structural Absence",
        "Exceptions Become Citizens",
        "Endpoints Are Truth",
        "Findings Graduate Into Invariants",
    ] {
        assert!(
            master.contains(pattern),
            "master design Answer Catalog must include {pattern}"
        );
    }
}

#[test]
fn glass_material_carrier_is_pane_not_surface() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let scene_primitive = std::fs::read_to_string(root.join("scene").join("primitive.rs"))
        .expect("scene primitive source should read");
    let paint = std::fs::read_to_string(root.join("paint").join("mod.rs"))
        .expect("paint source should read");
    let master = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs")
            .join("master_design.md"),
    )
    .expect("master design should read");

    assert!(scene_primitive.contains("pub struct Pane"));
    assert!(paint.contains("pub struct Pane"));
    assert!(
        !scene_primitive.contains("MaterialSurface") && !paint.contains("MaterialSurface"),
        "material carrier must not reintroduce a compound Surface name"
    );
    assert!(
        !scene_primitive.contains("pub struct Surface") && !paint.contains("pub struct Surface"),
        "Pane, not Surface, names shaped material"
    );
    assert!(master.contains("A material is a visual recipe; a pane is shaped material."));
}

#[test]
fn scene_no_longer_exposes_generic_filter_primitives() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let scene_mod =
        std::fs::read_to_string(root.join("scene").join("mod.rs")).expect("scene mod should read");
    let scene_primitive = std::fs::read_to_string(root.join("scene").join("primitive.rs"))
        .expect("scene primitive source should read");
    let native_paint =
        std::fs::read_to_string(root.join("platform").join("native").join("paint.rs"))
            .expect("native paint source should read");

    for source in [&scene_mod, &scene_primitive, &native_paint] {
        assert!(
            !source.contains("Primitive::Filter"),
            "scene-level filter primitive must not return after Pane"
        );
        assert!(
            !source.contains("scene::Filter"),
            "native paint must not carry a scene filter bridge after Pane"
        );
        assert!(
            !source.contains("FilterOp"),
            "scene-level filter ops must not return after Pane"
        );
    }
}

#[test]
fn paint_display_list_no_longer_routes_generic_filter_items() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let paint = std::fs::read_to_string(root.join("paint").join("mod.rs"))
        .expect("paint source should read");
    let batch = std::fs::read_to_string(root.join("render").join("batch.rs"))
        .expect("render batch source should read");
    let renderer = std::fs::read_to_string(root.join("render").join("renderer.rs"))
        .expect("renderer source should read");

    assert!(
        !paint.contains("Item::Filter"),
        "paint display list should not route generic filters after Pane"
    );
    assert!(
        !paint.contains("filter_op_outset"),
        "pane material bounds should not rewrap material layers as generic filter ops"
    );
    assert!(
        !paint.contains("LiquidFilter") && !paint.contains("FilterOp::Liquid"),
        "old generic liquid filter op should not return after Pane"
    );
    assert!(
        !batch.contains("ItemBatch::Filter"),
        "render batching should not carry generic filter batches after Pane"
    );
    assert!(
        !renderer.contains("RenderBatch::Filter"),
        "renderer should not dispatch generic filter batches after Pane"
    );
    assert!(
        !renderer.contains("fn encode_filter"),
        "filter encoding should be owned by Pane/material paths after Pane"
    );
    assert!(
        !renderer.contains("filter_source_decision"),
        "old display-list filter source decision helper should not return"
    );
}

#[test]
fn compositor_diagnostics_are_documented_debug_targets() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = root.join("src");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");
    let filter = [
        std::fs::read_to_string(source_root.join("render").join("filter").join("encode.rs"))
            .expect("filter encoder source should read"),
        std::fs::read_to_string(source_root.join("render").join("filter.rs"))
            .expect("filter renderer source should read"),
    ]
    .join("\n");
    let renderer = std::fs::read_to_string(source_root.join("render").join("renderer.rs"))
        .expect("renderer source should read");
    let presentation = std::fs::read_to_string(source_root.join("runtime").join("presentation.rs"))
        .expect("presentation source should read");
    let overlay = std::fs::read_to_string(source_root.join("overlay.rs"))
        .expect("overlay source should read");
    let native_popup =
        std::fs::read_to_string(source_root.join("platform").join("native").join("popup.rs"))
            .expect("native popup source should read");

    for target in [
        "wgpu_l3::render::filter_params",
        "wgpu_l3::render::material",
        "wgpu_l3::overlay::fade",
        "wgpu_l3::overlay::backend",
        "wgpu_l3::native_popup",
    ] {
        assert!(
            master.contains(target),
            "diagnostic target {target} must be documented"
        );
    }

    assert_debug_log_target(&filter, "wgpu_l3::render::filter_params");
    assert_debug_log_target(&renderer, "wgpu_l3::render::material");
    assert_debug_log_target(&presentation, "wgpu_l3::overlay::fade");
    assert_debug_log_target(&overlay, "wgpu_l3::overlay::backend");
    assert!(
        native_popup.contains("wgpu_l3::native_popup"),
        "native popup diagnostics must have an implementation site"
    );
}

#[test]
fn overlay_backend_selection_is_not_a_paint_id_exception() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scene_paint =
        std::fs::read_to_string(root.join("src").join("scene").join("paint").join("mod.rs"))
            .expect("scene paint source should read");
    let command_palette =
        std::fs::read_to_string(root.join("src").join("view").join("command_palette.rs"))
            .expect("command palette source should read");
    let overlay = std::fs::read_to_string(root.join("src").join("overlay.rs"))
        .expect("overlay source should read");

    assert!(
        !scene_paint.contains("CommandPalette::panel_id"),
        "command palette backend choice must not be a paint-layer id exception"
    );
    assert!(
        !scene_paint.contains("material_realization(")
            && !command_palette.contains("with_overlay_realization"),
        "material realization must not veto the shared floating-panel backend path"
    );
    assert!(
        !overlay.contains("RequiresParentCompositionBackdrop")
            && !overlay.contains("native_backdrop_materials_supported"),
        "overlay backend resolution should depend on popup capability, not parent-composition backdrop requirements"
    );
}

#[test]
fn native_popup_positioning_anchors_to_parent_client_origin() {
    let popup = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("platform")
            .join("native")
            .join("popup.rs"),
    )
    .expect("native popup source should read");

    assert!(
        popup.contains(".inner_position()"),
        "native popup position must anchor to the parent client-area origin"
    );
    assert!(
        popup.contains("falling back to outer origin"),
        "outer-position fallback must stay explicit and logged"
    );
}

#[test]
fn windows_native_popup_clicks_do_not_activate() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest =
        std::fs::read_to_string(root.join("Cargo.toml")).expect("cargo manifest should read");
    let sys_mod = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("sys")
            .join("mod.rs"),
    )
    .expect("native sys module should read");
    let windows = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("sys")
            .join("windows.rs"),
    )
    .expect("windows native sys source should read");
    let native_window = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("window.rs"),
    )
    .expect("native window source should read");
    let native_mod = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("mod.rs"),
    )
    .expect("native module source should read");
    let popup = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("popup.rs"),
    )
    .expect("native popup source should read");
    let adapter = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("adapter.rs"),
    )
    .expect("native adapter source should read");

    assert!(
        manifest.contains("\"Win32_UI_Shell\""),
        "popup subclass APIs must be enabled through the Windows Shell bindings"
    );
    assert!(
        manifest.contains("\"Win32_Graphics_Dwm\""),
        "popup dark-mode DWM sync must stay behind the Windows bindings"
    );
    assert!(
        manifest.contains("\"Win32_System_LibraryLoader\""),
        "undocumented accent policy must be late-bound through user32 lookup"
    );
    assert!(windows.contains("SetWindowSubclass"));
    assert!(windows.contains("DefSubclassProc"));
    assert!(windows.contains("RemoveWindowSubclass"));
    assert!(windows.contains("DWMWA_USE_IMMERSIVE_DARK_MODE"));
    assert!(windows.contains("SetWindowCompositionAttribute"));
    assert!(windows.contains("ACCENT_ENABLE_ACRYLICBLURBEHIND"));
    assert!(windows.contains("WCA_ACCENT_POLICY"));
    assert!(sys_mod.contains("accent_gradient_abgr"));
    assert!(windows.contains("WM_MOUSEACTIVATE"));
    assert!(
        windows.contains("return MA_NOACTIVATE as LRESULT"),
        "mouse activation must be answered without activating the popup"
    );
    assert!(
        !windows.contains("MA_NOACTIVATEANDEAT"),
        "native menus must receive the click that was prevented from activating"
    );
    assert!(windows.contains("WS_EX_NOACTIVATE"));
    assert!(windows.contains("WS_EX_TOOLWINDOW"));
    assert!(windows.contains("WS_EX_APPWINDOW"));
    assert!(windows.contains("GWL_STYLE"));
    assert!(windows.contains("SetWindowLongPtrW(hwnd, GWL_STYLE"));
    assert!(windows.contains("WS_POPUP"));
    for style in [
        "WS_CAPTION",
        "WS_SYSMENU",
        "WS_THICKFRAME",
        "WS_MINIMIZEBOX",
        "WS_MAXIMIZEBOX",
        "WS_BORDER",
        "WS_DLGFRAME",
    ] {
        assert!(
            windows.contains(style),
            "popup chrome/control style {style} must be explicitly cleared"
        );
    }
    assert!(windows.contains("SWP_FRAMECHANGED"));
    assert!(windows.contains("SWP_NOACTIVATE"));
    assert!(
        sys_mod.contains("install_popup_subclass") && sys_mod.contains("remove_popup_subclass"),
        "subclass lifecycle must stay behind the native sys seam"
    );
    assert!(
        native_window.contains("install_popup_subclass"),
        "popup creation must install the mouse-activation interceptor"
    );
    assert!(
        !native_window.contains("BackdropType::TransientWindow")
            && !native_window.contains("with_system_backdrop"),
        "nonactivating native popups must not use focus-coupled DWM system backdrop"
    );
    assert!(native_window.contains("CornerPreference::Round"));
    assert!(native_window.contains("with_no_redirection_bitmap(mode.no_redirection_bitmap())"));
    assert!(native_window.contains("with_undecorated_shadow(true)"));
    assert!(native_window.contains("with_has_shadow(true)"));
    assert!(!native_window.contains("with_no_redirection_bitmap(true)"));
    assert!(!native_window.contains("with_no_redirection_bitmap(false)"));
    assert!(!native_window.contains("with_undecorated_shadow(false)"));
    assert!(!native_window.contains("with_has_shadow(false)"));
    assert!(
        native_mod.contains("impl Drop for PopupWindow")
            && native_mod.contains("remove_popup_subclass"),
        "popup drop must remove the subclass before the HWND is released"
    );
    assert!(
        popup.contains("self.popups.remove(&key)") && adapter.contains("self.popups.remove(&key)"),
        "stale popup and parent-close cleanup must drive PopupWindow drop"
    );
}

#[test]
fn windows_native_popup_material_keeps_dx12_visual_available_without_forcing_it() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let context = std::fs::read_to_string(root.join("src").join("render").join("context.rs"))
        .expect("render context source should read");
    let surface = std::fs::read_to_string(root.join("src").join("render").join("surface.rs"))
        .expect("render surface source should read");
    let native_surface = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("surface.rs"),
    )
    .expect("native surface source should read");
    let native_mod = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("mod.rs"),
    )
    .expect("native module source should read");
    let native_window = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("window.rs"),
    )
    .expect("native window source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    assert!(
        context.contains("Dx12SwapchainKind::DxgiFromVisual"),
        "Windows should keep the DirectComposition Visual DX12 presentation path available"
    );
    assert!(
        context.contains("options.backends.with_env()")
            && native_surface.contains("default_native_backends()")
            && native_surface.contains("wgpu::Backends::all()"),
        "Windows should use normal backend selection while remaining overridable through WGPU_BACKEND"
    );
    assert!(
        !native_surface.contains("wgpu::Backends::DX12"),
        "Windows native popup acrylic must not require a hardcoded DX12 backend"
    );
    assert!(
        surface.contains("popup surface capabilities"),
        "native popup surface format and alpha capabilities must be logged"
    );
    assert!(
        surface.contains("supported={supported:?}"),
        "opaque popup fallback should report supported alpha modes"
    );
    assert!(
        native_mod.contains("PopupPresentationMode")
            && native_mod.contains("CompositionBacked")
            && native_mod.contains("RedirectedFallback"),
        "native popup presentation mode must be explicit"
    );
    assert!(
        native_window.contains("with_no_redirection_bitmap(mode.no_redirection_bitmap())"),
        "Windows native popups must pair no-redirection with the selected presentation mode"
    );
    assert!(
        native_mod.contains(
            "Self::RedirectedFallback => render::CompositeAlphaPreference::PreMultiplied"
        ) && native_mod.contains("PopupPresentationMode::RedirectedFallback.realization_for")
            && native_mod.contains("PopupMaterialRealization::WindowsAccentAcrylic"),
        "redirected Vulkan popups may realize OS material when the surface reports premultiplied alpha"
    );
    for phrase in [
        "WGPU_BACKEND",
        "system backdrop tracks activation state",
        "SetWindowCompositionAttribute",
        "Vulkan redirected popups",
        "DxgiFromVisual",
    ] {
        assert!(
            master.contains(phrase),
            "Windows native material diagnostic doctrine should mention {phrase}"
        );
    }
}

#[test]
fn native_popup_accent_realization_is_settle_rate() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let popup = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("popup.rs"),
    )
    .expect("native popup source should read");
    let native_mod = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("mod.rs"),
    )
    .expect("native module source should read");
    let adapter = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("adapter.rs"),
    )
    .expect("native adapter source should read");
    let platform = std::fs::read_to_string(root.join("src").join("platform").join("mod.rs"))
        .expect("platform source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    assert!(native_mod.contains("PopupAccentState"));
    assert!(native_mod.contains("POPUP_ACCENT_SETTLE_DELAY"));
    assert!(native_mod.contains("Duration::from_millis(150)"));
    assert!(popup.contains("popup.accent.set_desired(accent, now)"));
    assert!(popup.contains("apply_due_popup_accents"));
    assert_eq!(
        popup.matches("set_popup_accent_material(accent)").count(),
        1,
        "popup presentation must not call the Windows accent API at tint-sample rate"
    );
    assert!(
        adapter.contains("apply_due_popup_accents(std::time::Instant::now())")
            && platform.contains("self.backend.maintain(context)?"),
        "pending native accent state must drain from backend maintenance, not only popup presentations"
    );
    assert!(
        master.contains("OS-side realizations are settle-rate, not event-rate"),
        "native material doctrine should name settle-rate OS realization"
    );
}

#[test]
fn native_popup_presentations_defer_overlay_fade_until_premultiplied_audit() {
    let presentation = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("runtime")
            .join("presentation.rs"),
    )
    .expect("runtime presentation source should read");

    for call in [
        "append_scene_with_opacity(local.native_material(), 1.0)",
        "append_scene_with_opacity(local.opaque_fallback(), 1.0)",
    ] {
        assert!(
            presentation.contains(call),
            "native popup presentation should render full opacity until the premultiplied audit: {call}"
        );
    }
    assert!(
        !presentation
            .contains("append_scene_with_opacity(local.native_material(), layer.opacity())"),
        "native popup material must not apply semi-transparent overlay fade yet"
    );
}

#[test]
fn glass_tuner_foreground_fixture_compares_backed_and_unbacked_same_content() {
    let view = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("glass_tuner")
            .join("app")
            .join("view.rs"),
    )
    .expect("glass tuner view source should read");

    for phrase in [
        "Backed: in-frame surface reference",
        "Unbacked: native material boundary",
        "foreground_sample(Some(PANEL_SURFACE_COLOR), state)",
        "foreground_sample(None, state)",
        "foreground_sample_content(ui, tint_opacity, noise_opacity)",
        "Binding::<ForegroundEnabledItem>::menu()",
        "Binding::<ForegroundDisabledItem>::menu()",
        "Slider::new(\"Tint opacity\"",
        "Slider::new(\"Noise opacity\"",
        "Half-alpha quads",
    ] {
        assert!(
            view.contains(phrase),
            "foreground clarity fixture should contain {phrase}"
        );
    }

    assert_eq!(
        view.matches("foreground_sample_content(ui, tint_opacity, noise_opacity)")
            .count(),
        1,
        "backed and unbacked rows must share the same content helper"
    );
}

#[test]
fn premultiplied_popup_surfaces_pack_without_legacy_final_blit() {
    let renderer = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("render")
            .join("renderer.rs"),
    )
    .expect("renderer source should read");

    assert!(
        renderer.contains("preserve_surface_alpha")
            && renderer.contains(
                "canvas.composite_alpha_mode() == wgpu::CompositeAlphaMode::PreMultiplied"
            ),
        "renderer must explicitly detect premultiplied surfaces"
    );
    assert!(
        renderer.contains("pack_premultiplied_surface")
            && renderer.contains("supports_windows_premultiplied_popup_pack")
            && renderer.contains("popup_packer.pack_to_view"),
        "premultiplied non-sRGB popup surfaces should render through the Windows pack pass"
    );
    assert!(
        renderer.contains("filter_renderer.blit_to_view")
            && renderer.contains("} else {\n            canvas.draw"),
        "opaque/default surfaces should keep the composition texture plus final blit path"
    );
}

#[test]
fn popup_pack_shader_uses_exact_srgb_piecewise_transfer() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("render")
            .join("popup_pack.wgsl"),
    )
    .expect("popup pack shader should read");

    for phrase in [
        "0.0031308",
        "12.92 * v",
        "1.055 * pow(v, 1.0 / 2.4) - 0.055",
        "srgb_encode(straight) * alpha",
    ] {
        assert!(
            source.contains(phrase),
            "popup pack shader must contain exact sRGB packing phrase {phrase}"
        );
    }
    assert!(
        !source.contains("2.2"),
        "popup pack shader must not approximate sRGB with gamma 2.2"
    );
}

#[test]
fn native_popup_foreground_fix_is_packing_not_coverage_compensation() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let renderer = std::fs::read_to_string(root.join("src").join("render").join("renderer.rs"))
        .expect("renderer source should read");
    let popup_pack =
        std::fs::read_to_string(root.join("src").join("render").join("popup_pack.wgsl"))
            .expect("popup pack shader should read");

    assert!(renderer.contains("PackedPremultipliedSrgbForWindows"));
    assert!(
        !renderer.contains("coverage_compensation") && !popup_pack.contains("coverage"),
        "native popup clarity fix must not use a coverage-compensation approximation"
    );
}

#[test]
fn native_renderer_cache_is_keyed_by_render_target_format() {
    let native_mod = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("platform")
            .join("native")
            .join("mod.rs"),
    )
    .expect("native mod source should read");
    let surface = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("platform")
            .join("native")
            .join("surface.rs"),
    )
    .expect("native surface source should read");

    assert!(native_mod.contains("renderers: HashMap<wgpu::TextureFormat, render::Renderer>"));
    assert!(surface.contains("render_format_for_canvas"));
    assert!(surface.contains("render::scene_format_for_surface_format(format)"));
}

#[test]
fn native_alpha_readback_uses_clean_premultiplied_primitive_witness() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let renderer = std::fs::read_to_string(root.join("src").join("render").join("renderer.rs"))
        .expect("renderer source should read");

    for phrase in [
        "direct_premultiplied_alpha_witness_preserves_alpha_and_rgb",
        "scene.clear(paint::Color::rgba(0.0, 0.0, 0.0, 0.0))",
        "paint::Color::rgba(\n                    1.0, 0.0, 0.0, 0.5,",
        "copy_texture_to_buffer",
        "(sample[3] - 0.5).abs()",
        "(sample[0] - 0.5).abs()",
    ] {
        assert!(
            renderer.contains(phrase),
            "native alpha readback diagnostic must include {phrase}"
        );
    }
}

#[test]
fn native_alpha_probe_exposes_backend_and_attribute_bisection_knobs() {
    let probe = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("native_alpha_probe")
            .join("main.rs"),
    )
    .expect("native alpha probe source should read");

    for phrase in [
        "Dx12Visual",
        "Vulkan",
        "KeyCode::KeyV | KeyCode::KeyD",
        "KeyCode::KeyC",
        "AccentChoice",
        "ACCENT_ENABLE_ACRYLICBLURBEHIND",
        "SetWindowCompositionAttribute",
        "with_no_redirection_bitmap(config.no_redirection_bitmap)",
        "with_owner_window",
        "WS_EX_NOACTIVATE",
        "WS_EX_TOOLWINDOW",
        "WS_POPUP",
        "with_system_backdrop",
        "with_corner_preference",
        "with_undecorated_shadow",
        "owner+toolwindow",
        "nrb+backdrop",
        "wgpu_l3::native_alpha_probe",
        "using_resolution(adapter_limits.clone())",
        "clamp_surface_size",
        "max_texture_dimension_2d",
    ] {
        assert!(
            probe.contains(phrase),
            "native alpha probe must expose/log {phrase}"
        );
    }

    assert!(
        !probe.contains("downlevel_defaults()"),
        "native alpha probe must not request downlevel limits; tiling WMs can resize diagnostics past 2048px"
    );
}

#[test]
fn native_popup_alpha_doctrine_rejects_contaminated_witnesses() {
    let master = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs")
            .join("master_design.md"),
    )
    .expect("master design should read");

    for phrase in [
        "standalone primitive over a",
        "transparent clear",
        "readback that proves both alpha and premultiplied RGB",
        "clear-only witnesses",
        "nested inside panel body content",
        "native_alpha_probe",
    ] {
        assert!(
            master.contains(phrase),
            "native alpha doctrine must mention {phrase}"
        );
    }
}

#[test]
fn filter_texture_pools_are_capped_and_reported() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let filter = [
        std::fs::read_to_string(root.join("render").join("filter.rs"))
            .expect("filter renderer source should read"),
        std::fs::read_to_string(root.join("render").join("filter").join("resources.rs"))
            .expect("filter resources source should read"),
    ]
    .join("\n");
    let renderer = std::fs::read_to_string(root.join("render").join("renderer.rs"))
        .expect("renderer source should read");
    let diagnostics = std::fs::read_to_string(root.join("diagnostics").join("render.rs"))
        .expect("render diagnostics source should read");

    for (constant, entries, field) in [
        (
            "LAYER_POOL_LIMIT",
            "layer_pool_entries()",
            "filter_layer_pool_entries",
        ),
        (
            "SCRATCH_POOL_LIMIT",
            "scratch_pool_entries()",
            "filter_scratch_pool_entries",
        ),
    ] {
        assert!(
            filter.contains(&format!("const {constant}: usize = 8;")),
            "filter texture pool {constant} must keep an explicit retention cap"
        );
        assert!(
            filter.contains(&format!("pool.len() == {constant}")),
            "filter texture pool {constant} must drop entries at its cap"
        );
        assert!(
            renderer.contains(entries),
            "renderer stats must read {entries} for filter texture pool diagnostics"
        );
        assert!(
            diagnostics.contains(field),
            "render diagnostics must expose {field}"
        );
    }
}

#[test]
fn suboptimal_surface_reconfiguration_waits_for_present() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let surface = std::fs::read_to_string(root.join("src").join("render").join("surface.rs"))
        .expect("render surface source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    let suboptimal_start = surface
        .find("Suboptimal(surface_texture)")
        .expect("surface acquire should handle suboptimal textures");
    let outdated_start = surface[suboptimal_start..]
        .find("Outdated =>")
        .map(|offset| suboptimal_start + offset)
        .expect("outdated acquire branch should follow suboptimal");
    let suboptimal = &surface[suboptimal_start..outdated_start];
    assert!(
        suboptimal.contains("self.reconfigure_after_present = true")
            && !suboptimal.contains("self.reconfigure(render_context)"),
        "a live suboptimal SurfaceTexture must defer surface reconfiguration"
    );

    let present = surface
        .find("frame.present();")
        .expect("surface render should present an acquired frame");
    let deferred_reconfigure = surface[present..]
        .find("if self.reconfigure_after_present")
        .map(|offset| present + offset)
        .expect("surface render should apply deferred reconfiguration");
    assert!(
        present < deferred_reconfigure,
        "surface reconfiguration must occur only after presentation releases the texture"
    );
    assert!(
        master.contains("Render `Surface` owns surface configuration epochs"),
        "master design must name the owner of surface reconfiguration timing"
    );
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

fn assert_debug_log_target(source: &str, target: &str) {
    let source = source
        .split("#[cfg(test)]\nmod tests")
        .next()
        .unwrap_or(source);
    let mut found = false;
    for (index, _) in source.match_indices(target) {
        found = true;
        let start = index.saturating_sub(200);
        let context = &source[start..index];
        assert!(
            context.contains("log::debug!"),
            "diagnostic target {target} must be emitted through log::debug!"
        );
    }
    assert!(
        found,
        "diagnostic target {target} must have an implementation site"
    );
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
