use super::super::{geometry::Size, keymap, theme, view};
use super::{control, engine, flow, flow::gap_total, typography};

pub(in crate::layout) fn size_hint(
    node: &view::Node,
    constraints: flow::Constraints,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> flow::SizeHint {
    let max = constraints.max();
    let width = resolved_width(node, max.width(), engine, theme, profile);
    let height = resolved_height_for_width(node, width, max.height(), engine, theme, profile);

    flow::SizeHint::fixed(Size::new(width, height)).constrained(constraints)
}

pub(in crate::layout) fn intrinsic_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    let label_width = node
        .label_text()
        .map(|label| engine.label_width_with_style(label, typography::label_style(node, theme)))
        .unwrap_or_default();

    match node.role() {
        view::Role::MenuBar => menu_bar_intrinsic_width(node, engine, theme),
        view::Role::Menu => menu_title_width(node, engine, theme),
        view::Role::FloatingPanel => floating_panel_width(node, engine, theme, profile),
        view::Role::Scroll => scroll_intrinsic_width(node, engine, theme, profile),
        view::Role::Binding if control::is_menu_panel_row(node) => {
            menu_row_width(node, engine, theme, profile).max(theme.menu().panel_min_width)
        }
        view::Role::Binding => label_width.max(theme.menu().panel_min_width),
        view::Role::Label if control::is_palette_row(node) => {
            let shortcut_width = palette_shortcut_width(node, engine, theme, profile);
            control::palette_row_width(label_width, shortcut_width, theme)
        }
        view::Role::Button => button_intrinsic_width(node, engine, theme),
        view::Role::Slider => control::slider_row_width(label_width, theme),
        view::Role::TextBox => label_width.max(120),
        view::Role::Checkbox | view::Role::Radio => control::choice_row_width(label_width, theme),
        view::Role::SectionHeader => section_header_width(node, engine, theme),
        _ => control_intrinsic_width(node, label_width, theme),
    }
}

pub(in crate::layout) fn intrinsic_height(node: &view::Node, theme: &theme::Theme) -> i32 {
    let control_height = theme.control().height;
    match node.role() {
        view::Role::FloatingPanel => floating_panel_height(node, theme),
        view::Role::MenuBar | view::Role::Menu => theme.menu().bar_height,
        view::Role::Binding if control::is_menu_panel_row(node) => theme.menu().row_height,
        view::Role::Binding
        | view::Role::Button
        | view::Role::Checkbox
        | view::Role::Radio
        | view::Role::Slider
        | view::Role::TextBox => control_height,
        view::Role::Scroll => scroll_intrinsic_height(node, theme),
        view::Role::VirtualList => control_height,
        view::Role::Separator => theme.menu().row_height,
        view::Role::SectionHeader => section_header_height(theme),
        view::Role::Label => body_line_height(theme),
        view::Role::TextArea
        | view::Role::Panel
        | view::Role::Root
        | view::Role::Stack
        | view::Role::Table => control_height,
    }
}

