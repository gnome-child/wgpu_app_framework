use super::control::TextBox;
use super::{Node, View};
use crate::{command, context::Source, interaction, subject, view};

pub(crate) struct CommandPalette {
    query: String,
    selected: usize,
    entries: Vec<Entry>,
    max_results_height: i32,
}

pub(crate) struct Entry {
    trigger: command::AnyTrigger,
    label: String,
    section: String,
}

impl CommandPalette {
    pub(crate) fn new(
        query: String,
        selected: usize,
        entries: Vec<Entry>,
        max_results_height: i32,
    ) -> Self {
        Self {
            query,
            selected,
            entries,
            max_results_height,
        }
    }
}

impl Entry {
    pub(crate) fn new(trigger: command::AnyTrigger, label: String, section: String) -> Self {
        Self {
            trigger,
            label,
            section,
        }
    }
}

impl View {
    pub(crate) fn project_command_palette(&mut self, palette: CommandPalette) {
        self.root.push_child(panel_node(palette));
    }
}

fn panel_node(palette: CommandPalette) -> Node {
    let panel = Node::floating_panel(interaction::CommandPalette::panel_id())
        .with_subject(subject::Segment::from_label("Command Palette"))
        .with_floating_placement(view::node::FloatingPlacement::CenteredMaxEnvelope)
        .with_layout_axis(view::node::Axis::Vertical)
        .with_style(view::Style::new().with_width(view::Dimension::fixed(520)))
        .child(query_node(&palette.query));

    let mut results = Node::scroll()
        .with_interaction_id(interaction::CommandPalette::results_id())
        .with_layout_axis(view::node::Axis::Vertical)
        .with_style(
            view::Style::new()
                .with_width(view::Dimension::Grow)
                .with_height(view::Dimension::fit())
                .with_max_height(palette.max_results_height)
                .with_gap(0),
        );
    let mut previous_section = None;
    for (index, entry) in palette.entries.into_iter().enumerate() {
        if previous_section.as_deref() != Some(entry.section.as_str()) {
            results = results.child(section_node(&entry.section));
            previous_section = Some(entry.section.clone());
        }

        results = results.child(result_node(index == palette.selected, entry));
    }

    panel.child(results)
}

fn query_node(query: &str) -> Node {
    Node::text_box_state(
        TextBox::new(query)
            .with_placeholder("Search commands")
            .with_focus(interaction::CommandPalette::query_focus()),
    )
}

fn section_node(label: &str) -> Node {
    Node::section_header(label.to_owned())
        .with_style(view::Style::new().with_width(view::Dimension::Grow))
}

fn result_node(selected: bool, entry: Entry) -> Node {
    Node::label(entry.label)
        .with_selected(selected)
        .bind_trigger(entry.trigger, Source::Palette)
        .with_style(
            view::Style::new()
                .with_height(view::Dimension::fixed(26))
                .with_width(view::Dimension::Grow),
        )
}
