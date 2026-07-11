use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::{command, interaction, scene, session, view, virtual_list, widget};

/// Synchronous record source for a read-only virtual table.
pub trait Provider {
    fn len(&self) -> usize;
    fn key(&self, row: usize) -> virtual_list::Key;
    fn index_of(&self, key: virtual_list::Key) -> Option<usize>;
    fn cell(&self, row: usize, cell: Cell) -> view::Node;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Width {
    Fixed(i32),
    Weight(u16),
}

#[derive(Clone)]
pub struct Column {
    id: interaction::Id,
    label: String,
    width: Width,
    header: Option<view::Node>,
}

pub struct Table {
    id: interaction::Id,
    row_height: i32,
    header_height: i32,
    columns: Vec<Column>,
    provider: Rc<dyn Provider>,
    width: Option<view::Dimension>,
    height: Option<view::Dimension>,
    max_height: Option<i32>,
    background: Option<scene::Brush>,
}

pub struct TextEditor {
    cell: Cell,
    text: String,
    placeholder: Option<String>,
    validation: Arc<Validation>,
    trigger: Option<command::AnyValueTrigger<String>>,
}

pub struct NumberEditor {
    cell: Cell,
    value: i64,
    placeholder: Option<String>,
    validation: Arc<NumberValidation>,
    trigger: Option<command::AnyValueTrigger<String>>,
}

type Validation = dyn Fn(&str) -> Result<(), String> + Send + Sync;
type NumberValidation = dyn Fn(i64) -> Result<(), String> + Send + Sync;

#[derive(Clone)]
pub(crate) struct Edit {
    cell: Cell,
    validation: Arc<Validation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    table: interaction::Id,
    row: virtual_list::Key,
    column: interaction::Id,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct HeaderCell {
    table: interaction::Id,
    column: interaction::Id,
}

pub(crate) const COLUMN_MIN_WIDTH: i32 = 24;
pub(crate) const DIVIDER_HIT_WIDTH: i32 = 6;

#[derive(Clone)]
pub(crate) struct Model {
    table: interaction::Id,
    columns: Rc<RefCell<Vec<Column>>>,
}

impl HeaderCell {
    #[cfg(test)]
    pub(crate) fn table(self) -> interaction::Id {
        self.table
    }

    #[cfg(test)]
    pub(crate) fn column(self) -> interaction::Id {
        self.column
    }
}

impl Model {
    fn new(table: interaction::Id, columns: Vec<Column>) -> Self {
        Self {
            table,
            columns: Rc::new(RefCell::new(columns)),
        }
    }

    pub(crate) fn project_widths(&self, tables: &crate::interaction::Tables) {
        for column in self.columns.borrow_mut().iter_mut() {
            if let Some(width) = tables.width(HeaderCell {
                table: self.table,
                column: column.id,
            }) {
                column.width = Width::Fixed(width);
            }
        }
    }

    pub(crate) fn id(&self) -> interaction::Id {
        self.table
    }

    pub(crate) fn column_ids(&self) -> Vec<interaction::Id> {
        self.columns.borrow().iter().map(Column::id).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Row {
    table: interaction::Id,
    key: virtual_list::Key,
    index: usize,
}

#[derive(Clone)]
struct Rows {
    table: interaction::Id,
    model: Model,
    provider: Rc<dyn Provider>,
}

impl Width {
    pub fn fixed(value: i32) -> Self {
        Self::Fixed(value.max(0))
    }

    pub fn weight(value: u16) -> Self {
        Self::Weight(value.max(1))
    }

    fn dimension(self) -> view::Dimension {
        match self {
            Self::Fixed(value) => view::Dimension::fixed(value),
            Self::Weight(value) => view::Dimension::weight(value),
        }
    }
}

impl Column {
    pub fn new(id: impl Into<interaction::Id>, label: impl Into<String>, width: Width) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            width,
            header: None,
        }
    }

    pub fn header(mut self, header: impl widget::Widget) -> Self {
        self.header = Some(header.into_node());
        self
    }

    pub fn id(&self) -> interaction::Id {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn width(&self) -> Width {
        self.width
    }

    fn header_node(&self, table: interaction::Id) -> view::Node {
        let identity = HeaderCell {
            table,
            column: self.id,
        };
        sized(
            self.header
                .clone()
                .unwrap_or_else(|| view::Node::label(self.label.clone())),
            self.width,
        )
        .with_table_header_cell(identity)
    }
}

impl Table {
    pub fn new(
        id: impl Into<interaction::Id>,
        row_height: i32,
        columns: impl IntoIterator<Item = Column>,
        provider: impl Provider + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            row_height: row_height.max(1),
            header_height: 28,
            columns: columns.into_iter().collect(),
            provider: Rc::new(provider),
            width: None,
            height: None,
            max_height: None,
            background: None,
        }
    }

    pub fn header_height(mut self, height: i32) -> Self {
        self.header_height = height.max(1);
        self
    }

    pub fn width(mut self, width: view::Dimension) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: view::Dimension) -> Self {
        self.height = Some(height);
        self
    }

    pub fn max_height(mut self, height: i32) -> Self {
        self.max_height = Some(height.max(0));
        self
    }

