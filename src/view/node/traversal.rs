use super::super::{
    action::Action,
    control::{Control, TextArea, TextBox},
};
#[cfg(test)]
use super::super::{
    binding::Binding,
    control::{Button, Checkbox, Radio, Slider},
};
use super::{Node, Role};
use crate::{
    command, composition, context::Context as CommandContext, interaction, responder, session,
    state, subject,
};
use std::collections::HashMap;

impl Node {
    pub(in crate::view) fn hover_tip_text_retained(
        &self,
        retained: &composition::Node,
        target: &interaction::Target,
        blocked_by_feedback: bool,
    ) -> Option<(bool, Option<String>)> {
        let blocked_by_feedback = blocked_by_feedback || self.table_edit_error().is_some();
        if !blocked_by_feedback
            && self
                .node_pointer_target(require_retained_id(retained))
                .as_ref()
                == Some(target)
        {
            let text = self.binding().and_then(|binding| {
                binding
                    .hint()
                    .map(str::to_owned)
                    .or_else(|| binding.description().map(str::to_owned))
            });
            return Some((false, text));
        }
        if blocked_by_feedback
            && self
                .node_pointer_target(require_retained_id(retained))
                .as_ref()
                == Some(target)
        {
            return Some((true, None));
        }

        self.children.iter().enumerate().find_map(|(index, child)| {
            child.hover_tip_text_retained(
                retained_child(retained, index),
                target,
                blocked_by_feedback,
            )
        })
    }

    pub(in crate::view) fn first_table_rejection(&self) -> Option<(crate::table::Cell, String)> {
        self.table_cell()
            .zip(self.table_edit_error().map(str::to_owned))
            .or_else(|| self.children.iter().find_map(Node::first_table_rejection))
    }

    pub(in crate::view) fn has_standard_menu_bar(&self) -> bool {
        self.standard_menu_bar || self.children.iter().any(Node::has_standard_menu_bar)
    }

    pub(in crate::view) fn project_standard_menu_bar(
        &mut self,
        projection: &command::BarProjection,
    ) {
        if self.standard_menu_bar {
            self.children =
                super::standard_menu::project(projection, &self.standard_menu_extensions);
            return;
        }

        for child in &mut self.children {
            child.project_standard_menu_bar(projection);
        }
    }

    pub(in crate::view) fn resolve_standard_menu_extensions(
        &mut self,
        registry: &command::Registry,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &CommandContext,
    ) {
        if self.standard_menu_bar {
            for extension in &mut self.standard_menu_extensions {
                extension.resolve_commands(registry, chain, cx);
            }
            return;
        }
        for child in &mut self.children {
            child.resolve_standard_menu_extensions(registry, chain, cx);
        }
    }

    pub(in crate::view) fn context_path_retained(
        &self,
        retained: &composition::Node,
        target: composition::NodeId,
        path: &mut Vec<super::super::ContextOwner>,
    ) -> bool {
        if retained.node_id() == target {
            path.extend(self.context_owners(retained));
            return true;
        }

        for (index, child) in self.children.iter().enumerate() {
            let child_retained = retained_child(retained, index);
            let start = path.len();
            if child.context_path_retained(child_retained, target, path) {
                path.splice(start..start, self.context_owners(retained));
                return true;
            }
        }
        false
    }

    fn is_context_layer(&self) -> bool {
        self.context_menu
            || self.table_model().is_some()
            || self.table_row().is_some()
            || self.table_cell().is_some()
            || self.context_command_binding().is_some()
            || self.context_focus().is_some()
    }

    fn context_command_binding(&self) -> Option<&super::super::Binding> {
        self.context_binding().or_else(|| {
            self.binding()
                .filter(|binding| binding.source() == crate::context::Source::Button)
        })
    }

    fn context_focus(&self) -> Option<session::Focus> {
        self.text_area_model()
            .and_then(TextArea::focus)
            .or_else(|| self.text_box_model().and_then(TextBox::focus))
            .or_else(|| self.table_cell().map(session::Focus::table_cell))
    }