pub(in crate::layout) fn intrinsic_height_for_width(
    node: &view::Node,
    width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    let control_height = theme.control().height;
    let content_width =
        if node.participation() == Some(view::Participation::Table(view::TablePart::Cell)) {
            control::table_content_width(width, theme)
        } else {
            width
        };
    let height = match node.role() {
        view::Role::FloatingPanel => {
            floating_panel_height_for_width(node, content_width, engine, theme, profile)
        }
        view::Role::Label | view::Role::Panel
            if node.label_text().is_some() && node.children().is_empty() =>
        {
            if node.world_text_wrap() == Some(view::Wrap::None) {
                line_height(typography::label_style(node, theme))
            } else {
                node.label_text()
                    .map(|label| {
                        engine
                            .label_size_for_width_with_style(
                                label,
                                content_width,
                                typography::label_style(node, theme),
                            )
                            .height()
                    })
                    .unwrap_or_else(|| line_height(typography::label_style(node, theme)))
            }
        }
        view::Role::SectionHeader => node
            .label_text()
            .map(|label| {
                engine
                    .label_size_for_width_with_style(
                        &typography::section_header_text(label),
                        width,
                        typography::section_header_style(theme),
                    )
                    .height()
            })
            .unwrap_or_else(|| section_header_height(theme))
            .max(section_header_height(theme)),
        view::Role::TextBox
            if node
                .text_box_model()
                .is_some_and(view::TextBox::projects_inactive_display) =>
        {
            let measured = node
                .text_box_model()
                .map(|text_box| {
                    let text_area = view::TextArea::new(text_box.display_text().to_owned())
                        .with_wrap(node.world_text_wrap().unwrap_or(view::Wrap::None))
                        .read_only();
                    engine
                        .text_area_size_for_width(&text_area, content_width, theme)
                        .height()
                })
                .unwrap_or(control_height);
            if node.world_text_wrap().is_some() {
                measured
            } else {
                measured.max(control_height)
            }
        }
        view::Role::TextArea => {
            let measured = node
                .text_area_model()
                .map(|text_area| {
                    engine
                        .text_area_size_for_width(text_area, content_width, theme)
                        .height()
                })
                .unwrap_or(control_height);
            if node.world_text_wrap().is_some() {
                measured
            } else {
                measured.max(control_height)
            }
        }
        view::Role::Scroll => {
            scroll_intrinsic_height_for_width(node, content_width, engine, theme, profile)
        }
        view::Role::Panel | view::Role::Root | view::Role::Stack | view::Role::Table => {
            stack_intrinsic_height_for_width(node, content_width, engine, theme, profile)
        }
        _ => intrinsic_height(node, theme),
    };
    if matches!(
        node.participation(),
        Some(view::Participation::Table(part)) if part != view::TablePart::HeaderBand
    ) {
        height.saturating_add(theme.table().cell_padding.max(0).saturating_mul(2))
    } else {
        height
    }
}

pub(in crate::layout) fn floating_panel_height(node: &view::Node, theme: &theme::Theme) -> i32 {
    let content_height = if control::is_menu_panel(node) {
        node.children()
            .iter()
            .map(|child| intrinsic_height(child, theme))
            .sum::<i32>()
            .max(theme.menu().row_height)
    } else {
        stack_intrinsic_height(node, theme)
    };

    content_height.saturating_add(theme.floating_panel().padding.max(0).saturating_mul(2))
}

pub(in crate::layout) fn floating_panel_height_for_width(
    node: &view::Node,
    width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    let padding = theme.floating_panel().padding.max(0);
    let content_width = width.max(0).saturating_sub(padding.saturating_mul(2));
    let content_height = if let Some(hint) = node.auxiliary_hint() {
        let reserved = i32::from(hint.icon().is_some()).saturating_mul(
            theme
                .auxiliary_panel()
                .icon_extent
                .saturating_add(theme.auxiliary_panel().icon_gap),
        );
        stack_intrinsic_height_for_width(
            node,
            content_width.saturating_sub(reserved),
            engine,
            theme,
            profile,
        )
        .max(if hint.icon().is_some() {
            theme.auxiliary_panel().icon_extent
        } else {
            0
        })
    } else if control::is_menu_panel(node) {
        node.children()
            .iter()
            .map(|child| intrinsic_height_for_width(child, content_width, engine, theme, profile))
            .sum::<i32>()
            .max(theme.menu().row_height)
    } else {
        stack_intrinsic_height_for_width(node, content_width, engine, theme, profile)
    };

    content_height.saturating_add(padding.saturating_mul(2))
}

