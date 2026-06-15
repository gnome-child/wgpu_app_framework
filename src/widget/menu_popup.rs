use crate::geometry::{Rect, area, point, rect};
use crate::{action, icon, layout, menu, paint, text, theme, ui};

use super::{
    MENU_POPUP, MENU_SUBMENU_POPUP, Popup, floating_panel_with_theme, separator_with_theme,
};

const SEPARATOR_LINE_HEIGHT: f32 = 1.0;
const SUBMENU_GAP: f32 = 3.0;

const ROW_GLYPH: ui::Id = ui::Id::new("__menu_row_glyph");
const ROW_LABEL: ui::Id = ui::Id::new("__menu_row_label");
const ROW_SHORTCUT: ui::Id = ui::Id::new("__menu_row_shortcut");
const ROW_TRAILING: ui::Id = ui::Id::new("__menu_row_trailing");
const ROW_SEPARATOR: ui::Id = ui::Id::new("__menu_row_separator");

const ROW_IDS: [ui::Id; 32] = [
    ui::Id::new("__menu_row_00"),
    ui::Id::new("__menu_row_01"),
    ui::Id::new("__menu_row_02"),
    ui::Id::new("__menu_row_03"),
    ui::Id::new("__menu_row_04"),
    ui::Id::new("__menu_row_05"),
    ui::Id::new("__menu_row_06"),
    ui::Id::new("__menu_row_07"),
    ui::Id::new("__menu_row_08"),
    ui::Id::new("__menu_row_09"),
    ui::Id::new("__menu_row_10"),
    ui::Id::new("__menu_row_11"),
    ui::Id::new("__menu_row_12"),
    ui::Id::new("__menu_row_13"),
    ui::Id::new("__menu_row_14"),
    ui::Id::new("__menu_row_15"),
    ui::Id::new("__menu_row_16"),
    ui::Id::new("__menu_row_17"),
    ui::Id::new("__menu_row_18"),
    ui::Id::new("__menu_row_19"),
    ui::Id::new("__menu_row_20"),
    ui::Id::new("__menu_row_21"),
    ui::Id::new("__menu_row_22"),
    ui::Id::new("__menu_row_23"),
    ui::Id::new("__menu_row_24"),
    ui::Id::new("__menu_row_25"),
    ui::Id::new("__menu_row_26"),
    ui::Id::new("__menu_row_27"),
    ui::Id::new("__menu_row_28"),
    ui::Id::new("__menu_row_29"),
    ui::Id::new("__menu_row_30"),
    ui::Id::new("__menu_row_31"),
];

pub fn menu_popup<T>(
    tree: &ui::Tree,
    layout: &layout::Box,
    menu: &menu::Menu,
    actions: &action::Registry<T>,
    command_target: &action::Context,
    measurer: &mut text::Measurer,
) -> Option<Popup> {
    let theme = theme::Theme::default_dark();
    let anchor = anchor_rect(tree, layout, menu.id(), AnchorKind::TopLevel)?;
    let chrome = popup_chrome(menu, actions, &theme, measurer);
    let popup_rect = popup_rect(
        point::logical(anchor.origin.x(), anchor.origin.y() + anchor.area.height()),
        chrome,
        &theme,
    );

    Some(Popup::new(
        popup_rect,
        popup_node(MENU_POPUP, menu, actions, command_target, chrome, &theme),
    ))
}

