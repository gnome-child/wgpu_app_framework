use crate::geometry::{Rect, area, point};
use crate::{action, layout, menu, text, theme, ui};

const SEPARATOR_HEIGHT: f32 = 1.0;

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

pub fn popup<T>(
    tree: &ui::Tree,
    layout: &layout::Box,
    menu: &menu::Menu,
    actions: &action::Registry<T>,
) -> Option<ui::Popup> {
    let theme = theme::Theme::default_dark();
    let anchor = anchor_rect(tree, layout, menu.id())?;
    let body_height = body_height(menu, &theme);
    let density = theme.density();
    let popup_rect = Rect::rounded(
        point::logical(anchor.origin.x(), anchor.origin.y() + anchor.area.height()),
        area::logical(
            density.menu_popup_width(),
            body_height + theme.floating_panel().padding() * 2.0,
        ),
        theme.floating_panel().radius(),
    );

    Some(ui::Popup::new(
        popup_rect,
        popup_node(menu, actions, popup_rect.area.height(), &theme),
    ))
}

fn anchor_rect(tree: &ui::Tree, layout: &layout::Box, id: menu::Id) -> Option<Rect> {
    tree.intents()
        .into_iter()
        .find_map(|(path, intent)| match intent {
            ui::Intent::OpenMenu(menu) if menu == id => {
                layout.find_path(&path).map(layout::Box::rect)
            }
            _ => None,
        })
}

fn popup_node<T>(
    menu: &menu::Menu,
    actions: &action::Registry<T>,
    height: f32,
    theme: &theme::Theme,
) -> ui::Node {
    let mut popup = ui::control::floating_panel_with_theme(ui::widget::MENU_POPUP, theme)
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
                    item_node(id, item, actions, theme)
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
    theme: &theme::Theme,
) -> ui::Node {
    let action = item.action();
    ui::Node::leaf(id)
        .with_action(action)
        .with_action_target(ui::ActionTarget::Captured)
        .with_interactivity(ui::Interactivity::CONTROL)
        .with_label(document(item_label(item, actions), theme))
        .with_background(theme.menu().row_background())
        .with_hover_tint(theme.menu().row_hover_tint())
        .with_pressed_tint(theme.menu().row_pressed_tint())
        .with_disabled_tint(theme.menu().row_disabled_tint())
        .with_label_color(theme.text().primary())
        .with_disabled_label_color(theme.text().disabled())
        .with_radius(theme.radii().menu_item())
        .with_size(
            layout::Size::Fill,
            layout::Size::Fixed(theme.density().menu_row_height()),
        )
}

fn separator_node(id: ui::Id, theme: &theme::Theme) -> ui::Node {
    ui::widget::separator_with_theme(id, theme)
        .with_size(layout::Size::Fill, layout::Size::Fixed(SEPARATOR_HEIGHT))
}

fn item_label<T>(item: &menu::Item, actions: &action::Registry<T>) -> String {
    let label = item
        .label()
        .map(str::to_owned)
        .or_else(|| {
            actions
                .action(item.action())
                .map(|action| action.label().to_owned())
        })
        .unwrap_or_else(|| item.action().as_str().replace('_', " "));
    let shortcut = actions
        .action(item.action())
        .and_then(|action| action.shortcuts().first())
        .copied()
        .map(action::Shortcut::display_label);

    match shortcut {
        Some(shortcut) => format!("{label}    {shortcut}"),
        None => label,
    }
}

fn document(label: impl Into<String>, theme: &theme::Theme) -> text::Document {
    let mut block = text::Block::new(text::Align::Center);
    block.push_run(text::Run::new(
        label,
        text::Style::default()
            .with_size(theme.text().menu_size())
            .with_color(theme.text().primary()),
    ));

    text::Document::from_block(block)
}

fn body_height(menu: &menu::Menu, theme: &theme::Theme) -> f32 {
    menu.sections()
        .iter()
        .flat_map(menu::Section::nodes)
        .map(|node| match node {
            menu::Node::Item(_) => theme.density().menu_row_height(),
            menu::Node::Separator => SEPARATOR_HEIGHT,
        })
        .sum()
}

fn row_id(index: usize) -> ui::Id {
    ROW_IDS
        .get(index)
        .copied()
        .unwrap_or(ui::Id::new("__menu_row_overflow"))
}
