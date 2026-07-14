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
fn overlay_capabilities_are_one_valid_realization_species() {
    let overlay = include_str!("../overlay.rs");

    assert!(
        overlay.contains("pub(crate) enum Capabilities {")
            && overlay.contains("InFrameOnly,")
            && overlay.contains("AnimatedNativePopups,")
            && overlay.contains("ImmediateNativePopups,")
    );
    assert!(!overlay.contains("native_popups: bool"));
    assert!(!overlay.contains("native_popup_animation: bool"));
}

#[test]
fn layout_hit_has_one_action_source() {
    let hit = include_str!("../layout/hit.rs");

    assert!(
        hit.contains("enum Kind {")
            && hit.contains("Frame,")
            && hit.contains("Chrome(chrome::Chrome),")
            && hit.contains("Target(interaction::Target),")
            && hit.contains("kind: Kind,")
    );
    assert!(!hit.contains("chrome: Option<chrome::Chrome>"));
    assert!(!hit.contains("target: Option<interaction::Target>"));
}

#[test]
fn interaction_pruning_receipt_encodes_capture_implication() {
    let interaction = include_str!("../interaction/mod.rs");

    assert!(
        interaction.contains("enum PruneOutcome {")
            && interaction.contains("Unchanged,")
            && interaction.contains("Changed {")
            && interaction.contains("capture_removed: bool,")
            && interaction.contains("menu_removed: bool,")
            && interaction.contains("outcome: PruneOutcome,")
    );
    assert!(!interaction.contains("changed: bool"));
}

#[test]
fn pointer_press_state_is_one_lifecycle_species() {
    let pointer = include_str!("../interaction/pointer.rs");

    assert!(
        pointer.contains("press: Option<Press>")
            && pointer.contains("enum Press {")
            && pointer.contains("Captured {")
            && pointer.contains("Uncaptured {")
            && pointer.contains("intent: PressIntent")
    );
    for parallel in [
        "pressed: Option<Target>",
        "capture: Option<Capture>",
        "press_intent: Option<PressIntent>",
    ] {
        assert!(
            !pointer.contains(parallel),
            "pointer press lifecycle must not regain parallel state: {parallel}"
        );
    }
}

#[test]
fn pointer_position_and_surface_share_one_location() {
    let pointer = include_str!("../interaction/pointer.rs");

    assert!(
        pointer.contains("location: Option<Location>")
            && pointer.contains("pub(crate) struct Location {")
            && pointer.contains("point: Point")
            && pointer.contains("surface: crate::popup::Surface")
            && pointer.contains("pub(crate) fn location(&self) -> Option<Location>")
    );
    let interaction = include_str!("../interaction/mod.rs");
    assert!(interaction.contains("pub(crate) fn set_pointer_location("));
    assert!(!interaction.contains("fn set_pointer_position("));
    assert!(!pointer.contains("position: Option<Point>"));
    assert!(!pointer.contains("pub(super) surface: crate::popup::Surface"));
}

#[test]
fn session_cursor_state_encodes_pending_publication() {
    let window = include_str!("../session/window.rs");

    assert!(
        window.contains("enum Cursor {")
            && window.contains("Synced(pointer::Cursor)")
            && window.contains("Pending(pointer::Cursor)")
            && window.contains("fn take_pending(&mut self) -> Option<pointer::Cursor>")
    );
    assert!(!window.contains("cursor_changed: bool"));
}

#[test]
fn focus_text_target_projection_is_explicitly_optional() {
    let focus = include_str!("../session/focus.rs");
    let target = include_str!("../interaction/target.rs");

    assert!(focus.contains("pub fn text_target(self) -> Option<interaction::Target>"));
    assert!(!focus.contains("pub fn target(self) -> interaction::Id"));
    assert!(!focus.contains("pub fn into_target"));
    assert!(!target.contains("pub fn text_area(focus: session::Focus)"));
}

#[test]
fn command_focus_precedence_has_one_window_owner() {
    let window = include_str!("../session/window.rs");
    let focus = include_str!("../session/focus.rs");
    let menu = include_str!("../session/interaction/menu.rs");

    assert!(
        window.contains("pub(super) fn command_focus(&self) -> Option<Focus>")
            && window.contains("return palette.captured_focus();")
            && window.contains("self.menu_restore_focus.or(self.focus).or_else(||")
    );
    assert!(focus.contains("self.window(id).and_then(Window::command_focus)"));
    assert_eq!(menu.matches("window.command_focus()").count(), 2);
    assert!(!menu.contains("fn restore_focus_for_menu"));
}

#[test]
fn realized_material_parts_encode_tint_inside_frost() {
    let region = include_str!("../scene/region.rs");

    assert!(
        region.contains("enum Parts {")
            && region.contains("Frost {")
            && region.contains("surface_tint: bool,")
            && region.contains("parts: Parts,")
    );
    assert!(!region.contains("struct RealizedMaterialParts {\n    backdrop_frost: bool,"));
    assert!(!region.contains("struct RealizedMaterialParts {\r\n    backdrop_frost: bool,"));
}

#[test]
fn composition_key_has_one_structural_identity_species() {
    let tree = include_str!("../composition/tree.rs");

    assert!(
        tree.contains("enum Key {")
            && tree.contains("Ordinary {")
            && tree.contains("ProvidedRow {")
            && tree.contains("TableCell {")
            && tree.contains("TableHeaderCell {")
    );
    assert!(!tree.contains("provided: Option<crate::virtual_list::Key>"));
    assert!(!tree.contains("table_cell: Option<crate::table::Cell>"));
    assert!(!tree.contains("table_header_cell: Option<crate::table::HeaderCell>"));
}

#[test]
fn text_box_projects_cursor_and_selection_as_one_caret() {
    let text_box = include_str!("../view/control/text_box.rs");
    let owner = text_box
        .split("struct Caret")
        .next()
        .expect("TextBox declaration precedes its private caret");

    assert!(
        text_box.contains("struct Caret {")
            && text_box.contains("cursor: usize,")
            && text_box.contains("selection: Option<Range<usize>>,")
            && text_box.contains("caret: Option<Caret>,")
    );
    assert!(!owner.contains("cursor: Option<usize>"));
    assert!(!owner.contains("selection: Option<Range<usize>>"));
}

#[test]
fn virtual_list_frame_resolves_viewport_and_request_together() {
    let frame = include_str!("../layout/frame.rs");

    assert!(
        frame.contains("struct VirtualGeometry {")
            && frame.contains("viewport: Viewport,")
            && frame.contains("request: crate::virtual_list::Request,")
            && frame.contains("geometry: Option<VirtualGeometry>,")
    );
    let content = frame
        .split("struct VirtualListContent")
        .nth(1)
        .and_then(|source| source.split("struct VirtualGeometry").next())
        .expect("virtual-list content precedes its geometry");
    assert!(!content.contains("viewport: Option<Viewport>"));
    assert!(!content.contains("request: Option<crate::virtual_list::Request>"));
}

#[test]
fn view_binding_trigger_species_keeps_slider_factory_with_current_trigger() {
    let binding = include_str!("../view/binding.rs");

    assert!(
        binding.contains("enum Trigger {")
            && binding.contains("Fixed(command::AnyTrigger),")
            && binding.contains("Slider {")
            && binding.contains("current: command::AnyTrigger,")
            && binding.contains("factory: command::AnyValueTrigger<f64>,")
            && binding.contains("trigger: Trigger,")
    );
    assert!(!binding.contains("slider_trigger: Option<command::AnyValueTrigger<f64>>"));
}

#[test]
fn standard_menu_projected_entry_separates_catalog_and_authored_lifecycles() {
    let menu = include_str!("../view/node/standard_menu.rs");
    let entry = menu
        .split("enum ProjectedEntry")
        .nth(1)
        .and_then(|source| source.split("impl ProjectedEntry").next())
        .expect("projected entry declaration precedes its implementation");

    assert!(
        menu.contains("enum ProjectedEntry {")
            && menu.contains("Catalog {")
            && menu.contains("standard: Option<Standard>,")
            && menu.contains("node: Option<Node>,")
            && menu.contains("Authored {")
            && menu.contains("node: Node,")
            && menu.contains("after: Option<Standard>,")
    );
    assert!(!entry.contains("authored_after: Option<Standard>"));
}

#[test]
fn table_track_species_owns_column_only_resize_facts() {
    let table = include_str!("../layout/table.rs");
    let track = table
        .split("pub(crate) struct Track")
        .nth(1)
        .and_then(|source| source.split("struct Column").next())
        .expect("track declaration precedes its column facts");

    assert!(
        table.contains("enum Kind {")
            && table.contains("Column(Column),")
            && table.contains("Row,")
            && track.contains("kind: Kind,")
    );
    assert!(!track.contains("axis: Axis"));
    assert!(!track.contains("column: Option<Column>"));
}

#[test]
fn visual_scalar_species_separates_moving_and_resting_state() {
    let visual = include_str!("../scene/visual.rs");
    let scalar = visual
        .split("pub(crate) enum Scalar")
        .nth(1)
        .and_then(|source| source.split("pub(crate) struct Scrollbar").next())
        .expect("scalar declaration precedes scrollbar state");

    assert!(
        scalar.contains("Moving {")
            && scalar.contains("from: f32,")
            && scalar.contains("target: f32,")
            && scalar.contains("progress: f32,")
            && scalar.contains("Resting {")
            && scalar.contains("value: f32,")
    );
    assert!(!scalar.contains("motion: Motion"));
}

#[test]
fn hover_tip_lifecycle_is_idle_waiting_or_visible() {
    let pointer = include_str!("../interaction/pointer.rs");
    let lifecycle = pointer
        .split("enum HoverTip")
        .nth(1)
        .and_then(|source| source.split("pub(crate) struct Capture").next())
        .expect("hover-tip lifecycle precedes pointer capture");

    assert!(
        lifecycle.contains("Idle,")
            && lifecycle.contains("Waiting {")
            && lifecycle.contains("started_at: Instant,")
            && lifecycle.contains("Visible {")
            && lifecycle.contains("anchor: Point,")
    );
    assert!(!lifecycle.contains("started_at: Option<Instant>"));
    assert!(!lifecycle.contains("visible: bool"));
    assert!(!lifecycle.contains("anchor: Option<Point>"));
}

#[test]
fn interaction_command_surface_is_exclusive() {
    let interaction = include_str!("../interaction/mod.rs");
    let owner = interaction
        .split("pub(crate) struct Interaction")
        .nth(1)
        .and_then(|source| source.split("pub(crate) struct Pruned").next())
        .expect("interaction command surface precedes prune receipt");

    assert!(
        owner.contains("surface: Option<Surface>,")
            && owner.contains("enum Surface {")
            && owner.contains("Menu(Menu),")
            && owner.contains("CommandPalette(CommandPalette),")
    );
    assert!(!owner.contains("open_menu: Option<Menu>"));
    assert!(!owner.contains("command_palette: Option<CommandPalette>"));
}

#[test]
fn animation_vocabulary_is_platform_neutral() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let animation = std::fs::read_to_string(root.join("src/animation.rs"))
        .expect("animation source should read");
    let native_runner = std::fs::read_to_string(root.join("src/platform/runner/native.rs"))
        .expect("native runner source should read");

    assert!(
        !animation.contains("winit")
            && !animation.contains("ControlFlow")
            && !animation.contains("control_flow"),
        "platform-neutral schedule vocabulary must not realize an event-loop policy"
    );
    assert!(
        native_runner.contains("fn control_flow(schedule: animation::Schedule")
            && native_runner.contains("ControlFlow::WaitUntil(deadline)")
            && native_runner.contains("event_loop.set_control_flow(control_flow)"),
        "the winit runner must own Schedule-to-ControlFlow realization"
    );
}

#[test]
fn pointer_grammar_consumes_platform_facts_without_importing_platform_ffi() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let pointer = std::fs::read_to_string(root.join("src/pointer/mod.rs"))
        .expect("pointer source should read");
    let platform_event = std::fs::read_to_string(root.join("src/platform/event.rs"))
        .expect("platform event source should read");
    let native_runner = std::fs::read_to_string(root.join("src/platform/runner/native.rs"))
        .expect("native runner source should read");

    for os_detail in [
        "windows_sys",
        "GetDoubleClickTime",
        "GetSystemMetrics",
        "system_multi_click_settings",
    ] {
        assert!(
            !pointer.contains(os_detail),
            "pointer grammar must not own platform detail {os_detail}"
        );
    }
    assert!(
        platform_event.contains("pub(crate) fn system_multi_click_settings()")
            && platform_event.contains("GetDoubleClickTime")
            && native_runner.contains(".set_multi_click_settings("),
        "the native event adapter must realize and inject OS click thresholds"
    );
}

#[test]
fn icon_pack_dependency_has_one_owner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let icon = std::fs::read_to_string(src.join("icon.rs")).expect("icon source should read");
    let text_system = std::fs::read_to_string(src.join("text/layout/system.rs"))
        .expect("text layout system should read");

    assert!(
        icon.contains("iconflow::try_icon(")
            && icon.contains("iconflow::fonts()")
            && icon.contains("pub(crate) fn font_bytes()"),
        "the icon owner must resolve glyphs and expose embedded font bytes"
    );
    assert!(
        text_system.contains("crate::icon::font_bytes()") && !text_system.contains("iconflow"),
        "text may consume icon fonts without learning the selected pack dependency"
    );
}

#[test]
fn table_std_capabilities_have_no_framework_trait_mirrors() {
    let table = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/table.rs"),
    )
    .expect("table source should read");

    for retired in [
        "pub trait Value",
        "pub trait Sort",
        "pub trait EditText",
        "pub trait EditToggle",
        "RecordOrder",
    ] {
        assert!(
            !table.contains(retired),
            "{retired} should be structurally absent after std owns the meaning"
        );
    }
    for std_boundary in [
        "pub fn text<R, V>",
        "pub fn boolean<R, V>",
        "pub fn records(",
        "pub fn unsortable(self)",
        "V: Display + FromStr",
        "V: Display + Ord",
        "V: Clone + Into<bool> + From<bool>",
    ] {
        assert!(
            table.contains(std_boundary),
            "table species should expose {std_boundary}"
        );
    }
    assert!(
        !table.contains("sortable: bool")
            && table.contains("ordering: Option<Rc<OrderProjection>>"),
        "one optional ordering projection must own header and bounded-record sorting"
    );
}

