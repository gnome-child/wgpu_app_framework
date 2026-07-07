use super::{
    composition,
    geometry::{Point, Size},
    interaction, keymap,
    theme::Theme,
    view,
};
use crate::animation;

mod algorithm;
mod chrome;
mod control;
mod engine;
mod flow;
mod frame;
mod hit;
mod measure;
mod path;
mod text;
mod typography;
mod viewport;

pub(crate) use chrome::{Chrome, Kind as ChromeKind, Scrollbar};
pub(crate) use control::{
    choice_label_rect, choice_mark_rect, control_content_extent, menu_row_slots, palette_row_slots,
    slider_label_rect, slider_thumb_rect, slider_track_rect,
};
pub(crate) use engine::Engine;
pub(crate) use frame::Frame;
pub(crate) use hit::Hit;
pub(crate) use text::{Area as TextArea, Service as TextService};
pub(crate) use typography::{
    interface_text_style, section_header_style, section_header_text, shortcut_run_gap,
    shortcut_text_style,
};
pub(crate) use viewport::Viewport;

#[derive(Clone)]
pub(crate) struct Layout {
    size: Size,
    frames: Vec<Frame>,
    chrome: Vec<Chrome>,
}

impl Layout {
    #[cfg(test)]
    pub(crate) fn compose(view: &view::View, size: Size, engine: &mut Engine) -> Self {
        Self::compose_with_theme(view, size, engine, &Theme::default())
    }

    #[cfg(test)]
    pub(crate) fn compose_with_theme(
        view: &view::View,
        size: Size,
        engine: &mut Engine,
        theme: &Theme,
    ) -> Self {
        Self::compose_with_theme_at(
            view,
            size,
            engine,
            theme,
            animation::Frame::new(std::time::Instant::now()),
            keymap::Profile::default(),
        )
    }

    #[cfg(test)]
    pub(crate) fn compose_with_theme_at(
        view: &view::View,
        size: Size,
        engine: &mut Engine,
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
        engine: &mut Engine,
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
        engine: &mut Engine,
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

    pub(crate) fn size(&self) -> Size {
        self.size
    }

    pub(crate) fn frames(&self) -> &[Frame] {
        &self.frames
    }

    pub(crate) fn chrome(&self) -> &[Chrome] {
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
        engine: &mut Engine,
    ) -> Option<(view::Role, Option<view::Action>)> {
        if let Some(chrome) = self.chrome.iter().find(|chrome| chrome.target() == target) {
            return Some((
                view::Role::Scroll,
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
        let descendant = if let Some(selected_index) = selected_index {
            self.frames
                .iter()
                .filter(|frame| {
                    frame.binding_source() == Some(crate::context::Source::Palette)
                        && frame.is_descendant_of(viewport_frame)
                })
                .nth(selected_index)?
        } else {
            self.frames
                .iter()
                .find(|frame| frame.is_selected() && frame.is_descendant_of(viewport_frame))?
        };

        Some(viewport.reveal_rect(descendant.rect(), margin))
    }

    #[cfg(test)]
    pub(crate) fn find_role(&self, role: view::Role) -> Vec<&Frame> {
        self.frames
            .iter()
            .filter(|frame| frame.role() == role)
            .collect()
    }
}
