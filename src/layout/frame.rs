use super::super::{
    composition, context,
    geometry::{Point, Rect},
    interaction, keymap, scene,
    theme::Theme,
    view,
};
use super::{Viewport, control, engine, measure, path, table, text, typography};
use crate::{animation, text as text_model};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Clip {
    rect: Rect,
    rounding: scene::Rounding,
}

#[derive(Clone)]
pub(crate) struct ShortcutPart {
    run: keymap::ShortcutRun,
    width: i32,
}

impl ShortcutPart {
    pub(crate) fn run(&self) -> &keymap::ShortcutRun {
        &self.run
    }

    pub(crate) fn width(&self) -> i32 {
        self.width
    }
}

pub(super) struct Input<'a> {
    pub(super) node: &'a view::Node,
    pub(super) node_id: composition::NodeId,
    pub(super) path: path::Path,
    pub(super) rect: Rect,
    pub(super) floating_layer: bool,
    pub(super) clip: Option<Clip>,
    pub(super) animation_frame: animation::Frame,
    pub(super) keymap: keymap::Profile,
}

#[derive(Clone)]
enum FrameContent {
    Structural(StructuralRole),
    Menu,
    Binding,
    Separator(SeparatorContent),
    Text(TextContent),
    Button(WorldTextContent),
    Choice(ChoiceContent),
    Slider(SliderContent),
    Scroll(ScrollContent),
    VirtualList(VirtualListContent),
    FloatingPanel(FloatingPanelContent),
}

#[derive(Clone, Copy)]
enum StructuralRole {
    Root,
    Stack,
    MenuBar,
    Panel,
    Table,
}

#[derive(Clone)]
enum ChoiceContent {
    Checkbox(view::Checkbox),
    Radio(view::Radio),
}

#[derive(Clone)]
enum TextContent {
    Label {
        world_overflow: Option<text_model::Overflow>,
        world_wrap: Option<view::Wrap>,
        world_align: Option<view::Align>,
    },
    SectionHeader,
    Area {
        model: view::TextArea,
        display_model: view::TextArea,
        layout: text::Area,
        text_rect: Rect,
        world_overflow: Option<text_model::Overflow>,
        world_wrap: Option<view::Wrap>,
        world_align: Option<view::Align>,
    },
    InactiveField {
        input: TextBoxContent,
        model: view::TextArea,
        display_model: view::TextArea,
        layout: text::Area,
        text_rect: Rect,
        world_overflow: Option<text_model::Overflow>,
        world_wrap: Option<view::Wrap>,
        world_align: Option<view::Align>,
    },
    Field {
        input: TextBoxContent,
        layout: text::Field,
        text_rect: Rect,
        display_text: Option<String>,
    },
}

#[derive(Clone)]
struct TextBoxContent {
    model: view::TextBox,
    parts: control::InputParts,
}

#[derive(Clone, Copy)]
struct WorldTextContent {
    overflow: Option<text_model::Overflow>,
    wrap: Option<view::Wrap>,
    align: Option<view::Align>,
}

#[derive(Clone)]
struct SliderContent {
    model: view::Slider,
    track_rect: Rect,
}

#[derive(Clone)]
enum ScrollContent {
    Ordinary { viewport: Option<Viewport> },
    Table { resolved: Option<TableScroll> },
}

#[derive(Clone)]
struct TableScroll {
    viewport: Viewport,
    projection: table::Projection,
}

#[derive(Clone)]
struct VirtualListContent {
    geometry: Option<VirtualGeometry>,
    model: crate::virtual_list::Model,
}

#[derive(Clone)]
struct VirtualGeometry {
    viewport: Viewport,
    request: crate::virtual_list::Request,
}

#[derive(Clone)]
struct FloatingPanelContent {
    force_overlay_group: bool,
    native_popup_material_preference: view::NativePopupMaterialPreference,
    popup_placement: Option<crate::geometry::PlacementRequest>,
    popup_context: Option<crate::popup::ContextFingerprint>,
    policy: view::PanelPolicy,
}

#[derive(Clone)]
struct BoundContent {
    binding: view::Binding,
    shortcut_width: Option<i32>,
    shortcut_content_width: i32,
    shortcut_display: Option<Vec<ShortcutPart>>,
}

#[derive(Clone)]
struct LabelContent {
    text: String,
    width: i32,
    overflow_projection: Option<text::Selectable>,
}

#[derive(Clone, Copy)]
struct SeparatorContent {
    shortcut_width: Option<i32>,
}

#[derive(Clone)]
pub(crate) struct Frame {
    node_id: composition::NodeId,
    interaction_id: Option<interaction::Id>,
    path: path::Path,
    content: FrameContent,
    rect: Rect,
    active_rect: Rect,
    label: Option<LabelContent>,
    focused: bool,
    focus_visible: bool,
    selected: bool,
    active_item: bool,
    provided_row: Option<view::ProvidedRow>,
    table_row: Option<crate::table::Row>,
    table_cell: Option<crate::table::Cell>,
    table_header_cell: Option<crate::table::HeaderCell>,
    table_header_presentation: Option<crate::table::HeaderPresentation>,
    participation: Option<view::Participation>,
    floating_layer: bool,
    background: Option<scene::Brush>,
    clip: Option<Clip>,
    target: Option<interaction::Target>,
    binding: Option<BoundContent>,
    action: Option<view::Action>,
}