    pub fn background(mut self, background: scene::Brush) -> Self {
        self.background = Some(background);
        self
    }
}

impl widget::Widget for Table {
    fn into_node(self) -> view::Node {
        let model = Model::new(self.id, self.columns);
        let header = model.columns.borrow().iter().fold(
            view::Node::stack(view::Axis::Horizontal).with_style(
                view::Style::new()
                    .with_width(view::Dimension::grow())
                    .with_height(view::Dimension::fixed(self.header_height)),
            ),
            |header, column| header.child(column.header_node(self.id)),
        );
        let rows = Rows {
            table: self.id,
            model: model.clone(),
            provider: self.provider,
        };
        let body = widget::Widget::into_node(
            crate::VirtualList::new(self.id, self.row_height, rows)
                .selectable()
                .width(view::Dimension::grow())
                .height(view::Dimension::grow()),
        );
        let mut style = view::Style::new();
        if let Some(width) = self.width {
            style = style.with_width(width);
        }
        if let Some(height) = self.height {
            style = style.with_height(height);
        }
        if let Some(max_height) = self.max_height {
            style = style.with_max_height(max_height);
        }
        if let Some(background) = self.background {
            style = style.with_background(background);
        }

        view::Node::table(self.id)
            .with_table_model(model)
            .with_style(style)
            .child(header)
            .child(body)
    }
}

impl Cell {
    pub(crate) fn new(
        table: interaction::Id,
        row: virtual_list::Key,
        column: interaction::Id,
    ) -> Self {
        Self { table, row, column }
    }

    pub fn table(self) -> interaction::Id {
        self.table
    }

    pub fn row(self) -> virtual_list::Key {
        self.row
    }

    pub fn column(self) -> interaction::Id {
        self.column
    }
}

impl TextEditor {
    pub fn new(cell: Cell, text: impl Into<String>) -> Self {
        Self {
            cell,
            text: text.into(),
            placeholder: None,
            validation: Arc::new(|_| Ok(())),
            trigger: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn validate(
        mut self,
        validation: impl Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        self.validation = Arc::new(validation);
        self
    }

    pub fn on_commit<C>(
        mut self,
        map: impl Fn(Cell, String) -> C::Args + Send + Sync + 'static,
    ) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        let cell = self.cell;
        self.trigger = Some(command::AnyValueTrigger::command::<C>(
            move |text: String| map(cell, text),
        ));
        self
    }
}

impl NumberEditor {
    pub fn new(cell: Cell, value: i64) -> Self {
        Self {
            cell,
            value,
            placeholder: None,
            validation: Arc::new(|_| Ok(())),
            trigger: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn validate(
        mut self,
        validation: impl Fn(i64) -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        self.validation = Arc::new(validation);
        self
    }

    pub fn on_commit<C>(
        mut self,
        map: impl Fn(Cell, i64) -> C::Args + Send + Sync + 'static,
    ) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        let cell = self.cell;
        self.trigger = Some(command::AnyValueTrigger::command::<C>(
            move |text: String| {
                let value = text
                    .trim()
                    .parse::<i64>()
                    .expect("NumberEditor validates parsing before building command args");
                map(cell, value)
            },
        ));
        self
    }
}

impl widget::Widget for TextEditor {
    fn into_node(self) -> view::Node {
        editor_node(
            self.cell,
            self.text,
            self.placeholder,
            self.validation,
            self.trigger,
        )
    }
}

impl widget::Widget for NumberEditor {
    fn into_node(self) -> view::Node {
        let domain = Arc::clone(&self.validation);
        let validation: Arc<Validation> = Arc::new(move |text| {
            let value = text
                .trim()
                .parse::<i64>()
                .map_err(|_| "Enter a whole number".to_owned())?;
            domain(value)
        });
        editor_node(
            self.cell,
            self.value.to_string(),
            self.placeholder,
            validation,
            self.trigger,
        )
    }
}

impl Edit {
    pub(crate) fn cell(&self) -> Cell {
        self.cell
    }

    pub(crate) fn validate(&self, text: &str) -> Result<(), String> {
        (self.validation)(text)
    }
}

impl Row {
    #[cfg(test)]
    pub(crate) fn table(self) -> interaction::Id {
        self.table
    }

    #[cfg(test)]
    pub(crate) fn key(self) -> virtual_list::Key {
        self.key
    }

    pub(crate) fn index(self) -> usize {
        self.index
    }
}

impl virtual_list::Provider for Rows {
    fn len(&self) -> usize {
        self.provider.len()
    }

    fn key(&self, index: usize) -> virtual_list::Key {
        self.provider.key(index)
    }

    fn index_of(&self, key: virtual_list::Key) -> Option<usize> {
        self.provider.index_of(key)
    }

    fn row(&self, index: usize) -> view::Node {
        let key = self.provider.key(index);
        let columns = self.model.columns.borrow();
        let children: Vec<view::Node> = columns
            .iter()
            .map(|column| {
                let cell = Cell {
                    table: self.table,
                    row: key,
                    column: column.id,
                };
                sized(self.provider.cell(index, cell), column.width).with_table_cell(cell)
            })
            .collect();
        children.into_iter().fold(
            view::Node::stack(view::Axis::Horizontal).with_table_row(Row {
                table: self.table,
                key,
                index,
            }),
            view::Node::child,
        )
    }
}

fn sized(node: view::Node, width: Width) -> view::Node {
    let style = node.style().clone().with_width(width.dimension());
    node.with_style(style)
}

fn editor_node(
    cell: Cell,
    text: String,
    placeholder: Option<String>,
    validation: Arc<Validation>,
    trigger: Option<command::AnyValueTrigger<String>>,
) -> view::Node {
    let mut model = view::TextBox::new(text.clone()).with_focus(session::Focus::table_cell(cell));
    if let Some(placeholder) = placeholder {
        model = model.with_placeholder(placeholder);
    }
    let mut node = view::Node::text_box_state(model);
    if let Some(trigger) = trigger {
        node = node.bind_text_trigger(text, crate::context::Source::Input, trigger);
    }
    node.with_table_edit(Edit { cell, validation })
}