    fn context_owners(&self, retained: &composition::Node) -> Vec<super::super::ContextOwner> {
        if !self.is_context_layer() {
            return Vec::new();
        }
        let table = self
            .table_model()
            .map(crate::table::Model::id)
            .or_else(|| self.table_row().map(crate::table::Row::table))
            .or_else(|| self.table_cell().map(crate::table::Cell::table));
        if self.table_model().is_some() {
            return vec![super::super::ContextOwner::new(
                retained.element_id(),
                self.context_focus(),
                self.context_command_binding().cloned(),
                false,
                table,
                None,
                None,
                super::super::ContextService::Table,
            )];
        }
        if let Some(row) = self.table_row() {
            return vec![super::super::ContextOwner::new(
                retained.element_id(),
                None,
                self.context_command_binding().cloned(),
                false,
                table,
                Some(row),
                None,
                super::super::ContextService::None,
            )];
        }
        if let Some(cell) = self.table_cell() {
            let focus = self.context_focus();
            let text_member = self.text_area_model().is_some() || self.text_box_model().is_some();
            return vec![
                super::super::ContextOwner::new(
                    None,
                    None,
                    None,
                    false,
                    table,
                    None,
                    Some(cell),
                    super::super::ContextService::None,
                ),
                super::super::ContextOwner::new(
                    retained.element_id(),
                    focus,
                    self.context_command_binding().cloned(),
                    false,
                    table,
                    None,
                    None,
                    if text_member {
                        super::super::ContextService::Text
                    } else {
                        super::super::ContextService::None
                    },
                ),
            ];
        }
        let focus = self.context_focus();
        vec![super::super::ContextOwner::new(
            retained.element_id(),
            focus,
            self.context_command_binding().cloned(),
            self.role == Role::Root,
            table,
            None,
            None,
            if focus.is_some() {
                super::super::ContextService::Text
            } else {
                super::super::ContextService::None
            },
        )]
    }

    pub(in crate::view) fn table_cell_focus(
        &self,
        cell: crate::table::Cell,
    ) -> Option<session::Focus> {
        if self.table_cell() == Some(cell) {
            return self
                .focus_at(false)
                .or_else(|| self.children.iter().find_map(|child| child.focus_at(false)));
        }
        self.children
            .iter()
            .find_map(|child| child.table_cell_focus(cell))
    }

    pub(in crate::view) fn table_model_for_id(
        &self,
        id: interaction::Id,
    ) -> Option<&crate::table::Model> {
        if let Some(model) = self.table_model().filter(|model| model.id() == id) {
            return Some(model);
        }
        self.children
            .iter()
            .find_map(|child| child.table_model_for_id(id))
    }

    pub(in crate::view) fn project_table_widths(&mut self, tables: &interaction::Tables) {
        if let Some(model) = self.table_model() {
            model.project_widths(tables);
        }
        if let Some(header) = self.table_header_cell()
            && let Some(width) = tables.width(header)
        {
            self.style = self
                .style
                .clone()
                .with_width(super::super::Dimension::fixed(width));
        }
        if let Some(cell) = self.table_cell()
            && let Some(width) =
                tables.width(crate::table::HeaderCell::new(cell.table(), cell.column()))
        {
            self.style = self
                .style
                .clone()
                .with_width(super::super::Dimension::fixed(width));
        }
        for child in &mut self.children {
            child.project_table_widths(tables);
        }
    }

    pub(in crate::view) fn materialize_virtual_lists(
        &mut self,
        requests: &HashMap<interaction::Id, crate::virtual_list::Materialization>,
        measurements: &HashMap<interaction::Id, crate::virtual_list::Measurements>,
    ) {
        if let Some(model) = self.virtual_list.as_mut() {
            let request = requests
                .get(&model.id())
                .cloned()
                .unwrap_or_else(|| model.initial_materialization());
            self.children = model.materialize(&request, measurements.get(&model.id()));
        }

        for child in &mut self.children {
            child.materialize_virtual_lists(requests, measurements);
        }
    }