impl Frame {
    pub(super) fn new(input: Input<'_>, engine: &mut engine::Engine, theme: &Theme) -> Self {
        let Input {
            node,
            node_id,
            path,
            rect,
            floating_layer,
            clip,
            animation_frame,
            keymap,
        } = input;
        let target = target_for(node, node_id);
        let binding = node.binding().cloned();
        let text_box = node.text_box_model().cloned();
        let input_parts = text_box.as_ref().map(|text_box| {
            control::input_parts(
                rect,
                node.participation() == Some(view::Participation::Table(view::TablePart::Cell)),
                text_box.indicator_hint().is_some(),
                theme,
            )
        });
        let inactive_text_box = text_box
            .as_ref()
            .is_some_and(view::TextBox::projects_inactive_display);
        let text_area = node.text_area_model().cloned().or_else(|| {
            text_box
                .as_ref()
                .filter(|_| inactive_text_box)
                .map(|text_box| {
                    let mut text_area = view::TextArea::new(text_box.display_text().to_owned())
                        .with_wrap(node.world_text_wrap().unwrap_or(view::Wrap::None))
                        .read_only();
                    if let Some(focus) = text_box.focus() {
                        text_area = text_area.with_focus(focus);
                    }
                    text_area
                })
        });
        let now = animation_frame.now();
        let inactive_input_text_rect = inactive_text_box
            .then(|| input_parts.map(control::InputParts::text))
            .flatten();
        let text_area_text_rect = table_cell_text_rect_for(
            node,
            rect,
            inactive_input_text_rect,
            text_area.as_ref(),
            engine,
            theme,
        );
        let text_box_text_rect = input_parts.map_or(rect, control::InputParts::text);
        let text_box_layout = (!inactive_text_box)
            .then_some(text_box.as_ref())
            .flatten()
            .map(|text_box| engine.text_field_layout(text_box, text_box_text_rect, theme, now));
        let label_style = typography::label_style(node, theme);
        let world_text_overflow = node.world_text_overflow();
        let world_text_wrap = node.world_text_wrap();
        let world_text_align = node.world_text_align();
        let world_text_rect = match node.participation() {
            Some(view::Participation::Table(view::TablePart::Cell)) => {
                control::table_content_rect(rect, theme)
            }
            Some(view::Participation::Table(
                view::TablePart::Header | view::TablePart::HeaderControl,
            )) => control::table_header_label_rect(
                rect,
                node.table_header_presentation()
                    .is_some_and(|presentation| presentation.sort_direction().is_some()),
                theme,
            ),
            _ => rect,
        };
        let text_area_projection = text_area.as_ref().and_then(|text_area| {
            world_text_overflow.map(|overflow| {
                engine.resolve_selectable_text(
                    &text_area.buffer().text(),
                    text_area_text_rect.width(),
                    label_style,
                    world_text_wrap.unwrap_or_else(|| text_area.wrap()),
                    overflow,
                )
            })
        });
        let label_projection = text_area
            .is_none()
            .then(|| {
                label_for(node).and_then(|label| {
                    world_text_overflow.map(|overflow| {
                        engine.resolve_selectable_text(
                            label,
                            world_text_rect.width(),
                            label_style,
                            world_text_wrap.unwrap_or(view::Wrap::None),
                            overflow,
                        )
                    })
                })
            })
            .flatten();
        let text_area_display = text_area.as_ref().map(|text_area| {
            let Some(projection) = text_area_projection.as_ref() else {
                return text_area.clone();
            };
            let (buffer, state) =
                projection.project_buffer_state(text_area.buffer(), text_area.state());
            text_area.clone().with_resolved_presentation(buffer, state)
        });
        let text_area_color =
            if node.participation() == Some(view::Participation::Table(view::TablePart::Cell)) {
                theme.text().primary
            } else {
                theme.text_input().foreground
            };
        let text_area_layout = text_area_display.as_ref().map(|text_area| {
            engine.text_area_layout(text_area, text_area_text_rect, theme, text_area_color, now)
        });
        let overflow_projection = text_area_projection.or(label_projection);
        let label = if let Some(projection) = overflow_projection.as_ref() {
            Some(projection.visible().to_owned())
        } else {
            label_for(node).map(|label| match world_text_overflow {
                Some(overflow) => engine.resolve_label_overflow(
                    label,
                    world_text_rect.width(),
                    label_style,
                    overflow,
                ),
                None => label.to_owned(),
            })
        };
        let label_width = label
            .as_deref()
            .map(|label| {
                if node.role() == view::Role::SectionHeader {
                    engine.label_width_with_style(
                        &typography::section_header_text(label),
                        label_style,
                    )
                } else {
                    engine.label_width_with_style(label, label_style)
                }
            })
            .unwrap_or_default();
        if world_text_overflow.is_none() && world_text_wrap.is_none() {
            if let Some(label) = label.as_deref() {
                let diagnostic_label = if node.role() == view::Role::SectionHeader {
                    typography::section_header_text(label)
                } else {
                    label.to_owned()
                };
                engine.diagnose_author_text_overflow(
                    &diagnostic_label,
                    rect.width(),
                    rect.height(),
                    label_style,
                );
            }
        }
        let shortcut_display = binding
            .as_ref()
            .and_then(view::Binding::shortcut)
            .map(|shortcut| shortcut.display_parts(keymap, theme.shortcuts().display()));
        let (shortcut_display, shortcut_content_width) = shortcut_display
            .map(|display| {
                let mut width = 0_i32;
                let mut parts = Vec::with_capacity(display.runs().len());
                for (index, run) in display.runs().iter().cloned().enumerate() {
                    if index > 0 {
                        width = width.saturating_add(typography::shortcut_run_gap(theme));
                    }
                    let run_width = measure::shortcut_run_width(&run, engine, theme);
                    width = width.saturating_add(run_width);
                    parts.push(ShortcutPart {
                        run,
                        width: run_width,
                    });
                }

                (Some(parts), width)
            })
            .unwrap_or((None, 0));
        let shortcut_width = shortcut_display.as_ref().map(|_| shortcut_content_width);
        let slider = node.slider_model().cloned();
        let slider_track_rect = slider
            .as_ref()
            .map(|_| control::slider_track_rect(rect, label_width, theme));
        let active_rect = active_rect_for(node, rect, slider.as_ref(), label_width, theme);
        let text_box = text_box
            .zip(input_parts)
            .map(|(model, parts)| TextBoxContent { model, parts });
        let content = FrameContent::for_node(
            node,
            text_area,
            text_area_display,
            text_area_layout,
            text_area_text_rect,
            text_box,
            text_box_layout,
            text_box_text_rect,
            world_text_overflow,
            world_text_wrap,
            world_text_align,
            slider_track_rect,
        );
        let binding = binding.map(|binding| BoundContent {
            binding,
            shortcut_width,
            shortcut_content_width,
            shortcut_display,
        });
        let label = label.map(|text| LabelContent {
            text,
            width: label_width,
            overflow_projection,
        });
        Self {
            path,
            node_id,
            interaction_id: node.id(),
            content,
            rect,
            active_rect,
            label,
            focused: node.is_focused(),
            focus_visible: node.focus_visible(),
            selected: node.is_selected(),
            active_item: node.is_active_item(),
            provided_row: node.provided_row(),
            table_row: node.table_row(),
            table_cell: node.table_cell(),
            table_header_cell: node.table_header_cell(),
            table_header_presentation: node.table_header_presentation(),
            participation: node.participation(),
            floating_layer,
            background: node.style().background(),
            clip,
            target,
            binding,
            action: action_for(node),
        }
    }

