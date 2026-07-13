const COMMAND_MOD: &str = include_str!("../command/mod.rs");
const COMMAND_REGISTRY: &str = include_str!("../command/registry.rs");
const COMMAND_POPULATION: &str = include_str!("../command/population.rs");
const WIDGET_BINDING: &str = include_str!("../widget/binding.rs");
const CONTEXT_RUNTIME: &str = include_str!("../runtime/context_menu.rs");
const PALETTE_RUNTIME: &str = include_str!("../runtime/palette.rs");
const TEXT_SERVICE: &str = include_str!("../runtime/services/text/mod.rs");
const INTERACTION_MENU: &str = include_str!("../interaction/menu.rs");
const MENU_SESSION: &str = include_str!("../session/interaction/menu.rs");
const LAYOUT_ALGORITHM: &str = include_str!("../layout/algorithm.rs");
const LAYOUT_CONTROL: &str = include_str!("../layout/control.rs");
const SCENE_PAINT: &str = include_str!("../scene/paint/mod.rs");
const NATIVE_POPUP: &str = include_str!("../platform/native/popup.rs");
const WINDOWS_SYS: &str = include_str!("../platform/native/sys/windows.rs");

#[test]
fn command_surfaces_share_one_private_erased_resolution_boundary() {
    assert_eq!(
        COMMAND_POPULATION.matches("fn resolve_claimed").count(),
        1,
        "the population domain should own one claimed-action resolver"
    );
    assert!(!COMMAND_REGISTRY.contains("resolved_unit_commands"));
    assert!(!PALETTE_RUNTIME.contains("resolved_unit_commands"));
    assert!(COMMAND_POPULATION.contains("pub(crate) enum Palette"));
    assert!(COMMAND_POPULATION.contains("pub(crate) enum Context"));
    assert!(COMMAND_POPULATION.contains("pub(crate) enum Bar"));
    assert!(COMMAND_POPULATION.contains("claim: responder::Claim"));
    assert!(!COMMAND_POPULATION.contains("claim: Option<responder::Claim>"));
    let bar_entry = COMMAND_POPULATION
        .split("pub(crate) struct BarAction {")
        .nth(1)
        .and_then(|source| source.split("impl<'a> Population").next())
        .expect("bar entry projection should exist");
    assert!(bar_entry.contains("standard: Option<Standard>"));
    assert!(!bar_entry.contains("claim:"));
    assert!(!COMMAND_REGISTRY.contains("fn palette_candidates"));
    assert!(!COMMAND_REGISTRY.contains("fn context_candidates"));
    assert!(!COMMAND_REGISTRY.contains("fn bar_candidates"));
    assert!(COMMAND_MOD.contains("pub(crate) use trigger::{AnyTrigger, AnyValueTrigger}"));
    assert!(!COMMAND_MOD.contains("pub use trigger::{AnyTrigger"));
}

#[test]
fn menu_topology_placement_does_not_collide_with_widget_participation_form() {
    assert!(WIDGET_BINDING.contains("enum Form"));
    assert!(!WIDGET_BINDING.contains("enum Placement"));
}

#[test]
fn contextual_discovery_cannot_cross_into_global_registry_scanning() {
    assert!(CONTEXT_RUNTIME.contains("context_candidates"));
    assert!(!CONTEXT_RUNTIME.contains("palette_candidates"));
    assert!(PALETTE_RUNTIME.contains("palette_candidates"));
    assert!(!LAYOUT_ALGORITHM.contains("command::Registry"));
    assert!(!LAYOUT_CONTROL.contains("command::Registry"));
    assert!(!LAYOUT_ALGORITHM.contains("context_actions"));
}

#[test]
fn contextual_menus_reuse_menu_lifecycle_and_row_presentation() {
    assert!(INTERACTION_MENU.contains("enum Origin"));
    assert!(INTERACTION_MENU.contains("Context {"));
    assert!(!INTERACTION_MENU.contains("command::State"));
    assert!(!MENU_SESSION.contains("ContextMenuSession"));
    assert!(!LAYOUT_CONTROL.contains("context_menu"));
    assert!(!SCENE_PAINT.contains("paint_context_menu"));
    assert!(!SCENE_PAINT.contains("ContextMenuRow"));
}

#[test]
fn platform_hosts_supply_bounds_without_owning_menu_policy() {
    assert!(LAYOUT_ALGORITHM.contains("PlacementRequest"));
    assert!(NATIVE_POPUP.contains("placement.resolve(available)"));
    assert!(!NATIVE_POPUP.contains("TrackPopupMenu"));
    assert!(!WINDOWS_SYS.contains("TrackPopupMenu"));
    assert!(!WINDOWS_SYS.contains("CreatePopupMenu"));
}

#[test]
fn focused_text_history_and_transfer_share_one_claim_scope_without_sharing_meaning() {
    let targets = TEXT_SERVICE
        .split("fn targets")
        .nth(1)
        .expect("focused text target inventory should exist");
    assert!(targets.contains("for_provider::<document::Copy>()"));
    assert!(targets.contains("for_provider::<timeline::Undo>()"));

    let claim = TEXT_SERVICE
        .split("pub(super) fn claim")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn owns_command").next())
        .expect("focused text claim projection should exist");
    assert!(claim.contains("responder::Claim::service(scope_kind"));
    assert!(
        !claim.contains("document::Copy") && !claim.contains("timeline::Undo"),
        "one incoming scope kind applies to both capabilities; it cannot encode their menu groups"
    );
}
