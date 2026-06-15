use crate::geometry::{Rect, area, point, rect};
use crate::{action, layout, menu, paint, text, ui};

const POPUP_WIDTH: f32 = 240.0;
const POPUP_PADDING: f32 = 8.0;
const ROW_HEIGHT: f32 = 30.0;
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
    let anchor = anchor_rect(tree, layout, menu.id())?;
    let body_height = body_height(menu);
    let popup_rect = Rect::rounded(
        point::logical(anchor.origin.x(), anchor.origin.y() + anchor.area.height()),
        area::logical(POPUP_WIDTH, body_height + POPUP_PADDING * 2.0),
        rect::Radius::splat(0.10),
    );

    Some(ui::Popup::new(
        popup_rect,
        popup_node(menu, actions, popup_rect.area.height()),
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

fn popup_node<T>(menu: &menu::Menu, actions: &action::Registry<T>, height: f32) -> ui::Node {
    let mut popup = ui::Node::container(ui::widget::MENU_POPUP, layout::Axis::Vertical)
        .with_command_scope()
        .with_backdrop(
            ui::Backdrop::glass(paint::Brush::linear_gradient(
                paint::Color::rgba(0.10, 0.11, 0.13, 0.36),
                paint::Color::rgba(0.18, 0.20, 0.26, 0.46),
            ))
            .with_blur(0.82),
        )
        .with_stroke(paint::Stroke {
            brush: paint::Brush::linear_gradient(
                paint::Color::rgba(1.0, 1.0, 1.0, 0.10),
                paint::Color::rgba(1.0, 1.0, 1.0, 0.24),
            ),
            width: 1.0,
        })
        .with_shadow(
            paint::Brush::linear_gradient(
                paint::Color::rgba(0.0, 0.0, 0.0, 0.20),
                paint::Color::rgba(0.0, 0.0, 0.0, 0.46),
            ),
            18.0,
            1.0,
            point::logical(0.0, 7.0),
        )
        .with_radius(rect::Radius::splat(0.10))
        .with_padding(layout::Insets::splat(POPUP_PADDING))
        .with_size(
            layout::Size::Fixed(POPUP_WIDTH),
            layout::Size::Fixed(height),
        );

    let mut row = 0;
    for section in menu.sections() {
        for node in section.nodes() {
            popup.push_child(match node {
                menu::Node::Item(item) => {
                    let id = row_id(row);
                    row += 1;
                    item_node(id, item, actions)
                }
                menu::Node::Separator => {
                    let id = row_id(row);
                    row += 1;
                    separator_node(id)
                }
            });
        }
    }

    popup
}

fn item_node<T>(id: ui::Id, item: &menu::Item, actions: &action::Registry<T>) -> ui::Node {
    let action = item.action();
    ui::Node::leaf(id)
        .with_action(action)
        .with_action_target(ui::ActionTarget::Captured)
        .with_interactivity(ui::Interactivity::CONTROL)
        .with_label(document(item_label(item, actions)))
        .with_background(paint::Color::rgba(1.0, 1.0, 1.0, 0.00))
        .with_hover_tint(paint::Color::rgba(1.0, 1.0, 1.0, 0.10))
        .with_pressed_tint(paint::Color::rgba(0.0, 0.0, 0.0, 0.18))
        .with_disabled_tint(paint::Color::rgba(0.0, 0.0, 0.0, 0.30))
        .with_label_color(paint::Color::rgb(0.91, 0.93, 0.97))
        .with_disabled_label_color(paint::Color::rgb(0.44, 0.46, 0.50))
        .with_radius(rect::Radius::splat(0.16))
        .with_size(layout::Size::Fill, layout::Size::Fixed(ROW_HEIGHT))
}

fn separator_node(id: ui::Id) -> ui::Node {
    ui::widget::separator(id)
        .with_background(paint::Color::rgba(1.0, 1.0, 1.0, 0.14))
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

fn document(label: impl Into<String>) -> text::Document {
    let mut block = text::Block::new(text::Align::Center);
    block.push_run(text::Run::new(label, text::Style::default()));

    text::Document::from_block(block)
}

fn body_height(menu: &menu::Menu) -> f32 {
    menu.sections()
        .iter()
        .flat_map(menu::Section::nodes)
        .map(|node| match node {
            menu::Node::Item(_) => ROW_HEIGHT,
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