    pub(super) fn with_viewport(mut self, viewport: Viewport) -> Self {
        match &mut self.content {
            FrameContent::Scroll(ScrollContent::Ordinary { viewport: current }) => {
                *current = Some(viewport);
            }
            _ => panic!("only ordinary Scroll frame content accepts a viewport"),
        }
        self
    }

    pub(super) fn set_popup_placement(
        &mut self,
        placement: Option<crate::geometry::PlacementRequest>,
    ) {
        if let FrameContent::FloatingPanel(content) = &mut self.content {
            content.popup_placement = placement;
        } else {
            debug_assert!(
                false,
                "only FloatingPanel frame content accepts popup placement"
            );
        }
    }

    pub(super) fn with_table_scroll(
        mut self,
        viewport: Viewport,
        projection: table::Projection,
    ) -> Self {
        assert!(matches!(
            self.content,
            FrameContent::Scroll(ScrollContent::Table { .. })
        ));
        if let FrameContent::Scroll(ScrollContent::Table { resolved }) = &mut self.content {
            *resolved = Some(TableScroll {
                viewport,
                projection,
            });
        }
        self
    }

    pub(super) fn with_virtual_list(
        mut self,
        viewport: Viewport,
        request: crate::virtual_list::Request,
    ) -> Self {
        match &mut self.content {
            FrameContent::VirtualList(content) => {
                content.geometry = Some(VirtualGeometry { viewport, request });
            }
            _ => panic!("only VirtualList frame content accepts virtual geometry"),
        }
        self
    }

    pub(super) fn with_shortcut_width(mut self, width: i32) -> Self {
        let width = width.max(0);
        match (&mut self.content, self.binding.as_mut()) {
            (FrameContent::Separator(content), _) => content.shortcut_width = Some(width),
            (_, Some(content)) if content.binding.source() == context::Source::Menu => {
                content.shortcut_width = Some(width);
            }
            _ => panic!("only menu-row frame content accepts a shortcut width"),
        }
        self
    }

    pub(crate) fn is_descendant_of(&self, ancestor: &Self) -> bool {
        self.path.is_descendant_of(&ancestor.path)
    }

    pub(crate) fn path_depth(&self) -> usize {
        self.path.len()
    }

    pub(crate) fn node_id(&self) -> composition::NodeId {
        self.node_id
    }

    pub(crate) fn interaction_id(&self) -> Option<interaction::Id> {
        self.interaction_id
    }

    pub(crate) fn role(&self) -> view::Role {
        self.content.role()
    }

    pub(crate) fn rect(&self) -> Rect {
        self.rect
    }

    pub(crate) fn active_rect(&self) -> Rect {
        self.active_rect
    }

    pub(crate) fn label_text(&self) -> Option<&str> {
        self.label.as_ref().map(|label| label.text.as_str())
    }

    pub(crate) fn label_width(&self) -> i32 {
        self.label.as_ref().map_or(0, |label| label.width)
    }

    pub(crate) fn overflow_tip(&self) -> Option<&str> {
        if let (Some(text_box), Some(field)) = (self.text_box(), self.text_box_layout())
            && field.layout().content_area().width() > self.text_box_text_rect().width() as f32
        {
            return Some(text_box.text());
        }
        self.label
            .as_ref()
            .and_then(|label| label.overflow_projection.as_ref())
            .filter(|projection| projection.overflowed())
            .map(text::Selectable::source)
    }

    pub(crate) fn world_text_overflow(&self) -> Option<text_model::Overflow> {
        match &self.content {
            FrameContent::Text(
                TextContent::Label { world_overflow, .. }
                | TextContent::Area { world_overflow, .. }
                | TextContent::InactiveField { world_overflow, .. },
            ) => *world_overflow,
            FrameContent::Button(world) => world.overflow,
            _ => None,
        }
    }

