use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use crate::geometry::{area, point};
use crate::{paint, text_system};

const MEASURE_CACHE_CAPACITY: usize = 2048;
const DEFAULT_TEXT_FIELD_SIZE: f32 = 16.0;
const TEXT_FIELD_CARET_MARGIN: f32 = 5.0;
const TEXT_FIELD_CARET_BLINK_INTERVAL: Duration = Duration::from_millis(500);

pub type Cursor = glyphon::Cursor;
pub type Selection = glyphon::cosmic_text::Selection;

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    runs: Vec<Run>,
    align: Align,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    text: String,
    style: Style,
}

pub struct Engine {
    font_system: glyphon::FontSystem,
    cache: MeasureCache,
    #[cfg(test)]
    uncached_measure_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measure {
    max: Option<area::Logical>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    area: area::Logical,
    line_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextFieldLayout {
    selection_spans: Vec<SelectionSpan>,
    caret: Option<Caret>,
    scroll_x: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionSpan {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caret {
    x: f32,
    y: f32,
    height: f32,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    buffer: glyphon::Buffer,
    cursor: Cursor,
    selection: Selection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    buffer: Buffer,
    mode: FieldMode,
    obscuring: Obscuring,
    placeholder: Option<Document>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FieldMode {
    #[default]
    Editable,
    ReadOnly,
    Disabled,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Obscuring {
    #[default]
    None,
    Dot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextFieldState {
    scroll_x: f32,
    caret_epoch: Instant,
    preedit: Option<Preedit>,
    history: EditHistory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preedit {
    text: String,
    selection: Option<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    Insert(String),
    ImeCommit(String),
    Action(glyphon::Action),
    ExtendMotion(glyphon::cosmic_text::Motion),
    DeleteWordBackward,
    DeleteWordForward,
    SelectAll,
    SetCursor(Cursor),
    Pointer {
        kind: PointerEditKind,
        cursor: Cursor,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerEditKind {
    Click,
    DoubleClick,
    TripleClick,
    Drag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Copy,
    Cut,
    Paste,
    SelectAll,
    Undo,
    Redo,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CommandResult {
    pub text_changed: bool,
    pub selection_changed: bool,
    pub clipboard_changed: bool,
    pub unavailable: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TextEditResult {
    pub text_changed: bool,
    pub selection_changed: bool,
    pub change: Option<TextChange>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextCommandOutcome {
    pub result: CommandResult,
    pub change: Option<TextChange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextChange {
    before: BufferSnapshot,
    after: BufferSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HistoryKind {
    Typing,
    Boundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HistoryEntry {
    before: BufferSnapshot,
    after: BufferSnapshot,
    kind: HistoryKind,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EditHistory {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    current: Option<BufferSnapshot>,
}

struct FieldProjection {
    buffer: Buffer,
    source_boundaries: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardError {
    Unavailable,
}

pub type ClipboardResult<T> = Result<T, ClipboardError>;

pub trait Clipboard {
    fn read_text(&mut self) -> ClipboardResult<Option<String>>;

    fn write_text(&mut self, text: &str) -> ClipboardResult<()>;
}

impl CommandResult {
    pub fn buffer_changed(self) -> bool {
        self.text_changed || self.selection_changed
    }

    pub fn changed(self) -> bool {
        self.buffer_changed() || self.clipboard_changed
    }
}

impl TextEditResult {
    pub(crate) fn buffer_changed(&self) -> bool {
        self.text_changed || self.selection_changed
    }

    fn from_snapshots(before: BufferSnapshot, after: BufferSnapshot) -> Self {
        let text_changed = before.text != after.text;
        let selection_changed = before.cursor != after.cursor || before.selection != after.selection;
        let change = text_changed.then_some(TextChange { before, after });

        Self {
            text_changed,
            selection_changed,
            change,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    size: f32,
    color: paint::Color,
    weight: Weight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weight {
    Normal,
    Medium,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Debug)]
struct MeasureCache {
    entries: HashMap<MeasureKey, Metrics>,
    order: VecDeque<MeasureKey>,
    capacity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MeasureKey {
    blocks: Vec<BlockKey>,
    max: Option<BoundsKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BlockKey {
    align: Align,
    runs: Vec<RunKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RunKey {
    text: String,
    size: u32,
    weight: Weight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BoundsKey {
    width: u32,
    height: u32,
}

impl Document {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            blocks: vec![Block::plain(text)],
        }
    }

    pub fn from_block(block: Block) -> Self {
        Self {
            blocks: vec![block],
        }
    }

    pub fn push_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn first_style(&self) -> Option<Style> {
        self.blocks
            .iter()
            .flat_map(Block::runs)
            .find(|run| !run.is_empty())
            .map(Run::style)
    }

    pub fn with_color(mut self, color: paint::Color) -> Self {
        for block in &mut self.blocks {
            for run in &mut block.runs {
                run.style = run.style.with_color(color);
            }
        }

        self
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(Block::is_empty)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for Document {
    fn from(value: String) -> Self {
        Self::plain(value)
    }
}

impl From<&str> for Document {
    fn from(value: &str) -> Self {
        Self::plain(value)
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            font_system: text_system::font_system(),
            cache: MeasureCache::new(MEASURE_CACHE_CAPACITY),
            #[cfg(test)]
            uncached_measure_count: 0,
        }
    }

    pub fn measure(&mut self, document: &Document, measure: Measure) -> Metrics {
        if document.is_empty() {
            return Metrics::empty();
        }

        let key = MeasureKey::new(document, measure);
        if let Some(metrics) = self.cache.get(&key) {
            return metrics;
        }

        let metrics = self.measure_uncached(document, measure);
        self.cache.insert(key, metrics);
        metrics
    }

    fn measure_uncached(&mut self, document: &Document, measure: Measure) -> Metrics {
        #[cfg(test)]
        {
            self.uncached_measure_count += 1;
        }

        text_system::measure_document(&mut self.font_system, document, measure)
    }

    pub fn apply_text_edit(&mut self, buffer: &mut Buffer, edit: Edit) -> bool {
        self.apply_text_edit_with_result(buffer, edit).buffer_changed()
    }

    pub(crate) fn apply_text_edit_with_result(
        &mut self,
        buffer: &mut Buffer,
        edit: Edit,
    ) -> TextEditResult {
        let before = buffer.snapshot();
        buffer.prepare_for_edit(&mut self.font_system);
        let text_len = buffer.text().len();
        let edit = match edit {
            Edit::SetCursor(cursor) => Edit::SetCursor(buffer.clamp_cursor(cursor)),
            Edit::Pointer { kind, cursor } => Edit::Pointer {
                kind,
                cursor: buffer.clamp_cursor(cursor),
            },
            other => other,
        };
        let (cursor, selection) = {
            let mut editor = glyphon::Editor::new(&mut buffer.buffer);
            glyphon::Edit::set_cursor(&mut editor, buffer.cursor);
            glyphon::Edit::set_selection(&mut editor, buffer.selection);

            match edit {
                Edit::Insert(text) => {
                    let text = normalize_single_line(&text);
                    glyphon::Edit::insert_string(&mut editor, &text, None);
                }
                Edit::ImeCommit(text) => {
                    let text = normalize_single_line(&text);
                    glyphon::Edit::insert_string(&mut editor, &text, None);
                }
                Edit::Action(glyphon::Action::Enter) => {}
                Edit::Action(glyphon::Action::Insert(character)) => {
                    let text = normalize_single_line(&character.to_string());
                    glyphon::Edit::insert_string(&mut editor, &text, None);
                }
                Edit::Action(glyphon::Action::Motion(motion)) => {
                    if let Some((start, end)) = glyphon::Edit::selection_bounds(&editor) {
                        glyphon::Edit::set_cursor(
                            &mut editor,
                            collapsed_cursor_for_motion(motion, start, end),
                        );
                        glyphon::Edit::set_selection(&mut editor, Selection::None);
                    } else {
                        glyphon::Edit::action(
                            &mut editor,
                            &mut self.font_system,
                            glyphon::Action::Motion(motion),
                        );
                    }
                }
                Edit::Action(action) => {
                    glyphon::Edit::action(&mut editor, &mut self.font_system, action);
                }
                Edit::ExtendMotion(motion) => {
                    if glyphon::Edit::selection(&editor) == Selection::None {
                        let cursor = glyphon::Edit::cursor(&editor);
                        glyphon::Edit::set_selection(&mut editor, Selection::Normal(cursor));
                    }
                    glyphon::Edit::action(
                        &mut editor,
                        &mut self.font_system,
                        glyphon::Action::Motion(motion),
                    );
                }
                Edit::DeleteWordBackward => {
                    if glyphon::Edit::selection(&editor) == Selection::None {
                        let cursor = glyphon::Edit::cursor(&editor);
                        glyphon::Edit::set_selection(&mut editor, Selection::Normal(cursor));
                        glyphon::Edit::action(
                            &mut editor,
                            &mut self.font_system,
                            glyphon::Action::Motion(glyphon::cosmic_text::Motion::PreviousWord),
                        );
                    }
                    glyphon::Edit::action(
                        &mut editor,
                        &mut self.font_system,
                        glyphon::Action::Backspace,
                    );
                }
                Edit::DeleteWordForward => {
                    if glyphon::Edit::selection(&editor) == Selection::None {
                        let cursor = glyphon::Edit::cursor(&editor);
                        glyphon::Edit::set_selection(&mut editor, Selection::Normal(cursor));
                        glyphon::Edit::action(
                            &mut editor,
                            &mut self.font_system,
                            glyphon::Action::Motion(glyphon::cosmic_text::Motion::NextWord),
                        );
                    }
                    glyphon::Edit::action(
                        &mut editor,
                        &mut self.font_system,
                        glyphon::Action::Delete,
                    );
                }
                Edit::SelectAll => {
                    let end = Cursor::new(0, text_len);
                    glyphon::Edit::set_cursor(&mut editor, end);
                    glyphon::Edit::set_selection(
                        &mut editor,
                        if end.index == 0 {
                            Selection::None
                        } else {
                            Selection::Normal(Cursor::new(0, 0))
                        },
                    );
                }
                Edit::SetCursor(cursor) => {
                    glyphon::Edit::set_cursor(&mut editor, cursor);
                    glyphon::Edit::set_selection(&mut editor, Selection::None);
                }
                Edit::Pointer { kind, cursor } => match kind {
                    PointerEditKind::Click => {
                        glyphon::Edit::set_cursor(&mut editor, cursor);
                        glyphon::Edit::set_selection(&mut editor, Selection::None);
                    }
                    PointerEditKind::DoubleClick => {
                        glyphon::Edit::set_cursor(&mut editor, cursor);
                        glyphon::Edit::set_selection(&mut editor, Selection::Word(cursor));
                    }
                    PointerEditKind::TripleClick => {
                        let end = Cursor::new(0, text_len);
                        glyphon::Edit::set_cursor(&mut editor, end);
                        glyphon::Edit::set_selection(
                            &mut editor,
                            if end.index == 0 {
                                Selection::None
                            } else {
                                Selection::Normal(Cursor::new(0, 0))
                            },
                        );
                    }
                    PointerEditKind::Drag => {
                        if glyphon::Edit::selection(&editor) == Selection::None {
                            let cursor = glyphon::Edit::cursor(&editor);
                            glyphon::Edit::set_selection(&mut editor, Selection::Normal(cursor));
                        }
                        glyphon::Edit::set_cursor(&mut editor, cursor);
                    }
                },
            }

            glyphon::Edit::shape_as_needed(&mut editor, &mut self.font_system, false);
            (
                glyphon::Edit::cursor(&editor),
                glyphon::Edit::selection(&editor),
            )
        };

        buffer.cursor = buffer.clamp_cursor(cursor);
        buffer.selection = buffer.clamp_selection(selection);
        if buffer.selected_range().is_none() {
            buffer.selection = Selection::None;
        }

        let after = buffer.snapshot();

        TextEditResult::from_snapshots(before, after)
    }

    pub fn apply_text_command(
        &mut self,
        buffer: &mut Buffer,
        command: Command,
        clipboard: &mut dyn Clipboard,
    ) -> CommandResult {
        self.apply_text_command_with_result(buffer, command, clipboard)
            .result
    }

    pub(crate) fn apply_text_command_with_result(
        &mut self,
        buffer: &mut Buffer,
        command: Command,
        clipboard: &mut dyn Clipboard,
    ) -> TextCommandOutcome {
        let before = buffer.snapshot();
        let mut result = CommandResult::default();

        match command {
            Command::Copy => {
                let Some(selection) = buffer.selected_text() else {
                    return TextCommandOutcome {
                        result,
                        change: None,
                    };
                };

                match clipboard.write_text(&selection) {
                    Ok(()) => result.clipboard_changed = true,
                    Err(_) => result.unavailable = true,
                }
            }
            Command::Cut => {
                let Some(selection) = buffer.selected_text() else {
                    return TextCommandOutcome {
                        result,
                        change: None,
                    };
                };

                match clipboard.write_text(&selection) {
                    Ok(()) => {
                        result.clipboard_changed = true;
                        self.apply_text_edit(buffer, Edit::insert(""));
                    }
                    Err(_) => result.unavailable = true,
                }
            }
            Command::Paste => match clipboard.read_text() {
                Ok(Some(text)) if !normalize_single_line(&text).is_empty() => {
                    self.apply_text_edit(buffer, Edit::insert(text));
                }
                Ok(_) => {}
                Err(_) => result.unavailable = true,
            },
            Command::SelectAll => {
                self.apply_text_edit(buffer, Edit::SelectAll);
            }
            Command::Undo | Command::Redo => {
                result.unavailable = true;
            }
        }

        let after = buffer.snapshot();
        result.text_changed = before.text != after.text;
        result.selection_changed =
            before.cursor != after.cursor || before.selection != after.selection;
        let change = result
            .text_changed
            .then_some(TextChange { before, after });

        TextCommandOutcome { result, change }
    }

    pub fn text_field_layout(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> TextFieldLayout {
        self.text_field_layout_at(buffer, style, area, state, Instant::now())
    }

    pub fn text_field_layout_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> TextFieldLayout {
        self.text_field_layout_for_field_at(field, style, area, state, Instant::now())
    }

    pub fn text_field_layout_at(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
        now: Instant,
    ) -> TextFieldLayout {
        let (prepared, vertical_offset) = self.prepare_text_field_buffer(buffer, style, area);
        let scroll_x = state.scroll_x();
        let selection_spans = buffer
            .selection_bounds()
            .map(|(start, end)| {
                prepared
                    .layout_runs()
                    .filter_map(move |run| {
                        let (x, width) = run.highlight(start, end)?;

                        Some(SelectionSpan {
                            x: x - scroll_x,
                            y: vertical_offset + run.line_top,
                            width,
                            height: run.line_height,
                        })
                    })
                    .filter(|span| span.width > 0.0)
                    .collect()
            })
            .unwrap_or_default();
        let caret = (!buffer.has_selection() && state.caret_visible(now))
            .then(|| {
                cursor_position(&prepared, buffer.cursor).map(|(x, y)| Caret {
                    x: x as f32 - scroll_x,
                    y: vertical_offset + y as f32,
                    height: prepared.metrics().line_height,
                })
            })
            .flatten();

        TextFieldLayout {
            selection_spans,
            caret,
            scroll_x,
        }
    }

    pub fn text_field_layout_for_field_at(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
        now: Instant,
    ) -> TextFieldLayout {
        let projection = FieldProjection::new(field);
        let mut layout =
            self.text_field_layout_at(&projection.buffer, style, area, state.clone(), now);

        if !field.paints_caret() {
            layout.caret = None;
        }

        layout
    }

    pub fn text_field_cursor_at(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        position: point::Logical,
        state: TextFieldState,
    ) -> Option<Cursor> {
        let (prepared, vertical_offset) = self.prepare_text_field_buffer(buffer, style, area);
        prepared.hit(
            position.x() + state.scroll_x(),
            position.y() - vertical_offset,
        )
    }

    pub fn text_field_cursor_at_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        position: point::Logical,
        state: TextFieldState,
    ) -> Option<Cursor> {
        let projection = FieldProjection::new(field);
        let display_cursor =
            self.text_field_cursor_at(&projection.buffer, style, area, position, state)?;

        Some(projection.source_cursor(display_cursor))
    }

    pub fn text_field_caret(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> Option<Caret> {
        let (prepared, vertical_offset) = self.prepare_text_field_buffer(buffer, style, area);
        let scroll_x = state.scroll_x();
        let (x, y) = cursor_position(&prepared, buffer.cursor)?;

        Some(Caret {
            x: x as f32 - scroll_x,
            y: vertical_offset + y as f32,
            height: prepared.metrics().line_height,
        })
    }

    pub fn text_field_caret_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> Option<Caret> {
        if !field.paints_caret() {
            return None;
        }

        let projection = FieldProjection::new(field);
        self.text_field_caret(&projection.buffer, style, area, state)
    }

    pub fn text_field_reveal_scroll(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> TextFieldState {
        let (prepared, _) = self.prepare_text_field_buffer(buffer, style, area);
        let width = area.width().max(0.0);
        let max_scroll = (prepared
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0_f32, f32::max)
            - width)
            .max(0.0);
        let Some((caret_x, _)) = cursor_position(&prepared, buffer.cursor) else {
            let scroll_x = state.scroll_x().clamp(0.0, max_scroll);
            return state.with_scroll_x(scroll_x);
        };

        let caret_x = caret_x as f32;
        let mut scroll_x = state.scroll_x().clamp(0.0, max_scroll);
        if caret_x > scroll_x + width - TEXT_FIELD_CARET_MARGIN {
            scroll_x = caret_x + TEXT_FIELD_CARET_MARGIN - width;
        } else if caret_x < scroll_x + TEXT_FIELD_CARET_MARGIN {
            scroll_x = caret_x - TEXT_FIELD_CARET_MARGIN;
        }

        state.with_scroll_x(scroll_x.clamp(0.0, max_scroll))
    }

    pub fn text_field_reveal_scroll_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: TextFieldState,
    ) -> TextFieldState {
        let projection = FieldProjection::new(field);
        self.text_field_reveal_scroll(&projection.buffer, style, area, state)
    }

    fn prepare_text_field_buffer(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
    ) -> (glyphon::Buffer, f32) {
        let font_size = style.size().max(1.0);
        let line_height = font_size * 1.25;
        let buffer_height = area.height().max(0.0).min(line_height);
        let vertical_offset = (area.height().max(0.0) - buffer_height).max(0.0) * 0.5;
        let attrs = text_system::attrs_for_style(style);
        let mut prepared = buffer.buffer.clone();

        prepared.set_wrap(&mut self.font_system, glyphon::Wrap::None);
        prepared.set_metrics_and_size(
            &mut self.font_system,
            glyphon::Metrics::relative(font_size, 1.25),
            Some(area.width().max(0.0)),
            Some(buffer_height),
        );
        prepared.set_text(
            &mut self.font_system,
            buffer.text(),
            &attrs,
            glyphon::Shaping::Advanced,
            Some(glyphon::cosmic_text::Align::Left),
        );
        prepared.shape_until_scroll(&mut self.font_system, false);

        (prepared, vertical_offset)
    }

    #[cfg(test)]
    pub fn uncached_measure_count(&self) -> usize {
        self.uncached_measure_count
    }

    #[cfg(test)]
    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }

    #[cfg(test)]
    fn with_cache_capacity(capacity: usize) -> Self {
        Self {
            font_system: text_system::font_system(),
            cache: MeasureCache::new(capacity),
            uncached_measure_count: 0,
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Measure {
    pub fn unbounded() -> Self {
        Self { max: None }
    }

    pub fn bounded(max: area::Logical) -> Self {
        Self {
            max: Some(area::logical(max.width().max(0.0), max.height().max(0.0))),
        }
    }

    pub fn max(self) -> Option<area::Logical> {
        self.max
    }
}

impl Metrics {
    pub fn new(area: area::Logical, line_count: usize) -> Self {
        Self { area, line_count }
    }

    pub fn empty() -> Self {
        Self::new(area::logical(0.0, 0.0), 0)
    }

    pub fn area(self) -> area::Logical {
        self.area
    }

    pub fn width(self) -> f32 {
        self.area.width()
    }

    pub fn height(self) -> f32 {
        self.area.height()
    }

    pub fn line_count(self) -> usize {
        self.line_count
    }
}

impl TextFieldLayout {
    pub fn empty() -> Self {
        Self {
            selection_spans: Vec::new(),
            caret: None,
            scroll_x: 0.0,
        }
    }

    pub fn selection_spans(&self) -> &[SelectionSpan] {
        &self.selection_spans
    }

    pub fn caret(&self) -> Option<Caret> {
        self.caret
    }

    pub fn scroll_x(&self) -> f32 {
        self.scroll_x
    }
}

impl SelectionSpan {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn x(self) -> f32 {
        self.x
    }

    pub fn y(self) -> f32 {
        self.y
    }

    pub fn width(self) -> f32 {
        self.width
    }

    pub fn height(self) -> f32 {
        self.height
    }
}

impl Caret {
    pub fn new(x: f32, y: f32, height: f32) -> Self {
        Self { x, y, height }
    }

    pub fn x(self) -> f32 {
        self.x
    }

    pub fn y(self) -> f32 {
        self.y
    }

    pub fn height(self) -> f32 {
        self.height
    }
}

impl Buffer {
    pub fn new() -> Self {
        Self::from_text("")
    }

    pub fn from_text(text: impl Into<String>) -> Self {
        let text = normalize_single_line(&text.into());
        let mut buffer = glyphon::Buffer::new_empty(default_text_field_metrics());
        buffer.lines.push(glyphon::BufferLine::new(
            text.clone(),
            glyphon::cosmic_text::LineEnding::None,
            glyphon::AttrsList::new(&glyphon::Attrs::new()),
            glyphon::Shaping::Advanced,
        ));

        Self {
            buffer,
            cursor: Cursor::new(0, text.len()),
            selection: Selection::None,
        }
    }

    pub fn text(&self) -> &str {
        self.buffer
            .lines
            .first()
            .map(glyphon::BufferLine::text)
            .unwrap_or("")
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn selection_bounds(&self) -> Option<(Cursor, Cursor)> {
        selection_bounds(&self.buffer, self.cursor, self.selection)
    }

    pub fn selected_range(&self) -> Option<std::ops::Range<usize>> {
        let (start, end) = self.selection_bounds()?;

        if start.line != 0 || end.line != 0 {
            return None;
        }

        (start.index < end.index).then_some(start.index..end.index)
    }

    pub fn selected_text(&self) -> Option<String> {
        let range = self.selected_range()?;

        Some(self.text()[range].to_owned())
    }

    pub fn has_selection(&self) -> bool {
        self.selected_range().is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.text().is_empty()
    }

    fn prepare_for_edit(&mut self, font_system: &mut glyphon::FontSystem) {
        self.buffer.set_wrap(font_system, glyphon::Wrap::None);
        self.buffer.shape_until_scroll(font_system, false);
        self.cursor = self.clamp_cursor(self.cursor);
        self.selection = self.clamp_selection(self.selection);
    }

    fn clamp_cursor(&self, cursor: Cursor) -> Cursor {
        Cursor::new(0, floor_boundary(self.text(), cursor.index))
    }

    fn clamp_selection(&self, selection: Selection) -> Selection {
        match selection {
            Selection::None => Selection::None,
            Selection::Normal(cursor) => Selection::Normal(self.clamp_cursor(cursor)),
            Selection::Line(cursor) => Selection::Line(self.clamp_cursor(cursor)),
            Selection::Word(cursor) => Selection::Word(self.clamp_cursor(cursor)),
        }
    }

    fn snapshot(&self) -> BufferSnapshot {
        BufferSnapshot {
            text: self.text().to_owned(),
            cursor: self.cursor,
            selection: self.selection,
        }
    }

    fn restore_snapshot(&mut self, snapshot: &BufferSnapshot) {
        *self = Self::from_text(snapshot.text.clone());
        self.cursor = self.clamp_cursor(snapshot.cursor);
        self.selection = self.clamp_selection(snapshot.selection);

        if self.selected_range().is_none() {
            self.selection = Selection::None;
        }
    }
}

impl Field {
    pub fn new(buffer: impl Into<Buffer>) -> Self {
        Self {
            buffer: buffer.into(),
            mode: FieldMode::Editable,
            obscuring: Obscuring::None,
            placeholder: None,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn mode(&self) -> FieldMode {
        self.mode
    }

    pub fn obscuring(&self) -> Obscuring {
        self.obscuring
    }

    pub fn placeholder(&self) -> Option<&Document> {
        self.placeholder.as_ref()
    }

    pub fn with_mode(mut self, mode: FieldMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn read_only(self) -> Self {
        self.with_mode(FieldMode::ReadOnly)
    }

    pub fn disabled(self) -> Self {
        self.with_mode(FieldMode::Disabled)
    }

    pub fn with_obscuring(mut self, obscuring: Obscuring) -> Self {
        self.obscuring = obscuring;
        self
    }

    pub fn obscured_dot(self) -> Self {
        self.with_obscuring(Obscuring::Dot)
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<Document>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn is_editable(&self) -> bool {
        self.mode == FieldMode::Editable
    }

    pub fn is_read_only(&self) -> bool {
        self.mode == FieldMode::ReadOnly
    }

    pub fn is_disabled(&self) -> bool {
        self.mode == FieldMode::Disabled
    }

    pub fn is_selectable(&self) -> bool {
        !self.is_disabled()
    }

    pub fn accepts_text_input(&self) -> bool {
        self.is_editable()
    }

    pub fn paints_caret(&self) -> bool {
        self.is_editable()
    }

    pub fn allows_text_mutation(&self) -> bool {
        self.is_editable()
    }

    pub fn allows_copy(&self) -> bool {
        self.is_selectable() && self.obscuring == Obscuring::None
    }

    pub fn allows_cut(&self) -> bool {
        self.is_editable() && self.obscuring == Obscuring::None
    }

    pub fn presentation_text(&self) -> String {
        match self.obscuring {
            Obscuring::None => self.buffer.text().to_owned(),
            Obscuring::Dot => obscured_dot_text(self.buffer.text()),
        }
    }
}

impl From<Buffer> for Field {
    fn from(value: Buffer) -> Self {
        Self::new(value)
    }
}

impl From<String> for Field {
    fn from(value: String) -> Self {
        Self::new(Buffer::from(value))
    }
}

impl From<&str> for Field {
    fn from(value: &str) -> Self {
        Self::new(Buffer::from(value))
    }
}

impl TextFieldState {
    pub fn new(scroll_x: f32) -> Self {
        Self::new_at(scroll_x, Instant::now())
    }

    pub fn new_at(scroll_x: f32, caret_epoch: Instant) -> Self {
        Self {
            scroll_x: scroll_x.max(0.0),
            caret_epoch,
            preedit: None,
            history: EditHistory::default(),
        }
    }

    pub fn scroll_x(&self) -> f32 {
        self.scroll_x
    }

    pub fn with_scroll_x(mut self, scroll_x: f32) -> Self {
        self.scroll_x = scroll_x.max(0.0);
        self
    }

    pub fn reset_caret_blink(mut self, now: Instant) -> Self {
        self.caret_epoch = now;
        self
    }

    pub fn with_preedit(mut self, preedit: Option<Preedit>) -> Self {
        self.preedit = preedit;
        self
    }

    pub fn preedit(&self) -> Option<&Preedit> {
        self.preedit.as_ref()
    }

    pub(crate) fn sync_history(&mut self, buffer: &Buffer) -> bool {
        self.history.sync(buffer.snapshot())
    }

    pub(crate) fn record_history(&mut self, change: TextChange, kind: HistoryKind) {
        self.history.record(change, kind);
    }

    pub(crate) fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub(crate) fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub(crate) fn apply_undo(&mut self, buffer: &mut Buffer) -> CommandResult {
        self.history.undo(buffer)
    }

    pub(crate) fn apply_redo(&mut self, buffer: &mut Buffer) -> CommandResult {
        self.history.redo(buffer)
    }

    pub fn caret_visible(&self, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.caret_epoch);
        let interval = TEXT_FIELD_CARET_BLINK_INTERVAL.as_millis();

        if interval == 0 {
            return true;
        }

        (elapsed.as_millis() / interval) % 2 == 0
    }

    pub fn next_caret_deadline(&self, now: Instant) -> Instant {
        let elapsed = now.saturating_duration_since(self.caret_epoch);
        let interval_ms = TEXT_FIELD_CARET_BLINK_INTERVAL.as_millis();
        let remainder = elapsed.as_millis() % interval_ms;
        let wait_ms = if remainder == 0 {
            interval_ms
        } else {
            interval_ms - remainder
        };

        now.checked_add(Duration::from_millis(wait_ms.min(u64::MAX as u128) as u64))
            .unwrap_or(now)
    }
}

impl FieldProjection {
    fn new(field: &Field) -> Self {
        match field.obscuring {
            Obscuring::None => Self {
                buffer: field.buffer.clone(),
                source_boundaries: None,
            },
            Obscuring::Dot => {
                let source_boundaries = source_char_boundaries(field.buffer.text());
                let mut buffer = Buffer::from_text(obscured_dot_text(field.buffer.text()));

                if let Some((start, end)) = field.buffer.selection_bounds() {
                    buffer.cursor = Self::display_cursor(&source_boundaries, end);
                    buffer.selection = Selection::Normal(Self::display_cursor(
                        &source_boundaries,
                        start,
                    ));
                } else {
                    buffer.cursor = Self::display_cursor(&source_boundaries, field.buffer.cursor);
                    buffer.selection = Selection::None;
                }

                Self {
                    buffer,
                    source_boundaries: Some(source_boundaries),
                }
            }
        }
    }

    fn source_cursor(&self, cursor: Cursor) -> Cursor {
        let Some(source_boundaries) = self.source_boundaries.as_ref() else {
            return cursor;
        };

        Cursor::new(0, self.source_index(source_boundaries, cursor.index))
    }

    fn display_cursor(source_boundaries: &[usize], cursor: Cursor) -> Cursor {
        Cursor::new(0, display_index(source_boundaries, cursor.index))
    }

    fn source_index(&self, source_boundaries: &[usize], display_index: usize) -> usize {
        let display_index = floor_boundary(self.buffer.text(), display_index);
        let character = self.buffer.text()[..display_index].chars().count();

        source_boundaries
            .get(character.min(source_boundaries.len().saturating_sub(1)))
            .copied()
            .unwrap_or(0)
    }
}

impl Preedit {
    pub fn new(text: impl Into<String>, selection: Option<(usize, usize)>) -> Self {
        Self {
            text: text.into(),
            selection,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }
}

fn obscured_dot_text(text: &str) -> String {
    "•".repeat(text.chars().count())
}

fn source_char_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries = vec![0];

    for (index, _) in text.char_indices().skip(1) {
        boundaries.push(index);
    }

    boundaries.push(text.len());
    boundaries
}

fn display_index(source_boundaries: &[usize], source_index: usize) -> usize {
    let source_index = floor_boundary_for_boundaries(source_boundaries, source_index);
    let character = source_boundaries
        .partition_point(|boundary| *boundary <= source_index)
        .saturating_sub(1);

    "•".len() * character
}

fn floor_boundary_for_boundaries(boundaries: &[usize], index: usize) -> usize {
    boundaries
        .iter()
        .copied()
        .take_while(|boundary| *boundary <= index)
        .last()
        .unwrap_or(0)
}

impl Default for TextFieldState {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl EditHistory {
    fn sync(&mut self, snapshot: BufferSnapshot) -> bool {
        if self.current.as_ref() == Some(&snapshot) {
            return false;
        }

        if self
            .current
            .as_ref()
            .is_some_and(|current| current.text == snapshot.text)
        {
            self.current = Some(snapshot);
            return false;
        }

        let changed = self.current.is_some() || !self.undo.is_empty() || !self.redo.is_empty();
        self.undo.clear();
        self.redo.clear();
        self.current = Some(snapshot);
        changed
    }

    fn record(&mut self, change: TextChange, kind: HistoryKind) {
        if change.before == change.after {
            self.current = Some(change.after);
            return;
        }

        if self.current.as_ref().is_some_and(|current| current != &change.before) {
            self.undo.clear();
            self.redo.clear();
        }

        if kind == HistoryKind::Typing
            && let Some(last) = self.undo.last_mut()
            && last.kind == HistoryKind::Typing
            && last.after == change.before
        {
            last.after = change.after.clone();
            self.redo.clear();
            self.current = Some(change.after);
            return;
        }

        self.undo.push(HistoryEntry {
            before: change.before,
            after: change.after.clone(),
            kind,
        });
        self.redo.clear();
        self.current = Some(change.after);
    }

    fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn undo(&mut self, buffer: &mut Buffer) -> CommandResult {
        let Some(entry) = self.undo.pop() else {
            return CommandResult {
                unavailable: true,
                ..CommandResult::default()
            };
        };

        let before = buffer.snapshot();
        buffer.restore_snapshot(&entry.before);
        let after = buffer.snapshot();
        self.current = Some(after.clone());
        self.redo.push(entry);
        command_result_from_snapshots(before, after)
    }

    fn redo(&mut self, buffer: &mut Buffer) -> CommandResult {
        let Some(entry) = self.redo.pop() else {
            return CommandResult {
                unavailable: true,
                ..CommandResult::default()
            };
        };

        let before = buffer.snapshot();
        buffer.restore_snapshot(&entry.after);
        let after = buffer.snapshot();
        self.current = Some(after.clone());
        self.undo.push(entry);
        command_result_from_snapshots(before, after)
    }
}

fn command_result_from_snapshots(before: BufferSnapshot, after: BufferSnapshot) -> CommandResult {
    CommandResult {
        text_changed: before.text != after.text,
        selection_changed: before.cursor != after.cursor || before.selection != after.selection,
        clipboard_changed: false,
        unavailable: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BufferSnapshot {
    text: String,
    cursor: Cursor,
    selection: Selection,
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot() == other.snapshot()
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for Buffer {
    fn from(value: String) -> Self {
        Self::from_text(value)
    }
}

impl From<&str> for Buffer {
    fn from(value: &str) -> Self {
        Self::from_text(value)
    }
}

impl Edit {
    pub fn insert(text: impl Into<String>) -> Self {
        Self::Insert(text.into())
    }

    pub fn ime_commit(text: impl Into<String>) -> Self {
        Self::ImeCommit(text.into())
    }

    pub fn action(action: glyphon::Action) -> Self {
        Self::Action(action)
    }

    pub fn motion(motion: glyphon::cosmic_text::Motion) -> Self {
        Self::Action(glyphon::Action::Motion(motion))
    }

    pub fn extend_motion(motion: glyphon::cosmic_text::Motion) -> Self {
        Self::ExtendMotion(motion)
    }

    pub fn delete_word_backward() -> Self {
        Self::DeleteWordBackward
    }

    pub fn delete_word_forward() -> Self {
        Self::DeleteWordForward
    }

    pub fn set_cursor(cursor: Cursor) -> Self {
        Self::SetCursor(cursor)
    }

    pub fn pointer(kind: PointerEditKind, cursor: Cursor) -> Self {
        Self::Pointer { kind, cursor }
    }

    pub(crate) fn history_kind(&self) -> HistoryKind {
        match self {
            Self::Insert(text) if text.chars().count() == 1 => HistoryKind::Typing,
            Self::Insert(_)
            | Self::ImeCommit(_)
            | Self::Action(glyphon::Action::Backspace | glyphon::Action::Delete)
            | Self::Action(glyphon::Action::Insert(_))
            | Self::DeleteWordBackward
            | Self::DeleteWordForward => HistoryKind::Boundary,
            Self::Action(_)
            | Self::ExtendMotion(_)
            | Self::SelectAll
            | Self::SetCursor(_)
            | Self::Pointer { .. } => HistoryKind::Boundary,
        }
    }

    pub(crate) fn mutates_text(&self) -> bool {
        matches!(
            self,
            Self::Insert(_)
                | Self::ImeCommit(_)
                | Self::Action(
                    glyphon::Action::Backspace
                        | glyphon::Action::Delete
                        | glyphon::Action::Insert(_)
                )
                | Self::DeleteWordBackward
                | Self::DeleteWordForward
        )
    }
}

impl Block {
    pub fn new(align: Align) -> Self {
        Self {
            runs: Vec::new(),
            align,
        }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            runs: vec![Run::new(text, Style::default())],
            align: Align::Start,
        }
    }

    pub fn push_run(&mut self, run: Run) {
        self.runs.push(run);
    }

    pub fn runs(&self) -> &[Run] {
        &self.runs
    }

    pub fn align(&self) -> Align {
        self.align
    }

    pub fn set_align(&mut self, align: Align) {
        self.align = align;
    }

    pub fn with_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.runs.iter().all(Run::is_empty)
    }
}

impl Run {
    pub fn new(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

impl Style {
    pub fn size(self) -> f32 {
        self.size
    }

    pub fn color(self) -> paint::Color {
        self.color
    }

    pub fn weight(self) -> Weight {
        self.weight
    }

    pub fn with_color(mut self, color: paint::Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn with_weight(mut self, weight: Weight) -> Self {
        self.weight = weight;
        self
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            size: 16.0,
            color: paint::Color::rgb(0.92, 0.94, 0.98),
            weight: Weight::Normal,
        }
    }
}

impl MeasureCache {
    fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            capacity,
        }
    }

    fn get(&self, key: &MeasureKey) -> Option<Metrics> {
        self.entries.get(key).copied()
    }

    fn insert(&mut self, key: MeasureKey, metrics: Metrics) {
        if self.capacity == 0 {
            return;
        }

        if let Some(entry) = self.entries.get_mut(&key) {
            *entry = metrics;
            return;
        }

        while self.entries.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            } else {
                break;
            }
        }

        self.order.push_back(key.clone());
        self.entries.insert(key, metrics);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl MeasureKey {
    fn new(document: &Document, measure: Measure) -> Self {
        Self {
            blocks: document
                .blocks()
                .iter()
                .filter(|block| !block.is_empty())
                .map(BlockKey::new)
                .collect(),
            max: measure.max().map(BoundsKey::new),
        }
    }
}

impl BlockKey {
    fn new(block: &Block) -> Self {
        Self {
            align: block.align(),
            runs: block.runs().iter().map(RunKey::new).collect(),
        }
    }
}

impl RunKey {
    fn new(run: &Run) -> Self {
        let style = run.style();

        Self {
            text: run.text().to_owned(),
            size: finite_bits(style.size().max(1.0)),
            weight: style.weight(),
        }
    }
}

impl BoundsKey {
    fn new(bounds: area::Logical) -> Self {
        Self {
            width: finite_bits(bounds.width().max(0.0)),
            height: finite_bits(bounds.height().max(0.0)),
        }
    }
}

fn finite_bits(value: f32) -> u32 {
    if value.is_finite() {
        value.to_bits()
    } else if value.is_sign_negative() {
        0.0_f32.to_bits()
    } else {
        f32::INFINITY.to_bits()
    }
}

fn default_text_field_metrics() -> glyphon::Metrics {
    glyphon::Metrics::relative(DEFAULT_TEXT_FIELD_SIZE, 1.25)
}

fn normalize_single_line(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '\r' | '\n' => ' ',
            _ => character,
        })
        .collect()
}

fn selection_bounds(
    buffer: &glyphon::Buffer,
    cursor: Cursor,
    selection: Selection,
) -> Option<(Cursor, Cursor)> {
    let mut buffer = buffer.clone();
    let text = buffer
        .lines
        .first()
        .map(glyphon::BufferLine::text)
        .unwrap_or("")
        .to_owned();
    let mut editor = glyphon::Editor::new(&mut buffer);
    glyphon::Edit::set_cursor(
        &mut editor,
        Cursor::new(0, floor_boundary(&text, cursor.index)),
    );
    glyphon::Edit::set_selection(
        &mut editor,
        match selection {
            Selection::None => Selection::None,
            Selection::Normal(cursor) => {
                Selection::Normal(Cursor::new(0, floor_boundary(&text, cursor.index)))
            }
            Selection::Line(cursor) => {
                Selection::Line(Cursor::new(0, floor_boundary(&text, cursor.index)))
            }
            Selection::Word(cursor) => {
                Selection::Word(Cursor::new(0, floor_boundary(&text, cursor.index)))
            }
        },
    );

    glyphon::Edit::selection_bounds(&editor)
        .filter(|(start, end)| start.line == 0 && end.line == 0 && start.index < end.index)
}

fn collapsed_cursor_for_motion(
    motion: glyphon::cosmic_text::Motion,
    start: Cursor,
    end: Cursor,
) -> Cursor {
    match motion {
        glyphon::cosmic_text::Motion::Left
        | glyphon::cosmic_text::Motion::Previous
        | glyphon::cosmic_text::Motion::LeftWord
        | glyphon::cosmic_text::Motion::PreviousWord
        | glyphon::cosmic_text::Motion::Home
        | glyphon::cosmic_text::Motion::SoftHome
        | glyphon::cosmic_text::Motion::ParagraphStart
        | glyphon::cosmic_text::Motion::BufferStart => start,
        glyphon::cosmic_text::Motion::Right
        | glyphon::cosmic_text::Motion::Next
        | glyphon::cosmic_text::Motion::RightWord
        | glyphon::cosmic_text::Motion::NextWord
        | glyphon::cosmic_text::Motion::End
        | glyphon::cosmic_text::Motion::ParagraphEnd
        | glyphon::cosmic_text::Motion::BufferEnd => end,
        _ => end,
    }
}

fn cursor_position(buffer: &glyphon::Buffer, cursor: Cursor) -> Option<(i32, i32)> {
    let mut buffer = buffer.clone();
    let mut editor = glyphon::Editor::new(&mut buffer);
    glyphon::Edit::set_cursor(&mut editor, cursor);

    glyphon::Edit::cursor_position(&editor)
}

fn floor_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }

    index
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[derive(Debug, Default)]
    struct MockClipboard {
        text: Option<String>,
        unavailable: bool,
    }

    impl MockClipboard {
        fn with_text(text: &str) -> Self {
            Self {
                text: Some(text.to_owned()),
                unavailable: false,
            }
        }

        fn unavailable() -> Self {
            Self {
                text: None,
                unavailable: true,
            }
        }
    }

    impl Clipboard for MockClipboard {
        fn read_text(&mut self) -> ClipboardResult<Option<String>> {
            if self.unavailable {
                Err(ClipboardError::Unavailable)
            } else {
                Ok(self.text.clone())
            }
        }

        fn write_text(&mut self, text: &str) -> ClipboardResult<()> {
            if self.unavailable {
                Err(ClipboardError::Unavailable)
            } else {
                self.text = Some(text.to_owned());
                Ok(())
            }
        }
    }

    fn record_edit(
        engine: &mut Engine,
        state: &mut TextFieldState,
        buffer: &mut Buffer,
        edit: Edit,
    ) -> TextEditResult {
        state.sync_history(buffer);
        let kind = edit.history_kind();
        let result = engine.apply_text_edit_with_result(buffer, edit);
        if let Some(change) = result.change.clone() {
            state.record_history(change, kind);
        }
        result
    }

    fn record_command(
        engine: &mut Engine,
        state: &mut TextFieldState,
        buffer: &mut Buffer,
        command: Command,
        clipboard: &mut dyn Clipboard,
    ) -> CommandResult {
        state.sync_history(buffer);
        let outcome = engine.apply_text_command_with_result(buffer, command, clipboard);
        if let Some(change) = outcome.change.clone() {
            state.record_history(change, HistoryKind::Boundary);
        }
        outcome.result
    }

    #[test]
    fn document_stores_block_run_and_style_data() {
        let style = Style::default()
            .with_size(18.0)
            .with_color(paint::Color::RED)
            .with_weight(Weight::Bold);
        let mut block = Block::new(Align::Center);
        block.push_run(Run::new("Label", style));
        let document = Document::from_block(block);

        assert_eq!(document.blocks().len(), 1);
        assert_eq!(document.blocks()[0].align(), Align::Center);
        assert_eq!(document.blocks()[0].runs()[0].text(), "Label");
        assert_eq!(document.blocks()[0].runs()[0].style(), style);
    }

    #[test]
    fn empty_document_is_empty() {
        assert!(Document::new().is_empty());
        assert!(Document::plain("").is_empty());
        assert!(!Document::plain("x").is_empty());
    }

    #[test]
    fn document_color_can_be_overridden() {
        let document = Document::plain("Label").with_color(paint::Color::BLACK);

        assert_eq!(
            document.blocks()[0].runs()[0].style().color(),
            paint::Color::BLACK
        );
    }

    #[test]
    fn engine_returns_non_zero_metrics_for_non_empty_text() {
        let mut engine = Engine::new();
        let metrics = engine.measure(&Document::plain("Label"), Measure::unbounded());

        assert!(metrics.width() > 0.0);
        assert!(metrics.height() > 0.0);
        assert_eq!(metrics.line_count(), 1);
    }

    #[test]
    fn longer_text_measures_wider_than_shorter_text() {
        let mut engine = Engine::new();
        let short = engine.measure(&Document::plain("Run"), Measure::unbounded());
        let long = engine.measure(&Document::plain("Run workspace task"), Measure::unbounded());

        assert!(long.width() > short.width());
        assert!(long.height() >= short.height());
    }

    #[test]
    fn larger_font_measures_taller_than_smaller_font() {
        let mut engine = Engine::new();
        let small = Document::from_block({
            let mut block = Block::new(Align::Start);
            block.push_run(Run::new("Label", Style::default().with_size(10.0)));
            block
        });
        let large = Document::from_block({
            let mut block = Block::new(Align::Start);
            block.push_run(Run::new("Label", Style::default().with_size(24.0)));
            block
        });

        let small = engine.measure(&small, Measure::unbounded());
        let large = engine.measure(&large, Measure::unbounded());

        assert!(large.height() > small.height());
    }

    #[test]
    fn repeated_measurement_reuses_cached_metrics() {
        let mut engine = Engine::new();
        let document = Document::plain("Cached Label");

        let first = engine.measure(&document, Measure::unbounded());
        let second = engine.measure(&document, Measure::unbounded());

        assert_eq!(first, second);
        assert_eq!(engine.uncached_measure_count(), 1);
        assert_eq!(engine.cache_len(), 1);
    }

    #[test]
    fn color_only_changes_reuse_cached_metrics() {
        let mut engine = Engine::new();
        let red = Document::plain("Cached Label").with_color(paint::Color::RED);
        let black = Document::plain("Cached Label").with_color(paint::Color::BLACK);

        let red = engine.measure(&red, Measure::unbounded());
        let black = engine.measure(&black, Measure::unbounded());

        assert_eq!(red, black);
        assert_eq!(engine.uncached_measure_count(), 1);
    }

    #[test]
    fn shaping_relevant_document_and_bounds_changes_use_distinct_cache_keys() {
        let mut engine = Engine::new();
        let base = styled_document("Cached Label", Align::Start, 16.0, Weight::Normal);
        let text = styled_document("Different Label", Align::Start, 16.0, Weight::Normal);
        let size = styled_document("Cached Label", Align::Start, 20.0, Weight::Normal);
        let weight = styled_document("Cached Label", Align::Start, 16.0, Weight::Bold);
        let align = styled_document("Cached Label", Align::End, 16.0, Weight::Normal);

        engine.measure(&base, Measure::unbounded());
        engine.measure(&text, Measure::unbounded());
        engine.measure(&size, Measure::unbounded());
        engine.measure(&weight, Measure::unbounded());
        engine.measure(&align, Measure::unbounded());
        engine.measure(&base, Measure::bounded(area::logical(40.0, 100.0)));

        assert_eq!(engine.uncached_measure_count(), 6);
        assert_eq!(engine.cache_len(), 6);
    }

    #[test]
    fn bounded_fifo_cache_evicts_oldest_entries() {
        let mut engine = Engine::with_cache_capacity(2);
        let first = Document::plain("First");
        let second = Document::plain("Second");
        let third = Document::plain("Third");

        engine.measure(&first, Measure::unbounded());
        engine.measure(&second, Measure::unbounded());
        engine.measure(&third, Measure::unbounded());
        engine.measure(&first, Measure::unbounded());

        assert_eq!(engine.cache_len(), 2);
        assert_eq!(engine.uncached_measure_count(), 4);
    }

    #[test]
    fn buffer_inserts_and_deletes_text() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("ab");

        engine.apply_text_edit(&mut buffer, Edit::insert("c"));
        engine.apply_text_edit(
            &mut buffer,
            Edit::motion(glyphon::cosmic_text::Motion::Left),
        );
        engine.apply_text_edit(&mut buffer, Edit::action(glyphon::Action::Backspace));

        assert_eq!(buffer.text(), "ac");
        assert_eq!(buffer.cursor().index, 1);

        engine.apply_text_edit(&mut buffer, Edit::action(glyphon::Action::Delete));

        assert_eq!(buffer.text(), "a");
        assert_eq!(buffer.cursor().index, 1);
    }

    #[test]
    fn buffer_select_all_replaces_selection() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        assert_eq!(buffer.selected_range(), Some(0..5));

        engine.apply_text_edit(&mut buffer, Edit::insert("hi"));

        assert_eq!(buffer.text(), "hi");
        assert_eq!(buffer.cursor().index, 2);
        assert_eq!(buffer.selected_range(), None);
    }

    #[test]
    fn text_command_copy_writes_selection_without_mutating_buffer() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");
        let mut clipboard = MockClipboard::default();

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let result = engine.apply_text_command(&mut buffer, Command::Copy, &mut clipboard);

        assert_eq!(clipboard.text.as_deref(), Some("hello"));
        assert_eq!(buffer.text(), "hello");
        assert_eq!(buffer.selected_range(), Some(0..5));
        assert!(result.clipboard_changed);
        assert!(!result.buffer_changed());
        assert!(!result.unavailable);
    }

    #[test]
    fn text_command_cut_copies_and_deletes_selection() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");
        let mut clipboard = MockClipboard::default();

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let result = engine.apply_text_command(&mut buffer, Command::Cut, &mut clipboard);

        assert_eq!(clipboard.text.as_deref(), Some("hello"));
        assert_eq!(buffer.text(), "");
        assert_eq!(buffer.selected_range(), None);
        assert!(result.clipboard_changed);
        assert!(result.text_changed);
        assert!(result.selection_changed);
        assert!(!result.unavailable);
    }

    #[test]
    fn text_command_paste_replaces_selection_and_normalizes_line_endings() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");
        let mut clipboard = MockClipboard::with_text("a\nb\rc");

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let result = engine.apply_text_command(&mut buffer, Command::Paste, &mut clipboard);

        assert_eq!(buffer.text(), "a b c");
        assert_eq!(buffer.selected_range(), None);
        assert!(result.text_changed);
        assert!(result.selection_changed);
        assert!(!result.clipboard_changed);
        assert!(!result.unavailable);
    }

    #[test]
    fn text_command_paste_without_text_or_clipboard_does_not_mutate() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");
        let mut empty_clipboard = MockClipboard::default();

        let empty = engine.apply_text_command(&mut buffer, Command::Paste, &mut empty_clipboard);

        assert_eq!(buffer.text(), "hello");
        assert!(!empty.changed());
        assert!(!empty.unavailable);

        let mut unavailable_clipboard = MockClipboard::unavailable();
        let unavailable =
            engine.apply_text_command(&mut buffer, Command::Paste, &mut unavailable_clipboard);

        assert_eq!(buffer.text(), "hello");
        assert!(!unavailable.changed());
        assert!(unavailable.unavailable);
    }

    #[test]
    fn text_history_coalesces_typing_into_one_undo_step() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::new();

        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("a"));
        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("b"));
        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("c"));

        assert_eq!(buffer.text(), "abc");
        assert_eq!(state.history.undo.len(), 1);
        assert!(state.can_undo());

        let undo = state.apply_undo(&mut buffer);
        assert_eq!(buffer.text(), "");
        assert!(undo.text_changed);
        assert!(state.can_redo());

        let redo = state.apply_redo(&mut buffer);
        assert_eq!(buffer.text(), "abc");
        assert!(redo.text_changed);
    }

    #[test]
    fn text_history_keeps_paste_cut_delete_word_delete_and_ime_as_separate_steps() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::from_text("hello");
        let mut clipboard = MockClipboard::with_text(" pasted");

        record_command(
            &mut engine,
            &mut state,
            &mut buffer,
            Command::Paste,
            &mut clipboard,
        );
        record_edit(
            &mut engine,
            &mut state,
            &mut buffer,
            Edit::action(glyphon::Action::Backspace),
        );
        record_edit(
            &mut engine,
            &mut state,
            &mut buffer,
            Edit::delete_word_backward(),
        );
        record_edit(&mut engine, &mut state, &mut buffer, Edit::ime_commit("x"));

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        let mut clipboard = MockClipboard::default();
        record_command(
            &mut engine,
            &mut state,
            &mut buffer,
            Command::Cut,
            &mut clipboard,
        );

        assert_eq!(state.history.undo.len(), 5);
    }

    #[test]
    fn text_history_undo_restores_text_cursor_and_selection() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::from_text("hello");

        state.sync_history(&buffer);
        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("x"));

        assert_eq!(buffer.text(), "x");
        assert!(!buffer.has_selection());

        let undo = state.apply_undo(&mut buffer);
        assert_eq!(buffer.text(), "hello");
        assert_eq!(buffer.selected_text().as_deref(), Some("hello"));
        assert!(undo.text_changed);
        assert!(undo.selection_changed);

        let redo = state.apply_redo(&mut buffer);
        assert_eq!(buffer.text(), "x");
        assert!(!buffer.has_selection());
        assert!(redo.text_changed);
    }

    #[test]
    fn text_history_new_edit_after_undo_clears_redo() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::new();

        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("a"));
        state.apply_undo(&mut buffer);
        assert!(state.can_redo());

        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("b"));

        assert_eq!(buffer.text(), "b");
        assert!(!state.can_redo());
        assert!(state.can_undo());
    }

    #[test]
    fn text_history_external_buffer_replacement_clears_stale_history() {
        let mut engine = Engine::new();
        let mut state = TextFieldState::default();
        let mut buffer = Buffer::new();

        record_edit(&mut engine, &mut state, &mut buffer, Edit::insert("a"));
        assert!(state.can_undo());

        let external = Buffer::from_text("external");
        assert!(state.sync_history(&external));
        assert!(!state.can_undo());
        assert!(!state.can_redo());
    }

    #[test]
    fn buffer_shift_motion_extends_selection() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");

        engine.apply_text_edit(
            &mut buffer,
            Edit::extend_motion(glyphon::cosmic_text::Motion::Left),
        );

        assert_eq!(buffer.cursor().index, 4);
        assert_eq!(buffer.selected_range(), Some(4..5));

        engine.apply_text_edit(
            &mut buffer,
            Edit::extend_motion(glyphon::cosmic_text::Motion::Home),
        );

        assert_eq!(buffer.cursor().index, 0);
        assert_eq!(buffer.selected_range(), Some(0..5));
    }

    #[test]
    fn buffer_plain_motion_collapses_selection() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        engine.apply_text_edit(
            &mut buffer,
            Edit::motion(glyphon::cosmic_text::Motion::Left),
        );

        assert_eq!(buffer.cursor().index, 0);
        assert_eq!(buffer.selected_range(), None);

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);
        engine.apply_text_edit(
            &mut buffer,
            Edit::motion(glyphon::cosmic_text::Motion::Right),
        );

        assert_eq!(buffer.cursor().index, 5);
        assert_eq!(buffer.selected_range(), None);
    }

    #[test]
    fn buffer_word_delete_uses_cosmic_word_motion() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello world again");

        engine.apply_text_edit(&mut buffer, Edit::delete_word_backward());

        assert_eq!(buffer.text(), "hello world ");
        assert_eq!(buffer.cursor().index, "hello world ".len());

        engine.apply_text_edit(&mut buffer, Edit::set_cursor(Cursor::new(0, 0)));
        engine.apply_text_edit(&mut buffer, Edit::delete_word_forward());

        assert_eq!(buffer.text(), " world ");
        assert_eq!(buffer.cursor().index, 0);
    }

    #[test]
    fn buffer_pointer_double_click_selects_word_and_triple_click_selects_all() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello world");

        engine.apply_text_edit(
            &mut buffer,
            Edit::pointer(PointerEditKind::DoubleClick, Cursor::new(0, 1)),
        );

        assert_eq!(buffer.selected_range(), Some(0..5));

        engine.apply_text_edit(
            &mut buffer,
            Edit::pointer(PointerEditKind::TripleClick, Cursor::new(0, 7)),
        );

        assert_eq!(buffer.selected_range(), Some(0.."hello world".len()));
    }

    #[test]
    fn buffer_pointer_drag_extends_from_click_anchor() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello world");

        engine.apply_text_edit(
            &mut buffer,
            Edit::pointer(PointerEditKind::Click, Cursor::new(0, 0)),
        );
        engine.apply_text_edit(
            &mut buffer,
            Edit::pointer(PointerEditKind::Drag, Cursor::new(0, 5)),
        );

        assert_eq!(buffer.selected_range(), Some(0..5));
    }

    #[test]
    fn buffer_edits_preserve_unicode_boundaries() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("aé🙂");

        engine.apply_text_edit(&mut buffer, Edit::set_cursor(Cursor::new(0, 3)));
        assert_eq!(buffer.cursor().index, "aé".len());

        engine.apply_text_edit(&mut buffer, Edit::action(glyphon::Action::Backspace));
        assert_eq!(buffer.text(), "a🙂");

        engine.apply_text_edit(&mut buffer, Edit::motion(glyphon::cosmic_text::Motion::End));
        engine.apply_text_edit(&mut buffer, Edit::action(glyphon::Action::Backspace));
        assert_eq!(buffer.text(), "a");
        assert!(buffer.text().is_char_boundary(buffer.cursor().index));
    }

    #[test]
    fn buffer_normalizes_inserted_line_endings_to_spaces() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("a\nb");

        assert_eq!(buffer.text(), "a b");

        engine.apply_text_edit(&mut buffer, Edit::insert("\nc\r"));

        assert_eq!(buffer.text(), "a b c ");
    }

    #[test]
    fn text_field_selection_layout_uses_shaped_text_span() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);

        let layout = engine.text_field_layout(
            &buffer,
            Style::default().with_size(16.0),
            area::logical(240.0, 32.0),
            TextFieldState::default(),
        );
        let span = layout
            .selection_spans()
            .first()
            .expect("select all should create a highlight span");

        assert!(span.width() > 0.0);
        assert!(span.width() < 240.0);
        assert!(span.x() >= 0.0);
    }

    #[test]
    fn obscured_text_field_hit_testing_maps_display_cursor_to_source_cursor() {
        let mut engine = Engine::new();
        let field = Field::new("åb").obscured_dot();
        let cursor = engine
            .text_field_cursor_at_for_field(
                &field,
                Style::default().with_size(16.0),
                area::logical(200.0, 24.0),
                point::logical(200.0, 8.0),
                TextFieldState::default(),
            )
            .expect("hit testing should return a cursor");

        assert_eq!(field.presentation_text(), "••");
        assert_eq!(field.buffer().text(), "åb");
        assert_eq!(cursor, Cursor::new(0, field.buffer().text().len()));
    }

    #[test]
    fn text_field_reveal_scroll_keeps_caret_inside_content_rect() {
        let mut engine = Engine::new();
        let buffer = Buffer::from_text("hello world this is a long single-line field");
        let area = area::logical(80.0, 32.0);
        let state = engine.text_field_reveal_scroll(
            &buffer,
            Style::default().with_size(16.0),
            area,
            TextFieldState::default(),
        );

        assert!(state.scroll_x() > 0.0);

        let layout =
            engine.text_field_layout(&buffer, Style::default().with_size(16.0), area, state);
        let caret = layout.caret().expect("focused long text should have caret");

        assert!(caret.x() >= 0.0);
        assert!(caret.x() <= area.width());
    }

    #[test]
    fn text_field_caret_visibility_follows_blink_phase() {
        let mut engine = Engine::new();
        let buffer = Buffer::from_text("hello");
        let area = area::logical(100.0, 24.0);
        let epoch = Instant::now();
        let state = TextFieldState::new_at(0.0, epoch);

        let visible = engine.text_field_layout_at(
            &buffer,
            Style::default().with_size(16.0),
            area,
            state.clone(),
            epoch,
        );
        let hidden = engine.text_field_layout_at(
            &buffer,
            Style::default().with_size(16.0),
            area,
            state.clone(),
            epoch + Duration::from_millis(500),
        );
        let visible_again = engine.text_field_layout_at(
            &buffer,
            Style::default().with_size(16.0),
            area,
            state,
            epoch + Duration::from_millis(1000),
        );

        assert!(visible.caret().is_some());
        assert_eq!(hidden.caret(), None);
        assert!(visible_again.caret().is_some());
    }

    #[test]
    fn text_field_selection_suppresses_caret_layout() {
        let mut engine = Engine::new();
        let mut buffer = Buffer::from_text("hello");
        let area = area::logical(100.0, 24.0);
        let epoch = Instant::now();

        engine.apply_text_edit(&mut buffer, Edit::SelectAll);

        let layout = engine.text_field_layout_at(
            &buffer,
            Style::default().with_size(16.0),
            area,
            TextFieldState::new_at(0.0, epoch),
            epoch,
        );

        assert_eq!(layout.caret(), None);
        assert!(!layout.selection_spans().is_empty());
    }

    fn styled_document(text: &str, align: Align, size: f32, weight: Weight) -> Document {
        let mut block = Block::new(align);
        block.push_run(Run::new(
            text,
            Style::default().with_size(size).with_weight(weight),
        ));

        Document::from_block(block)
    }
}
