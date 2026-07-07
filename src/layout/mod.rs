use super::{
    composition,
    geometry::{Point, Size},
    interaction, keymap,
    theme::Theme,
    view,
};
use crate::animation;
use std::time::Instant;

mod algorithm;
pub(crate) mod chrome;
pub(crate) mod control;
pub mod engine;
pub(crate) mod flow;
pub(crate) mod frame;
pub(crate) mod hit;
mod measure;
pub(crate) mod path;
pub(crate) mod text;
pub(crate) mod typography;
pub mod viewport;

use frame::Frame;
use hit::Hit;

#[derive(Clone)]
pub struct Layout {
    size: Size,
    frames: Vec<Frame>,
    chrome: Vec<chrome::Chrome>,
}

impl Layout {
    pub fn compose(view: &view::View, size: Size, engine: &mut engine::Engine) -> Self {
        Self::compose_with_theme(view, size, engine, &Theme::default())
    }

    pub fn compose_with_theme(
        view: &view::View,
        size: Size,
        engine: &mut engine::Engine,
        theme: &Theme,
    ) -> Self {
        Self::compose_with_theme_at(
            view,
            size,
            engine,
            theme,
            animation::Frame::new(Instant::now()),
            keymap::Profile::default(),
        )
    }

    pub(crate) fn compose_with_theme_at(
        view: &view::View,
        size: Size,
        engine: &mut engine::Engine,
        theme: &Theme,
        frame: animation::Frame,
        keymap: keymap::Profile,
    ) -> Self {
        let tree = composition::Tree::layout(view);
        Self::compose_view_tree_with_theme_at(view, &tree, size, engine, theme, frame, keymap)
    }

    pub(crate) fn compose_composition_with_theme_at(
        composition: &composition::Composition,
        size: Size,
        engine: &mut engine::Engine,
        theme: &Theme,
        frame: animation::Frame,
        keymap: keymap::Profile,
    ) -> Self {
        Self::compose_view_tree_with_theme_at(
            composition.view(),
            composition.tree(),
            size,
            engine,
            theme,
            frame,
            keymap,
        )
    }

    fn compose_view_tree_with_theme_at(
        view: &view::View,
        tree: &composition::Tree,
        size: Size,
        engine: &mut engine::Engine,
        theme: &Theme,
        frame: animation::Frame,
        keymap: keymap::Profile,
    ) -> Self {
        let size = size.sanitized();
        let frames =
            algorithm::compose_frames(view.root(), tree.root(), size, engine, theme, frame, keymap);
        let chrome = chrome::project(&frames, theme);

        Self {
            size,
            frames,
            chrome,
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub(crate) fn frames(&self) -> &[Frame] {
        &self.frames
    }

    pub(crate) fn chrome(&self) -> &[chrome::Chrome] {
        &self.chrome
    }

    pub(crate) fn hit_test(&self, point: Point) -> Option<Hit> {
        if let Some((owner, chrome)) = self
            .chrome
            .iter()
            .rev()
            .filter(|chrome| chrome.accepts_hit(point))
            .find_map(|chrome| {
                let owner = self
                    .frames
                    .iter()
                    .rev()
                    .find(|frame| frame.target() == Some(chrome.scroll_target()))?;
                owner.clip_contains(point).then_some((owner, chrome))
            })
        {
            return Some(Hit::chrome(owner.clone(), chrome.clone()));
        }

        self.frames
            .iter()
            .rev()
            .find(|frame| frame.accepts_hit(point))
            .cloned()
            .map(Hit::new)
    }

    pub(crate) fn drag_action_for_target(
        &self,
        target: &interaction::Target,
        point: Point,
        engine: &mut engine::Engine,
    ) -> Option<(view::node::Role, Option<view::Action>)> {
        if let Some(chrome) = self.chrome.iter().find(|chrome| chrome.target() == target) {
            return Some((
                view::node::Role::Scroll,
                Some(view::Action::scroll_to(
                    chrome.scroll_target().clone(),
                    chrome.scroll_offset_at(point),
                )),
            ));
        }

        self.frames
            .iter()
            .find(|frame| frame.target() == Some(target))
            .map(|frame| {
                (
                    frame.role(),
                    frame.drag_action_at_with_engine(point, engine),
                )
            })
    }

    pub(crate) fn scroll_target_at(
        &self,
        point: Point,
        delta: interaction::ScrollDelta,
    ) -> Option<interaction::Target> {
        self.frames
            .iter()
            .rev()
            .find(|frame| {
                frame.viewport().is_some_and(|viewport| {
                    viewport.rect().contains(point)
                        && frame.clip_contains(point)
                        && viewport.can_consume(delta)
                })
            })
            .and_then(Frame::target)
            .cloned()
    }

    pub(crate) fn active_descendant_reveal_offset(
        &self,
        viewport_target: &interaction::Target,
        selected_index: Option<usize>,
        margin: i32,
    ) -> Option<interaction::ScrollOffset> {
        let viewport_frame = self
            .frames
            .iter()
            .find(|frame| frame.target() == Some(viewport_target))?;
        let viewport = viewport_frame.viewport()?;
        let viewport_path = viewport_frame.path();
        let descendant = if let Some(selected_index) = selected_index {
            self.frames
                .iter()
                .filter(|frame| {
                    frame.binding_source() == Some(crate::context::Source::Palette)
                        && frame.path().is_descendant_of(viewport_path)
                })
                .nth(selected_index)?
        } else {
            self.frames
                .iter()
                .find(|frame| frame.is_selected() && frame.path().is_descendant_of(viewport_path))?
        };

        Some(viewport.reveal_rect(descendant.rect(), margin))
    }

    #[cfg(test)]
    pub(crate) fn find_role(&self, role: view::node::Role) -> Vec<&Frame> {
        self.frames
            .iter()
            .filter(|frame| frame.role() == role)
            .collect()
    }
}
