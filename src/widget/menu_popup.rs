use crate::geometry::{Rect, area, point, rect};
use crate::ui::{self, layout};
use crate::{action, icon, paint, text, theme};

use super::{
    MENU_POPUP, MENU_SUBMENU_POPUP, TEXT_CONTEXT_MENU_POPUP, floating_panel_with_theme, menu,
    separator_with_theme,
};

const SEPARATOR_LINE_HEIGHT: f32 = 1.0;
const SUBMENU_GAP: f32 = 3.0;

const ROW_GLYPH: ui::Id = ui::Id::new("__menu_row_glyph");
const ROW_LABEL: ui::Id = ui::Id::new("__menu_row_label");
const ROW_SHORTCUT: ui::Id = ui::Id::new("__menu_row_shortcut");
const ROW_TRAILING: ui::Id = ui::Id::new("__menu_row_trailing");
const ROW_SEPARATOR: ui::Id = ui::Id::new("__menu_row_separator");

pub trait Presenter {
    fn item_label(&self, surface: Option<&ui::floating::Surface>, item: &menu::Item) -> String;

    fn shortcut_label(&self, action: action::Key) -> Option<String>;

    fn menu_can_open(&self, surface: &ui::floating::Surface, menu: &menu::Menu) -> bool;
}

pub fn menu_popup(
    tree: &ui::Tree,
    layout: &ui::Frame,
    surface: &ui::floating::Surface,
    menu: &menu::Menu,
    presenter: &dyn Presenter,
    measurer: &mut text::layout::Engine,
) -> Option<ui::Popup> {
    let theme = theme::Theme::default_dark();
    let anchor = anchor_rect(tree, layout, menu.id(), AnchorKind::TopLevel)?;
    let chrome = popup_chrome_with_surface(menu, presenter, surface, &theme, measurer);
    let popup_rect = popup_rect(
        point::logical(anchor.origin.x(), anchor.origin.y() + anchor.area.height()),
        chrome,
        &theme,
        layout.rect(),
    );

    Some(ui::Popup::new(
        popup_rect,
        popup_node(MENU_POPUP, menu, presenter, surface, chrome, &theme),
    ))
}

pub fn submenu_popup(
    tree: &ui::Tree,
    layout: &ui::Frame,
    surface: &ui::floating::Surface,
    menu: &menu::Menu,
    presenter: &dyn Presenter,
    measurer: &mut text::layout::Engine,
) -> Option<ui::Popup> {
    let theme = theme::Theme::default_dark();
    let anchor = anchor_rect(tree, layout, menu.id(), AnchorKind::Submenu)?;
    let padding = theme.floating_panel().padding();
    let chrome = popup_chrome_with_surface(menu, presenter, surface, &theme, measurer);
    let popup_rect = popup_rect(
        point::logical(
            anchor.origin.x() + anchor.area.width() + SUBMENU_GAP,
            anchor.origin.y() - padding,
        ),
        chrome,
        &theme,
        layout.rect(),
    );

    Some(ui::Popup::new(
        popup_rect,
        popup_node(MENU_SUBMENU_POPUP, menu, presenter, surface, chrome, &theme),
    ))
}