pub(in crate::layout) fn floating_panel_max_envelope_height_for_width(
    node: &view::Node,
    width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    let padding = theme.floating_panel().padding.max(0);
    let content_width = width.max(0).saturating_sub(padding.saturating_mul(2));
    let content_height = if control::is_menu_panel(node) {
        node.children()
            .iter()
            .map(|child| intrinsic_height_for_width(child, content_width, engine, theme, profile))
            .sum::<i32>()
            .max(theme.menu().row_height)
    } else {
        stack_max_envelope_height_for_width(node, content_width, engine, theme, profile)
    };

    content_height.saturating_add(padding.saturating_mul(2))
}

pub(in crate::layout) fn floating_panel_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    let content_width = if let Some(hint) = node.auxiliary_hint() {
        let reserved = i32::from(hint.icon().is_some()).saturating_mul(
            theme
                .auxiliary_panel()
                .icon_extent
                .saturating_add(theme.auxiliary_panel().icon_gap),
        );
        stack_intrinsic_width(node, engine, theme, profile).saturating_add(reserved)
    } else if control::is_menu_panel(node) {
        node.children()
            .iter()
            .map(|child| {
                menu_row_width(child, engine, theme, profile)
                    .max(intrinsic_width(child, engine, theme, profile))
            })
            .max()
            .unwrap_or_default()
            .max(theme.menu().panel_min_width)
    } else {
        stack_intrinsic_width(node, engine, theme, profile)
    };

    content_width.saturating_add(theme.floating_panel().padding.max(0).saturating_mul(2))
}

fn stack_max_envelope_height_for_width(
    node: &view::Node,
    width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    let children = node.children();
    if children.is_empty() {
        return stack_intrinsic_height_for_width(node, width, engine, theme, profile);
    }

    let padding = node.style().padding();
    let content_width = width.max(0).saturating_sub(padding.horizontal());
    let child_height = match node.axis() {
        Some(view::Axis::Horizontal) | Some(view::Axis::Overlay) => children
            .iter()
            .map(|child| {
                let child_width = resolved_row_width(child, content_width, engine, theme, profile)
                    .min(content_width);
                max_envelope_height_for_width(child, child_width, engine, theme, profile)
            })
            .max()
            .unwrap_or_default(),
        Some(view::Axis::Vertical) | None => children
            .iter()
            .map(|child| {
                max_envelope_height_for_width(child, content_width, engine, theme, profile)
            })
            .sum::<i32>()
            .saturating_add(gap_total(layout_gap(node, theme), children.len())),
    };

    child_height.saturating_add(padding.vertical())
}

fn max_envelope_height_for_width(
    node: &view::Node,
    width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    if node.role() == view::Role::Scroll
        && node.style().height() == Some(view::Dimension::Fit)
        && let Some(max_height) = node.style().max_height()
    {
        return max_height;
    }

    intrinsic_height_for_width(node, width, engine, theme, profile)
}

fn stack_intrinsic_height(node: &view::Node, theme: &theme::Theme) -> i32 {
    let children = node.children();
    if children.is_empty() {
        return theme.control().height;
    }

    let child_height = match node.axis() {
        Some(view::Axis::Horizontal) | Some(view::Axis::Overlay) => children
            .iter()
            .map(|child| intrinsic_height(child, theme))
            .max()
            .unwrap_or_default(),
        Some(view::Axis::Vertical) | None => children
            .iter()
            .map(|child| intrinsic_or_fixed_height(child, theme))
            .sum::<i32>()
            .saturating_add(gap_total(layout_gap(node, theme), children.len())),
    };

    capped_height(
        node,
        child_height.saturating_add(node.style().padding().vertical()),
    )
}