#[test]
fn examples_launch_through_the_application_ceiling() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

    for example in ["text_editor", "control_gallery", "glass_tuner"] {
        let main = std::fs::read_to_string(root.join(example).join("main.rs"))
            .expect("example main should read");
        let runtime = std::fs::read_to_string(root.join(example).join("app/runtime.rs"))
            .expect("example runtime should read");

        assert!(
            main.contains("wgpu_l3::platform::launch("),
            "{example} should launch its app through the blessed ceiling"
        );
        for lower_layer in ["platform::Runner", "platform::Platform", "Host"] {
            assert!(
                !runtime.contains(lower_layer),
                "{example} ordinary runtime construction should not name {lower_layer}"
            );
        }
    }
}

#[test]
fn frame_preparation_has_one_recipe() {
    let presentation = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("runtime")
            .join("presentation.rs"),
    )
    .expect("runtime presentation source should read");

    assert_eq!(
        presentation
            .matches("paint_parts_with_clear_theme_and_visuals")
            .count(),
        1,
        "base scene painting must have one prepared-frame owner"
    );
    assert_eq!(
        presentation.matches("self.prepare_frame(").count(),
        2,
        "immediate and pending rendering must consume the same frame recipe"
    );
}

#[test]
fn renderer_dependencies_stay_at_rendering_and_observation_boundaries() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let rendering_roots = [
        src_dir.join("platform").join("native"),
        src_dir.join("render"),
        src_dir.join("paint"),
        src_dir.join("text"),
    ];

    assert_imports_only_under_any(&src_dir, &rendering_roots, &["paint"]);

    let mut observation_roots = rendering_roots.to_vec();
    observation_roots.push(src_dir.join("diagnostics"));
    assert_imports_only_under_any(&src_dir, &observation_roots, &["render"]);
}

#[test]
fn renderer_publishes_render_facts_without_importing_diagnostics() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let render = root.join("render");
    let report = std::fs::read_to_string(render.join("report.rs"))
        .expect("renderer report source should read");
    let diagnostics = std::fs::read_to_string(root.join("diagnostics").join("mod.rs"))
        .expect("diagnostics module should read");
    let report_projection = ["pub use crate::", "render::RenderReport;"].concat();

    assert_source_patterns_absent(&render, &["crate::diagnostics".to_owned()]);
    assert!(
        report.contains("pub struct RenderReport")
            && report.contains("pub(crate) struct DrawStats"),
        "the renderer must own its public receipt and private draw facts"
    );
    assert!(
        diagnostics.contains(&report_projection)
            && !diagnostics.contains("Report as RenderReport")
            && !root.join("diagnostics").join("draw.rs").exists(),
        "diagnostics must observe the exact renderer-owned receipt without an alias or duplicate facts"
    );
}

#[test]
fn layout_publishes_text_facts_without_importing_diagnostics() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let layout = root.join("layout");
    let text =
        std::fs::read_to_string(layout.join("text.rs")).expect("layout text source should read");
    let layout_mod =
        std::fs::read_to_string(layout.join("mod.rs")).expect("layout module should read");
    let diagnostics = std::fs::read_to_string(root.join("diagnostics").join("mod.rs"))
        .expect("diagnostics module should read");

    assert_source_patterns_absent(
        &layout,
        &[
            "crate::diagnostics".to_owned(),
            "super::super::diagnostics".to_owned(),
            "diagnostics::".to_owned(),
        ],
    );
    assert!(
        text.contains("pub struct Text")
            && text.contains("pub(crate) fn add(&mut self, diagnostics: Self)")
            && layout_mod.contains("pub use text::Text;")
            && diagnostics.contains("pub use crate::layout::Text;")
            && !root.join("diagnostics").join("text.rs").exists(),
        "layout must own its exact text fact while diagnostics projects and accumulates it"
    );
}

#[test]
fn view_callback_context_is_a_facade_responsibility() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let view = src.join("view");
    let context_path = view.join("context.rs");
    let context =
        std::fs::read_to_string(&context_path).expect("view callback context should read");
    let view_mod = std::fs::read_to_string(view.join("mod.rs")).expect("view module should read");
    let runtime = std::fs::read_to_string(src.join("runtime").join("mod.rs"))
        .expect("runtime module should read");
    let builder = std::fs::read_to_string(src.join("runtime").join("builder.rs"))
        .expect("runtime builder should read");
    let presentation = std::fs::read_to_string(src.join("runtime").join("presentation.rs"))
        .expect("runtime presentation should read");
    let slots = std::fs::read_to_string(root.join("tools").join("one_way_slots.json"))
        .expect("one-way slot map should read");

    assert!(
        context.contains("pub struct Context")
            && context.contains("window: window::Id")
            && context.contains("diagnostics: Diagnostics")
            && context.contains("pub fn diagnostics(&self) -> &Diagnostics")
            && view_mod.contains("pub use context::Context;"),
        "the established view facade must expose one immutable per-window callback envelope"
    );
    assert!(
        runtime.contains("type ViewCallback<M, V> = Box<dyn Fn(&M, view::Context) -> V>;")
            && builder.contains("callback: impl Fn(&M, view::Context) -> V2 + 'static")
            && presentation.contains("view::Context::new(")
            && presentation.contains("self.diagnostics.get(window).cloned().unwrap_or_default()"),
        "runtime must construct the facade context exactly when invoking the application view callback"
    );
    assert_pattern_only_in(&view, "diagnostics", &context_path);
    assert!(
        slots.contains("\"src/view/context.rs\": \"facade\"")
            && !slots.contains("\"src/view\": \"facade\""),
        "the gauge must assign only the dedicated callback source to facade responsibility"
    );
}

#[test]
fn semantic_scene_lowering_belongs_to_renderer() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let render_scene = std::fs::read_to_string(root.join("render").join("scene.rs"))
        .expect("renderer scene projection should read");
    let native = root.join("platform").join("native");
    let surface = std::fs::read_to_string(native.join("surface.rs"))
        .expect("native surface source should read");
    let popup =
        std::fs::read_to_string(native.join("popup.rs")).expect("native popup source should read");

    assert!(
        render_scene.contains("fn to_paint_scene_at_scale(")
            && render_scene.contains("struct PopupProjection"),
        "renderer must own semantic scene lowering and its scale-resolved popup projection"
    );
    assert!(
        surface.contains("render::scene::to_paint_scene_at_scale")
            && popup.contains("render::scene::to_paint_scene_at_scale")
            && popup.contains("render::scene::PopupProjection::resolve"),
        "native realization must consume the renderer scene contract"
    );
    assert!(
        !native.join("paint.rs").exists() && !native.join("color.rs").exists(),
        "the native adapter must not retain semantic-to-paint or renderer-color ownership"
    );
}

#[test]
fn native_platform_uses_first_party_renderer_surface_contracts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let platform = src.join("platform");
    let render_mod = std::fs::read_to_string(src.join("render").join("mod.rs"))
        .expect("render module should read");
    let render_context = std::fs::read_to_string(src.join("render").join("context.rs"))
        .expect("render context should read");
    let render_surface = std::fs::read_to_string(src.join("render").join("surface.rs"))
        .expect("render surface should read");
    let slots = std::fs::read_to_string(root.join("tools").join("one_way_slots.json"))
        .expect("one-way slot map should read");

    assert_source_patterns_absent(&platform, &["wgpu::".to_owned(), "wgpu_hal::".to_owned()]);
    for contract in [
        "pub(crate) struct Backends(wgpu::Backends)",
        "pub(crate) struct Backend(wgpu::Backend)",
        "pub(crate) fn backend(&self) -> Backend",
    ] {
        assert!(
            render_context.contains(contract),
            "renderer context boundary must contain {contract}"
        );
    }
    for contract in [
        "pub(crate) struct Format(wgpu::TextureFormat)",
        "pub(crate) struct Target(wgpu::SurfaceTargetUnsafe)",
        "pub(crate) enum WindowsPopupSupport",
        "self.inner.as_hal::<wgpu_hal::api::Dx12>()",
    ] {
        assert!(
            render_surface.contains(contract),
            "renderer surface boundary must contain {contract}"
        );
    }
    for module in ["canvas", "context", "surface"] {
        assert!(
            render_mod.contains(&format!("pub(crate) mod {module};")),
            "supporting renderer concepts must remain namespaced under {module}"
        );
    }
    for central in [
        "pub(crate) use canvas::Canvas;",
        "pub(crate) use context::Context;",
        "pub(crate) use surface::Surface;",
    ] {
        assert!(
            render_mod.contains(central),
            "renderer parent must project only its same-named central type: {central}"
        );
    }
    for retired_alias in ["CanvasOptions", "ContextOptions", "FrameOutcome"] {
        assert!(
            !render_mod.contains(retired_alias),
            "renderer parent must not retain compound support alias {retired_alias}"
        );
    }
    assert!(
        slots.contains("\"wgpu\": [\"renderer\"]")
            && slots.contains("\"wgpu_hal\": [\"renderer\"]"),
        "wgpu and its HAL must have one renderer owner"
    );
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
fn paint_stays_with_display_list_and_rendering_consumers() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let allowed_roots = [
        src_dir.join("paint"),
        src_dir.join("render"),
        src_dir.join("platform").join("native"),
    ];

    assert_imports_only_under_any(&src_dir, &allowed_roots, &["paint"]);
}

#[test]
fn paint_keeps_policy_and_not_shared_coordinate_modules() {
    let paint_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("paint");
    let paint_mod =
        std::fs::read_to_string(paint_dir.join("mod.rs")).expect("paint module should read");

    for module in ["area", "point"] {
        assert!(
            !paint_dir.join(format!("{module}.rs")).exists()
                && !paint_mod.contains(&format!("mod {module};")),
            "renderer-neutral {module} coordinates should stay with geometry"
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

    assert!(
        paint_mod.contains("use crate::geometry::{area, point};"),
        "paint should consume geometry-owned unit-qualified coordinates"
    );
}

#[test]
fn shaped_text_crosses_scene_and_paint_without_renderer_types() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let text_output =
        std::fs::read_to_string(src_dir.join("text").join("layout").join("output.rs"))
            .expect("text layout output should read");
    let scene_primitive = std::fs::read_to_string(src_dir.join("scene").join("primitive.rs"))
        .expect("scene primitive should read");
    let paint_mod = std::fs::read_to_string(src_dir.join("paint").join("mod.rs"))
        .expect("paint module should read");

    assert!(
        text_output.contains("pub(crate) struct ShapedBuffer"),
        "text should own the opaque shared handle for its shaped output"
    );
    assert!(
        !text_output.contains("pub fn buffer(&self) -> Rc<RefCell<glyphon::Buffer>>"),
        "text layout should not expose its renderer buffer representation publicly"
    );
    assert!(
        scene_primitive.contains("buffer: text_model::layout::ShapedBuffer")
            && paint_mod.contains("buffer: text::layout::ShapedBuffer"),
        "scene and paint should transport the text-owned shaped handle"
    );
    assert_source_patterns_absent(
        &src_dir.join("scene"),
        &[concat!("glyph", "on::").to_owned()],
    );
    assert_source_patterns_absent(
        &src_dir.join("paint"),
        &[concat!("glyph", "on::").to_owned()],
    );
}

#[test]
fn provided_list_selection_endpoints_are_atomic() {
    let selection = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("selection.rs"),
    )
    .expect("provided-list selection source should read");

    assert!(
        selection.contains("struct Endpoint {")
            && selection.contains("key: Key,")
            && selection.contains("index: usize,")
            && selection.contains("anchor: Option<Endpoint>,")
            && selection.contains("active: Option<Endpoint>,"),
        "anchor and active should each store one atomic key/index endpoint"
    );
    for parallel_field in [
        "anchor: Option<Key>",
        "anchor_index:",
        "active: Option<Key>",
        "active_index:",
    ] {
        assert!(
            !selection.contains(parallel_field),
            "selection must not restore independently optional endpoint state: {parallel_field}"
        );
    }
}

#[test]
fn read_only_text_vocabulary_does_not_depend_on_mutation() {
    let text_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("text");
    let text_mod =
        std::fs::read_to_string(text_dir.join("mod.rs")).expect("text module should read");
    let edit_mod = std::fs::read_to_string(text_dir.join("edit").join("mod.rs"))
        .expect("text edit module should read");

    for module in ["selection", "surface", "view"] {
        assert!(
            text_mod.contains(&format!("pub mod {module};")),
            "always-present text {module} should be a first-class module"
        );
    }
    for projection in [
        "pub use edit::Edit;",
        "pub use surface::Surface;",
        "pub use view::View;",
    ] {
        assert!(
            text_mod.contains(projection),
            "same-named central types should be the only parent projections: {projection}"
        );
    }
    for old_file in ["caret.rs", "motion.rs", "state.rs", "surface", "view.rs"] {
        assert!(
            !text_dir.join("edit").join(old_file).exists(),
            "read-only text vocabulary must not remain housed under mutation: {old_file}"
        );
    }
    for retired_projection in [
        "pub use caret::CaretMap;",
        "pub use motion::Motion;",
        "pub use state::State;",
        "pub use surface::",
        "pub use view::",
    ] {
        assert!(
            !edit_mod.contains(retired_projection),
            "mutation must not preserve a compatibility projection for {retired_projection}"
        );
    }
    for lower in ["selection", "surface", "layout"] {
        assert_source_patterns_absent(
            &text_dir.join(lower),
            &["super::edit".to_owned(), "text::edit".to_owned()],
        );
    }
    let view =
        std::fs::read_to_string(text_dir.join("view.rs")).expect("text view module should read");
    for forbidden in ["super::edit", "text::edit"] {
        assert!(
            !view.contains(forbidden),
            "always-present text view must not depend on mutation: {forbidden}"
        );
    }
}

