pub mod menu;
pub mod scroll;

mod control;
mod menu_popup;
mod popup;

use crate::geometry::{Rect, area, point};
use crate::{layout_old, text, theme, ui};

pub use self::menu::Menu;
pub use self::menu_popup::{menu_popup, submenu_popup};
pub use self::popup::Popup;
pub use control::{
    button, button_with_theme, floating_panel, floating_panel_with_theme, icon_button,
    icon_button_with_theme, labeled_button, labeled_button_with_theme, panel, panel_with_theme,
};

pub const MENU_POPUP: ui::Id = ui::Id::new("__menu_popup");
pub const MENU_SUBMENU_POPUP: ui::Id = ui::Id::new("__menu_submenu_popup");

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scroll {
    offset: point::Logical,
    bars: scroll::Bars,
    style: scroll::Style,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    outer: Rect,
    viewport: Rect,
    content_size: area::Logical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Metrics {
    Scroll(scroll::Metrics),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Scroll(scroll::Part),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    target: ui::Path,
    part: Part,
}

pub fn label(id: ui::Id, label: impl Into<String>) -> ui::Node {
    label_with_theme(id, label, &theme::Theme::default_dark())
}

pub fn label_with_theme(id: ui::Id, label: impl Into<String>, theme: &theme::Theme) -> ui::Node {
    ui::Node::leaf(id)
        .with_label(document_with_theme(label, theme, text::Align::Center))
        .with_label_color(theme.text().secondary())
        .with_size(
            layout_old::Size::Fill,
            layout_old::Size::Fixed(theme.density().label_height()),
        )
}

pub fn separator(id: ui::Id) -> ui::Node {
    separator_with_theme(id, &theme::Theme::default_dark())
}

pub fn separator_with_theme(id: ui::Id, theme: &theme::Theme) -> ui::Node {
    ui::Node::leaf(id)
        .with_background(theme.surfaces().separator())
        .with_size(layout_old::Size::Fill, layout_old::Size::Fixed(1.0))
}

pub fn scroll_view(id: ui::Id) -> ui::Node {
    scroll_view_with_theme(id, &theme::Theme::default_dark())
}

pub fn scroll_view_with_theme(id: ui::Id, theme: &theme::Theme) -> ui::Node {
    let scroll = theme.scroll();
    let scroll = Scroll::new()
        .with_bars(scroll::Bars::vertical())
        .with_style(scroll::Style::new(
            scroll.thickness(),
            scroll.min_thumb_length(),
            scroll.track(),
            scroll.thumb(),
            scroll.thumb_hover_tint(),
            scroll.thumb_pressed_tint(),
            scroll.corner(),
        ));

    ui::Node::container(id, layout_old::Axis::Vertical)
        .clipped()
        .with_scroll(scroll)
        .with_size(layout_old::Size::Fill, layout_old::Size::Fill)
}

pub fn menu_bar(id: ui::Id, bar: menu::Bar) -> ui::Node {
    menu_bar_with_theme(id, bar, &theme::Theme::default_dark())
}

pub fn menu_bar_with_theme(id: ui::Id, bar: menu::Bar, theme: &theme::Theme) -> ui::Node {
    let mut node = ui::Node::container(id, layout_old::Axis::Horizontal)
        .with_menu_bar(bar.clone())
        .with_background(theme.menu().bar_background())
        .with_stroke(theme.menu().bar_stroke())
        .with_size(
            layout_old::Size::Fill,
            layout_old::Size::Fixed(theme.density().menu_bar_height()),
        );

    for menu in bar.menus() {
        node.push_child(menu_title(menu, theme));
    }

    node
}

pub fn document(label: impl Into<String>) -> text::Document {
    let mut block = text::Block::new(text::Align::Center);
    block.push_run(text::Run::new(label, text::Style::default()));

    text::Document::from_block(block)
}

fn document_with_theme(
    label: impl Into<String>,
    theme: &theme::Theme,
    align: text::Align,
) -> text::Document {
    let mut block = text::Block::new(align);
    block.push_run(text::Run::new(
        label,
        text::Style::default()
            .with_size(theme.text().menu_size())
            .with_color(theme.text().primary()),
    ));

    text::Document::from_block(block)
}

fn menu_title(menu: &menu::Menu, theme: &theme::Theme) -> ui::Node {
    let outline = theme.menu().title_focus_outline();
    let horizontal_padding = theme.density().menu_title_horizontal_padding() * 0.5;

    ui::Node::leaf(ui::Id::new(menu.id().as_str()))
        .with_intent(ui::Intent::OpenMenu(menu.id()))
        .with_interactivity(ui::Interactivity::CONTROL)
        .with_label(document_with_theme(
            menu.label(),
            theme,
            text::Align::Center,
        ))
        .with_background(theme.menu().title_background())
        .with_hover_tint(theme.menu().title_hover_tint())
        .with_pressed_tint(theme.menu().title_pressed_tint())
        .with_active_tint(theme.menu().title_active_tint())
        .with_focus_outline(outline.brush(), outline.width(), outline.offset())
        .with_label_color(theme.text().primary())
        .with_rounding(theme.roundings().menu_title())
        .with_padding(layout_old::Insets {
            left: horizontal_padding,
            top: 0.0,
            right: horizontal_padding,
            bottom: 0.0,
        })
        .with_size(layout_old::Size::Fit, layout_old::Size::Fill)
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            offset: point::logical(0.0, 0.0),
            bars: scroll::Bars::vertical(),
            style: scroll::Style::default(),
        }
    }

    pub fn offset(self) -> point::Logical {
        self.offset
    }

    pub fn bars(self) -> scroll::Bars {
        self.bars
    }

    pub fn style(self) -> scroll::Style {
        self.style
    }

    pub fn with_offset(mut self, offset: point::Logical) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_bars(mut self, bars: scroll::Bars) -> Self {
        self.bars = bars;
        self
    }

    pub fn with_style(mut self, style: scroll::Style) -> Self {
        self.style = style;
        self
    }
}

