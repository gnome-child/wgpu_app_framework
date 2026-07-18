use std::{
    any::Any, cell::RefCell, cmp::Ordering, collections::HashMap, fmt::Display,
    marker::PhantomData, rc::Rc, str::FromStr, sync::Arc,
};

use crate::{command, context, interaction, list, scene, session, subject, text, view, widget};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presentation {
    #[default]
    Compact,
    Expanded,
}

impl Presentation {
    fn text_policy(self, compact_overflow: text::Overflow) -> (view::Wrap, text::Overflow) {
        match self {
            Self::Compact => (view::Wrap::None, compact_overflow),
            Self::Expanded => (view::Wrap::Word, text::Overflow::Clip),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeaderPresentation {
    sort_direction: Option<SortDirection>,
}

impl HeaderPresentation {
    fn new(sort_direction: Option<SortDirection>) -> Self {
        Self { sort_direction }
    }

    pub(crate) fn sort_direction(self) -> Option<SortDirection> {
        self.sort_direction
    }
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

impl From<SortIntent> for SortState {
    fn from(intent: SortIntent) -> Self {
        Self::new(intent.column, intent.direction)
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
    fn key(&self, row: usize) -> list::Key;
    fn index_of(&self, key: list::Key) -> Option<usize>;
    fn cell(&self, row: usize, cell: Cell) -> view::Node;

    /// Revision of the data projected by every cell in one stable row.
    fn item_revision(&self, row: usize) -> u64;

    /// Monotonic revision covering order and every value reachable by
    /// [`Provider::cell`] during residency-only presentation.
    fn residency_revision(&self) -> Option<u64> {
        None
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A bounded, application-owned record projection for a typed table.
pub struct Source<R> {
    len: usize,
    key: Rc<dyn Fn(usize) -> list::Key>,
    index_of: Rc<dyn Fn(list::Key) -> Option<usize>>,
    record: Rc<dyn Fn(usize) -> R>,
    item_revision: Rc<dyn Fn(usize) -> u64>,
    residency_revision: Option<u64>,
    records: Option<Rc<[R]>>,
}

impl<R> Clone for Source<R> {
    fn clone(&self) -> Self {
        Self {
            len: self.len,
            key: Rc::clone(&self.key),
            index_of: Rc::clone(&self.index_of),
            record: Rc::clone(&self.record),
            item_revision: Rc::clone(&self.item_revision),
            residency_revision: self.residency_revision,
            records: self.records.clone(),
        }
    }
}

impl<R> Source<R> {
    pub fn new(
        len: usize,
        key: impl Fn(usize) -> list::Key + 'static,
        index_of: impl Fn(list::Key) -> Option<usize> + 'static,
        record: impl Fn(usize) -> R + 'static,
        item_revision: impl Fn(usize) -> u64 + 'static,
    ) -> Self {
        Self {
            len,
            key: Rc::new(key),
            index_of: Rc::new(index_of),
            record: Rc::new(record),
            item_revision: Rc::new(item_revision),
            residency_revision: None,
            records: None,
        }
    }

    /// Declares the immutable snapshot generation captured by this source.
    /// An unchanged value must prove that key order and every projected cell
    /// value are unchanged.
    pub fn residency_revision(mut self, revision: u64) -> Self {
        self.residency_revision = Some(revision);
        self
    }
}

impl<R> Source<R>
where
    R: Clone + 'static,
{
    /// Builds a bounded in-memory source whose order follows the table's
    /// projected sort state and the selected column's derived `Ord` ordering.
    pub fn records(records: impl Into<Rc<[R]>>, key: impl Fn(&R) -> list::Key + 'static) -> Self {
        let records = records.into();
        let generation = records.as_ptr() as usize as u64;
        let keys: Rc<[list::Key]> = records.iter().map(key).collect::<Vec<_>>().into();
        let mut indices = HashMap::with_capacity(keys.len());
        for (index, key) in keys.iter().copied().enumerate() {
            assert!(
                indices.insert(key, index).is_none(),
                "table record sources require unique stable keys"
            );
        }
        let indices = Rc::new(indices);
        Self {
            len: records.len(),
            key: {
                let keys = Rc::clone(&keys);
                Rc::new(move |index| keys[index])
            },
            index_of: Rc::new(move |key| indices.get(&key).copied()),
            record: {
                let records = Rc::clone(&records);
                Rc::new(move |index| records[index].clone())
            },
            item_revision: Rc::new(move |_| generation),
            residency_revision: Some(generation),
            records: Some(records),
        }
    }
}

type CellProjection<R> = dyn Fn(&R, Cell, Presentation) -> view::Node;
type ValueValidation<V> = dyn Fn(&V) -> Result<(), String> + Send + Sync;
type OrderProjection = dyn Fn(&dyn Any, &dyn Any) -> Ordering;
type RowContext = dyn Fn(list::Key) -> command::AnyTrigger;

/// A heterogeneous typed column after its value and capabilities are erased.
pub struct TypedColumn<R> {
    column: Column,
    cell: Rc<CellProjection<R>>,
}

#[doc(hidden)]
pub struct DefaultSort;

#[doc(hidden)]
pub struct NoSort;

/// A textual column while its std capabilities remain available to the builder.
///
/// Sorting is the default when the value carries its std ordering. Values
/// without `Ord` must explicitly opt out:
///
/// ```compile_fail
/// use wgpu_l3::{table::Column, view::Dimension};
/// struct Row { value: f64 }
/// let _ = Column::text("value", "Value", Dimension::fixed(80),
///     |row: &Row| &row.value).build();
/// ```
///
/// ```
/// use wgpu_l3::{table::Column, view::Dimension};
/// struct Row { value: f64 }
/// let _ = Column::text("value", "Value", Dimension::fixed(80),
///     |row: &Row| &row.value).unsortable().build();
/// ```
///
/// ```compile_fail
/// use wgpu_l3::{command, table::Column, view::Dimension};
/// struct Shown;
/// impl std::fmt::Display for Shown {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         f.write_str("shown")
///     }
/// }
/// struct Row { value: Shown }
/// struct Commit;
/// impl command::Command for Commit {
///     type Args = ();
///     type Output = ();
///     const NAME: &'static str = "example.commit";
/// }
/// let _ = Column::text("value", "Value", Dimension::fixed(80),
///     |row: &Row| &row.value).editable::<Commit>(|_, _| ());
/// ```
pub struct TextColumn<R, V, S = DefaultSort> {
    column: Column,
    accessor: Rc<dyn for<'a> Fn(&'a R) -> &'a V>,
    cell: Option<Rc<CellProjection<R>>>,
    overflow: text::Overflow,
    align: view::Align,
    input: text::Input,
    validation: Arc<ValueValidation<V>>,
    sort: PhantomData<S>,
}

/// A Boolean-medium column with optional reverse conversion for interaction.
///
/// Boolean presentation defaults to derived `Ord` sorting; `.unsortable()`
/// explicitly removes the ordering requirement and header affordance.
///
/// Read-only projection needs only the forward conversion; toggling is absent
/// until the value can honestly be reconstructed from the Boolean medium:
///
/// ```compile_fail
/// use wgpu_l3::{command, table::Column, view::Dimension};
/// #[derive(Clone)]
/// struct OneWay(bool);
/// impl From<OneWay> for bool { fn from(value: OneWay) -> Self { value.0 } }
/// struct Row { value: OneWay }
/// struct Commit;
/// impl command::Command for Commit {
///     type Args = ();
///     type Output = ();
///     const NAME: &'static str = "example.commit_boolean";
/// }
/// let _ = Column::boolean("value", "Value", Dimension::fixed(80),
///     |row: &Row| &row.value).toggle::<Commit>(|_, _| ());
/// ```
pub struct BooleanColumn<R, V, S = DefaultSort> {
    column: Column,
    accessor: Rc<dyn for<'a> Fn(&'a R) -> &'a V>,
    cell: Option<Rc<CellProjection<R>>>,
    sort: PhantomData<S>,
}

struct ResolvedOrder {
    sort: SortState,
    rows: Vec<usize>,
    indices: Vec<usize>,
}

struct TypedProvider<R> {
    source: Source<R>,
    cells: HashMap<interaction::Id, Rc<CellProjection<R>>>,
    projected_record: RefCell<Option<(usize, R)>>,
    presentation: Rc<std::cell::Cell<Presentation>>,
    sort: Rc<std::cell::Cell<Option<SortState>>>,
    order: RefCell<Option<ResolvedOrder>>,
    orderings: HashMap<interaction::Id, Rc<OrderProjection>>,
}

#[derive(Clone)]
pub struct Column {
    id: interaction::Id,
    label: String,
    width: view::Dimension,
    resize_override: Option<i32>,
    header: Option<view::Node>,
    ordering: Option<Rc<OrderProjection>>,
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
    sort_projection: Option<Rc<std::cell::Cell<Option<SortState>>>>,
    row_context: Option<Rc<RowContext>>,
    configuration: Option<crate::scroll::Configuration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    table: interaction::Id,
    row: list::Key,
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
    pub(crate) fn new(table: interaction::Id, column: interaction::Id) -> Self {
        Self { table, column }
    }

    pub(crate) fn table(self) -> interaction::Id {
        self.table
    }

    pub(crate) fn column(self) -> interaction::Id {
        self.column
    }
}

impl Model {
    pub(crate) fn same_scene_state(&self, other: &Self) -> bool {
        self.table == other.table && self.column_dimensions() == other.column_dimensions()
    }

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

    pub(crate) fn column_dimensions(&self) -> Vec<(HeaderCell, view::Dimension, Option<i32>)> {
        self.columns
            .borrow()
            .iter()
            .map(|column| {
                (
                    HeaderCell {
                        table: self.table,
                        column: column.id,
                    },
                    column.width,
                    column.resize_override,
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Row {
    table: interaction::Id,
    key: list::Key,
    index: usize,
}

#[derive(Clone)]
struct Rows {
    table: interaction::Id,
    model: Model,
    provider: Rc<dyn Provider>,
    row_context: Option<Rc<RowContext>>,
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
            ordering: None,
        }
    }

    pub fn text<R, V>(
        id: impl Into<interaction::Id>,
        label: impl Into<String>,
        width: view::Dimension,
        accessor: impl for<'a> Fn(&'a R) -> &'a V + 'static,
    ) -> TextColumn<R, V>
    where
        R: 'static,
        V: Display + 'static,
    {
        TextColumn {
            column: Self::new(id, label, width),
            accessor: Rc::new(accessor),
            cell: None,
            overflow: text::Overflow::EllipsisEnd,
            align: view::Align::Start,
            input: text::Input::unrestricted(),
            validation: Arc::new(|_| Ok(())),
            sort: PhantomData,
        }
    }

    pub fn boolean<R, V>(
        id: impl Into<interaction::Id>,
        label: impl Into<String>,
        width: view::Dimension,
        accessor: impl for<'a> Fn(&'a R) -> &'a V + 'static,
    ) -> BooleanColumn<R, V>
    where
        R: 'static,
        V: Clone + Into<bool> + 'static,
    {
        BooleanColumn {
            column: Self::new(id, label, width),
            accessor: Rc::new(accessor),
            cell: None,
            sort: PhantomData,
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

    fn header_node(&self, table: interaction::Id, sort: Option<SortState>) -> view::Node {
        let identity = HeaderCell {
            table,
            column: self.id,
        };
        let derived = self.ordering.is_some().then(|| {
            let current = sort.filter(|sort| sort.column == self.id);
            let direction = match current.map(|sort| sort.direction) {
                Some(SortDirection::Ascending) => SortDirection::Descending,
                Some(SortDirection::Descending) | None => SortDirection::Ascending,
            };
            (
                view::Node::button(self.label.clone())
                    .with_world_text_policy(
                        self.label.clone(),
                        view::Wrap::None,
                        text::Overflow::EllipsisEnd,
                    )
                    .bind_command::<SortBy>(
                        SortIntent {
                            table,
                            column: self.id,
                            direction,
                        },
                        context::Source::Button,
                    ),
                current.map(|sort| sort.direction),
            )
        });
        let ordinary = || {
            (
                view::Node::world_text(self.label.clone(), text::Overflow::EllipsisEnd),
                None,
            )
        };
        let (header, sort_direction) = self
            .header
            .clone()
            .map(|header| (header, None))
            .or(derived)
            .unwrap_or_else(ordinary);
        sized(
            header.with_table_header_presentation(HeaderPresentation::new(sort_direction)),
            self.effective_width(),
        )
        .with_table_header_cell(identity)
    }
}

impl<R, V, S> TextColumn<R, V, S>
where
    R: 'static,
    V: Display + 'static,
{
    pub fn overflow(mut self, overflow: text::Overflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn align(mut self, align: view::Align) -> Self {
        self.align = align;
        self
    }

    pub fn input(mut self, input: text::Input) -> Self {
        self.input = input;
        self
    }

    pub fn validate<E>(
        mut self,
        validation: impl Fn(&V) -> Result<(), E> + Send + Sync + 'static,
    ) -> Self
    where
        E: Display,
    {
        self.validation =
            Arc::new(move |value| validation(value).map_err(|error| error.to_string()));
        self
    }

    pub fn unsortable(self) -> TextColumn<R, V, NoSort> {
        TextColumn {
            column: self.column,
            accessor: self.accessor,
            cell: self.cell,
            overflow: self.overflow,
            align: self.align,
            input: self.input,
            validation: self.validation,
            sort: PhantomData,
        }
    }

    fn build_with_order(mut self, ordering: Option<Rc<OrderProjection>>) -> TypedColumn<R> {
        self.column.ordering = ordering;
        let accessor = Rc::clone(&self.accessor);
        let overflow = self.overflow;
        let align = self.align;
        let cell = self.cell.unwrap_or_else(|| {
            Rc::new(move |record, cell, presentation| {
                let value = accessor(record);
                display_value_node(cell, value.to_string(), align, overflow, presentation)
            })
        });
        TypedColumn {
            column: self.column,
            cell,
        }
    }
}

impl<R, V> TextColumn<R, V>
where
    R: 'static,
    V: Display + Ord + 'static,
{
    pub fn build(self) -> TypedColumn<R> {
        let ordering = ordering_projection(Rc::clone(&self.accessor));
        self.build_with_order(Some(ordering))
    }
}

impl<R, V> TextColumn<R, V, NoSort>
where
    R: 'static,
    V: Display + 'static,
{
    pub fn build(self) -> TypedColumn<R> {
        self.build_with_order(None)
    }
}

impl<R, V, S> TextColumn<R, V, S>
where
    R: 'static,
    V: Display + FromStr + 'static,
    V::Err: Display,
{
    pub fn editable<C>(mut self, map: impl Fn(Cell, V) -> C::Args + Send + Sync + 'static) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        let accessor = Rc::clone(&self.accessor);
        let validation = Arc::clone(&self.validation);
        let map = Arc::new(map);
        let overflow = self.overflow;
        let align = self.align;
        let input = self.input;
        self.cell = Some(Rc::new(move |record, cell, presentation| {
            let value = accessor(record);
            let commit_validation = Arc::clone(&validation);
            let commit_map = Arc::clone(&map);
            let (wrap, overflow) = presentation.text_policy(overflow);
            widget::Widget::into_node(
                widget::TextBox::new(value.to_string())
                    .focus(session::Focus::table_cell(cell))
                    .input(input)
                    .inactive_display(align, wrap, overflow)
                    .try_commit_with_formatted::<C>(move |draft| {
                        let parsed = draft.parse::<V>().map_err(|error| error.to_string())?;
                        commit_validation(&parsed)?;
                        Ok(commit_map(cell, parsed))
                    }),
            )
        }));
        self
    }
}

impl<R, V, S> BooleanColumn<R, V, S>
where
    R: 'static,
    V: Clone + Into<bool> + 'static,
{
    pub fn unsortable(self) -> BooleanColumn<R, V, NoSort> {
        BooleanColumn {
            column: self.column,
            accessor: self.accessor,
            cell: self.cell,
            sort: PhantomData,
        }
    }

    fn build_with_order(mut self, ordering: Option<Rc<OrderProjection>>) -> TypedColumn<R> {
        self.column.ordering = ordering;
        let accessor = Rc::clone(&self.accessor);
        let cell = self.cell.unwrap_or_else(|| {
            Rc::new(move |record, _, _| {
                let value: bool = accessor(record).clone().into();
                widget::Widget::into_node(widget::Checkbox::new("", value))
            })
        });
        TypedColumn {
            column: self.column,
            cell,
        }
    }
}

impl<R, V> BooleanColumn<R, V>
where
    R: 'static,
    V: Clone + Into<bool> + Ord + 'static,
{
    pub fn build(self) -> TypedColumn<R> {
        let ordering = ordering_projection(Rc::clone(&self.accessor));
        self.build_with_order(Some(ordering))
    }
}

impl<R, V> BooleanColumn<R, V, NoSort>
where
    R: 'static,
    V: Clone + Into<bool> + 'static,
{
    pub fn build(self) -> TypedColumn<R> {
        self.build_with_order(None)
    }
}

impl<R, V, S> BooleanColumn<R, V, S>
where
    R: 'static,
    V: Clone + Into<bool> + From<bool> + 'static,
{
    pub fn toggle<C>(mut self, map: impl Fn(Cell, V) -> C::Args + 'static) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        let accessor = Rc::clone(&self.accessor);
        let map = Rc::new(map);
        self.cell = Some(Rc::new(move |record, cell, _| {
            let value: bool = accessor(record).clone().into();
            let next = V::from(!value);
            widget::Widget::into_node(
                widget::Checkbox::new("", value).trigger::<C>(map(cell, next)),
            )
        }));
        self
    }
}

fn ordering_projection<R, V>(accessor: Rc<dyn for<'a> Fn(&'a R) -> &'a V>) -> Rc<OrderProjection>
where
    R: 'static,
    V: Ord + 'static,
{
    Rc::new(move |left, right| {
        let left = left
            .downcast_ref::<R>()
            .expect("typed table ordering receives its declared record type");
        let right = right
            .downcast_ref::<R>()
            .expect("typed table ordering receives its declared record type");
        accessor(left).cmp(accessor(right))
    })
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
            sort_projection: None,
            row_context: None,
            configuration: None,
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
        let orderings = columns
            .iter()
            .filter_map(|column| {
                column
                    .column
                    .ordering
                    .as_ref()
                    .map(|ordering| (column.column.id, Rc::clone(ordering)))
            })
            .collect();
        let presentation = Rc::new(std::cell::Cell::new(Presentation::Compact));
        let sort = Rc::new(std::cell::Cell::new(None));
        let provider = TypedProvider {
            source,
            cells,
            projected_record: RefCell::new(None),
            presentation: Rc::clone(&presentation),
            sort: Rc::clone(&sort),
            order: RefCell::new(None),
            orderings,
        };
        let mut table = Self::new(
            id,
            row_height,
            columns.into_iter().map(|column| column.column),
            provider,
        );
        table.presentation_projection = Some(presentation);
        table.sort_projection = Some(sort);
        table
    }

    /// Projects application-owned sort state into derived header controls.
    pub fn sorted_by(
        mut self,
        column: impl Into<interaction::Id>,
        direction: SortDirection,
    ) -> Self {
        self.sort = Some(SortState::new(column, direction));
        if let Some(projection) = self.sort_projection.as_ref() {
            projection.set(self.sort);
        }
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

    pub fn configuration(mut self, configuration: crate::scroll::Configuration) -> Self {
        self.configuration = Some(configuration);
        self
    }

    /// Adds one typed context-only command to each virtual row. The stable row
    /// key supplies concrete arguments without changing primary-click behavior.
    pub fn context_rows<C>(mut self, map: impl Fn(list::Key) -> C::Args + 'static) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.row_context = Some(Rc::new(move |key| {
            command::AnyTrigger::command::<C>(map(key))
        }));
        self
    }
}

impl widget::Widget for Table {
    fn into_node(self) -> view::Node {
        let model = Model::new(self.id, self.columns);
        let header = model.columns.borrow().iter().fold(
            view::Node::stack(view::Axis::Horizontal)
                .with_style(
                    view::Style::new()
                        .with_width(view::Dimension::grow())
                        .with_height(view::Dimension::fixed(self.header_height)),
                )
                .with_table_header_band(),
            |header, column| header.child(column.header_node(self.id, self.sort)),
        );
        let rows = Rows {
            table: self.id,
            model: model.clone(),
            provider: self.provider,
            row_context: self.row_context,
        };
        let mut list = match self.presentation {
            Presentation::Compact => crate::List::new(self.id, self.row_height, rows.clone(), rows),
            Presentation::Expanded => {
                crate::List::variable(self.id, self.row_height, rows.clone(), rows)
            }
        };
        if let Some(configuration) = self.configuration {
            list = list.configuration(configuration);
        }
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
        let mut horizontal_scroll = view::Node::table_scroll(model)
            .with_subject(subject::Segment::from_label("Table columns"))
            .with_style(
                view::Style::new()
                    .with_width(view::Dimension::grow())
                    .with_height(view::Dimension::grow()),
            )
            .child(surface);
        if let Some(configuration) = self.configuration {
            horizontal_scroll = horizontal_scroll.with_scroll_configuration(configuration);
        }

        view::Node::table(self.id)
            .with_style(style)
            .child(horizontal_scroll)
    }
}

impl Cell {
    pub(crate) fn new(table: interaction::Id, row: list::Key, column: interaction::Id) -> Self {
        Self { table, row, column }
    }

    pub fn table(self) -> interaction::Id {
        self.table
    }

    pub fn row(self) -> list::Key {
        self.row
    }

    pub fn column(self) -> interaction::Id {
        self.column
    }
}

impl Row {
    pub(crate) fn table(self) -> interaction::Id {
        self.table
    }

    pub(crate) fn key(self) -> list::Key {
        self.key
    }

    pub(crate) fn index(self) -> usize {
        self.index
    }

    pub(crate) fn at_index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }
}

impl list::Model for Rows {
    fn len(&self) -> usize {
        self.provider.len()
    }

    fn key(&self, index: usize) -> list::Key {
        self.provider.key(index)
    }

    fn index_of(&self, key: list::Key) -> Option<usize> {
        self.provider.index_of(key)
    }

    fn membership_revision(&self) -> u64 {
        0
    }

    fn changes_since(&self, _revision: u64) -> Vec<list::Change> {
        Vec::new()
    }

    fn item_revision(&self, index: usize) -> u64 {
        let mut revision = self.provider.item_revision(index);
        let columns = self.model.columns.borrow();
        revision = revision
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(columns.len() as u64);
        for column in columns.iter() {
            for byte in column.id.as_str().bytes() {
                revision = revision
                    .wrapping_mul(0x0000_0100_0000_01b3)
                    .wrapping_add(u64::from(byte));
            }
            let width = match column.effective_width() {
                view::Dimension::Fit => [0, 0, 0],
                view::Dimension::Flexible { weight, minimum } => {
                    [1, u64::from(weight), minimum as u64]
                }
                view::Dimension::Fixed(value) => [2, value as u64, 0],
                view::Dimension::Percent(value) => [3, u64::from(value.to_bits()), 0],
            };
            for component in width {
                revision = revision
                    .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                    .wrapping_add(component);
            }
        }
        revision
            .wrapping_mul(2)
            .wrapping_add(u64::from(self.row_context.is_some()))
    }

    fn residency_revision(&self) -> Option<u64> {
        let mut revision = self.provider.residency_revision()?;
        let columns = self.model.columns.borrow();
        revision = revision
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(columns.len() as u64);
        for column in columns.iter() {
            for byte in column.id.as_str().bytes() {
                revision = revision
                    .wrapping_mul(0x0000_0100_0000_01b3)
                    .wrapping_add(u64::from(byte));
            }
            for component in match column.effective_width() {
                view::Dimension::Fit => [0, 0, 0],
                view::Dimension::Flexible { weight, minimum } => {
                    [1, u64::from(weight), minimum as u64]
                }
                view::Dimension::Fixed(value) => [2, value as u64, 0],
                view::Dimension::Percent(value) => [3, u64::from(value.to_bits()), 0],
            } {
                revision = revision
                    .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                    .wrapping_add(component);
            }
        }
        Some(
            revision
                .wrapping_mul(2)
                .wrapping_add(u64::from(self.row_context.is_some())),
        )
    }
}

impl list::Factory for Rows {
    fn revision(&self) -> u64 {
        0
    }

    fn bind(&self, _slot: list::Slot, index: usize) -> view::Node {
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
        let mut row = view::Node::stack(view::Axis::Horizontal).with_table_row(Row {
            table: self.table,
            key,
            index,
        });
        if let Some(context) = self.row_context.as_ref() {
            row = row.bind_context_trigger(context(key));
        }
        children.into_iter().fold(row, view::Node::child)
    }
}

impl<R> TypedProvider<R>
where
    R: 'static,
{
    fn refresh_order(&self) {
        let Some(records) = self.source.records.as_ref() else {
            return;
        };
        let Some(sort) = self.sort.get() else {
            self.order.borrow_mut().take();
            return;
        };
        if self
            .order
            .borrow()
            .as_ref()
            .is_some_and(|order| order.sort == sort)
        {
            return;
        }
        let Some(ordering) = self.orderings.get(&sort.column) else {
            self.order.borrow_mut().take();
            return;
        };
        let mut rows: Vec<_> = (0..records.len()).collect();
        rows.sort_by(|left, right| {
            let ordering = ordering(&records[*left] as &dyn Any, &records[*right] as &dyn Any);
            match sort.direction {
                SortDirection::Ascending => ordering,
                SortDirection::Descending => ordering.reverse(),
            }
        });
        let mut indices = vec![0; rows.len()];
        for (index, row) in rows.iter().copied().enumerate() {
            indices[row] = index;
        }
        *self.order.borrow_mut() = Some(ResolvedOrder {
            sort,
            rows,
            indices,
        });
    }

    fn source_row(&self, row: usize) -> usize {
        self.refresh_order();
        self.order
            .borrow()
            .as_ref()
            .map_or(row, |order| order.rows[row])
    }

    fn projected_index(&self, source_index: usize) -> usize {
        self.refresh_order();
        self.order
            .borrow()
            .as_ref()
            .map_or(source_index, |order| order.indices[source_index])
    }
}

impl<R> Provider for TypedProvider<R>
where
    R: 'static,
{
    fn len(&self) -> usize {
        self.source.len
    }

    fn key(&self, row: usize) -> list::Key {
        (self.source.key)(self.source_row(row))
    }

    fn index_of(&self, key: list::Key) -> Option<usize> {
        (self.source.index_of)(key).map(|index| self.projected_index(index))
    }

    fn item_revision(&self, row: usize) -> u64 {
        let source_row = self.source_row(row);
        (self.source.item_revision)(source_row)
            .wrapping_mul(2)
            .wrapping_add(match self.presentation.get() {
                Presentation::Compact => 0,
                Presentation::Expanded => 1,
            })
    }

    fn residency_revision(&self) -> Option<u64> {
        let mut revision = self.source.residency_revision?;
        revision = revision
            .wrapping_mul(2)
            .wrapping_add(match self.presentation.get() {
                Presentation::Compact => 0,
                Presentation::Expanded => 1,
            });
        if let Some(sort) = self.sort.get() {
            for byte in sort.column.as_str().bytes() {
                revision = revision
                    .wrapping_mul(0x0000_0100_0000_01b3)
                    .wrapping_add(u64::from(byte));
            }
            revision = revision.wrapping_mul(2).wrapping_add(match sort.direction {
                SortDirection::Ascending => 0,
                SortDirection::Descending => 1,
            });
        }
        Some(revision)
    }

    fn cell(&self, row: usize, cell: Cell) -> view::Node {
        let source_row = self.source_row(row);
        let mut projected = self.projected_record.borrow_mut();
        if projected
            .as_ref()
            .is_none_or(|(index, _)| *index != source_row)
        {
            *projected = Some((source_row, (self.source.record)(source_row)));
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

fn display_value_node(
    cell: Cell,
    text: String,
    align: view::Align,
    overflow: text::Overflow,
    presentation: Presentation,
) -> view::Node {
    let (wrap, overflow) = presentation.text_policy(overflow);
    view::Node::text_area_state(
        view::TextArea::new(text.clone())
            .with_focus(session::Focus::table_cell(cell))
            .with_wrap(wrap)
            .read_only(),
    )
    .with_world_text_policy(text, wrap, overflow)
    .with_world_text_alignment(align)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CommitText;

    impl command::Command for CommitText {
        type Args = (Cell, String);
        type Output = ();

        const NAME: &'static str = "test.commit_text";
    }

    struct CommitAddress;

    impl command::Command for CommitAddress {
        type Args = (Cell, std::net::IpAddr);
        type Output = ();

        const NAME: &'static str = "test.commit_address";
    }

    struct CommitSwitch;

    impl command::Command for CommitSwitch {
        type Args = (Cell, Switch);
        type Output = ();

        const NAME: &'static str = "test.commit_switch";
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Switch {
        Off,
        On,
    }

    #[test]
    fn typed_text_cells_preserve_selectable_read_only_and_editable_control_semantics() {
        struct Record {
            value: String,
        }
        let read_only = Column::text(
            "read-only",
            "Read only",
            view::Dimension::fixed(80),
            |record: &Record| &record.value,
        )
        .build();
        let editable = Column::text(
            "editable",
            "Editable",
            view::Dimension::fixed(80),
            |record: &Record| &record.value,
        )
        .editable::<CommitText>(|cell, value| (cell, value))
        .build();
        let record = Record {
            value: "alpha".to_owned(),
        };
        let cell = Cell::new(
            interaction::Id::new("light.table"),
            list::Key::new(1),
            interaction::Id::new("value"),
        );

        let read_only = (read_only.cell)(&record, cell, Presentation::Compact);
        let editable = (editable.cell)(&record, cell, Presentation::Compact);

        assert_eq!(read_only.role(), view::Role::TextArea);
        let read_only_model = read_only
            .text_area_model()
            .expect("read-only table text remains a selectable text surface");
        assert_eq!(read_only_model.mode(), text::surface::FieldMode::ReadOnly);
        assert_eq!(
            read_only_model.focus(),
            Some(session::Focus::table_cell(cell))
        );
        assert_eq!(read_only.label_text(), Some("alpha"));
        assert_eq!(editable.role(), view::Role::TextBox);
        assert!(editable.text_box_model().is_some());
    }

    impl From<Switch> for bool {
        fn from(value: Switch) -> Self {
            value == Switch::On
        }
    }

    impl From<bool> for Switch {
        fn from(value: bool) -> Self {
            if value { Self::On } else { Self::Off }
        }
    }

    #[test]
    fn std_capabilities_supply_text_edit_sort_and_foreign_type_citizenship() {
        struct Record {
            name: String,
            address: std::net::IpAddr,
            ratio: f64,
        }
        let name = Column::text(
            "name",
            "Name",
            view::Dimension::fixed(80),
            |record: &Record| &record.name,
        )
        .editable::<CommitText>(|cell, value| (cell, value))
        .build();
        let address = Column::text(
            "address",
            "Address",
            view::Dimension::fixed(120),
            |record: &Record| &record.address,
        )
        .editable::<CommitAddress>(|cell, value| (cell, value))
        .build();
        let ratio = Column::text(
            "ratio",
            "Ratio",
            view::Dimension::fixed(80),
            |record: &Record| &record.ratio,
        )
        .align(view::Align::End)
        .input(text::Input::decimal())
        .editable::<CommitText>(|cell, value| (cell, value.to_string()))
        .unsortable()
        .build();
        let record = Record {
            name: "Ada".to_owned(),
            address: "127.0.0.1".parse().expect("loopback address"),
            ratio: 1.5,
        };
        let cell = Cell::new(
            interaction::Id::new("std.table"),
            list::Key::new(0),
            interaction::Id::new("address"),
        );
        let address_node = (address.cell)(&record, cell, Presentation::Compact);
        let ratio_node = (ratio.cell)(&record, cell, Presentation::Compact);

        assert!(name.column.ordering.is_some());
        assert!(address.column.ordering.is_some());
        assert!(
            ratio.column.ordering.is_none(),
            "f64 explicitly opts out because std supplies no Ord"
        );
        assert_eq!(address_node.label_text(), Some("127.0.0.1"));
        assert!(
            address_node
                .text_commit()
                .expect("FromStr supplies the foreign commit recipe")
                .build("not an address".to_owned())
                .is_err()
        );
        let ratio_commit = ratio_node
            .text_commit()
            .expect("float FromStr supplies a commit recipe");
        assert!(ratio_commit.build("1.25".to_owned()).is_ok());
        assert!(ratio_commit.build("1e-".to_owned()).is_err());
    }

    #[test]
    fn typed_table_commit_parses_and_validates_once() {
        use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

        static PARSES: AtomicUsize = AtomicUsize::new(0);
        static VALIDATIONS: AtomicUsize = AtomicUsize::new(0);

        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct Counted(u32);

        impl Display for Counted {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl FromStr for Counted {
            type Err = std::num::ParseIntError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                PARSES.fetch_add(1, AtomicOrdering::SeqCst);
                text.parse().map(Self)
            }
        }

        struct Record {
            value: Counted,
        }

        PARSES.store(0, AtomicOrdering::SeqCst);
        VALIDATIONS.store(0, AtomicOrdering::SeqCst);
        let column = Column::text(
            "value",
            "Value",
            view::Dimension::fixed(80),
            |record: &Record| &record.value,
        )
        .validate(|_: &Counted| {
            VALIDATIONS.fetch_add(1, AtomicOrdering::SeqCst);
            Ok::<_, &'static str>(())
        })
        .editable::<CommitText>(|cell, value| (cell, value.to_string()))
        .build();
        let record = Record { value: Counted(1) };
        let cell = Cell::new(
            interaction::Id::new("counted.table"),
            list::Key::new(0),
            interaction::Id::new("value"),
        );
        let node = (column.cell)(&record, cell, Presentation::Compact);

        assert!(
            node.text_commit()
                .expect("editable column should carry one commit recipe")
                .build("2".to_owned())
                .is_ok()
        );
        assert_eq!(PARSES.load(AtomicOrdering::SeqCst), 1);
        assert_eq!(VALIDATIONS.load(AtomicOrdering::SeqCst), 1);
    }

    #[test]
    fn boolean_medium_separates_passive_projection_from_reverse_interaction() {
        struct Record {
            switch: Switch,
            enabled: bool,
            metadata: usize,
        }
        let passive = Column::boolean(
            "passive",
            "Passive",
            view::Dimension::fixed(80),
            |record: &Record| &record.switch,
        )
        .build();
        let interactive = Column::boolean(
            "interactive",
            "Interactive",
            view::Dimension::fixed(80),
            |record: &Record| &record.switch,
        )
        .toggle::<CommitSwitch>(|cell, value| (cell, value))
        .build();
        let field_not_whole_value = Column::boolean(
            "field",
            "Field",
            view::Dimension::fixed(80),
            |record: &Record| &record.enabled,
        )
        .build();
        let record = Record {
            switch: Switch::On,
            enabled: true,
            metadata: 7,
        };
        let cell = Cell::new(
            interaction::Id::new("std.table"),
            list::Key::new(0),
            interaction::Id::new("passive"),
        );

        assert!(passive.column.ordering.is_some());
        assert!(
            (passive.cell)(&record, cell, Presentation::Compact)
                .checkbox_model()
                .is_some_and(view::Checkbox::checked)
        );
        assert!(
            (interactive.cell)(&record, cell, Presentation::Compact)
                .binding()
                .is_some()
        );
        assert!(
            (field_not_whole_value.cell)(&record, cell, Presentation::Compact)
                .checkbox_model()
                .is_some_and(view::Checkbox::checked)
        );
        assert_eq!(
            record.metadata, 7,
            "extra state is never round-tripped through bool"
        );
    }

    #[test]
    fn bounded_record_sources_apply_the_same_default_ordering_as_headers() {
        #[derive(Clone)]
        struct Record {
            key: u64,
            group: i64,
            enabled: bool,
        }
        let records: Rc<[Record]> = vec![
            Record {
                key: 30,
                group: 2,
                enabled: true,
            },
            Record {
                key: 10,
                group: 1,
                enabled: false,
            },
            Record {
                key: 20,
                group: 1,
                enabled: true,
            },
            Record {
                key: 40,
                group: 3,
                enabled: false,
            },
        ]
        .into();
        let source = Source::records(Rc::clone(&records), |record| list::Key::new(record.key));
        let columns = || {
            vec![
                Column::text(
                    "group",
                    "Group",
                    view::Dimension::fixed(80),
                    |record: &Record| &record.group,
                )
                .build(),
                Column::boolean(
                    "enabled",
                    "Enabled",
                    view::Dimension::fixed(80),
                    |record: &Record| &record.enabled,
                )
                .build(),
            ]
        };

        let ascending = Table::typed("records", 24, columns(), source.clone())
            .sorted_by("group", SortDirection::Ascending);
        assert_eq!(
            (0..4)
                .map(|row| ascending.provider.key(row).value())
                .collect::<Vec<_>>(),
            [10, 20, 30, 40],
            "equal values retain their base record order"
        );
        assert_eq!(ascending.provider.index_of(list::Key::new(30)), Some(2));

        let descending = Table::typed("records", 24, columns(), source.clone())
            .sorted_by("group", SortDirection::Descending);
        assert_eq!(
            (0..4)
                .map(|row| descending.provider.key(row).value())
                .collect::<Vec<_>>(),
            [40, 30, 10, 20],
            "descending reverses the primary order without reversing ties"
        );

        let booleans = Table::typed("records", 24, columns(), source)
            .sorted_by("enabled", SortDirection::Ascending);
        assert_eq!(
            (0..4)
                .map(|row| booleans.provider.key(row).value())
                .collect::<Vec<_>>(),
            [10, 40, 30, 20]
        );

        let empty = Source::records(Rc::<[Record]>::from([]), |record| {
            list::Key::new(record.key)
        });
        let empty = Table::typed("empty.records", 24, columns(), empty)
            .sorted_by("group", SortDirection::Ascending);
        assert!(empty.provider.is_empty());

        let replacement: Rc<[Record]> = vec![Record {
            key: 50,
            group: 0,
            enabled: true,
        }]
        .into();
        let replacement = Source::records(replacement, |record| list::Key::new(record.key));
        let replacement = Table::typed("records", 24, columns(), replacement)
            .sorted_by("group", SortDirection::Ascending);
        assert_eq!(replacement.provider.key(0).value(), 50);
        assert_eq!(replacement.provider.index_of(list::Key::new(50)), Some(0));
    }

    #[test]
    fn typed_provider_projects_one_record_for_all_cells_in_a_row() {
        let projections = Rc::new(std::cell::Cell::new(0));
        let observed = Rc::clone(&projections);
        let source = Source::new(
            2,
            |row| list::Key::new(row as u64),
            |key| Some(key.value() as usize),
            move |row| {
                observed.set(observed.get() + 1);
                row
            },
            |row| row as u64,
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
        let ordering: Rc<OrderProjection> = Rc::new(|left, right| {
            left.downcast_ref::<usize>()
                .expect("usize record")
                .cmp(right.downcast_ref::<usize>().expect("usize record"))
        });
        let provider = TypedProvider {
            source,
            cells,
            projected_record: RefCell::new(None),
            presentation: Rc::new(std::cell::Cell::new(Presentation::Compact)),
            sort: Rc::new(std::cell::Cell::new(Some(SortState::new(
                first,
                SortDirection::Descending,
            )))),
            order: RefCell::new(None),
            orderings: [(first, ordering)].into_iter().collect(),
        };
        let key = list::Key::new(0);
        assert_eq!(Provider::key(&provider, 0), key);
        assert_eq!(Provider::index_of(&provider, key), Some(0));
        assert_eq!(
            projections.get(),
            0,
            "intent-only virtual sources are never enumerated to derive ordering"
        );
        Provider::cell(&provider, 0, Cell::new(table, key, first));
        Provider::cell(&provider, 0, Cell::new(table, key, second));
        assert_eq!(projections.get(), 1);
        Provider::cell(&provider, 1, Cell::new(table, list::Key::new(1), first));
        assert_eq!(projections.get(), 2);
    }
}