#[test]
fn command_context_consumes_caret_mapping_without_concrete_layout() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let context = std::fs::read_to_string(src_dir.join("context").join("mod.rs"))
        .expect("command context source should read");
    let layout_engine = std::fs::read_to_string(src_dir.join("layout").join("engine.rs"))
        .expect("layout engine source should read");
    let document_edit = std::fs::read_to_string(src_dir.join("document").join("edit.rs"))
        .expect("document command targets should read");

    assert!(
        context.contains("dyn text::selection::CaretMap")
            && context.contains("fn with_caret_map(")
            && context.contains("fn caret_map(&self)"),
        "command context must name the lower caret-mapping capability"
    );
    for forbidden in [
        "layout::".to_owned(),
        "TextService".to_owned(),
        format!("{}{}", "with_text_", "service"),
        format!("{}{}", "text_", "service"),
    ] {
        assert!(
            !context.contains(&forbidden),
            "command context must not retain concrete layout service vocabulary: {forbidden}"
        );
    }
    assert!(
        layout_engine.contains("fn text_caret_map(")
            && document_edit.contains("cx.caret_map()")
            && document_edit.contains("apply_selection_with_caret_map"),
        "runtime layout must realize the narrow capability consumed by document selection"
    );
    assert_source_patterns_absent(
        &src_dir,
        &[
            format!("{}{}", "with_text_", "service"),
            format!("{}{}", "text_", "service()"),
        ],
    );
}

#[test]
fn preedit_projection_is_explicit_and_draft_input_owns_its_lifecycle() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let text_dir = src_dir.join("text");
    let text_mod =
        std::fs::read_to_string(text_dir.join("mod.rs")).expect("text module should read");
    let preedit = std::fs::read_to_string(text_dir.join("preedit.rs"))
        .expect("text preedit module should read");
    let view =
        std::fs::read_to_string(text_dir.join("view.rs")).expect("text view module should read");
    let draft_input = std::fs::read_to_string(src_dir.join("draft/input/mod.rs"))
        .expect("draft input module should read");
    let layout_text = std::fs::read_to_string(src_dir.join("layout/text.rs"))
        .expect("layout text bridge should read");

    assert!(
        text_mod.contains("pub mod preedit;")
            && text_mod.contains("pub use preedit::Preedit;")
            && preedit.contains("pub struct Preedit"),
        "preedit should have one same-named module/type owner and one parent projection"
    );
    for forbidden in ["Preedit", "preedit"] {
        assert!(
            !view.contains(forbidden),
            "always-present view state must not absorb IME composition: {forbidden}"
        );
    }
    assert!(
        draft_input.contains("active: Option<Active>")
            && draft_input.contains("struct Active {")
            && draft_input.contains("target: Target")
            && draft_input.contains("preedit: Option<text::Preedit>")
            && !draft_input.contains("target: Option<Target>")
            && draft_input.contains("fn set_preedit(")
            && draft_input.contains("fn clear_preedit("),
        "draft input should nest preedit identity and lifecycle beneath its active target"
    );
    assert!(
        layout_text.contains("text_area.preedit()")
            && layout_text.contains("text_box.preedit()")
            && layout_text.contains("_with_preedit"),
        "layout should receive composition explicitly from the active input projection"
    );
    assert_source_patterns_absent(&src_dir, &[concat!("text::view::", "Preedit").to_owned()]);
}

#[test]
fn selection_operations_are_structurally_distinct_from_text_mutation() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let text_dir = src_dir.join("text");
    let selection_mod = std::fs::read_to_string(text_dir.join("selection/mod.rs"))
        .expect("text selection module should read");
    let selection_operation = std::fs::read_to_string(text_dir.join("selection/operation.rs"))
        .expect("text selection operation should read");
    let edit_operation = std::fs::read_to_string(text_dir.join("edit/operation.rs"))
        .expect("text edit operation should read");
    let history = std::fs::read_to_string(text_dir.join("edit/history.rs"))
        .expect("text edit history should read");
    let input =
        std::fs::read_to_string(src_dir.join("input/mod.rs")).expect("input module should read");
    let view_action = std::fs::read_to_string(src_dir.join("view/action.rs"))
        .expect("view action module should read");
    let keymap =
        std::fs::read_to_string(src_dir.join("keymap.rs")).expect("keymap module should read");
    let document_command = std::fs::read_to_string(src_dir.join("document/command.rs"))
        .expect("document command module should read");

    assert!(
        selection_mod.contains("pub use operation::{Operation, PointerKind, apply};")
            && selection_operation.contains("pub enum Operation")
            && selection_operation.contains("pub fn apply("),
        "selection should own its operation vocabulary and application path"
    );
    for mutation in [
        "Insert(String)",
        "ImeCommit(String)",
        "ReplaceRange",
        "Backspace",
        "DeleteWordForward",
    ] {
        assert!(
            !selection_operation.contains(mutation),
            "selection operation must not absorb text mutation: {mutation}"
        );
    }
    for selection in [
        "MovePosition",
        "ExtendPosition",
        "SelectAll",
        "SetPosition",
        "Pointer",
        "PointerEditKind",
    ] {
        assert!(
            !edit_operation.contains(selection),
            "text mutation must not preserve selection vocabulary: {selection}"
        );
        assert!(
            !history.contains(selection),
            "mutation history must not classify selection operations: {selection}"
        );
    }
    assert!(
        input.contains("TextSelection(text::selection::Operation)")
            && input.contains("TextEdit(text::Edit)"),
        "the public input boundary should distinguish selection from mutation"
    );
    assert!(
        view_action.contains("TextSelection(text::selection::Operation)")
            && !view_action.contains("TextEdit("),
        "pointer view actions should carry selection without a spare mutation route"
    );
    assert!(
        keymap.contains("pub(crate) enum TextOperation")
            && keymap.contains("Selection(text::selection::Operation)")
            && keymap.contains("Edit(text::Edit)"),
        "key routing should resolve to a private typed selection-or-mutation sum"
    );
    assert!(
        document_command.contains("pub struct ApplySelection;")
            && document_command.contains("type Args = text::selection::Operation;")
            && document_command.contains("type Args = text::Edit;"),
        "document routing should expose distinct selection and mutation commands"
    );
}

#[test]
fn old_paint_space_root_module_is_extinct() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let lib = std::fs::read_to_string(src_dir.join("lib.rs")).expect("crate root should read");
    let old_module = old_paint_space_module();

    assert!(
        !src_dir.join(old_module).exists(),
        "old paint-space root bucket should stay extinct"
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
fn geometry_namespaces_unit_species_without_compound_aliases() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let geometry_dir = src_dir.join("geometry");
    let geometry_mod =
        std::fs::read_to_string(geometry_dir.join("mod.rs")).expect("geometry module should read");

    for module in ["area", "point"] {
        assert!(
            geometry_mod.contains(&format!("pub mod {module};")),
            "unit species should use the public geometry namespace: {module}::Logical"
        );
    }

    assert!(
        geometry_mod.contains("pub use point::Point;"),
        "a module's same-named central type should be its sole parent projection"
    );
    assert!(
        !geometry_mod.contains("pub use area::")
            && !geometry_mod.contains("pub use point::{")
            && !geometry_mod.contains("pub use point::Logical"),
        "supporting coordinate species should remain simply named behind their module"
    );

    for module in ["rect", "size"] {
        assert!(
            !geometry_mod.contains(&format!("pub mod {module};")),
            "single-species geometry should retain its established root name: {module}"
        );
    }

    for file in ["area.rs", "point.rs"] {
        let source = std::fs::read_to_string(geometry_dir.join(file))
            .unwrap_or_else(|error| panic!("geometry/{file} should read: {error}"));
        assert!(
            source.contains("pub struct Logical"),
            "geometry/{file} should declare the namespaced Logical species"
        );
    }

    for alias in [
        "LogicalArea",
        "LogicalPoint",
        "PhysicalArea",
        "PhysicalPoint",
    ] {
        assert!(
            !geometry_mod.contains(alias),
            "compound coordinate alias should collapse to its namespaced declaration: {alias}"
        );
    }
    assert!(
        !geometry_mod.contains(" as Logical") && !geometry_mod.contains(" as Physical"),
        "parent projections should not preserve compound coordinate declarations behind aliases"
    );
    assert_source_patterns_absent(
        &src_dir,
        &[
            concat!("geometry::Logical", "Area").to_owned(),
            concat!("geometry::Logical", "Point").to_owned(),
        ],
    );
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
fn document_workflow_is_an_independent_lower_owner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let document = src.join("document");
    let text_area = std::fs::read_to_string(src.join("widget/control/text_area.rs"))
        .expect("text area widget source should read");
    let slots = std::fs::read_to_string(root.join("tools").join("one_way_slots.json"))
        .expect("one-way slot map should read");

    assert!(
        text_area.contains("pub fn from_document(document: &document::Document)")
            && text_area.contains("document.buffer().clone(), document.text_state()"),
        "the document projection should remain one named value-semantics constructor"
    );
    assert_source_patterns_absent(
        &document,
        &[
            "crate::runtime".to_owned(),
            "crate::session".to_owned(),
            "crate::view".to_owned(),
            "crate::widget".to_owned(),
            "crate::layout".to_owned(),
            "crate::platform".to_owned(),
        ],
    );
    assert!(
        slots.contains("\"document\": {")
            && slots.contains("\"modules\": [\"document\"]")
            && slots.contains("\"windows_sys\": [\"document\", \"platform\"]")
            && slots.contains("\"external_module_exceptions\": {}"),
        "document must be a named owner of its contracts and atomic replacement dependency"
    );
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
fn text_buffer_old_line_and_mmap_representations_stay_deleted() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let buffer = root.join("src").join("text").join("buffer");
    let document = std::fs::read_to_string(buffer.join("document.rs"))
        .expect("text document source should read");
    let span_tree = std::fs::read_to_string(buffer.join("document").join("span_tree.rs"))
        .expect("source span tree should read");
    let buffer_api =
        std::fs::read_to_string(buffer.join("mod.rs")).expect("text buffer API should read");
    let manifest =
        std::fs::read_to_string(root.join("Cargo.toml")).expect("Cargo manifest should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    for deleted in ["line.rs", "source.rs"] {
        assert!(
            !buffer.join("document").join(deleted).exists(),
            "the retired text-buffer representation must not return as {deleted}"
        );
    }
    for retired in [
        "TextLineTree",
        "Vec<TextLine>",
        "MappedTextSource",
        "memmap2",
    ] {
        assert!(
            !document.contains(retired)
                && !span_tree.contains(retired)
                && !manifest.contains(retired),
            "the retired text-buffer representation must not contain {retired}"
        );
    }
    for witness in ["struct SourceSpan", "struct SpanTree", "Arc<Node>"] {
        assert!(
            span_tree.contains(witness),
            "the source-span tree must retain {witness}"
        );
    }
    assert!(
        document.contains("tree: SpanTree") && document.contains("lines: LineIndex"),
        "TextDocument must have one source-span representation and its persistent line index"
    );
    assert!(
        span_tree.contains("GraphemeCursor")
            && document.contains(".floor_grapheme_boundary(")
            && !document.contains("grapheme_boundaries"),
        "grapheme navigation must stay lazy and chunk-aware instead of returning to eager indexes"
    );
    assert!(
        buffer_api.contains("pub fn from_mapped_file")
            && buffer_api.contains("Self::from_file(path)"),
        "the compatibility file API must delegate to owned source loading"
    );
    for decision in [
        "Owned sources, never retained mappings",
        "SIGBUS or an access violation",
        "locked against the atomic-rename save path",
    ] {
        assert!(
            master.contains(decision),
            "the mmap-retirement decision must retain {decision}"
        );
    }
}

#[test]
fn caret_affinity_has_one_position_to_cursor_owner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let text = root.join("src").join("text");
    let document = std::fs::read_to_string(text.join("buffer").join("document.rs"))
        .expect("text document source should read");
    let buffer = std::fs::read_to_string(text.join("buffer").join("mod.rs"))
        .expect("text buffer source should read");
    let selection = std::fs::read_to_string(text.join("selection").join("operation.rs"))
        .expect("text selection operation source should read");
    let glyph = std::fs::read_to_string(text.join("layout").join("glyph.rs"))
        .expect("text glyph source should read");
    let projection = std::fs::read_to_string(text.join("surface").join("projection.rs"))
        .expect("text projection source should read");
    let paint = std::fs::read_to_string(text.join("layout").join("area").join("paint.rs"))
        .expect("text-area paint source should read");
    let reveal = std::fs::read_to_string(text.join("layout").join("area").join("reveal.rs"))
        .expect("text-area reveal source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    assert!(
        document.contains("fn cursor_for_position(&self, position: Position) -> Cursor")
            && document.contains("cursor.index, position.affinity")
            && buffer.contains("inner.document.cursor_for_position(position)"),
        "Buffer must own the affinity-preserving Position-to-Cursor conversion"
    );
    assert!(
        selection
            .matches("buffer.cursor_for_position(position)")
            .count()
            == 2
            && !selection.contains("buffer.cursor_for_text_index(position.index)"),
        "set-position and pointer selection must consume the buffer-owned conversion"
    );
    assert!(
        glyph.contains("floor_grapheme_boundary(line_text, cursor.index)")
            && glyph.contains("cursor.affinity")
            && projection.matches("Position::with_affinity").count() >= 2
            && projection.matches("position.affinity").count() >= 2
            && paint.matches("source_cursor.affinity").count() >= 2
            && reveal.matches("source_cursor.affinity").count() >= 2,
        "clamping and every caret projection must preserve affinity"
    );
    assert!(
        master.contains("Caret affinity belongs to the caret position"),
        "master design must retain caret-affinity ownership"
    );
}