impl Default for Scroll {
    fn default() -> Self {
        Self::new()
    }
}

impl Frame {
    pub fn new(outer: Rect, viewport: Rect, content_size: area::Logical) -> Self {
        Self {
            outer,
            viewport,
            content_size,
        }
    }

    pub fn outer(self) -> Rect {
        self.outer
    }

    pub fn viewport(self) -> Rect {
        self.viewport
    }

    pub fn content_size(self) -> area::Logical {
        self.content_size
    }
}

impl Metrics {
    pub fn scroll(self) -> Option<scroll::Metrics> {
        match self {
            Self::Scroll(metrics) => Some(metrics),
        }
    }

    pub fn frame(self) -> Frame {
        match self {
            Self::Scroll(metrics) => metrics.frame(),
        }
    }

    pub fn hit_test(self, position: point::Logical) -> Option<Part> {
        match self {
            Self::Scroll(metrics) => metrics.hit_test(position).map(Part::Scroll),
        }
    }
}

impl Part {
    pub fn scroll(self) -> Option<scroll::Part> {
        match self {
            Self::Scroll(part) => Some(part),
        }
    }
}

impl Hit {
    pub fn new(target: ui::Path, part: Part) -> Self {
        Self { target, part }
    }

    pub fn target(&self) -> &ui::Path {
        &self.target
    }

    pub fn part(&self) -> Part {
        self.part
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint;

    const ROOT: ui::Id = ui::Id::new("root");

    #[test]
    fn scroll_stores_offset_bars_and_style() {
        let style = scroll::Style::new(
            8.0,
            14.0,
            paint::Color::BLACK,
            paint::Color::rgba(1.0, 1.0, 1.0, 1.0),
            paint::Color::rgba(1.0, 1.0, 1.0, 0.10),
            paint::Color::rgba(1.0, 1.0, 1.0, 0.20),
            paint::Color::BLACK,
        );
        let scroll = Scroll::new()
            .with_offset(point::logical(3.0, 9.0))
            .with_bars(scroll::Bars::both())
            .with_style(style);

        assert_eq!(scroll.offset(), point::logical(3.0, 9.0));
        assert_eq!(scroll.bars(), scroll::Bars::both());
        assert_eq!(scroll.style(), style);
    }

    #[test]
    fn scroll_bars_expose_enabled_axis_helpers() {
        assert!(!scroll::Bars::none().is_enabled());
        assert!(scroll::Bars::vertical().vertical_enabled());
        assert!(!scroll::Bars::vertical().horizontal_enabled());
        assert!(scroll::Bars::horizontal().horizontal_enabled());
        assert!(scroll::Bars::both().vertical_enabled());
        assert!(scroll::Bars::both().horizontal_enabled());
    }

    #[test]
    fn hit_reports_target_and_part() {
        let target = ui::Path::from(ROOT);
        let hit = Hit::new(target.clone(), Part::Scroll(scroll::Part::VerticalThumb));

        assert_eq!(hit.target(), &target);
        assert_eq!(hit.part(), Part::Scroll(scroll::Part::VerticalThumb));
    }

    #[test]
    fn menu_title_width_uses_measured_label_content() {
        let bar = menu::Bar::new()
            .menu(menu::Menu::new(menu::Id::new("a"), "A"))
            .menu(menu::Menu::new(
                menu::Id::new("workspace_tools"),
                "Workspace Tools",
            ));
        let mut tree = ui::Tree::new();
        let mut measurer = text::Measurer::new();

        tree.set_root(menu_bar(ROOT, bar));
        let layout = tree
            .layout(area::logical(320.0, 28.0), &mut measurer)
            .expect("menu bar should layout");

        assert!(
            layout.children()[1].rect().area.width() > layout.children()[0].rect().area.width()
        );
    }
}
