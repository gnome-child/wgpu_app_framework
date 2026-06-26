use crate::geometry::{Rect, area, point, rect};
use crate::{command, icon, layout, paint, text, theme, ui};

use super::{
    MENU_POPUP, MENU_SUBMENU_POPUP, Popup, TEXT_CONTEXT_MENU_POPUP, floating_panel_with_theme,
    menu, separator_with_theme,
};

const SEPARATOR_LINE_HEIGHT: f32 = 1.0;
const SUBMENU_GAP: f32 = 3.0;

const ROW_GLYPH: ui::Id = ui::Id::new("__menu_row_glyph");
const ROW_LABEL: ui::Id = ui::Id::new("__menu_row_label");
const ROW_SHORTCUT: ui::Id = ui::Id::new("__menu_row_shortcut");
const ROW_TRAILING: ui::Id = ui::Id::new("__menu_row_trailing");
const ROW_SEPARATOR: ui::Id = ui::Id::new("__menu_row_separator");
const TEXT_CONTEXT_MENU: menu::Id = menu::Id::new("__text_context_menu");

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

pub fn menu_popup(
    tree: &ui::Tree,
    layout: &ui::Frame,
    surface: &ui::floating::Surface,
    menu: &menu::Menu,
    commands: &command::Registry,
    measurer: &mut text::layout::Engine,
) -> Option<Popup> {
    let theme = theme::Theme::default_dark();
    let anchor = anchor_rect(tree, layout, menu.id(), AnchorKind::TopLevel)?;
    let chrome =
        popup_chrome_with_context(menu, commands, surface.command_context(), &theme, measurer);
    let popup_rect = popup_rect(
        point::logical(anchor.origin.x(), anchor.origin.y() + anchor.area.height()),
        chrome,
        &theme,
        layout.rect(),
    );

    Some(Popup::new(
        popup_rect,
        popup_node(
            MENU_POPUP,
            menu,
            commands,
            surface.command_context(),
            chrome,
            &theme,
        ),
    ))
}

pub fn submenu_popup(
    tree: &ui::Tree,
    layout: &ui::Frame,
    surface: &ui::floating::Surface,
    menu: &menu::Menu,
    commands: &command::Registry,
    measurer: &mut text::layout::Engine,
) -> Option<Popup> {
    let theme = theme::Theme::default_dark();
    let anchor = anchor_rect(tree, layout, menu.id(), AnchorKind::Submenu)?;
    let padding = theme.floating_panel().padding();
    let chrome =
        popup_chrome_with_context(menu, commands, surface.command_context(), &theme, measurer);
    let popup_rect = popup_rect(
        point::logical(
            anchor.origin.x() + anchor.area.width() + SUBMENU_GAP,
            anchor.origin.y() - padding,
        ),
        chrome,
        &theme,
        layout.rect(),
    );

    Some(Popup::new(
        popup_rect,
        popup_node(
            MENU_SUBMENU_POPUP,
            menu,
            commands,
            surface.command_context(),
            chrome,
            &theme,
        ),
    ))
}