#[test]
fn text_direction_and_obscuring_keep_their_domain_owners() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let text = root.join("src").join("text");
    let map = std::fs::read_to_string(text.join("layout").join("map.rs"))
        .expect("text layout map should read");
    let unicode =
        std::fs::read_to_string(text.join("unicode.rs")).expect("text unicode source should read");
    let projection = std::fs::read_to_string(text.join("surface").join("projection.rs"))
        .expect("text projection source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    assert!(
        map.contains("match (glyph_rtl, visual_left)") && !map.contains("rtl || glyph_rtl"),
        "each glyph's bidi level must own its hit direction"
    );
    assert!(
        unicode.contains("boundaries.last().copied() != Some(text.len())")
            && projection.contains("source_grapheme_boundaries(text).len().saturating_sub(1)"),
        "obscuring must derive its dot count from the unicode-owned unique boundaries"
    );
    assert!(
        master.contains("Hit mapping follows each glyph's own bidi level")
            && master.contains("an empty source has one boundary"),
        "master design must retain bidi-hit and obscuring ownership"
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
fn responder_scope_contains_routing_facts_not_ui_service_state() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let lib = std::fs::read_to_string(src.join("lib.rs")).expect("crate root should read");
    let identity =
        std::fs::read_to_string(src.join("identity.rs")).expect("identity source should read");
    let interaction = std::fs::read_to_string(src.join("interaction/mod.rs"))
        .expect("interaction source should read");
    let responder_scope = std::fs::read_to_string(src.join("responder/scope.rs"))
        .expect("responder scope source should read");
    let command_scope = std::fs::read_to_string(src.join("session/command_scope.rs"))
        .expect("session command scope source should read");

    assert!(
        identity.contains("pub struct Id(&'static str);")
            && lib.contains("mod identity;")
            && !lib.contains("pub mod identity;")
            && interaction.contains("pub use crate::identity::Id;")
            && !src.join("interaction/id.rs").exists(),
        "one lower declaration must own authored identity while interaction retains the established public projection"
    );
    assert!(
        responder_scope.contains("responder: Option<identity::Id>")
            && responder_scope.contains("kind: Kind")
            && !responder_scope.contains("focus: Option")
            && !responder_scope.contains("table"),
        "responder Scope must retain only route identity and traversal kind"
    );
    assert!(
        command_scope.contains("routing: responder::Scope")
            && command_scope.contains("focus: Option<Focus>")
            && command_scope.contains("table: Option<identity::Id>")
            && command_scope.contains("pub(crate) fn routing(self) -> responder::Scope")
            && command_scope.contains("Focus::table_cell_identity"),
        "session CommandScope must align the lower route with UI facts used to realize services"
    );
    assert_source_patterns_absent(
        &src.join("responder"),
        &[
            "interaction::".to_owned(),
            "session::".to_owned(),
            "crate::table".to_owned(),
        ],
    );
}

#[test]
fn runtime_input_projects_lower_keyboard_vocabulary() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let lib = std::fs::read_to_string(src.join("lib.rs")).expect("crate root should read");
    let keyboard =
        std::fs::read_to_string(src.join("keyboard.rs")).expect("keyboard source should read");
    let input =
        std::fs::read_to_string(src.join("input/mod.rs")).expect("input source should read");

    assert!(
        keyboard.contains("pub enum Key")
            && keyboard.contains("pub struct Modifiers")
            && lib.contains("mod keyboard;")
            && !lib.contains("pub mod keyboard;")
            && input.contains("pub use crate::keyboard::{Key, Modifiers};")
            && !src.join("input/key.rs").exists(),
        "input must preserve its public key projection while one private lower owner declares keyboard facts"
    );
    assert!(
        input.contains("pub enum Input")
            && input.contains("Focus(session::Focus)")
            && input.contains("PointerDown(interaction::Target)")
            && input.contains("pub use outcome::Outcome;")
            && input.contains("pub use text_drop::TextDrop;"),
        "the remaining input module must describe runtime ingress and its outcome payloads"
    );

    for relative in [
        "command/registry.rs",
        "command/spec.rs",
        "keymap.rs",
        "interaction/mod.rs",
        "interaction/pointer.rs",
        "session/interaction/pointer.rs",
    ] {
        let source = std::fs::read_to_string(src.join(relative))
            .unwrap_or_else(|_| panic!("{relative} should read"));
        assert!(
            !source.contains("input::Key") && !source.contains("input::Modifiers"),
            "{relative} must consume lower keyboard facts without depending on runtime input"
        );
    }
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
fn palette_query_does_not_reimplement_text_input() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let palette = std::fs::read_to_string(src_dir.join("runtime/palette.rs"))
        .expect("palette runtime source should read");
    let projection = std::fs::read_to_string(src_dir.join("view/command_palette.rs"))
        .expect("palette projection source should read");

    for forbidden in [
        "document::SelectAll",
        "document::Copy",
        "document::Cut",
        "document::Paste",
        "text::Edit",
    ] {
        assert!(
            !palette.contains(forbidden),
            "palette runtime must not implement standard text behavior: {forbidden}"
        );
    }
    assert!(
        projection.contains("Node::text_box_state") && projection.contains("TextBox::new(query)"),
        "the palette query must remain an ordinary standard text box projection"
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
        interaction_mod.contains("pub(crate) mod pointer;")
            && interaction_mod.contains("pub(crate) use pointer::Pointer;")
            && !interaction_mod.contains("pub(crate) use pointer::{"),
        "Pointer should be the sole parent projection while supporting concepts stay namespaced"
    );
    assert!(
        !pointer.contains("pub enum PressIntent"),
        "press intent should stay crate-internal"
    );
    for pattern in [
        "pointer_down_with_intent",
        "intent: interaction::PressIntent",
        "intent: interaction::pointer::PressIntent",
        "PointerDown {",
    ] {
        assert!(
            !input_mod.contains(pattern),
            "public input should expose named pointer gestures, not internal press intent: {pattern}"
        );
    }
}

#[test]
fn resolved_press_is_the_one_cursor_semantics_owner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let runtime_pointer = std::fs::read_to_string(src.join("runtime").join("pointer.rs"))
        .expect("runtime pointer source should read");
    let runtime_access = std::fs::read_to_string(src.join("runtime").join("access.rs"))
        .expect("runtime access source should read");
    let interaction_pointer = std::fs::read_to_string(src.join("interaction").join("pointer.rs"))
        .expect("interaction pointer source should read");
    let platform_event = std::fs::read_to_string(src.join("platform").join("event.rs"))
        .expect("platform event source should read");
    let host_event = std::fs::read_to_string(src.join("host").join("event.rs"))
        .expect("host event source should read");
    let shell_event = std::fs::read_to_string(src.join("shell").join("event.rs"))
        .expect("shell event source should read");
    let pointer_vocabulary = std::fs::read_to_string(src.join("pointer").join("mod.rs"))
        .expect("pointer vocabulary should read");
    let native_window_path = src.join("platform").join("native").join("window.rs");
    let native_window =
        std::fs::read_to_string(&native_window_path).expect("native window source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    assert!(
        runtime_pointer.contains("struct ResolvedPress")
            && runtime_pointer.contains("enum PressAdmission")
            && runtime_pointer.contains("pub(super) fn resolve_press(")
            && runtime_pointer.contains("if admission == PressAdmission::Target")
            && runtime_pointer.matches(".resolve_press(").count() >= 6
            && runtime_access.contains("self.resolve_press("),
        "move, down, up, drag, leave, modifiers, and presentation must share one prospective press resolver"
    );
    assert_eq!(
        runtime_pointer.matches("pointer::Cursor::Text").count(),
        1,
        "Text must have one logical selection site"
    );
    assert_eq!(
        runtime_pointer
            .matches("pointer::Cursor::ResizeHorizontal")
            .count(),
        1,
        "horizontal resize must have one logical selection site"
    );
    assert!(
        interaction_pointer.contains("cursor: pointer::Cursor")
            && interaction_pointer.contains("pub(crate) fn cursor(&self) -> pointer::Cursor")
            && runtime_pointer.contains(".map(interaction::pointer::Capture::cursor)")
            && !runtime_pointer.contains("captured_kind")
            && !runtime_pointer.contains("capture.target().kind"),
        "capture must preserve resolved cursor meaning instead of inferring it from target kind"
    );

    let modifier_body = runtime_pointer
        .split("pub(crate) fn pointer_modifiers_changed(")
        .nth(1)
        .and_then(|source| source.split("pub fn pointer_down_at(").next())
        .expect("stationary modifier handler should have a bounded body");
    assert!(
        modifier_body.contains("self.resolve_press(")
            && modifier_body.contains("self.set_pointer_cursor(")
            && modifier_body.contains("presented_layout(window)")
            && !modifier_body.contains("request_invalidation")
            && !modifier_body.contains("handle_view")
            && !modifier_body.contains("apply_window_update"),
        "modifier changes may update cursor truth but must not request or prepare a frame"
    );
    assert!(
        platform_event
            .matches("WinitWindowEvent::ModifiersChanged")
            .count()
            == 2
            && host_event.contains("shell::Event::PopupModifiersChanged")
            && shell_event.contains("self.pointer_modifiers_changed(window, modifiers)"),
        "parent and popup modifier events must reach the retained pointer clock"
    );
    assert!(
        runtime_pointer.contains("self.presented_layout(window)")
            && runtime_access.contains("self.presented_geometry.insert(")
            && runtime_access.contains("self.resolve_press("),
        "cursor resolution must consume last-presented geometry, including after a successful receipt"
    );

    assert!(
        pointer_vocabulary.contains("Default,")
            && pointer_vocabulary.contains("Text,")
            && pointer_vocabulary.contains("ResizeHorizontal,")
            && pointer_vocabulary
                .split("pub enum Cursor")
                .nth(1)
                .and_then(|source| source.split('}').next())
                .is_some_and(|body| body
                    .lines()
                    .filter(|line| line.trim().ends_with(','))
                    .count()
                    == 3),
        "cursor vocabulary must contain only demonstrated resolved-press species"
    );
    assert!(
        native_window.contains("pointer::Cursor::Default => CursorIcon::Default")
            && native_window.contains("pointer::Cursor::Text => CursorIcon::Text")
            && native_window.contains("pointer::Cursor::ResizeHorizontal => CursorIcon::EwResize"),
        "the native window adapter must exhaustively map the semantic cursor vocabulary"
    );
    assert_pattern_only_in(&src.join("platform"), "CursorIcon::", &native_window_path);

    for path in [
        src.join("view").join("node"),
        src.join("widget"),
        src.join("scene"),
    ] {
        assert_source_patterns_absent(&path, &["pointer::Cursor".to_owned()]);
    }
    for path in [
        src.join("layout").join("frame.rs"),
        src.join("interaction").join("target.rs"),
    ] {
        let source = std::fs::read_to_string(&path).expect("semantic data source should read");
        assert!(
            !source.contains("pointer::Cursor"),
            "{} must retain meaning rather than application cursor assignment",
            path.display()
        );
    }
    assert_source_patterns_absent(
        &src,
        &[
            format!("{}{}", "Pointer", "Plan"),
            format!("{}{}", "ResolvedPointer", "Plan"),
            format!("{}{}", "Pointer", "Capability"),
            format!("{}{}", "Pointer", "Affordance"),
            format!("{}{}", "Cursor", "Cue"),
            format!("{}{}", "Pointer", "Behavior"),
            format!("{}{}", "Cursor", "Policy"),
            format!("{}{}", "hit_promises_", "text_edit"),
            format!("{}{}", "pointer_cursor_", "for_hit"),
        ],
    );
    assert!(
        master.contains("one private `ResolvedPress`")
            && master.contains("`PressAdmission` determines")
            && master.contains("Applications do not assign cursors")
            && master.contains("does not parse, validate, resolve commands")
            && master.contains("physical surface, and modifiers as truth"),
        "master doctrine must retain press admission, pure hover, application boundaries, and pointer clocks"
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
fn frame_content_is_the_single_role_payload_representation() {
    let frame = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("layout")
            .join("frame.rs"),
    )
    .expect("layout frame module should read");
    let fields = frame
        .split("pub(crate) struct Frame {")
        .nth(1)
        .expect("Frame declaration should exist")
        .split("\n}")
        .next()
        .expect("Frame fields should end");

    assert!(
        fields.contains("content: FrameContent"),
        "Frame must carry one typed role payload"
    );
    for displaced in [
        "role: view::Role",
        "checkbox: Option<view::Checkbox>",
        "radio: Option<view::Radio>",
        "slider: Option<view::Slider>",
        "slider_track_rect: Option<Rect>",
        "text_area: Option<view::TextArea>",
        "text_area_layout: Option<text::Area>",
        "text_box: Option<view::TextBox>",
        "text_box_layout: Option<text::Field>",
        "text_box_text_rect: Rect",
        "viewport: Option<Viewport>",
        "shortcut_display: Option<Vec<ShortcutPart>>",
        "shortcut_width: Option<i32>",
    ] {
        assert!(
            !fields.contains(displaced),
            "legacy Frame payload field must stay absent: {displaced}"
        );
    }
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
fn theme_owns_the_framework_default_canvas_color() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let window_dir = root.join("src/window");
    let theme =
        std::fs::read_to_string(root.join("src/theme/mod.rs")).expect("theme source should read");
    let scene =
        std::fs::read_to_string(root.join("src/scene/mod.rs")).expect("scene source should read");
    let window =
        std::fs::read_to_string(window_dir.join("mod.rs")).expect("window module should read");
    let defaults_path = window_dir.join("defaults.rs");
    let defaults = std::fs::read_to_string(&defaults_path).expect("window defaults should read");
    let kind =
        std::fs::read_to_string(window_dir.join("kind.rs")).expect("window kind should read");
    let options =
        std::fs::read_to_string(window_dir.join("options.rs")).expect("window options should read");
    let control_gallery =
        std::fs::read_to_string(root.join("examples/control_gallery/app/view.rs"))
            .expect("control gallery view should read");
    let glass_tuner = std::fs::read_to_string(root.join("examples/glass_tuner/app/view.rs"))
        .expect("glass tuner view should read");
    let master = std::fs::read_to_string(root.join("docs/master_design.md"))
        .expect("master design should read");
    let slots = std::fs::read_to_string(root.join("tools/one_way_slots.json"))
        .expect("one-way slot map should read");

    assert!(
        theme.contains(
            "pub(crate) const DEFAULT_CANVAS_COLOR: scene::Color = scene::Color::rgb(17, 18, 20)"
        ) && theme.contains("canvas: DEFAULT_CANVAS_COLOR"),
        "theme must own and consume the framework default canvas token"
    );
    assert!(
        !scene.contains("DEFAULT_CLEAR")
            && scene.contains("theme::DEFAULT_CANVAS_COLOR")
            && defaults.contains(
                "pub const DEFAULT_CANVAS_COLOR: color::Color = theme::DEFAULT_CANVAS_COLOR"
            )
            && window.contains("pub use defaults::{DEFAULT_CANVAS_COLOR, DEFAULT_TITLE};")
            && window.contains("pub use kind::Kind;")
            && window.contains("pub use options::Options;")
            && kind.contains("pub enum Kind")
            && options.contains("pub struct Options")
            && !options.contains("pub enum Kind")
            && !window.contains("Options as")
            && !window.contains("Kind as"),
        "scene and window defaults must project the theme-owned token"
    );
    assert_pattern_only_in(&window_dir, "theme", &defaults_path);
    assert!(
        slots.contains("\"src/window/defaults.rs\": \"facade\"")
            && slots.contains("\"src/window/options.rs\": \"facade\"")
            && !slots.contains("\"src/window/kind.rs\": \"facade\""),
        "the gauge must split application options/defaults from lower window kind"
    );
    for (name, source) in [
        ("control gallery", control_gallery),
        ("glass tuner", glass_tuner),
    ] {
        assert!(
            source.contains("window::DEFAULT_CANVAS_COLOR")
                && !source.contains("scene::Color::rgb(17, 18, 20)"),
            "{name} must consume, not recompute, the framework default canvas"
        );
    }
    assert!(
        theme.contains("root: scene::Color::rgb(17, 18, 20)"),
        "theme root remains an independent visual token despite equal default bytes"
    );
    assert!(
        master.contains("Theme also owns the framework default canvas color"),
        "master design must name the default canvas owner"
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
fn clipboard_outcomes_flow_from_system_to_text_commands() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let public = std::fs::read_to_string(src.join("clipboard").join("mod.rs"))
        .expect("clipboard source should read");
    let system = std::fs::read_to_string(src.join("clipboard").join("system.rs"))
        .expect("system clipboard source should read");
    let command = std::fs::read_to_string(src.join("document").join("command.rs"))
        .expect("document command source should read");
    let document_target = std::fs::read_to_string(src.join("document").join("edit.rs"))
        .expect("document edit target source should read");
    let context = std::fs::read_to_string(src.join("context").join("mod.rs"))
        .expect("command context source should read");
    let text_editor = std::fs::read_to_string(src.join("text").join("edit").join("editor.rs"))
        .expect("text editor source should read");
    let focused = std::fs::read_to_string(
        src.join("runtime")
            .join("services")
            .join("text")
            .join("focused")
            .join("transfer.rs"),
    )
    .expect("focused text transfer source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    for signature in [
        "pub fn put<T: Payload>(&self, payload: &T) -> Result<()>",
        "pub fn get<T: Payload>(&self) -> Result<Option<T>>",
        "pub fn contains<T: Payload>(&self) -> Result<bool>",
    ] {
        assert!(
            public.contains(signature),
            "public clipboard operations must expose their outcome: {signature}"
        );
    }
    assert!(
        system.contains("pub(super) fn read_text(&mut self) -> Result<Option<String>>")
            && system.contains("pub(super) fn write_text(&mut self, text: &str) -> Result<()>"),
        "the system adapter must propagate read and write results"
    );
    assert!(
        document_target.contains("match clipboard.put(&clipboard::Text::new(selection))")
            && document_target.contains("self.apply_edit(text::Edit::insert(\"\"))")
            && document_target.contains("Err(_) => unavailable()")
            && focused.contains("Ok(()) => self.edit_response(text::Edit::Delete, true)")
            && focused
                .contains("Err(_) => Response::output(document::Outcome::unavailable_result())"),
        "Cut must mutate text only after a confirmed clipboard write"
    );
    assert!(
        document_target.contains("Ok(_) => unchanged()")
            && document_target.contains("Err(_) => unavailable()")
            && focused.contains("Ok(None)")
            && focused.contains("Err(_)"),
        "Paste must distinguish an empty clipboard from a failed read"
    );
    assert!(
        !src.join("text/edit/action.rs").exists()
            && !src.join("text/edit/clipboard.rs").exists()
            && !text_editor.contains("Clipboard")
            && !text_editor.contains("Action"),
        "text mutation must not retain the retired command/clipboard intermediate"
    );
    assert!(
        context.contains("clipboard: Option<Clipboard>")
            && context.contains("fn clipboard(&self) -> Option<&Clipboard>")
            && !context.contains(&format!("{}{}", "clipboard_", "mut")),
        "command context must transport one clipboard capability without per-operation clones"
    );
    assert_source_patterns_absent(&src.join("clipboard"), &["crate::".to_owned()]);
    assert!(
        command.matches("unwrap_or(true)").count() == 1
            && document_target.contains("Paste::availability(cx)")
            && focused.contains("document::Paste::availability(cx)")
            && !document_target.contains("unwrap_or(true)")
            && !focused.contains("unwrap_or(true)"),
        "Paste availability policy must have one command-owned computer"
    );
    assert!(
        master.contains("Cut deletes only after `Ok(())`")
            && master.contains("keeps empty distinct from failed"),
        "master design must retain clipboard outcome doctrine"
    );
}

#[test]
fn deferred_save_completion_carries_version_generation_and_atomic_write_owner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let document = std::fs::read_to_string(root.join("src").join("document").join("mod.rs"))
        .expect("document source should read");
    let save = std::fs::read_to_string(root.join("src").join("document").join("save.rs"))
        .expect("document save source should read");
    let event = std::fs::read_to_string(
        root.join("examples")
            .join("text_editor")
            .join("app")
            .join("event.rs"),
    )
    .expect("text editor event source should read");
    let target = std::fs::read_to_string(
        root.join("examples")
            .join("text_editor")
            .join("app")
            .join("target.rs"),
    )
    .expect("text editor target source should read");
    let runtime = std::fs::read_to_string(
        root.join("examples")
            .join("text_editor")
            .join("app")
            .join("runtime.rs"),
    )
    .expect("text editor runtime source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    for owner in [
        "pub struct Identity",
        "pub struct Version",
        "pub struct SaveSnapshot",
    ] {
        assert!(
            save.contains(owner),
            "document save owner must retain {owner}"
        );
    }
    for atomic_step in ["create_new(true)", "temporary.sync_all()", "replace_file("] {
        assert!(
            save.contains(atomic_step),
            "document saves must retain atomic step {atomic_step}"
        );
    }
    assert!(
        document.contains("let snapshot = self.save_snapshot()")
            && document.contains("snapshot.write_to(&path)?")
            && !document.contains("std::fs::write(&path, self.buffer.text())"),
        "Document::save_to must use the atomic snapshot owner"
    );
    assert!(
        event.contains("version: document::Version") && event.contains("generation: u64"),
        "save completion must carry document version and save generation"
    );
    assert!(
        target.contains("let snapshot = state.document.save_snapshot()")
            && target.contains("state.save_generation = generation")
            && target.contains("state.document.identity() == version.identity()")
            && !target.contains("std::fs::write(&path, text)"),
        "the example must write and validate the captured save identity"
    );
    assert!(
        runtime.contains("accepts_save_completion(cx.state(), version, generation)"),
        "stale save completions must be rejected before a state transaction"
    );
    assert!(
        master.contains("only the latest generation for the same identity")
            && master.contains("leaves newer edits dirty"),
        "master design must retain save completion identity doctrine"
    );
}