fn stack_intrinsic_height_for_width(
    node: &view::Node,
    width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    let children = node.children();
    if children.is_empty() {
        return node
            .label_text()
            .map(|label| {
                engine
                    .label_size_for_width_with_style(label, width, theme.typography().body())
                    .height()
            })
            .unwrap_or_else(|| body_line_height(theme));
    }

    let padding = node.style().padding();
    let content_width = width.max(0).saturating_sub(padding.horizontal());
    match node.axis() {
        Some(view::Axis::Horizontal) | Some(view::Axis::Overlay) => children
            .iter()
            .map(|child| {
                let child_width = resolved_row_width(child, content_width, engine, theme, profile)
                    .min(content_width);
                intrinsic_height_for_width(child, child_width, engine, theme, profile)
            })
            .max()
            .unwrap_or_default()
            .saturating_add(padding.vertical()),
        Some(view::Axis::Vertical) | None => {
            let mut column = flow::Column::new()
                .gap(layout_gap(node, theme))
                .padding(padding)
                .align_items(node.style().align_items())
                .justify_content(node.style().justify_content());

            for child in children {
                let child_width = cross_axis_width(
                    child,
                    content_width,
                    engine,
                    node.style().align_items(),
                    theme,
                    profile,
                );
                let child_height =
                    intrinsic_or_fixed_height_for_width(child, child_width, engine, theme, profile);
                column = column.item(flow::Item::fixed(flow::SizeHint::fixed(Size::new(
                    child_width,
                    child_height,
                ))));
            }

            capped_height(node, column.size_hint().preferred().height())
        }
    }
}

fn intrinsic_or_fixed_height(node: &view::Node, theme: &theme::Theme) -> i32 {
    match node.style().height() {
        Some(view::Dimension::Fixed(height)) => capped_height(node, height),
        _ => intrinsic_height(node, theme),
    }
}

pub(in crate::layout) fn intrinsic_or_fixed_height_for_width(
    node: &view::Node,
    width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    match node.style().height() {
        Some(view::Dimension::Fixed(height)) => capped_height(node, height),
        _ => intrinsic_height_for_width(node, width, engine, theme, profile),
    }
}

fn stack_intrinsic_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    let children = node.children();
    if children.is_empty() {
        return 0;
    }

    let child_width = match node.axis() {
        Some(view::Axis::Horizontal) => children
            .iter()
            .map(|child| intrinsic_width(child, engine, theme, profile))
            .sum::<i32>()
            .saturating_add(gap_total(layout_gap(node, theme), children.len())),
        Some(view::Axis::Vertical) | Some(view::Axis::Overlay) | None => children
            .iter()
            .map(|child| intrinsic_width(child, engine, theme, profile))
            .max()
            .unwrap_or_default(),
    };

    child_width.saturating_add(node.style().padding().horizontal())
}

fn scroll_intrinsic_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    match node.axis() {
        Some(view::Axis::Horizontal) => theme.viewport().min_viewport_extent,
        Some(view::Axis::Vertical) | Some(view::Axis::Overlay) | None => {
            stack_intrinsic_width(node, engine, theme, profile)
        }
    }
}

fn scroll_intrinsic_height(node: &view::Node, theme: &theme::Theme) -> i32 {
    match node.axis() {
        Some(view::Axis::Horizontal) => stack_intrinsic_height(node, theme),
        Some(view::Axis::Vertical) | Some(view::Axis::Overlay) | None
            if node.style().height() == Some(view::Dimension::Fit)
                && node.style().max_height().is_some() =>
        {
            capped_height(node, stack_intrinsic_height(node, theme))
        }
        Some(view::Axis::Vertical) | Some(view::Axis::Overlay) | None => {
            capped_height(node, theme.viewport().min_viewport_extent)
        }
    }
}

fn scroll_intrinsic_height_for_width(
    node: &view::Node,
    width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    match node.axis() {
        Some(view::Axis::Horizontal) => {
            stack_intrinsic_height_for_width(node, width, engine, theme, profile)
        }
        Some(view::Axis::Vertical) | Some(view::Axis::Overlay) | None
            if node.style().height() == Some(view::Dimension::Fit)
                && node.style().max_height().is_some() =>
        {
            capped_height(
                node,
                stack_intrinsic_height_for_width(node, width, engine, theme, profile),
            )
        }
        Some(view::Axis::Vertical) | Some(view::Axis::Overlay) | None => {
            capped_height(node, theme.viewport().min_viewport_extent)
        }
    }
}