pub fn submenu_popup<T>(
    tree: &ui::Tree,
    layout: &layout::Box,
    menu: &menu::Menu,
    actions: &action::Registry<T>,
    command_target: &action::Context,
    measurer: &mut text::Measurer,
) -> Option<Popup> {
    let theme = theme::Theme::default_dark();
    let anchor = anchor_rect(tree, layout, menu.id(), AnchorKind::Submenu)?;
    let padding = theme.floating_panel().padding();
    let chrome = popup_chrome(menu, actions, &theme, measurer);
    let popup_rect = popup_rect(
        point::logical(
            anchor.origin.x() + anchor.area.width() + SUBMENU_GAP,
            anchor.origin.y() - padding,
        ),
        chrome,
        &theme,
    );

    Some(Popup::new(
        popup_rect,
        popup_node(
            MENU_SUBMENU_POPUP,
            menu,
            actions,
            command_target,
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
    layout: &layout::Box,
    id: menu::Id,
    kind: AnchorKind,
) -> Option<Rect> {
    tree.intents()
        .into_iter()
        .find_map(|(path, intent)| match (kind, intent) {
            (AnchorKind::TopLevel, ui::Intent::OpenMenu(menu)) if menu == id => {
                layout.find_path(&path).map(layout::Box::rect)
            }
            (AnchorKind::Submenu, ui::Intent::OpenSubmenu(menu)) if menu == id => {
                layout.find_path(&path).map(layout::Box::rect)
            }
            _ => None,
        })
}

fn popup_rect(origin: point::Logical, chrome: PopupChrome, theme: &theme::Theme) -> Rect {
    Rect::rounded(
        pixel_aligned_origin(origin),
        chrome.area,
        theme.floating_panel().rounding(),
    )
}

fn popup_node<T>(
    id: ui::Id,
    menu: &menu::Menu,
    actions: &action::Registry<T>,
    command_target: &action::Context,
    chrome: PopupChrome,
    theme: &theme::Theme,
) -> ui::Node {
    let mut popup = floating_panel_with_theme(id, theme)
        .with_command_scope()
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
                    item_node(id, item, actions, command_target, chrome, theme)
                }
                menu::Node::Submenu(menu) => {
                    let id = row_id(row);
                    row += 1;
                    submenu_node(id, menu, actions, command_target, chrome, theme)
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

fn item_node<T>(
    id: ui::Id,
    item: &menu::Item,
    actions: &action::Registry<T>,
    command_target: &action::Context,
    chrome: PopupChrome,
    theme: &theme::Theme,
) -> ui::Node {
    let action = item.action();
    let state = actions.state(action, command_target.clone());
    let color = if state.is_enabled() {
        theme.text().primary()
    } else {
        theme.text().disabled()
    };
    let check = state
        .is_active()
        .then(|| icon::Icon::phosphor(icon::Id::new("check")));
    let shortcut = shortcut_label(action, actions);

    menu_row(id, chrome, theme, color)
        .with_action(action)
        .with_action_target(ui::ActionTarget::Captured)
        .with_child(glyph_cell(
            ROW_GLYPH,
            check,
            chrome.glyph_width,
            color,
            theme,
        ))
        .with_child(text_cell(
            ROW_LABEL,
            item_label(item, actions),
            text::Align::Start,
            layout::Size::Fill,
            color,
            theme,
        ))
        .with_child(text_cell(
            ROW_SHORTCUT,
            shortcut.unwrap_or_default(),
            text::Align::End,
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

fn submenu_node<T>(
    id: ui::Id,
    menu: &menu::Menu,
    actions: &action::Registry<T>,
    command_target: &action::Context,
    chrome: PopupChrome,
    theme: &theme::Theme,
) -> ui::Node {
    let enabled = menu_can_open(menu, actions, command_target);
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
            text::Align::Start,
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
    ui::Node::container(id, layout::Axis::Horizontal)
        .with_intent(ui::Intent::CloseSubmenu)
        .with_interactivity(ui::Interactivity::CONTROL)
        .with_background(theme.menu().row_background())
        .with_focus_background(theme.menu().row_hover_tint())
        .with_hover_tint(theme.menu().row_hover_tint())
        .with_pressed_tint(theme.menu().row_pressed_tint())
        .with_disabled_tint(theme.menu().row_disabled_tint())
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
    let mut node = ui::Node::leaf(id)
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

fn text_cell(
    id: ui::Id,
    label: impl Into<String>,
    align: text::Align,
    width: layout::Size,
    color: paint::Color,
    theme: &theme::Theme,
) -> ui::Node {
    ui::Node::leaf(id)
        .with_label(document(label, theme, align, color))
        .with_label_color(color)
        .with_disabled_label_color(theme.text().disabled())
        .with_size(width, layout::Size::Fill)
}

fn separator_node(id: ui::Id, chrome: PopupChrome, theme: &theme::Theme) -> ui::Node {
    ui::Node::container(id, layout::Axis::Vertical)
        .with_intent(ui::Intent::CloseSubmenu)
        .with_interactivity(ui::Interactivity::NONE.with_hit_test(true))
        .with_align(layout::Align::Center)
        .with_cross_align(layout::Align::Stretch)
        .with_size(layout::Size::Fill, layout::Size::Fixed(chrome.row_height))
        .with_child(separator_with_theme(ROW_SEPARATOR, theme).with_size(
            layout::Size::Fill,
            layout::Size::Fixed(SEPARATOR_LINE_HEIGHT),
        ))
}

fn item_label<T>(item: &menu::Item, actions: &action::Registry<T>) -> String {
    item.label()
        .map(str::to_owned)
        .or_else(|| {
            actions
                .action(item.action())
                .map(|action| action.label().to_owned())
        })
        .unwrap_or_else(|| item.action().as_str().replace('_', " "))
}

fn shortcut_label<T>(action: action::Id, actions: &action::Registry<T>) -> Option<String> {
    actions
        .action(action)
        .and_then(|action| action.shortcuts().first())
        .copied()
        .map(action::Shortcut::display_label)
}

fn document(
    label: impl Into<String>,
    theme: &theme::Theme,
    align: text::Align,
    color: paint::Color,
) -> text::Document {
    let mut block = text::Block::new(align);
    block.push_run(text::Run::new(
        label,
        text::Style::default()
            .with_size(theme.text().menu_size())
            .with_color(color),
    ));

    text::Document::from_block(block)
}

fn popup_chrome<T>(
    menu: &menu::Menu,
    actions: &action::Registry<T>,
    theme: &theme::Theme,
    measurer: &mut text::Measurer,
) -> PopupChrome {
    let mut label_width = 0.0_f32;
    let mut shortcut_width = 0.0_f32;
    let mut row_count = 0_usize;

    for node in menu.sections().iter().flat_map(menu::Section::nodes) {
        row_count += 1;
        match node {
            menu::Node::Item(item) => {
                label_width =
                    label_width.max(measure_label(item_label(item, actions), theme, measurer));
                if let Some(shortcut) = shortcut_label(item.action(), actions) {
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
    measurer: &mut text::Measurer,
) -> f32 {
    measurer
        .measure(
            &document(label, theme, text::Align::Start, theme.text().primary()),
            text::Measure::unbounded(),
        )
        .width()
}

fn menu_can_open<T>(
    menu: &menu::Menu,
    actions: &action::Registry<T>,
    command_target: &action::Context,
) -> bool {
    menu.actions()
        .any(|action| actions.can_invoke(action, command_target.clone()))
}

fn row_id(index: usize) -> ui::Id {
    ROW_IDS
        .get(index)
        .copied()
        .unwrap_or(ui::Id::new("__menu_row_overflow"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTION_A: action::Id = action::Id::new("menu_action_a");
    const ACTION_B: action::Id = action::Id::new("menu_action_b");

    #[test]
    fn menu_separator_occupies_normal_row_height() {
        let theme = theme::Theme::default_dark();
        let registry = action::Registry::<()>::new();
        let mut measurer = text::Measurer::new();
        let menu = menu::Menu::new(menu::Id::new("test"), "Test").section(
            menu::Section::new()
                .action(ACTION_A)
                .separator()
                .action(ACTION_B),
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
        let registry = action::Registry::<()>::new();
        let mut measurer = text::Measurer::new();
        let menu = menu::Menu::new(menu::Id::new("test"), "Test")
            .section(menu::Section::new().separator());
        let metrics = popup_chrome(&menu, &registry, &theme, &mut measurer);

        assert_eq!(metrics.area.width(), theme.density().menu_popup_min_width());
    }

    #[test]
    fn menu_popup_width_grows_for_long_labels_and_shortcuts() {
        let theme = theme::Theme::default_dark();
        let mut registry = action::Registry::<()>::new();
        let mut measurer = text::Measurer::new();
        let short = menu::Menu::new(menu::Id::new("short"), "Short")
            .section(menu::Section::new().action(ACTION_A));
        let long = menu::Menu::new(menu::Id::new("long"), "Long").section(
            menu::Section::new().item(
                menu::Item::new(ACTION_B)
                    .with_label("Ridiculously Long Menu Item Label With Shortcut"),
            ),
        );

        registry.register(crate::Action::new(ACTION_A, "A"));
        registry.register(
            crate::Action::new(ACTION_B, "Long").with_shortcut(action::Shortcut::control('a')),
        );

        let short = popup_chrome(&short, &registry, &theme, &mut measurer);
        let long = popup_chrome(&long, &registry, &theme, &mut measurer);

        assert!(long.area.width() > short.area.width());
        assert!(long.shortcut_width > 0.0);
    }

    #[test]
    fn ordinary_menu_popup_uses_wider_default_floor_when_content_is_narrow() {
        let theme = theme::Theme::default_dark();
        let registry = action::Registry::<()>::new();
        let mut measurer = text::Measurer::new();
        let menu = menu::Menu::new(menu::Id::new("preview"), "Preview").section(
            menu::Section::new().item(menu::Item::new(ACTION_A).with_label("Toggle Preview")),
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
        let rect = popup_rect(point::logical(10.4, 20.6), chrome, &theme);

        assert_eq!(rect.origin, point::logical(10.0, 21.0));
        assert_eq!(rect.area, area::logical(121.0, 45.0));
    }

    #[test]
    fn menu_row_rounding_derives_from_popup_rounding_minus_padding() {
        let theme = theme::Theme::default_dark();
        let registry = action::Registry::<()>::new();
        let mut measurer = text::Measurer::new();
        let menu = menu::Menu::new(menu::Id::new("test"), "Test").section(
            menu::Section::new()
                .action(ACTION_A)
                .separator()
                .action(ACTION_B),
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
        let registry = action::Registry::<()>::new();
        let mut measurer = text::Measurer::new();
        let menu = menu::Menu::new(menu::Id::new("test"), "Test").section(
            menu::Section::new()
                .action(ACTION_A)
                .separator()
                .action(ACTION_B),
        );
        let chrome = popup_chrome(&menu, &registry, &theme, &mut measurer);
        let rows_height = chrome.row_height * chrome.row_count as f32;

        assert_eq!(chrome.area.height() - rows_height, chrome.padding * 2.0);
    }

    #[test]
    fn menu_popup_metrics_reuse_cached_label_and_shortcut_measurements() {
        let theme = theme::Theme::default_dark();
        let mut registry = action::Registry::<()>::new();
        let mut measurer = text::Measurer::new();
        let menu = menu::Menu::new(menu::Id::new("test"), "Test").section(
            menu::Section::new().item(menu::Item::new(ACTION_A).with_label("Measured Menu Item")),
        );

        registry.register(
            crate::Action::new(ACTION_A, "Measured").with_shortcut(action::Shortcut::control('m')),
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
        let mut registry = action::Registry::<()>::new();
        let item = menu::Item::new(ACTION_A);
        let context = action::Context::window(crate::window::Id::new(1));

        registry.register(
            crate::Action::new(ACTION_A, "Select All")
                .with_shortcut(action::Shortcut::control('a')),
        );
        let metrics = test_chrome(&theme, 42.0);
        let row = item_node(
            ui::Id::new("row"),
            &item,
            &registry,
            &context,
            metrics,
            &theme,
        );
        let label = row.children()[1]
            .label()
            .expect("menu row should have a label document");
        let shortcut = row.children()[2]
            .label()
            .expect("menu row should have a shortcut document");

        assert_eq!(label.blocks()[0].align(), text::Align::Start);
        assert_eq!(shortcut.blocks()[0].align(), text::Align::End);
    }

    #[test]
    fn menu_row_side_glyph_columns_are_square() {
        let theme = theme::Theme::default_dark();
        let mut registry = action::Registry::<()>::new();
        let item = menu::Item::new(ACTION_A);
        let context = action::Context::window(crate::window::Id::new(1));

        registry.register(crate::Action::new(ACTION_A, "Select All"));
        let row = item_node(
            ui::Id::new("row"),
            &item,
            &registry,
            &context,
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