#[test]
fn state_change_reasons_do_not_import_command_contracts() {
    let state_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("state");

    assert_source_patterns_absent(&state_dir, &["crate::command".to_owned()]);
}

#[test]
fn command_failure_is_owned_below_the_public_error_facade() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let owner = src_dir.join("command").join("error.rs");
    let facade = src_dir.join("error.rs");
    let owner_source = std::fs::read_to_string(&owner).expect("command error owner should read");
    let facade_source = std::fs::read_to_string(&facade).expect("error facade should read");

    assert!(
        owner_source.contains("pub enum Error")
            && owner_source.contains("UnknownCommand")
            && owner_source.contains("AmbiguousTarget"),
        "command must own registration, routing, and invocation failures"
    );
    assert_eq!(
        facade_source.trim(),
        "pub use crate::command::Error;",
        "the established error module must remain a facade, not a second owner"
    );
    assert_imports_only_under_any(&src_dir, &[facade, owner], &["error"]);
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
fn layered_presentation_and_frame_names_remain_documented_twins() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    assert_source_patterns_absent(
        &src,
        &[
            ["Presentation", " as "].concat(),
            ["Frame", " as "].concat(),
        ],
    );
    for seam in [
        ("shell/presentation.rs", "scene::Presentation"),
        ("layout/frame.rs", "animation::Frame"),
        ("render/canvas.rs", "render::Frame"),
        ("runtime/presentation.rs", "layout::Frame"),
    ] {
        let source = std::fs::read_to_string(src.join(seam.0))
            .unwrap_or_else(|error| panic!("{} should read: {error}", seam.0));
        assert!(
            source.contains(seam.1),
            "layer-twin seam {} must keep {} qualified",
            seam.0,
            seam.1
        );
    }
    for verdict in [
        "legitimate layer twins",
        "view/scene/shell",
        "animation/layout/",
        "No import scope aliases either word",
    ] {
        assert!(
            master.contains(verdict),
            "master design must retain the census verdict: {verdict}"
        );
    }
}

#[test]
fn history_coalescing_is_scoped_to_the_runtime_target() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let history =
        std::fs::read_to_string(src.join("runtime").join("transaction").join("history.rs"))
            .expect("runtime history source should read");
    let command =
        std::fs::read_to_string(src.join("command/mod.rs")).expect("command source should read");
    let document = std::fs::read_to_string(src.join("document/command.rs"))
        .expect("document command source should read");
    let text_history = std::fs::read_to_string(src.join("text/edit/history.rs"))
        .expect("text history source should read");
    let master = std::fs::read_to_string(root.join("docs/master_design.md"))
        .expect("master design should read");

    for witness in [
        "active.window == window",
        "same_focus_target(active.focus, focus)",
    ] {
        assert!(
            history.contains(witness),
            "history groups must include their runtime target identity: {witness}"
        );
    }
    assert!(
        command.contains("DEFAULT_HISTORY_GROUP_COALESCE_WINDOW")
            && command.contains("coalesce_window: Duration"),
        "command HistoryGroup must own the generic coalescing default"
    );
    assert!(
        document
            .contains(".with_coalesce_window(text::edit::history::TYPING_UNDO_COALESCE_WINDOW)")
            && text_history.contains(
                "pub const TYPING_UNDO_COALESCE_WINDOW: Duration = Duration::from_millis(1000)"
            ),
        "document typing must carry the text-owned coalescing window"
    );
    assert!(
        history.contains("group.coalesce_window()")
            && !history.contains("HISTORY_GROUP_COALESCE_WINDOW")
            && !history.contains("Duration::from_millis(1000)"),
        "runtime must consume the declared window instead of recomputing it"
    );
    assert!(
        master.contains("runtime timeline and text buffer consume the same typing-pause fact"),
        "master design must name typing coalescing ownership"
    );
}

#[test]
fn per_window_state_owns_departed_cleanup() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let departed = std::fs::read_to_string(src.join("runtime").join("departed.rs"))
        .expect("runtime departed source should read");
    let window_departed = std::fs::read_to_string(src.join("window").join("departed.rs"))
        .expect("window departed source should read");
    let notification_window = std::fs::read_to_string(src.join("notification").join("window.rs"))
        .expect("window notification binding should read");
    let session_service = std::fs::read_to_string(src.join("session").join("service.rs"))
        .expect("session service should read");
    let runtime_context = std::fs::read_to_string(src.join("runtime").join("context.rs"))
        .expect("runtime context should read");
    let native_adapter =
        std::fs::read_to_string(src.join("platform").join("native").join("adapter.rs"))
            .expect("native adapter should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    for listener in [
        "self.layout_cache",
        "self.overlays",
        "self.animation_schedules",
        "self.visual_animations",
        "self.composition",
        "self.diagnostics",
        "self.gesture",
    ] {
        assert!(
            departed.contains(listener),
            "Departed publisher must retain registered listener {listener}"
        );
    }
    assert!(
        native_adapter.contains("Listener<app_window::Departed> for Native"),
        "the native popup manager must own its Departed purge"
    );
    assert!(
        window_departed.contains("pub struct Departed;")
            && !window_departed.contains("notification")
            && notification_window.contains("impl super::Notification for app_window::Departed")
            && notification_window.contains("type Payload = app_window::Id;")
            && notification_window.contains("const NAME: &'static str = \"window.departed\";"),
        "window must own the fact while notification owns its generic delivery binding"
    );
    assert!(
        !session_service.contains("remove_window") && !runtime_context.contains("remove_window"),
        "close paths must publish Departed instead of carrying cleanup checklists"
    );
    assert!(
        master.contains("Per-window state subscribes to `window::Departed` or documents why not"),
        "master design must state the per-window lifecycle rule"
    );
}

