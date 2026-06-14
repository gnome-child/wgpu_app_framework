use crate::app::state::WindowState;
use crate::geometry::area;
use crate::{action, layout, paint, ui, window};

pub fn compose<T>(
    window: window::Id,
    tree: &ui::Tree,
    state: &mut WindowState,
    actions: &action::Registry<T>,
    logical_area: area::Logical,
) -> paint::Scene {
    let mut scene = paint::Scene::new();

    state.actions = tree.actions();
    state.action_targets = tree.action_targets();
    state.responders = tree.responders();
    state.interactivity = tree.interactivity();

    if let Some(layout) = tree.layout(logical_area) {
        state.focus_order = focus_order(&layout, &state.interactivity);
        state.clear_stale_focus();
        state.clear_stale_command_target();
        let command_target = state.command_context(window);

        let interaction = ui::Interaction::new(
            state.hovered.clone(),
            state.focused_path(),
            state.pressed.clone(),
        )
        .with_focus_visibility(state.focus_visibility())
        .with_command_target(command_target);
        state.layout = Some(layout.clone());

        tree.paint(&layout, actions, window, interaction, &mut scene);
    } else {
        state.layout = None;
        state.focus_order.clear();
        state.clear_focus();
        state.clear_command_target();
    }

    scene
}

fn focus_order(
    layout: &layout::Box,
    interactivity: &std::collections::HashMap<ui::Path, ui::Interactivity>,
) -> Vec<ui::Path> {
    let mut order = Vec::new();
    collect_focus_order(layout, interactivity, &mut order);
    order
}

fn collect_focus_order(
    layout: &layout::Box,
    interactivity: &std::collections::HashMap<ui::Path, ui::Interactivity>,
    order: &mut Vec<ui::Path>,
) {
    if interactivity
        .get(layout.path())
        .is_some_and(|interactivity| interactivity.focusable())
    {
        order.push(layout.path().clone());
    }

    for child in layout.children() {
        collect_focus_order(child, interactivity, order);
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::area;
    use crate::{Action, layout, paint};

    use super::*;

    const ROOT: ui::Id = ui::Id::new("root");
    const CHILD: ui::Id = ui::Id::new("child");
    const OTHER: ui::Id = ui::Id::new("other");
    const CLICK: action::Id = action::Id::new("click");

    #[test]
    fn compose_updates_state_and_preserves_paint_order() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let mut registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        registry.register(Action::new(CLICK, "Click"));
        tree.set_root(
            ui::control::panel(ROOT)
                .with_background(paint::Color::BLACK)
                .with_child(
                    ui::control::button(CHILD, CLICK)
                        .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
                ),
        );

        let scene = compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(100.0, 100.0),
        );

        assert!(state.layout.is_some());
        assert_eq!(
            state.actions.get(&ui::Path::new([ROOT, CHILD])),
            Some(&CLICK)
        );
        assert_eq!(
            state.action_targets.get(&ui::Path::new([ROOT, CHILD])),
            Some(&ui::ActionTarget::Origin)
        );
        assert!(state.responders.is_empty());
        assert!(state.interactivity.contains_key(&ui::Path::from(ROOT)));
        assert_eq!(state.focus_order, vec![ui::Path::new([ROOT, CHILD])]);
        assert_eq!(scene.items().len(), 2);
    }

    #[test]
    fn compose_clears_stale_focused_paths_after_tree_rebuild() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            focus: Some(crate::app::state::Focus::new(
                ui::Path::new([ROOT, CHILD]),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };
        let registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            ui::control::panel(ROOT).with_child(
                ui::control::button(OTHER, CLICK)
                    .with_responder(CLICK)
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(state.focused_path(), None);
    }

    #[test]
    fn compose_clears_stale_command_target_after_tree_rebuild() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            command_target: Some(action::Scope::Path(ui::Path::new([ROOT, CHILD]))),
            ..WindowState::default()
        };
        let registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            ui::control::panel(ROOT).with_child(
                ui::control::button(OTHER, CLICK)
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(state.command_target, None);
    }

    #[test]
    fn compose_stores_responder_paths() {
        let window = window::Id::new(1);
        let mut state = WindowState::default();
        let registry = action::Registry::<()>::new();
        let mut tree = ui::Tree::new();

        tree.set_root(
            ui::control::panel(ROOT).with_child(
                ui::Node::leaf(CHILD)
                    .with_responder(action::SELECT_ALL)
                    .with_size(layout::Size::Fixed(10.0), layout::Size::Fixed(10.0)),
            ),
        );

        compose(
            window,
            &tree,
            &mut state,
            &registry,
            area::logical(100.0, 100.0),
        );

        assert_eq!(
            state.responders.get(&ui::Path::new([ROOT, CHILD])),
            Some(&vec![action::SELECT_ALL])
        );
    }
}