pub(in crate::layout) fn menu_shortcut_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    keymap: keymap::Profile,
) -> i32 {
    node.children()
        .iter()
        .filter_map(|child| {
            child
                .binding()
                .and_then(view::Binding::shortcut)
                .map(|shortcut| {
                    shortcut_display_width(
                        &shortcut.display_parts(keymap, theme.shortcuts().display()),
                        engine,
                        theme,
                    )
                })
        })
        .max()
        .unwrap_or_default()
}

fn palette_shortcut_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    node.binding()
        .and_then(view::Binding::shortcut)
        .map(|shortcut| {
            shortcut_display_width(
                &shortcut.display_parts(profile, theme.shortcuts().display()),
                engine,
                theme,
            )
        })
        .unwrap_or_default()
}

pub(in crate::layout) fn shortcut_display_width(
    display: &keymap::ShortcutDisplay,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    let run_width = display
        .runs()
        .iter()
        .map(|run| shortcut_run_width(run, engine, theme))
        .sum::<i32>();

    run_width.saturating_add(
        typography::shortcut_run_gap(theme)
            .saturating_mul(display.runs().len().saturating_sub(1) as i32),
    )
}

pub(in crate::layout) fn shortcut_run_width(
    run: &keymap::ShortcutRun,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    match run {
        keymap::ShortcutRun::Text(value) => {
            engine.label_width_with_style(value, typography::shortcut_text_style(theme))
        }
        keymap::ShortcutRun::Icon(_) => shortcut_icon_extent(theme),
    }
}

pub(in crate::layout) fn shortcut_icon_extent(theme: &theme::Theme) -> i32 {
    typography::shortcut_text_style(theme)
        .size()
        .ceil()
        .max(1.0) as i32
}

fn menu_bar_intrinsic_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    node.children()
        .iter()
        .filter(|child| child.role() == view::Role::Menu)
        .map(|child| menu_title_width(child, engine, theme))
        .fold(0, i32::saturating_add)
}

pub(in crate::layout) fn menu_title_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    let label_width = node
        .label_text()
        .map(|label| {
            engine.label_width_with_style(
                label,
                typography::label_style_for(view::Role::Menu, None, theme),
            )
        })
        .unwrap_or_default();
    let content_width = if has_single_character_label(node) {
        let padding = theme.menu().padding.max(0);
        label_width.max(
            theme
                .menu()
                .bar_height
                .max(0)
                .saturating_sub(padding.saturating_mul(2)),
        )
    } else {
        label_width
    };

    content_width
        .max(0)
        .saturating_add(theme.menu().padding.max(0).saturating_mul(2))
}

fn menu_row_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    if node.role() == view::Role::Separator {
        return 0;
    }

    let label_width = node
        .label_text()
        .map(|label| engine.label_width_with_style(label, typography::label_style(node, theme)))
        .unwrap_or_default();
    let shortcut_width = node
        .binding()
        .and_then(view::Binding::shortcut)
        .map(|shortcut| {
            shortcut_display_width(
                &shortcut.display_parts(profile, theme.shortcuts().display()),
                engine,
                theme,
            )
        })
        .unwrap_or_default();

    control::menu_row_width(label_width, shortcut_width, theme)
}

pub(in crate::layout) fn layout_gap(node: &view::Node, theme: &theme::Theme) -> i32 {
    if node.role() == view::Role::FloatingPanel && !control::is_menu_panel(node) {
        return node
            .style()
            .gap_override()
            .unwrap_or(theme.floating_panel().content_gap)
            .max(0);
    }

    node.style().gap()
}

