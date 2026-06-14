use crate::app::state::WindowState;
use crate::geometry::area;
use crate::{action, paint, ui, window};

pub fn compose<T>(
    window: window::Id,
    tree: &ui::Tree,
    state: &mut WindowState,
    actions: &action::Registry<T>,
    logical_area: area::Logical,
) -> paint::Scene {
    let mut scene = paint::Scene::new();

    state.actions = tree.actions();
    state.interactivity = tree.interactivity();

    if let Some(layout) = tree.layout(logical_area) {
        let interaction = ui::Interaction::new(
            state.hovered.clone(),
            state.focused.clone(),
            state.pressed.clone(),
        );
        state.layout = Some(layout.clone());

        tree.paint(&layout, actions, window, interaction, &mut scene);
    } else {
        state.layout = None;
    }

    scene
}

#[cfg(test)]
mod tests {
    use crate::geometry::area;
    use crate::{Action, layout, paint};

    use super::*;

    const ROOT: ui::Id = ui::Id::new("root");
    const CHILD: ui::Id = ui::Id::new("child");
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
        assert!(state.interactivity.contains_key(&ui::Path::from(ROOT)));
        assert_eq!(scene.items().len(), 2);
    }
}