    pub(crate) fn world_text_wrap(&self) -> Option<view::Wrap> {
        match &self.content {
            FrameContent::Text(
                TextContent::Label { world_wrap, .. }
                | TextContent::Area { world_wrap, .. }
                | TextContent::InactiveField { world_wrap, .. },
            ) => *world_wrap,
            FrameContent::Button(world) => world.wrap,
            _ => None,
        }
    }

    pub(crate) fn world_text_align(&self) -> Option<view::Align> {
        match &self.content {
            FrameContent::Text(
                TextContent::Label { world_align, .. }
                | TextContent::Area { world_align, .. }
                | TextContent::InactiveField { world_align, .. },
            ) => *world_align,
            FrameContent::Button(world) => world.align,
            _ => None,
        }
    }

    pub(crate) fn text(&self) -> Option<&str> {
        match &self.content {
            FrameContent::Text(TextContent::Field { display_text, .. }) => display_text.as_deref(),
            _ => None,
        }
    }

    pub(crate) fn text_wrap(&self) -> Option<view::Wrap> {
        match &self.content {
            FrameContent::Text(
                TextContent::Area { model, .. } | TextContent::InactiveField { model, .. },
            ) => Some(model.wrap()),
            FrameContent::Text(TextContent::Field { .. }) => Some(view::Wrap::None),
            _ => None,
        }
    }

    pub(crate) fn is_focused(&self) -> bool {
        self.focused
    }

    pub(crate) fn text_task_focus(&self) -> Option<crate::session::Focus> {
        match &self.content {
            FrameContent::Text(TextContent::Area { model, .. }) => model.focus(),
            FrameContent::Text(TextContent::InactiveField { input, model, .. }) => {
                input.model.focus().or_else(|| model.focus())
            }
            FrameContent::Text(TextContent::Field { input, .. }) => input.model.focus(),
            _ => None,
        }
    }

    pub(crate) fn text_is_selectable(&self) -> bool {
        match &self.content {
            FrameContent::Text(TextContent::Area { model, .. }) => model.mode().is_selectable(),
            FrameContent::Text(TextContent::InactiveField { input, .. })
            | FrameContent::Text(TextContent::Field { input, .. }) => {
                input.model.mode().is_selectable()
            }
            _ => false,
        }
    }

    pub(crate) fn focus_visible(&self) -> bool {
        self.focus_visible
    }

    pub(crate) fn is_selected(&self) -> bool {
        self.selected
    }

    pub(crate) fn is_active_item(&self) -> bool {
        self.active_item
    }

    pub(crate) fn provided_row(&self) -> Option<view::ProvidedRow> {
        self.provided_row
    }

    pub(crate) fn table_row(&self) -> Option<crate::table::Row> {
        self.table_row
    }

    pub(crate) fn table_cell(&self) -> Option<crate::table::Cell> {
        self.table_cell
    }

    pub(crate) fn table_header_cell(&self) -> Option<crate::table::HeaderCell> {
        self.table_header_cell
    }

    pub(crate) fn table_header_presentation(&self) -> Option<crate::table::HeaderPresentation> {
        self.table_header_presentation
    }