    pub(in crate::view) fn collect_selectable_virtual_lists(
        &self,
        models: &mut Vec<crate::virtual_list::Model>,
    ) {
        if let Some(model) = self
            .virtual_list_model()
            .filter(|model| model.is_selectable())
        {
            models.push(model.clone());
        }
        for child in &self.children {
            child.collect_selectable_virtual_lists(models);
        }
    }

    pub(in crate::view) fn virtual_list_model_for_id(
        &self,
        id: interaction::Id,
    ) -> Option<&crate::virtual_list::Model> {
        if let Some(model) = self.virtual_list_model().filter(|model| model.id() == id) {
            return Some(model);
        }
        self.children
            .iter()
            .find_map(|child| child.virtual_list_model_for_id(id))
    }

    pub(in crate::view) fn selectable_virtual_list_for_focus_retained(
        &self,
        focus: session::Focus,
        retained: &composition::Node,
    ) -> Option<&crate::virtual_list::Model> {
        if let Some(model) = self
            .virtual_list_model()
            .filter(|model| model.is_selectable())
            && self.contains_focus_retained(focus, retained)
        {
            return Some(model);
        }
        self.children.iter().enumerate().find_map(|(index, child)| {
            child.selectable_virtual_list_for_focus_retained(focus, retained_child(retained, index))
        })
    }

    pub(in crate::view) fn project_virtual_selections(
        &mut self,
        selections: &[(interaction::Id, crate::selection::Selection)],
    ) {
        if let Some(model) = self
            .virtual_list_model()
            .filter(|model| model.is_selectable())
        {
            let selection = selections
                .iter()
                .find(|(list, _)| *list == model.id())
                .map(|(_, selection)| selection);
            for child in &mut self.children {
                let row = child
                    .provided_row()
                    .expect("selectable VirtualList children carry provider identity");
                child.selected = selection.is_some_and(|selection| selection.contains(row.key()));
                child.active_item =
                    selection.is_some_and(|selection| selection.active() == Some(row.key()));
            }
        }
        for child in &mut self.children {
            child.project_virtual_selections(selections);
        }
    }

    pub(in crate::view) fn project_active_table_cells(
        &mut self,
        tables: &interaction::Tables,
        selections: &[(interaction::Id, crate::selection::Selection)],
    ) {
        if let Some(cell) = self.table_cell() {
            let active_row = selections
                .iter()
                .find(|(table, _)| *table == cell.table())
                .and_then(|(_, selection)| selection.active());
            self.active_item = active_row == Some(cell.row())
                && tables.active_column(cell.table()) == Some(cell.column());
            self.table_edit_error = tables.rejection(cell).map(str::to_owned);
        }
        for child in &mut self.children {
            child.project_active_table_cells(tables, selections);
        }
    }

    #[cfg(test)]
    pub(in crate::view) fn collect_bindings<'a>(&'a self, bindings: &mut Vec<&'a Binding>) {
        if let Some(binding) = &self.binding {
            bindings.push(binding);
        }

