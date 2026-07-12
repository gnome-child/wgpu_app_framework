use std::{borrow::Cow, cell::RefCell, cmp::Ordering, collections::HashMap, rc::Rc, sync::Arc};

use crate::{
    command, context, interaction, scene, session, subject, text, view, virtual_list, widget,
};

/// Projects a value into a table cell.
pub trait Value {
    fn text(&self) -> Cow<'_, str>;

    fn align() -> view::Align
    where
        Self: Sized,
    {
        view::Align::Start
    }
}

/// Supplies the product ordering for a sortable value column.
pub trait Sort: Value {
    fn order(&self, other: &Self) -> Ordering;
}

/// Supplies syntax conversion and draft policy for a text-editable value.
pub trait EditText: Value + Sized {
    fn edit_text(&self) -> Cow<'_, str> {
        self.text()
    }

    fn parse(text: &str) -> Result<Self, String>;

    fn input() -> text::Input {
        text::Input::unrestricted()
    }
}

/// Supplies the next value for a directly toggled table cell.
pub trait EditToggle: Value + Sized {
    fn toggled(&self) -> Self;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presentation {
    #[default]
    Compact,
    Expanded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn reversed(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortState {
    column: interaction::Id,
    direction: SortDirection,
}

impl SortState {
    pub fn new(column: impl Into<interaction::Id>, direction: SortDirection) -> Self {
        Self {
            column: column.into(),
            direction,
        }
    }

    pub fn column(self) -> interaction::Id {
        self.column
    }

    pub fn direction(self) -> SortDirection {
        self.direction
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortIntent {
    table: interaction::Id,
    column: interaction::Id,
    direction: SortDirection,
}

impl SortIntent {
    pub fn table(self) -> interaction::Id {
        self.table
    }

    pub fn column(self) -> interaction::Id {
        self.column
    }

    pub fn direction(self) -> SortDirection {
        self.direction
    }
}

/// Canonical application intent emitted by derived sortable headers.
pub struct SortBy;

impl command::Command for SortBy {
    type Args = SortIntent;
    type Output = ();

    const NAME: &'static str = "wgpu_l3.table.sort_by";
}

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

/// A bounded, application-owned record projection for a typed table.
pub struct Source<R> {
    len: usize,
    key: Rc<dyn Fn(usize) -> virtual_list::Key>,
    index_of: Rc<dyn Fn(virtual_list::Key) -> Option<usize>>,
    record: Rc<dyn Fn(usize) -> R>,
}

impl<R> Clone for Source<R> {
    fn clone(&self) -> Self {
        Self {
            len: self.len,
            key: Rc::clone(&self.key),
            index_of: Rc::clone(&self.index_of),
            record: Rc::clone(&self.record),
        }
    }
}

impl<R> Source<R> {
    pub fn new(
        len: usize,
        key: impl Fn(usize) -> virtual_list::Key + 'static,
        index_of: impl Fn(virtual_list::Key) -> Option<usize> + 'static,
        record: impl Fn(usize) -> R + 'static,
    ) -> Self {
        Self {
            len,
            key: Rc::new(key),
            index_of: Rc::new(index_of),
            record: Rc::new(record),
        }
    }
}

type CellProjection<R> = dyn Fn(&R, Cell, Presentation) -> view::Node;
type RecordOrder<R> = dyn Fn(&R, &R) -> Ordering;
type ValueValidation<V> = dyn Fn(&V) -> Result<(), String> + Send + Sync;

/// A heterogeneous typed column after its value and capabilities are erased.
pub struct TypedColumn<R> {
    column: Column,
    cell: Rc<CellProjection<R>>,
    order: Option<Rc<RecordOrder<R>>>,
}

impl<R> TypedColumn<R> {
    pub fn order(&self, left: &R, right: &R) -> Option<Ordering> {
        self.order.as_ref().map(|order| order(left, right))
    }
}

/// A value column while its type capabilities are still available to the builder.
///
/// Capability verbs are absent when the value type lacks their trait:
///
/// ```compile_fail
/// use wgpu_l3::{table::Column, view::Dimension};
/// struct Row { value: f64 }
/// let _ = Column::value("value", "Value", Dimension::fixed(80),
///     |row: &Row| &row.value).sortable();
/// ```
///
/// ```compile_fail
/// use wgpu_l3::{command, table::Column, view::Dimension};
/// struct Row { value: bool }
/// struct Commit;
/// impl command::Command for Commit {
///     type Args = ();
///     type Output = ();
///     const NAME: &'static str = "example.commit";
/// }
/// let _ = Column::value("value", "Value", Dimension::fixed(80),
///     |row: &Row| &row.value).editable::<Commit>(|_, _| ());
/// ```
pub struct ValueColumn<R, V> {
    column: Column,
    accessor: Rc<dyn for<'a> Fn(&'a R) -> &'a V>,
    cell: Option<Rc<CellProjection<R>>>,
    order: Option<Rc<RecordOrder<R>>>,
    overflow: text::Overflow,
    validation: Arc<ValueValidation<V>>,
}

struct TypedProvider<R> {
    source: Source<R>,
    cells: HashMap<interaction::Id, Rc<CellProjection<R>>>,
    projected_record: RefCell<Option<(usize, R)>>,
    presentation: Rc<std::cell::Cell<Presentation>>,
}

#[derive(Clone)]
pub struct Column {
    id: interaction::Id,
    label: String,
    width: view::Dimension,
    resize_override: Option<i32>,
    header: Option<view::Node>,
    sortable: bool,
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
    sort: Option<SortState>,
    presentation: Presentation,
    presentation_projection: Option<Rc<std::cell::Cell<Presentation>>>,
}

pub struct TextEditor {
    cell: Cell,
    text: String,
    placeholder: Option<String>,
    validation: Arc<Validation>,
    trigger: Option<command::AnyValueTrigger<String>>,
    input: text::Input,
}

pub struct NumberEditor {
    cell: Cell,
    value: i64,
    placeholder: Option<String>,
    validation: Arc<NumberValidation>,
    trigger: Option<command::AnyValueTrigger<String>>,
    input: text::Input,
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
    pub(crate) fn table(self) -> interaction::Id {
        self.table
    }

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
            column.resize_override = tables.width(HeaderCell {
                table: self.table,
                column: column.id,
            });
        }
    }

    pub(crate) fn id(&self) -> interaction::Id {
        self.table
    }

    pub(crate) fn column_ids(&self) -> Vec<interaction::Id> {
        self.columns.borrow().iter().map(Column::id).collect()
    }

    pub(crate) fn column_dimensions(&self) -> Vec<(HeaderCell, view::Dimension)> {
        self.columns
            .borrow()
            .iter()
            .map(|column| {
                (
                    HeaderCell {
                        table: self.table,
                        column: column.id,
                    },
                    column.effective_width(),
                )
            })
            .collect()
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

impl Column {
    pub fn new(
        id: impl Into<interaction::Id>,
        label: impl Into<String>,
        width: view::Dimension,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            width,
            resize_override: None,
            header: None,
            sortable: false,
        }
    }

    pub fn value<R, V>(
        id: impl Into<interaction::Id>,
        label: impl Into<String>,
        width: view::Dimension,
        accessor: impl for<'a> Fn(&'a R) -> &'a V + 'static,
    ) -> ValueColumn<R, V>
    where
        R: 'static,
        V: Value + 'static,
    {
        ValueColumn {
            column: Self::new(id, label, width),
            accessor: Rc::new(accessor),
            cell: None,
            order: None,
            overflow: text::Overflow::EllipsisEnd,
            validation: Arc::new(|_| Ok(())),
        }
    }

    pub fn custom<R>(
        id: impl Into<interaction::Id>,
        label: impl Into<String>,
        width: view::Dimension,
        cell: impl Fn(&R, Cell) -> view::Node + 'static,
    ) -> TypedColumn<R>
    where
        R: 'static,
    {
        TypedColumn {
            column: Self::new(id, label, width),
            cell: Rc::new(move |record, identity, _| cell(record, identity)),
            order: None,
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

    pub fn width(&self) -> view::Dimension {
        self.width
    }

    fn effective_width(&self) -> view::Dimension {
        self.resize_override
            .map_or(self.width, view::Dimension::fixed)
    }

    fn header_node(
        &self,
        table: interaction::Id,
        sort: Option<SortState>,
        presentation: Presentation,
    ) -> view::Node {
        let identity = HeaderCell {
            table,
            column: self.id,
        };
        let derived = self.sortable.then(|| {
            let current = sort.filter(|sort| sort.column == self.id);
            let (glyph, direction) = match current.map(|sort| sort.direction) {
                Some(SortDirection::Ascending) => ("↑", SortDirection::Descending),
                Some(SortDirection::Descending) => ("↓", SortDirection::Ascending),
                None => ("↕", SortDirection::Ascending),
            };
            view::Node::button(format!("{} {glyph}", self.label)).bind_command::<SortBy>(
                SortIntent {
                    table,
                    column: self.id,
                    direction,
                },
                context::Source::Button,
            )
        });
        let ordinary = || match presentation {
            Presentation::Compact => view::Node::label(self.label.clone()),
            Presentation::Expanded => {
                view::Node::wrapped_world_text(self.label.clone(), view::Wrap::Word)
            }
        };
        sized(
            self.header.clone().or(derived).unwrap_or_else(ordinary),
            self.effective_width(),
        )
        .with_table_header_cell(identity)
    }
}

impl<R, V> ValueColumn<R, V>
where
    R: 'static,
    V: Value + 'static,
{
    pub fn overflow(mut self, overflow: text::Overflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn validate(
        mut self,
        validation: impl Fn(&V) -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        self.validation = Arc::new(validation);
        self
    }

    pub fn build(self) -> TypedColumn<R> {
        let accessor = Rc::clone(&self.accessor);
        let overflow = self.overflow;
        let cell = self.cell.unwrap_or_else(|| {
            Rc::new(move |record, _, presentation| {
                let value = accessor(record);
                value_node(value, overflow, presentation)
            })
        });
        TypedColumn {
            column: self.column,
            cell,
            order: self.order,
        }
    }
}

impl<R, V> ValueColumn<R, V>
where
    R: 'static,
    V: Sort + 'static,
{
    pub fn sortable(mut self) -> Self {
        let accessor = Rc::clone(&self.accessor);
        self.column.sortable = true;
        self.order = Some(Rc::new(move |left, right| {
            accessor(left).order(accessor(right))
        }));
        self
    }
}

impl<R, V> ValueColumn<R, V>
where
    R: 'static,
    V: EditText + 'static,
{
    pub fn editable<C>(mut self, map: impl Fn(Cell, V) -> C::Args + Send + Sync + 'static) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        let accessor = Rc::clone(&self.accessor);
        let validation = Arc::clone(&self.validation);
        let map = Arc::new(map);
        self.cell = Some(Rc::new(move |record, cell, _| {
            let value = accessor(record);
            let draft_validation = Arc::clone(&validation);
            let commit_validation = Arc::clone(&validation);
            let commit_map = Arc::clone(&map);
            widget::Widget::into_node(
                TextEditor::new(cell, value.edit_text().into_owned())
                    .input(V::input())
                    .validate(move |draft| {
                        let parsed = V::parse(draft)?;
                        draft_validation(&parsed)
                    })
                    .on_commit::<C>(move |cell, draft| {
                        let parsed = V::parse(&draft)
                            .expect("typed table editor validates syntax before commit mapping");
                        commit_validation(&parsed)
                            .expect("typed table editor validates domain before commit mapping");
                        commit_map(cell, parsed)
                    }),
            )
        }));
        self
    }
}

impl<R, V> ValueColumn<R, V>
where
    R: 'static,
    V: EditToggle + 'static,
{
    pub fn toggle<C>(mut self, map: impl Fn(Cell, V) -> C::Args + 'static) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        let accessor = Rc::clone(&self.accessor);
        let map = Rc::new(map);
        self.cell = Some(Rc::new(move |record, cell, _| {
            let next = accessor(record).toggled();
            widget::Widget::into_node(
                widget::Checkbox::new("", false).trigger::<C>(map(cell, next)),
            )
        }));
        self
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
            sort: None,
            presentation: Presentation::Compact,
            presentation_projection: None,
        }
    }

    pub fn typed<R>(
        id: impl Into<interaction::Id>,
        row_height: i32,
        columns: impl IntoIterator<Item = TypedColumn<R>>,
        source: Source<R>,
    ) -> Self
    where
        R: 'static,
    {
        let columns: Vec<_> = columns.into_iter().collect();
        let cells = columns
            .iter()
            .map(|column| (column.column.id, Rc::clone(&column.cell)))
            .collect();
        let presentation = Rc::new(std::cell::Cell::new(Presentation::Compact));
        let provider = TypedProvider {
            source,
            cells,
            projected_record: RefCell::new(None),
            presentation: Rc::clone(&presentation),
        };
        let mut table = Self::new(
            id,
            row_height,
            columns.into_iter().map(|column| column.column),
            provider,
        );
        table.presentation_projection = Some(presentation);
        table
    }

    /// Projects application-owned sort state into derived header controls.
    pub fn sorted_by(
        mut self,
        column: impl Into<interaction::Id>,
        direction: SortDirection,
    ) -> Self {
        self.sort = Some(SortState::new(column, direction));
        self
    }

    pub fn presentation(mut self, presentation: Presentation) -> Self {
        self.presentation = presentation;
        if let Some(projection) = self.presentation_projection.as_ref() {
            projection.set(presentation);
        }
        self
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
        let header_height = match self.presentation {
            Presentation::Compact => view::Dimension::fixed(self.header_height),
            Presentation::Expanded => view::Dimension::fit(),
        };
        let header = model.columns.borrow().iter().fold(
            view::Node::stack(view::Axis::Horizontal).with_style(
                view::Style::new()
                    .with_width(view::Dimension::grow())
                    .with_height(header_height),
            ),
            |header, column| {
                header.child(column.header_node(self.id, self.sort, self.presentation))
            },
        );
        let rows = Rows {
            table: self.id,
            model: model.clone(),
            provider: self.provider,
        };
        let list = match self.presentation {
            Presentation::Compact => crate::VirtualList::new(self.id, self.row_height, rows),
            Presentation::Expanded => crate::VirtualList::variable(self.id, self.row_height, rows),
        };
        let body = widget::Widget::into_node(
            list.selectable()
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

        let surface = view::Node::stack(view::Axis::Vertical)
            .with_style(
                view::Style::new()
                    .with_width(view::Dimension::grow())
                    .with_height(view::Dimension::grow()),
            )
            .child(header)
            .child(body);
        let horizontal_scroll = view::Node::scroll()
            .with_subject(subject::Segment::from_label("Table columns"))
            .with_layout_axis(view::Axis::Horizontal)
            .with_table_model(model)
            .with_style(
                view::Style::new()
                    .with_width(view::Dimension::grow())
                    .with_height(view::Dimension::grow()),
            )
            .child(surface);

        view::Node::table(self.id)
            .with_style(style)
            .child(horizontal_scroll)
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
            input: text::Input::unrestricted(),
        }
    }

    pub fn input(mut self, input: text::Input) -> Self {
        self.input = input;
        self
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
            input: text::Input::unrestricted(),
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
            self.input,
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
            self.input,
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
                sized(self.provider.cell(index, cell), column.effective_width())
                    .with_table_cell(cell)
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

impl<R> Provider for TypedProvider<R> {
    fn len(&self) -> usize {
        self.source.len
    }

    fn key(&self, row: usize) -> virtual_list::Key {
        (self.source.key)(row)
    }

    fn index_of(&self, key: virtual_list::Key) -> Option<usize> {
        (self.source.index_of)(key)
    }

    fn cell(&self, row: usize, cell: Cell) -> view::Node {
        let mut projected = self.projected_record.borrow_mut();
        if projected.as_ref().is_none_or(|(index, _)| *index != row) {
            *projected = Some((row, (self.source.record)(row)));
        }
        let record = &projected
            .as_ref()
            .expect("typed table projects the requested row")
            .1;
        self.cells
            .get(&cell.column)
            .expect("typed table declares a projection for every column")(
            record,
            cell,
            self.presentation.get(),
        )
    }
}

fn sized(node: view::Node, width: view::Dimension) -> view::Node {
    let style = node.style().clone().with_width(width);
    node.with_style(style)
}

fn editor_node(
    cell: Cell,
    text: String,
    placeholder: Option<String>,
    validation: Arc<Validation>,
    trigger: Option<command::AnyValueTrigger<String>>,
    input: text::Input,
) -> view::Node {
    let mut model = view::TextBox::new(text.clone())
        .with_focus(session::Focus::table_cell(cell))
        .with_input(input);
    if let Some(placeholder) = placeholder {
        model = model.with_placeholder(placeholder);
    }
    let mut node = view::Node::text_box_state(model);
    if let Some(trigger) = trigger {
        node = node.bind_text_trigger(text, crate::context::Source::Input, trigger);
    }
    node.with_table_edit(Edit { cell, validation })
}

fn value_node<V: Value>(
    value: &V,
    overflow: text::Overflow,
    presentation: Presentation,
) -> view::Node {
    let text = value.text().into_owned();
    let label = match presentation {
        Presentation::Compact => view::Node::world_text(text, overflow),
        Presentation::Expanded => view::Node::wrapped_world_text(text, view::Wrap::Word),
    };
    match V::align() {
        view::Align::Start | view::Align::Stretch => label,
        align @ (view::Align::Center | view::Align::End) => {
            view::Node::stack(view::Axis::Horizontal)
                .with_style(
                    view::Style::new()
                        .with_width(view::Dimension::grow())
                        .with_justify_content(align),
                )
                .child(label)
        }
    }
}

impl Value for String {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }
}

impl Sort for String {
    fn order(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl EditText for String {
    fn parse(text: &str) -> Result<Self, String> {
        Ok(text.to_owned())
    }
}

impl Value for bool {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(if *self { "✓" } else { "" })
    }
}

impl Sort for bool {
    fn order(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl EditToggle for bool {
    fn toggled(&self) -> Self {
        !self
    }
}

macro_rules! integer_value {
    ($input:expr; $($type:ty),+ $(,)?) => {
        $(
            impl Value for $type {
                fn text(&self) -> Cow<'_, str> {
                    Cow::Owned(self.to_string())
                }

                fn align() -> view::Align {
                    view::Align::End
                }
            }

            impl Sort for $type {
                fn order(&self, other: &Self) -> Ordering {
                    self.cmp(other)
                }
            }

            impl EditText for $type {
                fn parse(text: &str) -> Result<Self, String> {
                    text.trim()
                        .parse::<Self>()
                        .map_err(|_| "Enter a whole number".to_owned())
                }

                fn input() -> text::Input {
                    $input
                }
            }
        )+
    };
}

integer_value!(text::Input::signed_integer(); i8, i16, i32, i64, i128, isize);
integer_value!(text::Input::unsigned_integer(); u8, u16, u32, u64, u128, usize);

macro_rules! float_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl Value for $type {
                fn text(&self) -> Cow<'_, str> {
                    Cow::Owned(self.to_string())
                }

                fn align() -> view::Align {
                    view::Align::End
                }
            }
        )+
    };
}

float_value!(f32, f64);

#[cfg(test)]
mod tests {
    use super::*;

    struct Rank(i32);

    impl Value for Rank {
        fn text(&self) -> Cow<'_, str> {
            Cow::Owned(format!("rank {}", self.0))
        }
    }

    impl Sort for Rank {
        fn order(&self, other: &Self) -> Ordering {
            self.0.cmp(&other.0)
        }
    }

    impl EditText for Rank {
        fn parse(text: &str) -> Result<Self, String> {
            text.parse().map(Self).map_err(|_| "rank".to_owned())
        }

        fn input() -> text::Input {
            text::Input::signed_integer()
        }
    }

    impl EditToggle for Rank {
        fn toggled(&self) -> Self {
            Self(-self.0)
        }
    }

    #[test]
    fn open_value_capabilities_supply_projection_order_parse_and_toggle() {
        assert_eq!(Rank(7).text(), "rank 7");
        assert_eq!(Rank(2).order(&Rank(9)), Ordering::Less);
        assert_eq!(Rank::parse("-4").expect("signed rank").0, -4);
        assert_eq!(Rank(5).toggled().0, -5);
    }

    #[test]
    fn erased_sort_capability_retains_the_typed_product_order() {
        struct Record {
            rank: Rank,
        }
        let column = Column::value(
            "rank",
            "Rank",
            view::Dimension::fixed(80),
            |record: &Record| &record.rank,
        )
        .sortable()
        .build();
        assert_eq!(
            column.order(&Record { rank: Rank(3) }, &Record { rank: Rank(1) }),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn typed_provider_projects_one_record_for_all_cells_in_a_row() {
        let projections = Rc::new(std::cell::Cell::new(0));
        let observed = Rc::clone(&projections);
        let source = Source::new(
            2,
            |row| virtual_list::Key::new(row as u64),
            |key| Some(key.value() as usize),
            move |row| {
                observed.set(observed.get() + 1);
                row
            },
        );
        let table = interaction::Id::new("typed.test");
        let first = interaction::Id::new("first");
        let second = interaction::Id::new("second");
        let cells: HashMap<_, Rc<CellProjection<usize>>> = [first, second]
            .into_iter()
            .map(|id| {
                (
                    id,
                    Rc::new(|record: &usize, _, _| view::Node::label(record.to_string()))
                        as Rc<CellProjection<usize>>,
                )
            })
            .collect();
        let provider = TypedProvider {
            source,
            cells,
            projected_record: RefCell::new(None),
            presentation: Rc::new(std::cell::Cell::new(Presentation::Compact)),
        };
        let key = virtual_list::Key::new(0);
        Provider::cell(&provider, 0, Cell::new(table, key, first));
        Provider::cell(&provider, 0, Cell::new(table, key, second));
        assert_eq!(projections.get(), 1);
        Provider::cell(
            &provider,
            1,
            Cell::new(table, virtual_list::Key::new(1), first),
        );
        assert_eq!(projections.get(), 2);
    }
}