pub(in crate::layout) fn grows_vertical_space(node: &view::Node) -> bool {
    match node.style().height() {
        Some(view::Dimension::Flexible { .. }) => true,
        Some(view::Dimension::Fit | view::Dimension::Fixed(_) | view::Dimension::Percent(_)) => {
            false
        }
        None => matches!(
            node.role(),
            view::Role::TextArea
                | view::Role::Panel
                | view::Role::Scroll
                | view::Role::Stack
                | view::Role::Table
        ),
    }
}

fn capped_height(node: &view::Node, height: i32) -> i32 {
    node.style()
        .max_height()
        .map_or(height, |max_height| height.min(max_height))
}

pub(in crate::layout) fn resolved_width(
    node: &view::Node,
    parent_width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    match node.style().width() {
        Some(view::Dimension::Fit) => intrinsic_width(node, engine, theme, profile),
        Some(view::Dimension::Flexible { .. }) | None => parent_width,
        Some(view::Dimension::Fixed(width)) => width,
        Some(view::Dimension::Percent(percent)) => {
            ((parent_width.max(0) as f32) * percent).round() as i32
        }
    }
    .clamp(0, parent_width.max(0))
}

pub(in crate::layout) fn resolved_row_width(
    node: &view::Node,
    parent_width: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    match node.style().width() {
        None | Some(view::Dimension::Fit) => intrinsic_width(node, engine, theme, profile),
        Some(view::Dimension::Flexible { .. }) => parent_width,
        Some(view::Dimension::Fixed(width)) => width,
        Some(view::Dimension::Percent(percent)) => {
            ((parent_width.max(0) as f32) * percent).round() as i32
        }
    }
    .clamp(0, parent_width.max(0))
}

pub(in crate::layout) fn cross_axis_width(
    node: &view::Node,
    parent_width: i32,
    engine: &mut engine::Engine,
    align: view::Align,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    match align {
        view::Align::Stretch => resolved_width(node, parent_width, engine, theme, profile),
        view::Align::Start | view::Align::Center | view::Align::End => match node.style().width() {
            None | Some(view::Dimension::Fit) => intrinsic_width(node, engine, theme, profile),
            Some(view::Dimension::Flexible { .. }) => parent_width,
            Some(view::Dimension::Fixed(width)) => width,
            Some(view::Dimension::Percent(percent)) => {
                ((parent_width.max(0) as f32) * percent).round() as i32
            }
        }
        .clamp(0, parent_width.max(0)),
    }
}

pub(in crate::layout) fn cross_axis_height_for_width(
    node: &view::Node,
    width: i32,
    parent_height: i32,
    engine: &mut engine::Engine,
    align: view::Align,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    let height = match align {
        view::Align::Stretch => match node.style().height() {
            None | Some(view::Dimension::Flexible { .. }) => parent_height,
            Some(view::Dimension::Fit) => {
                intrinsic_height_for_width(node, width, engine, theme, profile)
            }
            Some(view::Dimension::Fixed(height)) => height,
            Some(view::Dimension::Percent(percent)) => {
                ((parent_height.max(0) as f32) * percent).round() as i32
            }
        },
        view::Align::Start | view::Align::Center | view::Align::End => {
            match node.style().height() {
                None | Some(view::Dimension::Fit) => {
                    intrinsic_height_for_width(node, width, engine, theme, profile)
                }
                Some(view::Dimension::Flexible { .. }) => parent_height,
                Some(view::Dimension::Fixed(height)) => height,
                Some(view::Dimension::Percent(percent)) => {
                    ((parent_height.max(0) as f32) * percent).round() as i32
                }
            }
        }
    };

    capped_height(node, height).clamp(0, parent_height.max(0))
}

pub(in crate::layout) fn main_axis_offset(align: view::Align, available: i32, content: i32) -> i32 {
    let slack = available.saturating_sub(content);
    match align {
        view::Align::Start | view::Align::Stretch => 0,
        view::Align::Center => slack / 2,
        view::Align::End => slack,
    }
}