        for child in &self.children {
            child.collect_bindings(bindings);
        }
    }

    #[cfg(test)]
    pub(in crate::view) fn collect_text_areas<'a>(&'a self, text_areas: &mut Vec<&'a TextArea>) {
        if let Some(text_area) = self.text_area_model() {
            text_areas.push(text_area);
        }

        for child in &self.children {
            child.collect_text_areas(text_areas);
        }
    }

    #[cfg(test)]
    pub(in crate::view) fn collect_buttons<'a>(&'a self, buttons: &mut Vec<&'a Button>) {
        if let Some(button) = self.button_model() {
            buttons.push(button);
        }

        for child in &self.children {
            child.collect_buttons(buttons);
        }
    }

    #[cfg(test)]
    pub(in crate::view) fn collect_checkboxes<'a>(&'a self, checkboxes: &mut Vec<&'a Checkbox>) {
        if let Some(checkbox) = self.checkbox_model() {
            checkboxes.push(checkbox);
        }

        for child in &self.children {
            child.collect_checkboxes(checkboxes);
        }
    }

    #[cfg(test)]
    pub(in crate::view) fn collect_radios<'a>(&'a self, radios: &mut Vec<&'a Radio>) {
        if let Some(radio) = self.radio_model() {
            radios.push(radio);
        }

        for child in &self.children {
            child.collect_radios(radios);
        }
    }

    #[cfg(test)]
    pub(in crate::view) fn collect_sliders<'a>(&'a self, sliders: &mut Vec<&'a Slider>) {
        if let Some(slider) = self.slider_model() {
            sliders.push(slider);
        }

        for child in &self.children {
            child.collect_sliders(sliders);
        }
    }

    #[cfg(test)]
    pub(in crate::view) fn collect_text_boxes<'a>(&'a self, text_boxes: &mut Vec<&'a TextBox>) {
        if let Some(text_box) = self.text_box_model() {
            text_boxes.push(text_box);
        }

        for child in &self.children {
            child.collect_text_boxes(text_boxes);
        }
    }

    pub(in crate::view) fn contains_focus(&self, focus: session::Focus) -> bool {
        self.contains_focus_at(&focus, false)
    }

    pub(in crate::view) fn contains_enabled_focus_retained(
        &self,
        focus: session::Focus,
        retained: &composition::Node,
    ) -> bool {
        self.contains_focus_retained_at(&focus, retained, true)
    }

    pub(in crate::view) fn contains_focus_retained(
        &self,
        focus: session::Focus,
        retained: &composition::Node,
    ) -> bool {
        self.contains_focus_retained_at(&focus, retained, false)
    }

    pub(in crate::view) fn contains_target_retained(
        &self,
        target: &interaction::Target,
        retained: &composition::Node,
    ) -> bool {
        self.node_pointer_target(retained.node_id()).as_ref() == Some(target)
            || self.children.iter().enumerate().any(|(index, child)| {
                child.contains_target_retained(target, retained_child(retained, index))
            })
    }

    pub(in crate::view) fn collect_virtual_list_pins_retained(
        &self,
        retained: &composition::Node,
        focus: Option<session::Focus>,
        targets: &[interaction::Target],
        pins: &mut std::collections::HashMap<interaction::Id, Vec<crate::virtual_list::Key>>,
    ) {
        if self.virtual_list_model().is_some() {
            for (child, retained_child) in self.children.iter().zip(retained.children()) {
                let Some(row) = child.provided_row() else {
                    continue;
                };
                let pinned_by_focus =
                    focus.is_some_and(|focus| child.contains_focus_retained(focus, retained_child));
                let pinned_by_target = targets
                    .iter()
                    .any(|target| child.contains_target_retained(target, retained_child));
                if pinned_by_focus || pinned_by_target {
                    pins.entry(row.list()).or_default().push(row.key());
                }
            }
        }

        for (child, retained_child) in self.children.iter().zip(retained.children()) {
            child.collect_virtual_list_pins_retained(retained_child, focus, targets, pins);
        }
    }

    pub(in crate::view) fn collect_focus_order_retained(
        &self,
        retained: &composition::Node,
        order: &mut Vec<session::Focus>,
    ) {
        self.collect_focus_order_retained_at(retained, order);
    }

    pub(in crate::view) fn collect_floating_panel_focus_order_retained(
        &self,
        retained: &composition::Node,
        order: &mut Vec<session::Focus>,
    ) -> bool {
        self.collect_floating_panel_focus_order_retained_at(retained, order)
    }

    pub(in crate::view) fn text_commit_for_focus(
        &self,
        focus: session::Focus,
    ) -> Option<super::super::TextCommit> {
        if self
            .text_box_model()
            .and_then(TextBox::focus)
            .is_some_and(|text_focus| text_focus.same_target(&focus))
        {
            return self.text_commit().cloned();
        }

        self.children
            .iter()
            .find_map(|child| child.text_commit_for_focus(focus))
    }

    pub(in crate::view) fn text_box_for_focus(&self, focus: session::Focus) -> Option<&TextBox> {
        if let Some(text_box) = self.text_box_model().filter(|text_box| {
            text_box
                .focus()
                .is_some_and(|text_focus| text_focus.same_target(&focus))
        }) {
            return Some(text_box);
        }

        self.children
            .iter()
            .find_map(|child| child.text_box_for_focus(focus))
    }

    pub(in crate::view) fn draft_text_for_focus(&self, focus: session::Focus) -> Option<String> {
        if let Some(text) = self.text_box_model().and_then(|text_box| {
            text_box
                .focus()
                .is_some_and(|candidate| candidate.same_target(&focus))
                .then(|| text_box.text().to_owned())
        }) {
            return Some(text);
        }
        if self.table_cell().is_some()
            && let Some(text) = self.text_area_model().and_then(|text_area| {
                text_area
                    .focus()
                    .is_some_and(|candidate| candidate.same_target(&focus))
                    .then(|| text_area.buffer().text())
            })
        {
            return Some(text);
        }

        self.children
            .iter()
            .find_map(|child| child.draft_text_for_focus(focus))
    }

    pub(in crate::view) fn text_surface_mode_for_focus(
        &self,
        focus: session::Focus,
    ) -> Option<crate::text::edit::FieldMode> {
        if let Some(mode) = self.text_box_model().and_then(|text_box| {
            text_box
                .focus()
                .is_some_and(|candidate| candidate.same_target(&focus))
                .then(|| text_box.mode())
        }) {
            return Some(mode);
        }
        if let Some(mode) = self.text_area_model().and_then(|text_area| {
            text_area
                .focus()
                .is_some_and(|candidate| candidate.same_target(&focus))
                .then(|| text_area.mode())
        }) {
            return Some(mode);
        }

        self.children
            .iter()
            .find_map(|child| child.text_surface_mode_for_focus(focus))
    }

    pub(in crate::view) fn text_input_target_for_focus(
        &self,
        focus: session::Focus,
    ) -> Option<interaction::Target> {
        if self
            .text_area_model()
            .and_then(TextArea::focus)
            .is_some_and(|text_focus| text_focus.same_target(&focus))
        {
            return self.text_control_target();
        }

        if self
            .text_box_model()
            .and_then(TextBox::focus)
            .is_some_and(|text_focus| text_focus.same_target(&focus))
        {
            return self.text_control_target();
        }

        self.children
            .iter()
            .find_map(|child| child.text_input_target_for_focus(focus))
    }

    #[cfg(test)]
    pub(in crate::view) fn collect_menus<'a>(&'a self, menus: &mut Vec<&'a Node>) {
        if self.role == Role::Menu {
            menus.push(self);
        }

        for child in &self.children {
            child.collect_menus(menus);
        }
    }

    #[cfg(test)]
    pub(in crate::view) fn collect_labels<'a>(&'a self, labels: &mut Vec<&'a str>) {
        if let Some(label) = &self.label {
            labels.push(label);
        }

        for child in &self.children {
            child.collect_labels(labels);
        }
    }

    #[cfg(test)]
    pub(in crate::view) fn collect_floating_panels<'a>(&'a self, panels: &mut Vec<&'a Node>) {
        if self.role == Role::FloatingPanel {
            panels.push(self);
        }

        for child in &self.children {
            child.collect_floating_panels(panels);
        }
    }

    pub(in crate::view) fn resolve_commands(
        &mut self,
        registry: &command::Registry,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &CommandContext,
    ) {
        if let Some(binding) = &mut self.binding {
            binding.resolve(registry, chain, cx);
        }

        for child in &mut self.children {
            child.resolve_commands(registry, chain, cx);
        }
    }

    pub(in crate::view) fn prune_hidden_commands(&mut self) {
        for child in &mut self.children {
            child.prune_hidden_commands();
        }

        self.children.retain(|child| !child.is_hidden_binding());
    }

    pub(in crate::view) fn project_layout_interaction_retained(
        &mut self,
        interaction: &interaction::Interaction,
        retained: &composition::Node,
    ) {
        self.project_layout_interaction_retained_at(interaction, retained);
    }

    fn project_layout_interaction_retained_at(
        &mut self,
        interaction: &interaction::Interaction,
        retained: &composition::Node,
    ) {
        let pointer_target = self.node_pointer_target(require_retained_id(retained));
        self.scroll_offset = if matches!(self.role, Role::Scroll | Role::VirtualList) {
            pointer_target
                .as_ref()
                .map(|target| interaction.scroll().offset(target))
                .unwrap_or_default()
        } else {
            interaction::ScrollOffset::default()
        };

        let text_area_target = if self.role == Role::TextArea {
            pointer_target.clone()
        } else {
            None
        };
        if let Some(Control::TextArea(text_area)) = &mut self.control {
            text_area.project_layout_interaction(interaction, text_area_target.as_ref());
        }

        if let Some(Control::TextBox(text_box)) = &mut self.control {
            text_box.project_layout_interaction(interaction, self.text_commit.is_some());
        }

        for (index, child) in self.children.iter_mut().enumerate() {
            child.project_layout_interaction_retained_at(
                interaction,
                retained_child(retained, index),
            );
        }
    }

    pub(in crate::view) fn project_focus_retained(
        &mut self,
        focus: Option<&session::Focus>,
        retained: &composition::Node,
    ) {
        self.project_focus_retained_at(focus, retained);
    }

    fn project_focus_retained_at(
        &mut self,
        focus: Option<&session::Focus>,
        retained: &composition::Node,
    ) {
        self.focused = self
            .focus_at_retained(retained, true)
            .as_ref()
            .is_some_and(|node_focus| focus.is_some_and(|focus| node_focus.same_target(focus)));
        self.focus_visible =
            self.focused && focus.is_some_and(|focus| focus.shows_focus_indicator());

        if let Some(Control::TextArea(text_area)) = &mut self.control {
            text_area.project_focus(focus);
        }

        if let Some(Control::TextBox(text_box)) = &mut self.control {
            text_box.project_focus(focus);
        }

        for (index, child) in self.children.iter_mut().enumerate() {
            child.project_focus_retained_at(focus, retained_child(retained, index));
        }
    }

    pub(in crate::view) fn floating_panel_for_menu(
        &self,
        menu: &interaction::Menu,
    ) -> Option<Node> {
        if self.role == Role::Menu && self.id == Some(menu.id()) {
            let mut panel = Node::floating_panel(menu.id()).with_panel_anchor_element(menu.id());
            panel.children = self.children.clone();
            return Some(panel);
        }

        self.children
            .iter()
            .find_map(|child| child.floating_panel_for_menu(menu))
    }
}