    pub(crate) fn table_projection(&self) -> Option<&table::Projection> {
        match &self.content {
            FrameContent::Scroll(ScrollContent::Table {
                resolved: Some(resolved),
            }) => Some(&resolved.projection),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn input_parts(&self) -> Option<control::InputParts> {
        self.text_box_content().map(|input| input.parts)
    }

    pub(crate) fn input_indicator_rect(&self) -> Option<Rect> {
        self.text_box_content()?.parts.indicator()
    }

    pub(crate) fn input_indicator_hint(&self) -> Option<&view::Hint> {
        self.text_box()?.indicator_hint()
    }

    pub(crate) fn input_indicator_target(&self) -> Option<interaction::Target> {
        self.text_box()?.indicator_target()
    }

    #[cfg(test)]
    pub(crate) fn input_is_invalid(&self) -> bool {
        self.text_box().is_some_and(view::TextBox::is_invalid)
    }

    #[cfg(test)]
    pub(crate) fn input_error_message(&self) -> Option<&str> {
        self.text_box()?.error_message()
    }

    pub(crate) fn force_overlay_group(&self) -> bool {
        self.content
            .floating_panel()
            .is_some_and(|content| content.force_overlay_group)
    }

    pub(crate) fn native_popup_material_preference(&self) -> view::NativePopupMaterialPreference {
        self.content
            .floating_panel()
            .map_or(view::NativePopupMaterialPreference::System, |content| {
                content.native_popup_material_preference
            })
    }

    pub(crate) fn popup_placement(&self) -> Option<crate::geometry::PlacementRequest> {
        self.content
            .floating_panel()
            .and_then(|content| content.popup_placement)
    }

    pub(crate) fn popup_context(&self) -> Option<crate::popup::ContextFingerprint> {
        self.content
            .floating_panel()
            .and_then(|content| content.popup_context)
    }

    pub(crate) fn panel_accepts_input(&self) -> bool {
        self.content
            .floating_panel()
            .is_none_or(|content| content.policy.accepts_input())
    }

    pub(crate) fn auxiliary_hint(&self) -> Option<&view::Hint> {
        self.content
            .floating_panel()
            .and_then(|content| content.policy.auxiliary_hint())
    }

    pub(crate) fn is_floating_layer(&self) -> bool {
        self.floating_layer
    }

    pub(crate) fn background(&self) -> Option<scene::Brush> {
        self.background
    }

    pub(crate) fn clip(&self) -> Option<Clip> {
        self.clip
    }

    pub(crate) fn viewport(&self) -> Option<Viewport> {
        match &self.content {
            FrameContent::Text(
                TextContent::Area { layout, .. } | TextContent::InactiveField { layout, .. },
            ) => Some(layout.viewport()),
            FrameContent::Scroll(ScrollContent::Ordinary { viewport }) => *viewport,
            FrameContent::Scroll(ScrollContent::Table { resolved }) => {
                resolved.as_ref().map(|resolved| resolved.viewport)
            }
            FrameContent::VirtualList(content) => {
                content.geometry.as_ref().map(|geometry| geometry.viewport)
            }
            _ => None,
        }
    }

    pub(crate) fn virtual_list_request(&self) -> Option<&crate::virtual_list::Request> {
        match &self.content {
            FrameContent::VirtualList(content) => {
                content.geometry.as_ref().map(|geometry| &geometry.request)
            }
            _ => None,
        }
    }

    pub(crate) fn virtual_row_index_at(&self, point: Point) -> Option<usize> {
        let FrameContent::VirtualList(content) = &self.content else {
            return None;
        };
        let viewport = content.geometry.as_ref()?.viewport;
        if !viewport.rect().contains(point) {
            return None;
        }
        let logical_y = point
            .y()
            .saturating_sub(viewport.rect().y())
            .saturating_add(viewport.resolved_scroll().y())
            .max(0);
        Some(content.model.index_at_offset(logical_y))
    }

    pub(crate) fn resolved_scroll(&self) -> Option<interaction::ScrollOffset> {
        self.viewport().map(Viewport::resolved_scroll)
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.binding().is_none_or(view::Binding::is_enabled)
    }

    pub(crate) fn checked(&self) -> Option<bool> {
        self.binding().and_then(view::Binding::checked)
    }

    pub(crate) fn shortcut_display(&self) -> Option<&[ShortcutPart]> {
        self.binding
            .as_ref()
            .and_then(|content| content.shortcut_display.as_deref())
    }

    pub(crate) fn shortcut_width(&self) -> i32 {
        match (&self.content, &self.binding) {
            (FrameContent::Separator(content), _) => content.shortcut_width.unwrap_or_default(),
            (_, Some(content)) => content.shortcut_width.unwrap_or_default(),
            _ => 0,
        }
    }

    pub(crate) fn shortcut_content_width(&self) -> i32 {
        self.binding
            .as_ref()
            .map_or(0, |content| content.shortcut_content_width)
    }

    pub(crate) fn checkbox(&self) -> Option<&view::Checkbox> {
        match &self.content {
            FrameContent::Choice(ChoiceContent::Checkbox(checkbox)) => Some(checkbox),
            _ => None,
        }
    }

    pub(crate) fn radio(&self) -> Option<&view::Radio> {
        match &self.content {
            FrameContent::Choice(ChoiceContent::Radio(radio)) => Some(radio),
            _ => None,
        }
    }

    pub(crate) fn slider(&self) -> Option<&view::Slider> {
        match &self.content {
            FrameContent::Slider(content) => Some(&content.model),
            _ => None,
        }
    }

    pub(crate) fn slider_track_rect(&self) -> Option<Rect> {
        match &self.content {
            FrameContent::Slider(content) => Some(content.track_rect),
            _ => None,
        }
    }

    fn text_box_content(&self) -> Option<&TextBoxContent> {
        match &self.content {
            FrameContent::Text(TextContent::InactiveField { input, .. })
            | FrameContent::Text(TextContent::Field { input, .. }) => Some(input),
            _ => None,
        }
    }

    pub(crate) fn text_box(&self) -> Option<&view::TextBox> {
        self.text_box_content().map(|input| &input.model)
    }

    pub(crate) fn text_area(&self) -> Option<&view::TextArea> {
        match &self.content {
            FrameContent::Text(
                TextContent::Area { model, .. } | TextContent::InactiveField { model, .. },
            ) => Some(model),
            _ => None,
        }
    }

    fn text_area_display(&self) -> Option<&view::TextArea> {
        match &self.content {
            FrameContent::Text(
                TextContent::Area { display_model, .. }
                | TextContent::InactiveField { display_model, .. },
            ) => Some(display_model),
            _ => None,
        }
    }

    fn text_area_source_position(
        &self,
        position: text_model::buffer::Position,
    ) -> text_model::buffer::Position {
        self.label
            .as_ref()
            .and_then(|label| label.overflow_projection.as_ref())
            .map_or(position, |projection| projection.source_position(position))
    }

    pub(crate) fn text_area_layout(&self) -> Option<&text::Area> {
        match &self.content {
            FrameContent::Text(
                TextContent::Area { layout, .. } | TextContent::InactiveField { layout, .. },
            ) => Some(layout),
            _ => None,
        }
    }

    pub(crate) fn text_area_text_rect(&self) -> Rect {
        match &self.content {
            FrameContent::Text(
                TextContent::Area { text_rect, .. } | TextContent::InactiveField { text_rect, .. },
            ) => *text_rect,
            _ => self.rect,
        }
    }

    pub(crate) fn text_box_layout(&self) -> Option<&text::Field> {
        match &self.content {
            FrameContent::Text(TextContent::Field { layout, .. }) => Some(layout),
            _ => None,
        }
    }

    pub(crate) fn text_box_text_rect(&self) -> Rect {
        match &self.content {
            FrameContent::Text(TextContent::InactiveField { text_rect, .. }) => *text_rect,
            FrameContent::Text(TextContent::Field { text_rect, .. }) => *text_rect,
            _ => self.rect,
        }
    }

    pub(crate) fn text_caret_rect(&self) -> Option<Rect> {
        if !self.is_focused() {
            return None;
        }

        if let Some(text_area) = self.text_area_layout() {
            let caret = text_area.layout().caret()?;
            return clipped_caret_rect(self.text_area_text_rect(), caret);
        }

        let field = self.text_box_layout()?;
        let caret = field.layout().caret()?;
        clipped_caret_rect(self.text_box_text_rect(), caret)
    }

    pub(crate) fn target(&self) -> Option<&interaction::Target> {
        self.target.as_ref()
    }

    fn binding(&self) -> Option<&view::Binding> {
        self.binding.as_ref().map(|content| &content.binding)
    }

    #[cfg(test)]
    pub(crate) fn action(&self) -> Option<&view::Action> {
        self.action.as_ref()
    }

    pub(crate) fn binding_source(&self) -> Option<context::Source> {
        self.binding().map(view::Binding::source)
    }

    pub(crate) fn is_menu_row(&self) -> bool {
        self.participation == Some(view::Participation::MenuRow)
    }

    pub(crate) fn is_palette_row(&self) -> bool {
        self.participation == Some(view::Participation::PaletteRow)
    }

    pub(crate) fn is_auxiliary_text(&self) -> bool {
        self.participation == Some(view::Participation::AuxiliaryText)
    }

    pub(crate) fn table_part(&self) -> Option<view::TablePart> {
        match self.participation {
            Some(view::Participation::Table(part)) => Some(part),
            _ => None,
        }
    }

    pub(crate) fn clip_contains(&self, point: Point) -> bool {
        self.clip.is_none_or(|clip| clip.contains(point))
    }

    pub(super) fn accepts_hit(&self, point: Point) -> bool {
        self.target.is_some() && self.active_rect.contains(point) && self.clip_contains(point)
    }

    pub(crate) fn action_at(&self, point: Point) -> Option<view::Action> {
        if self.role() == view::Role::Slider {
            let value = self.slider_value_at(point)?;
            if let Some(action) = self
                .binding()
                .and_then(|binding| binding.slider_action(value))
            {
                return Some(action);
            }
        }

        self.action.clone()
    }

    pub(crate) fn action_at_with_engine(
        &self,
        point: Point,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        if self.role() == view::Role::TextArea {
            let text_area = self.text_area()?;
            let display = self.text_area_display()?;
            let layout = self.text_area_layout()?;
            let position =
                engine.text_area_position_at(display, layout, self.text_area_text_rect(), point)?;

            return text_area.click_action(self.text_area_source_position(position));
        }

        if self.role() == view::Role::TextBox {
            let text_box = self.text_box()?;
            if let (Some(display), Some(layout)) =
                (self.text_area_display(), self.text_area_layout())
            {
                let position = engine.text_area_position_at(
                    display,
                    layout,
                    self.text_area_text_rect(),
                    point,
                )?;
                return text_box.click_action(self.text_area_source_position(position));
            }
            let layout = self.text_box_layout()?;
            let text_rect = self.text_box_text_rect();
            let position = engine.text_field_position_at(text_box, layout, text_rect, point)?;

            return text_box.click_action(position);
        }

        self.action_at(point)
    }

    pub(crate) fn text_action_at_with_engine(
        &self,
        point: Point,
        kind: crate::text::selection::PointerKind,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        if self.role() == view::Role::TextArea {
            let text_area = self.text_area()?;
            let display = self.text_area_display()?;
            let layout = self.text_area_layout()?;
            let position =
                engine.text_area_position_at(display, layout, self.text_area_text_rect(), point)?;
            return text_area.pointer_action(kind, self.text_area_source_position(position));
        }
        if self.role() == view::Role::TextBox {
            let text_box = self.text_box()?;
            if let (Some(display), Some(layout)) =
                (self.text_area_display(), self.text_area_layout())
            {
                let position = engine.text_area_position_at(
                    display,
                    layout,
                    self.text_area_text_rect(),
                    point,
                )?;
                return text_box.pointer_action(kind, self.text_area_source_position(position));
            }
            let layout = self.text_box_layout()?;
            let position = engine.text_field_position_at(
                text_box,
                layout,
                self.text_box_text_rect(),
                point,
            )?;
            return text_box.pointer_action(kind, position);
        }
        None
    }

    pub(crate) fn drag_action_at_with_engine(
        &self,
        point: Point,
        engine: &mut engine::Engine,
    ) -> Option<view::Action> {
        if self.role() == view::Role::TextArea {
            let text_area = self.text_area()?;
            let display = self.text_area_display()?;
            let layout = self.text_area_layout()?;
            let position =
                engine.text_area_position_at(display, layout, self.text_area_text_rect(), point)?;

            return Some(text_area.drag_action(self.text_area_source_position(position)));
        }

        if self.role() == view::Role::TextBox {
            let text_box = self.text_box()?;
            if let (Some(display), Some(layout)) =
                (self.text_area_display(), self.text_area_layout())
            {
                let position = engine.text_area_position_at(
                    display,
                    layout,
                    self.text_area_text_rect(),
                    point,
                )?;
                return Some(text_box.drag_action(self.text_area_source_position(position)));
            }
            let layout = self.text_box_layout()?;
            let text_rect = self.text_box_text_rect();
            let position = engine.text_field_position_at(text_box, layout, text_rect, point)?;

            return Some(text_box.drag_action(position));
        }

        self.action_at_with_engine(point, engine)
    }

    fn slider_value_at(&self, point: Point) -> Option<f64> {
        let slider = self.slider()?;
        let track = self.slider_track_rect()?;
        let width = track.width().max(1) as f64;
        let offset = point.x().saturating_sub(track.x()) as f64;
        let fraction = offset / width;

        Some(slider.value_at_fraction(fraction))
    }
}

impl FrameContent {
    fn for_node(
        node: &view::Node,
        text_area: Option<view::TextArea>,
        text_area_display: Option<view::TextArea>,
        text_area_layout: Option<text::Area>,
        text_area_text_rect: Rect,
        text_box: Option<TextBoxContent>,
        text_box_layout: Option<text::Field>,
        text_box_text_rect: Rect,
        world_text_overflow: Option<text_model::Overflow>,
        world_text_wrap: Option<view::Wrap>,
        world_text_align: Option<view::Align>,
        slider_track_rect: Option<Rect>,
    ) -> Self {
        match node.role() {
            view::Role::Root => Self::Structural(StructuralRole::Root),
            view::Role::Stack => Self::Structural(StructuralRole::Stack),
            view::Role::Table => Self::Structural(StructuralRole::Table),
            view::Role::MenuBar => Self::Structural(StructuralRole::MenuBar),
            view::Role::Menu => Self::Menu,
            view::Role::Binding => Self::Binding,
            view::Role::Separator => Self::Separator(SeparatorContent {
                shortcut_width: None,
            }),
            view::Role::TextArea => Self::Text(TextContent::Area {
                model: text_area.expect("TextArea role must carry TextArea content"),
                display_model: text_area_display
                    .expect("TextArea frame must carry a display model"),
                layout: text_area_layout.expect("TextArea frame must carry layout content"),
                text_rect: text_area_text_rect,
                world_overflow: world_text_overflow,
                world_wrap: world_text_wrap,
                world_align: world_text_align,
            }),
            view::Role::Button => Self::Button(WorldTextContent {
                overflow: world_text_overflow,
                wrap: world_text_wrap,
                align: world_text_align,
            }),
            view::Role::Checkbox => Self::Choice(ChoiceContent::Checkbox(
                node.checkbox_model()
                    .cloned()
                    .expect("Checkbox role must carry Checkbox content"),
            )),
            view::Role::Radio => Self::Choice(ChoiceContent::Radio(
                node.radio_model()
                    .cloned()
                    .expect("Radio role must carry Radio content"),
            )),
            view::Role::Slider => Self::Slider(SliderContent {
                model: node
                    .slider_model()
                    .cloned()
                    .expect("Slider role must carry Slider content"),
                track_rect: slider_track_rect.expect("Slider frame must carry track geometry"),
            }),
            view::Role::TextBox if text_box_layout.is_none() => {
                Self::Text(TextContent::InactiveField {
                    input: text_box.expect("inactive TextBox must carry TextBox content"),
                    model: text_area.expect("inactive TextBox must carry display content"),
                    display_model: text_area_display
                        .expect("inactive TextBox frame must carry a display model"),
                    layout: text_area_layout
                        .expect("inactive TextBox frame must carry layout content"),
                    text_rect: text_area_text_rect,
                    world_overflow: world_text_overflow,
                    world_wrap: world_text_wrap,
                    world_align: world_text_align,
                })
            }
            view::Role::TextBox => Self::Text(TextContent::Field {
                display_text: node
                    .label_text()
                    .is_none()
                    .then(|| text_box.as_ref().map(|input| input.model.display_text()))
                    .flatten()
                    .map(str::to_owned),
                input: text_box.expect("TextBox role must carry TextBox content"),
                layout: text_box_layout.expect("TextBox frame must carry layout content"),
                text_rect: text_box_text_rect,
            }),
            view::Role::Scroll if node.table_model().is_some() => {
                Self::Scroll(ScrollContent::Table { resolved: None })
            }
            view::Role::Scroll => Self::Scroll(ScrollContent::Ordinary { viewport: None }),
            view::Role::VirtualList => Self::VirtualList(VirtualListContent {
                geometry: None,
                model: node
                    .virtual_list_model()
                    .expect("VirtualList role must carry provider content")
                    .clone(),
            }),
            view::Role::Panel => Self::Structural(StructuralRole::Panel),
            view::Role::FloatingPanel => Self::FloatingPanel(FloatingPanelContent {
                force_overlay_group: node.force_overlay_group(),
                native_popup_material_preference: node.native_popup_material_preference(),
                popup_placement: None,
                popup_context: node.popup_context(),
                policy: node.panel_policy().clone(),
            }),
            view::Role::SectionHeader => Self::Text(TextContent::SectionHeader),
            view::Role::Label => Self::Text(TextContent::Label {
                world_overflow: world_text_overflow,
                world_wrap: world_text_wrap,
                world_align: world_text_align,
            }),
        }
    }

    fn role(&self) -> view::Role {
        match self {
            Self::Structural(StructuralRole::Root) => view::Role::Root,
            Self::Structural(StructuralRole::Stack) => view::Role::Stack,
            Self::Structural(StructuralRole::MenuBar) => view::Role::MenuBar,
            Self::Structural(StructuralRole::Panel) => view::Role::Panel,
            Self::Structural(StructuralRole::Table) => view::Role::Table,
            Self::Menu => view::Role::Menu,
            Self::Binding => view::Role::Binding,
            Self::Separator(_) => view::Role::Separator,
            Self::Text(TextContent::Area { .. }) => view::Role::TextArea,
            Self::Text(TextContent::InactiveField { .. }) => view::Role::TextBox,
            Self::Button(_) => view::Role::Button,
            Self::Choice(ChoiceContent::Checkbox(_)) => view::Role::Checkbox,
            Self::Choice(ChoiceContent::Radio(_)) => view::Role::Radio,
            Self::Slider(_) => view::Role::Slider,
            Self::Text(TextContent::Field { .. }) => view::Role::TextBox,
            Self::Scroll(_) => view::Role::Scroll,
            Self::VirtualList(_) => view::Role::VirtualList,
            Self::FloatingPanel(_) => view::Role::FloatingPanel,
            Self::Text(TextContent::SectionHeader) => view::Role::SectionHeader,
            Self::Text(TextContent::Label { .. }) => view::Role::Label,
        }
    }

    fn floating_panel(&self) -> Option<&FloatingPanelContent> {
        match self {
            Self::FloatingPanel(content) => Some(content),
            _ => None,
        }
    }
}

impl Clip {
    pub(super) fn new(rect: Rect) -> Self {
        Self {
            rect,
            rounding: scene::Rounding::none(),
        }
    }

    pub(super) fn rounded(rect: Rect, rounding: scene::Rounding) -> Self {
        Self { rect, rounding }
    }

    pub(crate) fn rect(self) -> Rect {
        self.rect
    }

    pub(crate) fn rounding(self) -> scene::Rounding {
        self.rounding
    }

    pub(crate) fn contains(self, point: Point) -> bool {
        self.rect.contains(point)
    }
}

fn table_cell_text_rect_for(
    node: &view::Node,
    rect: Rect,
    input_text_rect: Option<Rect>,
    text_area: Option<&view::TextArea>,
    engine: &engine::Engine,
    theme: &Theme,
) -> Rect {
    if node.participation() != Some(view::Participation::Table(view::TablePart::Cell)) {
        return rect;
    }

    let content = input_text_rect.unwrap_or_else(|| control::table_content_rect(rect, theme));
    let Some(text_area) = text_area else {
        return content;
    };
    let measured = engine.text_area_size_for_width(text_area, content.width(), theme);
    let height = measured.height().min(content.height()).max(0);
    let y = content
        .y()
        .saturating_add(content.height().saturating_sub(height) / 2);
    let single_line_height = (theme.typography().interface().size() * 1.25).ceil() as i32;
    let align = node.world_text_align().unwrap_or(view::Align::Start);
    let width = if measured.height() <= single_line_height
        && matches!(align, view::Align::Center | view::Align::End)
    {
        measured.width().min(content.width()).max(0)
    } else {
        content.width()
    };
    let x = match align {
        view::Align::Start | view::Align::Stretch => content.x(),
        view::Align::Center => content
            .x()
            .saturating_add(content.width().saturating_sub(width) / 2),
        view::Align::End => content.right().saturating_sub(width),
    };
    Rect::new(x, y, width, height)
}

fn clipped_caret_rect(rect: Rect, caret: crate::text::layout::Caret) -> Option<Rect> {
    let caret = Rect::new(
        rect.x().saturating_add(caret.x().floor() as i32),
        rect.y().saturating_add(caret.y().floor() as i32),
        1,
        caret.height().ceil().max(0.0) as i32,
    );
    let left = caret.x().max(rect.x());
    let top = caret.y().max(rect.y());
    let right = caret.right().min(rect.right());
    let bottom = caret.bottom().min(rect.bottom());

    (right > left && bottom > top).then(|| Rect::new(left, top, right - left, bottom - top))
}

fn active_rect_for(
    node: &view::Node,
    rect: Rect,
    slider: Option<&view::Slider>,
    label_width: i32,
    theme: &Theme,
) -> Rect {
    match node.role() {
        view::Role::Checkbox
            if matches!(node.participation(), Some(view::Participation::Table(_))) =>
        {
            control::table_choice_mark_rect(rect, theme)
        }
        view::Role::Checkbox | view::Role::Radio => control::choice_mark_rect(rect, theme),
        view::Role::Slider => slider
            .map(|slider| control::slider_active_rect(rect, slider, label_width, theme))
            .unwrap_or(rect),
        _ => rect,
    }
}

fn label_for(node: &view::Node) -> Option<&str> {
    node.label_text().or_else(|| {
        (node.role() == view::Role::Binding)
            .then(|| node.binding().and_then(view::Binding::label))
            .flatten()
    })
}

fn action_for(node: &view::Node) -> Option<view::Action> {
    if let Some(binding) = node.binding() {
        return binding.is_enabled().then(|| binding.action());
    }

    match node.role() {
        view::Role::Menu => node.menu_action(),
        view::Role::TextArea => node
            .text_area_model()
            .and_then(view::TextArea::focus_action),
        view::Role::TextBox => node.text_box_model().and_then(view::TextBox::focus_action),
        view::Role::Root
        | view::Role::Stack
        | view::Role::Table
        | view::Role::MenuBar
        | view::Role::Binding
        | view::Role::Separator
        | view::Role::Button
        | view::Role::Checkbox
        | view::Role::Radio
        | view::Role::Slider
        | view::Role::Scroll
        | view::Role::VirtualList
        | view::Role::Panel
        | view::Role::FloatingPanel
        | view::Role::SectionHeader
        | view::Role::Label => None,
    }
}

fn target_for(node: &view::Node, node_id: composition::NodeId) -> Option<interaction::Target> {
    node.node_pointer_target(node_id)
}