#[test]
fn snapshot_restore_has_one_transient_state_reset() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let snapshot = std::fs::read_to_string(root.join("runtime").join("snapshot.rs"))
        .expect("runtime snapshot source should read");

    assert_eq!(
        snapshot.matches("fn reset_transient_state").count(),
        1,
        "snapshot restore should have one transient-state reset owner"
    );
    for reset in [
        "self.composition.clear()",
        "self.animation_schedules.clear()",
        "self.visual_animations.clear()",
        "self.overlays.clear()",
        "self.layout_cache.clear()",
    ] {
        assert!(
            snapshot.contains(reset),
            "snapshot restore must discard transient presentation state: {reset}"
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
fn scene_owns_refraction_constraints_before_paint_projection() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scene_material = std::fs::read_to_string(root.join("src/scene/material.rs"))
        .expect("scene material source should read");
    let render_scene = std::fs::read_to_string(root.join("src/render/scene.rs"))
        .expect("renderer scene projection should read");
    let paint =
        std::fs::read_to_string(root.join("src/paint/mod.rs")).expect("paint source should read");
    let effects = std::fs::read_to_string(root.join("src/render/filter/effects.rs"))
        .expect("filter effects source should read");
    let master = std::fs::read_to_string(root.join("docs/master_design.md"))
        .expect("master design should read");

    assert!(
        scene_material.contains("const MAX_DISPLACEMENT: f32 = 4.0")
            && scene_material.contains("displacement.clamp(0.0, Self::MAX_DISPLACEMENT)"),
        "scene Refraction must own its domain constraints"
    );
    assert!(
        render_scene.contains("let refraction = refraction.clamped();"),
        "the scene-to-paint bridge must resolve refraction before projection"
    );
    assert!(
        !paint.contains("impl Refraction {") && !paint.contains("MAX_DISPLACEMENT"),
        "paint Refraction is a projection and must not recompute scene constraints"
    );
    assert!(
        !effects.contains(".clamp(") && !effects.contains(".max("),
        "renderer uniform lowering must forward resolved refraction values"
    );
    assert!(
        master.contains("Scene material values own their semantic constraints"),
        "master design must name the refraction constraint owner"
    );
}

#[test]
fn scene_no_longer_exposes_generic_filter_primitives() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let scene_mod =
        std::fs::read_to_string(root.join("scene").join("mod.rs")).expect("scene mod should read");
    let scene_primitive = std::fs::read_to_string(root.join("scene").join("primitive.rs"))
        .expect("scene primitive source should read");
    let render_scene = std::fs::read_to_string(root.join("render").join("scene.rs"))
        .expect("renderer scene projection should read");

    for source in [&scene_mod, &scene_primitive, &render_scene] {
        assert!(
            !source.contains("Primitive::Filter"),
            "scene-level filter primitive must not return after Pane"
        );
        assert!(
            !source.contains("scene::Filter"),
            "renderer scene projection must not carry a generic filter bridge after Pane"
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
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let popup = std::fs::read_to_string(root.join("platform").join("native").join("popup.rs"))
        .expect("native popup source should read");
    let contract =
        std::fs::read_to_string(root.join("popup.rs")).expect("popup contract source should read");

    assert!(
        contract.contains("pub(crate) struct Geometry {")
            && contract.matches("geometry: Geometry,").count() == 2
            && !contract.contains("#[allow(clippy::too_many_arguments)]")
            && popup.contains("crate::popup::Geometry::new("),
        "the selected host must deliver one namespaced realized-geometry value without a flattened constructor"
    );

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
fn native_cursor_routing_owns_the_physical_pointer_host() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let native = root.join("src").join("platform").join("native");
    let state =
        std::fs::read_to_string(native.join("mod.rs")).expect("native state source should read");
    let window = std::fs::read_to_string(native.join("window.rs"))
        .expect("native window source should read");
    let runner = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("runner")
            .join("native.rs"),
    )
    .expect("native runner source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    assert!(
        state.contains("cursor_hosts: HashMap<app_window::Id, CursorHost>")
            && state.contains("cursor_values: HashMap<app_window::Id, pointer::Cursor>"),
        "native must separate desired logical cursor from its physical host"
    );
    assert!(
        runner.contains(".route_cursor_host_event(raw_window, event)")
            && window.contains("CursorHost::Popup(key)")
            && window.contains("CursorHost::Outside")
            && window
                .contains("self.apply_cursor_to_host(parent, previous, pointer::Cursor::Default)")
            && window.contains("self.apply_cursor_to_host(parent, next, cursor)"),
        "raw events must reset the old physical host and apply the stored cursor to the new one"
    );
    assert!(
        master.contains("logical cursor value separate")
            && master.contains("physical window currently under the pointer")
            && !master.contains("application currently targets framework windows"),
        "master design must retain native cursor-host ownership"
    );
}

#[test]
fn ime_caret_geometry_follows_the_physical_text_host() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let frame = std::fs::read_to_string(src.join("layout").join("frame.rs"))
        .expect("layout frame source should read");
    let paint = [
        std::fs::read_to_string(src.join("scene").join("paint").join("text_area.rs"))
            .expect("text-area paint source should read"),
        std::fs::read_to_string(src.join("scene").join("paint").join("text_box.rs"))
            .expect("text-box paint source should read"),
    ]
    .join("\n");
    let runtime = std::fs::read_to_string(src.join("runtime").join("presentation.rs"))
        .expect("runtime presentation source should read");
    let platform = std::fs::read_to_string(src.join("platform").join("mod.rs"))
        .expect("platform source should read");
    let native_ime = std::fs::read_to_string(src.join("platform").join("native").join("ime.rs"))
        .expect("native IME source should read");
    let event = std::fs::read_to_string(src.join("platform").join("event.rs"))
        .expect("platform event source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    assert!(
        frame.contains("pub(crate) fn text_caret_rect(&self) -> Option<Rect>")
            && paint.matches("frame.text_caret_rect()").count() == 2,
        "caret paint and IME projection must consume one layout-owned rectangle"
    );
    assert!(
        runtime.contains("layout.text_caret_rect()")
            && runtime.contains("ime::Target::popup(layer.id(), area, layer.bounds())")
            && runtime.contains("layer.backend() == crate::overlay::Backend::NativePopup"),
        "runtime must project focused caret geometry through the resolved overlay host"
    );
    let popup_sync = platform
        .find("self.backend.present_overlay_popups")
        .expect("platform should synchronize popups");
    let ime_sync = platform[popup_sync..]
        .find("self.backend.set_ime(context, *update)")
        .expect("platform should apply IME projection after popup sync");
    assert!(
        ime_sync > 0,
        "IME host must exist before its cursor area is applied"
    );
    assert!(
        native_ime.contains("popup.host.window.set_ime_allowed(false)")
            && native_ime.contains("parent_window.set_ime_allowed(true)")
            && native_ime.contains("matches!(host, ImeHost::Popup(_))")
            && native_ime.contains("window.set_ime_cursor_area(target.area())"),
        "native IME routing must retain parent input authority while moving popup geometry"
    );
    let popup_adapter = event
        .split("pub(crate) fn popup_window_event")
        .nth(1)
        .expect("popup event adapter should exist");
    assert!(
        popup_adapter.contains("WinitWindowEvent::Ime(ime) => ime_window_event(ime)?"),
        "popup IME events must return to the logical parent input model"
    );
    assert!(
        master.contains("one geometry used")
            && master.contains("caret paint and IME placement")
            && master.contains("popup-local coordinates for a native floating panel"),
        "master design must retain IME geometry and host ownership"
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
    assert!(
        native_window.contains("CornerPreference::Round")
            && native_window.contains("CornerPreference::DoNotRound")
            && native_window.contains("composition_backed")
    );
    assert!(native_window.contains("with_no_redirection_bitmap(mode.no_redirection_bitmap())"));
    assert!(native_window.contains("with_undecorated_shadow(!composition_backed)"));
    assert!(native_window.contains("with_has_shadow(true)"));
    assert!(!native_window.contains("with_no_redirection_bitmap(true)"));
    assert!(!native_window.contains("with_no_redirection_bitmap(false)"));
    assert!(!native_window.contains("with_has_shadow(false)"));
    assert!(
        native_mod.contains("impl Drop for PopupHost")
            && native_mod.contains("remove_popup_subclass"),
        "popup-host drop must remove the subclass before the HWND is released"
    );
    assert!(
        popup.contains("self.popups.remove(&key)")
            && adapter.contains("self.popups.remove(&key)")
            && adapter.contains("self.popup_pool.remove(window)")
            && adapter.contains("self.popup_prewarm.remove(window)"),
        "stale sessions and parent-close cleanup must either pool or drop every popup host"
    );
}

#[test]
fn popup_pool_reuses_hosts_without_reusing_sessions() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let native_mod = std::fs::read_to_string(root.join("src/platform/native/mod.rs"))
        .expect("native platform source should read");
    let popup = std::fs::read_to_string(root.join("src/platform/native/popup.rs"))
        .expect("native popup source should read");
    let surface = std::fs::read_to_string(root.join("src/platform/native/surface.rs"))
        .expect("native surface source should read");

    assert!(
        native_mod.contains("Vec<PopupHost>")
            && !native_mod.contains("Vec<PopupWindow>")
            && native_mod.contains("struct PopupHost")
            && native_mod.contains("struct PopupWindow")
            && popup.contains("popup_pool_capacity")
            && popup.contains("(*capacity).max(depth)"),
        "the pool must retain reusable infrastructure, never semantic popup sessions"
    );
    assert!(
        popup.contains("let popup = PopupWindow::new(host, lifecycle_epoch, generation)")
            && popup.contains("let host = popup.into_host()")
            && popup.contains("self.raw_popups.remove(&popup.host.window.raw_id())")
            && popup.contains("self.release_ime_from_popup(key)")
            && popup.contains("popup.host.window.set_ime_allowed(false)")
            && popup.contains("popup.host.window.set_cursor(pointer::Cursor::Default)")
            && popup.contains("popup.host.window.hide_popup_before_teardown()")
            && popup.contains("popup.presentation_mode == mode")
            && popup.contains("popup.window.scale_factor() - scale_factor"),
        "acquisition must mint a fresh session and retirement must make the host inert before pooling"
    );
    assert!(
        native_mod
            .contains("first_present: PopupFirstPresentTrace::new(lifecycle_epoch, generation)")
            && popup.contains("popup.first_present = PopupFirstPresentTrace::new(now, generation)"),
        "fresh-session construction, not host reset, must own presentation receipts"
    );
    assert!(
        popup.contains("fn advance_popup_prewarm(")
            && surface.contains("PopupPrewarmState::Armed")
            && popup.contains("PopupPrewarmState::Scheduled")
            && popup.contains("prepare_popup_first_present()")
            && popup.contains("composition.prewarm_material()")
            && popup.contains("self.schedule_poll_request()"),
        "root-host prewarming must begin only after a stable parent frame and advance from an idle poll through a composition receipt"
    );
}

#[test]
fn windows_native_popup_material_prefers_dx12_without_overriding_explicit_choice() {
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
            && native_surface.contains("context::Backends::from_env()")
            && native_surface.contains("native_backend_attempts(explicit)"),
        "WGPU_BACKEND must remain authoritative over the native preference ladder"
    );
    assert_eq!(
        native_surface
            .matches("fn native_backend_attempts(")
            .count(),
        1,
        "native backend selection must have one policy owner"
    );
    assert!(
        native_surface.contains("first: context::Backends::dx12()")
            && native_surface.contains("fallback: Some(context::Backends::all())")
            && !native_surface.contains("native backend attempts are never empty"),
        "implicit Windows selection should try tenancy-capable DX12 then retain the ordinary fallback set"
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
            "Self::RedirectedFallback => render::surface::CompositeAlphaPreference::PreMultiplied"
        ) && native_mod.contains("PopupPresentationMode::RedirectedFallback.realization_for")
            && native_mod.contains("PopupMaterialRealization::WindowsAccentAcrylic"),
        "redirected Vulkan popups may realize OS material when the surface reports premultiplied alpha"
    );
    for phrase in [
        "WGPU_BACKEND",
        "host-backdrop region container",
        "SetWindowCompositionAttribute",
        "RedirectedFallback",
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

    assert!(native_mod.contains("type PopupAccentState = SysApplicator"));
    assert!(native_mod.contains("POPUP_SYS_SETTLE_DELAY"));
    assert!(native_mod.contains("Duration::from_millis(150)"));
    assert!(popup.contains("popup.accent.set_desired(accent, now)"));
    assert!(popup.contains("apply_due_popup_accents"));
    assert_eq!(
        popup.matches("set_popup_accent_material(accent)").count(),
        1,
        "popup presentation must not call the Windows accent API at tint-sample rate"
    );
    assert!(
        adapter.contains("let redraw_parents = self.apply_due_popup_accents")
            && adapter.contains("self.request_popup_parent_redraws(&redraw_parents)")
            && platform.contains("self.backend.maintain(context)?"),
        "backend maintenance must drain pending accents and redraw their parents"
    );
    assert!(
        master.contains("OS-side state crosses at the narrowest honest rate")
            && master.contains("legacy accent bridge"),
        "native material doctrine should name the legacy settle-rate realization"
    );
}

#[test]
fn popup_border_has_one_theme_datum_for_scene_and_windows() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let theme = std::fs::read_to_string(root.join("src").join("theme").join("mod.rs"))
        .expect("theme source should read");
    let theme_toml = std::fs::read_to_string(root.join("src").join("theme").join("toml.rs"))
        .expect("theme TOML source should read");
    let scene =
        std::fs::read_to_string(root.join("src").join("scene").join("paint").join("mod.rs"))
            .expect("scene paint source should read");
    let runtime = std::fs::read_to_string(root.join("src").join("runtime").join("presentation.rs"))
        .expect("runtime presentation source should read");
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
    let sys = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("sys")
            .join("mod.rs"),
    )
    .expect("native sys source should read");
    let windows = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("sys")
            .join("windows.rs"),
    )
    .expect("Windows sys source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    assert!(
        theme.contains("pub(crate) border: scene::Color")
            && theme_toml.contains("floating-panel.border")
            && scene.contains("Outline::new(frame.rect(), panel.border)"),
        "FloatingPanel.border must own both theme persistence and in-frame outline paint"
    );
    assert!(
        runtime.contains("layer.popup_border()") && popup.contains("presentation.border()"),
        "native popup presentation must carry the same border datum"
    );
    let desire = popup
        .find("popup.border.set_desired(presentation.border(), now)")
        .expect("native popup should record desired border");
    let prepare = popup[desire..]
        .find("if !popup.presentation_prepared")
        .map(|offset| desire + offset)
        .expect("native popup should become presentable after border realization");
    assert!(
        popup[desire..prepare].contains("apply_popup_border(key, popup, reason)"),
        "the creation border must apply before concealed first presentation"
    );
    assert!(
        native_mod.contains("type PopupBorderState = SysApplicator")
            && popup.contains("apply_due_popup_borders")
            && windows.contains("DWMWA_BORDER_COLOR")
            && popup.contains("suppress_popup_border"),
        "redirected border changes stay settle-rate while composition suppresses the DWM copy"
    );
    assert!(
        sys.contains("crate::color::bbggrr(r, g, b)"),
        "Windows COLORREF byte order must live in the color owner"
    );
    assert!(
        master.contains("`FloatingPanel.border` remains the one popup border datum")
            && master.contains("`COLORREF` (`0x00BBGGRR`)"),
        "master design must retain popup border ownership and byte order"
    );
}