pub fn text_context_menu_popup(
    surface: &ui::floating::Surface,
    menu: &menu::Menu,
    presenter: &dyn Presenter,
    measurer: &mut text::layout::Engine,
    bounds: Rect,
) -> Option<ui::Popup> {
    surface.context_menu_target()?;

    let theme = theme::Theme::default_dark();
    let chrome = popup_chrome_with_surface(menu, presenter, surface, &theme, measurer);
    let origin = match surface.anchor() {
        ui::floating::Anchor::Point(point) => point,
        ui::floating::Anchor::Rect(rect) => rect.origin,
    };
    let popup_rect = popup_rect(origin, chrome, &theme, bounds);

    Some(ui::Popup::new(
        popup_rect,
        popup_node(
            TEXT_CONTEXT_MENU_POPUP,
            menu,
            presenter,
            surface,
            chrome,
            &theme,
        ),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PopupChrome {
    area: area::Logical,
    body_width: f32,
    row_count: usize,
    shortcut_width: f32,
    padding: f32,
    row_height: f32,
    glyph_width: f32,
    row_rounding: rect::Rounding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnchorKind {
    TopLevel,
    Submenu,
}

fn anchor_rect(
    tree: &ui::Tree,
    layout: &ui::Frame,
    id: menu::Id,
    kind: AnchorKind,
) -> Option<Rect> {
    tree.intents()
        .into_iter()
        .find_map(|(path, intent)| match (kind, intent) {
            (AnchorKind::TopLevel, ui::Intent::OpenMenu(menu)) if menu == id => {
                layout.find_path(&path).map(|layout| layout.rect())
            }
            (AnchorKind::Submenu, ui::Intent::OpenSubmenu(menu)) if menu == id => {
                layout.find_path(&path).map(|layout| layout.rect())
            }
            _ => None,
        })
}

fn popup_rect(
    origin: point::Logical,
    chrome: PopupChrome,
    theme: &theme::Theme,
    bounds: Rect,
) -> Rect {
    let origin = clamped_origin(pixel_aligned_origin(origin), chrome.area, bounds);

    Rect::rounded(origin, chrome.area, theme.floating_panel().rounding())
}

fn clamped_origin(
    origin: point::Logical,
    popup_area: area::Logical,
    bounds: Rect,
) -> point::Logical {
    let min_x = bounds.origin.x();
    let min_y = bounds.origin.y();
    let max_x = (bounds.origin.x() + bounds.area.width() - popup_area.width()).max(min_x);
    let max_y = (bounds.origin.y() + bounds.area.height() - popup_area.height()).max(min_y);

    point::logical(
        origin.x().clamp(min_x, max_x),
        origin.y().clamp(min_y, max_y),
    )
}

fn popup_node(
    id: ui::Id,
    menu: &menu::Menu,
    presenter: &dyn Presenter,
    surface: &ui::floating::Surface,
    chrome: PopupChrome,
    theme: &theme::Theme,
) -> ui::Node {
    let mut popup = floating_panel_with_theme(theme)
        .key(id)
        .with_action_scope()
        .with_size(
            layout::Size::Fixed(chrome.area.width()),
            layout::Size::Fixed(chrome.area.height()),
        );

    let mut row = 0;
    for section in menu.sections() {
        for node in section.nodes() {
            popup.push_child(match node {
                menu::Node::Item(item) => {
                    let id = row_id(row);
                    row += 1;
                    item_node(id, item, presenter, surface, chrome, theme)
                }
                menu::Node::Submenu(menu) => {
                    let id = row_id(row);
                    row += 1;
                    submenu_node(id, menu, presenter, surface, chrome, theme)
                }
                menu::Node::Separator => {
                    let id = row_id(row);
                    row += 1;
                    separator_node(id, chrome, theme)
                }
            });
        }
    }

    popup
}

fn item_node(
    id: ui::Id,
    item: &menu::Item,
    presenter: &dyn Presenter,
    surface: &ui::floating::Surface,
    chrome: PopupChrome,
    theme: &theme::Theme,
) -> ui::Node {
    let action_id = item.action();
    let color = theme.text().primary();
    let shortcut = presenter.shortcut_label(action_id);

    menu_row(id, chrome, theme, color)
        .with_action_route(item.route())
        .with_action_subject(ui::ActionSubject::Captured)
        .with_child(active_glyph_cell(
            ROW_GLYPH,
            icon::Icon::phosphor(icon::Id::new("check")),
            chrome.glyph_width,
            color,
            theme,
        ))
        .with_child(text_cell(
            ROW_LABEL,
            presenter.item_label(Some(surface), item),
            text::document::Align::Start,
            layout::Size::Fill,
            color,
            theme,
        ))
        .with_child(text_cell(
            ROW_SHORTCUT,
            shortcut.unwrap_or_default(),
            text::document::Align::End,
            layout::Size::Fixed(chrome.shortcut_width),
            color,
            theme,
        ))
        .with_child(glyph_cell(
            ROW_TRAILING,
            None,
            chrome.glyph_width,
            color,
            theme,
        ))
}

fn submenu_node(
    id: ui::Id,
    menu: &menu::Menu,
    presenter: &dyn Presenter,
    surface: &ui::floating::Surface,
    chrome: PopupChrome,
    theme: &theme::Theme,
) -> ui::Node {
    let enabled = presenter.menu_can_open(surface, menu);
    let color = if enabled {
        theme.text().primary()
    } else {
        theme.text().disabled()
    };
    let chevron = enabled.then(|| icon::Icon::phosphor(icon::Id::new("caret-right")));
    let row = menu_row(id, chrome, theme, color)
        .with_child(glyph_cell(
            ROW_GLYPH,
            None,
            chrome.glyph_width,
            color,
            theme,
        ))
        .with_child(text_cell(
            ROW_LABEL,
            menu.label(),
            text::document::Align::Start,
            layout::Size::Fill,
            color,
            theme,
        ))
        .with_child(glyph_cell(
            ROW_SHORTCUT,
            None,
            chrome.shortcut_width,
            color,
            theme,
        ))
        .with_child(glyph_cell(
            ROW_TRAILING,
            chevron,
            chrome.glyph_width,
            color,
            theme,
        ));

    if enabled {
        row.with_intent(ui::Intent::OpenSubmenu(menu.id()))
            .with_interactivity(ui::Interactivity::CONTROL)
            .with_active_tint(theme.menu().row_hover_tint())
    } else {
        row.with_interactivity(ui::Interactivity::NONE.with_hit_test(true))
    }
}

fn menu_row(
    id: ui::Id,
    chrome: PopupChrome,
    theme: &theme::Theme,
    color: paint::Color,
) -> ui::Node {
    ui::Node::container(layout::Axis::Horizontal)
        .key(id)
        .with_intent(ui::Intent::CloseSubmenu)
        .with_interactivity(ui::Interactivity::CONTROL)
        .with_background(theme.menu().row_background())
        .with_focus_background(theme.menu().row_hover_tint())
        .with_hover_tint(theme.menu().row_hover_tint())
        .with_pressed_tint(theme.menu().row_pressed_tint())
        .with_label_color(color)
        .with_disabled_label_color(theme.text().disabled())
        .with_rounding(chrome.row_rounding)
        .with_size(layout::Size::Fill, layout::Size::Fixed(chrome.row_height))
}

fn glyph_cell(
    id: ui::Id,
    icon: Option<icon::Icon>,
    width: f32,
    color: paint::Color,
    theme: &theme::Theme,
) -> ui::Node {
    let mut node = ui::Node::leaf()
        .with_kind("menu_glyph")
        .key(id)
        .with_label_color(color)
        .with_disabled_label_color(theme.text().disabled())
        .with_size(layout::Size::Fixed(width), layout::Size::Fill);

    if let Some(icon) = icon {
        node = node
            .with_icon(icon)
            .with_icon_size(theme.text().menu_size() + 2.0);
    }

    node
}

fn active_glyph_cell(
    id: ui::Id,
    icon: icon::Icon,
    width: f32,
    color: paint::Color,
    theme: &theme::Theme,
) -> ui::Node {
    ui::Node::leaf()
        .with_kind("menu_glyph")
        .key(id)
        .with_label_color(color)
        .with_disabled_label_color(theme.text().disabled())
        .with_active_icon(icon)
        .with_icon_size(theme.text().menu_size() + 2.0)
        .with_size(layout::Size::Fixed(width), layout::Size::Fill)
}

fn text_cell(
    id: ui::Id,
    label: impl Into<String>,
    align: text::document::Align,
    width: layout::Size,
    color: paint::Color,
    theme: &theme::Theme,
) -> ui::Node {
    ui::Node::leaf()
        .with_kind("menu_text")
        .key(id)
        .with_label(document(label, theme, align, color))
        .with_label_color(color)
        .with_disabled_label_color(theme.text().disabled())
        .with_size(width, layout::Size::Fill)
}

fn separator_node(id: ui::Id, chrome: PopupChrome, theme: &theme::Theme) -> ui::Node {
    ui::Node::container(layout::Axis::Vertical)
        .with_kind("menu_separator")
        .key(id)
        .with_intent(ui::Intent::CloseSubmenu)
        .with_interactivity(ui::Interactivity::NONE.with_hit_test(true))
        .with_align(layout::Align::Center)
        .with_cross_align(layout::Align::Stretch)
        .with_size(layout::Size::Fill, layout::Size::Fixed(chrome.row_height))
        .with_child(separator_with_theme(theme).key(ROW_SEPARATOR).with_size(
            layout::Size::Fill,
            layout::Size::Fixed(SEPARATOR_LINE_HEIGHT),
        ))
}

fn document(
    label: impl Into<String>,
    theme: &theme::Theme,
    align: text::document::Align,
    color: paint::Color,
) -> text::document::Document {
    let mut block = text::document::Block::new(align);
    block.push_run(text::document::Run::new(
        label,
        theme
            .text()
            .style(text::document::Role::Menu)
            .with_color(text_color(color)),
    ));
    text::document::Document::from_block(block)
}

fn text_color(color: paint::Color) -> text::Color {
    text::Color::rgba(color.r, color.g, color.b, color.a)
}

#[cfg(test)]
fn popup_chrome(
    menu: &menu::Menu,
    presenter: &dyn Presenter,
    theme: &theme::Theme,
    measurer: &mut text::layout::Engine,
) -> PopupChrome {
    popup_chrome_for_surface(menu, presenter, None, theme, measurer)
}

fn popup_chrome_with_surface(
    menu: &menu::Menu,
    presenter: &dyn Presenter,
    surface: &ui::floating::Surface,
    theme: &theme::Theme,
    measurer: &mut text::layout::Engine,
) -> PopupChrome {
    popup_chrome_for_surface(menu, presenter, Some(surface), theme, measurer)
}

fn popup_chrome_for_surface(
    menu: &menu::Menu,
    presenter: &dyn Presenter,
    surface: Option<&ui::floating::Surface>,
    theme: &theme::Theme,
    measurer: &mut text::layout::Engine,
) -> PopupChrome {
    let mut label_width = 0.0_f32;
    let mut shortcut_width = 0.0_f32;
    let mut row_count = 0_usize;

    for node in menu.sections().iter().flat_map(menu::Section::nodes) {
        row_count += 1;
        match node {
            menu::Node::Item(item) => {
                label_width = label_width.max(measure_label(
                    presenter.item_label(surface, item),
                    theme,
                    measurer,
                ));
                if let Some(shortcut) = presenter.shortcut_label(item.action()) {
                    shortcut_width = shortcut_width.max(measure_label(shortcut, theme, measurer));
                }
            }
            menu::Node::Submenu(menu) => {
                label_width = label_width.max(measure_label(menu.label(), theme, measurer));
            }
            menu::Node::Separator => {}
        }
    }

    let padding = theme.floating_panel().padding();
    let row_height = theme.density().menu_row_height();
    let glyph_width = glyph_column_width(theme);
    let body_width = glyph_width + label_width + shortcut_width + glyph_width;
    let body_height = row_height * row_count as f32;
    let width = (body_width + padding * 2.0).max(theme.density().menu_popup_min_width());
    let height = body_height + padding * 2.0;
    let area = pixel_aligned_area(area::logical(width, height));

    PopupChrome {
        area,
        body_width,
        row_count,
        shortcut_width,
        padding,
        row_height,
        glyph_width,
        row_rounding: row_rounding(theme, area, padding),
    }
}

fn pixel_aligned_origin(origin: point::Logical) -> point::Logical {
    point::logical(origin.x().round(), origin.y().round())
}

fn pixel_aligned_area(area: area::Logical) -> area::Logical {
    area::logical(area.width().ceil(), area.height().ceil())
}

fn glyph_column_width(theme: &theme::Theme) -> f32 {
    theme.density().menu_row_height()
}

fn row_rounding(theme: &theme::Theme, popup_area: area::Logical, padding: f32) -> rect::Rounding {
    let rounding = theme.floating_panel().rounding().resolve(popup_area);
    rect::Rounding::new(
        rect::Radius::fixed((rounding[0] - padding).max(0.0)),
        rect::Radius::fixed((rounding[1] - padding).max(0.0)),
        rect::Radius::fixed((rounding[2] - padding).max(0.0)),
        rect::Radius::fixed((rounding[3] - padding).max(0.0)),
    )
}

fn measure_label(
    label: impl Into<String>,
    theme: &theme::Theme,
    measurer: &mut text::layout::Engine,
) -> f32 {
    measurer
        .measure(
            &document(
                label,
                theme,
                text::document::Align::Start,
                theme.text().primary(),
            ),
            text::layout::Measure::unbounded(),
        )
        .width()
}

fn row_id(index: usize) -> ui::Id {
    ui::Id::structural("__menu_row", index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, command};
    use std::collections::HashSet;

    struct CommandA;
    struct CommandB;

    impl Command for CommandA {
        type Args = ();
        type Output = ();

        const NAME: &'static str = "menu_command_a";
        const DISPLAY: &'static str = "A";
    }

    impl Command for CommandB {
        type Args = ();
        type Output = ();

        const NAME: &'static str = "menu_command_b";
        const DISPLAY: &'static str = "B";
    }

    const COMMAND_A: command::Key = command::Key::of::<CommandA>();
    const COMMAND_B: command::Key = command::Key::of::<CommandB>();

    impl Presenter for command::Registry {
        fn item_label(&self, surface: Option<&ui::floating::Surface>, item: &menu::Item) -> String {
            let command = command::Key::from_action(item.action());
            item.label()
                .map(str::to_owned)
                .or_else(|| {
                    surface
                        .map(|_| test_context())
                        .and_then(|context| self.presentation_key(command, context))
                        .map(|presentation| presentation.display().to_owned())
                })
                .or_else(|| {
                    self.command_key(command)
                        .map(|command| command.display().to_owned())
                })
                .unwrap_or_else(|| command.as_str().replace('_', " "))
        }

        fn shortcut_label(&self, action: action::Key) -> Option<String> {
            self.command_key(command::Key::from_action(action))
                .and_then(|command| command.shortcuts().first())
                .copied()
                .map(command::shortcut::Shortcut::display_label)
        }

        fn menu_can_open(&self, _surface: &ui::floating::Surface, menu: &menu::Menu) -> bool {
            let context = test_context();

            menu.actions().any(|route| {
                self.can_invoke_key(
                    command::binding::Route::from_action(route).command(),
                    context.clone(),
                )
            })
        }
    }

    fn test_context() -> command::call::Context {
        command::call::Context::window(crate::window::Id::new(1))
    }

    fn surface() -> ui::floating::Surface {
        ui::floating::Surface::new(
            ui::floating::Kind::Menu(menu::Id::new("test")),
            MENU_POPUP,
            ui::floating::Anchor::Rect(Rect::new(
                point::logical(0.0, 0.0),
                area::logical(0.0, 0.0),
            )),
            ui::floating::FocusPolicy::PreserveCurrentFocus,
        )
    }

    #[test]
    fn row_ids_are_structural_and_do_not_overflow() {
        let ids = (0..64).map(row_id).collect::<HashSet<_>>();

        assert_eq!(ids.len(), 64);
        assert!(row_id(0).is_structural());
        assert_ne!(row_id(31), row_id(32));
        assert_ne!(row_id(32), row_id(63));
    }

    fn paint_with_visual_states(
        root: ui::Node,
        interaction: ui::Interaction,
        visual_states: impl IntoIterator<Item = (ui::Path, ui::VisualState)>,
        scene: &mut paint::Scene,
    ) {
        let mut tree = ui::Tree::new();
        let mut measurer = text::layout::Engine::new();
        let mut composition = {
            tree.set_root(root);
            tree.compose(area::logical(160.0, 40.0), &mut measurer)
                .expect("menu row should compose")
        };
        let text_field_states = std::collections::HashMap::new();

        assert!(composition.set_visual_states(visual_states.into_iter().collect()));
        composition.paint(interaction, &text_field_states, &mut measurer, scene);
    }

    #[test]
    fn menu_separator_occupies_normal_row_height() {
        let theme = theme::Theme::default_dark();
        let registry = command::Registry::new();
        let mut measurer = text::layout::Engine::new();
        let menu = menu::Menu::new("Test").key(menu::Id::new("test")).section(
            menu::Section::new()
                .command_key(COMMAND_A)
                .separator()
                .command_key(COMMAND_B),
        );
        let metrics = popup_chrome(&menu, &registry, &theme, &mut measurer);

        assert_eq!(
            metrics.area.height(),
            metrics.row_height * metrics.row_count as f32 + metrics.padding * 2.0
        );
    }

    #[test]
    fn menu_popup_width_respects_theme_minimum() {
        let theme = theme::Theme::default_dark();
        let registry = command::Registry::new();
        let mut measurer = text::layout::Engine::new();
        let menu = menu::Menu::new("Test")
            .key(menu::Id::new("test"))
            .section(menu::Section::new().separator());
        let metrics = popup_chrome(&menu, &registry, &theme, &mut measurer);

        assert_eq!(metrics.area.width(), theme.density().menu_popup_min_width());
    }

    #[test]
    fn menu_popup_width_grows_for_long_labels_and_shortcuts() {
        let theme = theme::Theme::default_dark();
        let mut registry = command::Registry::new();
        let mut measurer = text::layout::Engine::new();
        let short = menu::Menu::new("Short")
            .key(menu::Id::new("short"))
            .section(menu::Section::new().command_key(COMMAND_A));
        let long = menu::Menu::new("Long").key(menu::Id::new("long")).section(
            menu::Section::new().item(
                menu::Item::key(COMMAND_B)
                    .with_label("Ridiculously Long Menu Item Label With Shortcut"),
            ),
        );

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("A"),
        );
        registry.register(
            command::definition::Definition::for_command::<CommandB, command::TestTarget>()
                .with_display("Long")
                .shortcut(command::shortcut::Shortcut::ctrl('a')),
        );

        let short = popup_chrome(&short, &registry, &theme, &mut measurer);
        let long = popup_chrome(&long, &registry, &theme, &mut measurer);

        assert!(long.area.width() > short.area.width());
        assert!(long.shortcut_width > 0.0);
    }

    #[test]
    fn ordinary_menu_popup_uses_wider_default_floor_when_content_is_narrow() {
        let theme = theme::Theme::default_dark();
        let registry = command::Registry::new();
        let mut measurer = text::layout::Engine::new();
        let menu = menu::Menu::new("Preview")
            .key(menu::Id::new("preview"))
            .section(
                menu::Section::new().item(menu::Item::key(COMMAND_A).with_label("Toggle Preview")),
            );
        let chrome = popup_chrome(&menu, &registry, &theme, &mut measurer);

        assert!(chrome.body_width + chrome.padding * 2.0 < theme.density().menu_popup_min_width());
        assert_eq!(chrome.area.width(), theme.density().menu_popup_min_width());
    }

    #[test]
    fn popup_rect_aligns_origin_and_area_to_whole_logical_pixels() {
        let theme = theme::Theme::default_dark();
        let chrome = PopupChrome {
            area: pixel_aligned_area(area::logical(120.2, 44.1)),
            body_width: 108.2,
            row_count: 1,
            shortcut_width: 0.0,
            padding: theme.floating_panel().padding(),
            row_height: theme.density().menu_row_height(),
            glyph_width: glyph_column_width(&theme),
            row_rounding: rect::Rounding::fixed(4.0),
        };
        let rect = popup_rect(
            point::logical(10.4, 20.6),
            chrome,
            &theme,
            Rect::new(point::logical(0.0, 0.0), area::logical(400.0, 300.0)),
        );

        assert_eq!(rect.origin, point::logical(10.0, 21.0));
        assert_eq!(rect.area, area::logical(121.0, 45.0));
    }

    #[test]
    fn menu_row_rounding_derives_from_popup_rounding_minus_padding() {
        let theme = theme::Theme::default_dark();
        let registry = command::Registry::new();
        let mut measurer = text::layout::Engine::new();
        let menu = menu::Menu::new("Test").key(menu::Id::new("test")).section(
            menu::Section::new()
                .command_key(COMMAND_A)
                .separator()
                .command_key(COMMAND_B),
        );
        let chrome = popup_chrome(&menu, &registry, &theme, &mut measurer);
        let popup_rounding = theme.floating_panel().rounding().resolve(chrome.area);
        let row_rounding = chrome
            .row_rounding
            .resolve(area::logical(chrome.body_width, chrome.row_height));

        assert_eq!(
            row_rounding,
            [
                (popup_rounding[0] - chrome.padding).max(0.0),
                (popup_rounding[1] - chrome.padding).max(0.0),
                (popup_rounding[2] - chrome.padding).max(0.0),
                (popup_rounding[3] - chrome.padding).max(0.0),
            ]
        );
    }

    #[test]
    fn menu_row_rounding_clamps_to_zero_when_padding_exceeds_parent_radius() {
        let theme = theme::Theme::default_dark();
        let rounding = row_rounding(&theme, area::logical(120.0, 80.0), 20.0)
            .resolve(area::logical(80.0, 22.0));

        assert_eq!(rounding, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn menu_popup_bottom_empty_space_matches_side_inset() {
        let theme = theme::Theme::default_dark();
        let registry = command::Registry::new();
        let mut measurer = text::layout::Engine::new();
        let menu = menu::Menu::new("Test").key(menu::Id::new("test")).section(
            menu::Section::new()
                .command_key(COMMAND_A)
                .separator()
                .command_key(COMMAND_B),
        );
        let chrome = popup_chrome(&menu, &registry, &theme, &mut measurer);
        let rows_height = chrome.row_height * chrome.row_count as f32;

        assert_eq!(chrome.area.height() - rows_height, chrome.padding * 2.0);
    }

    #[test]
    fn menu_popup_metrics_reuse_cached_label_and_shortcut_measurements() {
        let theme = theme::Theme::default_dark();
        let mut registry = command::Registry::new();
        let mut measurer = text::layout::Engine::new();
        let menu = menu::Menu::new("Test").key(menu::Id::new("test")).section(
            menu::Section::new().item(menu::Item::key(COMMAND_A).with_label("Measured Menu Item")),
        );

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("Measured")
                .shortcut(command::shortcut::Shortcut::ctrl('m')),
        );

        let first = popup_chrome(&menu, &registry, &theme, &mut measurer);
        let uncached = measurer.uncached_measure_count();
        let second = popup_chrome(&menu, &registry, &theme, &mut measurer);

        assert_eq!(first, second);
        assert!(uncached > 0);
        assert_eq!(measurer.uncached_measure_count(), uncached);
    }

    #[test]
    fn menu_row_text_alignment_uses_left_label_and_right_shortcut() {
        let theme = theme::Theme::default_dark();
        let mut registry = command::Registry::new();
        let item = menu::Item::key(COMMAND_A);

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("Select All")
                .shortcut(command::shortcut::Shortcut::ctrl('a')),
        );
        let metrics = test_chrome(&theme, 42.0);
        let row = item_node(
            ui::Id::new("row"),
            &item,
            &registry,
            &surface(),
            metrics,
            &theme,
        );
        let label = row.children()[1]
            .label()
            .expect("menu row should have a label document");
        let shortcut = row.children()[2]
            .label()
            .expect("menu row should have a shortcut document");

        assert_eq!(label.blocks()[0].align(), text::document::Align::Start);
        assert_eq!(shortcut.blocks()[0].align(), text::document::Align::End);
        assert_eq!(
            label.blocks()[0].runs()[0].style().size(),
            theme.text().menu_size()
        );
        assert_eq!(
            shortcut.blocks()[0].runs()[0].style().size(),
            theme.text().menu_size()
        );
    }

    #[test]
    fn menu_row_side_glyph_columns_are_square() {
        let theme = theme::Theme::default_dark();
        let mut registry = command::Registry::new();
        let item = menu::Item::key(COMMAND_A);

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("Select All"),
        );
        let row = item_node(
            ui::Id::new("row"),
            &item,
            &registry,
            &surface(),
            test_chrome(&theme, 0.0),
            &theme,
        );

        assert_eq!(
            row.children()[0].layout().width(),
            layout::Size::Fixed(test_chrome(&theme, 0.0).glyph_width)
        );
        assert_eq!(
            row.children()[3].layout().width(),
            layout::Size::Fixed(test_chrome(&theme, 0.0).glyph_width)
        );
        assert_eq!(
            row.layout().height(),
            layout::Size::Fixed(test_chrome(&theme, 0.0).row_height)
        );
    }

    #[test]
    fn disabled_menu_item_keeps_content_color_tokens_for_paint_projection() {
        let theme = theme::Theme::default_dark();
        let mut registry = command::Registry::new();
        let item = menu::Item::key(COMMAND_A);
        let context = command::call::Context::window(crate::window::Id::new(1));

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("Unavailable"),
        );
        registry.set_state_key(COMMAND_A, context.clone(), command::State::unavailable());

        let row = item_node(
            ui::Id::new("row"),
            &item,
            &registry,
            &surface(),
            test_chrome(&theme, 0.0),
            &theme,
        );

        assert_eq!(row.style().disabled_tint(), None);
        assert_eq!(row.style().label_color(), Some(theme.text().primary()));
        assert_eq!(
            row.style().disabled_label_color(),
            Some(theme.text().disabled())
        );
        for child in row.children() {
            assert_eq!(child.style().label_color(), Some(theme.text().primary()));
            assert_eq!(
                child.style().disabled_label_color(),
                Some(theme.text().disabled())
            );
        }
    }

    #[test]
    fn disabled_menu_item_paints_label_and_shortcut_with_disabled_color() {
        let theme = theme::Theme::default_dark();
        let mut registry = command::Registry::new();
        let item = menu::Item::key(COMMAND_A);
        let window = crate::window::Id::new(1);
        let context = command::call::Context::window(window);
        let row_id = ui::Id::new("row");
        let row_path = ui::Path::from(row_id);

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("Select All")
                .shortcut(command::shortcut::Shortcut::ctrl('a')),
        );
        registry.set_state_key(COMMAND_A, context.clone(), command::State::unavailable());

        let row = item_node(
            row_id,
            &item,
            &registry,
            &surface(),
            test_chrome(&theme, 42.0),
            &theme,
        );
        let mut scene = paint::Scene::new();

        paint_with_visual_states(
            row,
            ui::Interaction::default(),
            [(row_path, ui::VisualState::unavailable())],
            &mut scene,
        );

        let label = scene
            .items()
            .iter()
            .filter_map(|item| match item {
                paint::Item::Text(text) => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(label.len(), 2);
        for text in label {
            assert_eq!(
                text.document.blocks()[0].runs()[0].style().color(),
                text_color(theme.text().disabled())
            );
        }
    }

    #[test]
    fn enabled_menu_item_paints_label_and_shortcut_with_primary_color() {
        let theme = theme::Theme::default_dark();
        let mut registry = command::Registry::new();
        let item = menu::Item::key(COMMAND_A);

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("Select All")
                .shortcut(command::shortcut::Shortcut::ctrl('a')),
        );

        let row = item_node(
            ui::Id::new("row"),
            &item,
            &registry,
            &surface(),
            test_chrome(&theme, 42.0),
            &theme,
        );
        let mut tree = ui::Tree::new();
        let mut measurer = text::layout::Engine::new();
        let mut scene = paint::Scene::new();

        tree.set_root(row);
        let layout = tree
            .layout(area::logical(160.0, 40.0), &mut measurer)
            .expect("menu row should layout");
        tree.paint(&layout, ui::Interaction::default(), &mut scene);

        let label = scene
            .items()
            .iter()
            .filter_map(|item| match item {
                paint::Item::Text(text) => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(label.len(), 2);
        for text in label {
            assert_eq!(
                text.document.blocks()[0].runs()[0].style().color(),
                text_color(theme.text().primary())
            );
        }
    }

    #[test]
    fn disabled_menu_item_suppresses_hover_and_press_tints() {
        let theme = theme::Theme::default_dark();
        let mut registry = command::Registry::new();
        let item = menu::Item::key(COMMAND_A);
        let window = crate::window::Id::new(1);
        let context = command::call::Context::window(window);
        let row_id = ui::Id::new("row");
        let row_path = ui::Path::from(row_id);

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("Select All"),
        );
        registry.set_state_key(COMMAND_A, context.clone(), command::State::unavailable());

        let row = item_node(
            row_id,
            &item,
            &registry,
            &surface(),
            test_chrome(&theme, 0.0),
            &theme,
        );
        let mut scene = paint::Scene::new();

        paint_with_visual_states(
            row,
            ui::Interaction::new(Some(row_path.clone()), None, Some(row_path)),
            [(ui::Path::from(row_id), ui::VisualState::unavailable())],
            &mut scene,
        );

        assert!(
            scene
                .items()
                .iter()
                .all(|item| !matches!(item, paint::Item::Tint(_)))
        );
    }

    #[test]
    fn enabled_menu_item_hover_emits_hover_tint() {
        let theme = theme::Theme::default_dark();
        let mut registry = command::Registry::new();
        let item = menu::Item::key(COMMAND_A);
        let row_id = ui::Id::new("row");
        let row_path = ui::Path::from(row_id);

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("Select All"),
        );

        let row = item_node(
            row_id,
            &item,
            &registry,
            &surface(),
            test_chrome(&theme, 0.0),
            &theme,
        );
        let mut tree = ui::Tree::new();
        let mut measurer = text::layout::Engine::new();
        let mut scene = paint::Scene::new();

        tree.set_root(row);
        let layout = tree
            .layout(area::logical(160.0, 40.0), &mut measurer)
            .expect("menu row should layout");
        tree.paint(
            &layout,
            ui::Interaction::new(Some(row_path), None, None),
            &mut scene,
        );

        assert!(
            scene
                .items()
                .iter()
                .any(|item| matches!(item, paint::Item::Tint(tint) if tint.brush == theme.menu().row_hover_tint()))
        );
    }

    #[test]
    fn disabled_submenu_row_dims_content_without_disabled_tint() {
        let theme = theme::Theme::default_dark();
        let registry = command::Registry::new();
        let submenu = menu::Menu::new("Disabled")
            .key(menu::Id::new("disabled"))
            .section(menu::Section::new().item(menu::Item::key(COMMAND_A)));

        let row = submenu_node(
            ui::Id::new("row"),
            &submenu,
            &registry,
            &surface(),
            test_chrome(&theme, 0.0),
            &theme,
        );

        assert_eq!(row.style().disabled_tint(), None);
        assert_eq!(row.style().label_color(), Some(theme.text().disabled()));
        assert!(row.interactivity().hit_test());
        assert!(!row.interactivity().actionable());
    }

    #[test]
    fn menu_separator_row_centers_thin_divider_line() {
        let theme = theme::Theme::default_dark();
        let chrome = test_chrome(&theme, 0.0);
        let row = separator_node(ui::Id::new("separator"), chrome, &theme);
        let line = row
            .children()
            .first()
            .expect("separator row should own a line");

        assert_eq!(
            row.layout().height(),
            layout::Size::Fixed(chrome.row_height)
        );
        assert_eq!(line.layout().height(), layout::Size::Fixed(1.0));
        assert_eq!(row.intent(), Some(ui::Intent::CloseSubmenu));
        assert!(row.interactivity().hit_test());
        assert!(!row.interactivity().focusable());
        assert!(!row.interactivity().actionable());
    }

    fn test_chrome(theme: &theme::Theme, shortcut_width: f32) -> PopupChrome {
        let padding = theme.floating_panel().padding();
        let row_height = theme.density().menu_row_height();
        let glyph_width = glyph_column_width(theme);
        let body_width = glyph_width * 2.0 + shortcut_width + 80.0;
        let area = area::logical(body_width + padding * 2.0, row_height + padding * 2.0);

        PopupChrome {
            area,
            body_width,
            row_count: 1,
            shortcut_width,
            padding,
            row_height,
            glyph_width,
            row_rounding: row_rounding(theme, area, padding),
        }
    }
}