pub fn text_context_menu_popup(
    surface: &ui::floating::Surface,
    commands: &command::Registry,
    measurer: &mut text::layout::Engine,
    bounds: Rect,
) -> Option<Popup> {
    surface.context_menu_target()?;

    let theme = theme::Theme::default_dark();
    let menu = text_context_menu();
    let chrome =
        popup_chrome_with_context(&menu, commands, surface.command_context(), &theme, measurer);
    let origin = match surface.anchor() {
        ui::floating::Anchor::Point(point) => point,
        ui::floating::Anchor::Rect(rect) => rect.origin,
    };
    let popup_rect = popup_rect(origin, chrome, &theme, bounds);

    Some(Popup::new(
        popup_rect,
        popup_node(
            TEXT_CONTEXT_MENU_POPUP,
            &menu,
            commands,
            surface.command_context(),
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

fn text_context_menu() -> menu::Menu {
    menu::Menu::new("Text").key(TEXT_CONTEXT_MENU).section(
        menu::Section::new()
            .text::<crate::text::command::Undo>()
            .text::<crate::text::command::Redo>()
            .separator()
            .text::<crate::text::command::Cut>()
            .text::<crate::text::command::Copy>()
            .text::<crate::text::command::Paste>()
            .separator()
            .text::<crate::text::command::SelectAll>(),
    )
}

fn popup_node(
    id: ui::Id,
    menu: &menu::Menu,
    commands: &command::Registry,
    command_subject: &command::call::Context,
    chrome: PopupChrome,
    theme: &theme::Theme,
) -> ui::Node {
    let mut popup = floating_panel_with_theme(theme)
        .key(id)
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
                    item_node(id, item, commands, command_subject, chrome, theme)
                }
                menu::Node::Submenu(menu) => {
                    let id = row_id(row);
                    row += 1;
                    submenu_node(id, menu, commands, command_subject, chrome, theme)
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
    commands: &command::Registry,
    command_subject: &command::call::Context,
    chrome: PopupChrome,
    theme: &theme::Theme,
) -> ui::Node {
    let command_id = item.command();
    let state = commands.state_key(command_id, command_subject.clone());
    let color = theme.text().primary();
    let check = state
        .is_active()
        .then(|| icon::Icon::phosphor(icon::Id::new("check")));
    let shortcut = shortcut_label(command_id, commands);

    menu_row(id, chrome, theme, color)
        .with_command_route(item.route())
        .with_command_subject(ui::CommandSubject::Captured)
        .with_child(glyph_cell(
            ROW_GLYPH,
            check,
            chrome.glyph_width,
            color,
            theme,
        ))
        .with_child(text_cell(
            ROW_LABEL,
            item_label_for_context(item, commands, Some(command_subject)),
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
    commands: &command::Registry,
    command_subject: &command::call::Context,
    chrome: PopupChrome,
    theme: &theme::Theme,
) -> ui::Node {
    let enabled = menu_can_open(menu, commands, command_subject);
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

fn item_label_for_context(
    item: &menu::Item,
    commands: &command::Registry,
    command_subject: Option<&command::call::Context>,
) -> String {
    item.label()
        .map(str::to_owned)
        .or_else(|| {
            command_subject
                .and_then(|context| commands.presentation_key(item.command(), context.clone()))
                .map(|presentation| presentation.display().to_owned())
        })
        .or_else(|| {
            commands
                .command_key(item.command())
                .map(|command| command.display().to_owned())
        })
        .unwrap_or_else(|| item.command().as_str().replace('_', " "))
}

fn shortcut_label(command: command::Key, commands: &command::Registry) -> Option<String> {
    commands
        .command_key(command)
        .and_then(|command| command.shortcuts().first())
        .copied()
        .map(command::shortcut::Shortcut::display_label)
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
            .with_color(color),
    ));
    text::document::Document::from_block(block)
}

#[cfg(test)]
fn popup_chrome(
    menu: &menu::Menu,
    commands: &command::Registry,
    theme: &theme::Theme,
    measurer: &mut text::layout::Engine,
) -> PopupChrome {
    popup_chrome_for_context(menu, commands, None, theme, measurer)
}

fn popup_chrome_with_context(
    menu: &menu::Menu,
    commands: &command::Registry,
    command_subject: &command::call::Context,
    theme: &theme::Theme,
    measurer: &mut text::layout::Engine,
) -> PopupChrome {
    popup_chrome_for_context(menu, commands, Some(command_subject), theme, measurer)
}

fn popup_chrome_for_context(
    menu: &menu::Menu,
    commands: &command::Registry,
    command_subject: Option<&command::call::Context>,
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
                    item_label_for_context(item, commands, command_subject),
                    theme,
                    measurer,
                ));
                if let Some(shortcut) = shortcut_label(item.command(), commands) {
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

fn menu_can_open(
    menu: &menu::Menu,
    commands: &command::Registry,
    command_subject: &command::call::Context,
) -> bool {
    menu.commands()
        .any(|route| commands.can_invoke_key(route.command(), command_subject.clone()))
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
    use crate::Command;

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
        let context = command::call::Context::window(crate::window::Id::new(1));

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
        let context = command::call::Context::window(crate::window::Id::new(1));

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("Select All"),
        );
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
            &context,
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

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("Select All")
                .shortcut(command::shortcut::Shortcut::ctrl('a')),
        );
        registry.set_state_key(COMMAND_A, context.clone(), command::State::unavailable());

        let row = item_node(
            ui::Id::new("row"),
            &item,
            &registry,
            &context,
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
        tree.paint(
            &layout,
            &registry,
            window,
            ui::Interaction::default(),
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
                theme.text().disabled()
            );
        }
    }

    #[test]
    fn enabled_menu_item_paints_label_and_shortcut_with_primary_color() {
        let theme = theme::Theme::default_dark();
        let mut registry = command::Registry::new();
        let item = menu::Item::key(COMMAND_A);
        let window = crate::window::Id::new(1);
        let context = command::call::Context::window(window);

        registry.register(
            command::definition::Definition::for_command::<CommandA, command::TestTarget>()
                .with_display("Select All")
                .shortcut(command::shortcut::Shortcut::ctrl('a')),
        );

        let row = item_node(
            ui::Id::new("row"),
            &item,
            &registry,
            &context,
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
        tree.paint(
            &layout,
            &registry,
            window,
            ui::Interaction::default(),
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
                theme.text().primary()
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
            &context,
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
            &registry,
            window,
            ui::Interaction::new(Some(row_path.clone()), None, Some(row_path)),
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
        let window = crate::window::Id::new(1);
        let context = command::call::Context::window(window);
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
            &context,
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
            &registry,
            window,
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
        let context = command::call::Context::window(crate::window::Id::new(1));
        let submenu = menu::Menu::new("Disabled")
            .key(menu::Id::new("disabled"))
            .section(menu::Section::new().item(menu::Item::key(COMMAND_A)));

        let row = submenu_node(
            ui::Id::new("row"),
            &submenu,
            &registry,
            &context,
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