pub(in crate::layout) fn cross_axis_offset(
    origin: i32,
    available: i32,
    size: i32,
    align: view::Align,
) -> i32 {
    origin.saturating_add(main_axis_offset(align, available, size))
}

pub(in crate::layout) fn resolved_height(
    node: &view::Node,
    parent_height: i32,
    theme: &theme::Theme,
) -> i32 {
    let height = match node.style().height() {
        Some(view::Dimension::Fit) => intrinsic_height(node, theme),
        Some(view::Dimension::Flexible { .. }) | None => {
            if grows_vertical_space(node) {
                parent_height
            } else {
                intrinsic_height(node, theme)
            }
        }
        Some(view::Dimension::Fixed(height)) => height,
        Some(view::Dimension::Percent(percent)) => {
            ((parent_height.max(0) as f32) * percent).round() as i32
        }
    };

    capped_height(node, height).clamp(0, parent_height.max(0))
}

pub(in crate::layout) fn resolved_height_for_width(
    node: &view::Node,
    width: i32,
    parent_height: i32,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
    profile: keymap::Profile,
) -> i32 {
    let height = match node.style().height() {
        Some(view::Dimension::Fit) => {
            intrinsic_height_for_width(node, width, engine, theme, profile)
        }
        Some(view::Dimension::Flexible { .. }) | None => {
            if grows_vertical_space(node) {
                parent_height
            } else {
                intrinsic_height_for_width(node, width, engine, theme, profile)
            }
        }
        Some(view::Dimension::Fixed(height)) => height,
        Some(view::Dimension::Percent(percent)) => {
            ((parent_height.max(0) as f32) * percent).round() as i32
        }
    };

    capped_height(node, height).clamp(0, parent_height.max(0))
}

fn control_intrinsic_width(node: &view::Node, label_width: i32, theme: &theme::Theme) -> i32 {
    if has_single_character_label(node) {
        let content_width = label_width.max(control::control_content_extent(
            theme.control().height,
            theme,
        ));

        control::padded_control_extent(content_width, theme)
    } else {
        control::padded_control_extent(label_width, theme)
    }
}

fn button_intrinsic_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    let Some(button) = node.button_model() else {
        return control_intrinsic_width(
            node,
            node.label_text()
                .map(|label| {
                    engine.label_width_with_style(label, typography::label_style(node, theme))
                })
                .unwrap_or_default(),
            theme,
        );
    };
    let mut max_width: i32 = 0;
    let mut all_single_character = true;
    let mut count = 0;

    for label in button.measurement_labels() {
        count += 1;
        max_width = max_width.max(engine.label_width_with_style(
            label,
            typography::label_style_for(view::Role::Button, None, theme),
        ));
        all_single_character &= label.chars().count() == 1;
    }

    if count > 0 && all_single_character {
        max_width = max_width.max(control::control_content_extent(
            theme.control().height,
            theme,
        ));
    }

    control::padded_control_extent(max_width, theme)
}

fn section_header_width(
    node: &view::Node,
    engine: &mut engine::Engine,
    theme: &theme::Theme,
) -> i32 {
    node.label_text()
        .map(|label| {
            engine.label_width_with_style(
                &typography::section_header_text(label),
                typography::section_header_style(theme),
            )
        })
        .unwrap_or_default()
}

pub(crate) fn section_header_height(theme: &theme::Theme) -> i32 {
    (typography::section_header_style(theme).size() * 1.25).ceil() as i32
        + theme.control().padding.max(0)
}

fn body_line_height(theme: &theme::Theme) -> i32 {
    line_height(theme.typography().body())
}

fn line_height(style: theme::TypeStyle) -> i32 {
    style.size().ceil().max(1.0) as i32
}

fn has_single_character_label(node: &view::Node) -> bool {
    node.label_text()
        .is_some_and(|label| label.chars().count() == 1)
}