#[test]
fn sys_side_realizations_share_one_settle_applicator() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let native_mod = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("mod.rs"),
    )
    .expect("native module source should read");
    let settle = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("settle.rs"),
    )
    .expect("sys settle source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    for fact in [
        "desired: Option<T>",
        "applied: Option<T>",
        "desired_changed_at: Option<Instant>",
        "pub(super) fn due(",
    ] {
        assert!(
            settle.contains(fact),
            "SysApplicator must own realization fact {fact}"
        );
    }
    for client in [
        "type PopupGeometryState = SysApplicator<PopupGeometry>",
        "type PopupAccentState = SysApplicator<sys::PopupAccentMaterial>",
        "type PopupBorderState = SysApplicator<scene::Color>",
    ] {
        assert!(
            native_mod.contains(client),
            "sys realization must ride the shared applicator: {client}"
        );
    }
    assert!(
        native_mod.contains("state.due(now, Duration::ZERO, |_, _| true)")
            && native_mod.contains("accent_presence(applied) != accent_presence(desired)")
            && native_mod.contains("state.due(now, POPUP_SYS_SETTLE_DELAY, |_, _| false)"),
        "geometry, accent, and border should retain only their distinct due policy"
    );
    assert!(
        master.contains("`SysApplicator<T>`")
            && master.contains("continues to own desired/applied snapshots"),
        "master design must retain the shared sys applicator owner"
    );
}

#[test]
fn native_popup_first_present_is_visible_traced_and_compositor_synchronized() {
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
    let surface = std::fs::read_to_string(root.join("src").join("render").join("surface.rs"))
        .expect("render surface source should read");
    let windows = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("sys")
            .join("windows.rs"),
    )
    .expect("Windows native source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    let desired = popup
        .find("popup.accent.set_desired(accent, now)")
        .expect("popup presentation should record desired accent");
    let draw = popup[desired..]
        .find("renderer.draw")
        .map(|offset| desired + offset)
        .expect("popup presentation should draw after resolving material");
    let before_draw = &popup[desired..draw];

    assert!(
        before_draw.contains("popup_accent_due(&popup.accent, now)")
            && before_draw.contains("apply_popup_accent(key, popup, reason)"),
        "an already-due popup accent must apply before the imminent frame"
    );
    let prepare = before_draw
        .find("prepare_popup_first_present()")
        .expect("popup must become presentable while concealed before its first acquire");
    assert!(
        prepare < before_draw.len(),
        "concealed preparation must precede renderer draw/acquire"
    );
    let presented = popup[draw..]
        .find("record_presented(key, generation, timing)")
        .map(|offset| draw + offset)
        .expect("the current frame must be recorded after draw");
    let expose = popup[presented..]
        .find("expose_popup_after_present()")
        .map(|offset| presented + offset)
        .expect("popup must expose only after a current present");
    assert!(
        draw < presented && presented < expose,
        "first visibility must order draw -> present -> expose"
    );
    assert!(
        popup.contains("if self.present_popup_overlay(context, presentation, now)?")
            && popup.contains("queue_popup_parent_redraw(&mut redraw_parents")
            && native_mod.contains("AwaitingFirst")
            && native_mod.contains("AwaitingConfirmation")
            && native_mod.contains("Complete"),
        "first-present lifecycle must retain one evidence-gated fallback redraw"
    );
    assert!(
        native_mod.contains("PopupFirstPresentTrace") && !native_mod.contains("PopupFrameRecovery"),
        "fallback confirmation must remain finite, not become a retry budget"
    );
    for stage in ["created", "configured", "prepared-concealed", "exposed"] {
        assert!(
            popup.contains(&format!("first-present stage={stage}")),
            "first-present trace must record {stage}"
        );
    }
    for transition in [
        "\"acquire\"",
        "\"confirmation-acquire\"",
        "\"synchronized\"",
        "\"visibility-sync-failed\"",
        "\"confirmation-synchronized\"",
        "\"confirmation-sync-failed\"",
    ] {
        assert!(
            popup.contains(transition),
            "first-present trace must retain transition {transition}"
        );
    }
    assert!(
        popup.contains("super::sys::synchronize_popup_presentation()")
            && windows.contains("DwmFlush()")
            && windows.contains("DWMWA_CLOAK")
            && windows.contains("set_popup_cloaked(window, true)?")
            && windows.contains("set_popup_cloaked(window, false)")
            && popup.contains("popup.first_present.needs_redraw()"),
        "the current frame must cross the DWM barrier while concealed, while a no-present outcome retries"
    );
    assert!(
        !popup.contains("(\"presented\", true)"),
        "a successful first present must not request a blind confirmation frame"
    );
    for outcome in [
        "Suboptimal",
        "Outdated",
        "Timeout",
        "Occluded",
        "Validation",
    ] {
        assert!(
            surface.contains(outcome),
            "surface acquire trace must distinguish {outcome}"
        );
    }
    let close_stale = popup
        .find("self.close_stale_popups(&synchronized_parents, &active)")
        .expect("popup synchronization should close stale entries");
    let drain_accents = popup
        .find("self.apply_due_popup_accents(now)")
        .expect("popup synchronization should drain due accents");
    assert!(
        close_stale < drain_accents,
        "stale popup removal must precede deferred accent calls"
    );
    for phrase in [
        "Every popup show cycle begins concealed",
        "`DWMWA_CLOAK`",
        "first user-visible pixels therefore follow a current",
        "unbounded retry budget",
    ] {
        assert!(
            master.contains(phrase),
            "native popup first-show doctrine should mention {phrase}"
        );
    }
}

#[test]
fn native_popup_presentation_does_not_wait_behind_the_parent_frame() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let platform = std::fs::read_to_string(root.join("src").join("platform").join("mod.rs"))
        .expect("platform source should read");
    let popup = platform
        .find("self.backend.present_overlay_popups")
        .expect("platform should present native popups");
    let parent = platform
        .find("for presentation in work.presentations()")
        .expect("platform should present parent windows");

    assert!(
        popup < parent,
        "an independently presentable popup must not pay the parent renderer's frame cost before visibility"
    );
}

#[test]
fn composition_popup_closeout_has_one_geometry_edge_and_timeline_owner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let renderer = std::fs::read_to_string(root.join("src/render/renderer.rs"))
        .expect("renderer source should read");
    let render_scene = std::fs::read_to_string(root.join("src/render/scene.rs"))
        .expect("renderer scene projection should read");
    let composition = std::fs::read_to_string(root.join("src/platform/native/composition.rs"))
        .expect("composition source should read");
    let native_window = std::fs::read_to_string(root.join("src/platform/native/window.rs"))
        .expect("native window source should read");
    let popup = std::fs::read_to_string(root.join("src/platform/native/popup.rs"))
        .expect("native popup source should read");
    assert!(
        !renderer.contains("backdrop_layers.is_empty()")
            && renderer.contains("paint::GlassBase::Transparent")
            && renderer.contains("paint::GlassBase::Fallback"),
        "resolved material truth, not an empty operation list, must select the popup base"
    );
    assert!(
        render_scene.contains("paint::shadow_visual_bounds")
            && render_scene.contains("paint::union_visual_bounds")
            && !render_scene.to_ascii_lowercase().contains("shadow_margin"),
        "popup visual reach must consume shared shadow bounds without an arbitrary margin"
    );
    assert!(
        composition.contains("project_shadow(recipe, silhouette, scale_factor)")
            && composition.contains("sync_shadow(projected_shadow)")
            && composition.contains("panel_offset_physical")
            && !composition.contains("panel_offset_dips"),
        "the desktop composition shadow must derive from the shared device-space material-region silhouette"
    );
    assert!(
        native_window.contains("with_undecorated_shadow(!composition_backed)")
            && native_window.contains("CornerPreference::DoNotRound")
            && popup.contains("suppress_popup_border"),
        "composition popups must suppress independent DWM shadow, corner, and border realization"
    );
    assert_eq!(
        composition.matches(".StartAnimation(").count(),
        1,
        "the composition root must be the only animated popup opacity owner"
    );
    assert!(
        composition.contains("self.root")
            && composition.contains("HSTRING::from(\"Opacity\")")
            && !composition.contains("DwmFlush")
            && !composition.contains("synchronize_popup_presentation"),
        "composition animation must target the root without borrowing the first-present barrier as a clock"
    );
}

#[test]
fn composition_popup_readiness_is_receipted_and_generation_bound() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let composition = std::fs::read_to_string(root.join("src/platform/native/composition.rs"))
        .expect("composition source should read");
    let popup = std::fs::read_to_string(root.join("src/platform/native/popup.rs"))
        .expect("native popup source should read");
    let popup_production = popup
        .split("#[cfg(test)]")
        .next()
        .expect("popup production source should precede tests");

    assert!(
        composition.contains("GetCommitBatch(CompositionBatchTypes::Effect)")
            && composition.contains("material_generation")
            && composition.contains("commit.batch.IsEnded()")
            && !composition.contains("CompositionBatchTypes::None"),
        "material readiness must consume the current generation's Effect receipt"
    );
    assert!(
        composition.contains("self.compositor.RequestCommitAsync()")
            && composition.contains("prepared-root commit completed")
            && composition.contains("entrance.take_committed(generation)")
            && popup_production.contains("entrance_readiness(popup.generation.serial())")
            && popup_production.contains("awaits prepared-root commit before exposure"),
        "a concealed entrance must receive its current generation's root-property commit before exposure"
    );
    assert!(
        popup_production.contains("for barrier in 1..=2")
            && popup_production.contains("material_readiness.mark_ready(generation)")
            && popup_production.contains("abandon_material_prewarm")
            && !popup_production.contains("thread::sleep")
            && !popup_production.contains("from_millis"),
        "exposure must use evidenced host frames, reject stale generations, and fall back without a readiness delay"
    );
}

#[test]
fn material_regions_derive_identity_and_provenance_at_pane_emission() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scene = std::fs::read_to_string(root.join("src").join("scene").join("mod.rs"))
        .expect("scene source should read");
    let region = std::fs::read_to_string(root.join("src").join("scene").join("region.rs"))
        .expect("material region source should read");
    let paint =
        std::fs::read_to_string(root.join("src").join("scene").join("paint").join("mod.rs"))
            .expect("scene paint source should read");

    assert!(
        paint.contains("scene.push_material_pane(")
            && paint.contains("frame.node_id()")
            && paint.contains("clip,"),
        "material request identity and inherited clip must derive where the retained frame emits its pane"
    );
    assert!(
        region.contains("id: composition::NodeId")
            && region.contains("clips: Vec<Clip>")
            && region.contains("opacity: f32")
            && scene.contains("material_regions: Vec<MaterialRegion>"),
        "the request must retain identity, clip provenance, opacity, and ordered collection ownership"
    );
    assert!(
        !region.contains("enumerate()"),
        "primitive position and traversal ordinals must never become material identity"
    );
    assert!(
        scene.contains("ghost.material_regions.clear()")
            && scene.contains("region.with_parent_opacity(opacity)")
            && scene.contains("region.translated(dx, dy)"),
        "ghosting, overlay opacity, and popup localization must project the same retained request"
    );
}

#[test]
fn native_popup_fade_uses_common_composition_owner_when_earned() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let presentation = std::fs::read_to_string(root.join("src/runtime/presentation.rs"))
        .expect("runtime presentation source should read");
    let overlay =
        std::fs::read_to_string(root.join("src/overlay.rs")).expect("overlay source should read");
    let popup = std::fs::read_to_string(root.join("src/platform/native/popup.rs"))
        .expect("native popup source should read");
    assert!(
        presentation.contains("let popup_scene = local.scene().clone()")
            && presentation.contains("layer.opacity()")
            && overlay.contains("opacity: f32")
            && !presentation.contains("with_material_opacity"),
        "native presentation must retain one root opacity source without baking it into material regions"
    );
    assert!(
        overlay.contains("struct RetiringPopup")
            && overlay.contains("lifecycle: Lifecycle::RetiringPopup")
            && overlay.contains("backend: Backend::NativePopup"),
        "native exit fade must retain the native surface rather than allocate a parent ghost"
    );
    assert!(
        overlay.contains("enum Lifecycle {")
            && overlay.contains("Live { state: State, elapsed: Duration }")
            && overlay.contains("Ghost { elapsed: Duration }")
            && overlay.contains("RetiringPopup { elapsed: Duration }")
            && overlay.contains("lifecycle: Lifecycle,")
            && !overlay.contains("state: Option<State>")
            && !overlay.contains("elapsed: Option<Duration>"),
        "resolved overlay species must make lifecycle state and elapsed time structurally valid"
    );
    assert!(
        popup.contains("composition.apply_fade(presentation.fade(), Instant::now())")
            && popup.contains("(!uses_composition).then(||")
            && popup.contains("append_scene_with_opacity("),
        "tenancy must project opacity at the common tree while legacy realization applies it once in paint"
    );
}

#[test]
fn actual_material_reports_alone_authorize_residual_subtraction() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime = std::fs::read_to_string(root.join("src/runtime/presentation.rs"))
        .expect("runtime presentation source should read");
    let overlay =
        std::fs::read_to_string(root.join("src/overlay.rs")).expect("overlay source should read");
    let popup = std::fs::read_to_string(root.join("src/platform/native/popup.rs"))
        .expect("native popup source should read");
    let windows = std::fs::read_to_string(root.join("src/platform/native/sys/windows.rs"))
        .expect("Windows popup source should read");

    assert!(
        runtime.contains("native_popup_request(layer.bounds())")
            && runtime.contains("let popup_scene = local.scene().clone()")
            && !runtime.contains("with_material_opacity")
            && !runtime.contains("opaque_fallback_scene"),
        "runtime must submit one intact localized request rather than forecast-filtered scenes"
    );
    assert!(
        !overlay.contains("opaque_fallback_scene")
            && popup.contains("MaterialRealizationReport::new")
            && popup.contains("resolve_material("),
        "the native boundary must report actual parts and delegate complement ownership to one resolver"
    );
    assert!(
        windows.contains("pub(super) fn set_popup_accent_material(")
            && windows.contains(") -> bool")
            && popup.contains("if popup.host.window.set_popup_accent_material(accent)"),
        "an attempted accent forecast must not be recorded as a successful realization"
    );
}