impl Node {
    pub(in crate::view) fn focus_action_retained(
        &self,
        focus: &session::Focus,
        retained: &composition::Node,
    ) -> Option<Action> {
        self.focus_action_retained_at(focus, retained)
    }

    pub(in crate::view) fn subject_path_for_focus_retained(
        &self,
        focus: session::Focus,
        retained: &composition::Node,
    ) -> Option<subject::Path> {
        self.subject_path_for_focus_retained_at(&focus, retained, &mut Vec::new())
    }

    fn contains_focus_at(&self, focus: &session::Focus, require_enabled: bool) -> bool {
        self.focus_at(require_enabled)
            .as_ref()
            .is_some_and(|node_focus| node_focus.same_target(focus))
            || self
                .children
                .iter()
                .any(|child| child.contains_focus_at(focus, require_enabled))
    }

    fn contains_focus_retained_at(
        &self,
        focus: &session::Focus,
        retained: &composition::Node,
        require_enabled: bool,
    ) -> bool {
        self.focus_at_retained(retained, require_enabled)
            .as_ref()
            .is_some_and(|node_focus| node_focus.same_target(focus))
            || self.children.iter().enumerate().any(|(index, child)| {
                child.contains_focus_retained_at(
                    focus,
                    retained_child(retained, index),
                    require_enabled,
                )
            })
    }

