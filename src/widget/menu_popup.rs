use crate::geometry::{Rect, area, point};
use crate::{action, icon, layout, menu, paint, text, theme, ui};

use super::{
    MENU_POPUP, MENU_SUBMENU_POPUP, Popup, floating_panel_with_theme, separator_with_theme,
};

const SEPARATOR_HEIGHT: f32 = 1.0;
const ROW_ICON_WIDTH: f32 = 28.0;
const ROW_SHORTCUT_WIDTH: f32 = 72.0;
const ROW_TRAILING_WIDTH: f32 = 22.0;
const SUBMENU_GAP: f32 = 3.0;

const ROW_GLYPH: ui::Id = ui::Id::new("__menu_row_glyph");
const ROW_LABEL: ui::Id = ui::Id::new("__menu_row_label");
const ROW_SHORTCUT: ui::Id = ui::Id::new("__menu_row_shortcut");
const ROW_TRAILING: ui::Id = ui::Id::new("__menu_row_trailing");

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
) -> Option<Popup> {
    let theme = theme::Theme::default_dark();
    let anchor = anchor_rect(tree, layout, menu.id(), AnchorKind::TopLevel)?;
    let popup_rect = popup_rect(
        point::logical(anchor.origin.x(), anchor.origin.y() + anchor.area.height()),
        menu,
        &theme,
    );

    Some(Popup::new(
        popup_rect,
        popup_node(
            MENU_POPUP,
            menu,
            actions,
            command_target,
            popup_rect.area.height(),
            &theme,
        ),
    ))
}

pub fn submenu_popup<T>(
    tree: &ui::Tree,
    layout: &layout::Box,
    menu: &menu::Menu,
    actions: &action::Registry<T>,
    command_target: &action::Context,
) -> Option<Popup> {
    let theme = theme::Theme::default_dark();
    let anchor = anchor_rect(tree, layout, menu.id(), AnchorKind::Submenu)?;
    let padding = theme.floating_panel().padding();
    let popup_rect = popup_rect(
        point::logical(
            anchor.origin.x() + anchor.area.width() + SUBMENU_GAP,
            anchor.origin.y() - padding,
        ),
        menu,
        &theme,
    );

    Some(Popup::new(
        popup_rect,
        popup_node(
            MENU_SUBMENU_POPUP,
            menu,
            actions,
            command_target,
            popup_rect.area.height(),
            &theme,
        ),
    ))
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

fn popup_rect(origin: point::Logical, menu: &menu::Menu, theme: &theme::Theme) -> Rect {
    Rect::rounded(
        origin,
        area::logical(
            theme.density().menu_popup_width(),
            body_height(menu, theme) + theme.floating_panel().padding() * 2.0,
        ),
        theme.floating_panel().radius(),
    )
}

fn popup_node<T>(
    id: ui::Id,
    menu: &menu::Menu,
    actions: &action::Registry<T>,
    command_target: &action::Context,
    height: f32,
    theme: &theme::Theme,
) -> ui::Node {
    let mut popup = floating_panel_with_theme(id, theme)
        .with_command_scope()
        .with_size(
            layout::Size::Fixed(theme.density().menu_popup_width()),
            layout::Size::Fixed(height),
        );

    let mut row = 0;
    for section in menu.sections() {
        for node in section.nodes() {
            popup.push_child(match node {
                menu::Node::Item(item) => {
                    let id = row_id(row);
                    row += 1;
                    item_node(id, item, actions, command_target, theme)
                }
                menu::Node::Submenu(menu) => {
                    let id = row_id(row);
                    row += 1;
                    submenu_node(id, menu, actions, command_target, theme)
                }
                menu::Node::Separator => {
                    let id = row_id(row);
                    row += 1;
                    separator_node(id, theme)
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

    menu_row(id, theme, color)
        .with_action(action)
        .with_action_target(ui::ActionTarget::Captured)
        .with_child(glyph_cell(ROW_GLYPH, check, ROW_ICON_WIDTH, color, theme))
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
            layout::Size::Fixed(ROW_SHORTCUT_WIDTH),
            color,
            theme,
        ))
        .with_child(glyph_cell(
            ROW_TRAILING,
            None,
            ROW_TRAILING_WIDTH,
            color,
            theme,
        ))
}

fn submenu_node<T>(
    id: ui::Id,
    menu: &menu::Menu,
    actions: &action::Registry<T>,
    command_target: &action::Context,
    theme: &theme::Theme,
) -> ui::Node {
    let enabled = menu_can_open(menu, actions, command_target);
    let color = if enabled {
        theme.text().primary()
    } else {
        theme.text().disabled()
    };
    let chevron = enabled.then(|| icon::Icon::phosphor(icon::Id::new("caret-right")));
    let row = menu_row(id, theme, color)
        .with_child(glyph_cell(ROW_GLYPH, None, ROW_ICON_WIDTH, color, theme))
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
            ROW_SHORTCUT_WIDTH,
            color,
            theme,
        ))
        .with_child(glyph_cell(
            ROW_TRAILING,
            chevron,
            ROW_TRAILING_WIDTH,
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

fn menu_row(id: ui::Id, theme: &theme::Theme, color: paint::Color) -> ui::Node {
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
        .with_radius(theme.radii().menu_item())
        .with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().menu_row_height()),
        )
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

fn separator_node(id: ui::Id, theme: &theme::Theme) -> ui::Node {
    separator_with_theme(id, theme)
        .with_intent(ui::Intent::CloseSubmenu)
        .with_size(layout::Size::Fill, layout::Size::Fixed(SEPARATOR_HEIGHT))
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

fn body_height(menu: &menu::Menu, theme: &theme::Theme) -> f32 {
    menu.sections()
        .iter()
        .flat_map(menu::Section::nodes)
        .map(|node| match node {
            menu::Node::Item(_) | menu::Node::Submenu(_) => theme.density().menu_row_height(),
            menu::Node::Separator => SEPARATOR_HEIGHT,
        })
        .sum()
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