#[test]
fn windows_tenancy_is_earned_by_dx12_wrap_and_keeps_legacy_fallback() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo = std::fs::read_to_string(root.join("Cargo.toml")).expect("manifest should read");
    let composition = std::fs::read_to_string(root.join("src/platform/native/composition.rs"))
        .expect("composition owner should read");
    let popup = std::fs::read_to_string(root.join("src/platform/native/popup.rs"))
        .expect("native popup source should read");
    let surface = std::fs::read_to_string(root.join("src/platform/native/surface.rs"))
        .expect("native surface source should read");
    let render_surface = std::fs::read_to_string(root.join("src/render/surface.rs"))
        .expect("render surface source should read");

    assert!(
        cargo.contains("wgpu-hal = { version = \"29.0.3\"")
            && cargo.contains("windows = { version = \"0.62.2\""),
        "direct HAL and Windows bindings must match wgpu's pinned ABI family"
    );
    for receipt in [
        "CreateCompositionSurfaceForSwapChain",
        "CreateDesktopWindowTarget",
        "InsertAtTop(&content)",
        ".dx12()",
    ] {
        assert!(
            composition.contains(receipt),
            "single-HWND tenancy route is missing {receipt}"
        );
    }
    for receipt in [
        "SurfaceTargetUnsafe::CompositionVisual",
        "self.inner.as_hal::<wgpu_hal::api::Dx12>()",
    ] {
        assert!(
            render_surface.contains(receipt),
            "renderer-owned DX12 surface bridge is missing {receipt}"
        );
    }
    assert!(
        !composition.contains("transmute") && !composition.contains("from_raw"),
        "matching Windows versions must eliminate cross-version COM ownership transfer"
    );
    assert!(
        surface.contains("native_backend_attempts(explicit)")
            && surface.contains("first: context::Backends::dx12()")
            && surface.contains("fallback: Some(context::Backends::all())")
            && popup.contains("retaining legacy popup realization"),
        "DX12-first tenancy must preserve explicit overrides and a clean legacy fallback"
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
            && renderer.contains("windows_popup_support()")
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
fn renderer_alpha_conventions_have_one_blend_owner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let render = root.join("src").join("render");
    let alpha = std::fs::read_to_string(render.join("alpha.rs"))
        .expect("alpha convention owner should read");
    let quad =
        std::fs::read_to_string(render.join("quad.rs")).expect("quad renderer source should read");
    let filter_setup = std::fs::read_to_string(render.join("filter").join("setup.rs"))
        .expect("filter setup source should read");
    let filter_shader =
        std::fs::read_to_string(render.join("filter.wgsl")).expect("filter shader should read");
    let popup_pack = std::fs::read_to_string(render.join("popup_pack.rs"))
        .expect("popup pack source should read");
    let master = std::fs::read_to_string(root.join("docs").join("master_design.md"))
        .expect("master design should read");

    assert!(alpha.contains("FragmentOutput::Straight => Some(wgpu::BlendState::ALPHA_BLENDING)"));
    assert!(alpha.contains("wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING"));
    assert!(alpha.contains("FragmentOutput::Replace => None"));
    assert!(quad.contains("render::alpha::FragmentOutput::Straight"));
    assert_eq!(
        filter_setup
            .matches("render::alpha::FragmentOutput::Replace")
            .count(),
        5,
        "filter scratch/transform pipelines must replace associated RGBA"
    );
    assert_eq!(
        filter_setup
            .matches("render::alpha::FragmentOutput::Premultiplied")
            .count(),
        2,
        "filtered and pixel-aligned composite pipelines must use premultiplied source-over"
    );
    assert!(popup_pack.contains("render::alpha::FragmentOutput::Replace"));
    assert!(
        popup_pack.contains("pipelines.entry(output_format)")
            && !popup_pack.contains("ensure_pipeline")
            && !popup_pack.contains("pipeline should be initialized"),
        "popup pack pipelines must be initialized and consumed through one cache entry"
    );
    assert!(filter_shader.contains("return vec4<f32>(color.rgb * coverage, color.a * coverage);"));
    let renderer =
        std::fs::read_to_string(render.join("renderer.rs")).expect("renderer source should read");
    assert!(renderer.contains("premultiplied_group_witness_applies_opacity_once"));
    assert!(renderer.contains("(sample[3] - 0.25).abs()"));
    assert!(renderer.contains("(sample[0] - 0.25).abs()"));
    assert!(master.contains("The alpha pipeline has one convention at each stage:"));
    assert!(master.contains("readiness is not implementation"));

    for source in [&quad, &filter_setup, &popup_pack] {
        assert!(
            !source.contains("BlendState::ALPHA_BLENDING")
                && !source.contains("BlendState::PREMULTIPLIED_ALPHA_BLENDING"),
            "renderer pipelines must choose blend semantics through render::alpha"
        );
    }
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

    let render_surface = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("render")
            .join("surface.rs"),
    )
    .expect("render surface source should read");

    assert!(native_mod.contains("renderers: HashMap<render::surface::Format, render::Renderer>"));
    assert!(surface.contains("canvas().surface().render_format()"));
    assert!(
        surface.contains("renderers.entry(format)")
            && !surface.contains("ensure_renderer")
            && !surface.contains("renderer should exist"),
        "native renderer creation and consumption must share one format-keyed cache entry"
    );
    assert!(render_surface.contains("scene_format_for_surface_format(format)"));
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

    let lost = surface
        .find("Lost =>")
        .map(|start| &surface[start..])
        .expect("surface acquire should handle lost surfaces");
    assert!(
        lost.contains("self.reconfigure(render_context)")
            && lost.contains("AcquireOutcome::Lost")
            && !lost.contains("Err(Error::Lost)"),
        "surface loss must rebuild the surface configuration epoch and skip one frame instead of terminating the runtime"
    );
}

#[test]
fn windows_material_regions_are_keyed_projections_with_report_after_success() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let composition = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("composition.rs"),
    )
    .expect("Windows composition source should read");

    assert!(
        composition.contains("HashMap<composition::NodeId, RegionVisual>")
            && composition.contains("self.material_regions.insert(id, region)")
            && composition.contains(".get_mut(&id)")
            && composition.contains("self.material_regions.remove(&id)"),
        "retained declaring identity must own the active visual mapping independently from scene order, even when host infrastructure is recycled"
    );
    let apply = composition
        .find("region.apply(projected)")
        .expect("region projection should be applied");
    let order = composition[apply..]
        .find("children.InsertAtTop(&region.visual)")
        .map(|offset| apply + offset)
        .expect("successful region visuals should be placed in current scene order");
    let report = composition[order..]
        .find("MaterialRealizationReport::new")
        .map(|offset| order + offset)
        .expect("successful region realization should produce a report");
    assert!(
        apply < order && order < report,
        "reports must follow successful geometry and order projection"
    );
    assert!(
        composition.contains("render::scene::physical_rounded_rect")
            && !composition.contains("enumerate()"),
        "material realization must share raster projection and never derive identity from order"
    );
    assert!(
        !composition.contains("paint::") && !composition.contains("use crate::{composition, paint"),
        "native composition must consume the renderer's physical projection without private paint vocabulary"
    );
}

#[test]
fn windows_accent_bridge_is_absent_from_composition_tenancy() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let popup = std::fs::read_to_string(
        root.join("src")
            .join("platform")
            .join("native")
            .join("popup.rs"),
    )
    .expect("native popup source should read");

    assert!(
        popup.contains("if !uses_composition {")
            && popup.contains("recorded legacy native popup accent desire")
            && popup.contains("popup_accent_due(&popup.accent, now)"),
        "accent desire and application must be narrowed to the non-tenancy bridge"
    );
}

#[test]
fn menu_population_has_one_owner_and_examples_declare_only_meaning() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let population = std::fs::read_to_string(root.join("src/command/population.rs"))
        .expect("population owner should read");
    let palette = std::fs::read_to_string(root.join("src/runtime/palette.rs"))
        .expect("palette adapter should read");
    let context = std::fs::read_to_string(root.join("src/runtime/context_menu.rs"))
        .expect("context adapter should read");
    let presentation = std::fs::read_to_string(root.join("src/runtime/presentation.rs"))
        .expect("presentation adapter should read");
    let widget_ui = std::fs::read_to_string(root.join("src/widget/ui.rs"))
        .expect("widget UI source should read");

    assert!(
        population.contains("pub(crate) struct Population")
            && population.contains("fn palette_candidates")
            && population.contains("fn context_candidates")
            && population.contains("fn bar_candidates")
            && population.contains("fn standard_bar"),
        "one population owner must hold the shared discovery and resolution machinery"
    );
    assert!(
        palette.contains("population.palette_candidates()")
            && context.contains("population.context_candidates(binding, targets)")
            && presentation.contains("population.standard_bar("),
        "palette, context, and bar must remain policy adapters into the same owner"
    );
    assert!(
        !root.join("src/command/surface.rs").exists(),
        "the superseded parallel population owner must remain absent"
    );
    assert!(
        widget_ui.contains("pub fn menu_bar(") && widget_ui.contains("pub fn standard_menu_bar("),
        "automatic population removes required authorship without removing the authored escape hatch"
    );

    for relative in [
        "examples/text_editor/app/view.rs",
        "examples/control_gallery/app/view.rs",
    ] {
        let source = std::fs::read_to_string(root.join(relative))
            .unwrap_or_else(|_| panic!("{relative} should read"));
        assert_eq!(
            source.matches("ui.standard_menu_bar();").count(),
            1,
            "{relative} should opt in exactly once"
        );
        for authored in ["ui.menu_bar(", "ui.menu(", "ui.separator()"] {
            assert!(
                !source.contains(authored),
                "{relative} must not re-author conventional topology with {authored}"
            );
        }
    }

    let tuner = std::fs::read_to_string(root.join("examples/glass_tuner/app/view.rs"))
        .expect("glass tuner view should read");
    assert!(
        !tuner.contains("standard_menu_bar"),
        "registration must not create ambient bars in an application that did not opt in"
    );
}

#[test]
fn deferred_tasks_have_one_worker_execution_path() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let executor = std::fs::read_to_string(root.join("src/task/executor.rs"))
        .expect("task executor source should read");
    let task =
        std::fs::read_to_string(root.join("src/task/mod.rs")).expect("task source should read");
    let context = std::fs::read_to_string(root.join("src/context/mod.rs"))
        .expect("command context source should read");
    let native = std::fs::read_to_string(root.join("src/platform/runner/native.rs"))
        .expect("native runner source should read");
    let handler = std::fs::read_to_string(root.join("src/platform/runner/handler.rs"))
        .expect("native runner handler source should read");
    let work = std::fs::read_to_string(root.join("src/shell/work.rs"))
        .expect("shell work source should read");
    let runtime_tests = std::fs::read_to_string(root.join("src/tests/runtime_tests.rs"))
        .expect("runtime tests source should read");
    let master = std::fs::read_to_string(root.join("docs/master_design.md"))
        .expect("master design should read");

    assert!(
        executor.contains("thread::Builder::new()")
            && executor.contains("wgpu_l3-worker-")
            && executor.contains("sender.send(Box::new(job))"),
        "task module must own execution on named worker threads"
    );
    assert!(
        executor.contains("(!workers.is_empty()).then_some(sender)")
            && executor.contains("Err(error)")
            && !executor.contains("worker executor thread should start"),
        "worker startup failure must leave an honest rejecting executor rather than panic or strand jobs"
    );
    assert!(
        task.contains("pub fn future(")
            && task.contains("pollster::block_on(future)")
            && !task.contains("winit")
            && !task.contains("windows")
            && !task.contains("wgpu"),
        "task must own future realization without importing UI, renderer, or platform machinery"
    );
    assert!(
        context.contains("tasks: Option<task::Sink>")
            && context.contains("fn with_tasks(")
            && context.contains("pub fn spawn<")
            && !context.contains(&format!("{}{}", "with_services_", "source")),
        "command context must carry the tasks-owned sink explicitly rather than an aggregate service bag"
    );
    assert_source_patterns_absent(
        &root.join("src"),
        &[format!("{}{}", "with_services_", "source")],
    );
    assert!(
        native.contains("executor.spawn(move ||")
            && native.contains("proxy.send_event(RunnerEvent::TaskCompleted")
            && native.contains("if !scheduled")
            && native.contains("cancel_task(id)"),
        "native runner must send worker results through its event-loop proxy and cancel rejected work"
    );
    assert!(
        handler.contains("runtime.accept_task_completion(id, event)")
            && handler.contains("runtime.dispatch_next_task_completion()"),
        "only accepted task ids may dispatch completion events on the UI thread"
    );
    let needs_poll = work
        .split("pub fn needs_poll(&self) -> bool")
        .nth(1)
        .and_then(|source| source.split("pub(crate) fn animation_schedule").next())
        .expect("Work::needs_poll body should exist");
    assert!(needs_poll.contains("self.task_completions > 0"));
    assert!(
        !needs_poll.contains("pending_tasks"),
        "pending worker jobs must not create UI poll wakes"
    );
    let executor_test = runtime_tests
        .split("fn task_executor_runs_future_work_off_the_calling_thread()")
        .nth(1)
        .and_then(|source| source.split("#[test]").next())
        .expect("executor runtime test body should exist");
    assert!(
        executor_test.contains("recv_timeout") && !executor_test.contains("sleep("),
        "executor coverage may use a failure ceiling but must not add real-time sleeps"
    );
    assert!(
        master.contains("Owns deferred job execution through a bounded worker pool")
            && master.contains("measurement-boundary mismatch")
            && master.contains("executor tests must not sleep"),
        "master design must name task ownership and retain the suite-runtime audit"
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

fn assert_pattern_only_in(path: &std::path::Path, pattern: &str, allowed: &std::path::Path) {
    for entry in std::fs::read_dir(path).expect("framework source directory should be readable") {
        let path = entry
            .expect("framework source entry should be readable")
            .path();
        if path.is_dir() {
            assert_pattern_only_in(&path, pattern, allowed);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("framework source file should read");
        if source.contains(pattern) {
            assert_eq!(
                path, allowed,
                "{pattern} must appear only in the named ownership site"
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