    fn collect_focus_order_retained_at(
        &self,
        retained: &composition::Node,
        order: &mut Vec<session::Focus>,
    ) {
        if let Some(focus) = self.focus_at_retained(retained, true) {
            push_focus(order, focus.keyboard());
        }

        if self.role == Role::Menu {
            return;
        }

        for (index, child) in self.children.iter().enumerate() {
            child.collect_focus_order_retained_at(retained_child(retained, index), order);
        }
    }

    fn collect_floating_panel_focus_order_retained_at(
        &self,
        retained: &composition::Node,
        order: &mut Vec<session::Focus>,
    ) -> bool {
        if self.role == Role::FloatingPanel {
            if self.panel_policy().accepts_input() {
                self.collect_focus_order_retained_at(retained, order);
                return true;
            }
            return false;
        }

        let mut found = false;
        for (index, child) in self.children.iter().enumerate() {
            found |= child.collect_floating_panel_focus_order_retained_at(
                retained_child(retained, index),
                order,
            );
        }

        found
    }

    fn focus_action_retained_at(
        &self,
        focus: &session::Focus,
        retained: &composition::Node,
    ) -> Option<Action> {
        if self
            .focus_at_retained(retained, true)
            .as_ref()
            .is_some_and(|node_focus| node_focus.same_target(focus))
        {
            return self.keyboard_activation_action();
        }

        if self.role == Role::Menu {
            return None;
        }

        self.children.iter().enumerate().find_map(|(index, child)| {
            child.focus_action_retained_at(focus, retained_child(retained, index))
        })
    }

    fn subject_path_for_focus_retained_at(
        &self,
        focus: &session::Focus,
        retained: &composition::Node,
        segments: &mut Vec<subject::Segment>,
    ) -> Option<subject::Path> {
        let pushed = retained.subject().cloned().inspect(|segment| {
            segments.push(segment.clone());
        });

        if self
            .focus_at_retained(retained, false)
            .as_ref()
            .is_some_and(|node_focus| node_focus.same_target(focus))
        {
            return Some(subject::Path::new(segments.clone()));
        }

        for (index, child) in self.children.iter().enumerate() {
            if let Some(path) = child.subject_path_for_focus_retained_at(
                focus,
                retained_child(retained, index),
                segments,
            ) {
                return Some(path);
            }
        }

        if pushed.is_some() {
            segments.pop();
        }
        None
    }

    fn focus_at(&self, require_enabled: bool) -> Option<session::Focus> {
        if let Some(focus) = self.text_area_model().and_then(TextArea::focus) {
            return Some(focus);
        }

        if let Some(focus) = self.text_box_model().and_then(TextBox::focus) {
            return Some(focus);
        }

        if !self.is_keyboard_focusable(require_enabled) {
            return None;
        }

        self.pointer_target()
            .map(|target| session::Focus::control(&target))
    }

    fn focus_at_retained(
        &self,
        retained: &composition::Node,
        require_enabled: bool,
    ) -> Option<session::Focus> {
        if let Some(focus) = self.text_area_model().and_then(TextArea::focus) {
            return Some(focus);
        }

        if let Some(focus) = self.text_box_model().and_then(TextBox::focus) {
            return Some(focus);
        }

        if !self.is_keyboard_focusable(require_enabled) {
            return None;
        }

        self.node_pointer_target(require_retained_id(retained))
            .map(|target| session::Focus::control(&target))
    }

    fn is_keyboard_focusable(&self, require_enabled: bool) -> bool {
        match self.role {
            Role::Menu => true,
            Role::Binding | Role::Button | Role::Checkbox | Role::Radio | Role::Slider => self
                .binding
                .as_ref()
                .is_some_and(|binding| !require_enabled || binding.is_enabled()),
            Role::TextArea | Role::TextBox => true,
            Role::Root
            | Role::Stack
            | Role::Table
            | Role::MenuBar
            | Role::Separator
            | Role::Scroll
            | Role::Panel
            | Role::FloatingPanel
            | Role::SectionHeader
            | Role::Label => false,
            Role::VirtualList => self
                .virtual_list_model()
                .is_some_and(crate::virtual_list::Model::is_selectable),
        }
    }
}

fn push_focus(order: &mut Vec<session::Focus>, focus: session::Focus) {
    if !order.iter().any(|existing| existing.same_target(&focus)) {
        order.push(focus);
    }
}

fn retained_child(parent: &composition::Node, index: usize) -> &composition::Node {
    parent
        .children()
        .get(index)
        .expect("composition tree must match view child order")
}

fn require_retained_id(node: &composition::Node) -> composition::NodeId {
    node.retained_id()
        .expect("retained view traversal requires retained composition identity")
}
