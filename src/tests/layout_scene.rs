use super::*;
use crate::feedback;

#[derive(Clone)]
struct MillionRowProvider {
    row_calls: Rc<Cell<usize>>,
}

#[derive(Clone)]
struct VariableRowProvider {
    row_calls: Rc<Cell<usize>>,
}

#[derive(Clone)]
struct StableExtentProvider;

#[derive(Clone)]
struct WrappedExtentProvider {
    row_calls: Rc<Cell<usize>>,
}

impl crate::virtual_list::Provider for StableExtentProvider {
    fn len(&self) -> usize {
        100
    }

    fn key(&self, index: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(index as u64)
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        let index = key.value() as usize;
        (index < self.len()).then_some(index)
    }

    fn row(&self, index: usize) -> view::Node {
        view::Node::world_text(format!("Stable row {index}"), text::Overflow::EllipsisEnd)
            .with_style(view::Style::new().with_height(view::Dimension::fixed(24)))
    }
}

impl crate::virtual_list::Provider for WrappedExtentProvider {
    fn len(&self) -> usize {
        10_000
    }

    fn key(&self, index: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(index as u64)
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        let index = key.value() as usize;
        (index < self.len()).then_some(index)
    }

    fn row(&self, index: usize) -> view::Node {
        self.row_calls.set(self.row_calls.get() + 1);
        let text = if index % 3 == 0 {
            format!("Short row {index}")
        } else {
            format!(
                "Wrapped variable row {index} reports its own intrinsic block size under the supplied width"
            )
        };
        let focus = match index {
            0 => "measured.row.0",
            1 => "measured.row.1",
            _ => "measured.row.other",
        };
        view::Node::text_area_state(
            view::TextArea::new(text)
                .with_focus(session::Focus::text(focus))
                .with_wrap(view::Wrap::Word)
                .read_only(),
        )
    }
}

impl crate::virtual_list::Provider for VariableRowProvider {
    fn len(&self) -> usize {
        10_000
    }

    fn key(&self, index: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(index as u64)
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        let index = key.value() as usize;
        (index < self.len()).then_some(index)
    }

    fn row(&self, index: usize) -> view::Node {
        self.row_calls.set(self.row_calls.get() + 1);
        view::Node::world_text(format!("Variable row {index}"), text::Overflow::EllipsisEnd)
            .with_style(
                view::Style::new().with_height(view::Dimension::fixed(match index % 3 {
                    0 => 18,
                    1 => 32,
                    _ => 47,
                })),
            )
    }
}

impl crate::virtual_list::Provider for MillionRowProvider {
    fn len(&self) -> usize {
        1_000_000
    }

    fn key(&self, index: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(index as u64)
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        let index = key.value() as usize;
        (index < self.len()).then_some(index)
    }

    fn row(&self, index: usize) -> view::Node {
        self.row_calls.set(self.row_calls.get() + 1);
        view::Node::world_text(format!("Provider row {index}"), text::Overflow::EllipsisEnd)
    }
}

#[derive(Clone)]
struct MillionTableProvider {
    cell_calls: Rc<Cell<usize>>,
}

#[derive(Clone)]
struct WrappedTableProvider;

impl crate::table::Provider for WrappedTableProvider {
    fn len(&self) -> usize {
        2
    }

    fn key(&self, row: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(row as u64)
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        let row = key.value() as usize;
        (row < self.len()).then_some(row)
    }

    fn cell(&self, row: usize, cell: crate::table::Cell) -> view::Node {
        let text = match (row, cell.column().as_str()) {
            (0, "detail") => "Short detail".to_owned(),
            (1, "detail") => {
                "A deliberately wrapped detail whose height must come from its narrow track"
                    .to_owned()
            }
            (_, "name") => format!("Row {row}"),
            _ => String::new(),
        };
        view::Node::wrapped_world_text(text, view::Wrap::Word)
    }
}

impl crate::table::Provider for MillionTableProvider {
    fn len(&self) -> usize {
        1_000_000
    }

    fn key(&self, row: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(row as u64)
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        let row = key.value() as usize;
        (row < self.len()).then_some(row)
    }

    fn cell(&self, row: usize, cell: crate::table::Cell) -> view::Node {
        self.cell_calls.set(self.cell_calls.get() + 1);
        match cell.column().as_str() {
            "action" => widget::Widget::into_node(widget::Button::new(format!("Open {row}"))),
            "detail" => view::Node::world_text(
                format!("A deliberately long record detail for logical row {row}"),
                text::Overflow::EllipsisMiddle,
            ),
            _ => view::Node::world_text(format!("Record {row}"), text::Overflow::EllipsisEnd),
        }
    }
}

#[test]
fn unchanged_second_commit_paints_zero_scene_nodes() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Retained scene reuse"));
        })
        .view(|_, _| {
            widget::view_node(
                view::Node::stack(view::Axis::Vertical)
                    .child(view::Node::world_text(
                        "Stable title",
                        text::Overflow::EllipsisEnd,
                    ))
                    .child(widget::Widget::into_node(widget::Button::new(
                        "Stable action",
                    ))),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 100);
    app.show_scene(window, size)
        .expect("first commit should populate retained scene fragments");

    app.diagnostics_mut(window)
        .expect("window diagnostics")
        .begin_renderer_measurement();
    app.request_redraw(window);
    app.show_scene(window, size)
        .expect("unchanged second commit should remain drawable");

    let render = &app.diagnostics(window).expect("window diagnostics").render;
    assert_eq!(render.semantic_commits_created, 0);
    assert_eq!(render.scene_nodes_rebuilt, 0);
    assert_eq!(render.scene_paint_calls, 0);
    assert!(render.scene_nodes_reused > 0);
}

#[test]
fn one_sibling_content_change_repaints_only_that_scene_identity() {
    let changed = Rc::new(Cell::new(false));
    let changed_for_view = Rc::clone(&changed);
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Retained sibling change"));
        })
        .view(move |_, _| {
            widget::view_node(
                view::Node::stack(view::Axis::Vertical)
                    .child(view::Node::world_text(
                        "Stable sibling",
                        text::Overflow::EllipsisEnd,
                    ))
                    .child(view::Node::world_text(
                        if changed_for_view.get() {
                            "BBBB"
                        } else {
                            "AAAA"
                        },
                        text::Overflow::EllipsisEnd,
                    )),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 100);
    app.show_scene(window, size)
        .expect("first commit should retain both siblings");

    changed.set(true);
    app.diagnostics_mut(window)
        .expect("window diagnostics")
        .begin_renderer_measurement();
    app.present(window).expect("changed view should reconcile");
    app.show_scene(window, size)
        .expect("changed sibling should produce the next commit");

    let changes = app
        .composition(window)
        .expect("composition should remain installed")
        .changes();
    assert_eq!(changes.changed().len(), 1);
    assert!(changes.added().is_empty());
    assert!(changes.removed().is_empty());
    let render = &app.diagnostics(window).expect("window diagnostics").render;
    assert_eq!(render.semantic_commits_created, 1);
    assert_eq!(render.scene_nodes_rebuilt, 1);
    assert_eq!(render.scene_paint_calls, 1);
    assert!(render.scene_nodes_reused > 0);
}

#[test]
fn departed_scene_nodes_are_removed_once() {
    let show_extra = Rc::new(Cell::new(true));
    let show_extra_for_view = Rc::clone(&show_extra);
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Retained scene removal"));
        })
        .view(move |_, _| {
            let stack = view::Node::stack(view::Axis::Vertical).child(view::Node::world_text(
                "Stable",
                text::Overflow::EllipsisEnd,
            ));
            let stack = if show_extra_for_view.get() {
                stack.child(view::Node::world_text(
                    "Departing",
                    text::Overflow::EllipsisEnd,
                ))
            } else {
                stack
            };
            widget::view_node(stack)
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 100);
    app.show_scene(window, size)
        .expect("first commit should retain both nodes");

    show_extra.set(false);
    app.diagnostics_mut(window)
        .expect("window diagnostics")
        .begin_renderer_measurement();
    app.present(window).expect("changed view should reconcile");
    app.show_scene(window, size)
        .expect("removed node should produce the next commit");
    assert!(
        app.diagnostics(window)
            .expect("window diagnostics")
            .render
            .scene_nodes_removed
            > 0
    );

    app.diagnostics_mut(window)
        .expect("window diagnostics")
        .begin_renderer_measurement();
    app.request_redraw(window);
    app.show_scene(window, size)
        .expect("unchanged post-removal commit should remain drawable");
    assert_eq!(
        app.diagnostics(window)
            .expect("window diagnostics")
            .render
            .scene_nodes_removed,
        0
    );
}

#[derive(Clone)]
struct MutableTableProvider {
    keys: Rc<RefCell<Vec<u64>>>,
}

#[derive(Debug, Clone)]
struct EditableRecord {
    key: u64,
    name: String,
    count: i64,
}

#[derive(Debug, Clone)]
struct EditableTableState {
    records: Vec<EditableRecord>,
}

impl State for EditableTableState {}

#[derive(Clone)]
struct EditableTableProvider {
    records: Vec<EditableRecord>,
}

#[derive(Clone)]
struct SetRecordNameArgs {
    cell: crate::table::Cell,
    value: String,
}

#[derive(Clone)]
struct SetRecordCountArgs {
    cell: crate::table::Cell,
    value: i64,
}

struct SetRecordName;
struct SetRecordCount;

#[derive(Debug, Clone)]
struct TaskGateState {
    records: Vec<EditableRecord>,
    invocations: Vec<&'static str>,
}

impl State for TaskGateState {}

#[derive(Clone)]
enum TaskGateArgs {
    Button,
    Checkbox,
    Slider(f64),
}

struct InvokeTaskGate;
struct InvokeTaskGateShortcut;

impl Command for InvokeTaskGate {
    type Args = TaskGateArgs;
    type Output = ();

    const NAME: &'static str = "test.invoke_task_gate";
}

impl Command for InvokeTaskGateShortcut {
    type Args = ();
    type Output = ();

    const NAME: &'static str = "test.invoke_task_gate_shortcut";
}

impl Command for SetRecordName {
    type Args = SetRecordNameArgs;
    type Output = ();

    const NAME: &'static str = "test.set_record_name";
}

impl Command for SetRecordCount {
    type Args = SetRecordCountArgs;
    type Output = ();

    const NAME: &'static str = "test.set_record_count";
}

impl Target<SetRecordName> for EditableTableState {
    fn state(&self, _: &SetRecordNameArgs, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, args: SetRecordNameArgs, _: &mut Context) -> Response<()> {
        let Some(record) = self
            .records
            .iter_mut()
            .find(|record| record.key == args.cell.row().value())
        else {
            return Response::output(());
        };
        record.name = args.value;
        Response::changed(())
    }
}

impl Target<SetRecordCount> for EditableTableState {
    fn state(&self, _: &SetRecordCountArgs, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, args: SetRecordCountArgs, _: &mut Context) -> Response<()> {
        let Some(record) = self
            .records
            .iter_mut()
            .find(|record| record.key == args.cell.row().value())
        else {
            return Response::output(());
        };
        record.count = args.value;
        Response::changed(())
    }
}

impl Target<SetRecordName> for TaskGateState {
    fn state(&self, _: &SetRecordNameArgs, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, args: SetRecordNameArgs, _: &mut Context) -> Response<()> {
        if let Some(record) = self
            .records
            .iter_mut()
            .find(|record| record.key == args.cell.row().value())
        {
            record.name = args.value;
        }
        Response::changed(())
    }
}

impl Target<SetRecordCount> for TaskGateState {
    fn state(&self, _: &SetRecordCountArgs, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, args: SetRecordCountArgs, _: &mut Context) -> Response<()> {
        if let Some(record) = self
            .records
            .iter_mut()
            .find(|record| record.key == args.cell.row().value())
        {
            record.count = args.value;
        }
        Response::changed(())
    }
}

impl Target<InvokeTaskGate> for TaskGateState {
    fn state(&self, _: &TaskGateArgs, _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, args: TaskGateArgs, _: &mut Context) -> Response<()> {
        self.invocations.push(match args {
            TaskGateArgs::Button => "button",
            TaskGateArgs::Checkbox => "checkbox",
            TaskGateArgs::Slider(value) => {
                let _ = value;
                "slider"
            }
        });
        Response::changed(())
    }
}

impl Target<InvokeTaskGateShortcut> for TaskGateState {
    fn state(&self, _: &(), _: &Context) -> command::State {
        command::State::enabled()
    }

    fn invoke(&mut self, _: (), _: &mut Context) -> Response<()> {
        self.invocations.push("shortcut");
        Response::changed(())
    }
}

impl crate::table::Provider for EditableTableProvider {
    fn len(&self) -> usize {
        self.records.len()
    }

    fn key(&self, row: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(self.records[row].key)
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        self.records
            .iter()
            .position(|record| record.key == key.value())
    }

    fn cell(&self, row: usize, cell: crate::table::Cell) -> view::Node {
        let record = &self.records[row];
        match cell.column().as_str() {
            "name" => widget::Widget::into_node(
                widget::TextBox::new(record.name.clone())
                    .focus(session::Focus::table_cell(cell))
                    .inactive_display(
                        view::Align::Start,
                        view::Wrap::None,
                        text::Overflow::EllipsisEnd,
                    )
                    .try_commit_with::<SetRecordName, String>(move |value| {
                        (!value.trim().is_empty())
                            .then_some(())
                            .ok_or_else(|| "Name is required".to_owned())?;
                        Ok(SetRecordNameArgs { cell, value })
                    }),
            ),
            "count" => widget::Widget::into_node(
                widget::TextBox::new(record.count.to_string())
                    .focus(session::Focus::table_cell(cell))
                    .input(text::Input::signed_integer())
                    .inactive_display(
                        view::Align::End,
                        view::Wrap::None,
                        text::Overflow::EllipsisEnd,
                    )
                    .try_commit_with::<SetRecordCount, String>(move |value| {
                        let value = value
                            .trim()
                            .parse::<i64>()
                            .map_err(|_| "Enter a whole number".to_owned())?;
                        (0..=100)
                            .contains(&value)
                            .then_some(())
                            .ok_or_else(|| "Count must be from 0 to 100".to_owned())?;
                        Ok(SetRecordCountArgs { cell, value })
                    }),
            ),
            "action" => widget::Widget::into_node(
                widget::Button::new("Run").trigger::<InvokeTaskGate>(TaskGateArgs::Button),
            ),
            "enabled" => widget::Widget::into_node(
                widget::Checkbox::new("Enabled", false)
                    .trigger::<InvokeTaskGate>(TaskGateArgs::Checkbox),
            ),
            _ => unreachable!("editable test provider received an unknown column"),
        }
    }
}

fn editable_table_app(state: EditableTableState) -> Runtime<EditableTableState, (), View> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .install(document::Editing::standard())
                .register::<SetRecordName>(command::Spec::new("Set record name"))
                .register::<SetRecordCount>(command::Spec::new("Set record count"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<SetRecordName>()
                .target::<SetRecordCount>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Editable table"));
            cx.open_window(window::Options::new("Editable table second window"));
        })
        .view(|state, _| {
            widget::view_node(
                crate::Table::new(
                    "editable.table",
                    24,
                    [
                        crate::table::Column::new("name", "Name", view::Dimension::weight(1)),
                        crate::table::Column::new("count", "Count", view::Dimension::fixed(100)),
                    ],
                    EditableTableProvider {
                        records: state.records.clone(),
                    },
                )
                .height(view::Dimension::fixed(124)),
            )
        })
}

fn task_gate_app(state: TaskGateState) -> Runtime<TaskGateState, (), View> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .install(document::Editing::standard())
                .register::<SetRecordName>(command::Spec::new("Set record name"))
                .register::<SetRecordCount>(command::Spec::new("Set record count"))
                .register::<InvokeTaskGate>(command::Spec::new("Invoke task gate"))
                .register::<InvokeTaskGateShortcut>(
                    command::Spec::new("Invoke task gate shortcut")
                        .key_chord(command::KeyChord::new("Ctrl+G")),
                );
        })
        .responders(|responders| {
            responders
                .app()
                .target::<SetRecordName>()
                .target::<SetRecordCount>()
                .target::<InvokeTaskGate>()
                .target::<InvokeTaskGateShortcut>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Task gate"));
        })
        .view(|state, _| {
            widget::view(|ui| {
                ui.add(
                    crate::Table::new(
                        "task.gate.table",
                        24,
                        [
                            crate::table::Column::new("name", "Name", view::Dimension::weight(1)),
                            crate::table::Column::new("count", "Count", view::Dimension::fixed(70)),
                            crate::table::Column::new(
                                "action",
                                "Action",
                                view::Dimension::fixed(70),
                            ),
                            crate::table::Column::new(
                                "enabled",
                                "Enabled",
                                view::Dimension::fixed(60),
                            ),
                        ],
                        EditableTableProvider {
                            records: state.records.clone(),
                        },
                    )
                    .height(view::Dimension::fixed(160)),
                );
                ui.button(
                    widget::Button::new("Dependent button")
                        .trigger::<InvokeTaskGate>(TaskGateArgs::Button),
                );
                ui.checkbox(
                    widget::Checkbox::new("Dependent checkbox", false)
                        .trigger::<InvokeTaskGate>(TaskGateArgs::Checkbox),
                );
                ui.slider(
                    widget::Slider::new("Dependent slider", 0.25, 0.0..=1.0)
                        .trigger_with::<InvokeTaskGate, _>(TaskGateArgs::Slider),
                );
            })
        })
}

fn row_gate_app(state: TaskGateState) -> Runtime<TaskGateState, (), View> {
    Runtime::new(state)
        .commands(|commands| {
            commands
                .install(document::Editing::standard())
                .register::<SetRecordName>(command::Spec::new("Set record name"))
                .register::<SetRecordCount>(command::Spec::new("Set record count"))
                .register::<InvokeTaskGate>(command::Spec::new("Invoke task gate"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<SetRecordName>()
                .target::<SetRecordCount>()
                .target::<InvokeTaskGate>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Row gate"));
        })
        .view(|state, _| {
            widget::view_node(
                crate::Table::new(
                    "task.gate.table",
                    24,
                    [
                        crate::table::Column::new("name", "Name", view::Dimension::weight(1)),
                        crate::table::Column::new("count", "Count", view::Dimension::fixed(70)),
                        crate::table::Column::new("action", "Action", view::Dimension::fixed(70)),
                        crate::table::Column::new("enabled", "Enabled", view::Dimension::fixed(60)),
                    ],
                    EditableTableProvider {
                        records: state.records.clone(),
                    },
                )
                .height(view::Dimension::fixed(160)),
            )
        })
}

impl crate::table::Provider for MutableTableProvider {
    fn len(&self) -> usize {
        self.keys.borrow().len()
    }

    fn key(&self, row: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(self.keys.borrow()[row])
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        self.keys
            .borrow()
            .iter()
            .position(|candidate| *candidate == key.value())
    }

    fn cell(&self, row: usize, cell: crate::table::Cell) -> view::Node {
        let key = self.keys.borrow()[row];
        view::Node::world_text(
            format!("{} {key}", cell.column().as_str()),
            text::Overflow::EllipsisEnd,
        )
    }
}

#[derive(Clone)]
struct MutableKeyProvider {
    keys: Rc<RefCell<Vec<u64>>>,
}

impl crate::virtual_list::Provider for MutableKeyProvider {
    fn len(&self) -> usize {
        self.keys.borrow().len()
    }

    fn key(&self, index: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(self.keys.borrow()[index])
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        self.keys
            .borrow()
            .iter()
            .position(|candidate| *candidate == key.value())
    }

    fn row(&self, index: usize) -> view::Node {
        let key = self.keys.borrow()[index];
        view::Node::world_text(format!("Key {key}"), text::Overflow::EllipsisEnd)
    }
}

#[derive(Clone, Copy)]
enum PinnedRowKind {
    Text,
    Capture,
}

#[derive(Clone)]
struct PinnedRowProvider {
    keys: Rc<RefCell<Vec<u64>>>,
    kind: PinnedRowKind,
}

impl crate::virtual_list::Provider for PinnedRowProvider {
    fn len(&self) -> usize {
        self.keys.borrow().len()
    }

    fn key(&self, index: usize) -> crate::virtual_list::Key {
        crate::virtual_list::Key::new(self.keys.borrow()[index])
    }

    fn index_of(&self, key: crate::virtual_list::Key) -> Option<usize> {
        self.keys
            .borrow()
            .iter()
            .position(|candidate| *candidate == key.value())
    }

    fn row(&self, index: usize) -> view::Node {
        let key = self.keys.borrow()[index];
        match self.kind {
            PinnedRowKind::Text if key == 0 => widget::Widget::into_node(
                widget::TextBox::new("Text 0").focus(session::Focus::text("virtual.text.0")),
            ),
            PinnedRowKind::Text if key == 1 => widget::Widget::into_node(
                widget::TextBox::new("Text 1").focus(session::Focus::text("virtual.text.1")),
            ),
            PinnedRowKind::Text => {
                view::Node::world_text(format!("Text {key}"), text::Overflow::EllipsisEnd)
            }
            PinnedRowKind::Capture => widget::Widget::into_node(
                widget::Scroll::new()
                    .height(view::Dimension::fixed(20))
                    .child(widget::Label::world(
                        format!("Capture {key}"),
                        text::Overflow::EllipsisEnd,
                    )),
            ),
        }
    }
}

macro_rules! palette_test_command {
    ($name:ident, $command:literal) => {
        struct $name;

        impl Command for $name {
            type Args = ();
            type Output = ();

            const NAME: &'static str = $command;
        }

        impl Target<$name> for SourceState {
            fn state(&self, _: &(), _: &Context) -> command::State {
                command::State::enabled()
            }

            fn invoke(&mut self, _: (), cx: &mut Context) -> Response<()> {
                self.sources.push(cx.source());
                Response::changed(())
            }
        }
    };
}

palette_test_command!(PaletteOne, "palette.one");
palette_test_command!(PaletteTwo, "palette.two");
palette_test_command!(PaletteThree, "palette.three");
palette_test_command!(PaletteFour, "palette.four");
palette_test_command!(PaletteFive, "palette.five");
palette_test_command!(PaletteSix, "palette.six");
palette_test_command!(PaletteSeven, "palette.seven");
palette_test_command!(PaletteEight, "palette.eight");
palette_test_command!(PaletteNine, "palette.nine");
palette_test_command!(PaletteTen, "palette.ten");
palette_test_command!(PaletteEleven, "palette.eleven");
palette_test_command!(PaletteTwelve, "palette.twelve");

struct DisabledTextSubmit;

impl Command for DisabledTextSubmit {
    type Args = String;
    type Output = ();

    const NAME: &'static str = "test.disabled_text_submit";
}

impl Target<DisabledTextSubmit> for SourceState {
    fn state(&self, _: &String, _: &Context) -> command::State {
        command::State::disabled()
    }

    fn invoke(&mut self, _: String, _: &mut Context) -> Response<()> {
        Response::changed(())
    }
}

#[test]
fn million_row_virtual_list_converges_to_a_bounded_first_frame() {
    let row_calls = Rc::new(Cell::new(0));
    let provider = MillionRowProvider {
        row_calls: Rc::clone(&row_calls),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Million rows"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::new("million.rows", 20, provider.clone())
                    .width(view::Dimension::grow())
                    .height(view::Dimension::fixed(100)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();

    let scene = app
        .show_scene(window, geometry::Size::new(240, 100))
        .expect("virtual list should render");
    let values = scene
        .scene()
        .texts()
        .into_iter()
        .map(scene::Text::value)
        .collect::<Vec<_>>();

    assert!(!values.is_empty());
    assert!(values.len() <= 9, "visible rows plus overscan stay bounded");
    assert_eq!(values[0], "Provider row 0");
    assert!(
        row_calls.get() <= 48,
        "initial bootstrap plus converged materialization must not scale with provider length"
    );

    let projected = app.present(window).expect("view should remain projectable");
    assert!(projected.labels().len() <= 9);
}

#[test]
fn million_row_virtual_list_large_scrolls_stay_exact_and_bounded() {
    let row_calls = Rc::new(Cell::new(0));
    let provider = MillionRowProvider {
        row_calls: Rc::clone(&row_calls),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Million-row jump"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::new("million.jump", 24, provider.clone())
                    .width(view::Dimension::grow())
                    .height(view::Dimension::grow()),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let compact = geometry::Size::new(240, 100);
    let initial = app
        .show_scene(window, compact)
        .expect("initial virtual list should render");
    let list = initial.layout().find_role(view::Role::VirtualList)[0];
    let point = frame_point_at(list.rect());
    let projection = initial
        .layout()
        .scroll_projections()
        .iter()
        .find(|projection| projection.node() == list.node_id())
        .expect("virtual list should declare one authoritative scroll projection");
    let owner = projection.node();
    let target = projection.target().clone();
    let maximum = projection.viewport().max_scroll();
    assert_eq!(
        maximum,
        interaction::ScrollOffset::new(0, 23_999_900),
        "one million 24-pixel rows must expose the gallery-scale integral extent"
    );
    let calls_before_jump = row_calls.get();

    app.scroll_at(
        window,
        compact,
        point,
        interaction::ScrollDelta::vertical(12_000_000),
    )
    .expect("jump scroll should be handled");
    let jumped = app
        .show_scene(window, compact)
        .expect("jumped virtual list should render");
    let jumped_values = jumped
        .scene()
        .texts()
        .into_iter()
        .map(scene::Text::value)
        .collect::<Vec<_>>();

    assert!(
        jumped_values.iter().any(|value| value.contains("499998")),
        "jump should derive the distant logical range arithmetically"
    );
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window interaction")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 12_000_000)
    );
    assert_eq!(
        jumped.properties().scroll_offset(owner),
        Some(interaction::ScrollOffset::new(0, 12_000_000)),
        "the scene property must derive from the same large integral position"
    );
    assert!(jumped_values.len() <= 9);
    assert!(
        jumped
            .layout()
            .frames()
            .iter()
            .filter(|frame| frame.role() != view::Role::Root)
            .count()
            <= 10,
        "the stable view root is infrastructure; materialized list frames stay bounded"
    );
    let jump_row_calls = row_calls.get().saturating_sub(calls_before_jump);
    assert!(
        jump_row_calls <= 16,
        "a gallery-scale relative jump must retain bounded materialization; observed {jump_row_calls} row builds"
    );

    let near_maximum = interaction::ScrollOffset::new(0, maximum.y() - 1);
    let calls_before_absolute = row_calls.get();
    app.handle_input(window, Input::scroll_to(target.clone(), near_maximum))
        .expect("a gallery-scale absolute thumb position should be accepted");
    let near_end = app
        .show_scene(window, compact)
        .expect("the odd offset immediately below the gallery maximum should render");
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window interaction")
            .scroll()
            .offset(&target),
        near_maximum
    );
    assert_eq!(
        near_end.properties().scroll_offset(owner),
        Some(near_maximum)
    );
    assert!(
        near_end
            .scene()
            .texts()
            .iter()
            .any(|text| text.value() == "Provider row 999999"),
        "the viewport must render through the final row instead of outrunning residency"
    );
    assert!(
        row_calls.get().saturating_sub(calls_before_absolute) <= 16,
        "a gallery-scale absolute jump must retain bounded materialization"
    );

    app.scroll_at(
        window,
        compact,
        point,
        interaction::ScrollDelta::vertical(100),
    )
    .expect("the final relative tick should clamp to the exact maximum");
    let at_end = app
        .show_scene(window, compact)
        .expect("the exact gallery maximum should render");
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window interaction")
            .scroll()
            .offset(&target),
        maximum
    );
    assert_eq!(at_end.properties().scroll_offset(owner), Some(maximum));

    let tall = app
        .show_scene(window, geometry::Size::new(240, 180))
        .expect("resized virtual list should render");
    assert!(tall.scene().texts().len() <= 13);
    assert!(
        tall.layout()
            .frames()
            .iter()
            .filter(|frame| frame.role() != view::Role::Root)
            .count()
            <= 14
    );
}

#[test]
fn virtual_scroll_residency_does_not_bridge_a_distant_focus_pin() {
    let keys = Rc::new(RefCell::new((0..1_000).collect::<Vec<_>>()));
    let provider = PinnedRowProvider {
        keys,
        kind: PinnedRowKind::Text,
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Virtual resident window"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::new("resident.rows", 24, provider.clone())
                    .width(view::Dimension::grow())
                    .height(view::Dimension::fixed(500)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(260, 500);
    let initial = app
        .show_scene(window, size)
        .expect("resident-window list should render");
    let list_rect = initial.layout().find_role(view::Role::VirtualList)[0].rect();
    assert!(app.focus_virtual_row(
        window,
        interaction::Id::new("resident.rows"),
        crate::virtual_list::Key::new(0),
        session::Focus::text("virtual.text.0"),
    ));
    app.scroll_at(
        window,
        size,
        frame_point_at(list_rect),
        interaction::ScrollDelta::vertical(2_400),
    )
    .expect("resident-window list should scroll away from its focus pin");
    let scrolled = app
        .show_scene(window, size)
        .expect("scrolled resident-window list should render");
    let list = &scrolled.layout().find_role(view::Role::VirtualList)[0];
    let requested = list
        .virtual_list_request()
        .expect("virtual list should declare its contiguous requested range")
        .range();
    assert!(!requested.contains(&0));
    assert!(
        scrolled
            .layout()
            .frames()
            .iter()
            .any(|frame| { frame.provided_row().is_some_and(|row| row.index() == 0) })
    );

    let projection = scrolled
        .layout()
        .scroll_projections()
        .iter()
        .find(|projection| projection.node() == list.node_id())
        .expect("virtual list should retain one scroll projection");
    let visible = projection.viewport().visible_content();
    let resident = projection
        .resident_bounds()
        .expect("requested virtual rows should prove contiguous residency");
    let layer = projection.layer_bounds();
    let resident_behind = visible.y().saturating_sub(resident.y());
    let layer_behind = visible.y().saturating_sub(layer.y());
    assert!(
        layer_behind > resident_behind,
        "the distant pin may extend layer geometry but cannot manufacture resident rows: visible={visible:?} resident={resident:?} layer={layer:?} requested={requested:?}"
    );

    let baseline = projection.viewport().resolved_scroll();
    let beyond_resident = interaction::ScrollOffset::new(
        baseline.x(),
        baseline
            .y()
            .saturating_sub(resident_behind)
            .saturating_sub(1),
    );
    assert_eq!(
        projection.viewport().resolve(beyond_resident),
        beyond_resident
    );
    assert!(
        !scrolled
            .layout()
            .scroll_property_accepts(projection.target(), beyond_resident),
        "property scrolling must replenish before exposing a hole beyond the requested row window"
    );
}

#[test]
fn variable_virtual_list_measures_mixed_rows_with_bounded_runtime_work() {
    let row_calls = Rc::new(Cell::new(0));
    let provider = VariableRowProvider {
        row_calls: Rc::clone(&row_calls),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Variable rows"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::variable("variable.rows", 24, provider.clone())
                    .width(view::Dimension::grow())
                    .height(view::Dimension::fixed(120)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(240, 120))
        .expect("variable list should converge");
    let rows = rendered
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| frame.provided_row().map(|row| (row.index(), frame.rect())))
        .collect::<Vec<_>>();

    assert!(!rows.is_empty());
    assert!(rows.len() <= 12);
    for (index, rect) in rows {
        assert_eq!(
            rect.height(),
            match index % 3 {
                0 => 18,
                1 => 32,
                _ => 47,
            }
        );
    }
    assert!(row_calls.get() <= 64);
}

#[test]
fn variable_measurements_survive_a_same_range_mode_transition_and_rebuild() {
    let variable = Rc::new(Cell::new(false));
    let variable_for_view = Rc::clone(&variable);
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Retained variable measurements"));
        })
        .view(move |_, _| {
            let list = if variable_for_view.get() {
                crate::VirtualList::variable("retained.measurements", 24, StableExtentProvider)
            } else {
                crate::VirtualList::new("retained.measurements", 24, StableExtentProvider)
            };
            widget::view_node(
                list.width(view::Dimension::grow())
                    .height(view::Dimension::fixed(72)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(180, 72);
    app.show_scene(window, size)
        .expect("uniform source should establish the same visible range");

    variable.set(true);
    app.request_redraw(window);
    app.show_scene(window, size)
        .expect("variable source should attach measured geometry");
    let first = app
        .composition(window)
        .and_then(|composition| {
            composition.virtual_list_model(interaction::Id::new("retained.measurements"))
        })
        .and_then(crate::virtual_list::Model::measurements)
        .expect("variable list should expose its retained measurement owner");

    app.request_redraw(window);
    app.show_scene(window, size)
        .expect("same-range rebuild should preserve measured geometry");
    let rebuilt = app
        .composition(window)
        .and_then(|composition| {
            composition.virtual_list_model(interaction::Id::new("retained.measurements"))
        })
        .and_then(crate::virtual_list::Model::measurements)
        .expect("rebuilt variable list should keep measured geometry");

    assert!(
        first == rebuilt,
        "same range and pins must not hide a changed measurement owner"
    );
}

#[test]
fn measured_virtual_sequence_covers_a_short_viewport_through_pin_scroll_and_resize() {
    let row_calls = Rc::new(Cell::new(0));
    let provider = WrappedExtentProvider {
        row_calls: Rc::clone(&row_calls),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Measured sequence coverage"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::variable("measured.sequence", 24, provider.clone())
                    .width(view::Dimension::grow())
                    .height(view::Dimension::fixed(72)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let initial_size = geometry::Size::new(180, 72);
    let initial = app
        .show_scene(window, initial_size)
        .expect("wrapped variable list should render");
    let list = initial.layout().find_role(view::Role::VirtualList)[0].clone();
    drop(initial);
    assert!(app.focus_virtual_row(
        window,
        interaction::Id::new("measured.sequence"),
        crate::virtual_list::Key::new(0),
        session::Focus::text("measured.row.0"),
    ));
    app.scroll_at(
        window,
        initial_size,
        frame_point_at(list.rect()),
        interaction::ScrollDelta::vertical(720),
    )
    .expect("variable list should scroll");
    let scrolled = app
        .show_scene(window, initial_size)
        .expect("scrolled variable list should render");
    assert!(scrolled.layout().frames().iter().any(|frame| {
        frame
            .provided_row()
            .is_some_and(|row| row.index() == 0 && frame.rect().bottom() <= list.rect().y())
    }));
    let retained = app
        .composition(window)
        .and_then(|composition| {
            composition.virtual_list_model(interaction::Id::new("measured.sequence"))
        })
        .and_then(crate::virtual_list::Model::measurements)
        .expect("scrolled list should retain measured geometry");

    app.request_redraw(window);
    let narrow_size = geometry::Size::new(110, 72);
    let narrowed = app
        .show_scene(window, narrow_size)
        .expect("width invalidation should remeasure without losing the sequence");
    let rebuilt = app
        .composition(window)
        .and_then(|composition| {
            composition.virtual_list_model(interaction::Id::new("measured.sequence"))
        })
        .and_then(crate::virtual_list::Model::measurements)
        .expect("remeasured list should retain its geometry owner");
    assert!(retained == rebuilt);

    let viewport = narrowed.layout().find_role(view::Role::VirtualList)[0]
        .viewport()
        .expect("variable list viewport")
        .visible_content();
    let mut visible = narrowed
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| {
            frame
                .provided_row()
                .filter(|_| {
                    frame.rect().bottom() > viewport.y() && frame.rect().y() < viewport.bottom()
                })
                .map(|_| frame.rect())
        })
        .collect::<Vec<_>>();
    visible.sort_unstable_by_key(|rect| rect.y());
    assert!(!visible.is_empty());
    assert!(visible[0].y() <= viewport.y());
    assert!(
        visible
            .last()
            .is_some_and(|rect| rect.bottom() >= viewport.bottom())
    );
    assert!(visible.iter().any(|rect| rect.height() > 24));
    assert!(row_calls.get() <= 96, "variable work must remain bounded");
}

#[test]
fn million_row_table_composes_public_cells_with_bounded_aligned_tracks() {
    let cell_calls = Rc::new(Cell::new(0));
    let provider = MillionTableProvider {
        cell_calls: Rc::clone(&cell_calls),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Million-row table"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::Table::new(
                    "records",
                    20,
                    [
                        crate::table::Column::new("name", "Name", view::Dimension::fixed(80)),
                        crate::table::Column::new("detail", "Detail", view::Dimension::weight(1)),
                        crate::table::Column::new("action", "Action", view::Dimension::weight(2)),
                    ],
                    provider.clone(),
                )
                .header_height(28)
                .width(view::Dimension::grow())
                .height(view::Dimension::fixed(128)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 128);
    let rendered = app
        .show_scene(window, size)
        .expect("record table should render");

    let headers = rendered
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| frame.table_header_cell().map(|cell| (cell, frame.rect())))
        .collect::<Vec<_>>();
    let cells = rendered
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| {
            frame
                .table_cell()
                .map(|cell| (cell, frame.rect(), frame.role()))
        })
        .collect::<Vec<_>>();

    assert_eq!(headers.len(), 3);
    assert!(!cells.is_empty());
    assert!(
        cells.len() <= 27,
        "visible cells plus overscan stay bounded"
    );
    assert!(
        cell_calls.get() <= 144,
        "table work must not scale with provider length"
    );
    assert_eq!(headers[0].1.width(), 80);
    assert_eq!(headers[1].1.width(), 80);
    assert_eq!(headers[2].1.width(), 160);
    for (header, header_rect) in &headers {
        let first_row = cells
            .iter()
            .find(|(cell, _, _)| {
                cell.row() == crate::virtual_list::Key::new(0) && cell.column() == header.column()
            })
            .expect("every header track should align to a first-row cell");
        assert_eq!(first_row.1.x(), header_rect.x());
        assert_eq!(first_row.1.width(), header_rect.width());
    }
    assert!(cells.iter().any(|(_, _, role)| *role == view::Role::Button));
    assert!(
        rendered
            .layout()
            .frames()
            .iter()
            .any(|frame| frame.world_text_overflow() == Some(text::Overflow::EllipsisMiddle))
    );
    let measurement = layout::Engine::new();
    let interface = Theme::default().typography().interface();
    for column in ["name", "detail"] {
        let frame = rendered
            .layout()
            .frames()
            .iter()
            .find(|frame| {
                frame.table_cell().is_some_and(|cell| {
                    cell.row() == crate::virtual_list::Key::new(0)
                        && cell.column() == interaction::Id::new(column)
                })
            })
            .expect("first-row world-text cell");
        let content = layout::table_content_rect(frame.rect(), &Theme::default());
        let painted = rendered
            .scene()
            .texts()
            .into_iter()
            .find(|text| text.rect() == content)
            .expect("table text should paint in its canonical content rectangle");
        let approved = measurement.test_label_width_with_style(painted.value(), interface);
        assert!(
            approved <= content.width(),
            "approved {column} text width {approved} must fit painted width {}",
            content.width()
        );
        for scale in [1.0_f32, 1.25, 1.5, 2.0] {
            assert!(
                (approved as f32 * scale).ceil() <= (content.width() as f32 * scale).ceil(),
                "approved {column} text must fit after {scale}x projection"
            );
        }
    }
    let column_tracks = rendered
        .layout()
        .table_tracks()
        .iter()
        .filter(|track| track.axis() == layout::table::Axis::Column)
        .collect::<Vec<_>>();
    let row_tracks = rendered
        .layout()
        .table_tracks()
        .iter()
        .filter(|track| track.axis() == layout::table::Axis::Row)
        .collect::<Vec<_>>();
    assert_eq!(column_tracks.len(), headers.len());
    assert_eq!(
        row_tracks.len(),
        rendered
            .layout()
            .frames()
            .iter()
            .filter(|frame| frame.table_row().is_some())
            .count()
            + 1
    );
    for track in column_tracks {
        let identity = track.column_identity().expect("column track identity");
        let header_rect = headers
            .iter()
            .find_map(|(header, rect)| (*header == identity).then_some(*rect))
            .expect("column track should resolve to its header");
        assert_eq!(track.boundary(), header_rect.right());
        assert_eq!(
            track.rule_rect().x() + track.rule_rect().width() / 2,
            track.boundary()
        );
        assert!(rendered.scene().rules().iter().any(|rule| {
            rule.axis() == scene::Axis::Vertical && rule.rect() == track.rule_rect()
        }));
    }
    let header_bottom = headers[0].1.bottom();
    let row_bottoms = rendered
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| frame.table_row().map(|_| frame.rect().bottom()))
        .collect::<Vec<_>>();
    for track in row_tracks {
        assert_eq!(
            track.rule_rect().y() + track.rule_rect().height() / 2,
            track.boundary()
        );
        assert!(track.boundary() == header_bottom || row_bottoms.contains(&track.boundary()));
        assert!(rendered.scene().rules().iter().any(|rule| {
            rule.axis() == scene::Axis::Horizontal && rule.rect() == track.rule_rect()
        }));
    }
}

#[test]
fn table_internal_scroll_uses_a_subject_instead_of_a_painted_label() {
    let table = widget::Widget::into_node(crate::Table::new(
        "subject.records",
        20,
        [crate::table::Column::new(
            "name",
            "Name",
            view::Dimension::fixed(80),
        )],
        MillionTableProvider {
            cell_calls: Rc::new(Cell::new(0)),
        },
    ));
    let scroll = table
        .children()
        .first()
        .expect("table should own its horizontal scroll node");

    assert_eq!(scroll.role(), view::Role::Scroll);
    assert_eq!(
        scroll.subject().map(subject::Segment::label),
        Some("Table columns")
    );
    assert_eq!(scroll.label_text(), None);
}

#[test]
fn table_header_band_fills_the_scroll_surface_beyond_its_columns() {
    let provider = MillionTableProvider {
        cell_calls: Rc::new(Cell::new(0)),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Table header band"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::Table::new(
                    "header.band.records",
                    20,
                    [crate::table::Column::new(
                        "name",
                        "Name",
                        view::Dimension::fixed(80),
                    )],
                    provider.clone(),
                )
                .width(view::Dimension::grow())
                .height(view::Dimension::fixed(108)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 108);
    let rendered = app
        .show_scene(window, size)
        .expect("narrow table tracks should render in a wider viewport");
    let header_band = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_part() == Some(view::TablePart::HeaderBand))
        .expect("table should expose one full-width header band");
    let last_header = rendered
        .layout()
        .frames()
        .iter()
        .filter(|frame| frame.table_header_cell().is_some())
        .map(layout::Frame::rect)
        .max_by_key(|rect| rect.right())
        .expect("table should expose its column header");
    let horizontal = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.role() == view::Role::Scroll && frame.table_projection().is_some())
        .expect("table should expose its horizontal viewport");
    let visible_right = horizontal
        .viewport()
        .expect("horizontal table viewport")
        .visible_content()
        .right();

    assert_eq!(header_band.rect().right(), visible_right);
    assert!(header_band.rect().right() > last_header.right());
    assert!(rendered.scene().quads().iter().any(|quad| {
        quad.rect() == header_band.rect()
            && quad.fill() == Theme::default().table().header_background
    }));
}

#[test]
fn table_projects_minimum_tracks_once_and_scrolls_header_body_and_rules_together() {
    let provider = MillionTableProvider {
        cell_calls: Rc::new(Cell::new(0)),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Horizontally scrolling table"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::Table::new(
                    "wide.records",
                    20,
                    [
                        crate::table::Column::new("name", "Name", view::Dimension::fixed(100)),
                        crate::table::Column::new(
                            "detail",
                            "Detail",
                            view::Dimension::weight(1).minimum(120),
                        ),
                        crate::table::Column::new("action", "Action", view::Dimension::fixed(90)),
                    ],
                    provider.clone(),
                )
                .height(view::Dimension::fixed(108)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 108);
    let initial = app
        .show_scene(window, size)
        .expect("wide table should render");
    let horizontal = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.role() == view::Role::Scroll && frame.table_projection().is_some())
        .expect("table should compose one horizontal scroll owner");
    let table_scroll_target = horizontal
        .target()
        .expect("table horizontal projection should expose its shared target")
        .clone();
    let viewport = horizontal.viewport().expect("horizontal viewport");
    assert_eq!(viewport.content().width(), 310);
    assert_eq!(viewport.max_scroll(), interaction::ScrollOffset::new(70, 0));
    assert!(
        initial
            .layout()
            .chrome()
            .iter()
            .any(|chrome| chrome.viewport() == viewport)
    );
    assert!(
        initial
            .scene()
            .texts()
            .iter()
            .all(|text| text.value() != "Table columns"),
        "the table's internal scroll subject must not paint as a caption"
    );

    let vertical_scrollbar = |rendered: &scene::Presentation| {
        let viewport = rendered
            .layout()
            .frames()
            .iter()
            .find(|frame| {
                frame.role() == view::Role::VirtualList
                    && frame
                        .viewport()
                        .is_some_and(|viewport| viewport.max_scroll().y() > 0)
            })
            .and_then(layout::Frame::viewport)
            .expect("table body should expose a vertical viewport");
        let track = rendered
            .layout()
            .chrome()
            .iter()
            .find_map(|chrome| (chrome.viewport() == viewport).then(|| chrome.track()))
            .expect("table body should project a vertical scrollbar");
        (viewport, track)
    };
    let (initial_vertical_viewport, initial_vertical_track) = vertical_scrollbar(&initial);
    assert_eq!(
        initial
            .layout()
            .chrome()
            .iter()
            .filter(|chrome| chrome.scroll_target() == &table_scroll_target)
            .map(layout::Chrome::axis)
            .collect::<std::collections::HashSet<_>>(),
        [
            interaction::ScrollbarAxis::Horizontal,
            interaction::ScrollbarAxis::Vertical,
        ]
        .into_iter()
        .collect(),
        "table chrome axes must consume one shared interaction target"
    );
    assert_eq!(initial_vertical_viewport.rect().right(), 310);
    assert_eq!(initial_vertical_viewport.visible_frame().right(), 240);
    assert_eq!(
        initial_vertical_track.right(),
        initial_vertical_viewport
            .visible_frame()
            .right()
            .saturating_sub(Theme::dark().scrollbar().appearance.margin)
    );
    let vertical_hit = initial
        .layout()
        .hit_test(frame_point_at(initial_vertical_track))
        .expect("the visible vertical scrollbar should be interactive");
    assert!(vertical_hit.is_chrome());
    assert_eq!(
        vertical_hit
            .target()
            .expect("scrollbar chrome should expose a target")
            .kind(),
        interaction::Kind::Scrollbar
    );

    let geometry_for = |rendered: &scene::Presentation, column: &'static str| {
        let column = interaction::Id::new(column);
        let header = rendered
            .layout()
            .frames()
            .iter()
            .find(|frame| {
                frame
                    .table_header_cell()
                    .is_some_and(|cell| cell.column() == column)
            })
            .expect("header geometry")
            .rect();
        let body = rendered
            .layout()
            .frames()
            .iter()
            .find(|frame| {
                frame.table_cell().is_some_and(|cell| {
                    cell.row() == crate::virtual_list::Key::new(0) && cell.column() == column
                })
            })
            .expect("body geometry")
            .rect();
        let boundary = rendered
            .layout()
            .table_tracks()
            .iter()
            .find(|track| {
                track
                    .column_identity()
                    .is_some_and(|cell| cell.column() == column)
            })
            .expect("track geometry")
            .boundary();
        (header, body, boundary)
    };
    let initial_name = geometry_for(&initial, "name");
    let initial_detail = geometry_for(&initial, "detail");
    let initial_action = geometry_for(&initial, "action");
    assert_eq!(initial_name.0.width(), 100);
    assert_eq!(initial_detail.0.width(), 120);
    assert_eq!(initial_action.0.width(), 90);
    for (header, body, boundary) in [initial_name, initial_detail, initial_action] {
        assert_eq!(header.x(), body.x());
        assert_eq!(header.width(), body.width());
        assert_eq!(header.right(), boundary);
    }
    assert_eq!(initial_action.0.right(), 310);
    let initial_action_track = initial
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track
                .column_identity()
                .is_some_and(|cell| cell.column() == interaction::Id::new("action"))
        })
        .expect("far-right track");
    let initial_action_divider = initial_action_track
        .divider_hit_rect()
        .expect("projected far-right hit zone");
    assert!(
        initial_action_divider.x() >= 240,
        "an offscreen divider must not clamp into a false viewport-edge target"
    );
    let detail_column_track = initial
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track.axis() == layout::table::Axis::Column
                && track
                    .column_identity()
                    .is_some_and(|cell| cell.column() == interaction::Id::new("detail"))
        })
        .expect("detail column rule");
    let detail_column_rule = detail_column_track.rule_rect();
    let first_row_track = initial
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track.axis() == layout::table::Axis::Row
                && track.boundary() == initial_detail.1.bottom()
        })
        .expect("first body row rule");
    let first_row_rule = first_row_track.rule_rect();
    let column_scrolls = initial
        .layout()
        .scroll_ancestry(detail_column_track.owner_node());
    let row_scrolls = initial
        .layout()
        .scroll_ancestry(first_row_track.owner_node());
    assert_eq!(column_scrolls.len(), 1, "header rules move horizontally");
    assert_eq!(row_scrolls.len(), 2, "body rules move on both axes");
    assert_eq!(
        row_scrolls.first(),
        column_scrolls.first(),
        "header and body rules inherit the same horizontal owner"
    );
    let detail_text = initial
        .scene()
        .texts()
        .into_iter()
        .find(|text| text.rect() == layout::table_content_rect(initial_detail.1, &Theme::default()))
        .expect("first detail cell text")
        .rect();
    let alternate_row = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_row().is_some_and(|row| row.index() == 1))
        .expect("second body row with the alternating background")
        .rect();
    let translate = |rect: geometry::Rect, x: i32, y: i32| {
        geometry::Rect::new(
            rect.x().saturating_add(x),
            rect.y().saturating_add(y),
            rect.width(),
            rect.height(),
        )
    };
    let table_theme = Theme::default();
    assert!(initial.scene().quads().iter().any(|quad| {
        quad.rect() == alternate_row && quad.fill() == table_theme.table().alternate_row_tint
    }));
    assert!(initial.scene().quads().iter().any(|quad| {
        quad.rect() == initial_detail.0 && quad.fill() == table_theme.table().header_background
    }));

    let before_scroll = app
        .diagnostics(window)
        .expect("table diagnostics")
        .pipeline
        .clone();
    app.scroll_at(
        window,
        size,
        frame_point_at(initial_name.0),
        interaction::ScrollDelta::horizontal(70),
    )
    .expect("horizontal delta should be consumed by the table scroll owner");
    let after_scroll = app
        .diagnostics(window)
        .expect("table diagnostics")
        .pipeline
        .clone();
    assert_eq!(after_scroll.routing_layouts, before_scroll.routing_layouts);
    assert_eq!(after_scroll.frames_prepared, before_scroll.frames_prepared);
    let scrolled = app
        .show_scene(window, size)
        .expect("scrolled wide table should render");
    assert_eq!(
        app.diagnostics(window)
            .expect("table diagnostics")
            .pipeline
            .frames_prepared,
        before_scroll.frames_prepared + 1
    );
    let scrolled_name = geometry_for(&scrolled, "name");
    let scrolled_detail = geometry_for(&scrolled, "detail");
    let scrolled_action = geometry_for(&scrolled, "action");
    assert!(scrolled.property_only());
    assert!(std::sync::Arc::ptr_eq(initial.commit(), scrolled.commit()));
    let horizontal_projection = scrolled
        .layout()
        .scroll_projections()
        .iter()
        .find(|projection| projection.viewport().max_scroll().x() > 0)
        .expect("table should retain horizontal property topology");
    assert_eq!(
        scrolled
            .properties()
            .scroll_offset(horizontal_projection.node()),
        Some(interaction::ScrollOffset::new(70, 0))
    );
    let (scrolled_vertical_viewport, scrolled_vertical_track) = vertical_scrollbar(&scrolled);
    assert_eq!(
        scrolled_vertical_viewport.visible_frame(),
        initial_vertical_viewport.visible_frame()
    );
    assert_eq!(scrolled_vertical_track, initial_vertical_track);
    for (before, after) in [
        (initial_name, scrolled_name),
        (initial_detail, scrolled_detail),
        (initial_action, scrolled_action),
    ] {
        assert_eq!(
            after, before,
            "property ticks must not rewrite commit geometry"
        );
    }
    assert_eq!(scrolled_action.0.right(), 310);
    let horizontal_delta = interaction::ScrollOffset::new(-70, 0);
    let horizontal_alternate_row =
        translate(alternate_row, horizontal_delta.x(), horizontal_delta.y());
    let horizontal_detail_header =
        translate(initial_detail.0, horizontal_delta.x(), horizontal_delta.y());
    let horizontal_column_rule = translate(
        detail_column_rule,
        horizontal_delta.x(),
        horizontal_delta.y(),
    );
    let horizontal_row_rule = translate(first_row_rule, horizontal_delta.x(), horizontal_delta.y());
    let horizontal_detail_text = translate(detail_text, horizontal_delta.x(), horizontal_delta.y());
    assert!(scrolled.scene().quads().iter().any(|quad| {
        quad.rect() == horizontal_alternate_row
            && quad.fill() == table_theme.table().alternate_row_tint
    }));
    assert!(scrolled.scene().quads().iter().any(|quad| {
        quad.rect() == horizontal_detail_header
            && quad.fill() == table_theme.table().header_background
    }));
    assert!(scrolled.scene().rules().iter().any(|rule| {
        rule.axis() == scene::Axis::Vertical && rule.rect() == horizontal_column_rule
    }));
    assert!(scrolled.scene().rules().iter().any(|rule| {
        rule.axis() == scene::Axis::Horizontal && rule.rect() == horizontal_row_rule
    }));
    assert!(
        scrolled
            .scene()
            .texts()
            .iter()
            .any(|text| text.rect() == horizontal_detail_text),
        "horizontal property tick must reveal translated cell text immediately"
    );
    assert_eq!(
        scrolled
            .layout()
            .table_tracks()
            .iter()
            .find(|track| {
                track
                    .column_identity()
                    .is_some_and(|cell| cell.column() == interaction::Id::new("action"))
            })
            .and_then(layout::table::Track::divider_hit_rect)
            .expect("revealed far-right hit zone")
            .right(),
        initial_action_divider.right()
    );

    app.handle_input(
        window,
        Input::scroll(
            table_scroll_target.clone(),
            interaction::ScrollDelta::vertical(20),
        ),
    )
    .expect("vertical delta should be consumed by the same table owner");
    let diagonally_scrolled = app
        .show_scene(window, size)
        .expect("diagonally scrolled table should render");
    assert!(diagonally_scrolled.property_only());
    assert!(std::sync::Arc::ptr_eq(
        initial.commit(),
        diagonally_scrolled.commit()
    ));
    let body_delta = interaction::ScrollOffset::new(-70, -20);
    assert!(diagonally_scrolled.scene().quads().iter().any(|quad| {
        quad.rect() == translate(alternate_row, body_delta.x(), body_delta.y())
            && quad.fill() == table_theme.table().alternate_row_tint
    }));
    assert!(diagonally_scrolled.scene().quads().iter().any(|quad| {
        quad.rect() == horizontal_detail_header
            && quad.fill() == table_theme.table().header_background
    }));
    assert!(diagonally_scrolled.scene().rules().iter().any(|rule| {
        rule.axis() == scene::Axis::Vertical && rule.rect() == horizontal_column_rule
    }));
    assert!(diagonally_scrolled.scene().rules().iter().any(|rule| {
        rule.axis() == scene::Axis::Horizontal
            && rule.rect() == translate(first_row_rule, body_delta.x(), body_delta.y())
    }));
    assert!(
        diagonally_scrolled
            .scene()
            .texts()
            .iter()
            .any(|text| { text.rect() == translate(detail_text, body_delta.x(), body_delta.y()) })
    );

    let detail_track = scrolled
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track
                .column_identity()
                .is_some_and(|cell| cell.column() == interaction::Id::new("detail"))
        })
        .expect("visible detail divider");
    let resize_start = frame_point_at(
        detail_track
            .divider_hit_rect()
            .expect("visible detail hit zone"),
    );
    let resize_start = geometry::Point::new(resize_start.x() - 70, resize_start.y());
    let resize_end = geometry::Point::new(resize_start.x() + 20, resize_start.y());
    app.pointer_down_at(window, size, resize_start)
        .expect("scrolled divider should capture");
    app.pointer_move_at(window, size, resize_end)
        .expect("scrolled divider should resize from projected header origin");
    app.pointer_up_at(window, size, resize_end)
        .expect("scrolled divider resize should finish");
    let resized = app
        .show_scene(window, size)
        .expect("resized scrolled table should render");
    let resized_detail = geometry_for(&resized, "detail");
    assert_eq!(resized_detail.0.width(), 140);
    assert_eq!(resized_detail.1.width(), 140);
    assert_eq!(resized_detail.0.right(), resized_detail.2);
    assert_eq!(
        resized_detail.2,
        scrolled_detail.2 - 70 + 20,
        "the semantic resize commit rebases the presented scroll before applying the new width"
    );
}

#[test]
fn stationary_pointer_reprojects_header_hover_in_the_presented_scroll_frame() {
    let mut state = control_gallery::State::default();
    state.show_advanced = false;
    let mut app = control_gallery::app(state);
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(300, 700);
    let initial = app
        .show_scene(window, size)
        .expect("gallery table should render");
    let table_viewport = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.role() == view::Role::Scroll && frame.table_projection().is_some())
        .and_then(layout::Frame::viewport)
        .expect("table should expose horizontal viewport");
    assert!(table_viewport.max_scroll().x() > 0);
    let header = initial
        .layout()
        .frames()
        .iter()
        .filter(|frame| frame.table_header_cell().is_some() && frame.target().is_some())
        .filter(|frame| frame.rect().right() <= table_viewport.visible_frame().right())
        .max_by_key(|frame| frame.rect().right())
        .expect("a visible sortable header should exist");
    let point = geometry::Point::new(
        header.rect().right().saturating_sub(8),
        header.rect().y().saturating_add(1),
    );
    let original = header.target().expect("sortable header target").clone();
    let scroll_delta = interaction::ScrollDelta::horizontal(16);
    let horizontal = initial
        .layout()
        .scroll_target_at(point, scroll_delta)
        .expect("header point should resolve the table horizontal viewport");

    app.pointer_move_at(window, size, point)
        .expect("header hover should be handled");
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().hovered()),
        Some(&original)
    );
    let hovered = app
        .show_scene(window, size)
        .expect("header hover should present before scrolling");
    app.handle_input(window, Input::scroll(horizontal, scroll_delta))
        .expect("horizontal table scroll should be handled");

    let skipped = app
        .render_scene(window, size)
        .expect("scrolled candidate should prepare");
    assert!(skipped.property_only());
    assert!(std::sync::Arc::ptr_eq(hovered.commit(), skipped.commit()));
    assert!(skipped.properties().serial() > hovered.properties().serial());
    let baseline_target = skipped
        .layout()
        .hit_test(point)
        .and_then(|hit| hit.target().cloned())
        .expect("retained baseline layout should remain hittable");
    assert_eq!(baseline_target, original);
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().hovered()),
        Some(&original),
        "candidate hover paints locally but remains uncommitted"
    );
    app.finish_render_report(
        window,
        skipped.epoch(),
        skipped.invalidation(),
        skipped.layout(),
        skipped.stack(),
        skipped.property_only(),
        diagnostics::RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now())
            .with_presented(false),
    );
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().hovered()),
        Some(&original),
        "skipped geometry must not leak into retained hover"
    );

    let shown = app
        .show_scene(window, size)
        .expect("scrolled frame should retry and present");
    assert!(shown.property_only());
    assert!(std::sync::Arc::ptr_eq(hovered.commit(), shown.commit()));
    let shown_target = app
        .hit_test(window, size, point)
        .and_then(|hit| hit.target().cloned())
        .expect("presented header target");
    assert_ne!(shown_target, original);
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().hovered()),
        Some(&shown_target)
    );
}

#[test]
fn stationary_pointer_transfers_hover_as_virtual_table_rows_scroll_beneath_it() {
    let mut state = control_gallery::State::default();
    state.show_advanced = false;
    let mut app = control_gallery::app(state);
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 700);
    let initial = app
        .show_scene(window, size)
        .expect("gallery table should render");
    let cell = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(1)
                    && cell.column() == interaction::Id::new("record")
            })
        })
        .expect("second visible record cell");
    let point = frame_point_at(cell.rect());
    let original = initial
        .layout()
        .hit_test(point)
        .and_then(|hit| hit.target().cloned())
        .expect("record cell should be interactive");
    app.pointer_move_at(window, size, point)
        .expect("record hover should be handled");
    let hovered = app
        .show_scene(window, size)
        .expect("record hover should present before scrolling");

    app.scroll_at(window, size, point, interaction::ScrollDelta::vertical(48))
        .expect("table body should scroll beneath the stationary point");
    let shown = app
        .show_scene(window, size)
        .expect("scrolled table should present");
    assert!(shown.property_only());
    assert!(std::sync::Arc::ptr_eq(hovered.commit(), shown.commit()));
    let projected = app
        .hit_test(window, size, point)
        .and_then(|hit| hit.target().cloned())
        .expect("stationary point should hit a replacement row");

    assert_ne!(projected, original);
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().hovered()),
        Some(&projected)
    );
}

#[test]
fn sticky_header_keeps_stationary_hover_while_only_the_body_scrolls() {
    let mut state = control_gallery::State::default();
    state.show_advanced = false;
    let mut app = control_gallery::app(state);
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 700);
    let initial = app
        .show_scene(window, size)
        .expect("gallery table should render");
    let header = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_header_cell().is_some_and(|cell| {
                cell.column() == interaction::Id::new("count") && frame.target().is_some()
            })
        })
        .expect("Count header should be sortable");
    let point = frame_point_at(header.rect());
    let target = header.target().expect("Count header target").clone();
    let body = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(1)
                    && cell.column() == interaction::Id::new("count")
            })
        })
        .expect("Count body cell");
    let vertical = initial
        .layout()
        .scroll_target_at(
            frame_point_at(body.rect()),
            interaction::ScrollDelta::vertical(48),
        )
        .expect("body should resolve its vertical viewport");

    app.pointer_move_at(window, size, point)
        .expect("Count header hover should be handled");
    app.handle_input(
        window,
        Input::scroll(vertical, interaction::ScrollDelta::vertical(48)),
    )
    .expect("body should scroll without moving the pointer");
    let shown = app
        .show_scene(window, size)
        .expect("scrolled body should present");
    let shown_header = shown
        .layout()
        .hit_test(point)
        .expect("sticky header should remain under the pointer");

    assert_eq!(shown_header.target(), Some(&target));
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().hovered()),
        Some(&target)
    );
    assert!(shown.scene().quads().iter().any(|quad| {
        quad.rect() == shown_header.frame().rect()
            && quad.fill() == Theme::default().table().header_hover_tint
    }));
}

#[test]
fn table_keyboard_navigation_reveals_current_cell_across_horizontal_overflow() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Keyboard reveal table"));
        })
        .view(|_, _| {
            widget::view_node(
                crate::Table::new(
                    "keyboard.reveal.table",
                    20,
                    [
                        crate::table::Column::new("name", "Name", view::Dimension::fixed(100)),
                        crate::table::Column::new("detail", "Detail", view::Dimension::fixed(120)),
                        crate::table::Column::new("action", "Action", view::Dimension::fixed(90)),
                    ],
                    MillionTableProvider {
                        cell_calls: Rc::new(Cell::new(0)),
                    },
                )
                .height(view::Dimension::fixed(108)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 108);
    let initial = app
        .show_scene(window, size)
        .expect("wide table should render");
    let name = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(0)
                    && cell.column() == interaction::Id::new("name")
            })
        })
        .expect("first name cell");
    app.pointer_down_at(window, size, frame_point_at(name.rect()))
        .expect("cell click should establish grid focus");
    app.handle_input(
        window,
        Input::key_down(input::Key::End, input::Modifiers::default()),
    )
    .expect("End should move to the row's final cell");
    let revealed = app
        .show_scene(window, size)
        .expect("final cell should be revealed");
    let horizontal = revealed
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_projection().is_some())
        .and_then(layout::Frame::viewport)
        .expect("table horizontal viewport");
    assert_eq!(
        horizontal.resolved_scroll().x(),
        horizontal.max_scroll().x()
    );
    assert!(revealed.layout().frames().iter().any(|frame| {
        frame.is_active_item()
            && frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(0)
                    && cell.column() == interaction::Id::new("action")
            })
    }));
    let table_target = revealed
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_projection().is_some())
        .and_then(layout::Frame::target)
        .expect("table viewport should expose its shared target")
        .clone();
    app.handle_input(
        window,
        Input::key_down(
            input::Key::End,
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("Ctrl+End should reveal the far row and column through one target");
    let diagonal = app
        .show_scene(window, size)
        .expect("far table cell should materialize and reveal");
    let maximum = diagonal
        .layout()
        .scroll_projections()
        .iter()
        .filter(|projection| projection.target() == &table_target)
        .map(|projection| projection.viewport().max_scroll())
        .fold(
            interaction::ScrollOffset::default(),
            |maximum, candidate| {
                interaction::ScrollOffset::new(
                    maximum.x().max(candidate.x()),
                    maximum.y().max(candidate.y()),
                )
            },
        );
    assert!(maximum.x() > 0 && maximum.y() > 0);
    let scroll = app
        .session()
        .interaction(window)
        .expect("table interaction")
        .scroll();
    assert_eq!(scroll.desired_offset(&table_target), maximum);
    assert_eq!(
        scroll.offset(&table_target),
        maximum,
        "horizontal and vertical reveal projections must combine before admission"
    );
    assert!(diagonal.layout().frames().iter().any(|frame| {
        frame.is_active_item()
            && frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(999_999)
                    && cell.column() == interaction::Id::new("action")
            })
    }));

    app.handle_input(
        window,
        Input::key_down(
            input::Key::Home,
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("Ctrl+Home should move to the table's first cell");
    let returned = app
        .show_scene(window, size)
        .expect("first cell should be revealed");
    let horizontal = returned
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_projection().is_some())
        .and_then(layout::Frame::viewport)
        .expect("table horizontal viewport");
    assert_eq!(horizontal.resolved_scroll().x(), 0);
}

#[test]
fn table_gutter_scrollbar_and_body_clip_share_visible_viewport_geometry() {
    let view = widget::view(|ui| {
        ui.add(
            crate::Table::new(
                "gutter.records",
                20,
                [
                    crate::table::Column::new("name", "Name", view::Dimension::fixed(160)),
                    crate::table::Column::new("detail", "Detail", view::Dimension::fixed(150)),
                ],
                MillionTableProvider {
                    cell_calls: Rc::new(Cell::new(0)),
                },
            )
            .height(view::Dimension::fixed(108)),
        );
    });
    let mut theme = Theme::dark();
    theme.scrollbar_mut().metrics.policy = crate::theme::ScrollbarPolicy::GutterAlways;
    let mut engine = layout::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(240, 108),
        &mut engine,
        &theme,
    );
    let body = layout
        .frames()
        .iter()
        .find(|frame| frame.role() == view::Role::VirtualList)
        .expect("table body frame");
    let viewport = body.viewport().expect("table body viewport");
    let track = layout
        .chrome()
        .iter()
        .find_map(|chrome| (chrome.viewport() == viewport).then(|| chrome.track()))
        .expect("table body gutter scrollbar");

    assert_eq!(
        viewport.visible_content().right(),
        viewport
            .visible_frame()
            .right()
            .saturating_sub(theme.scrollbar().metrics.thickness)
    );
    assert_eq!(
        track.right(),
        viewport
            .visible_frame()
            .right()
            .saturating_sub(theme.scrollbar().appearance.margin)
    );
    let body_id = body
        .target()
        .and_then(interaction::Target::element_id)
        .expect("table body target identity");
    assert_eq!(
        layout.virtual_list_page(body_id, 20),
        Some((viewport.visible_content().height().max(1) as usize / 20).max(1))
    );
}

#[test]
fn expanded_table_rows_measure_intrinsic_content_at_resolved_track_widths() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Expanded track measurement"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::Table::new(
                    "expanded.measurement",
                    24,
                    [
                        crate::table::Column::new("name", "Name", view::Dimension::fixed(80)),
                        crate::table::Column::new("detail", "Detail", view::Dimension::fixed(100)),
                    ],
                    WrappedTableProvider,
                )
                .presentation(crate::table::Presentation::Expanded)
                .width(view::Dimension::fixed(180))
                .height(view::Dimension::fixed(180)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(180, 180);
    let rendered = app
        .show_scene(window, size)
        .expect("expanded table should render");
    let rows = rendered
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| frame.table_row().map(|row| (row.index(), frame.rect())))
        .collect::<Vec<_>>();

    let initial_heights = rows
        .iter()
        .map(|(index, rect)| (*index, rect.height()))
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(
        initial_heights[&0], 24,
        "short expanded rows keep the configured floor when interface text needs no more space"
    );
    assert!(initial_heights[&1] > initial_heights[&0]);
    for frame in rendered.layout().frames().iter().filter(|frame| {
        frame
            .table_cell()
            .is_some_and(|cell| cell.column() == interaction::Id::new("detail"))
    }) {
        assert_eq!(frame.rect().width(), 100);
    }

    let detail_track = rendered
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track
                .column_identity()
                .is_some_and(|cell| cell.column() == interaction::Id::new("detail"))
        })
        .expect("detail column track");
    let resize_start = frame_point_at(
        detail_track
            .divider_hit_rect()
            .expect("detail resize hit zone"),
    );
    let resize_end = geometry::Point::new(220, resize_start.y());
    app.pointer_down_at(window, size, resize_start)
        .expect("detail resize should capture");
    app.pointer_move_at(window, size, resize_end)
        .expect("detail resize should update the session override");
    app.pointer_up_at(window, size, resize_end)
        .expect("detail resize should finish");
    let resized = app
        .show_scene(window, size)
        .expect("resized expanded table should render");
    let resized_rows = resized
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| {
            frame
                .table_row()
                .map(|row| (row.index(), frame.rect().height()))
        })
        .collect::<Vec<_>>();
    let resized_detail_widths = resized
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| {
            frame.table_cell().and_then(|cell| {
                (cell.column() == interaction::Id::new("detail")).then_some(frame.rect().width())
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(resized_rows[0], (0, 24));
    assert!(resized_rows[1].1 < initial_heights[&1]);
    assert_eq!(resized_detail_widths, vec![140, 140]);

    app.scroll_at(
        window,
        size,
        geometry::Point::new(40, 60),
        interaction::ScrollDelta::horizontal(40),
    )
    .expect("expanded table should scroll horizontally");
    let scrolled = app
        .show_scene(window, size)
        .expect("scrolled expanded table should render");
    assert_eq!(
        scrolled
            .layout()
            .frames()
            .iter()
            .filter_map(|frame| {
                frame
                    .table_row()
                    .map(|row| (row.index(), frame.rect().height()))
            })
            .collect::<Vec<_>>(),
        resized_rows
    );
}

#[test]
fn table_header_stays_fixed_while_keyed_rows_scroll_reorder_and_shrink() {
    let keys = Rc::new(RefCell::new((0..100).collect::<Vec<_>>()));
    let provider = MutableTableProvider {
        keys: Rc::clone(&keys),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Mutable table"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::Table::new(
                    "mutable.table",
                    20,
                    [
                        crate::table::Column::new("first", "First", view::Dimension::fixed(90)),
                        crate::table::Column::new("second", "Second", view::Dimension::weight(1)),
                    ],
                    provider.clone(),
                )
                .height(view::Dimension::fixed(128)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(260, 128);
    let initial = app
        .show_scene(window, size)
        .expect("mutable table should render");
    let header_y = initial
        .layout()
        .frames()
        .iter()
        .find_map(|frame| frame.table_header_cell().map(|_| frame.rect().y()))
        .expect("header should render");
    let list = initial.layout().find_role(view::Role::VirtualList)[0];

    app.scroll_at(
        window,
        size,
        frame_point_at(list.rect()),
        interaction::ScrollDelta::vertical(400),
    )
    .expect("body scroll should be handled");
    let scrolled = app
        .show_scene(window, size)
        .expect("scrolled table should render");
    assert!(
        scrolled
            .scene()
            .texts()
            .iter()
            .any(|text| text.value() == "first 20")
    );
    assert!(
        scrolled
            .layout()
            .frames()
            .iter()
            .filter_map(|frame| frame.table_header_cell().map(|_| frame.rect().y()))
            .all(|y| y == header_y)
    );

    keys.borrow_mut()[18..27].reverse();
    app.request_redraw(window);
    app.show_scene(window, size)
        .expect("reordered table should render");
    assert!(
        app.composition(window)
            .expect("table composition should remain installed")
            .changes()
            .is_empty(),
        "row and cell identity should follow provider keys through reorder"
    );

    keys.borrow_mut().truncate(3);
    app.request_redraw(window);
    let shrunk = app
        .show_scene(window, size)
        .expect("shrunk table should render");
    let visible_rows = shrunk
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| frame.table_row())
        .collect::<Vec<_>>();
    assert_eq!(visible_rows.len(), 3);
    assert!(
        visible_rows
            .iter()
            .all(|row| row.table() == interaction::Id::new("mutable.table"))
    );
    assert_eq!(visible_rows[0].key(), crate::virtual_list::Key::new(0));
}

#[test]
fn table_column_resize_uses_capture_and_stays_window_local() {
    let keys = Rc::new(RefCell::new((0..20).collect::<Vec<_>>()));
    let provider = MutableTableProvider { keys };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Table width one"));
            cx.open_window(window::Options::new("Table width two"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::Table::new(
                    "resizable.table",
                    20,
                    [
                        crate::table::Column::new("first", "First", view::Dimension::fixed(80)),
                        crate::table::Column::new("second", "Second", view::Dimension::weight(1)),
                    ],
                    provider.clone(),
                )
                .height(view::Dimension::fixed(108)),
            )
        });
    app.start();
    let first_window = app.session().windows()[0].id();
    let second_window = app.session().windows()[1].id();
    let size = geometry::Size::new(260, 108);
    let initial = app
        .show_scene(first_window, size)
        .expect("first table should render");
    let table_rect = initial.layout().find_role(view::Role::Table)[0].rect();
    let final_track = initial
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track
                .column_identity()
                .is_some_and(|column| column.column() == interaction::Id::new("second"))
        })
        .expect("final column should expose a resize track");
    let final_hit_rect = final_track
        .divider_hit_rect()
        .expect("final resize track hit zone");
    assert_eq!(final_hit_rect.width(), crate::table::DIVIDER_HIT_WIDTH);
    assert_eq!(final_hit_rect.right(), table_rect.right());
    assert_eq!(final_track.boundary(), table_rect.right());
    assert_eq!(
        initial
            .layout()
            .hit_test(frame_point_at(final_hit_rect))
            .and_then(|hit| hit.target().map(interaction::Target::kind)),
        Some(interaction::Kind::TableDivider)
    );
    let divider = initial
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track
                .column_identity()
                .is_some_and(|column| column.column() == interaction::Id::new("first"))
        })
        .expect("first column should expose a resize track");
    let first_header_x = initial
        .layout()
        .frames()
        .iter()
        .find_map(|frame| {
            frame
                .table_header_cell()
                .filter(|cell| cell.column() == interaction::Id::new("first"))
                .map(|_| frame.rect().x())
        })
        .expect("first header should render");
    let initial_rule_x = divider.boundary();
    let divider_rect = divider.divider_hit_rect().expect("resize track hit zone");
    assert_eq!(frame_point_at(divider_rect).x(), initial_rule_x);
    let start = frame_point_at(divider_rect);
    let dragged = geometry::Point::new(start.x().saturating_add(44), start.y());
    let expected_width = dragged.x().saturating_sub(first_header_x);

    app.pointer_move_at(first_window, size, start)
        .expect("divider hover should be handled");
    assert_eq!(
        app.session().window(first_window).expect("window").cursor(),
        crate::pointer::Cursor::ResizeHorizontal
    );
    app.pointer_down_at(first_window, size, start)
        .expect("divider press should capture");
    app.pointer_move_at(first_window, size, dragged)
        .expect("captured divider should resize beyond its old bounds");
    assert_eq!(
        app.session().window(first_window).expect("window").cursor(),
        crate::pointer::Cursor::ResizeHorizontal,
        "capture preserves the resolved divider cursor outside its old hit zone"
    );
    app.pointer_up_at(first_window, size, dragged)
        .expect("divider release should finish resize");

    let resized = app
        .show_scene(first_window, size)
        .expect("resized table should render");
    let untouched = app
        .show_scene(second_window, size)
        .expect("second table should render independently");
    let width_for =
        |rendered: &scene::Presentation, table: interaction::Id, column: interaction::Id| {
            rendered
                .layout()
                .frames()
                .iter()
                .find_map(|frame| {
                    frame
                        .table_header_cell()
                        .filter(|cell| cell.table() == table && cell.column() == column)
                        .map(|_| frame.rect().width())
                })
                .expect("requested table header should render")
        };
    assert_eq!(
        width_for(
            &resized,
            interaction::Id::new("resizable.table"),
            interaction::Id::new("first")
        ),
        expected_width
    );
    assert_eq!(
        width_for(
            &untouched,
            interaction::Id::new("resizable.table"),
            interaction::Id::new("first")
        ),
        80
    );
    assert!(resized.layout().frames().iter().any(|frame| {
        frame.table_cell().is_some_and(|cell| {
            cell.column() == interaction::Id::new("first") && frame.rect().width() == expected_width
        })
    }));
    let resized_rule_x = resized
        .layout()
        .table_tracks()
        .iter()
        .find_map(|track| {
            track
                .column_identity()
                .filter(|column| column.column() == interaction::Id::new("first"))
                .map(|_| track.boundary())
        })
        .expect("resized column track");
    assert_eq!(resized_rule_x.saturating_sub(initial_rule_x), 44);

    let resized_track = resized
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track
                .column_identity()
                .is_some_and(|column| column.column() == interaction::Id::new("first"))
        })
        .expect("resized column track");
    let shrink_start = frame_point_at(
        resized_track
            .divider_hit_rect()
            .expect("resized track hit zone"),
    );
    let shrink_to = geometry::Point::new(first_header_x.saturating_sub(100), shrink_start.y());
    app.pointer_down_at(first_window, size, shrink_start)
        .expect("resized track should capture again");
    app.pointer_move_at(first_window, size, shrink_to)
        .expect("captured track should clamp at its minimum");
    app.pointer_up_at(first_window, size, shrink_to)
        .expect("minimum-width resize should finish");
    let minimized = app
        .show_scene(first_window, size)
        .expect("minimum-width table should render");
    assert_eq!(
        width_for(
            &minimized,
            interaction::Id::new("resizable.table"),
            interaction::Id::new("first")
        ),
        crate::table::COLUMN_MIN_WIDTH
    );
}

#[test]
fn held_count_enabled_boundary_moves_with_the_pointer_without_reallocating_other_tracks() {
    fn header_rect(rendered: &scene::Presentation, column: &'static str) -> geometry::Rect {
        rendered
            .layout()
            .frames()
            .iter()
            .find_map(|frame| {
                frame
                    .table_header_cell()
                    .filter(|cell| cell.column() == interaction::Id::new(column))
                    .map(|_| frame.rect())
            })
            .expect("requested header track")
    }

    fn body_rect(rendered: &scene::Presentation, column: &'static str) -> geometry::Rect {
        rendered
            .layout()
            .frames()
            .iter()
            .find_map(|frame| {
                frame
                    .table_cell()
                    .filter(|cell| {
                        cell.row() == crate::virtual_list::Key::new(0)
                            && cell.column() == interaction::Id::new(column)
                    })
                    .map(|_| frame.rect())
            })
            .expect("requested first-row cell")
    }

    fn column_track<'a>(
        rendered: &'a scene::Presentation,
        column: &'static str,
    ) -> &'a layout::table::Track {
        rendered
            .layout()
            .table_tracks()
            .iter()
            .find(|track| {
                track
                    .column_identity()
                    .is_some_and(|cell| cell.column() == interaction::Id::new(column))
            })
            .expect("requested column boundary")
    }

    fn table_extent(rendered: &scene::Presentation) -> (i32, i32) {
        rendered
            .layout()
            .frames()
            .iter()
            .find_map(|frame| {
                let projection = frame.table_projection()?;
                let viewport = frame.viewport()?;
                Some((projection.content_width(), viewport.max_scroll().x()))
            })
            .expect("table scroll projection")
    }

    let mut state = control_gallery::State::default();
    state.show_advanced = false;
    let mut app = control_gallery::app(state);
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 700);
    let initial = app
        .show_scene(window, size)
        .expect("gallery table before held-boundary reduction");
    let columns = ["record", "detail", "note", "count", "enabled", "action"];
    let initial_headers = columns
        .iter()
        .map(|column| (*column, header_rect(&initial, column)))
        .collect::<Vec<_>>();
    let initial_bodies = columns
        .iter()
        .map(|column| (*column, body_rect(&initial, column)))
        .collect::<Vec<_>>();
    let initial_extent = table_extent(&initial);
    let count_track = column_track(&initial, "count");
    let start = frame_point_at(
        count_track
            .divider_hit_rect()
            .expect("Count/Enabled resize zone"),
    );
    assert_eq!(start.x(), count_track.boundary());
    drop(initial);

    let before_drag = app
        .diagnostics(window)
        .expect("table diagnostics")
        .pipeline
        .clone();
    let before_drag_frame = app
        .diagnostics(window)
        .expect("table diagnostics")
        .frame
        .clone();
    app.pointer_down_at(window, size, start)
        .expect("Count/Enabled boundary should capture");
    assert_eq!(
        app.diagnostics(window)
            .expect("table diagnostics")
            .pipeline
            .routing_layouts,
        before_drag.routing_layouts
    );
    for delta in [9, 18, 27, 36] {
        let pointer = geometry::Point::new(start.x() + delta, start.y());
        app.pointer_move_at(window, size, pointer)
            .expect("held boundary should follow the drag");
        if delta == 9 {
            let before_present = &app.diagnostics(window).expect("table diagnostics").pipeline;
            assert_eq!(before_present.routing_layouts, before_drag.routing_layouts);
            assert_eq!(before_present.frames_prepared, before_drag.frames_prepared);
        }
        let rendered = app
            .show_scene(window, size)
            .expect("held-boundary projection");
        let diagnostics = app.diagnostics(window).expect("table diagnostics");
        assert_eq!(
            diagnostics.frame.view_rebuilds, before_drag_frame.view_rebuilds,
            "divider movement projects session widths without rebuilding the application view"
        );
        assert_eq!(
            diagnostics.frame.layout_recomposes,
            before_drag_frame.layout_recomposes + (delta / 9) as usize,
            "each selected presentation width composes once"
        );
        let count = column_track(&rendered, "count");
        assert_eq!(count.boundary(), pointer.x());
        assert_eq!(
            frame_point_at(count.divider_hit_rect().expect("moved resize zone")).x(),
            pointer.x()
        );
        assert_eq!(
            count.rule_rect().x() + count.rule_rect().width() / 2,
            pointer.x()
        );
        let projected_divider = rendered
            .layout()
            .hit_test(pointer)
            .and_then(|hit| hit.target().cloned())
            .expect("moved divider should remain under the pointer");
        assert_eq!(projected_divider.kind(), interaction::Kind::TableDivider);
        assert_eq!(
            app.session()
                .interaction(window)
                .and_then(|interaction| interaction.pointer().hovered()),
            Some(&projected_divider),
            "successful resize frames must not leave hover on old boundary geometry"
        );

        for column in ["record", "detail", "note"] {
            let before = initial_headers
                .iter()
                .find(|(candidate, _)| *candidate == column)
                .expect("left header baseline")
                .1;
            assert_eq!(header_rect(&rendered, column), before);
            let before = initial_bodies
                .iter()
                .find(|(candidate, _)| *candidate == column)
                .expect("left body baseline")
                .1;
            assert_eq!(body_rect(&rendered, column), before);
        }

        let initial_count = initial_headers
            .iter()
            .find(|(column, _)| *column == "count")
            .expect("Count baseline")
            .1;
        let resized_count = header_rect(&rendered, "count");
        assert_eq!(resized_count.x(), initial_count.x());
        assert_eq!(resized_count.width(), initial_count.width() + delta);
        let count_body = body_rect(&rendered, "count");
        assert_eq!(count_body.x(), resized_count.x());
        assert_eq!(count_body.width(), resized_count.width());

        for column in ["enabled", "action"] {
            let before = initial_headers
                .iter()
                .find(|(candidate, _)| *candidate == column)
                .expect("right header baseline")
                .1;
            let after = header_rect(&rendered, column);
            assert_eq!(after.x(), before.x() + delta);
            assert_eq!(after.width(), before.width());
            let body = body_rect(&rendered, column);
            assert_eq!(body.x(), after.x());
            assert_eq!(body.width(), after.width());
        }

        let extent = table_extent(&rendered);
        assert_eq!(extent.0, initial_extent.0 + delta);
        assert_eq!(extent.1, initial_extent.1 + delta);
    }
    let end = geometry::Point::new(start.x() + 36, start.y());
    app.pointer_up_at(window, size, end)
        .expect("held boundary should release");
    let settled = app.show_scene(window, size).expect("settled manual width");
    let settled_count = header_rect(&settled, "count");
    let settled_detail = header_rect(&settled, "detail");
    let settled_note = header_rect(&settled, "note");
    let wider = app
        .show_scene(
            window,
            geometry::Size::new(size.width() + 40, size.height()),
        )
        .expect("wider viewport should re-resolve live weights");
    assert_eq!(header_rect(&wider, "count").width(), settled_count.width());
    assert!(header_rect(&wider, "detail").width() > settled_detail.width());
    assert!(header_rect(&wider, "note").width() > settled_note.width());
}

#[test]
fn one_hundred_divider_positions_coalesce_into_one_layout_at_the_latest_width() {
    let mut state = control_gallery::State::default();
    state.show_advanced = false;
    let mut app = control_gallery::app(state);
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 700);
    let initial = app
        .show_scene(window, size)
        .expect("gallery table should render");
    let count = initial
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track
                .column_identity()
                .is_some_and(|cell| cell.column() == interaction::Id::new("count"))
        })
        .expect("Count track");
    let start = frame_point_at(count.divider_hit_rect().expect("Count resize zone"));
    let before = app.diagnostics(window).expect("table diagnostics").clone();
    drop(initial);

    app.pointer_down_at(window, size, start)
        .expect("Count divider should capture");
    for delta in 1..=100 {
        app.pointer_move_at(
            window,
            size,
            geometry::Point::new(start.x() + delta, start.y()),
        )
        .expect("raw divider position should update session truth");
    }

    let pending = app.diagnostics(window).expect("table diagnostics");
    assert_eq!(pending.frame.view_rebuilds, before.frame.view_rebuilds);
    assert_eq!(
        pending.frame.layout_recomposes,
        before.frame.layout_recomposes
    );
    assert_eq!(
        pending.pipeline.frames_prepared,
        before.pipeline.frames_prepared
    );

    let shown = app
        .show_scene(window, size)
        .expect("latest divider width should present once");
    let count = shown
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track
                .column_identity()
                .is_some_and(|cell| cell.column() == interaction::Id::new("count"))
        })
        .expect("resized Count track");
    assert_eq!(count.boundary(), start.x() + 100);
    let after = app.diagnostics(window).expect("table diagnostics");
    assert_eq!(after.frame.view_rebuilds, before.frame.view_rebuilds);
    assert_eq!(
        after.frame.layout_recomposes,
        before.frame.layout_recomposes + 1
    );
    assert_eq!(
        after.pipeline.frames_prepared,
        before.pipeline.frames_prepared + 1
    );
}

#[test]
fn table_column_resize_stays_table_local_in_one_window() {
    let keys = Rc::new(RefCell::new((0..20).collect::<Vec<_>>()));
    let provider = MutableTableProvider { keys };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Independent tables"));
        })
        .view(move |_, _| {
            let table = |id: &'static str| {
                widget::Widget::into_node(
                    crate::Table::new(
                        id,
                        20,
                        [
                            crate::table::Column::new("first", "First", view::Dimension::fixed(80)),
                            crate::table::Column::new(
                                "second",
                                "Second",
                                view::Dimension::weight(1),
                            ),
                        ],
                        provider.clone(),
                    )
                    .height(view::Dimension::fixed(108)),
                )
            };
            View::new(
                view::Node::root().child(
                    view::Node::stack(view::Axis::Vertical)
                        .child(table("table.one"))
                        .child(table("table.two")),
                ),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(260, 216);
    let initial = app.show_scene(window, size).expect("tables should render");
    let divider = initial
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track.column_identity().is_some_and(|column| {
                column.table() == interaction::Id::new("table.one")
                    && column.column() == interaction::Id::new("first")
            })
        })
        .expect("first table resize track should render");
    let start = frame_point_at(divider.divider_hit_rect().expect("resize track hit zone"));
    let dragged = geometry::Point::new(start.x() + 30, start.y());
    app.pointer_down_at(window, size, start)
        .expect("first table divider should capture");
    app.pointer_move_at(window, size, dragged)
        .expect("first table divider should resize");
    app.pointer_up_at(window, size, dragged)
        .expect("resize should finish");
    let resized = app
        .show_scene(window, size)
        .expect("independent tables should rerender");
    let widths = resized
        .layout()
        .frames()
        .iter()
        .filter_map(|frame| {
            frame
                .table_header_cell()
                .filter(|cell| cell.column() == interaction::Id::new("first"))
                .map(|cell| (cell.table(), frame.rect().width()))
        })
        .collect::<std::collections::HashMap<_, _>>();
    assert!(widths[&interaction::Id::new("table.one")] > 80);
    assert_eq!(widths[&interaction::Id::new("table.two")], 80);
}

#[test]
fn removing_a_column_releases_its_synthetic_resize_capture() {
    let keys = Rc::new(RefCell::new((0..20).collect::<Vec<_>>()));
    let provider = MutableTableProvider { keys };
    let show_first = Rc::new(RefCell::new(true));
    let view_show_first = Rc::clone(&show_first);
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Removable table column"));
        })
        .view(move |_, _| {
            let mut columns = vec![crate::table::Column::new(
                "second",
                "Second",
                view::Dimension::weight(1),
            )];
            if *view_show_first.borrow() {
                columns.insert(
                    0,
                    crate::table::Column::new("first", "First", view::Dimension::fixed(80)),
                );
            }
            widget::view_node(
                crate::Table::new("removable.table", 20, columns, provider.clone())
                    .height(view::Dimension::fixed(108)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(260, 108);
    let initial = app
        .show_scene(window, size)
        .expect("removable table should render");
    let first_track = initial
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track
                .column_identity()
                .is_some_and(|column| column.column() == interaction::Id::new("first"))
        })
        .expect("removable column track");
    let point = frame_point_at(
        first_track
            .divider_hit_rect()
            .expect("removable column hit zone"),
    );
    app.pointer_down_at(window, size, point)
        .expect("column resize should capture");
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .is_some()
    );

    *show_first.borrow_mut() = false;
    app.request_redraw(window);
    app.show_scene(window, size)
        .expect("table should rebuild without the captured column");
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .is_none(),
        "removing the header node must release synthetic divider capture"
    );
}

#[test]
fn table_keyboard_tracks_a_keyed_logical_row_and_column_without_scanning() {
    let cell_calls = Rc::new(Cell::new(0));
    let provider = MillionTableProvider {
        cell_calls: Rc::clone(&cell_calls),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Keyboard table"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::Table::new(
                    "keyboard.table",
                    20,
                    [
                        crate::table::Column::new("name", "Name", view::Dimension::fixed(80)),
                        crate::table::Column::new("detail", "Detail", view::Dimension::fixed(100)),
                        crate::table::Column::new("action", "Action", view::Dimension::weight(1)),
                    ],
                    provider.clone(),
                )
                .height(view::Dimension::fixed(128)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let table = interaction::Id::new("keyboard.table");
    let size = geometry::Size::new(320, 128);
    let initial = app
        .show_scene(window, size)
        .expect("keyboard table should render");
    let detail_row_one = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(1)
                    && cell.column() == interaction::Id::new("detail")
            })
        })
        .expect("second-row detail cell should render");
    app.pointer_down_at(window, size, frame_point_at(detail_row_one.rect()))
        .expect("cell click should choose row and column");
    assert_eq!(
        app.session().active_table_cell(window, table),
        Some(crate::table::Cell::new(
            table,
            crate::virtual_list::Key::new(1),
            interaction::Id::new("detail")
        ))
    );

    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowRight, input::Modifiers::default()),
    )
    .expect("Right should move across declared columns");
    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
    )
    .expect("Down should reuse keyed row movement");
    assert_eq!(
        app.session().active_table_cell(window, table),
        Some(crate::table::Cell::new(
            table,
            crate::virtual_list::Key::new(2),
            interaction::Id::new("action")
        ))
    );

    let calls_before_end = cell_calls.get();
    app.handle_input(
        window,
        Input::key_down(
            input::Key::End,
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("Ctrl+End should move to the final logical row and column");
    let moved = app
        .show_scene(window, size)
        .expect("distant active table cell should materialize");
    assert_eq!(
        app.session().active_table_cell(window, table),
        Some(crate::table::Cell::new(
            table,
            crate::virtual_list::Key::new(999_999),
            interaction::Id::new("action")
        ))
    );
    assert!(moved.layout().frames().iter().any(|frame| {
        frame.is_active_item()
            && frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(999_999)
                    && cell.column() == interaction::Id::new("action")
            })
    }));
    let distant_cell_calls = cell_calls.get().saturating_sub(calls_before_end);
    assert!(
        distant_cell_calls <= 51,
        "logical navigation may materialize two eight-row, three-column windows plus one three-cell reveal pin: {distant_cell_calls} cell calls"
    );
}

#[test]
fn editable_table_text_and_number_cells_commit_reject_and_cancel_by_cell_identity() {
    let mut app = editable_table_app(EditableTableState {
        records: vec![EditableRecord {
            key: 7,
            name: String::new(),
            count: 4,
        }],
    });
    app.start();
    let window = app.session().windows()[0].id();
    let other_window = app.session().windows()[1].id();
    let size = geometry::Size::new(320, 124);
    app.show_scene(window, size)
        .expect("editable table should render");
    let name = crate::table::Cell::new(
        interaction::Id::new("editable.table"),
        crate::virtual_list::Key::new(7),
        interaction::Id::new("name"),
    );
    let count = crate::table::Cell::new(
        interaction::Id::new("editable.table"),
        crate::virtual_list::Key::new(7),
        interaction::Id::new("count"),
    );

    app.handle_input(window, Input::focus(session::Focus::table_cell(name)))
        .expect("text cell should focus");
    app.handle_input(
        window,
        Input::key_down(input::Key::F2, input::Modifiers::default()),
    )
    .expect("F2 should deliberately enter the text cell");
    app.handle_input(window, Input::text_commit("Ada"))
        .expect("text cell should accept a draft");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("Enter should commit the typed text command");
    assert_eq!(app.state().records[0].name, "Ada");

    app.handle_input(window, Input::focus(session::Focus::table_cell(count)))
        .expect("numeric cell should focus");
    app.handle_input(
        window,
        Input::key_down(input::Key::F2, input::Modifiers::default()),
    )
    .expect("F2 should deliberately enter the numeric cell");
    app.handle_input(window, Input::text_commit("2"))
        .expect("numeric cell should accept draft text");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("Enter should parse and commit the typed numeric command");
    assert_eq!(app.state().records[0].count, 42);
    assert!(app.undo());
    assert_eq!(app.state().records[0].count, 4);
    assert!(app.redo());
    assert_eq!(app.state().records[0].count, 42);

    app.handle_input(
        window,
        Input::key_down(input::Key::F2, input::Modifiers::default()),
    )
    .expect("F2 should re-enter the committed numeric cell");
    app.handle_input(
        window,
        Input::text_selection(text::selection::Operation::SelectAll),
    )
    .expect("numeric draft should select for replacement");
    app.handle_input(window, Input::text_commit("-"))
        .expect("invalid numeric draft should remain editable");
    let rejected_outcome = app
        .handle_input(
            window,
            Input::key_down(input::Key::Enter, input::Modifiers::default()),
        )
        .expect("invalid numeric submit should be handled as rejection");
    assert_eq!(
        rejected_outcome.effect().invalidation(),
        Some(response::effect::Invalidation::Rebuild),
        "creating an anchored panel must rebuild immediately instead of waiting for unrelated input"
    );
    assert_eq!(app.state().records[0].count, 42);
    assert_eq!(
        app.session()
            .text_input_feedback(window, session::Focus::table_cell(count)),
        Some((feedback::Severity::Error, "Enter a whole number"))
    );
    assert_eq!(
        app.session()
            .text_input_feedback(other_window, session::Focus::table_cell(count)),
        None
    );
    assert_ne!(
        text_target(session::Focus::table_cell(count)),
        text_target(session::Focus::table_cell(crate::table::Cell::new(
            interaction::Id::new("other.table"),
            count.row(),
            count.column(),
        )))
    );
    assert!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&session::Focus::table_cell(count)))
    );
    assert_eq!(
        text_draft(&app, window, session::Focus::table_cell(count)).text(),
        "-"
    );
    app.handle_input(window, Input::focus(session::Focus::table_cell(name)))
        .expect("focus movement should apply the chosen commit-before-leave policy");
    assert!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&session::Focus::table_cell(count)))
    );
    let rejected = app
        .show_scene(window, size)
        .expect("rejected editor should retain visible presentation");
    let rejected_field = rejected
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(count))
        .expect("rejected cell input");
    assert!(rejected_field.input_is_invalid());
    assert_eq!(
        rejected_field.input_error_message(),
        Some("Enter a whole number")
    );
    let indicator = rejected_field
        .input_indicator_rect()
        .expect("rejected input owns a trailing indicator");
    assert!(
        rejected
            .scene()
            .icons()
            .iter()
            .any(|icon| { icon.icon().id().as_str() == "x-circle" && icon.rect() == indicator })
    );
    assert!(
        rejected.layout().frames().iter().all(|frame| {
            frame.interaction_id() != Some(interaction::Id::new("feedback.hover"))
        }),
        "validation projects inline and opens no panel until the input is inspected"
    );

    app.handle_input(
        window,
        Input::key_down(input::Key::Backspace, input::Modifiers::default()),
    )
    .expect("correcting a rejected draft should remain editable");
    assert_eq!(
        app.session()
            .text_input_feedback(window, session::Focus::table_cell(count)),
        None,
        "a rejection may not outlive the draft that produced it"
    );
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("corrected draft should commit");
    assert_eq!(app.state().records[0].count, 42);

    app.handle_input(
        window,
        Input::key_down(input::Key::F2, input::Modifiers::default()),
    )
    .expect("F2 should re-enter the corrected numeric cell");
    app.handle_input(
        window,
        Input::text_selection(text::selection::Operation::SelectAll),
    )
    .expect("numeric draft should select for replacement");
    app.handle_input(window, Input::text_commit("-"))
        .expect("invalid numeric draft should remain editable");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("invalid submit should reject again");

    app.handle_input(window, Input::cancel())
        .expect("Escape should cancel the rejected draft");
    assert_eq!(
        app.session()
            .text_input_feedback(window, session::Focus::table_cell(count)),
        None
    );
    let cancelled = app
        .show_scene(window, size)
        .expect("cancelled committed value should render");
    assert!(
        cancelled
            .layout()
            .frames()
            .iter()
            .any(|frame| { frame.table_cell() == Some(count) && frame.label_text() == Some("42") })
    );
}

const LONG_REJECTED_COUNT: &str = "999999999999999999999999999999999999999999999999999999999999";

fn rejected_count_input_app() -> (
    Runtime<EditableTableState, (), View>,
    window::Id,
    geometry::Size,
    crate::table::Cell,
) {
    let mut app = editable_table_app(EditableTableState {
        records: vec![EditableRecord {
            key: 7,
            name: "Ada".to_owned(),
            count: 4,
        }],
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 124);
    let cell = crate::table::Cell::new(
        interaction::Id::new("editable.table"),
        crate::virtual_list::Key::new(7),
        interaction::Id::new("count"),
    );
    let initial = app
        .show_scene(window, size)
        .expect("editable count should materialize");
    let initial_count = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("initial count TextBox");
    assert!(!initial_count.input_is_invalid());
    assert_eq!(initial_count.input_indicator_rect(), None);
    drop(initial);
    app.handle_input(window, Input::focus(session::Focus::table_cell(cell)))
        .expect("count cell should focus");
    app.handle_input(
        window,
        Input::key_down(input::Key::F2, input::Modifiers::default()),
    )
    .expect("count cell should enter text participation");
    app.handle_input(
        window,
        Input::text_selection(text::selection::Operation::SelectAll),
    )
    .expect("count draft should select for replacement");
    app.handle_input(window, Input::text_commit(LONG_REJECTED_COUNT))
        .expect("long integer syntax remains a lawful draft");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("out-of-range integer representation should reject");

    (app, window, size, cell)
}

#[test]
fn rejection_projects_inline_without_opening_a_panel() {
    let (mut app, window, size, cell) = rejected_count_input_app();
    let rejected = app
        .show_scene(window, size)
        .expect("rejected input should render inline");
    let frame = rejected
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("rejected count TextBox");
    let parts = frame
        .input_parts()
        .expect("TextBox should expose one authoritative input decomposition");
    let indicator = parts
        .indicator()
        .expect("rejected TextBox should reserve trailing indicator geometry");

    assert!(frame.input_is_invalid());
    assert_eq!(frame.input_error_message(), Some("Enter a whole number"));
    assert_eq!(parts.text(), frame.text_box_text_rect());
    assert_eq!(Some(indicator), frame.input_indicator_rect());
    assert!(rect_contains(frame.rect(), parts.content()));
    assert!(rect_contains(parts.content(), parts.text()));
    assert!(rect_contains(parts.content(), indicator));
    assert!(parts.text().right() <= indicator.x());
    assert!(
        rejected
            .layout()
            .frames()
            .iter()
            .all(|frame| frame.role() != view::Role::FloatingPanel),
        "rejection alone must not open a panel"
    );
    assert!(rejected.scene().icons().iter().any(|icon| {
        icon.icon().id().as_str() == "x-circle"
            && icon.rect() == indicator
            && icon.color() == Theme::default().auxiliary_panel().error
    }));
    assert!(
        rejected
            .scene()
            .text_viewports()
            .iter()
            .any(|viewport| viewport.rect() == parts.text())
    );
    assert!(
        frame
            .text_caret_rect()
            .is_some_and(|caret| rect_contains(parts.text(), caret))
    );

    assert_eq!(
        rejected
            .layout()
            .hit_test(frame_point_at(parts.text()))
            .and_then(|hit| hit.target().map(interaction::Target::kind)),
        Some(interaction::Kind::TextArea)
    );
    assert_eq!(
        rejected
            .layout()
            .hit_test(frame_point_at(indicator))
            .and_then(|hit| hit.target().map(interaction::Target::kind)),
        Some(interaction::Kind::Indicator)
    );

    for scale in [1.0_f64, 1.25, 1.5, 2.0] {
        let scaled_text_right = f64::from(parts.text().right()) * scale;
        let scaled_indicator_x = f64::from(indicator.x()) * scale;
        assert!(
            scaled_text_right <= scaled_indicator_x,
            "text and indicator must remain disjoint at scale {scale}"
        );
    }
    drop(rejected);

    app.handle_input(
        window,
        Input::text_selection(text::selection::Operation::SelectAll),
    )
    .expect("rejected draft selection remains editable");
    let selected = app
        .show_scene(window, size)
        .expect("selected rejected draft should render");
    let selected_frame = selected
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("selected rejected TextBox");
    let text_rect = selected_frame.text_box_text_rect();
    assert!(selected.scene().quads().iter().any(|quad| {
        quad.fill() == Theme::default().text().selection && rect_contains(text_rect, quad.rect())
    }));
}

#[test]
fn invalid_text_box_and_indicator_hover_share_the_error_tip() {
    let (mut app, window, size, cell) = rejected_count_input_app();
    let initial = app
        .show_scene(window, size)
        .expect("rejected input should render before hover");
    let frame = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("rejected count TextBox");
    let text_rect = frame.text_box_text_rect();
    let indicator = frame
        .input_indicator_rect()
        .expect("invalid TextBox indicator");
    assert_eq!(frame.overflow_tip(), Some(LONG_REJECTED_COUNT));
    let box_point = frame_point_at(text_rect);
    let moved_box_point = geometry::Point::new(text_rect.x() + 1, box_point.y());
    let indicator_point = frame_point_at(indicator);
    drop(initial);

    app.pointer_move_at(window, size, box_point)
        .expect("invalid TextBox surface should become hover-eligible");
    assert_eq!(
        app.session().window(window).expect("window").cursor(),
        pointer::Cursor::Text,
        "the owning admitted text surface still promises selection"
    );
    let before_delay = app
        .show_scene(window, size)
        .expect("TextBox hover dwell should schedule");
    assert!(
        before_delay
            .layout()
            .frames()
            .iter()
            .all(|frame| frame.interaction_id() != Some(interaction::Id::new("feedback.hover")))
    );
    drop(before_delay);
    let crate::animation::Schedule::At(deadline) = app.animation_schedule() else {
        panic!("invalid TextBox hover should schedule one dwell deadline");
    };
    app.invalidate_due_animation_frames(deadline);
    let visible = app
        .show_scene_after_overlay_fade(window, size)
        .expect("invalid TextBox hover should reveal its error tip");
    let panel = visible
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.interaction_id() == Some(interaction::Id::new("feedback.hover")))
        .expect("invalid TextBox should reveal one hover panel");
    assert_eq!(
        panel.popup_placement().expect("hover placement").anchor(),
        geometry::placement::Anchor::Point(box_point)
    );
    assert_eq!(
        panel.auxiliary_hint().map(view::Hint::description),
        Some("Enter a whole number")
    );
    assert_eq!(
        panel.auxiliary_hint().map(view::Hint::tone),
        Some(view::Tone::Error)
    );
    assert!(visible.scene().texts().iter().any(|text| {
        text.value() == "Enter a whole number" && text.wrap() == scene::TextWrap::WordOrGlyph
    }));
    assert!(visible.scene().icons().iter().any(|icon| {
        icon.icon().id().as_str() == "x-circle" && rect_contains(panel.rect(), icon.rect())
    }));
    drop(visible);

    app.pointer_move_at(window, size, moved_box_point)
        .expect("movement within the owning TextBox should retain the reveal snapshot");
    let moved = app
        .show_scene_after_overlay_fade(window, size)
        .expect("same-target movement should preserve the error tip");
    let moved_panel = moved
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.interaction_id() == Some(interaction::Id::new("feedback.hover")))
        .expect("same-target error tip");
    assert_eq!(
        moved_panel
            .popup_placement()
            .expect("retained hover placement")
            .anchor(),
        geometry::placement::Anchor::Point(box_point),
        "the visible panel remains fixed to its reveal snapshot"
    );
    drop(moved);

    app.pointer_move_at(window, size, geometry::Point::new(1, 1))
        .expect("leaving the invalid TextBox should dismiss its panel");
    let dismissed = app
        .show_scene_after_overlay_fade(window, size)
        .expect("TextBox error tip should dismiss");
    assert!(
        dismissed
            .layout()
            .frames()
            .iter()
            .all(|frame| frame.interaction_id() != Some(interaction::Id::new("feedback.hover")))
    );
    drop(dismissed);

    app.pointer_move_at(window, size, indicator_point)
        .expect("the exact indicator target should expose the same error");
    assert_eq!(
        app.session().window(window).expect("window").cursor(),
        pointer::Cursor::Default,
        "the explanatory glyph is not itself a text-selection surface"
    );
    app.show_scene(window, size)
        .expect("indicator dwell should schedule");
    let crate::animation::Schedule::At(deadline) = app.animation_schedule() else {
        panic!("indicator hover should schedule one dwell deadline");
    };
    app.invalidate_due_animation_frames(deadline);
    let indicator_visible = app
        .show_scene_after_overlay_fade(window, size)
        .expect("indicator hover should reveal the shared error tip");
    let indicator_panel = indicator_visible
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.interaction_id() == Some(interaction::Id::new("feedback.hover")))
        .expect("indicator error tip");
    assert_eq!(
        indicator_panel
            .popup_placement()
            .expect("indicator hover placement")
            .anchor(),
        geometry::placement::Anchor::Point(indicator_point)
    );
    assert_eq!(
        indicator_panel
            .auxiliary_hint()
            .map(view::Hint::description),
        Some("Enter a whole number")
    );
    drop(indicator_visible);

    app.pointer_move_at(window, size, geometry::Point::new(1, 1))
        .expect("leaving the indicator should dismiss only the panel");
    let final_frame = app
        .show_scene_after_overlay_fade(window, size)
        .expect("inline invalidity should remain after panel dismissal");
    let invalid = final_frame
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("retained invalid TextBox");
    assert!(invalid.input_is_invalid());
    assert_eq!(invalid.input_indicator_rect(), Some(indicator));
    assert!(
        final_frame
            .layout()
            .frames()
            .iter()
            .all(|frame| frame.interaction_id() != Some(interaction::Id::new("feedback.hover")))
    );
}

#[test]
fn rejected_departure_blocks_other_cell_activation_selection_and_click_chain() {
    let mut app = editable_table_app(EditableTableState {
        records: vec![
            EditableRecord {
                key: 7,
                name: "Ada".to_owned(),
                count: 4,
            },
            EditableRecord {
                key: 8,
                name: "Grace".to_owned(),
                count: 8,
            },
        ],
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 124);
    let table = interaction::Id::new("editable.table");
    let invalid = crate::table::Cell::new(
        table,
        crate::virtual_list::Key::new(7),
        interaction::Id::new("count"),
    );
    let destination = crate::table::Cell::new(
        table,
        crate::virtual_list::Key::new(8),
        interaction::Id::new("name"),
    );

    app.show_scene(window, size)
        .expect("editable rows should materialize");
    app.handle_input(window, Input::focus(session::Focus::table_cell(invalid)))
        .expect("source cell should focus");
    app.handle_input(
        window,
        Input::key_down(input::Key::F2, input::Modifiers::default()),
    )
    .expect("source cell should enter its text task");
    app.handle_input(
        window,
        Input::text_selection(text::selection::Operation::SelectAll),
    )
    .expect("numeric draft should select for replacement");
    app.handle_input(window, Input::text_commit("-"))
        .expect("invalid syntax should remain a lawful draft");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("commit attempt should retain one rejection");

    let rejected = app
        .show_scene(window, size)
        .expect("rejected source should remain visible");
    let destination_point = rejected
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(destination))
        .map(frame_point)
        .expect("destination cell should materialize");
    assert_eq!(
        app.session()
            .selection(window, table)
            .and_then(|selection| selection.active()),
        Some(invalid.row())
    );

    app.open_context_menu_at(window, size, destination_point)
        .expect("context selection should attempt the same departure");
    assert_eq!(active_text_table_cell(app.session(), window), Some(invalid));
    assert_eq!(
        app.session()
            .selection(window, table)
            .and_then(|selection| selection.active()),
        Some(invalid.row()),
        "rejected context departure must not change the focal row"
    );
    assert!(
        app.session()
            .interaction(window)
            .and_then(interaction::Interaction::open_menu)
            .is_none(),
        "rejected context departure must not open a menu"
    );

    app.pointer_down_at(window, size, destination_point)
        .expect("rejected departure is a handled gesture");
    assert!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&session::Focus::table_cell(invalid)))
    );
    assert_eq!(active_text_table_cell(app.session(), window), Some(invalid));
    assert_eq!(
        app.session()
            .selection(window, table)
            .and_then(|selection| selection.active()),
        Some(invalid.row()),
        "row selection is a continuation of accepted departure"
    );
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().pressed())
            .is_none(),
        "the destination member never receives pointer-down"
    );
    app.pointer_up_at(window, size, destination_point)
        .expect("release after a rejected press is inert");

    app.handle_input(window, Input::cancel())
        .expect("cancel should retire the invalid source task");
    app.show_scene(window, size)
        .expect("cancellation should reproject the resting cells");
    app.pointer_down_at(window, size, destination_point)
        .expect("the first corrected gesture should be admitted");
    app.pointer_up_at(window, size, destination_point)
        .expect("the admitted gesture should release normally");
    assert_eq!(
        app.session()
            .selection(window, table)
            .and_then(|selection| selection.active()),
        Some(destination.row())
    );
    assert_eq!(
        active_text_table_cell(app.session(), window),
        None,
        "row selection alone does not activate its TextBox"
    );

    app.show_scene(window, size)
        .expect("selected destination should reproject before participation");
    app.pointer_down_at(window, size, destination_point)
        .expect("the second corrected gesture should activate the TextBox");
    app.pointer_up_at(window, size, destination_point)
        .expect("the caret gesture should release normally");
    assert!(
        text_draft(&app, window, session::Focus::table_cell(destination))
            .selected_text()
            .is_none(),
        "selection and rejection contribute nothing to the global repeated-click chain"
    );
    app.pointer_down_at(window, size, destination_point)
        .expect("the third corrected gesture should be the second text click");
    app.pointer_up_at(window, size, destination_point)
        .expect("the word-selection gesture should release normally");
    assert_eq!(
        text_draft(&app, window, session::Focus::table_cell(destination))
            .selected_text()
            .as_deref(),
        Some("Grace")
    );
}

#[test]
fn rejected_task_transition_blocks_controls_shortcuts_and_tab() {
    let mut app = task_gate_app(TaskGateState {
        records: vec![EditableRecord {
            key: 7,
            name: "Ada".to_owned(),
            count: 4,
        }],
        invocations: Vec::new(),
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(360, 400);
    let invalid = crate::table::Cell::new(
        interaction::Id::new("task.gate.table"),
        crate::virtual_list::Key::new(7),
        interaction::Id::new("count"),
    );

    app.show_scene(window, size)
        .expect("task-gate controls should render");
    app.handle_input(window, Input::focus(session::Focus::table_cell(invalid)))
        .expect("source cell should focus");
    app.handle_input(
        window,
        Input::key_down(input::Key::F2, input::Modifiers::default()),
    )
    .expect("source cell should enter its text task");
    app.handle_input(
        window,
        Input::text_selection(text::selection::Operation::SelectAll),
    )
    .expect("numeric draft should select for replacement");
    app.handle_input(window, Input::text_commit("-"))
        .expect("invalid syntax should remain a draft");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("invalid commit should retain the task");

    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("Tab rejection is handled locally");
    app.handle_input(window, Input::shortcut("Ctrl+G"))
        .expect("shortcut rejection is handled locally");
    assert!(app.state().invocations.is_empty());
    assert!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&session::Focus::table_cell(invalid)))
    );

    let rendered = app
        .show_scene(window, size)
        .expect("rejected task and controls should remain visible");
    let button = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.role() == view::Role::Button && frame.label_text() == Some("Dependent button")
        })
        .map(|frame| frame_point_at(frame.active_rect()))
        .expect("dependent button geometry");
    let checkbox = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.role() == view::Role::Checkbox && frame.label_text() == Some("Dependent checkbox")
        })
        .map(|frame| frame_point_at(frame.active_rect()))
        .expect("dependent checkbox geometry");
    let slider = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.role() == view::Role::Slider)
        .map(|frame| {
            frame_point_at(layout::slider_track_rect(
                frame.rect(),
                frame.label_width(),
                &Theme::default(),
            ))
        })
        .expect("dependent slider geometry");
    drop(rendered);

    for point in [button, checkbox, slider] {
        app.pointer_down_at(window, size, point)
            .expect("dependent control press should be consumed by rejection");
        app.pointer_up_at(window, size, point)
            .expect("release after rejection should be inert");
        assert!(app.state().invocations.is_empty());
        assert!(
            app.session()
                .focused(window)
                .is_some_and(|focus| focus.same_target(&session::Focus::table_cell(invalid)))
        );
        assert!(
            app.session()
                .interaction(window)
                .and_then(|interaction| interaction.pointer().capture())
                .is_none()
        );
    }

    app.pointer_down_at(
        window,
        size,
        geometry::Point::new(size.width() - 1, size.height() - 1),
    )
    .expect("click-away rejection should be handled locally");
    assert!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&session::Focus::table_cell(invalid)))
    );
    assert!(app.state().invocations.is_empty());
}

#[test]
fn selectable_rows_gate_members_by_pre_gesture_focality_and_modifiers() {
    let mut app = row_gate_app(TaskGateState {
        records: vec![
            EditableRecord {
                key: 7,
                name: "Ada".to_owned(),
                count: 4,
            },
            EditableRecord {
                key: 8,
                name: "Grace".to_owned(),
                count: 8,
            },
        ],
        invocations: Vec::new(),
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(360, 400);
    let table = interaction::Id::new("task.gate.table");
    let initial = app
        .show_scene(window, size)
        .expect("action rows should render");
    let action_point = |row| {
        initial
            .layout()
            .frames()
            .iter()
            .find(|frame| {
                frame.role() == view::Role::Button
                    && frame.table_cell().is_some_and(|cell| {
                        cell.row() == crate::virtual_list::Key::new(row)
                            && cell.column() == interaction::Id::new("action")
                    })
            })
            .map(|frame| frame_point_at(frame.active_rect()))
            .expect("row action should materialize")
    };
    let row7 = action_point(7);
    let row8 = action_point(8);
    drop(initial);

    app.pointer_down_at(window, size, row8)
        .expect("first action click should select its row");
    app.pointer_up_at(window, size, row8)
        .expect("selection-only action click should release");
    assert!(app.state().invocations.is_empty());
    assert_eq!(
        app.session()
            .selection(window, table)
            .and_then(|selection| selection.active()),
        Some(crate::virtual_list::Key::new(8))
    );

    app.show_scene(window, size)
        .expect("selected action row should reproject");
    app.pointer_down_at(window, size, row8)
        .expect("already-focal action should press");
    app.pointer_up_at(window, size, row8)
        .expect("already-focal action should invoke");
    assert_eq!(app.state().invocations, vec!["button"]);

    let shift = input::Modifiers::new(true, false, false, false);
    app.pointer_down_at_with_modifiers(window, size, row7, shift)
        .expect("Shift action gesture should extend selection only");
    app.pointer_up_at(window, size, row7)
        .expect("Shift selection gesture should release inertly");
    let selection = app
        .session()
        .selection(window, table)
        .expect("range selection should remain installed");
    assert!(selection.contains(crate::virtual_list::Key::new(7)));
    assert!(selection.contains(crate::virtual_list::Key::new(8)));
    assert_eq!(selection.active(), Some(crate::virtual_list::Key::new(7)));
    assert_eq!(app.state().invocations, vec!["button"]);

    app.show_scene(window, size)
        .expect("range-selected rows should reproject");
    app.pointer_down_at(window, size, row8)
        .expect("selected-but-not-focal row should only become focal");
    app.pointer_up_at(window, size, row8)
        .expect("selected-but-not-focal gesture should release inertly");
    assert_eq!(app.state().invocations, vec!["button"]);

    app.show_scene(window, size)
        .expect("newly focal action row should reproject");
    app.pointer_down_at(window, size, row8)
        .expect("subsequent focal action should press");
    app.pointer_up_at(window, size, row8)
        .expect("subsequent focal action should invoke");
    assert_eq!(app.state().invocations, vec!["button", "button"]);

    let control = input::Modifiers::new(false, true, false, false);
    app.pointer_down_at_with_modifiers(window, size, row8, control)
        .expect("Ctrl action gesture should toggle selection only");
    app.pointer_up_at(window, size, row8)
        .expect("Ctrl selection gesture should release inertly");
    assert_eq!(app.state().invocations, vec!["button", "button"]);
}

#[test]
fn text_task_deactivates_when_focal_row_changes_and_reentry_is_selection_only() {
    let mut app = editable_table_app(EditableTableState {
        records: vec![
            EditableRecord {
                key: 7,
                name: "Ada".to_owned(),
                count: 4,
            },
            EditableRecord {
                key: 8,
                name: "Grace".to_owned(),
                count: 8,
            },
        ],
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 124);
    let table = interaction::Id::new("editable.table");
    let row7 = crate::table::Cell::new(
        table,
        crate::virtual_list::Key::new(7),
        interaction::Id::new("name"),
    );
    let row8 = crate::table::Cell::new(
        table,
        crate::virtual_list::Key::new(8),
        interaction::Id::new("name"),
    );
    let initial = app
        .show_scene(window, size)
        .expect("text rows should render");
    let point_for = |cell| {
        initial
            .layout()
            .frames()
            .iter()
            .find(|frame| frame.table_cell() == Some(cell))
            .map(frame_point)
            .expect("text cell should materialize")
    };
    let row7_point = point_for(row7);
    let row8_point = point_for(row8);
    drop(initial);

    app.pointer_down_at(window, size, row7_point)
        .expect("first row gesture should establish focality");
    app.pointer_up_at(window, size, row7_point)
        .expect("selection-only gesture should release");
    app.show_scene(window, size)
        .expect("focal row should reproject");
    app.pointer_down_at(window, size, row7_point)
        .expect("second row gesture should activate text");
    app.pointer_up_at(window, size, row7_point)
        .expect("text activation should release");
    assert_eq!(active_text_table_cell(app.session(), window), Some(row7));

    app.show_scene(window, size)
        .expect("active text should reproject");
    app.pointer_down_at(window, size, row8_point)
        .expect("changing focal row should commit and deactivate text");
    app.pointer_up_at(window, size, row8_point)
        .expect("new-row selection should release");
    assert_eq!(active_text_table_cell(app.session(), window), None);
    assert_eq!(
        app.session()
            .selection(window, table)
            .and_then(|selection| selection.active()),
        Some(row8.row())
    );

    app.show_scene(window, size)
        .expect("inactive old row should reproject");
    app.pointer_down_at(window, size, row7_point)
        .expect("returning to the old row should select only");
    app.pointer_up_at(window, size, row7_point)
        .expect("return selection should release");
    assert_eq!(
        active_text_table_cell(app.session(), window),
        None,
        "a previously active TextBox must require a new participation gesture"
    );
}

#[test]
fn table_text_selects_row_before_participation_and_keeps_one_text_box_identity() {
    let clipboard = crate::clipboard::Clipboard::default();
    let mut app = editable_table_app(EditableTableState {
        records: vec![EditableRecord {
            key: 7,
            name: "Ada Lovelace".to_owned(),
            count: 4,
        }],
    })
    .with_clipboard(clipboard.clone());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 124);
    let table = interaction::Id::new("editable.table");
    let cell = crate::table::Cell::new(
        table,
        crate::virtual_list::Key::new(7),
        interaction::Id::new("name"),
    );
    let initial = app
        .show_scene(window, size)
        .expect("table text should render");
    let frame = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("name TextBox");
    let identity = frame.node_id();
    assert_eq!(frame.role(), view::Role::TextBox);
    assert!(frame.text_area_layout().is_some());
    assert!(frame.text_box_layout().is_none());
    let start = geometry::Point::new(
        frame.rect().x() + 8,
        frame.rect().y() + frame.rect().height() / 2,
    );
    let end = geometry::Point::new(frame.rect().x() + 52, start.y());

    app.pointer_down_at(window, size, start)
        .expect("first press should select the row only");
    app.pointer_up_at(window, size, start)
        .expect("selection-only press should release inertly");
    assert_eq!(active_text_table_cell(app.session(), window), None);
    assert_eq!(
        app.session()
            .selection(window, table)
            .and_then(|selection| selection.active()),
        Some(cell.row())
    );

    let selected_row = app
        .show_scene(window, size)
        .expect("selected resting TextBox should reproject");
    let selected_frame = selected_row
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("selected resting TextBox");
    assert_eq!(selected_frame.node_id(), identity);
    assert_eq!(selected_frame.role(), view::Role::TextBox);
    assert!(selected_frame.text_area_layout().is_some());

    app.pointer_down_at(window, size, start)
        .expect("second press should enter ordinary text participation");
    app.pointer_drag_at(window, size, end)
        .expect("active TextBox drag should extend selection");
    app.pointer_up_at(window, size, end)
        .expect("active TextBox drag should release");
    assert_eq!(active_text_table_cell(app.session(), window), Some(cell));
    let focus = session::Focus::table_cell(cell);
    let selected = text_draft(&app, window, focus)
        .selected_text()
        .expect("active TextBox should own its selection");
    assert!(!selected.is_empty());
    app.handle_input(window, Input::shortcut("Ctrl+C"))
        .expect("copy should route to the active TextBox");
    assert_eq!(
        clipboard.text().expect("clipboard available"),
        Some(selected)
    );

    let active = app
        .show_scene(window, size)
        .expect("active TextBox should render");
    let active_frame = active
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("active TextBox frame");
    assert_eq!(active_frame.node_id(), identity);
    assert_eq!(active_frame.role(), view::Role::TextBox);
    assert!(active_frame.text_area_layout().is_none());
    assert!(active_frame.text_box_layout().is_some());
    assert!(active.scene().quads().iter().any(|quad| {
        quad.fill() == Theme::default().text().selection
            && rect_contains(active_frame.text_box_text_rect(), quad.rect())
    }));

    let context_node = active
        .layout()
        .context_node_at(start)
        .expect("active TextBox should retain a contextual node");
    let context_path = app
        .composition(window)
        .expect("active TextBox composition")
        .context_path_for_node(context_node);
    assert!(context_path.iter().any(|owner| owner.cell() == Some(cell)));
    app.open_context_menu_at(window, size, start)
        .expect("active TextBox context should open");
    let context = app.present(window).expect("active context should project");
    let select_all = context
        .bindings()
        .into_iter()
        .filter(|binding| {
            binding.source() == context::Source::Menu
                && binding.command_type() == std::any::TypeId::of::<document::SelectAll>()
        })
        .collect::<Vec<_>>();
    assert_eq!(select_all.len(), 1);
    app.handle_view(window, select_all[0].action())
        .expect("Select All should invoke through text");
    assert!(
        !app.session()
            .selection(window, table)
            .is_some_and(crate::selection::Selection::is_all)
    );
    assert_eq!(
        text_draft(&app, window, focus).selected_text().as_deref(),
        Some("Ada Lovelace")
    );

    app.handle_input(window, Input::cancel())
        .expect("cancel should retire the text task");
    let resting = app
        .show_scene(window, size)
        .expect("resting TextBox should return");
    let resting_frame = resting
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("resting TextBox frame");
    assert_eq!(resting_frame.node_id(), identity);
    assert_eq!(resting_frame.role(), view::Role::TextBox);
    assert!(resting_frame.text_area_layout().is_some());
    assert_eq!(active_text_table_cell(app.session(), window), None);
}

#[test]
fn inactive_table_text_draft_retains_storage_without_repainting_selection() {
    let mut app = editable_table_app(EditableTableState {
        records: vec![
            EditableRecord {
                key: 7,
                name: "Ada Lovelace".to_owned(),
                count: 4,
            },
            EditableRecord {
                key: 8,
                name: "Grace Hopper".to_owned(),
                count: 5,
            },
        ],
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 148);
    let first = crate::table::Cell::new(
        interaction::Id::new("editable.table"),
        crate::virtual_list::Key::new(7),
        interaction::Id::new("name"),
    );
    let second = crate::table::Cell::new(
        interaction::Id::new("editable.table"),
        crate::virtual_list::Key::new(8),
        interaction::Id::new("name"),
    );
    let initial = app
        .show_scene(window, size)
        .expect("two display rows should render");
    let first_rect = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(first))
        .expect("first name cell")
        .rect();
    let second_rect = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(second))
        .expect("second name cell")
        .rect();
    drop(initial);

    let first_start = geometry::Point::new(first_rect.x() + 8, first_rect.y() + 12);
    let first_end = geometry::Point::new(first_rect.x() + 58, first_start.y());
    app.pointer_down_at(window, size, first_start)
        .expect("first display cell should select its row");
    app.pointer_up_at(window, size, first_start)
        .expect("selection-only gesture should release");
    app.show_scene(window, size)
        .expect("selected first row should reproject");
    app.pointer_down_at(window, size, first_start)
        .expect("already-focal first cell should begin text selection");
    app.pointer_drag_at(window, size, first_end)
        .expect("first display cell should extend selection");
    app.pointer_up_at(window, size, first_end)
        .expect("first display cell should retain its draft");
    let selected = app
        .show_scene(window, size)
        .expect("active selection should render");
    assert!(
        selected
            .layout()
            .frames()
            .iter()
            .find(|frame| frame.table_cell() == Some(first))
            .and_then(layout::Frame::text_box_layout)
            .is_some_and(|field| !field.layout().selection_spans().is_empty())
    );
    drop(selected);

    let second_point = frame_point_at(second_rect);
    app.pointer_down_at(window, size, second_point)
        .expect("second display cell should select its row first");
    app.pointer_up_at(window, size, second_point)
        .expect("selection-only second-row gesture should release");
    assert_eq!(active_text_table_cell(app.session(), window), None);
    app.show_scene(window, size)
        .expect("selected second row should reproject");
    app.pointer_down_at(window, size, second_point)
        .expect("already-focal second cell should become the active text target");
    app.pointer_up_at(window, size, second_point)
        .expect("active second cell should release");
    let second_target = text_target(session::Focus::table_cell(second));
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.text_input().target()),
        Some(&second_target)
    );
    let inactive = app
        .show_scene(window, size)
        .expect("inactive retained selection should reproject");
    let first_layout = inactive
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(first))
        .and_then(layout::Frame::text_area_layout)
        .expect("first cell retains selectable layout");
    assert!(
        first_layout.layout().selection_spans().is_empty(),
        "an inactive retained draft owns storage, not selection paint"
    );
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction
                .text_input()
                .draft_for(&text_target(session::Focus::table_cell(first))))
            .and_then(crate::draft::State::selected_text)
            .is_some(),
        "leaving presentation does not discard the useful source selection"
    );
}

#[test]
fn ellipsized_table_selection_paints_visible_glyphs_and_copies_source_ranges() {
    const SOURCE: &str =
        "HEAD application-owned value with deliberately omitted content TAIL_SENTINEL";
    let clipboard = crate::clipboard::Clipboard::default();
    let mut app = editable_table_app(EditableTableState {
        records: vec![EditableRecord {
            key: 7,
            name: SOURCE.to_owned(),
            count: 4,
        }],
    })
    .with_clipboard(clipboard.clone());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 124);
    let cell = crate::table::Cell::new(
        interaction::Id::new("editable.table"),
        crate::virtual_list::Key::new(7),
        interaction::Id::new("name"),
    );
    let initial = app
        .show_scene(window, size)
        .expect("ellipsized display cell should render");
    let frame = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("ellipsized name cell");
    assert!(frame.label_text().is_some_and(|text| text.ends_with('…')));
    let text_rect = frame.text_area_text_rect();
    drop(initial);

    let start = geometry::Point::new(text_rect.x(), text_rect.y() + text_rect.height() / 2);
    let end = geometry::Point::new(text_rect.right() - 1, start.y());
    app.pointer_down_at(window, size, start)
        .expect("first ellipsized click should select the row");
    app.pointer_up_at(window, size, start)
        .expect("selection-only ellipsized click should release");
    app.show_scene(window, size)
        .expect("selected ellipsized row should reproject");
    app.pointer_down_at(window, size, start)
        .expect("visible source head should begin text selection");
    app.pointer_drag_at(window, size, end)
        .expect("dragging across ellipsis should select the omitted source tail");
    app.pointer_up_at(window, size, end)
        .expect("ellipsized selection should release");

    let focus = session::Focus::table_cell(cell);
    let selected = text_draft(&app, window, focus)
        .selected_text()
        .expect("ellipsized display owns a source selection");
    assert!(selected.ends_with("TAIL_SENTINEL"));
    app.handle_input(window, Input::shortcut("Ctrl+C"))
        .expect("copy should consume the mapped source range");
    assert_eq!(
        clipboard.text().expect("clipboard available"),
        Some(selected)
    );

    let rendered = app
        .show_scene(window, size)
        .expect("ellipsized selection should share its shaped viewport");
    let frame = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("selected ellipsized cell");
    let selectable = frame
        .text_box_layout()
        .expect("the active TextBox owns text layout");
    assert!(!selectable.layout().selection_spans().is_empty());
    assert!(selectable.render_surface().is_some());
    assert!(
        rendered
            .scene()
            .text_viewports()
            .iter()
            .any(|viewport| viewport.rect() == frame.text_box_text_rect())
    );
}

#[test]
fn compact_ellipsized_table_cells_project_no_hidden_text_scrollbars() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(760, 700))
        .expect("compact gallery table should render");

    assert!(
        rendered
            .layout()
            .chrome()
            .iter()
            .all(|chrome| chrome.scroll_target().table_cell().is_none()),
        "ellipsized display text has visible extent, not a hidden scroll extent"
    );
}

#[test]
fn display_newlines_are_compact_residue_and_expanded_line_breaks() {
    #[derive(Clone)]
    struct Record {
        value: Multiline,
    }
    #[derive(Clone)]
    struct Multiline;
    impl std::fmt::Display for Multiline {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("alpha\nbeta")
        }
    }
    let render = |presentation| {
        let mut app = Runtime::new(SourceState::default())
            .started(|cx| {
                cx.open_window(window::Options::new("Multiline Display"));
            })
            .view(move |_, _| {
                let source = crate::table::Source::new(
                    1,
                    |_| crate::virtual_list::Key::new(0),
                    |key| (key.value() == 0).then_some(0),
                    |_| Record { value: Multiline },
                );
                widget::view_node(
                    crate::Table::typed(
                        "multiline.display",
                        24,
                        [crate::table::Column::text(
                            "value",
                            "Value",
                            view::Dimension::fixed(90),
                            |record: &Record| &record.value,
                        )
                        .unsortable()
                        .build()],
                        source,
                    )
                    .presentation(presentation)
                    .width(view::Dimension::fixed(90))
                    .height(view::Dimension::fixed(90)),
                )
            });
        app.start();
        let window = app.session().windows()[0].id();
        app.show_scene(window, geometry::Size::new(90, 90))
            .expect("multiline table should render")
    };
    let compact = render(crate::table::Presentation::Compact);
    let compact_cell = compact
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell().is_some())
        .expect("compact multiline cell");
    assert_eq!(compact_cell.label_text(), Some("alpha…"));
    assert!(
        compact_cell
            .label_text()
            .is_some_and(|value| !value.contains('\n'))
    );
    assert!(
        compact
            .layout()
            .chrome()
            .iter()
            .all(|chrome| chrome.scroll_target().table_cell().is_none())
    );

    let expanded = render(crate::table::Presentation::Expanded);
    let expanded_cell = expanded
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell().is_some())
        .expect("expanded multiline cell");
    assert_eq!(expanded_cell.label_text(), Some("alpha\nbeta"));
    assert!(expanded_cell.rect().height() >= compact_cell.rect().height());
}

#[test]
fn table_cell_text_input_without_a_draft_never_falls_through_to_document_editing() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 700);
    app.show_scene(window, size)
        .expect("control gallery table should render");

    let display_only = crate::table::Cell::new(
        interaction::Id::new("control_gallery.records"),
        crate::virtual_list::Key::new(0),
        interaction::Id::new("record"),
    );
    app.handle_input(
        window,
        Input::focus(session::Focus::table_cell(display_only).keyboard()),
    )
    .expect("display-only table cell should become current");
    let display_input = app
        .handle_input(
            window,
            Input::text_edit(text::Edit::insert("late display input")),
        )
        .expect("table-owned input without a draft should be inert");
    assert!(!display_input.is_handled());

    let stale = crate::table::Cell::new(
        interaction::Id::new("control_gallery.records"),
        crate::virtual_list::Key::new(u64::MAX),
        interaction::Id::new("note"),
    );
    app.handle_input(
        window,
        Input::focus(session::Focus::table_cell(stale).pointer()),
    )
    .expect("a late input target may outlive its materialized cell");
    let stale_input = app
        .handle_input(window, Input::text_commit("late stale input"))
        .expect("late table input should be inert instead of fatal");
    assert!(!stale_input.is_handled());

    let trigger = app.trigger::<document::ApplyEdit>(text::Edit::insert("programmatic"));
    let error = app
        .invoke_focused(window, trigger)
        .output
        .expect_err("programmatic missing-target errors must remain visible");
    assert!(matches!(
        error,
        Error::MissingTarget {
            command: "document.apply_edit"
        }
    ));
}

#[test]
fn table_focus_presentation_follows_modality_and_active_edit_surface() {
    let mut app = editable_table_app(EditableTableState {
        records: vec![EditableRecord {
            key: 7,
            name: "Ada".to_owned(),
            count: 4,
        }],
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 124);
    let cell = crate::table::Cell::new(
        interaction::Id::new("editable.table"),
        crate::virtual_list::Key::new(7),
        interaction::Id::new("name"),
    );
    let initial = app
        .show_scene(window, size)
        .expect("editable table should render");
    let point = frame_point_at(
        initial
            .layout()
            .frames()
            .iter()
            .find(|frame| frame.table_cell() == Some(cell))
            .expect("editable display cell")
            .rect(),
    );

    app.pointer_down_at(window, size, point)
        .expect("single click should make the cell current");
    app.pointer_up_at(window, size, point)
        .expect("single click should release without editing");
    let pointer_current = app
        .show_scene(window, size)
        .expect("pointer-current cell should render");
    let display = pointer_current
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("pointer-current display cell");
    let current_row = pointer_current
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame
                .provided_row()
                .is_some_and(|row| row.key() == cell.row())
        })
        .expect("pointer-current row");
    assert_eq!(active_text_table_cell(app.session(), window), None);
    assert!(display.is_focused());
    assert!(!display.focus_visible());
    assert!(current_row.is_active_item());
    assert!(pointer_current.scene().quads().iter().any(|quad| {
        quad.rect() == current_row.rect() && quad.fill() == Theme::default().menu().row_hover_tint
    }));
    assert!(pointer_current.scene().outlines().iter().all(|outline| {
        outline.rect() != display.rect() || outline.color() != Theme::default().focus().color
    }));

    app.handle_input(
        window,
        Input::focus(session::Focus::table_cell(cell).keyboard()),
    )
    .expect("keyboard focus should retain the current cell");
    let keyboard_current = app
        .show_scene(window, size)
        .expect("keyboard-current cell should render");
    let display = keyboard_current
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("keyboard-current display cell");
    assert!(display.focus_visible());
    assert!(keyboard_current.scene().outlines().iter().any(|outline| {
        outline.rect() == display.rect() && outline.color() == Theme::default().focus().color
    }));

    app.handle_input(
        window,
        Input::focus(session::Focus::table_cell(cell).pointer()),
    )
    .expect("pointer modality should replace keyboard visibility");
    app.handle_input(
        window,
        Input::key_down(input::Key::F2, input::Modifiers::default()),
    )
    .expect("deliberate edit activation should be handled");
    let editing = app
        .show_scene(window, size)
        .expect("pointer-focused active editor should render");
    let editor = editing
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("active editor");
    let inset = geometry::Rect::new(
        editor.rect().x() + 1,
        editor.rect().y() + 1,
        editor.rect().width() - 2,
        editor.rect().height() - 2,
    );
    assert_eq!(editor.table_part(), Some(view::TablePart::Cell));
    assert!(editor.focus_visible());
    assert!(editing.scene().outlines().iter().any(|outline| {
        outline.rect() == inset && outline.color() == Theme::default().focus().color
    }));
}

#[test]
fn editable_table_draft_pins_through_scroll_follows_reorder_and_dies_on_deletion() {
    let mut app = editable_table_app(EditableTableState {
        records: (0..50)
            .map(|key| EditableRecord {
                key,
                name: String::new(),
                count: key as i64,
            })
            .collect(),
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 124);
    let initial = app
        .show_scene(window, size)
        .expect("editable records should render");
    let cell = crate::table::Cell::new(
        interaction::Id::new("editable.table"),
        crate::virtual_list::Key::new(0),
        interaction::Id::new("name"),
    );
    let focus = session::Focus::table_cell(cell);
    app.handle_input(window, Input::focus(focus))
        .expect("editable cell should focus");
    app.handle_input(
        window,
        Input::key_down(input::Key::F2, input::Modifiers::default()),
    )
    .expect("F2 should deliberately enter the pinned cell");
    app.handle_input(window, Input::text_commit("draft"))
        .expect("editable cell should retain draft");
    let list = initial.layout().find_role(view::Role::VirtualList)[0];
    app.scroll_at(
        window,
        size,
        frame_point_at(list.rect()),
        interaction::ScrollDelta::vertical(720),
    )
    .expect("table body should scroll");
    let scrolled = app
        .show_scene(window, size)
        .expect("focused edited row should remain pinned");
    assert!(
        scrolled
            .layout()
            .frames()
            .iter()
            .any(|frame| frame.table_cell() == Some(cell))
    );
    assert_eq!(text_draft(&app, window, focus).text(), "draft");

    app.change(
        state::Reason::programmatic("reorder editable rows"),
        |state| {
            state.records.reverse();
        },
    );
    app.show_scene(window, size)
        .expect("reordered edited row should render by key");
    assert_eq!(text_draft(&app, window, focus).text(), "draft");

    app.change(state::Reason::programmatic("delete edited row"), |state| {
        state.records.retain(|record| record.key != 0);
    });
    app.show_scene(window, size)
        .expect("provider deletion should reconcile edited identity");
    let target = text_target(focus);
    assert!(
        app.session()
            .interaction(window)
            .expect("window interaction")
            .text_input()
            .draft_for(&target)
            .is_none()
    );
    assert_eq!(active_text_table_cell(app.session(), window), None);
    assert!(app.session().focused(window).is_none());
}

#[test]
fn editable_table_keyboard_enters_commits_leaves_and_materializes_cells() {
    let mut app = editable_table_app(EditableTableState {
        records: (0..50)
            .map(|key| EditableRecord {
                key,
                name: String::new(),
                count: key as i64,
            })
            .collect(),
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 124);
    let initial = app
        .show_scene(window, size)
        .expect("editable keyboard table should render");
    let first_name = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(0)
                    && cell.column() == interaction::Id::new("name")
            })
        })
        .expect("first editable name should render");
    app.pointer_down_at(window, size, frame_point_at(first_name.rect()))
        .expect("pointer should establish table row/column activity");
    assert!(app.focus(
        window,
        session::Focus::control(&interaction::Target::scroll(
            "editable.table",
            "Editable rows",
        )),
    ));
    app.handle_input(
        window,
        Input::key_down(
            input::Key::End,
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("Ctrl+End should select and reveal the final table cell");
    app.show_scene(window, size)
        .expect("reveal should materialize the distant active row before entry");
    assert_eq!(
        app.session()
            .active_table_cell(window, interaction::Id::new("editable.table"))
            .map(crate::table::Cell::row),
        Some(crate::virtual_list::Key::new(49))
    );
    app.handle_input(
        window,
        Input::key_down(input::Key::Home, input::Modifiers::default()),
    )
    .expect("Home should move to the first cell in the final row");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("Enter should materialize and enter the active editor");
    let last_name = crate::table::Cell::new(
        interaction::Id::new("editable.table"),
        crate::virtual_list::Key::new(49),
        interaction::Id::new("name"),
    );
    assert!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&session::Focus::table_cell(last_name)))
    );
    app.handle_input(window, Input::text_commit("X"))
        .expect("entered cell should accept text");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("Enter should commit and attempt to move down");
    assert_eq!(app.state().records[49].name, "X");
    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("Tab should leave after committing and enter the next public editor");
    let last_count = crate::table::Cell::new(
        interaction::Id::new("editable.table"),
        crate::virtual_list::Key::new(49),
        interaction::Id::new("count"),
    );
    assert!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&session::Focus::table_cell(last_count)))
    );
}

#[test]
fn table_edit_commit_keys_move_canonical_current_cell_without_trapping_tab() {
    let mut app = editable_table_app(EditableTableState {
        records: (0..3)
            .map(|key| EditableRecord {
                key,
                name: String::new(),
                count: key as i64,
            })
            .collect(),
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 124);
    let initial = app.show_scene(window, size).expect("editable table");
    let first_name = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(0)
                    && cell.column() == interaction::Id::new("name")
            })
        })
        .expect("first name cell");
    app.pointer_down_at(window, size, frame_point_at(first_name.rect()))
        .expect("first cell should become current");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("Enter should begin editing");
    app.handle_input(window, Input::text_commit("A"))
        .expect("first draft");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("Enter should commit and move down");
    assert_eq!(app.state().records[0].name, "A");
    assert_eq!(active_text_table_cell(app.session(), window), None);
    assert_eq!(
        app.session()
            .active_table_cell(window, interaction::Id::new("editable.table")),
        Some(crate::table::Cell::new(
            interaction::Id::new("editable.table"),
            crate::virtual_list::Key::new(1),
            interaction::Id::new("name"),
        ))
    );

    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("Enter should begin the next edit");
    app.handle_input(window, Input::text_commit("B"))
        .expect("second draft");
    app.handle_input(
        window,
        Input::key_down(
            input::Key::Enter,
            input::Modifiers::new(true, false, false, false),
        ),
    )
    .expect("Shift+Enter should commit and move up");
    assert_eq!(app.state().records[1].name, "B");
    assert_eq!(
        app.session()
            .active_table_cell(window, interaction::Id::new("editable.table"))
            .map(crate::table::Cell::row),
        Some(crate::virtual_list::Key::new(0))
    );

    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("Tab should advance within the row");
    assert_eq!(
        app.session()
            .active_table_cell(window, interaction::Id::new("editable.table"))
            .map(crate::table::Cell::column),
        Some(interaction::Id::new("count"))
    );
    app.handle_input(
        window,
        Input::key_down(
            input::Key::Tab,
            input::Modifiers::new(true, false, false, false),
        ),
    )
    .expect("Shift+Tab should move backward within the row");
    assert_eq!(
        app.session()
            .active_table_cell(window, interaction::Id::new("editable.table"))
            .map(crate::table::Cell::column),
        Some(interaction::Id::new("name"))
    );
}

#[test]
fn virtual_list_growth_shrink_and_reorder_follow_stable_provider_keys() {
    let keys = Rc::new(RefCell::new((0..100).collect::<Vec<_>>()));
    let provider = MutableKeyProvider {
        keys: Rc::clone(&keys),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Mutable virtual rows"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::new("mutable.rows", 20, provider.clone())
                    .width(view::Dimension::grow())
                    .height(view::Dimension::fixed(100)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 100);
    app.show_scene(window, size)
        .expect("initial mutable list should render");

    keys.borrow_mut()[..7].reverse();
    app.request_redraw(window);
    let reordered = app
        .show_scene(window, size)
        .expect("reordered list should render");
    assert_eq!(reordered.scene().texts()[0].value(), "Key 6");
    assert!(
        app.composition(window)
            .expect("composition should remain installed")
            .changes()
            .is_empty(),
        "reordering materialized rows with stable keys must retain their identities"
    );

    keys.borrow_mut().extend(100..120);
    app.request_redraw(window);
    app.show_scene(window, size)
        .expect("grown list should render");
    assert!(
        app.composition(window)
            .expect("composition should remain installed")
            .changes()
            .is_empty(),
        "offscreen growth must not churn materialized identities"
    );

    keys.borrow_mut().truncate(3);
    app.diagnostics_mut(window)
        .expect("window diagnostics")
        .begin_renderer_measurement();
    app.request_redraw(window);
    let shrunk = app
        .show_scene(window, size)
        .expect("shrunk list should render");
    assert_eq!(shrunk.scene().texts().len(), 3);
    assert!(
        app.diagnostics(window)
            .expect("window diagnostics")
            .render
            .scene_nodes_removed
            > 0,
        "provider deletion must retire the departed retained scene nodes"
    );
}

#[test]
fn virtual_list_focus_and_active_edit_pin_while_inactive_drafts_dematerialize() {
    let keys = Rc::new(RefCell::new((0..50).collect::<Vec<_>>()));
    let provider = PinnedRowProvider {
        keys: Rc::clone(&keys),
        kind: PinnedRowKind::Text,
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Pinned virtual text"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::new("pinned.text.rows", 24, provider.clone())
                    .width(view::Dimension::grow())
                    .height(view::Dimension::fixed(96)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(260, 96);
    let initial = app
        .show_scene(window, size)
        .expect("virtual text rows should render");
    let list_rect = initial.layout().find_role(view::Role::VirtualList)[0].rect();
    let first = session::Focus::text("virtual.text.0");
    let second = session::Focus::text("virtual.text.1");
    let first_target = text_target(first);
    let second_target = text_target(second);

    app.handle_input(window, Input::focus(first))
        .expect("first virtual text row should focus");
    app.handle_input(window, Input::text_commit("draft zero"))
        .expect("first virtual text row should own a draft");
    app.handle_input(window, Input::focus(second))
        .expect("second virtual text row should focus");
    app.handle_input(window, Input::text_commit("draft one"))
        .expect("second virtual text row should own a draft");

    app.scroll_at(
        window,
        size,
        frame_point_at(list_rect),
        interaction::ScrollDelta::vertical(720),
    )
    .expect("virtual text rows should scroll");
    let scrolled = app
        .show_scene(window, size)
        .expect("scrolled virtual text rows should render");
    assert!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&second))
    );
    assert!(
        scrolled.layout().find_role(view::Role::TextBox).len() <= 10,
        "visible rows plus one focused pin stay bounded"
    );
    let input = app
        .session()
        .interaction(window)
        .expect("virtual text interaction should remain installed")
        .text_input();
    assert_eq!(
        input
            .draft_for(&first_target)
            .expect("inactive dematerialized draft should survive")
            .text(),
        "Text 0draft zero"
    );
    assert_eq!(
        input
            .draft_for(&second_target)
            .expect("focused pinned draft should survive")
            .text(),
        "Text 1draft one"
    );

    app.clear_focus(window);
    let active_only = app
        .show_scene(window, size)
        .expect("active edit should render without focus");
    assert!(app.session().focused(window).is_none());
    assert!(
        active_only.layout().find_role(view::Role::TextBox).len() <= 10,
        "the active edit target pins independently of focus"
    );

    keys.borrow_mut().retain(|key| *key != 1);
    app.request_redraw(window);
    app.show_scene(window, size)
        .expect("provider deletion should reconcile");
    let input = app
        .session()
        .interaction(window)
        .expect("interaction should remain installed")
        .text_input();
    assert!(
        input.draft_for(&second_target).is_none(),
        "actual provider deletion ends the row draft"
    );
    assert!(
        input.draft_for(&first_target).is_some(),
        "an existing dematerialized row keeps its inactive draft"
    );

    app.scroll_at(
        window,
        size,
        frame_point_at(list_rect),
        interaction::ScrollDelta::vertical(-720),
    )
    .expect("virtual text rows should scroll back");
    app.show_scene(window, size)
        .expect("rematerialized draft row should render");
    let projected = app.present(window).expect("virtual rows should project");
    assert!(projected.text_boxes().iter().any(|text_box| {
        text_box
            .focus()
            .is_some_and(|focus| focus.same_target(&first))
            && text_box.text() == "Text 0draft zero"
    }));
}

#[test]
fn virtual_list_pointer_capture_pins_until_provider_deletion() {
    let keys = Rc::new(RefCell::new((0..50).collect::<Vec<_>>()));
    let provider = PinnedRowProvider {
        keys: Rc::clone(&keys),
        kind: PinnedRowKind::Capture,
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Pinned virtual capture"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::new("pinned.capture.rows", 24, provider.clone())
                    .width(view::Dimension::grow())
                    .height(view::Dimension::fixed(96)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(260, 96);
    let initial = app
        .show_scene(window, size)
        .expect("virtual capture rows should render");
    let nested_rect = initial.layout().find_role(view::Role::Scroll)[0].rect();
    let list_rect = initial.layout().find_role(view::Role::VirtualList)[0].rect();

    app.pointer_down_at(window, size, frame_point_at(nested_rect))
        .expect("nested scroll should capture the pointer");
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .is_some()
    );
    app.scroll_at(
        window,
        size,
        frame_point_at(list_rect),
        interaction::ScrollDelta::vertical(720),
    )
    .expect("outer virtual list should scroll");
    let scrolled = app
        .show_scene(window, size)
        .expect("captured row should remain materialized");
    assert!(scrolled.layout().find_role(view::Role::Scroll).len() <= 10);
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .is_some(),
        "capture should survive ordinary row dematerialization"
    );

    keys.borrow_mut().retain(|key| *key != 0);
    app.request_redraw(window);
    app.show_scene(window, size)
        .expect("deleted captured row should reconcile");
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .is_none(),
        "provider deletion must release capture"
    );
}

#[test]
fn virtual_list_materializes_logical_target_before_focus_transfer() {
    let keys = Rc::new(RefCell::new((0..50).collect::<Vec<_>>()));
    let provider = PinnedRowProvider {
        keys,
        kind: PinnedRowKind::Text,
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Virtual focus transfer"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::new("logical.focus.rows", 24, provider.clone())
                    .width(view::Dimension::grow())
                    .height(view::Dimension::fixed(96)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(260, 96);
    let initial = app
        .show_scene(window, size)
        .expect("logical focus list should render");
    let list_rect = initial.layout().find_role(view::Role::VirtualList)[0].rect();
    app.scroll_at(
        window,
        size,
        frame_point_at(list_rect),
        interaction::ScrollDelta::vertical(720),
    )
    .expect("logical focus list should scroll");
    app.show_scene(window, size)
        .expect("offscreen logical focus list should render");
    let first = session::Focus::text("virtual.text.0");
    assert!(
        !app.composition(window)
            .expect("composition should be installed")
            .view()
            .contains_focus(first)
    );

    assert!(app.focus_virtual_row(
        window,
        interaction::Id::new("logical.focus.rows"),
        crate::virtual_list::Key::new(0),
        first,
    ));
    assert!(
        app.composition(window)
            .expect("composition should remain installed")
            .view()
            .contains_focus(first)
    );
    assert!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&first)),
        "focus changes only after the keyed row exists in the composition"
    );
}

#[test]
fn selectable_virtual_list_handles_click_toggle_range_and_bounded_select_all() {
    let row_calls = Rc::new(Cell::new(0));
    let provider = MillionRowProvider {
        row_calls: Rc::clone(&row_calls),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Selectable million rows"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::new("selectable.million", 20, provider.clone())
                    .selectable()
                    .width(view::Dimension::grow())
                    .height(view::Dimension::fixed(100)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let list_id = interaction::Id::new("selectable.million");
    let size = geometry::Size::new(260, 100);
    let initial = app
        .show_scene(window, size)
        .expect("selectable virtual list should render");
    let viewport = initial.layout().find_role(view::Role::VirtualList)[0]
        .viewport()
        .expect("selectable list should own a viewport")
        .rect();
    let row_point = |index: i32| {
        geometry::Point::new(
            viewport.x().saturating_add(8),
            viewport
                .y()
                .saturating_add(index.saturating_mul(20))
                .saturating_add(10),
        )
    };
    assert!(
        app.session()
            .selection(window, list_id)
            .expect("selectable list should install empty state")
            .is_empty()
    );

    app.pointer_down_at(window, size, row_point(1))
        .expect("plain click should select a row");
    let primary = input::Modifiers::new(false, true, false, false);
    app.pointer_down_at_with_modifiers(window, size, row_point(2), primary)
        .expect("primary click should toggle a row");
    let extend = input::Modifiers::new(true, false, false, false);
    app.pointer_down_at_with_modifiers(window, size, row_point(4), extend)
        .expect("shift click should extend from the toggle anchor");
    let selection = app
        .session()
        .selection(window, list_id)
        .expect("selection should remain installed");
    assert_eq!(selection.len(), 3);
    assert!(!selection.contains(crate::virtual_list::Key::new(1)));
    assert!(selection.contains(crate::virtual_list::Key::new(2)));
    assert!(selection.contains(crate::virtual_list::Key::new(3)));
    assert!(selection.contains(crate::virtual_list::Key::new(4)));
    assert_eq!(selection.anchor(), Some(crate::virtual_list::Key::new(2)));
    assert_eq!(selection.active(), Some(crate::virtual_list::Key::new(4)));

    app.handle_input(
        window,
        Input::key_down(
            input::Key::ArrowDown,
            input::Modifiers::new(true, false, false, false),
        ),
    )
    .expect("shift+down should extend the keyed range from its retained anchor");
    let extended = app
        .session()
        .selection(window, list_id)
        .expect("keyboard range should remain installed");
    assert_eq!(extended.anchor(), Some(crate::virtual_list::Key::new(2)));
    assert_eq!(extended.active(), Some(crate::virtual_list::Key::new(5)));
    assert_eq!(extended.len(), 4);
    app.handle_input(
        window,
        Input::key_down(
            input::Key::ArrowUp,
            input::Modifiers::new(true, false, false, false),
        ),
    )
    .expect("shift+up should contract the same keyed range");

    let selected_scene = app
        .show_scene(window, size)
        .expect("selected rows should render");
    assert_eq!(
        selected_scene
            .layout()
            .frames()
            .iter()
            .filter(|frame| frame.provided_row().is_some() && frame.is_selected())
            .count(),
        3
    );
    assert_eq!(
        selected_scene
            .layout()
            .frames()
            .iter()
            .filter(|frame| frame.is_active_item())
            .count(),
        1
    );

    app.handle_input(window, Input::key_down(input::Key::Character('a'), primary))
        .expect("primary+A should select all logical rows");
    let selection = app
        .session()
        .selection(window, list_id)
        .expect("selection should remain installed");
    assert!(selection.is_all());
    assert_eq!(selection.len(), 1_000_000);
    let calls_before = row_calls.get();
    let all_scene = app
        .show_scene(window, size)
        .expect("select-all should remain renderable");
    assert!(all_scene.layout().frames().len() <= 10);
    assert!(row_calls.get().saturating_sub(calls_before) <= 16);

    app.handle_input(
        window,
        Input::key_down(input::Key::End, input::Modifiers::default()),
    )
    .expect("End should move the active item to the final logical row");
    let moved = app
        .show_scene(window, size)
        .expect("offscreen active row should reveal");
    let selection = app
        .session()
        .selection(window, list_id)
        .expect("selection should remain installed");
    assert_eq!(selection.len(), 1);
    assert_eq!(
        selection.active(),
        Some(crate::virtual_list::Key::new(999_999))
    );
    assert!(
        moved
            .scene()
            .texts()
            .iter()
            .any(|text| text.value() == "Provider row 999999")
    );
    assert!(moved.layout().frames().len() <= 10);
}

#[test]
fn selectable_virtual_list_reconciles_reorder_and_deleted_active_key() {
    let keys = Rc::new(RefCell::new((0..20).collect::<Vec<_>>()));
    let provider = MutableKeyProvider {
        keys: Rc::clone(&keys),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Mutable selection"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::new("mutable.selection", 20, provider.clone())
                    .selectable()
                    .width(view::Dimension::grow())
                    .height(view::Dimension::fixed(100)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let list = interaction::Id::new("mutable.selection");
    let size = geometry::Size::new(240, 100);
    let initial = app
        .show_scene(window, size)
        .expect("mutable selection should render");
    let viewport = initial.layout().find_role(view::Role::VirtualList)[0]
        .viewport()
        .expect("mutable selection should have a viewport")
        .rect();
    let point = |row: i32| {
        geometry::Point::new(
            viewport.x().saturating_add(8),
            viewport.y().saturating_add(row * 20 + 10),
        )
    };
    app.pointer_down_at(window, size, point(1))
        .expect("first selection should succeed");
    app.pointer_down_at_with_modifiers(
        window,
        size,
        point(3),
        input::Modifiers::new(false, true, false, false),
    )
    .expect("toggle selection should succeed");

    keys.borrow_mut()[..5].reverse();
    app.request_redraw(window);
    app.show_scene(window, size)
        .expect("reordered selection should render");
    let selection = app
        .session()
        .selection(window, list)
        .expect("selection should remain installed");
    assert!(selection.contains(crate::virtual_list::Key::new(1)));
    assert!(selection.contains(crate::virtual_list::Key::new(3)));
    assert_eq!(selection.active(), Some(crate::virtual_list::Key::new(3)));

    keys.borrow_mut().retain(|key| *key != 3);
    app.request_redraw(window);
    app.show_scene(window, size)
        .expect("selection should reconcile provider deletion");
    let selection = app
        .session()
        .selection(window, list)
        .expect("selection should remain installed");
    assert_eq!(selection.len(), 1);
    assert!(selection.contains(crate::virtual_list::Key::new(1)));
    assert_eq!(selection.active(), Some(crate::virtual_list::Key::new(1)));
    assert_eq!(selection.anchor(), Some(crate::virtual_list::Key::new(1)));
}

#[test]
fn virtual_selection_is_window_local_and_survives_runtime_snapshot_restore() {
    let provider = MillionRowProvider {
        row_calls: Rc::new(Cell::new(0)),
    };
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Selection one"));
            cx.open_window(window::Options::new("Selection two"));
        })
        .view(move |_, _| {
            widget::view_node(
                crate::VirtualList::new("window.selection", 20, provider.clone())
                    .selectable()
                    .width(view::Dimension::grow())
                    .height(view::Dimension::fixed(100)),
            )
        });
    app.start();
    let first_window = app.session().windows()[0].id();
    let second_window = app.session().windows()[1].id();
    let list = interaction::Id::new("window.selection");
    let size = geometry::Size::new(240, 100);
    app.show_scene(first_window, size)
        .expect("first selection window should render");
    app.show_scene(second_window, size)
        .expect("second selection window should render");

    app.pointer_down_at(first_window, size, geometry::Point::new(8, 30))
        .expect("first window should select row one");
    app.pointer_down_at(second_window, size, geometry::Point::new(8, 50))
        .expect("second window should select row two");
    assert!(
        app.session()
            .selection(first_window, list)
            .expect("first selection should exist")
            .contains(crate::virtual_list::Key::new(1))
    );
    assert!(
        app.session()
            .selection(second_window, list)
            .expect("second selection should exist")
            .contains(crate::virtual_list::Key::new(2))
    );

    let snapshot = app.snapshot();
    app.pointer_down_at(first_window, size, geometry::Point::new(8, 70))
        .expect("first window should temporarily select row three");
    app.restore(snapshot);
    app.show_scene(first_window, size)
        .expect("restored first selection window should render");
    app.show_scene(second_window, size)
        .expect("restored second selection window should render");
    assert!(
        app.session()
            .selection(first_window, list)
            .expect("restored first selection should exist")
            .contains(crate::virtual_list::Key::new(1))
    );
    assert!(
        app.session()
            .selection(second_window, list)
            .expect("restored second selection should exist")
            .contains(crate::virtual_list::Key::new(2))
    );
}

#[test]
fn virtual_selection_is_list_local_within_one_window() {
    let first_provider = MillionRowProvider {
        row_calls: Rc::new(Cell::new(0)),
    };
    let second_provider = first_provider.clone();
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Two selections"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.add(
                        crate::VirtualList::new("selection.first", 20, first_provider.clone())
                            .selectable()
                            .height(view::Dimension::fixed(80)),
                    );
                    ui.add(
                        crate::VirtualList::new("selection.second", 20, second_provider.clone())
                            .selectable()
                            .height(view::Dimension::fixed(80)),
                    );
                });
            })
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 160);
    let initial = app
        .show_scene(window, size)
        .expect("two selectable lists should render");
    let lists = initial.layout().find_role(view::Role::VirtualList);
    let first_viewport = lists[0].viewport().expect("first viewport").rect();
    let second_viewport = lists[1].viewport().expect("second viewport").rect();
    app.pointer_down_at(
        window,
        size,
        geometry::Point::new(first_viewport.x() + 8, first_viewport.y() + 30),
    )
    .expect("first list should select row one");
    app.pointer_down_at(
        window,
        size,
        geometry::Point::new(second_viewport.x() + 8, second_viewport.y() + 50),
    )
    .expect("second list should select row two");

    let first = app
        .session()
        .selection(window, interaction::Id::new("selection.first"))
        .expect("first list selection should exist");
    let second = app
        .session()
        .selection(window, interaction::Id::new("selection.second"))
        .expect("second list selection should exist");
    assert_eq!(first.len(), 1);
    assert!(first.contains(crate::virtual_list::Key::new(1)));
    assert_eq!(second.len(), 1);
    assert!(second.contains(crate::virtual_list::Key::new(2)));
}

#[test]
fn world_text_resolves_overflow_during_layout_before_scene_paint() {
    let source = "C:/very/long/provider/path/to/report.csv";
    let view = View::new(view::Node::world_text(
        source,
        text::Overflow::EllipsisMiddle,
    ));
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(96, 24), &mut layout_engine);
    let frame = layout
        .find_role(view::Role::Label)
        .into_iter()
        .next()
        .expect("world text frame should exist");
    let resolved = frame.label_text().expect("world text should be resolved");

    assert_ne!(resolved, source);
    assert!(resolved.contains('…'));
    assert_eq!(
        frame.world_text_overflow(),
        Some(text::Overflow::EllipsisMiddle)
    );

    let scene = scene::Scene::paint(&layout);
    let painted = scene.texts().into_iter().next().expect("text should paint");
    assert_eq!(painted.value(), resolved);
    assert_eq!(painted.overflow(), text::Overflow::EllipsisMiddle);
    assert_eq!(painted.wrap(), scene::TextWrap::None);
    assert_eq!(
        layout_engine.take_text_diagnostics().author_text_overflows,
        0
    );
}

#[test]
fn wrapped_world_text_preserves_source_and_shares_width_with_measure_and_paint() {
    let source = "Provider-authored words wrap through the shared shaping cache without omission.";
    let compose = |width, engine: &mut layout::Engine| {
        let label = crate::Widget::into_node(widget::Label::wrapped(source)).with_style(
            view::Style::new()
                .with_width(view::Dimension::fixed(width))
                .with_height(view::Dimension::fit()),
        );
        let view = View::new(
            view::Node::stack(view::Axis::Horizontal)
                .with_style(
                    view::Style::new()
                        .with_width(view::Dimension::fixed(width))
                        .with_height(view::Dimension::fit())
                        .with_align_items(view::Align::Start),
                )
                .child(label),
        );
        layout::Layout::compose(&view, geometry::Size::new(width, 200), engine)
    };
    let mut layout_engine = layout::Engine::new();
    let wide = compose(240, &mut layout_engine);
    let narrow = compose(92, &mut layout_engine);
    let wide_frame = wide
        .find_role(view::Role::Label)
        .into_iter()
        .next()
        .expect("wide wrapped label");
    let narrow_frame = narrow
        .find_role(view::Role::Label)
        .into_iter()
        .next()
        .expect("narrow wrapped label");

    assert_eq!(wide_frame.label_text(), Some(source));
    assert_eq!(narrow_frame.label_text(), Some(source));
    assert_eq!(wide_frame.world_text_wrap(), Some(view::Wrap::Word));
    assert_eq!(narrow_frame.world_text_wrap(), Some(view::Wrap::Word));
    assert!(narrow_frame.rect().height() >= wide_frame.rect().height());
    assert!(narrow_frame.rect().height() > Theme::default().menu().row_height);

    let narrow_scene = scene::Scene::paint(&narrow);
    let painted = narrow_scene
        .texts()
        .into_iter()
        .find(|text| text.value() == source)
        .expect("wrapped source should paint unchanged");
    assert_eq!(painted.rect(), narrow_frame.rect());
    assert_eq!(painted.wrap(), scene::TextWrap::WordOrGlyph);
    assert_eq!(painted.overflow(), text::Overflow::Clip);
    assert_eq!(
        layout_engine.take_text_diagnostics().author_text_overflows,
        0
    );
}

#[test]
fn overflowing_author_text_is_reported_without_mutating_its_value() {
    let source = "This authored sentence cannot fit in one short frame.";
    let view = View::new(view::Node::label(source));
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(48, 16), &mut layout_engine);
    let frame = layout
        .find_role(view::Role::Label)
        .into_iter()
        .next()
        .expect("author text frame should exist");

    assert_eq!(frame.label_text(), Some(source));
    assert_eq!(
        layout_engine.take_text_diagnostics().author_text_overflows,
        1
    );
}

#[test]
fn text_editor_view_composes_to_layout_without_runtime_mutation() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let revision = app.revision();
    let mut layout_engine = layout::Engine::new();
    let _: &layout::Layout = &layout::Layout::compose(
        &projected,
        geometry::Size::new(800, 600),
        &mut layout_engine,
    );
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(800, 600),
        &mut layout_engine,
    );

    assert_eq!(layout.size(), geometry::Size::new(800, 600));
    assert_eq!(app.revision(), revision);
    assert_eq!(app.session().file_dialog(window), None);

    let menus = layout.find_role(view::Role::Menu);
    assert_eq!(menus.len(), 3);
    assert_eq!(menus[0].label_text(), Some("File"));
    assert_eq!(menus[1].label_text(), Some("Edit"));
    assert_eq!(menus[2].label_text(), Some("View"));

    let text_areas = layout.find_role(view::Role::TextArea);
    assert_eq!(text_areas.len(), 1);
    assert_eq!(text_areas[0].rect().y(), Theme::default().menu().bar_height);
    assert!(text_areas[0].rect().height() > 0);
    assert!(
        text_areas[0]
            .target()
            .expect("text area should expose a target")
            .captures()
    );

    let menu_hit = layout
        .hit_test(geometry::Point::new(10, 10))
        .expect("file menu should be hit");
    assert_eq!(menu_hit.frame().role(), view::Role::Menu);
    assert_eq!(menu_hit.frame().label_text(), Some("File"));
    assert!(matches!(
        menu_hit.action(),
        Some(view::Action::ToggleMenu(menu)) if menu.label() == "File"
    ));

    let text_hit = layout
        .hit_test(geometry::Point::new(10, 80))
        .expect("text area should be hit");
    assert_eq!(text_hit.frame().role(), view::Role::TextArea);
    assert!(matches!(text_hit.action(), Some(view::Action::Focus(_))));
}

#[test]
fn menu_bar_titles_use_their_own_padded_label_widths_without_gaps() {
    let view = View::new(
        view::Node::root().child(
            view::Node::menu_bar()
                .child(view::Node::menu("menu.file", "File"))
                .child(view::Node::menu("menu.selection", "Selection"))
                .child(view::Node::menu("menu.view", "V")),
        ),
    );
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(400, 120), &mut layout_engine);
    let menus = layout.find_role(view::Role::Menu);

    assert_eq!(menus.len(), 3);
    assert!(menus[1].rect().width() > menus[0].rect().width());
    assert!(menus[0].rect().width() > menus[2].rect().width());
    assert_eq!(menus[0].rect().right(), menus[1].rect().x());
    assert_eq!(menus[1].rect().right(), menus[2].rect().x());
}

#[test]
fn single_character_menu_titles_preserve_configured_side_padding() {
    let view = View::new(
        view::Node::root().child(
            view::Node::menu_bar()
                .child(view::Node::menu("menu.a", "A"))
                .child(view::Node::menu("menu.b", "B")),
        ),
    );
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(160, 80), &mut layout_engine);
    let menus = layout.find_role(view::Role::Menu);

    assert_eq!(menus.len(), 2);
    for menu in &menus {
        assert_eq!(menu.rect().height(), Theme::default().menu().bar_height);
        assert!(
            menu.rect().width() >= Theme::default().menu().padding.saturating_mul(2),
            "even a one-character menu title keeps both configured side insets"
        );
    }
    assert_eq!(menus[0].rect().right(), menus[1].rect().x());
}

#[test]
fn menu_bar_defaults_match_system_menu_scale() {
    let theme = Theme::default();

    assert_eq!(theme.typography().interface().size(), 12.0);
    assert_eq!(
        theme.typography().interface().weight(),
        text::document::Weight::Normal
    );
    assert_eq!(theme.control().height, 22);
    assert_eq!(theme.menu().bar_height, 22);
    assert_eq!(theme.menu().row_height, 22);
}

#[test]
fn menu_bar_intrinsic_height_matches_bar_content_height() {
    let theme = Theme::default();
    let view = View::new(
        view::Node::root().child(
            view::Node::stack(view::Axis::Vertical)
                .child(view::Node::menu_bar().child(view::Node::menu("menu.file", "File")))
                .child(view::Node::label("Below")),
        ),
    );
    let mut engine = layout::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(240, 120),
        &mut engine,
        &theme,
    );
    let menu = layout
        .find_role(view::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("File"))
        .expect("menu title should be laid out");
    let below = layout
        .find_role(view::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Below"))
        .expect("following label should be laid out");

    assert_eq!(menu.rect().height(), theme.menu().bar_height);
    assert_eq!(below.rect().y(), menu.rect().bottom());
}

#[test]
fn menu_bar_title_typography_uses_interface_domain() {
    let view = View::new(
        view::Node::root().child(
            view::Node::menu_bar()
                .child(view::Node::menu("menu.file", "File"))
                .child(view::Node::menu("menu.selection", "Selection")),
        ),
    );
    let body_large = Theme::from_toml_str(
        r##"
        [typography]
        body-size = 28.0
        "##,
    )
    .expect("body-large theme should parse");
    let interface_large = Theme::from_toml_str(
        r##"
        [typography]
        body-size = 28.0
        interface-size = 18.0
        interface-weight = "bold"
        "##,
    )
    .expect("interface-large theme should parse");
    let mut engine = layout::Engine::new();
    let default_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(400, 120),
        &mut engine,
        &Theme::default(),
    );
    let body_large_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(400, 120),
        &mut engine,
        &body_large,
    );
    let interface_large_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(400, 120),
        &mut engine,
        &interface_large,
    );
    let default_selection = default_layout
        .find_role(view::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Selection"))
        .expect("default selection menu should be laid out")
        .rect();
    let body_large_selection = body_large_layout
        .find_role(view::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Selection"))
        .expect("body-large selection menu should be laid out")
        .rect();
    let interface_large_selection = interface_large_layout
        .find_role(view::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Selection"))
        .expect("interface-large selection menu should be laid out")
        .rect();
    let scene = scene::Scene::paint_with_theme(&interface_large_layout, &interface_large);
    let selection_text = scene_text(&scene, "Selection");

    assert_eq!(body_large_selection.width(), default_selection.width());
    assert!(interface_large_selection.width() > default_selection.width());
    assert_eq!(selection_text.style().size(), 18.0);
    assert_eq!(
        selection_text.style().weight(),
        text::document::Weight::Bold
    );
}

#[test]
fn generic_scroll_measures_content_clips_children_and_paints_scrollbar() {
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.add(
                widget::Scroll::new()
                    .id("scroll.generic")
                    .height(view::Dimension::fixed(72))
                    .children(|ui| {
                        for index in 0..8 {
                            ui.label(format!("Row {index}"));
                        }
                    }),
            );
        });
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(220, 120), &mut layout_engine);
    let scroll = layout
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let viewport = scroll
        .viewport()
        .expect("scroll should resolve viewport geometry");

    assert_eq!(scroll.rect().height(), 72);
    assert_eq!(viewport.rect(), scroll.rect());
    assert!(viewport.content().height() > viewport.rect().height());
    assert_eq!(
        viewport.max_scroll().y(),
        viewport
            .content()
            .height()
            .saturating_sub(viewport.rect().height())
    );

    let mut theme = Theme::dark();
    theme.scrollbar_mut().metrics.policy = crate::theme::ScrollbarPolicy::GutterAlways;
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(220, 120),
        &mut layout_engine,
        &theme,
    );
    let scroll = layout
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out with gutter theme");
    let viewport = scroll
        .viewport()
        .expect("scroll should resolve gutter viewport geometry");
    let scene = scene::Scene::paint_with_theme(&layout, &theme);
    assert!(
        scene
            .primitives()
            .iter()
            .any(|primitive| matches!(primitive, scene::Primitive::Clip(clip) if clip.rect() == viewport.rect())),
        "scroll children should be clipped to the viewport"
    );
    let thumb = theme.scrollbar().appearance.thumb;
    assert!(
        scene
            .quads()
            .iter()
            .any(|quad| quad.fill() == thumb && rect_contains(scroll.rect(), quad.rect())),
        "scrollbar thumb should be projected from viewport geometry"
    );
}

#[test]
fn deferred_focus_outline_retains_its_generic_scroll_clip() {
    let focus = session::Focus::text("scroll.clipped.focus").keyboard();
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Clipped Focus"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.add(
                    widget::Scroll::new()
                        .id("scroll.clipped.focus")
                        .height(view::Dimension::fixed(48))
                        .children(|ui| {
                            ui.text_box(widget::TextBox::new("Focused field").focus(focus));
                            for index in 0..5 {
                                ui.label(format!("Following row {index}"));
                            }
                        }),
                );
            })
        });
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 100);
    assert!(app.focus(window, focus));
    let initial = app
        .show_scene(window, size)
        .expect("focused scroll should render");
    let scroll = first_scroll_frame(&initial);
    let viewport = scroll
        .viewport()
        .expect("scroll should expose viewport")
        .rect();
    app.scroll_at(
        window,
        size,
        frame_point_at(viewport),
        interaction::ScrollDelta::vertical(16),
    )
    .expect("scroll should be handled");

    let rendered = app
        .show_scene(window, size)
        .expect("partly clipped focus should render");
    assert!(rendered.property_only());
    assert!(std::sync::Arc::ptr_eq(initial.commit(), rendered.commit()));
    let focused = rendered
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("focused field should remain laid out");
    let focused_rect = focused.rect();
    let presented_focused = geometry::Rect::new(
        focused.rect().x(),
        focused.rect().y().saturating_sub(16),
        focused.rect().width(),
        focused.rect().height(),
    );
    assert!(presented_focused.y() < viewport.y());
    assert!(presented_focused.bottom() > viewport.y());

    assert_outline_is_scoped_by_clip(rendered.scene(), focused_rect, viewport);

    app.scroll_at(
        window,
        size,
        frame_point_at(viewport),
        interaction::ScrollDelta::vertical(64),
    )
    .expect("second scroll should be handled");
    let fully_clipped = app
        .show_scene(window, size)
        .expect("fully clipped focus should render");
    let focused = fully_clipped
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("focused field should remain laid out while clipped");
    let scroll_offset = fully_clipped
        .stack()
        .scroll_offset(scroll.node_id())
        .expect("focused scroll should carry a receipted property");
    let presented_focused = geometry::Rect::new(
        focused.rect().x().saturating_sub(scroll_offset.x()),
        focused.rect().y().saturating_sub(scroll_offset.y()),
        focused.rect().width(),
        focused.rect().height(),
    );
    assert!(presented_focused.bottom() <= viewport.y());
    assert_outline_is_scoped_by_clip(fully_clipped.scene(), focused.rect(), viewport);
}

#[test]
fn late_scrollbar_chrome_retains_its_owner_viewport_clip_for_paint_and_hit() {
    let focus = session::Focus::text("scroll.clipped.area");
    let source = (0..12)
        .map(|index| format!("Scrollable line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let view = View::new(
        view::Node::root().child(
            view::Node::stack(view::Axis::Vertical)
                .child(
                    view::Node::scroll()
                        .with_interaction_id("scroll.clipped.area.outer")
                        .with_style(view::Style::new().with_height(view::Dimension::fixed(42)))
                        .child(
                            view::Node::text_area_state(
                                view::TextArea::new(source)
                                    .with_focus(focus)
                                    .with_wrap(view::Wrap::None),
                            )
                            .with_style(view::Style::new().with_height(view::Dimension::fixed(72))),
                        ),
                )
                .child(view::Node::label("Below viewport")),
        ),
    );
    let mut theme = Theme::dark();
    theme.scrollbar_mut().metrics.policy = crate::theme::ScrollbarPolicy::GutterAlways;
    let mut engine = layout::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(240, 100),
        &mut engine,
        &theme,
    );
    let area = layout
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("nested text area");
    let clip = area.clip().expect("outer viewport clip").rect();
    let area_target = area.target().expect("text area scroll target");
    let chrome = layout
        .chrome()
        .iter()
        .find(|chrome| chrome.scroll_target() == area_target)
        .expect("text area should project scrollbar chrome");
    let track = chrome.track();
    assert!(
        rect_contains(clip, track),
        "text-area track {track:?} must consume the same clipped visible frame as other viewport species {clip:?}"
    );
    let escaped_point = geometry::Point::new(track.x(), clip.bottom() + 1);
    assert!(
        !chrome.accepts_hit(escaped_point),
        "the chrome hit scope must consume the same viewport clip as paint"
    );

    let scene = scene::Scene::paint_with_theme(&layout, &theme);
    assert_quad_is_scoped_by_active_clip(&scene, theme.scrollbar().appearance.thumb, track, clip);
}

#[test]
fn viewport_content_extent_equals_placed_child_bounds() {
    let padding = view::Padding::edges(5, 7, 3, 11);
    let scroll = view::Node::scroll()
        .with_interaction_id("scroll.placed")
        .with_style(
            view::Style::new()
                .with_height(view::Dimension::fixed(72))
                .with_padding(padding)
                .with_gap(3),
        )
        .child(view::Node::section_header("Application"))
        .child(
            view::Node::label("Fixed row")
                .with_style(view::Style::new().with_height(view::Dimension::fixed(31))),
        )
        .child(view::Node::label("Body row"));
    let view = View::new(view::Node::root().child(scroll));
    let mut engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 120), &mut engine);
    let scroll = layout
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let viewport = scroll
        .viewport()
        .expect("scroll should resolve viewport geometry");
    let children = immediate_scroll_child_frames(&layout, scroll);
    let child_right = children
        .iter()
        .map(|frame| frame.rect().right())
        .max()
        .expect("scroll should have children");
    let child_bottom = children
        .iter()
        .map(|frame| frame.rect().bottom())
        .max()
        .expect("scroll should have children");
    let expected = geometry::Size::new(
        viewport.rect().width().max(
            child_right
                .saturating_sub(viewport.rect().x())
                .saturating_add(padding.right()),
        ),
        viewport.rect().height().max(
            child_bottom
                .saturating_sub(viewport.rect().y())
                .saturating_add(padding.bottom()),
        ),
    );

    assert_eq!(viewport.content(), expected);
}

#[test]
fn viewport_max_scroll_reaches_last_placed_descendant() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Scroll To Last"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.add(
                    widget::Scroll::new()
                        .id("scroll.last")
                        .height(view::Dimension::fixed(72))
                        .children(|ui| {
                            for index in 0..12 {
                                ui.label(format!("Row {index}"));
                            }
                        }),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 140);
    let initial = app
        .show_scene(window, size)
        .expect("scroll scene should render");
    let scroll = initial
        .layout()
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    app.scroll_at(
        window,
        size,
        frame_point_at(
            scroll
                .viewport()
                .expect("scroll should expose viewport")
                .rect(),
        ),
        interaction::ScrollDelta::vertical(10_000),
    )
    .expect("scroll input should be handled");

    let rendered = app
        .show_scene(window, size)
        .expect("scroll scene should render after scroll");
    let scroll = rendered
        .layout()
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let viewport = scroll
        .viewport()
        .expect("scroll should expose viewport")
        .rect();
    let last = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.label_text() == Some("Row 11"))
        .expect("last row should be laid out");
    let offset = rendered
        .stack()
        .scroll_offset(scroll.node_id())
        .expect("scroll should carry its receipted property");
    let presented_last = geometry::Rect::new(
        last.rect().x().saturating_sub(offset.x()),
        last.rect().y().saturating_sub(offset.y()),
        last.rect().width(),
        last.rect().height(),
    );

    assert!(presented_last.bottom() <= viewport.bottom());
}

#[test]
fn grow_children_collapse_to_intrinsic_inside_scroll_axis() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Scroll::new()
                .height(view::Dimension::fixed(80))
                .children(|ui| {
                    ui.add(
                        widget::Element::new()
                            .label("Grow")
                            .height(view::Dimension::grow()),
                    );
                    ui.label("After");
                }),
        );
    });
    let mut engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(220, 120), &mut engine);
    let scroll = layout
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let grow = layout
        .find_role(view::Role::Panel)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Grow"))
        .expect("grow child should be laid out");
    let after = layout
        .find_role(view::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("After"))
        .expect("following label should be laid out");

    assert!(grow.rect().height() < scroll.rect().height());
    assert_eq!(after.rect().y(), grow.rect().bottom());
}

#[test]
fn justify_content_is_start_when_scroll_content_exceeds_viewport() {
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.add(
                widget::Scroll::new()
                    .height(view::Dimension::fixed(40))
                    .layout(|layout| layout.justify_content(view::Align::End))
                    .children(|ui| {
                        for index in 0..4 {
                            ui.add(
                                widget::Element::new()
                                    .label(format!("Row {index}"))
                                    .height(view::Dimension::fixed(24)),
                            );
                        }
                    }),
            );
        });
    });
    let mut engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(220, 100), &mut engine);
    let scroll = layout
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let first = layout
        .find_role(view::Role::Panel)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Row 0"))
        .expect("first row should be laid out");

    assert!(
        scroll
            .viewport()
            .expect("scroll should expose viewport")
            .is_scrollable()
    );
    assert_eq!(first.rect().y(), scroll.rect().y());
}

#[test]
fn generic_scroll_feedback_clamps_session_offset_after_present() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Generic Scroll"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.add(
                        widget::Scroll::new()
                            .id("scroll.feedback")
                            .height(view::Dimension::fixed(72))
                            .children(|ui| {
                                for index in 0..8 {
                                    ui.label(format!("Row {index}"));
                                }
                            }),
                    );
                });
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let initial = app
        .show_scene(window, size)
        .expect("scroll view should render");
    let scroll = initial
        .layout()
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let expected_max_scroll = scroll
        .viewport()
        .expect("scroll should resolve viewport geometry")
        .max_scroll();
    let target = scroll
        .target()
        .expect("scroll should expose a target")
        .clone();
    let scroll_node = scroll.node_id();
    assert!(
        initial
            .layout()
            .scroll_property_accepts(&target, expected_max_scroll),
        "a fully realized ordinary scroll must admit its maximum property offset"
    );
    assert!(
        app.presented_layout(window)
            .is_some_and(|layout| layout.scroll_property_accepts(&target, expected_max_scroll)),
        "runtime routing must retain the same accepted layout"
    );

    let point = frame_point_at(scroll.rect());
    let delta = interaction::ScrollDelta::vertical(400);
    assert_eq!(
        initial.layout().scroll_target_at(point, delta),
        Some(target.clone())
    );
    app.scroll_at(window, size, point, delta)
        .expect("scroll input should be handled");
    let interaction = app
        .session()
        .interaction(window)
        .expect("scroll input should retain interaction");
    assert_eq!(
        interaction.scroll().desired_offset(&target),
        expected_max_scroll
    );
    assert_eq!(interaction.scroll().offset(&target), expected_max_scroll);
    assert!(
        app.session()
            .window(window)
            .expect("scroll window should remain open")
            .property_tick_requested(),
        "an admitted ordinary wheel scroll must request a property tick"
    );
    let presented = app
        .show_scene(window, size)
        .expect("scroll feedback should render");
    assert!(presented.property_only());
    assert_eq!(
        presented.stack().scroll_offset(scroll_node),
        Some(expected_max_scroll),
        "the successful presentation must carry the exact admitted property"
    );

    let offset = app
        .session()
        .interaction(window)
        .expect("window should have interaction")
        .scroll()
        .offset(&target);

    assert_eq!(offset, expected_max_scroll);
}

#[test]
fn gutter_scrollbar_metrics_reduce_viewport_width() {
    let mut theme = Theme::dark();
    theme.scrollbar_mut().metrics.policy = crate::theme::ScrollbarPolicy::GutterAlways;
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.add(
                widget::Scroll::new()
                    .id("scroll.gutter")
                    .height(view::Dimension::fixed(72))
                    .children(|ui| {
                        for index in 0..8 {
                            ui.label(format!("Row {index}"));
                        }
                    }),
            );
        });
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(220, 120),
        &mut layout_engine,
        &theme,
    );
    let scroll = layout
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let viewport = scroll
        .viewport()
        .expect("scroll should resolve viewport geometry");

    assert!(viewport.rect().width() < scroll.rect().width());
    assert_eq!(
        scroll
            .rect()
            .width()
            .saturating_sub(viewport.rect().width()),
        theme.scrollbar().metrics.thickness
    );
}

#[test]
fn generic_scroll_pointer_drag_updates_viewport_offset() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Generic Scroll Drag"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.add(
                        widget::Scroll::new()
                            .id("scroll.drag")
                            .height(view::Dimension::fixed(72))
                            .children(|ui| {
                                for index in 0..8 {
                                    ui.label(format!("Row {index}"));
                                }
                            }),
                    );
                });
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let initial = app
        .show_scene(window, size)
        .expect("scroll view should render");
    let scroll = initial
        .layout()
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out");
    let target = scroll
        .target()
        .expect("scroll should expose a target")
        .clone();
    let scroll_node = scroll.node_id();
    let expected_max_scroll = scroll
        .viewport()
        .expect("scroll should resolve viewport geometry")
        .max_scroll();
    let track = initial
        .layout()
        .chrome()
        .iter()
        .map(layout::Chrome::track)
        .next()
        .expect("scrollbar chrome should be projected");
    let scrollbar_target = initial.layout().chrome()[0].target().clone();
    let press = geometry::Point::new(track.x().saturating_add(track.width() / 2), track.y() + 1);
    let drag = geometry::Point::new(
        track.x().saturating_add(track.width() / 2),
        track.bottom().saturating_sub(1),
    );

    app.pointer_down_at(window, size, press)
        .expect("scroll pointer down should be handled");
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .map(interaction::pointer::Capture::target),
        Some(&scrollbar_target),
        "the scrollbar that won hit testing must own capture"
    );
    app.pointer_drag_at(window, size, drag)
        .expect("scroll pointer drag should be handled");
    let scroll = app
        .session()
        .interaction(window)
        .expect("window should have interaction")
        .scroll();
    assert_eq!(
        scroll.offset(&target),
        expected_max_scroll,
        "a fully resident ordinary scroll admits through the shared property path"
    );
    assert_eq!(scroll.desired_offset(&target), expected_max_scroll);
    let presented = app
        .show_scene(window, size)
        .expect("scroll feedback should render");
    assert_eq!(
        presented.stack().scroll_offset(scroll_node),
        Some(expected_max_scroll),
        "the presented property snapshot must contain the admitted scroll"
    );

    let offset = app
        .session()
        .interaction(window)
        .expect("window should have interaction")
        .scroll()
        .offset(&target);

    assert_eq!(offset, expected_max_scroll);
}

#[test]
fn older_successful_scroll_receipt_cannot_regress_admitted_property() {
    let mut app = scroll_app();
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let initial = app
        .show_scene(window, size)
        .expect("scroll view should render");
    let scroll = first_scroll_frame(&initial);
    let target = scroll
        .target()
        .expect("scroll should expose a target")
        .clone();
    let node = scroll.node_id();
    let point = frame_point_at(scroll.rect());

    app.scroll_at(window, size, point, interaction::ScrollDelta::vertical(16))
        .expect("first scroll should be admitted");
    let older = app
        .render_scene(window, size)
        .expect("older property candidate should prepare");
    assert_eq!(
        older.stack().scroll_offset(node),
        Some(interaction::ScrollOffset::new(0, 16))
    );

    app.scroll_at(window, size, point, interaction::ScrollDelta::vertical(16))
        .expect("second scroll should be admitted");
    let newer = app
        .render_scene(window, size)
        .expect("newer property candidate should prepare");
    assert!(newer.epoch() > older.epoch());
    assert_eq!(
        newer.stack().scroll_offset(node),
        Some(interaction::ScrollOffset::new(0, 32))
    );

    for candidate in [&newer, &older] {
        app.finish_render_report(
            window,
            candidate.epoch(),
            candidate.invalidation(),
            candidate.layout(),
            candidate.stack(),
            candidate.property_only(),
            diagnostics::RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now()),
        );
    }
    app.finish_active_refresh(
        window,
        older.epoch(),
        older.invalidation(),
        older.layout(),
        older.stack(),
        diagnostics::RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now()),
    );

    assert_eq!(
        app.session()
            .interaction(window)
            .expect("scroll interaction should remain")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 32)
    );
    assert_eq!(
        app.presented_properties(window)
            .and_then(|properties| properties.scroll_offset(node)),
        Some(interaction::ScrollOffset::new(0, 32))
    );
    assert_eq!(
        app.acknowledged_presentation_epoch(window),
        Some(newer.epoch())
    );
}

#[test]
fn in_window_scroll_inputs_coalesce_into_one_literal_zero_property_tick() {
    let mut app = scroll_app();
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let initial = app
        .show_scene(window, size)
        .expect("scroll view should warm its retained commit");
    let scroll = first_scroll_frame(&initial);
    let point = frame_point_at(scroll.rect());
    let target = scroll.target().expect("scroll target").clone();
    let projection = initial
        .layout()
        .scroll_projections()
        .iter()
        .find(|projection| projection.target() == &target)
        .cloned()
        .expect("scroll should declare retained property topology");
    let initial_serial = initial.properties().serial();
    let initial_commit = std::sync::Arc::clone(initial.commit());

    app.diagnostics_mut(window)
        .expect("window diagnostics")
        .begin_renderer_measurement();
    for tick in 0..3 {
        let outcome = app
            .scroll_at(window, size, point, interaction::ScrollDelta::vertical(8))
            .expect("each in-window delta should be accepted");
        assert_eq!(
            outcome.effect(),
            &response::Effect::None,
            "in-window delta {tick} must stay off the content clock"
        );
    }
    let absolute = app
        .handle_input(
            window,
            Input::scroll_to(target.clone(), interaction::ScrollOffset::new(0, 32)),
        )
        .expect("in-window absolute scroll should be accepted");
    assert_eq!(
        absolute.effect(),
        &response::Effect::None,
        "wheel and scrollbar positions must share the property-tick admission path"
    );
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window interaction")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 32)
    );

    let tick = app
        .render_scene(window, size)
        .expect("coalesced property tick should prepare once");
    assert!(tick.property_only());
    assert!(std::sync::Arc::ptr_eq(&initial_commit, tick.commit()));
    assert!(tick.properties().serial() > initial_serial);
    assert_eq!(
        tick.properties().scroll_offset(projection.node()),
        Some(interaction::ScrollOffset::new(0, 32))
    );
    app.finish_render_report(
        window,
        tick.epoch(),
        tick.invalidation(),
        tick.layout(),
        tick.stack(),
        tick.property_only(),
        diagnostics::RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now()),
    );

    let render = &app.diagnostics(window).expect("window diagnostics").render;
    assert_eq!(render.property_ticks, 1);
    assert_eq!(render.changed_property_values, 1);
    assert_eq!(render.semantic_commits_created, 0);
    assert_eq!(render.scene_nodes_rebuilt, 0);
    assert_eq!(render.scene_paint_calls, 0);
    assert_eq!(
        render.visible_property_serial,
        tick.properties().serial().value()
    );
}

#[test]
fn retained_scroll_layer_preserves_the_content_owners_actual_runway() {
    let mut app = nested_clipped_scroll_app();
    app.start();
    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(240, 180))
        .expect("nested scrolls should render");
    let projections = rendered.layout().scroll_projections();

    assert_eq!(projections.len(), 2, "both scroll owners need projections");
    assert!(std::ptr::eq(
        projections,
        rendered.layout().scroll_projections()
    ));
    for projection in projections {
        let viewport = projection.viewport();
        let visible = viewport.visible_content();
        let bounds = projection.layer_bounds();
        let resident = projection
            .resident_bounds()
            .expect("rendered scroll projection must have complete residency");
        assert!(bounds.x() <= visible.x());
        assert!(bounds.y() <= visible.y());
        assert!(bounds.right() >= visible.right());
        assert!(bounds.bottom() >= visible.bottom());
        assert_eq!(resident, bounds);
        assert!(resident.x() <= visible.x());
        assert!(resident.y() <= visible.y());
        assert!(resident.right() >= visible.right());
        assert!(resident.bottom() >= visible.bottom());
    }
}

#[test]
fn control_gallery_keeps_viewport_clips_outside_repeated_nested_scroll_scopes() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(760, 660))
        .expect("control gallery should render");
    let table_scrolls = rendered
        .layout()
        .scroll_projections()
        .iter()
        .filter(|projection| projection.target().kind() != interaction::Kind::TextArea)
        .map(layout::ScrollProjection::node)
        .collect::<std::collections::HashSet<_>>();
    let mut group_depth = 0_usize;
    let mut clip_depth = 0_usize;
    let mut scroll_depth = 0_usize;
    let mut scrolls = Vec::new();
    for draw in rendered
        .stack()
        .base()
        .drawable_commit()
        .order()
        .expect("gallery uses explicit order")
    {
        match draw {
            scene::Draw::PushGroup { .. } => group_depth += 1,
            scene::Draw::PopGroup => group_depth = group_depth.saturating_sub(1),
            scene::Draw::PushClip { .. } => clip_depth += 1,
            scene::Draw::PopClip => clip_depth = clip_depth.saturating_sub(1),
            scene::Draw::PushScroll { node } => {
                if table_scrolls.contains(node) {
                    scrolls.push((*node, group_depth, clip_depth, scroll_depth));
                }
                scroll_depth += 1;
            }
            scene::Draw::PopScroll => scroll_depth = scroll_depth.saturating_sub(1),
            scene::Draw::Content { .. } => {}
        }
    }
    let unique = scrolls
        .iter()
        .map(|(node, ..)| *node)
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        unique.len(),
        2,
        "gallery table should retain two scroll owners"
    );
    assert!(
        scrolls
            .iter()
            .all(|(_, group, clip, _)| *group == 0 && *clip > 0),
        "every repeated scroll scope must begin inside a fixed viewport clip: {scrolls:?}"
    );
    let outer = scrolls
        .iter()
        .filter(|(_, _, _, depth)| *depth == 0)
        .map(|(node, ..)| *node)
        .collect::<Vec<_>>();
    let nested = scrolls
        .iter()
        .filter(|(_, _, _, depth)| *depth == 1)
        .map(|(node, ..)| *node)
        .collect::<Vec<_>>();
    assert!(outer.len() >= 2 && outer.iter().all(|node| *node == outer[0]));
    assert!(nested.len() >= 2 && nested.iter().all(|node| *node == nested[0]));
    assert_ne!(outer[0], nested[0]);

    let layout = rendered.layout();
    let owner = layout
        .frames()
        .iter()
        .find(|frame| {
            table_scrolls.contains(&frame.node_id()) && frame.virtual_list_request().is_some()
        })
        .expect("the gallery table must expose its virtual residency owner");
    let request = owner
        .virtual_list_request()
        .expect("the table residency owner must name its contiguous request");
    let requested_roots = layout
        .frames()
        .iter()
        .filter(|frame| {
            frame.provided_row().is_some_and(|row| {
                row.list() == request.id() && request.range().contains(&row.index())
            })
        })
        .collect::<Vec<_>>();
    assert!(!requested_roots.is_empty());
    let expected = layout
        .frames()
        .iter()
        .filter(|frame| {
            if frame.node_id() == owner.node_id() {
                return true;
            }
            layout.scroll_ancestry(frame.node_id()).last() == Some(&owner.node_id())
                && requested_roots
                    .iter()
                    .any(|root| frame.node_id() == root.node_id() || frame.is_descendant_of(root))
        })
        .map(layout::Frame::node_id)
        .collect::<std::collections::HashSet<_>>();
    let residency = rendered
        .stack()
        .base()
        .residencies()
        .iter()
        .find(|residency| residency.scroll() == owner.node_id())
        .expect("the complete virtual table must carry local scene residency");
    let actual = residency
        .node_ids()
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        actual, expected,
        "virtual table residency must contain exactly its requested row roots and nearest-scroll descendants"
    );

    let drawable = rendered.stack().base().drawable_commit();
    let painted_expected = drawable
        .nodes()
        .iter()
        .filter(|node| expected.contains(&node.id()) && !node.content().is_empty())
        .map(|node| node.id())
        .collect::<std::collections::HashSet<_>>();
    let painted_actual = residency
        .draw_order()
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        painted_actual, painted_expected,
        "every painted descendant in the admitted row range must be bound into the residency draw order"
    );
}

#[test]
fn control_gallery_property_tick_does_not_move_the_table_viewport_clip() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 700);
    let initial = app
        .show_scene(window, size)
        .expect("control gallery should render");
    let cell = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(1)
                    && cell.column() == interaction::Id::new("detail")
            })
        })
        .expect("gallery should materialize the table witness cell");
    let point = geometry::Point::new(cell.rect().x() + 1, cell.rect().y() + 1);
    let target = initial
        .layout()
        .scroll_target_at(point, interaction::ScrollDelta::vertical(4))
        .expect("table witness should route vertical scrolling");
    let viewport = initial
        .layout()
        .scroll_projections()
        .iter()
        .find(|projection| {
            projection.target() == &target && projection.viewport().max_scroll().y() > 0
        })
        .map(|projection| projection.viewport().visible_content())
        .expect("table should expose its visible viewport");

    app.scroll_at(window, size, point, interaction::ScrollDelta::vertical(4))
        .expect("small table scroll should be accepted by active residency");
    let tick = app
        .render_scene(window, size)
        .expect("property tick should render");
    assert!(tick.property_only());

    let translated = geometry::Rect::new(
        viewport.x(),
        viewport.y().saturating_sub(4),
        viewport.width(),
        viewport.height(),
    );
    assert!(
        tick.scene().primitives().iter().any(
            |primitive| matches!(primitive, scene::Primitive::Clip(clip) if clip.rect() == viewport)
        ),
        "the table property tick must retain its fixed viewport clip"
    );
    assert!(
        !tick.scene().primitives().iter().any(
            |primitive| matches!(primitive, scene::Primitive::Clip(clip) if clip.rect() == translated)
        ),
        "the viewport clip must not translate with row content and expose a bottom-edge band"
    );
}

#[test]
fn viewport_intrinsics_ignore_content_extent() {
    let mut theme = Theme::dark();
    theme.viewport_mut().min_viewport_extent = 64;
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.add(
                widget::Scroll::new()
                    .height(view::Dimension::fit())
                    .children(|ui| {
                        for index in 0..12 {
                            ui.label(format!("Tall row {index}"));
                        }
                    }),
            );
        });
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(240, 400),
        &mut layout_engine,
        &theme,
    );
    let scroll = layout
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("fit scroll should be laid out");
    let viewport = scroll.viewport().expect("scroll should expose viewport");

    assert_eq!(scroll.rect().height(), 64);
    assert!(viewport.content().height() > scroll.rect().height());
}

#[test]
fn scrollbar_thumb_wins_hit_test_over_content() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Scroll::new()
                .id("scroll.hit")
                .height(view::Dimension::fixed(72))
                .children(|ui| {
                    for index in 0..8 {
                        ui.button(widget::Button::new(format!("Row {index}")).trigger::<Ping>(()));
                    }
                }),
        );
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(220, 120), &mut layout_engine);
    let track = first_scrollbar_track(&layout);
    let hit = layout
        .hit_test(geometry::Point::new(
            track.x().saturating_add(track.width() / 2),
            track.y().saturating_add(track.height() / 2),
        ))
        .expect("scrollbar chrome should be hit");

    assert!(hit.is_chrome());
    assert_eq!(
        hit.target().expect("chrome should expose target").kind(),
        interaction::Kind::Scrollbar
    );
}

#[test]
fn scrollbar_hover_envelope_wins_hit_test_over_content() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Scroll::new()
                .id("scroll.hover-envelope")
                .height(view::Dimension::fixed(72))
                .children(|ui| {
                    for index in 0..8 {
                        ui.button(widget::Button::new(format!("Row {index}")).trigger::<Ping>(()));
                    }
                }),
        );
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(220, 120), &mut layout_engine);
    let chrome = layout
        .chrome()
        .first()
        .expect("scrollbar chrome should be projected");
    let track = chrome.track();
    let interaction_track = chrome.interaction_track();
    let point = geometry::Point::new(
        interaction_track.x().saturating_add(1),
        interaction_track
            .y()
            .saturating_add(interaction_track.height() / 2),
    );

    assert!(interaction_track.width() > track.width());
    assert!(!track.contains(point));
    let hit = layout
        .hit_test(point)
        .expect("expanded visible scrollbar band should be hit");
    assert!(hit.is_chrome());
    assert_eq!(
        hit.target().expect("chrome should expose target").kind(),
        interaction::Kind::Scrollbar
    );
}

#[test]
fn table_scrollbar_chrome_never_participates_in_the_row_beneath_it() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Table scrollbar ownership"));
        })
        .view(|_, _| {
            widget::view_node(
                crate::Table::new(
                    "scrollbar.owner.table",
                    20,
                    [
                        crate::table::Column::new("name", "Name", view::Dimension::fixed(120)),
                        crate::table::Column::new("detail", "Detail", view::Dimension::fixed(180)),
                    ],
                    MillionTableProvider {
                        cell_calls: Rc::new(Cell::new(0)),
                    },
                )
                .height(view::Dimension::fixed(128)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 128);
    let initial = app
        .show_scene(window, size)
        .expect("virtual table should render");
    let scrollbar = initial
        .layout()
        .chrome()
        .iter()
        .find(|chrome| chrome.target().label_text() == "Vertical Scrollbar")
        .expect("virtual table should project vertical chrome");
    let band = scrollbar.interaction_track();
    let point = geometry::Point::new(
        band.x().saturating_add(band.width() / 2),
        band.y().saturating_add(band.height() / 2),
    );
    let scrollbar_target = scrollbar.target().clone();
    let table = interaction::Id::new("scrollbar.owner.table");
    let selection_before = app.session().selection(window, table).cloned();
    let active_cell_before = app.session().active_table_cell(window, table);

    app.pointer_down_at(window, size, point)
        .expect("scrollbar press should be handled");

    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .map(interaction::pointer::Capture::target),
        Some(&scrollbar_target),
        "chrome must own the press and capture"
    );
    assert_eq!(
        app.session().selection(window, table),
        selection_before.as_ref(),
        "vertical chrome must not select the virtual row beneath it"
    );
    assert_eq!(
        app.session().active_table_cell(window, table),
        active_cell_before,
        "vertical chrome must not change the active cell beneath it"
    );
}

#[test]
fn overlay_auto_hides_idle_appears_after_activity_and_fades_out() {
    let mut app = scroll_app();
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let theme = Theme::dark();
    let now = std::time::Instant::now();

    let idle = app
        .show_scene_at(window, size, now)
        .expect("idle scroll view should render");
    let scroll = first_scroll_frame(&idle);
    assert!(
        !scene_has_scrollbar_thumb(idle.scene(), &theme, scroll.rect()),
        "overlay scrollbar should be hidden before activity"
    );

    app.scroll_at(
        window,
        size,
        frame_point_at(scroll.rect()),
        interaction::ScrollDelta::vertical(80),
    )
    .expect("scroll should be handled");

    let activity_at = now + std::time::Duration::from_millis(10);
    app.show_scene_at(window, size, activity_at)
        .expect("activity frame should start scrollbar fade-in");
    let visible_at = activity_at + std::time::Duration::from_millis(260);
    let visible = app
        .show_scene_at(window, size, visible_at)
        .expect("active scroll view should render");
    let scroll = first_scroll_frame(&visible);
    assert!(
        scene_has_scrollbar_thumb(visible.scene(), &theme, scroll.rect()),
        "overlay scrollbar should appear after scroll activity"
    );

    let fade_start =
        activity_at + std::time::Duration::from_millis(theme.scrollbar().appearance.fade_delay_ms);
    app.show_scene_at(window, size, fade_start)
        .expect("fade deadline should render");
    let faded = app
        .show_scene_at(
            window,
            size,
            fade_start
                + std::time::Duration::from_millis(
                    theme.scrollbar().appearance.fade_duration_ms + 20,
                ),
        )
        .expect("faded scroll view should render");
    let scroll = first_scroll_frame(&faded);
    assert!(
        !scene_has_scrollbar_thumb(faded.scene(), &theme, scroll.rect()),
        "overlay scrollbar should fade out after inactivity"
    );
}

#[test]
fn two_axis_table_activity_and_fade_follow_one_scroll_owner() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Two-axis table activity"));
        })
        .view(|_, _| {
            widget::view_node(
                crate::Table::new(
                    "two.axis.activity.table",
                    20,
                    [
                        crate::table::Column::new("name", "Name", view::Dimension::fixed(100)),
                        crate::table::Column::new("detail", "Detail", view::Dimension::fixed(120)),
                        crate::table::Column::new("action", "Action", view::Dimension::fixed(90)),
                    ],
                    MillionTableProvider {
                        cell_calls: Rc::new(Cell::new(0)),
                    },
                )
                .height(view::Dimension::fixed(108)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 108);
    let theme = Theme::dark();
    let now = std::time::Instant::now();
    let initial = app
        .show_scene_at(window, size, now)
        .expect("two-axis table should render");
    let table_target = initial
        .layout()
        .chrome()
        .iter()
        .find(|chrome| chrome.axis() == interaction::ScrollbarAxis::Horizontal)
        .expect("wide table should expose horizontal chrome")
        .scroll_target()
        .clone();
    let chrome = initial
        .layout()
        .chrome()
        .iter()
        .filter(|chrome| chrome.scroll_target() == &table_target)
        .map(|chrome| (chrome.axis(), chrome.owner()))
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(chrome.len(), 2, "both axes must share one scroll target");
    let body_point = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell().is_some())
        .map(|frame| frame_point_at(frame.rect()))
        .expect("table body should expose a scroll input point");

    for (axis, owner) in &chrome {
        assert_eq!(
            initial
                .properties()
                .scrollbar(*owner, *axis)
                .map(|value| value.0),
            Some(0.0),
            "both overlay bars should begin idle"
        );
    }

    app.scroll_at(
        window,
        size,
        body_point,
        interaction::ScrollDelta::vertical(80),
    )
    .expect("vertical table movement should be handled");
    let activity_at = now + std::time::Duration::from_millis(10);
    app.show_scene_at(window, size, activity_at)
        .expect("activity should begin the shared fade-in");
    let visible_at = activity_at
        + std::time::Duration::from_millis(theme.scrollbar().appearance.fade_duration_ms + 20);
    let visible = app
        .show_scene_at(window, size, visible_at)
        .expect("both active bars should render");
    for (axis, owner) in &chrome {
        assert!(
            visible
                .properties()
                .scrollbar(*owner, *axis)
                .is_some_and(|value| value.0 > 0.99),
            "{axis:?} bar must become visible from activity on the shared owner"
        );
    }

    let fade_start =
        activity_at + std::time::Duration::from_millis(theme.scrollbar().appearance.fade_delay_ms);
    app.show_scene_at(window, size, fade_start)
        .expect("shared inactivity deadline should begin fade-out");
    let faded = app
        .show_scene_at(
            window,
            size,
            fade_start
                + std::time::Duration::from_millis(
                    theme.scrollbar().appearance.fade_duration_ms + 20,
                ),
        )
        .expect("both inactive bars should render their hidden state");
    for (axis, owner) in chrome {
        assert_eq!(
            faded
                .properties()
                .scrollbar(owner, axis)
                .map(|value| value.0),
            Some(0.0),
            "{axis:?} bar must fade from the same inactivity clock"
        );
    }
}

#[test]
fn two_axis_table_scrollbar_capture_and_mutation_are_axis_symmetric() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Two-axis table scrollbar input"));
        })
        .view(|_, _| {
            widget::view_node(
                crate::Table::new(
                    "two.axis.input.table",
                    20,
                    [
                        crate::table::Column::new("name", "Name", view::Dimension::fixed(100)),
                        crate::table::Column::new("detail", "Detail", view::Dimension::fixed(120)),
                        crate::table::Column::new("action", "Action", view::Dimension::fixed(90)),
                    ],
                    MillionTableProvider {
                        cell_calls: Rc::new(Cell::new(0)),
                    },
                )
                .height(view::Dimension::fixed(108)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 108);
    let initial = app
        .show_scene(window, size)
        .expect("two-axis table should render");
    let table_target = initial
        .layout()
        .chrome()
        .iter()
        .find(|chrome| chrome.axis() == interaction::ScrollbarAxis::Horizontal)
        .expect("wide table horizontal chrome")
        .scroll_target()
        .clone();
    let body_point = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell().is_some())
        .map(|frame| frame_point_at(frame.rect()))
        .expect("table body input point");
    app.scroll_at(
        window,
        size,
        body_point,
        interaction::ScrollDelta::vertical(80),
    )
    .expect("vertical wheel should establish a nonzero other-axis value");
    let vertically_scrolled = app
        .show_scene(window, size)
        .expect("vertical property should present");
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("table interaction")
            .scroll()
            .offset(&table_target)
            .y(),
        80
    );

    let horizontal = vertically_scrolled
        .layout()
        .chrome()
        .iter()
        .find(|chrome| {
            chrome.scroll_target() == &table_target
                && chrome.axis() == interaction::ScrollbarAxis::Horizontal
        })
        .expect("horizontal table chrome");
    let horizontal_target = horizontal.target().clone();
    let horizontal_track = horizontal.track();
    let horizontal_max = horizontal.maximum_offset();
    let horizontal_press = geometry::Point::new(
        horizontal_track.x().saturating_add(1),
        horizontal_track
            .y()
            .saturating_add(horizontal_track.height() / 2),
    );
    let horizontal_drag = geometry::Point::new(
        horizontal_track.right().saturating_sub(1),
        horizontal_press.y(),
    );
    app.pointer_down_at(window, size, horizontal_press)
        .expect("horizontal scrollbar should capture");
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .map(interaction::pointer::Capture::target),
        Some(&horizontal_target)
    );
    app.pointer_drag_at(window, size, horizontal_drag)
        .expect("horizontal scrollbar drag should mutate x");
    let after_horizontal_drag = app
        .session()
        .interaction(window)
        .expect("table interaction")
        .scroll()
        .desired_offset(&table_target);
    assert_eq!(
        after_horizontal_drag,
        interaction::ScrollOffset::new(horizontal_max, 80),
        "horizontal chrome must preserve the desired vertical component"
    );
    app.pointer_up_at(window, size, horizontal_drag)
        .expect("horizontal capture should release");
    let horizontally_scrolled = app
        .show_scene(window, size)
        .expect("horizontal property should present");
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("table interaction")
            .scroll()
            .offset(&table_target),
        after_horizontal_drag,
        "receipt aggregation must retain both projected axes"
    );

    let vertical = horizontally_scrolled
        .layout()
        .chrome()
        .iter()
        .find(|chrome| {
            chrome.scroll_target() == &table_target
                && chrome.axis() == interaction::ScrollbarAxis::Vertical
        })
        .expect("vertical table chrome");
    let vertical_target = vertical.target().clone();
    let vertical_track = vertical.track();
    let vertical_max = vertical.maximum_offset();
    let vertical_press = geometry::Point::new(
        vertical_track
            .x()
            .saturating_add(vertical_track.width() / 2),
        vertical_track.y().saturating_add(1),
    );
    let vertical_drag = geometry::Point::new(
        vertical_press.x(),
        vertical_track.bottom().saturating_sub(1),
    );
    app.pointer_down_at(window, size, vertical_press)
        .expect("vertical scrollbar should capture");
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .map(interaction::pointer::Capture::target),
        Some(&vertical_target)
    );
    app.pointer_drag_at(window, size, vertical_drag)
        .expect("vertical scrollbar drag should mutate y");
    let after_vertical_drag = app
        .session()
        .interaction(window)
        .expect("table interaction")
        .scroll()
        .desired_offset(&table_target);
    assert_eq!(
        after_vertical_drag,
        interaction::ScrollOffset::new(horizontal_max, vertical_max),
        "vertical chrome must preserve the desired horizontal component"
    );
    app.pointer_up_at(window, size, vertical_drag)
        .expect("vertical capture should release");
    app.show_scene(window, size)
        .expect("far vertical residency should present");
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("table interaction")
            .scroll()
            .offset(&table_target),
        after_vertical_drag
    );
}

#[test]
fn gutter_always_reserves_base_gutter_and_remains_visible() {
    let mut theme = Theme::dark();
    theme.scrollbar_mut().metrics.policy = crate::theme::ScrollbarPolicy::GutterAlways;
    let mut app = scroll_app().theme(move |_| theme.clone());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let rendered = app
        .show_scene(window, size)
        .expect("gutter scroll view should render");
    let scroll = first_scroll_frame(&rendered);
    let viewport = scroll.viewport().expect("scroll should expose viewport");
    let theme = Theme::dark();

    assert_eq!(
        scroll
            .rect()
            .width()
            .saturating_sub(viewport.rect().width()),
        theme.scrollbar().metrics.thickness
    );
    assert!(
        scene_has_scrollbar_thumb(rendered.scene(), &theme, scroll.rect()),
        "gutter scrollbar should paint while idle"
    );
}

#[test]
fn two_axis_scroll_gutters_follow_viewport_capability_not_stack_axis() {
    let view = widget::view_node(
        view::Node::scroll()
            .with_interaction_id("two.axis.gutter")
            .with_layout_axis(view::Axis::Overlay)
            .with_style(
                view::Style::new()
                    .with_width(view::Dimension::fixed(120))
                    .with_height(view::Dimension::fixed(90)),
            )
            .child(
                view::Node::panel().with_style(
                    view::Style::new()
                        .with_width(view::Dimension::fixed(240))
                        .with_height(view::Dimension::fixed(180)),
                ),
            ),
    );
    let mut theme = Theme::dark();
    theme.scrollbar_mut().metrics.policy = crate::theme::ScrollbarPolicy::GutterAlways;
    let mut engine = layout::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(120, 90),
        &mut engine,
        &theme,
    );
    let frame = layout
        .frames()
        .iter()
        .find(|frame| frame.role() == view::Role::Scroll)
        .expect("two-axis scroll frame");
    let viewport = frame.viewport().expect("two-axis viewport");
    let gutter = theme.scrollbar().metrics.thickness;

    assert_eq!(frame.rect().width() - viewport.rect().width(), gutter);
    assert_eq!(frame.rect().height() - viewport.rect().height(), gutter);
}

#[test]
fn text_area_uses_the_same_two_axis_gutter_geometry_as_other_viewports() {
    let document = (0..120)
        .map(|line| format!("text area line {line:03} {}", "wide".repeat(80)))
        .collect::<Vec<_>>()
        .join("\n");
    let mut theme = Theme::dark();
    theme.scrollbar_mut().metrics.policy = crate::theme::ScrollbarPolicy::GutterAlways;
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        wrap_text: false,
        ..text_editor::State::default()
    })
    .theme(move |_| theme.clone());
    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(520, 180))
        .expect("text editor should render with gutter policy");
    let frame = rendered
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area frame");
    let viewport = frame.viewport().expect("text area viewport");
    let gutter = Theme::dark().scrollbar().metrics.thickness;

    assert_eq!(frame.rect().width() - viewport.rect().width(), gutter);
    assert_eq!(frame.rect().height() - viewport.rect().height(), gutter);
    assert_eq!(viewport.visible_frame(), frame.rect());
    assert_eq!(viewport.visible_content(), viewport.rect());
    assert_eq!(
        rendered
            .layout()
            .chrome()
            .iter()
            .filter(|chrome| chrome.scroll_target() == frame.target().expect("text target"))
            .map(layout::Chrome::axis)
            .collect::<std::collections::HashSet<_>>(),
        [
            interaction::ScrollbarAxis::Horizontal,
            interaction::ScrollbarAxis::Vertical,
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn two_axis_text_scrollbars_share_activity_but_keep_per_axis_hover_and_mutation() {
    let document = (0..120)
        .map(|line| format!("wide text line {line:03} {}", "horizontal".repeat(80)))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        wrap_text: false,
        ..text_editor::State::default()
    });
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(520, 180);
    let now = std::time::Instant::now();
    let theme = Theme::dark();
    let initial = app
        .show_scene_at(window, size, now)
        .expect("wide, long text should render");
    let text = initial
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area frame");
    let text_target = text.target().expect("text scroll target").clone();
    let chrome = initial
        .layout()
        .chrome()
        .iter()
        .filter(|chrome| chrome.scroll_target() == &text_target)
        .map(|chrome| {
            (
                chrome.axis(),
                (
                    chrome.owner(),
                    chrome.target().clone(),
                    chrome.track(),
                    chrome.maximum_offset(),
                ),
            )
        })
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(chrome.len(), 2);
    assert_eq!(
        chrome
            .values()
            .map(|(owner, _, _, _)| *owner)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        1,
        "text bars intentionally share one semantic frame while retaining two property slots"
    );
    let (horizontal_owner, horizontal_target, horizontal_track, horizontal_max) =
        chrome[&interaction::ScrollbarAxis::Horizontal].clone();
    let (vertical_owner, vertical_target, vertical_track, vertical_max) =
        chrome[&interaction::ScrollbarAxis::Vertical].clone();
    let horizontal_hover = geometry::Point::new(
        horizontal_track
            .x()
            .saturating_add(horizontal_track.width() / 2),
        horizontal_track
            .y()
            .saturating_add(horizontal_track.height() / 2),
    );

    app.pointer_move_at(window, size, horizontal_hover)
        .expect("horizontal text chrome should hover");
    let activity_at = now + std::time::Duration::from_millis(10);
    app.show_scene_at(window, size, activity_at)
        .expect("hover should begin per-axis thickness and shared activity transitions");
    let active = app
        .show_scene_at(
            window,
            size,
            activity_at
                + std::time::Duration::from_millis(
                    theme.scrollbar().appearance.fade_duration_ms + 20,
                ),
        )
        .expect("hover transition should settle");
    let horizontal_visual = active
        .properties()
        .scrollbar(horizontal_owner, interaction::ScrollbarAxis::Horizontal)
        .expect("horizontal scrollbar property");
    let vertical_visual = active
        .properties()
        .scrollbar(vertical_owner, interaction::ScrollbarAxis::Vertical)
        .expect("vertical scrollbar property");
    assert!(
        horizontal_visual.0 > 0.99 && vertical_visual.0 > 0.99,
        "horizontal={horizontal_visual:?} vertical={vertical_visual:?}"
    );
    assert_eq!(
        horizontal_visual.1,
        theme.scrollbar().appearance.hover_thickness as f32
    );
    assert_eq!(
        vertical_visual.1,
        theme.scrollbar().appearance.overlay_thickness as f32,
        "hover thickness must remain local to one bar even though activity is shared"
    );

    let horizontal_press =
        geometry::Point::new(horizontal_track.x().saturating_add(1), horizontal_hover.y());
    let horizontal_drag = geometry::Point::new(
        horizontal_track.right().saturating_sub(1),
        horizontal_hover.y(),
    );
    app.pointer_down_at(window, size, horizontal_press)
        .expect("horizontal text scrollbar should capture");
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .map(interaction::pointer::Capture::target),
        Some(&horizontal_target)
    );
    app.pointer_drag_at(window, size, horizontal_drag)
        .expect("horizontal text scrollbar should mutate x");
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("text interaction")
            .scroll()
            .desired_offset(&text_target),
        interaction::ScrollOffset::new(horizontal_max, 0)
    );
    app.pointer_up_at(window, size, horizontal_drag)
        .expect("horizontal text capture should release");
    app.show_scene(window, size)
        .expect("horizontal text scroll should present");

    let vertical_press = geometry::Point::new(
        vertical_track
            .x()
            .saturating_add(vertical_track.width() / 2),
        vertical_track.y().saturating_add(1),
    );
    let vertical_drag = geometry::Point::new(
        vertical_press.x(),
        vertical_track.bottom().saturating_sub(1),
    );
    app.pointer_down_at(window, size, vertical_press)
        .expect("vertical text scrollbar should capture");
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .map(interaction::pointer::Capture::target),
        Some(&vertical_target)
    );
    app.pointer_drag_at(window, size, vertical_drag)
        .expect("vertical text scrollbar should mutate y");
    let expected = interaction::ScrollOffset::new(horizontal_max, vertical_max);
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("text interaction")
            .scroll()
            .desired_offset(&text_target),
        expected
    );
    app.pointer_up_at(window, size, vertical_drag)
        .expect("vertical text capture should release");
    app.show_scene(window, size)
        .expect("far text residency should present");
    let admitted = app
        .session()
        .interaction(window)
        .expect("text interaction")
        .scroll()
        .offset(&text_target);
    assert_eq!(
        admitted, expected,
        "vertical text chrome must preserve the admitted horizontal component"
    );
}

#[test]
fn hover_thickness_does_not_change_scroll_layout_rects() {
    let mut app = scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(220, 120);
    let now = std::time::Instant::now();
    let initial = app
        .show_scene_at(window, size, now)
        .expect("scroll view should render");
    let initial_scroll = first_scroll_frame(&initial).rect();
    let initial_viewport = first_scroll_frame(&initial)
        .viewport()
        .expect("scroll should expose viewport")
        .rect();
    let track = first_scrollbar_track(initial.layout());

    app.pointer_move_at(
        window,
        size,
        geometry::Point::new(
            track.x().saturating_add(track.width() / 2),
            track.y().saturating_add(track.height() / 2),
        ),
    )
    .expect("hovering scrollbar should be handled");
    let hovered = app
        .show_scene_at(window, size, now + std::time::Duration::from_millis(260))
        .expect("hovered scroll view should render");
    let hovered_scroll = first_scroll_frame(&hovered);

    assert_eq!(hovered_scroll.rect(), initial_scroll);
    assert_eq!(
        hovered_scroll
            .viewport()
            .expect("scroll should expose hovered viewport")
            .rect(),
        initial_viewport
    );
}

#[test]
fn text_area_projects_scrollbars_like_generic_viewports() {
    let document = (0..120)
        .map(|line| format!("text area line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    });
    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(520, 180))
        .expect("text editor should render");
    let text_area = rendered
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let viewport = text_area
        .viewport()
        .expect("text area should expose shared viewport geometry");
    let target = text_area
        .target()
        .expect("text area should expose scroll target");

    assert!(viewport.max_scroll().y() > 0);
    assert_eq!(viewport.max_scroll().x(), 0);
    assert_eq!(
        rendered
            .layout()
            .chrome()
            .iter()
            .filter(|chrome| chrome.scroll_target() == target)
            .map(layout::Chrome::axis)
            .collect::<std::collections::HashSet<_>>(),
        [interaction::ScrollbarAxis::Vertical].into_iter().collect(),
        "vertical text overflow must not synthesize a horizontal scrollbar"
    );
}

#[test]
fn text_area_scrollbar_hit_does_not_route_to_text_editing() {
    let document = (0..120)
        .map(|line| format!("drag line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    });
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(520, 180);
    let rendered = app
        .show_scene(window, size)
        .expect("text editor should render");
    let track = first_scrollbar_track(rendered.layout());
    let point = geometry::Point::new(
        track.x().saturating_add(track.width() / 2),
        track.y().saturating_add(track.height() / 2),
    );
    let before_position = app.state().document.position();
    let before_selection = app.state().document.selected_text();
    let hit = app
        .hit_test(window, size, point)
        .expect("scrollbar should hit");

    assert!(hit.is_chrome());
    assert_eq!(
        hit.target().expect("scrollbar should expose target").kind(),
        interaction::Kind::Scrollbar
    );

    app.pointer_down_at(window, size, point)
        .expect("scrollbar pointer down should be handled");
    app.pointer_drag_at(
        window,
        size,
        geometry::Point::new(
            track.x().saturating_add(track.width() / 2),
            track.bottom().saturating_sub(1),
        ),
    )
    .expect("scrollbar pointer drag should be handled");

    assert_eq!(app.state().document.position(), before_position);
    assert_eq!(app.state().document.selected_text(), before_selection);
}

#[test]
fn viewport_clip_applies_inside_floating_panel() {
    let view = widget::view(|ui| {
        ui.add(
            widget::panel::Floating::new("tests.floating.scroll")
                .width(view::Dimension::fixed(180))
                .height(view::Dimension::fixed(120))
                .children(|ui| {
                    ui.add(
                        widget::Scroll::new()
                            .id("scroll.floating")
                            .height(view::Dimension::fixed(48))
                            .children(|ui| {
                                for index in 0..6 {
                                    ui.label(format!("Floating row {index}"));
                                }
                            }),
                    );
                }),
        );
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 180), &mut layout_engine);
    let scroll = layout
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("floating scroll should be laid out");
    let viewport = scroll.viewport().expect("scroll should expose viewport");
    let scene = scene::Scene::paint_with_theme(&layout, &Theme::dark());
    let clip_index = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(primitive, scene::Primitive::Clip(clip) if clip.rect() == viewport.rect())
        })
        .expect("floating scroll should push a viewport clip");
    let text_index = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "Floating row 0"
            )
        })
        .expect("floating scroll content should paint");
    let pop_index = scene
        .primitives()
        .iter()
        .skip(text_index)
        .position(|primitive| matches!(primitive, scene::Primitive::PopClip))
        .map(|offset| text_index + offset)
        .expect("floating scroll should pop the viewport clip after content");

    assert!(clip_index < text_index);
    assert!(text_index < pop_index);
}

#[test]
fn scrolled_out_content_is_not_interactive() {
    let focus = session::Focus::text("clip.search");
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<PaletteOne>(command::Spec::new("Result"));
        })
        .responders(|responders| {
            responders.app().target::<PaletteOne>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Clip Hit"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.text_box(widget::TextBox::new("").focus(focus));
                    ui.add(
                        widget::Scroll::new()
                            .id("clip.results")
                            .height(view::Dimension::fixed(72))
                            .children(|ui| {
                                for index in 0..8 {
                                    ui.button(
                                        widget::Button::new(format!("Result {index}"))
                                            .trigger::<PaletteOne>(()),
                                    );
                                }
                            }),
                    );
                });
            })
        });
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 160);
    let initial = app
        .show_scene(window, size)
        .expect("initial clipped view should render");
    let scroll = first_scroll_frame(&initial);
    app.scroll_at(
        window,
        size,
        frame_point_at(
            scroll
                .viewport()
                .expect("scroll should expose viewport")
                .rect(),
        ),
        interaction::ScrollDelta::vertical(56),
    )
    .expect("scroll should be handled");
    app.request_redraw(window);
    let rendered = app
        .show_scene(window, size)
        .expect("scrolled view should render");
    let search = rendered
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("search box should be laid out");
    let point = rect_bottom_point(search.rect());

    assert!(
        rendered.layout().frames().iter().any(|frame| {
            frame.target().is_some() && frame.rect().contains(point) && !frame.clip_contains(point)
        }),
        "a scrolled-out result should geometrically overlap the search box"
    );

    let hit = rendered
        .layout()
        .hit_test(point)
        .expect("visible search box should be hit");

    assert_eq!(hit.frame().role(), view::Role::TextBox);
}

#[test]
fn table_text_cursor_follows_row_press_admission_without_pointer_motion() {
    let mut app = editable_table_app(EditableTableState {
        records: vec![
            EditableRecord {
                key: 7,
                name: "Ada".to_owned(),
                count: 4,
            },
            EditableRecord {
                key: 8,
                name: "Grace".to_owned(),
                count: 8,
            },
        ],
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 124);
    let table = interaction::Id::new("editable.table");
    let row7 = crate::table::Cell::new(
        table,
        crate::virtual_list::Key::new(7),
        interaction::Id::new("name"),
    );
    let row8 = crate::table::Cell::new(
        table,
        crate::virtual_list::Key::new(8),
        interaction::Id::new("name"),
    );
    let initial = app
        .show_scene(window, size)
        .expect("table cursor rows should render");
    let point_for = |cell| {
        initial
            .layout()
            .frames()
            .iter()
            .find(|frame| frame.table_cell() == Some(cell))
            .map(frame_point)
            .expect("table text cell should materialize")
    };
    let row7_point = point_for(row7);
    let row8_point = point_for(row8);
    drop(initial);

    app.pointer_move_at(window, size, row7_point)
        .expect("non-focal text hover should resolve");
    assert_eq!(
        app.session().window(window).expect("window").cursor(),
        pointer::Cursor::Default,
        "a selection-only row must not advertise text participation"
    );
    assert!(app.take_cursor_updates().is_empty());

    app.pointer_down_at(window, size, row7_point)
        .expect("first text press should select only");
    app.pointer_up_at(window, size, row7_point)
        .expect("selection-only text press should release");
    assert_eq!(active_text_table_cell(app.session(), window), None);
    app.show_scene(window, size)
        .expect("focal row should present under the stationary pointer");
    assert_eq!(
        app.session().window(window).expect("window").cursor(),
        pointer::Cursor::Text,
        "successful presentation of focality re-resolves the stationary pointer"
    );
    assert_eq!(
        app.take_cursor_updates()
            .last()
            .map(|update| update.cursor()),
        Some(pointer::Cursor::Text)
    );

    for modifiers in [
        input::Modifiers::new(true, false, false, false),
        input::Modifiers::new(false, true, false, false),
        input::Modifiers::new(false, false, false, true),
    ] {
        app.pointer_modifiers_changed(window, modifiers)
            .expect("stationary selection modifier should resolve");
        assert_eq!(
            app.session().window(window).expect("window").cursor(),
            pointer::Cursor::Default
        );
        assert!(
            !app.session()
                .window(window)
                .expect("window")
                .redraw_requested(),
            "cursor-only modifier projection must not request a frame"
        );
        assert_eq!(
            app.take_cursor_updates()
                .last()
                .map(|update| update.cursor()),
            Some(pointer::Cursor::Default)
        );

        app.pointer_modifiers_changed(window, input::Modifiers::default())
            .expect("modifier release should resolve");
        assert_eq!(
            app.session().window(window).expect("window").cursor(),
            pointer::Cursor::Text
        );
        assert_eq!(
            app.take_cursor_updates()
                .last()
                .map(|update| update.cursor()),
            Some(pointer::Cursor::Text)
        );
    }

    let shift = input::Modifiers::new(true, false, false, false);
    app.pointer_down_at_with_modifiers(window, size, row8_point, shift)
        .expect("shift press should extend selection only");
    app.pointer_up_at(window, size, row8_point)
        .expect("shift selection should release");
    app.show_scene(window, size)
        .expect("range selection should reproject");
    assert_eq!(
        app.session().window(window).expect("window").cursor(),
        pointer::Cursor::Default,
        "a held selection modifier still suppresses the newly focal member"
    );

    app.pointer_modifiers_changed(window, input::Modifiers::default())
        .expect("shift release should admit focal text");
    assert_eq!(
        app.session().window(window).expect("window").cursor(),
        pointer::Cursor::Text
    );
    app.take_cursor_updates();

    app.pointer_move_at(window, size, row7_point)
        .expect("selected non-focal text hover should resolve");
    assert_eq!(
        app.session().window(window).expect("window").cursor(),
        pointer::Cursor::Default,
        "membership alone does not admit a selected-but-not-focal row member"
    );
    app.pointer_move_at(window, size, row8_point)
        .expect("focal text hover should resolve");
    assert_eq!(
        app.session().window(window).expect("window").cursor(),
        pointer::Cursor::Text
    );
}

#[test]
fn pointer_cursor_uses_text_for_editable_text_regions() {
    let text_box_focus = session::Focus::text("cursor.box");
    let text_area_focus = session::Focus::text("cursor.area");
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Cursor Text"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.text_box(widget::TextBox::new("field").focus(text_box_focus));
                    ui.label("not editable");
                    ui.text_area(widget::TextArea::new("short").focus(text_area_focus));
                });
            })
        });
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(280, 180);
    let rendered = app
        .show_scene(window, size)
        .expect("cursor test should render");
    let text_box = rendered
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let label = rendered
        .layout()
        .find_role(view::Role::Label)
        .into_iter()
        .next()
        .expect("label should be laid out");
    let text_area = rendered
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");

    assert_eq!(
        cursor_after_move(&mut app, window, size, frame_point(text_box)),
        Some(pointer::Cursor::Text)
    );
    assert_eq!(
        cursor_after_move(&mut app, window, size, frame_point(label)),
        Some(pointer::Cursor::Default)
    );
    assert_eq!(
        cursor_after_move(&mut app, window, size, rect_bottom_point(text_area.rect())),
        Some(pointer::Cursor::Text),
        "text area tail space still places a caret"
    );
}

#[test]
fn pointer_cursor_uses_text_for_read_only_selectable_text_but_not_disabled_text() {
    let read_only_focus = session::Focus::text("cursor.read-only");
    let disabled_focus = session::Focus::text("cursor.disabled-area");
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Cursor Text Modes"));
        })
        .view(move |_, _| {
            View::new(
                view::Node::root().child(
                    view::Node::stack(view::Axis::Vertical)
                        .child(
                            view::Node::text_area_state(
                                view::TextArea::new("selectable")
                                    .with_focus(read_only_focus)
                                    .read_only(),
                            )
                            .with_style(view::Style::new().with_height(view::Dimension::fixed(80))),
                        )
                        .child(
                            view::Node::text_area_state(
                                view::TextArea::new("unavailable")
                                    .with_focus(disabled_focus)
                                    .with_mode(text::surface::FieldMode::Disabled),
                            )
                            .with_style(view::Style::new().with_height(view::Dimension::fixed(80))),
                        ),
                ),
            )
        });
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(280, 180);
    let rendered = app
        .show_scene(window, size)
        .expect("text mode cursor test should render");
    let frames = rendered.layout().find_role(view::Role::TextArea);
    assert_eq!(frames.len(), 2);

    assert_eq!(
        cursor_after_move(&mut app, window, size, frame_point(frames[0])),
        Some(pointer::Cursor::Text),
        "selectability, not mutation, earns the I-beam"
    );
    app.pointer_move_at(window, size, frame_point(frames[1]))
        .expect("disabled text hover should resolve");
    assert_eq!(
        app.session().window(window).expect("window").cursor(),
        pointer::Cursor::Default,
        "disabled text admits no caret or selection gesture"
    );
}

#[test]
fn text_box_editability_is_independent_from_base_argument_command_state() {
    let focus = session::Focus::text("cursor.disabled");
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<DisabledTextSubmit>(command::Spec::new("Disabled Submit"));
        })
        .responders(|responders| {
            responders.app().target::<DisabledTextSubmit>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Disabled Cursor"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.label("seed");
                    ui.text_box(
                        widget::TextBox::new("disabled")
                            .focus(focus)
                            .on_commit::<DisabledTextSubmit>(),
                    );
                });
            })
        });
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(280, 90);
    let rendered = app
        .show_scene(window, size)
        .expect("disabled cursor test should render");
    let label = rendered
        .layout()
        .find_role(view::Role::Label)
        .into_iter()
        .next()
        .expect("label should be laid out");
    let text_box = rendered
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");

    assert_eq!(
        cursor_after_move(&mut app, window, size, frame_point(label)),
        None,
        "initial default cursor should not produce an update"
    );
    assert_eq!(
        cursor_after_move(&mut app, window, size, frame_point(text_box)),
        Some(pointer::Cursor::Text),
        "commit state is resolved from the future draft and cannot disable editing from base args"
    );
}

#[test]
fn pointer_cursor_uses_default_for_text_area_scrollbar_chrome() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(
            (0..80)
                .map(|line| format!("line {line:02}"))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        ..text_editor::State::default()
    });
    app.start();

    let window = app.session().windows()[0].id();
    let size = text_editor::window_size();
    let rendered = app
        .show_scene(window, size)
        .expect("text editor should render");
    let text_area = rendered
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let text_point = frame_point(text_area);
    assert_eq!(
        cursor_after_move(&mut app, window, size, text_point),
        Some(pointer::Cursor::Text)
    );

    let track = first_scrollbar_track(rendered.layout());
    assert_eq!(
        cursor_after_move(&mut app, window, size, frame_point_at(track)),
        Some(pointer::Cursor::Default),
        "scrollbar chrome is not text editing"
    );
}

#[test]
fn pointer_cursor_stays_with_palette_query_after_results_scroll() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(360, 260);
    app.handle_input(window, input::Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let initial = app
        .show_scene_after_overlay_fade(window, size)
        .expect("open palette should render");
    let results = command_palette_results_frame(&initial);
    app.scroll_at(
        window,
        size,
        frame_point_at(
            results
                .viewport()
                .expect("results should expose viewport")
                .rect(),
        ),
        interaction::ScrollDelta::vertical(84),
    )
    .expect("palette results should scroll");
    let scrolled = app
        .show_scene(window, size)
        .expect("scrolled palette should render");
    let query = scrolled
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("palette query should be laid out");
    let point = rect_bottom_point(query.rect());

    let hit = app
        .hit_test(window, size, point)
        .expect("visible palette query should remain hittable");
    assert_eq!(hit.frame().role(), view::Role::TextBox);
    assert_eq!(
        cursor_after_move(&mut app, window, size, point),
        Some(pointer::Cursor::Text),
        "cursor follows the visible query hit, not clipped rows behind it"
    );
}

#[test]
fn pointer_cursor_keeps_text_during_captured_text_drag() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_text("drag me"),
        ..text_editor::State::default()
    });
    app.start();

    let window = app.session().windows()[0].id();
    let size = text_editor::window_size();
    let rendered = app
        .show_scene(window, size)
        .expect("drag cursor test should render");
    let text_area = rendered
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let point = frame_point(text_area);
    app.pointer_down_at(window, size, point)
        .expect("text pointer down should capture");
    assert_eq!(
        drain_cursor_updates(&mut app, window, size)
            .last()
            .map(|update| update.cursor()),
        Some(pointer::Cursor::Text)
    );

    app.pointer_left_at(window)
        .expect("pointer left should preserve text capture");
    assert_eq!(
        drain_cursor_updates(&mut app, window, size)
            .last()
            .map(|update| update.cursor()),
        None,
        "text cursor is already active while capture remains"
    );

    app.pointer_up_at(window, size, geometry::Point::new(1, 1))
        .expect("pointer up should release capture");
    assert_eq!(
        drain_cursor_updates(&mut app, window, size)
            .last()
            .map(|update| update.cursor()),
        Some(pointer::Cursor::Default)
    );
}

#[test]
fn cursor_updates_drain_without_redraw() {
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Cursor Work"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.label("plain");
            })
        });
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(160, 80);
    app.show_scene(window, size)
        .expect("initial scene should render");
    assert!(!app.session().window(window).unwrap().redraw_requested());
    assert!(app.set_pointer_cursor_for_test(window, pointer::Cursor::Text));

    let work = app.drain_scenes(|id| {
        assert_eq!(id, window);
        size
    });

    assert!(work.presentations().is_empty());
    assert_eq!(
        work.cursor_updates()
            .iter()
            .map(|update| update.cursor())
            .collect::<Vec<_>>(),
        vec![pointer::Cursor::Text]
    );
}

#[test]
fn editable_text_surfaces_use_active_text_input_foreground() {
    let theme = Theme::from_toml_str(
        r##"
        [text-input]
        foreground = "#ff0000"
        "##,
    )
    .expect("theme should parse");
    let document = TextDocument::from_multiline_text("area");
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.text_box(widget::TextBox::new("field"));
            ui.text_area(widget::TextArea::from_document(&document));
        });
    });
    let mut engine = layout::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(260, 120),
        &mut engine,
        &theme,
    );
    let scene = scene::Scene::paint_with_theme(&layout, &theme);
    let expected = text_color_channels(scene::Color::rgb(255, 0, 0));
    let surface_colors = scene
        .text_viewports()
        .iter()
        .flat_map(|viewport| viewport.surfaces().iter())
        .map(|surface| surface.default_color().channels())
        .collect::<Vec<_>>();

    assert!(surface_colors.len() >= 2);
    assert!(
        surface_colors
            .iter()
            .all(|channels| text_color_channels_equal(*channels, expected)),
        "all editable text surfaces should use text-input foreground"
    );
}

#[test]
fn text_box_placeholder_uses_text_input_placeholder_color() {
    let theme = Theme::from_toml_str(
        r##"
        [text-input]
        placeholder = "#00ff00"
        "##,
    )
    .expect("theme should parse");
    let view = widget::view(|ui| {
        ui.text_box(widget::TextBox::new("").placeholder("Find"));
    });
    let mut engine = layout::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(220, 64),
        &mut engine,
        &theme,
    );
    let scene = scene::Scene::paint_with_theme(&layout, &theme);

    assert_eq!(
        scene_text(&scene, "Find").color(),
        scene::Color::rgb(0, 255, 0)
    );
}

#[test]
fn editable_caret_uses_text_input_caret_color() {
    let focus = session::Focus::text("caret.theme");
    let theme = Theme::from_toml_str(
        r##"
        [text-input]
        caret = "#00ff00"
        "##,
    )
    .expect("theme should parse");
    let mut app = Runtime::new(SourceState::default())
        .theme(move |_| theme.clone())
        .started(|cx| {
            cx.open_window(window::Options::new("Caret Theme"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("abcd").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    app.show_scene(window, size)
        .expect("initial render should install composition");
    app.handle_input(window, Input::focus(focus))
        .expect("focus should be handled");
    let rendered = app
        .show_scene(window, size)
        .expect("focused text box should render");

    assert!(rendered.scene().rules().iter().any(|rule| {
        rule.color() == scene::Color::rgb(0, 255, 0)
            && rule.axis() == scene::Axis::Vertical
            && rule.thickness_px() == 2
    }));
}

#[test]
fn command_palette_search_box_wins_over_clipped_results() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.show_scene(window, size)
        .expect("initial palette app should render");
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let initial = app
        .show_scene(window, size)
        .expect("open palette should render");
    let results = command_palette_results_frame(&initial);
    app.scroll_at(
        window,
        size,
        frame_point_at(
            results
                .viewport()
                .expect("results should expose viewport")
                .rect(),
        ),
        interaction::ScrollDelta::vertical(180),
    )
    .expect("palette results should scroll");
    let rendered = app
        .show_scene(window, size)
        .expect("scrolled palette should render");
    let query = rendered
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("palette query should be laid out");
    let point = rect_bottom_point(query.rect());
    let results = command_palette_results_frame(&rendered);
    let results_node = results.node_id();
    let offset = rendered
        .stack()
        .scroll_offset(results_node)
        .expect("palette results should carry a receipted scroll property");

    assert!(
        rendered.layout().frames().iter().any(|frame| {
            let rect = geometry::Rect::new(
                frame.rect().x().saturating_sub(offset.x()),
                frame.rect().y().saturating_sub(offset.y()),
                frame.rect().width(),
                frame.rect().height(),
            );
            frame.target().is_some()
                && rendered
                    .layout()
                    .scroll_ancestry(frame.node_id())
                    .contains(&results_node)
                && rect.contains(point)
                && !frame.clip_contains(point)
        }),
        "a clipped palette result should geometrically overlap the query box"
    );

    let hit = rendered
        .layout()
        .hit_test(point)
        .expect("palette query should be hit");

    assert_eq!(hit.frame().role(), view::Role::TextBox);

    app.pointer_down_at(window, size, point)
        .expect("query pointer down should be handled");

    assert!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&interaction::CommandPalette::query_focus()))
    );
}

#[test]
fn dismissed_palette_ghost_is_paint_only() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let opened = app
        .show_scene_after_overlay_fade(window, size)
        .expect("open palette should render");
    let panel_rect = command_palette_panel_frame(&opened).rect();
    let revision = app.revision();

    let dismissed = app
        .handle_input(window, Input::cancel())
        .expect("escape should dismiss the palette");
    assert!(dismissed.is_handled());
    assert!(!dismissed.changed_state());
    assert_eq!(app.revision(), revision);
    assert!(
        app.session()
            .interaction(window)
            .and_then(interaction::Interaction::command_palette)
            .is_none(),
        "dismissal should remove the live palette immediately"
    );

    let ghost = app
        .show_scene(window, size)
        .expect("palette ghost should render after dismissal");
    assert_eq!(app.revision(), revision);
    assert!(
        ghost
            .layout()
            .find_role(view::Role::FloatingPanel)
            .into_iter()
            .all(|frame| {
                frame.target().and_then(interaction::Target::element_id)
                    != Some(interaction::CommandPalette::panel_id())
            }),
        "ghosts must not remain in layout"
    );
    assert!(
        ghost
            .scene()
            .texts()
            .iter()
            .any(|text| text.value() == "Palette One"),
        "the departed palette should still be visible as a ghost"
    );

    if let Some(hit) = app.hit_test(window, size, frame_point_at(panel_rect)) {
        assert_ne!(hit.frame().role(), view::Role::FloatingPanel);
        assert!(
            !hit.frame().is_palette_row(),
            "ghost palette rows must not participate in hit testing"
        );
    }

    let during_fade = app
        .show_scene_at(
            window,
            size,
            std::time::Instant::now()
                + std::time::Duration::from_millis(Theme::default().overlay().exit_fade_ms / 2),
        )
        .expect("ghost fade frame should render");
    assert_eq!(app.revision(), revision);
    assert!(
        during_fade
            .scene()
            .texts()
            .iter()
            .any(|text| text.value() == "Palette One")
    );

    let expired = app
        .show_scene_at(
            window,
            size,
            std::time::Instant::now()
                + std::time::Duration::from_millis(Theme::default().overlay().exit_fade_ms + 1),
        )
        .expect("expired ghost frame should render");
    assert_eq!(app.revision(), revision);
    assert!(
        expired
            .scene()
            .texts()
            .iter()
            .all(|text| text.value() != "Palette One"),
        "ghost should be removed after the exit duration"
    );
}

#[test]
fn palette_results_scroll_id_is_not_painted() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let rendered = app
        .show_scene_after_overlay_fade(window, size)
        .expect("open palette should render");
    let results = command_palette_results_frame(&rendered);

    assert_eq!(results.label_text(), None);
    assert_eq!(
        results.target().and_then(interaction::Target::element_id),
        Some(interaction::CommandPalette::results_id())
    );
    assert!(
        rendered
            .scene()
            .texts()
            .iter()
            .all(|text| text.value() != "Command Results"),
        "the results viewport id must not paint as visible text"
    );
}

#[test]
fn command_palette_section_headers_use_bold_caption_uppercase_presentation() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let rendered = app
        .show_scene_after_overlay_fade(window, size)
        .expect("open palette should render");
    let theme = Theme::dark();
    let header = scene_text(rendered.scene(), "APPLICATION");

    assert_eq!(header.style().size(), theme.typography().caption().size());
    assert_eq!(header.style().weight(), text::document::Weight::Bold);
    assert_eq!(header.color(), theme.text().muted);
    assert_eq!(header.align(), theme.command_palette().section_alignment());
    assert!(
        rendered
            .scene()
            .texts()
            .iter()
            .all(|text| text.value() != "Application"),
        "section source labels stay mixed-case data; only presentation uppercases"
    );
}

#[test]
fn command_palette_rows_use_interface_shortcut_typography() {
    let theme = Theme::from_toml_str(
        r##"
        [typography]
        interface-size = 13.0
        interface-weight = "medium"
        body-size = 19.0
        body-weight = "bold"
        caption-size = 8.0
        caption-weight = "medium"
        hint-size = 48.0
        hint-weight = "normal"

        [command-palette]
        section-alignment = "center"
        "##,
    )
    .expect("theme should parse");
    let expected = theme.clone();
    let mut app = command_palette_scroll_app().theme(move |_| theme.clone());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let rendered = app
        .show_scene_after_overlay_fade(window, size)
        .expect("open palette should render");
    let command = scene_text(rendered.scene(), "Palette One");
    let row = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.label_text() == Some("Palette One"))
        .expect("palette row should be laid out");
    let parts = layout::palette_row_parts(row.rect(), row.shortcut_width(), &expected);
    let shortcut = scene_text_in_rect(rendered.scene(), "R", parts.shortcut);
    let shortcut_icon = scene_icon_in_rect(rendered.scene(), "caret-up", parts.shortcut);

    assert_eq!(
        command.style().size(),
        expected.typography().interface().size()
    );
    assert_eq!(command.style().weight(), text::document::Weight::Medium);
    assert_eq!(command.color(), expected.text().primary);
    assert_eq!(
        shortcut.style().size(),
        expected.typography().interface().size()
    );
    assert_eq!(
        shortcut.style().weight(),
        expected.typography().interface().weight()
    );
    assert_ne!(shortcut.style().size(), expected.typography().body().size());
    assert_ne!(shortcut.style().size(), expected.typography().hint().size());
    assert_eq!(shortcut.color(), expected.text().muted);
    assert_eq!(shortcut.align(), scene::TextAlign::Start);
    assert_eq!(shortcut_icon.color(), expected.text().muted);
    assert!(
        row.shortcut_width() > 0 && row.shortcut_width() < 120,
        "palette shortcut parts should be measured from interface typography, not body or hint"
    );
}

#[test]
fn command_palette_formats_shortcuts_with_active_keymap_profile() {
    let mut app = command_palette_scroll_app().keymap(keymap::Profile::mac());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Primary+Shift+P"))
        .expect("mac palette shortcut should open");
    let rendered = app
        .show_scene_after_overlay_fade(window, size)
        .expect("open palette should render");
    let row = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.label_text() == Some("Palette One"))
        .expect("palette row should be laid out");
    let parts = layout::palette_row_parts(row.rect(), row.shortcut_width(), &Theme::dark());

    assert!(scene_icon_in_rect(rendered.scene(), "command", parts.shortcut).size() > 0.0);
    assert_eq!(
        scene_text_in_rect(rendered.scene(), "R", parts.shortcut).align(),
        scene::TextAlign::Start
    );
    assert!(
        rendered
            .scene()
            .texts()
            .iter()
            .all(|text| text.value() != "Primary+R"),
        "semantic shortcuts must not paint directly"
    );
}

#[test]
fn shortcut_display_style_changes_paint_and_measure_together() {
    let platform_theme = Theme::dark();
    let text_theme = Theme::from_toml_str(
        r##"
        [shortcuts]
        display = "text"
        "##,
    )
    .expect("text shortcut display theme should parse");
    let mut platform_app = command_palette_scroll_app().theme(move |_| platform_theme.clone());
    let mut text_app = command_palette_scroll_app().theme(move |_| text_theme.clone());
    platform_app.start();
    text_app.start();

    let platform_window = platform_app.session().windows()[0].id();
    let text_window = text_app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    platform_app
        .handle_input(platform_window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette should open");
    text_app
        .handle_input(text_window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette should open");
    let platform = platform_app
        .show_scene_after_overlay_fade(platform_window, size)
        .expect("platform palette should render");
    let text = text_app
        .show_scene_after_overlay_fade(text_window, size)
        .expect("text palette should render");
    let platform_row = platform
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.label_text() == Some("Palette One"))
        .expect("platform palette row should be laid out");
    let text_row = text
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.label_text() == Some("Palette One"))
        .expect("text palette row should be laid out");
    let text_shortcut = scene_text(text.scene(), "Ctrl+R");

    assert_ne!(platform_row.shortcut_width(), text_row.shortcut_width());
    assert_eq!(text_shortcut.align(), scene::TextAlign::Start);
}

#[test]
fn command_palette_uses_panel_padding_and_content_gap() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let rendered = app
        .show_scene(window, size)
        .expect("open palette should render");
    let theme = Theme::dark();
    let panel = command_palette_panel_frame(&rendered);
    let query = rendered
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("palette query should be laid out");
    let results = command_palette_results_frame(&rendered);

    assert_eq!(
        query.rect().y().saturating_sub(panel.rect().y()),
        theme.floating_panel().padding
    );
    assert_eq!(
        results.rect().y().saturating_sub(query.rect().bottom()),
        theme.floating_panel().content_gap
    );
}

#[test]
fn explicit_zero_floating_panel_gap_disables_default_content_gap() {
    let mut theme = Theme::dark();
    theme.floating_panel_mut().content_gap = 9;
    let view = widget::view(|ui| {
        ui.add(
            widget::panel::Floating::new("tests.gap.zero")
                .layout(|layout| layout.gap(0))
                .children(|ui| {
                    ui.label("Alpha");
                    ui.label("Beta");
                }),
        );
    });
    let mut engine = layout::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(240, 160),
        &mut engine,
        &theme,
    );
    let labels = layout.find_role(view::Role::Label);

    assert_eq!(labels.len(), 2);
    assert_eq!(labels[1].rect().y(), labels[0].rect().bottom());
}

#[test]
fn command_palette_results_shrink_until_themed_cap_then_scroll() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let many = app
        .show_scene(window, size)
        .expect("open palette should render");
    let theme = Theme::dark();
    let many_results = command_palette_results_frame(&many);
    let many_viewport = many_results
        .viewport()
        .expect("results should expose viewport");

    assert_eq!(
        many_results.rect().height(),
        theme.command_palette().max_results_height()
    );
    assert!(many_viewport.content().height() > many_viewport.rect().height());

    app.handle_input(window, Input::text_commit("twelve"))
        .expect("typing should filter palette query");
    let few = app
        .show_scene(window, size)
        .expect("filtered palette should render");
    let few_results = command_palette_results_frame(&few);
    let few_viewport = few_results
        .viewport()
        .expect("results should expose viewport");

    assert!(few_results.rect().height() < theme.command_palette().max_results_height());
    assert_eq!(
        few_viewport.content().height(),
        few_viewport.rect().height()
    );
}

#[test]
fn command_palette_uses_centered_max_envelope_placement() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let expanded = app
        .show_scene(window, size)
        .expect("open palette should render");
    let expanded_panel = command_palette_panel_frame(&expanded);
    let expanded_top = expanded_panel.rect().y();

    assert_eq!(
        expanded_panel
            .rect()
            .x()
            .saturating_add(expanded_panel.rect().width() / 2),
        size.width() / 2
    );
    assert_eq!(
        expanded_panel.rect().y(),
        size.height().saturating_sub(expanded_panel.rect().height()) / 2
    );

    app.handle_input(window, Input::text_commit("twelve"))
        .expect("typing should filter palette query");
    let short = app
        .show_scene(window, size)
        .expect("filtered palette should render");
    let short_panel = command_palette_panel_frame(&short);

    assert_eq!(short_panel.rect().y(), expanded_top);
    assert!(short_panel.rect().height() < expanded_panel.rect().height());
}

#[test]
fn arrow_selection_scrolls_palette_result_into_view() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    for _ in 0..10 {
        app.handle_input(
            window,
            Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
        )
        .expect("palette arrow navigation should be handled");
    }

    let rendered = app
        .show_scene_after_overlay_fade(window, size)
        .expect("palette should render after keyboard navigation");
    let results = command_palette_results_frame(&rendered);
    let selected = selected_palette_result_frame(&rendered);
    let viewport = results
        .viewport()
        .expect("results should expose viewport")
        .rect();

    assert!(selected.rect().y() >= viewport.y());
    assert!(selected.rect().bottom() <= viewport.bottom());
    assert_tint_quad(
        rendered.scene(),
        selected.rect(),
        Theme::default().menu().row_hover_tint,
    );
    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction")
            .scroll()
            .offset(results.target().expect("results should expose a target"))
            .y()
            > 0,
        "arrow navigation should scroll the results viewport"
    );
}

#[test]
fn palette_arrow_navigation_reaches_last_command() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    for _ in 0..32 {
        app.handle_input(
            window,
            Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
        )
        .expect("palette arrow navigation should be handled");
    }

    let rendered = app
        .show_scene_after_overlay_fade(window, size)
        .expect("palette should render after keyboard navigation");
    let results = command_palette_results_frame(&rendered);
    let selected = selected_palette_result_frame(&rendered);
    let viewport = results
        .viewport()
        .expect("results should expose viewport")
        .rect();

    assert_eq!(selected.label_text(), Some("Close Window"));
    assert!(selected.rect().y() >= viewport.y());
    assert!(selected.rect().bottom() <= viewport.bottom());
}

#[test]
fn palette_reveal_uses_selected_frame_rect() {
    let mut theme = Theme::dark();
    theme.viewport_mut().reveal_margin = 12;
    let expected_margin = theme.viewport().reveal_margin;
    let mut app = command_palette_scroll_app().theme(move |_| theme.clone());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    for _ in 0..10 {
        app.handle_input(
            window,
            Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
        )
        .expect("palette arrow navigation should be handled");
    }

    let rendered = app
        .show_scene_after_overlay_fade(window, size)
        .expect("palette should render after keyboard navigation");
    let results = command_palette_results_frame(&rendered);
    let selected = selected_palette_result_frame(&rendered);
    let viewport = results
        .viewport()
        .expect("results should expose viewport")
        .rect();

    assert_eq!(
        selected.rect().bottom(),
        viewport.bottom() - expected_margin
    );
}

#[test]
fn palette_query_keeps_focus_while_active_result_moves() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
    )
    .expect("palette arrow navigation should be handled");
    app.show_scene(window, size)
        .expect("palette should render after keyboard navigation");

    assert_eq!(
        app.session().focused(window),
        Some(interaction::CommandPalette::query_focus())
    );
}

#[test]
fn active_descendant_reveal_request_clears_after_resolution() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
    )
    .expect("palette arrow navigation should be handled");

    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction")
            .scroll()
            .active_descendant_targets()
            .contains(&interaction::CommandPalette::results_target())
    );

    app.show_scene(window, size)
        .expect("palette should render after keyboard navigation");

    assert!(
        !app.session()
            .interaction(window)
            .expect("window should have interaction")
            .scroll()
            .active_descendant_targets()
            .contains(&interaction::CommandPalette::results_target())
    );
}

#[test]
fn label_means_visible_text() {
    let view = View::new(
        view::Node::root()
            .child(view::Node::panel().with_interaction_id("identity.only"))
            .child(view::Node::panel().with_label("Visible Panel")),
    );
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 120), &mut layout_engine);
    let scene = scene::Scene::paint(&layout);

    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value() == "Visible Panel"),
        "labels are visible presentation text"
    );
    assert!(
        scene
            .texts()
            .iter()
            .all(|text| text.value() != "identity.only"),
        "interaction ids are invisible identity"
    );
}

#[test]
fn typography_metrics_affect_label_measurement() {
    let default = Theme::dark();
    let large = Theme::from_toml_str(
        r##"
        [typography]
        body-size = 28.0
        "##,
    )
    .expect("theme should parse");
    let view = View::new(
        view::Node::root().child(
            view::Node::panel()
                .with_style(view::Style::new().with_align_items(view::Align::Start))
                .child(view::Node::label("Typography")),
        ),
    );
    let mut engine = layout::Engine::new();
    let default_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(320, 120),
        &mut engine,
        &default,
    );
    let large_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(320, 120),
        &mut engine,
        &large,
    );
    let default_label = default_layout
        .find_role(view::Role::Label)
        .into_iter()
        .next()
        .expect("default label should be laid out")
        .rect();
    let large_label = large_layout
        .find_role(view::Role::Label)
        .into_iter()
        .next()
        .expect("large label should be laid out")
        .rect();

    assert!(
        large_label.width() > default_label.width()
            || large_label.height() > default_label.height(),
        "type size is a layout-visible metric, not paint-only appearance"
    );
}

#[test]
fn interface_metrics_affect_system_widgets_without_body_metrics() {
    let default = Theme::dark();
    let body_large = Theme::from_toml_str(
        r##"
        [typography]
        body-size = 28.0
        "##,
    )
    .expect("body-large theme should parse");
    let interface_large = Theme::from_toml_str(
        r##"
        [typography]
        interface-size = 18.0
        interface-weight = "bold"
        "##,
    )
    .expect("interface-large theme should parse");
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.add(
                widget::Element::new()
                    .row()
                    .height(view::Dimension::fit())
                    .children(|ui| {
                        ui.button(widget::Button::new("System Command"));
                    }),
            );
            ui.checkbox(widget::Checkbox::new("System Choice", true));
            ui.text_box(widget::TextBox::new("").placeholder("Find"));
            ui.add(
                widget::Element::new()
                    .row()
                    .height(view::Dimension::fit())
                    .children(|ui| {
                        ui.label("Content Label");
                    }),
            );
        });
    });
    let mut engine = layout::Engine::new();
    let default_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(420, 160),
        &mut engine,
        &default,
    );
    let body_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(420, 160),
        &mut engine,
        &body_large,
    );
    let interface_layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(420, 160),
        &mut engine,
        &interface_large,
    );
    let default_scene = scene::Scene::paint_with_theme(&default_layout, &default);
    let body_scene = scene::Scene::paint_with_theme(&body_layout, &body_large);
    let interface_scene = scene::Scene::paint_with_theme(&interface_layout, &interface_large);
    let default_button = default_layout
        .find_role(view::Role::Button)
        .into_iter()
        .next()
        .expect("default button should be laid out")
        .rect();
    let body_button = body_layout
        .find_role(view::Role::Button)
        .into_iter()
        .next()
        .expect("body-large button should be laid out")
        .rect();
    let interface_button = interface_layout
        .find_role(view::Role::Button)
        .into_iter()
        .next()
        .expect("interface-large button should be laid out")
        .rect();
    let default_label = default_layout
        .find_role(view::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Content Label"))
        .expect("default content label should be laid out")
        .rect();
    let body_label = body_layout
        .find_role(view::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Content Label"))
        .expect("body-large content label should be laid out")
        .rect();
    let interface_label = interface_layout
        .find_role(view::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Content Label"))
        .expect("interface-large content label should be laid out")
        .rect();

    assert_eq!(body_button.width(), default_button.width());
    assert!(interface_button.width() > default_button.width());
    assert_eq!(body_button.height(), default.control().height);
    assert_eq!(interface_button.height(), interface_large.control().height);
    assert!(body_label.width() > default_label.width());
    assert_eq!(interface_label.width(), default_label.width());

    assert_eq!(
        scene_text(&body_scene, "System Command").style().size(),
        body_large.typography().interface().size()
    );
    assert_eq!(
        scene_text(&interface_scene, "System Command")
            .style()
            .weight(),
        text::document::Weight::Bold
    );
    assert_eq!(
        scene_text(&interface_scene, "Find").style().size(),
        interface_large.typography().interface().size()
    );
    assert_eq!(
        scene_text(&default_scene, "Content Label").style().size(),
        default.typography().body().size()
    );
}

#[test]
fn scroll_target_at_ignores_clipped_viewports() {
    let mut app = nested_clipped_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 180);
    scroll_outer_until_inner_overlaps_search(&mut app, window, size);
    let rendered = app
        .show_scene(window, size)
        .expect("nested clipped scroll should render");
    let inner = scroll_frame_with_label(&rendered, "Inner Scroll");
    let point = rect_top_point(
        inner
            .viewport()
            .expect("inner scroll should expose viewport")
            .rect(),
    );

    assert!(
        !inner.clip_contains(point),
        "inner viewport acquisition point should be clipped by the outer viewport"
    );
    assert_eq!(
        rendered
            .layout()
            .scroll_target_at(point, interaction::ScrollDelta::vertical(24)),
        None
    );
}

#[test]
fn fully_clipped_viewports_project_no_scrollbar_chrome() {
    let mut app = nested_clipped_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 180);
    scroll_outer_until_inner_overlaps_search(&mut app, window, size);
    let rendered = app
        .show_scene(window, size)
        .expect("nested clipped scroll should render");
    let inner = scroll_frame_with_label(&rendered, "Inner Scroll");
    let inner_target = inner
        .target()
        .expect("inner scroll should expose a target")
        .clone();
    assert_eq!(
        inner
            .viewport()
            .expect("inner scroll should expose a viewport")
            .visible_frame()
            .height(),
        0
    );
    assert!(
        rendered
            .layout()
            .chrome()
            .iter()
            .all(|chrome| chrome.scroll_target() != &inner_target)
    );
}

#[test]
fn scrollbar_drag_does_not_dismiss_owning_palette() {
    let mut app = command_palette_scroll_app();
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(640, 420);
    app.show_scene(window, size)
        .expect("initial palette app should render");
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let initial = app
        .show_scene(window, size)
        .expect("open palette should render");
    let results = command_palette_results_frame(&initial);
    let target = results
        .target()
        .expect("palette results should expose a scroll target")
        .clone();
    let track = initial
        .layout()
        .chrome()
        .iter()
        .find(|chrome| chrome.scroll_target() == &target)
        .map(layout::Chrome::track)
        .expect("palette results should project scrollbar chrome");
    let press = frame_point_at(track);
    let drag = geometry::Point::new(
        track.x().saturating_add(track.width() / 2),
        track.bottom().saturating_sub(1),
    );

    app.pointer_down_at(window, size, press)
        .expect("palette scrollbar pointer down should be handled");

    assert!(
        app.session()
            .interaction(window)
            .and_then(interaction::Interaction::command_palette)
            .is_some(),
        "palette should stay open on its own scrollbar press"
    );
    assert_eq!(
        app.session().focused(window),
        Some(interaction::CommandPalette::query_focus()),
        "palette scrollbar should not steal query focus"
    );

    app.pointer_drag_at(window, size, drag)
        .expect("palette scrollbar drag should be handled");

    assert!(
        app.session()
            .interaction(window)
            .and_then(interaction::Interaction::command_palette)
            .is_some(),
        "palette should stay open while dragging its own scrollbar"
    );
    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction")
            .scroll()
            .offset(&target)
            .y()
            > 0,
        "palette scrollbar drag should update the results scroll offset"
    );
    assert_eq!(
        app.session().focused(window),
        Some(interaction::CommandPalette::query_focus())
    );
}

#[test]
fn layout_hit_testing_uses_stable_identity_and_topmost_popup_order() {
    let duplicate = View::new(
        view::Node::root().child(
            view::Node::menu_bar()
                .child(view::Node::menu("menu.first", "Same"))
                .child(view::Node::menu("menu.second", "Same")),
        ),
    );
    let mut layout_engine = layout::Engine::new();
    let duplicate_layout = layout::Layout::compose(
        &duplicate,
        geometry::Size::new(320, 120),
        &mut layout_engine,
    );
    let duplicate_menus = duplicate_layout.find_role(view::Role::Menu);

    assert_eq!(duplicate_menus.len(), 2);
    assert_eq!(duplicate_menus[0].label_text(), Some("Same"));
    assert_eq!(duplicate_menus[1].label_text(), Some("Same"));
    assert_ne!(duplicate_menus[0].target(), duplicate_menus[1].target());

    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");

    app.handle_view(
        window,
        file.menu_action().expect("menu should expose an action"),
    )
    .expect("menu action should be handled");

    let projected = app
        .present(window)
        .expect("open menu should project a popup");
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(800, 600),
        &mut layout_engine,
    );
    let popup_hit = layout
        .hit_test(geometry::Point::new(10, 34))
        .expect("popup should be hit above the text area");

    assert_eq!(popup_hit.frame().role(), view::Role::Binding);
    assert_eq!(popup_hit.frame().label_text(), Some("New"));
    assert_eq!(
        popup_hit
            .target()
            .expect("popup command should expose a target")
            .kind(),
        interaction::Kind::Command
    );
}

#[test]
fn runtime_host_pointer_coordinates_route_to_view_actions() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    app.show_scene(window, size)
        .expect("initial scene should install a composition");

    let hit = app
        .hit_test(window, size, geometry::Point::new(10, 10))
        .expect("file menu should be hit");
    assert_eq!(hit.frame().role(), view::Role::Menu);
    assert_eq!(hit.frame().label_text(), Some("File"));

    let moved = app
        .pointer_move_at(window, size, geometry::Point::new(10, 10))
        .expect("coordinate pointer move should be handled");
    assert!(moved.is_handled());

    let pressed = app
        .pointer_down_at(window, size, geometry::Point::new(10, 10))
        .expect("coordinate pointer down should be handled");
    assert!(pressed.is_handled());

    let released = app
        .pointer_up_at(window, size, geometry::Point::new(10, 10))
        .expect("coordinate pointer up should be handled");
    assert!(released.is_handled());
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu())
            .map(|menu| menu.label()),
        Some("File")
    );
}

#[test]
fn runtime_host_scroll_coordinates_route_to_scroll_target() {
    let document = (0..120)
        .map(|line| format!("scroll line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let presentation = app
        .show_scene(window, size)
        .expect("initial scene should install a composition");
    let text_area = presentation
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let target = text_area
        .target()
        .expect("text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 4, text_area.rect().y() + 4);
    let semantic_commit = std::sync::Arc::clone(presentation.commit());
    let drawable_commit = std::sync::Arc::clone(presentation.stack().base().drawable_commit());

    let outcome = app
        .scroll_at(window, size, point, interaction::ScrollDelta::vertical(96))
        .expect("coordinate scroll should be handled");

    assert!(outcome.is_handled());
    assert_eq!(outcome.effect(), &response::Effect::None);
    let scroll = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .scroll();
    assert_eq!(
        scroll.offset(&target),
        interaction::ScrollOffset::new(0, 96)
    );
    assert_eq!(
        scroll.desired_offset(&target),
        interaction::ScrollOffset::new(0, 96)
    );

    let scrolled = app
        .show_scene(window, size)
        .expect("scrolled scene should render");
    assert!(scrolled.property_only());
    assert!(std::sync::Arc::ptr_eq(&semantic_commit, scrolled.commit()));
    assert!(std::sync::Arc::ptr_eq(
        &drawable_commit,
        scrolled.stack().base().drawable_commit()
    ));
    let projection = scrolled
        .layout()
        .scroll_projections()
        .iter()
        .find(|projection| projection.target() == &target)
        .expect("text area should retain the universal scroll projection");
    assert_eq!(
        scrolled.properties().scroll_offset(projection.node()),
        Some(interaction::ScrollOffset::new(0, 96))
    );
    let chrome = scrolled
        .layout()
        .chrome()
        .iter()
        .find(|chrome| {
            chrome.scroll_target() == &target
                && chrome.axis() == interaction::ScrollbarAxis::Vertical
        })
        .expect("scrolled text area should project vertical scrollbar chrome");
    let baseline_thumb = chrome.thumb_with_thickness(
        crate::theme::Theme::dark()
            .scrollbar()
            .appearance
            .overlay_thickness,
    );
    let theme = crate::theme::Theme::dark();
    let (thumb_r, thumb_g, thumb_b, _) = theme.scrollbar().appearance.thumb.channels();
    let properties = crate::scene::Properties::new(
        scrolled.commit(),
        scrolled.properties().serial(),
        vec![
            crate::scene::PropertyValue::Scrollbar {
                node: chrome.owner(),
                axis: chrome.axis(),
                opacity: 1.0,
                thickness: theme.scrollbar().appearance.overlay_thickness as f32,
            },
            crate::scene::PropertyValue::ScrollOffset {
                node: projection.node(),
                value: interaction::ScrollOffset::new(0, 96),
            },
        ],
        Vec::new(),
    )
    .expect("text area fixture should declare the shared scroll and scrollbar properties");
    let projected = scrolled
        .commit()
        .compatibility_scene(&properties)
        .expect("text area scrollbar should project from the admitted scroll property");
    let thumb_quads = projected
        .quads()
        .into_iter()
        .filter_map(|quad| {
            let (r, g, b, a) = quad.fill().channels();
            ((r, g, b) == (thumb_r, thumb_g, thumb_b) && a > 0).then_some((quad.rect(), a))
        })
        .collect::<Vec<_>>();
    assert!(
        thumb_quads
            .iter()
            .any(|(rect, _)| rect.y() > baseline_thumb.y()),
        "text scrollbar must advance from the authored baseline with the same admitted property as its content: baseline={baseline_thumb:?}, actual={thumb_quads:?}"
    );
}

#[test]
fn platform_wheel_down_scroll_moves_text_area_content_up() {
    use winit::event::MouseScrollDelta;

    let document = (0..160)
        .map(|line| format!("direction line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(520, 180);
    let initial = app
        .show_scene(window, size)
        .expect("initial scene should render");
    let initial_y = first_visible_text_area_surface_y(&initial);
    let text_area = initial
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let target = text_area
        .target()
        .expect("text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 8, text_area.rect().y() + 8);
    let wheel_down = platform::scroll_delta(MouseScrollDelta::LineDelta(0.0, -16.0), 1.0);

    let outcome = app
        .scroll_at(window, size, point, wheel_down)
        .expect("wheel scroll should route to text area");
    assert!(outcome.is_handled());
    let scroll = app
        .session()
        .interaction(window)
        .expect("window should retain interaction state")
        .scroll();
    assert_eq!(scroll.offset(&target), interaction::ScrollOffset::default());
    assert_eq!(
        scroll.desired_offset(&target),
        interaction::ScrollOffset::new(0, 448)
    );

    let scrolled = app
        .show_scene(window, size)
        .expect("scrolled scene should render");
    let scrolled_y = first_visible_text_area_surface_y(&scrolled);
    let scroll_y = scrolled
        .layout()
        .find_role(view::Role::TextArea)
        .first()
        .and_then(|frame| frame.text_area_layout())
        .map(|text_area| text_area.layout().scroll_y())
        .expect("text area should have a text layout");

    assert!(scroll_y > 0.0);
    assert!(scrolled_y < initial_y);
}

#[test]
fn text_area_input_clamps_to_presented_extent_before_admission() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text("short\ntext"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let presentation = app
        .show_scene(window, size)
        .expect("initial scene should install a composition");
    let text_area = presentation
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let target = text_area
        .target()
        .expect("text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 4, text_area.rect().y() + 4);

    let frame_scroll_commits = app
        .diagnostics(window)
        .expect("window should have diagnostics")
        .scroll
        .frame_scroll_commits;
    let outcome = app
        .scroll_at(
            window,
            size,
            point,
            interaction::ScrollDelta::vertical(4_000),
        )
        .expect("coordinate scroll should be handled");
    assert_eq!(outcome.effect(), &response::Effect::None);
    let scroll = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .scroll();
    assert_eq!(scroll.offset(&target), interaction::ScrollOffset::default());
    assert_eq!(
        scroll.desired_offset(&target),
        interaction::ScrollOffset::default()
    );
    assert!(
        !app.session()
            .window(window)
            .expect("text area window should remain open")
            .property_tick_requested(),
        "a clamped no-op must schedule no property work"
    );
    assert_eq!(
        app.diagnostics(window)
            .expect("window should have diagnostics after input")
            .scroll
            .frame_scroll_commits,
        frame_scroll_commits
    );
}

#[test]
fn text_area_caret_reveal_resolves_framework_owned_scroll_after_edit() {
    let document = (0..120)
        .map(|line| format!("reveal line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let focus = app
        .present(window)
        .expect("initial view should present")
        .text_areas()[0]
        .focus()
        .expect("text area should declare focus");
    let presentation = app
        .show_scene(window, size)
        .expect("initial scene should install a composition");
    let text_area = presentation
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let target = text_area
        .target()
        .expect("text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 4, text_area.rect().y() + 4);

    app.scroll_at(window, size, point, interaction::ScrollDelta::vertical(240))
        .expect("coordinate scroll should be handled");
    let scroll = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .scroll();
    assert_eq!(
        scroll.offset(&target),
        interaction::ScrollOffset::new(0, 240)
    );
    assert_eq!(
        scroll.desired_offset(&target),
        interaction::ScrollOffset::new(0, 240)
    );

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    let moved = app
        .handle_input(
            window,
            Input::text_selection(text::selection::Operation::set_position(
                text::buffer::Position::new(0),
            )),
        )
        .expect("caret move should be handled");

    assert!(moved.is_handled());
    assert!(moved.changed_state());
    assert!(moved.effect().contains_invalidation());

    let revealed = app
        .show_scene(window, size)
        .expect("revealed scene should render");

    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::default()
    );
    let text_area = revealed
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out after reveal");
    assert_eq!(
        text_area
            .text_area_layout()
            .expect("text area should use text area layout")
            .layout()
            .scroll_y(),
        0.0
    );
}

#[test]
fn text_editor_layout_paints_to_renderer_neutral_scene() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    app.invoke(app.trigger::<text_editor::ToggleDebugPanel>(()))
        .output
        .expect("debug panel toggle should resolve");
    app.invoke(app.trigger::<text_editor::LoadStressText>(()))
        .output
        .expect("stress text command should resolve");
    let projected = app.present(window).expect("window should have a view");
    let revision = app.revision();
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(800, 600),
        &mut layout_engine,
    );
    let _: &Scene = &scene::Scene::paint(&layout);
    let scene = scene::Scene::paint(&layout);
    assert_eq!(scene.size(), layout.size());
    assert!(!scene.is_empty());
    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value().contains("File"))
    );
    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value().contains("loaded Unicode stress fixture"))
    );
    assert!(
        scene
            .text_viewports()
            .iter()
            .any(|viewport| !viewport.surfaces().is_empty())
    );
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (28, 28, 30, 255)
            && layout
                .find_role(view::Role::TextArea)
                .iter()
                .any(|frame| frame.rect() == quad.rect())
    }));
    assert_eq!(scene.clear().channels(), (17, 18, 20, 255));
    assert_eq!(app.revision(), revision);
}

#[test]
fn text_area_selection_highlight_is_clipped_to_text_area_viewport() {
    let text = (0..180)
        .map(|line| format!("highlight line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut document = TextDocument::from_multiline_text(text);
    let selected = document.apply_selection(text::selection::Operation::SelectAll);

    assert!(selected.selection_changed());

    let mut app = text_editor::app(text_editor::State {
        document,
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(520, 180);
    let initial = app
        .show_scene(window, size)
        .expect("initial scene should render");
    let text_area = initial
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let target = text_area
        .target()
        .expect("text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 8, text_area.rect().y() + 8);

    app.scroll_at(window, size, point, interaction::ScrollDelta::vertical(220))
        .expect("scroll should route to text area");

    let scrolled = app
        .show_scene(window, size)
        .expect("scrolled scene should render");
    let text_area_rect = scrolled
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out after scrolling")
        .rect();
    let highlights = scrolled
        .scene()
        .quads()
        .into_iter()
        .filter(|quad| quad.fill().channels() == (10, 132, 255, 96))
        .collect::<Vec<_>>();

    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should retain interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 220)
    );
    assert!(!highlights.is_empty());

    let mut clips = Vec::new();
    let mut clipped_highlights = 0_usize;
    for primitive in scrolled.scene().primitives() {
        match primitive {
            scene::Primitive::Clip(clip) => clips.push(clip.rect()),
            scene::Primitive::PopClip => {
                clips.pop();
            }
            scene::Primitive::Quad(quad) if quad.fill().channels() == (10, 132, 255, 96) => {
                assert!(
                    rect_contains(text_area_rect, quad.rect()) || clips.contains(&text_area_rect),
                    "selection highlight must either be preclipped or remain under the fixed text viewport clip: bounds {text_area_rect:?}, highlight {:?}, clips={clips:?}",
                    quad.rect()
                );
                clipped_highlights += 1;
            }
            _ => {}
        }
    }
    assert_eq!(clipped_highlights, highlights.len());
}

#[test]
fn text_area_selection_highlight_paints_below_menu_bar_chrome() {
    let text = (0..24)
        .map(|line| format!("selected line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut document = TextDocument::from_multiline_text(text);
    document.apply_selection(text::selection::Operation::SelectAll);

    let mut app = text_editor::app(text_editor::State {
        document,
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(520, 180))
        .expect("selected text area should render");
    let primitives = rendered.scene().primitives();
    let menu_bar_rect = rendered
        .layout()
        .find_role(view::Role::MenuBar)
        .into_iter()
        .next()
        .expect("menu bar should be laid out")
        .rect();
    let highlight = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad)
                    if quad.fill().channels() == (10, 132, 255, 96)
            )
        })
        .expect("selection highlight should be painted");
    let menu_bar_chrome = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad)
                    if quad.fill().channels() == (28, 28, 30, 255)
                        && quad.rect() == menu_bar_rect
            )
        })
        .expect("menu bar chrome should be painted");
    let file_menu_text = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "File"
            )
        })
        .expect("menu bar file text should be painted");

    assert!(
        highlight < menu_bar_chrome,
        "selection highlight should paint below menu bar background"
    );
    assert!(
        highlight < file_menu_text,
        "selection highlight should paint below menu bar text"
    );
}

#[test]
fn text_editor_wrap_command_changes_text_area_paint_wrap() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text("alpha beta gamma"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let wrapped = app
        .show_scene(window, geometry::Size::new(320, 180))
        .expect("wrapped scene should render");

    assert_eq!(
        wrapped
            .layout()
            .find_role(view::Role::TextArea)
            .first()
            .and_then(|frame| frame.text_wrap()),
        Some(view::Wrap::Word)
    );
    assert!(!wrapped.scene().text_viewports().is_empty());

    app.invoke(app.trigger::<text_editor::ToggleWrapText>(()))
        .output
        .expect("wrap toggle should resolve");

    let unwrapped = app
        .show_scene(window, geometry::Size::new(320, 180))
        .expect("unwrapped scene should render");

    assert_eq!(
        unwrapped
            .layout()
            .find_role(view::Role::TextArea)
            .first()
            .and_then(|frame| frame.text_wrap()),
        Some(view::Wrap::None)
    );
    assert!(!unwrapped.scene().text_viewports().is_empty());
}

#[test]
fn scene_paints_controls_from_semantic_state() {
    let view = widget::view(|ui| {
        ui.column(|ui| {
            ui.checkbox(widget::Checkbox::new("Wrap", true));
            ui.radio(widget::Radio::new("Soft tabs", true));
            ui.slider(widget::Slider::new("Zoom", 5.0, 0.0..=10.0));
        });
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(320, 120), &mut layout_engine);
    let scene = scene::Scene::paint(&layout);
    let checkbox = layout
        .find_role(view::Role::Checkbox)
        .into_iter()
        .next()
        .expect("checkbox should be laid out");
    let radio = layout
        .find_role(view::Role::Radio)
        .into_iter()
        .next()
        .expect("radio should be laid out");
    let slider = layout
        .find_role(view::Role::Slider)
        .into_iter()
        .next()
        .expect("slider should be laid out");

    assert!(scene.texts().iter().any(|text| text.value() == "Wrap"));
    assert!(scene.texts().iter().any(|text| text.value() == "Soft tabs"));
    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value().starts_with("Zoom: 5.00"))
    );
    assert!(
        scene
            .texts()
            .iter()
            .all(|text| !text.value().contains("..")),
        "slider range should not wrap into clipped control text"
    );
    assert!(
        scene
            .texts()
            .iter()
            .all(|text| !text.value().starts_with("[") && !text.value().starts_with("(")),
        "control state should be painted, not encoded in labels"
    );
    assert!(scene.icons().iter().any(|icon| {
        icon.icon().id().as_str() == "check" && rect_contains(checkbox.rect(), icon.rect())
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (245, 245, 247, 255)
            && rect_contains(checkbox.rect(), quad.rect())
            && quad.rounding() == scene::Rounding::fixed(4.0)
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (10, 132, 255, 255)
            && rect_contains(radio.rect(), quad.rect())
            && quad.rounding() == scene::Rounding::relative(1.0)
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (58, 58, 60, 255)
            && rect_contains(slider.rect(), quad.rect())
            && quad.rect().height() == 4
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (10, 132, 255, 255)
            && rect_contains(slider.rect(), quad.rect())
            && quad.rect().height() == 4
    }));
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (245, 245, 247, 255)
            && rect_contains(slider.rect(), quad.rect())
            && quad.rounding() == scene::Rounding::relative(1.0)
    }));
}

#[test]
fn choice_marks_paint_pressed_tint_above_mark_without_label_overlay() {
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Record"));
        })
        .responders(|responders| {
            responders.app().target::<RecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Choice Pressed Paint"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.checkbox(widget::Checkbox::new("Wrap", true).trigger::<RecordSource>(()));
                    ui.radio(widget::Radio::new("Soft tabs", true).trigger::<RecordSource>(()));
                });
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(320, 120);
    let initial = app
        .show_scene(window, size)
        .expect("choice view should render");
    let checkbox = initial
        .layout()
        .find_role(view::Role::Checkbox)
        .into_iter()
        .next()
        .expect("checkbox should be laid out");
    let checkbox_point = frame_point_at(checkbox.active_rect());

    let checkbox_down = app
        .pointer_down_at(window, size, checkbox_point)
        .expect("checkbox pointer down should be handled");
    assert!(checkbox_down.is_handled());
    let checkbox_pressed = app
        .show_scene(window, size)
        .expect("checkbox pressed state should render");
    let pressed_checkbox = checkbox_pressed
        .layout()
        .find_role(view::Role::Checkbox)
        .into_iter()
        .next()
        .expect("pressed checkbox should be laid out");
    assert!(pressed_checkbox.is_enabled());
    assert_choice_pressed_tint_above_mark_chrome(&checkbox_pressed, pressed_checkbox.active_rect());
    assert_no_choice_label_overlay(&checkbox_pressed, pressed_checkbox.rect());

    app.handle_input(window, Input::cancel())
        .expect("cancel should reset checkbox pressed state");
    let released = app
        .show_scene(window, size)
        .expect("choice view should render after checkbox release");
    let radio = released
        .layout()
        .find_role(view::Role::Radio)
        .into_iter()
        .next()
        .expect("radio should be laid out");
    let radio_point = frame_point_at(radio.active_rect());

    let radio_down = app
        .pointer_down_at(window, size, radio_point)
        .expect("radio pointer down should be handled");
    assert!(radio_down.is_handled());
    let radio_pressed = app
        .show_scene(window, size)
        .expect("radio pressed state should render");
    let pressed_radio = radio_pressed
        .layout()
        .find_role(view::Role::Radio)
        .into_iter()
        .next()
        .expect("pressed radio should be laid out");
    assert!(pressed_radio.is_enabled());
    assert_choice_pressed_tint_above_mark_chrome(&radio_pressed, pressed_radio.active_rect());
    assert_no_choice_label_overlay(&radio_pressed, pressed_radio.rect());
}

#[test]
fn scene_paint_accepts_theme_data_variants() {
    let view = widget::view(|ui| {
        ui.button(widget::Button::new("Action"));
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(180, 60), &mut layout_engine);
    let dark = scene::Scene::paint(&layout);
    let light_theme = Theme::light();
    let light = scene::Scene::paint_with_theme(&layout, &light_theme);
    let root = geometry::Rect::new(0, 0, 180, 60);
    let dark_root = dark
        .quads()
        .into_iter()
        .find(|quad| quad.rect() == root)
        .expect("dark scene should paint the root");
    let light_root = light
        .quads()
        .into_iter()
        .find(|quad| quad.rect() == root)
        .expect("light scene should paint the root");

    assert_eq!(dark.clear(), Theme::default().surfaces().canvas);
    assert_eq!(dark.clear(), window::DEFAULT_CANVAS_COLOR);
    assert_eq!(light.clear(), light_theme.surfaces().canvas);
    assert_ne!(dark.clear(), light.clear());
    assert_ne!(dark_root.fill(), light_root.fill());
}

#[test]
fn theme_toml_tokens_drive_layout_and_scene_primitives() {
    let theme = Theme::from_toml_str(
        r##"
        [palette]
        brand = "#112233"

        [surfaces]
        root = "brand"

        [text]
        primary = "#445566"

        [typography]
        interface-size = 13.5
        interface-weight = "bold"

        [control]
        button-background = "#334455"
        rounding = { fixed = 9.0 }
        height = 30

        [menu]
        bar-background = "#010203"
        bar-height = 34
        row-height = 34

        [floating-panel]
        material = { kind = "glass", recipe = "panel-dark", blur-sigma = 24.0, tint = { from = "#22334488", to = "#33445599" }, tint-opacity = 1.0, refraction-displacement = 3.0, refraction-splay = 1.5, refraction-feather = 12.0, refraction-curve = 2.5 }
        rounding = { fixed = 13.0 }
        padding = 10
        "##,
    )
    .expect("theme TOML should parse");
    let view = View::new(
        view::Node::root()
            .child(
                view::Node::stack(view::Axis::Vertical)
                    .child(view::Node::menu_bar().child(view::Node::menu("menu.file", "File")))
                    .child(view::Node::button("Run")),
            )
            .child(view::Node::floating_panel("panel").child(view::Node::label("Item"))),
    );
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose_with_theme(
        &view,
        geometry::Size::new(220, 120),
        &mut layout_engine,
        &theme,
    );
    let scene = scene::Scene::paint_with_theme(&layout, &theme);
    let menu_bar = layout
        .find_role(view::Role::MenuBar)
        .into_iter()
        .next()
        .expect("menu bar should be laid out");
    let button = layout
        .find_role(view::Role::Button)
        .into_iter()
        .next()
        .expect("button should be laid out");
    let popup = layout
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("popup should be laid out");
    let item = layout
        .find_role(view::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Item"))
        .expect("popup item should be laid out");

    assert_eq!(menu_bar.rect().height(), 34);
    assert_eq!(button.rect().height(), 30);
    assert_eq!(
        popup.rect().height(),
        item.rect()
            .height()
            .saturating_add(theme.floating_panel().padding.saturating_mul(2))
    );
    assert!(scene.quads().iter().any(|quad| {
        quad.rect() == menu_bar.rect() && quad.fill() == scene::Color::rgb(1, 2, 3)
    }));
    let menu_title = scene_text(&scene, "File");
    assert_eq!(menu_title.style().size(), 13.5);
    assert_eq!(menu_title.style().weight(), text::document::Weight::Bold);
    assert!(scene.quads().iter().any(|quad| {
        quad.rect() == button.rect()
            && quad.fill() == scene::Color::rgb(51, 68, 85)
            && quad.rounding() == scene::Rounding::fixed(9.0)
    }));
    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value() == "Run" && text.color() == scene::Color::rgb(68, 85, 102))
    );
    let pane = scene_pane_at(&scene, popup.rect()).expect("popup should paint one glass pane");
    assert_eq!(pane.rounding(), scene::Rounding::fixed(13.0));
    let scene::Material::Glass(glass) = pane.material() else {
        panic!("popup pane should carry glass material");
    };
    assert!(glass.backdrop_layers().iter().any(|layer| {
        matches!(layer, scene::BackdropLayer::Blur(blur) if blur.sigma() == 24.0)
    }));
    assert!(glass.backdrop_layers().iter().any(|layer| {
        matches!(
            layer,
            scene::BackdropLayer::Refraction(refraction) if refraction.displacement() == 3.0
                && refraction.splay() == 1.5
                && refraction.feather() == 12.0
                && refraction.curve() == 2.5
        )
    }));
    assert!(glass.surface_layers().iter().any(|layer| {
        matches!(
            layer,
            scene::SurfaceLayer::Tint { brush, opacity }
                if *brush == scene::Brush::linear_gradient(
                    scene::Color::rgba(34, 51, 68, 136),
                    scene::Color::rgba(51, 68, 85, 153),
                ) && *opacity == 1.0
        )
    }));
}

#[test]
fn menu_bar_labels_are_center_aligned() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(520, 180))
        .expect("text editor should render");
    let file = rendered
        .scene()
        .texts()
        .into_iter()
        .find(|text| text.value() == "File")
        .expect("file menu label should paint");

    assert_eq!(file.align(), scene::TextAlign::Center);
}

#[test]
fn open_menu_projects_menu_bar_state_without_popup_title_text() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file_menu = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should exist")
        .menu_action()
        .expect("file menu should have an action");

    app.handle_view(window, file_menu)
        .expect("menu action should be handled");

    let rendered = app
        .show_scene_after_overlay_fade(window, geometry::Size::new(800, 600))
        .expect("open file menu should render");
    assert_eq!(
        rendered
            .scene()
            .texts()
            .into_iter()
            .filter(|text| text.value() == "File")
            .count(),
        1
    );
}

#[test]
fn control_gallery_example_renders_interactive_widget_scene() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(760, 520))
        .expect("control gallery should render");
    let scene = rendered.scene();

    assert!(scene.texts().iter().any(|text| text.value() == "Controls"));
    assert!(scene.texts().iter().any(|text| text.value() == "Wrap text"));
    assert!(scene.texts().iter().any(|text| text.value() == "Design"));
    assert!(
        scene
            .texts()
            .iter()
            .any(|text| text.value().starts_with("Level: 42.00"))
    );
    assert!(
        scene
            .icons()
            .iter()
            .any(|icon| icon.icon().id().as_str() == "check")
    );
    assert!(scene.quads().iter().any(|quad| {
        quad.fill().channels() == (10, 132, 255, 255)
            && quad.rounding() == scene::Rounding::relative(1.0)
    }));
}

#[test]
fn control_gallery_table_emits_sort_intent_and_app_owns_provider_order() {
    let mut state = control_gallery::State::default();
    state.show_advanced = false;
    let mut app = control_gallery::app(state);
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 660);
    let initial = app
        .show_scene(window, size)
        .expect("gallery record table should render");
    assert!(
        initial
            .layout()
            .frames()
            .iter()
            .any(|frame| frame.table_cell().is_some() && frame.label_text() == Some("Record 0"))
    );
    let sort_text = initial
        .scene()
        .texts()
        .into_iter()
        .find(|text| text.value() == "Record")
        .unwrap_or_else(|| {
            panic!(
                "gallery should paint its sort header; painted={:?}",
                initial
                    .scene()
                    .texts()
                    .iter()
                    .map(|text| text.value())
                    .collect::<Vec<_>>()
            )
        });
    let sort_point = frame_point_at(sort_text.rect());
    let sort_header = initial
        .layout()
        .find_role(view::Role::Button)
        .into_iter()
        .find(|frame| frame.rect().contains(sort_point))
        .expect("gallery should provide an ordinary command-bound sort header");
    let sort_icon = initial
        .scene()
        .icons()
        .into_iter()
        .find(|icon| {
            icon.icon().id().as_str() == "caret-up"
                && sort_header.rect().contains(frame_point_at(icon.rect()))
        })
        .expect("active ascending sort should paint one trailing chevron");
    assert_eq!(
        sort_text.rect(),
        layout::table_header_label_rect(sort_header.rect(), true, &Theme::default())
    );
    assert_eq!(
        sort_icon.rect(),
        layout::table_sort_indicator_rect(sort_header.rect(), &Theme::default())
    );
    let count_header = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_part() == Some(view::TablePart::HeaderControl)
                && frame.label_text() == Some("Count")
        })
        .expect("inactive sortable count header");
    for label in ["Detail", "Note", "Enabled"] {
        assert!(
            initial.layout().frames().iter().any(|frame| {
                frame.table_part() == Some(view::TablePart::HeaderControl)
                    && frame.label_text() == Some(label)
            }),
            "{label} should derive a sortable header from its Ord value"
        );
    }
    assert!(initial.scene().icons().iter().all(|icon| {
        !matches!(icon.icon().id().as_str(), "caret-up" | "caret-down")
            || !count_header.rect().contains(frame_point_at(icon.rect()))
    }));
    let sort_track = initial
        .layout()
        .table_tracks()
        .iter()
        .find(|track| track.header_node() == Some(sort_header.node_id()))
        .expect("sort header should own a column track");
    let boundary_point = frame_point_at(
        sort_track
            .divider_hit_rect()
            .expect("sort header resize hit zone"),
    );
    app.pointer_down_at(window, size, boundary_point)
        .expect("sort boundary press should start resizing");
    app.pointer_up_at(window, size, boundary_point)
        .expect("sort boundary release should not activate the header");
    assert_eq!(
        app.state().record_sort.direction(),
        crate::table::SortDirection::Ascending
    );
    let point = frame_point_at(sort_icon.rect());
    app.pointer_down_at(window, size, point)
        .expect("decorative sort chevron should route to the header target");
    app.pointer_up_at(window, size, point)
        .expect("sort header release should emit its command");

    assert_eq!(
        app.state().record_sort.direction(),
        crate::table::SortDirection::Descending
    );
    let sorted = app
        .show_scene(window, size)
        .expect("application-updated provider order should render");
    let descending_header = sorted
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_part() == Some(view::TablePart::HeaderControl)
                && frame.label_text() == Some("Record")
        })
        .expect("descending record header");
    assert!(sorted.scene().icons().iter().any(|icon| {
        icon.icon().id().as_str() == "caret-down"
            && descending_header
                .rect()
                .contains(frame_point_at(icon.rect()))
    }));
    assert!(
        sorted.layout().frames().iter().any(
            |frame| frame.table_cell().is_some() && frame.label_text() == Some("Record 999999")
        )
    );

    let enabled_header = sorted
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_part() == Some(view::TablePart::HeaderControl)
                && frame.label_text() == Some("Enabled")
        })
        .expect("Enabled should derive a Boolean sort header");
    let enabled_point = frame_point_at(enabled_header.rect());
    drop(sorted);
    app.pointer_down_at(window, size, enabled_point)
        .expect("Enabled header should press");
    app.pointer_up_at(window, size, enabled_point)
        .expect("Enabled header should emit ascending sort intent");
    assert_eq!(app.state().record_sort.column().as_str(), "enabled");
    assert_eq!(
        app.state().record_sort.direction(),
        crate::table::SortDirection::Ascending
    );
    let enabled_ascending = app
        .show_scene(window, size)
        .expect("ascending Boolean order should render");
    let first_enabled = enabled_ascending
        .layout()
        .frames()
        .iter()
        .filter(|frame| {
            frame
                .table_cell()
                .is_some_and(|cell| cell.column() == interaction::Id::new("enabled"))
        })
        .min_by_key(|frame| frame.rect().y())
        .expect("ascending Boolean order should materialize an Enabled cell");
    assert!(
        first_enabled
            .checkbox()
            .is_some_and(|checkbox| !checkbox.checked()),
        "false values lead ascending Boolean order"
    );
    let enabled_header = enabled_ascending
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_part() == Some(view::TablePart::HeaderControl)
                && frame.label_text() == Some("Enabled")
        })
        .expect("active Enabled header");
    let enabled_point = frame_point_at(enabled_header.rect());
    drop(enabled_ascending);
    app.pointer_down_at(window, size, enabled_point)
        .expect("active Enabled header should press");
    app.pointer_up_at(window, size, enabled_point)
        .expect("active Enabled header should emit descending sort intent");
    let enabled_descending = app
        .show_scene(window, size)
        .expect("descending Boolean order should render");
    let first_enabled = enabled_descending
        .layout()
        .frames()
        .iter()
        .filter(|frame| {
            frame
                .table_cell()
                .is_some_and(|cell| cell.column() == interaction::Id::new("enabled"))
        })
        .min_by_key(|frame| frame.rect().y())
        .expect("descending Boolean order should materialize an Enabled cell");
    assert!(
        first_enabled
            .checkbox()
            .is_some_and(view::Checkbox::checked),
        "true values lead descending Boolean order"
    );
}

#[test]
fn expanded_sort_header_stays_single_line_beside_a_trailing_active_chevron() {
    const LABEL: &str = "A deliberately long sortable column header";
    let source = crate::table::Source::new(
        1,
        |_| crate::virtual_list::Key::new(0),
        |key| (key == crate::virtual_list::Key::new(0)).then_some(0),
        |_| "value".to_owned(),
    );
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<crate::table::SortBy>(command::Spec::new("Sort table"));
        })
        .responders(|responders| {
            responders.app().target::<crate::table::SortBy>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Single-line sort header"));
        })
        .view(move |_, _| {
            let columns = vec![
                crate::table::Column::text(
                    "long",
                    LABEL,
                    view::Dimension::fixed(120),
                    |value: &String| value,
                )
                .build(),
            ];
            widget::view_node(
                crate::Table::typed("wrapped.sort", 24, columns, source.clone())
                    .sorted_by("long", crate::table::SortDirection::Ascending)
                    .presentation(crate::table::Presentation::Expanded)
                    .width(view::Dimension::fixed(120))
                    .height(view::Dimension::fixed(160)),
            )
        });
    app.start();
    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(120, 160))
        .expect("expanded sortable header should render");
    let header = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_part() == Some(view::TablePart::HeaderControl))
        .expect("sortable header frame");
    let label = rendered
        .scene()
        .texts()
        .into_iter()
        .find(|text| {
            text.rect() == layout::table_header_label_rect(header.rect(), true, &Theme::default())
        })
        .expect("overflow-resolved header label");
    let chevron = rendered
        .scene()
        .icons()
        .into_iter()
        .find(|icon| icon.icon().id().as_str() == "caret-up")
        .expect("active sort chevron");

    let header_height = header.rect().height();
    assert_eq!(label.wrap(), scene::TextWrap::None);
    assert_eq!(label.overflow(), text::Overflow::EllipsisEnd);
    assert!(label.value().contains('…'));
    assert_eq!(
        label.rect(),
        layout::table_header_label_rect(header.rect(), true, &Theme::default())
    );
    assert_eq!(
        chevron.rect(),
        layout::table_sort_indicator_rect(header.rect(), &Theme::default())
    );
    assert!(label.rect().right() <= chevron.rect().x());
    let measured = layout::Engine::new()
        .test_label_width_with_style(label.value(), Theme::default().typography().interface());
    for scale in [1.0_f32, 1.25, 1.5, 2.0] {
        assert!((measured as f32 * scale).ceil() <= (label.rect().width() as f32 * scale).ceil());
    }

    let track = rendered
        .layout()
        .table_tracks()
        .iter()
        .find(|track| track.header_node() == Some(header.node_id()))
        .expect("long header track");
    let boundary = frame_point_at(track.divider_hit_rect().expect("header resize zone"));
    let narrower = geometry::Point::new(boundary.x() - 32, boundary.y());
    drop(rendered);
    app.pointer_down_at(window, geometry::Size::new(120, 160), boundary)
        .expect("header resize should capture");
    app.pointer_drag_at(window, geometry::Size::new(120, 160), narrower)
        .expect("header resize should move");
    app.pointer_up_at(window, geometry::Size::new(120, 160), narrower)
        .expect("header resize should release");
    let resized = app
        .show_scene(window, geometry::Size::new(120, 160))
        .expect("resized expanded header");
    let resized_header = resized
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_part() == Some(view::TablePart::HeaderControl))
        .expect("resized sortable header");
    assert_eq!(resized_header.rect().height(), header_height);
    assert!(resized.scene().texts().iter().any(|text| {
        text.rect()
            == layout::table_header_label_rect(resized_header.rect(), true, &Theme::default())
            && text.wrap() == scene::TextWrap::None
            && text.overflow() == text::Overflow::EllipsisEnd
    }));
}

#[test]
fn control_gallery_compact_and_expanded_tables_share_tracks_and_change_row_flow() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 700);
    let compact = app.show_scene(window, size).expect("compact gallery table");
    let compact_tracks = compact
        .layout()
        .table_tracks()
        .iter()
        .filter(|track| track.axis() == layout::table::Axis::Column)
        .map(layout::table::Track::boundary)
        .collect::<Vec<_>>();
    let compact_header_heights = compact
        .layout()
        .frames()
        .iter()
        .filter(|frame| frame.table_header_cell().is_some())
        .map(|frame| frame.rect().height())
        .collect::<Vec<_>>();
    assert!(
        compact
            .layout()
            .frames()
            .iter()
            .filter(|frame| frame.table_row().is_some())
            .all(|frame| frame.rect().height() == 24)
    );
    assert!(compact.layout().frames().iter().any(|frame| {
        frame.table_cell().is_some()
            && frame.label_text().is_some_and(|value| value.contains('…'))
            && frame.world_text_overflow() == Some(text::Overflow::EllipsisMiddle)
            && frame.world_text_wrap() == Some(view::Wrap::None)
    }));
    let compact_note = compact
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(0)
                    && cell.column() == interaction::Id::new("note")
            })
        })
        .expect("compact editable note display");
    let note_identity = compact_note.node_id();
    assert_eq!(compact_note.role(), view::Role::TextBox);
    assert_eq!(compact_note.table_part(), Some(view::TablePart::Cell));
    assert_eq!(compact_note.world_text_wrap(), Some(view::Wrap::None));
    assert_eq!(
        compact_note.world_text_overflow(),
        Some(text::Overflow::EllipsisEnd)
    );
    let compact_record = compact
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(0)
                    && cell.column() == interaction::Id::new("record")
            })
        })
        .expect("compact ordinary record display");
    assert_eq!(compact_record.role(), view::Role::TextArea);
    assert_eq!(compact_record.world_text_wrap(), Some(view::Wrap::None));
    let compact_count = compact
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(0)
                    && cell.column() == interaction::Id::new("count")
            })
        })
        .expect("compact numeric display");
    let count_identity = compact_count.node_id();
    assert_eq!(compact_count.role(), view::Role::TextBox);
    assert_eq!(compact_count.world_text_align(), Some(view::Align::End));
    assert_eq!(
        compact_count.text_area_text_rect().right(),
        layout::table_content_rect(compact_count.rect(), &Theme::default()).right()
    );
    assert_eq!(compact_count.label_text(), Some("0"));
    assert_eq!(compact_count.overflow_tip(), None);
    assert!(
        !compact_count
            .text_area_layout()
            .expect("count uses resolved selectable shaping")
            .render_surfaces()
            .is_empty()
    );
    let toggle = compact
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.role() == view::Role::Checkbox && frame.label_text() == Some("Expanded rows")
        })
        .expect("visible presentation toggle");
    let point = frame_point_at(layout::choice_mark_rect(toggle.rect(), &Theme::default()));
    drop(compact);
    app.pointer_down_at(window, size, point)
        .expect("presentation toggle press");
    app.pointer_up_at(window, size, point)
        .expect("presentation toggle release");
    assert!(app.state().expanded_rows);

    let expanded = app
        .show_scene(window, size)
        .expect("expanded gallery table");
    let expanded_tracks = expanded
        .layout()
        .table_tracks()
        .iter()
        .filter(|track| track.axis() == layout::table::Axis::Column)
        .map(layout::table::Track::boundary)
        .collect::<Vec<_>>();
    assert_eq!(expanded_tracks, compact_tracks);
    assert_eq!(
        expanded
            .layout()
            .frames()
            .iter()
            .filter(|frame| frame.table_header_cell().is_some())
            .map(|frame| frame.rect().height())
            .collect::<Vec<_>>(),
        compact_header_heights
    );
    let expanded_note = expanded
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(0)
                    && cell.column() == interaction::Id::new("note")
            })
        })
        .expect("expanded editable note display");
    assert_eq!(expanded_note.node_id(), note_identity);
    assert_eq!(expanded_note.role(), view::Role::TextBox);
    assert_eq!(expanded_note.table_part(), Some(view::TablePart::Cell));
    assert_eq!(expanded_note.world_text_wrap(), Some(view::Wrap::Word));
    assert_eq!(
        expanded_note.world_text_overflow(),
        Some(text::Overflow::Clip)
    );
    assert_eq!(expanded_note.overflow_tip(), None);
    let expanded_rows = expanded
        .layout()
        .frames()
        .iter()
        .filter(|frame| frame.table_row().is_some())
        .map(|frame| frame.rect().height())
        .collect::<Vec<_>>();
    assert!(!expanded_rows.is_empty());
    assert!(expanded_rows.iter().all(|height| *height >= 24));
    assert!(expanded_rows.iter().any(|height| *height > 24));
    let expanded_count = expanded
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(0)
                    && cell.column() == interaction::Id::new("count")
            })
        })
        .expect("expanded numeric display");
    assert_eq!(expanded_count.node_id(), count_identity);
    assert_eq!(expanded_count.role(), view::Role::TextBox);
    assert_eq!(expanded_count.world_text_align(), Some(view::Align::End));
    assert_eq!(
        expanded_count.text_area_text_rect().right(),
        layout::table_content_rect(expanded_count.rect(), &Theme::default()).right()
    );
    assert_eq!(expanded_count.label_text(), Some("0"));
    assert!(
        !expanded_count
            .text_area_layout()
            .expect("expanded count uses the same selectable shaping")
            .render_surfaces()
            .is_empty()
    );
    assert!(expanded.layout().frames().iter().any(|frame| {
        frame.table_cell().is_some()
            && frame
                .label_text()
                .is_some_and(|value| value.starts_with("Application-owned detail for record 0"))
            && frame.world_text_wrap() == Some(view::Wrap::Word)
            && frame.world_text_overflow() == Some(text::Overflow::Clip)
    }));
    assert!(
        expanded
            .layout()
            .frames()
            .iter()
            .all(|frame| { frame.table_header_cell().is_none() || frame.rect().height() == 30 })
    );
}

#[test]
fn table_mode_toggle_preserves_pinned_active_editor_through_scroll_resize_and_rebuild() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 700);
    let cell = crate::table::Cell::new(
        interaction::Id::new("control_gallery.records"),
        crate::virtual_list::Key::new(0),
        interaction::Id::new("note"),
    );
    app.show_scene(window, size)
        .expect("compact gallery table should render");
    app.handle_input(
        window,
        Input::focus(session::Focus::table_cell(cell).keyboard()),
    )
    .expect("note cell should focus");
    app.handle_input(
        window,
        Input::key_down(input::Key::F2, input::Modifiers::default()),
    )
    .expect("F2 should deliberately activate the note editor");
    app.handle_input(window, Input::text_commit("retained draft"))
        .expect("active note editor should accept a draft");

    let editing = app
        .show_scene(window, size)
        .expect("active compact editor should render");
    let editor = editing
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("active compact note editor");
    let editor_identity = editor.node_id();
    let list = editing.layout().find_role(view::Role::VirtualList)[0].clone();
    assert_eq!(editor.table_part(), Some(view::TablePart::Cell));
    assert_eq!(editor.role(), view::Role::TextBox);
    drop(editing);

    app.scroll_at(
        window,
        size,
        frame_point_at(list.rect()),
        interaction::ScrollDelta::vertical(720),
    )
    .expect("short table viewport should scroll");
    let scrolled = app
        .show_scene(window, size)
        .expect("pinned editor should survive scrolling");
    let pinned = scrolled
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("active editor row should remain pinned offscreen");
    assert_eq!(pinned.node_id(), editor_identity);
    assert!(pinned.rect().bottom() <= list.rect().y());
    drop(scrolled);

    app.change(state::Reason::programmatic("expand table rows"), |state| {
        state.expanded_rows = true;
    });
    let expanded = app
        .show_scene(window, size)
        .expect("expanded table should retain the active editor");
    let expanded_editor = expanded
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("expanded active editor");
    assert_eq!(expanded_editor.node_id(), editor_identity);
    assert_eq!(expanded_editor.role(), view::Role::TextBox);
    let note_track = expanded
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track
                .column_identity()
                .is_some_and(|header| header.column() == interaction::Id::new("note"))
        })
        .expect("note column track")
        .clone();
    let before_width = expanded
        .layout()
        .frames()
        .iter()
        .find(|frame| Some(frame.node_id()) == note_track.header_node())
        .expect("note header frame")
        .rect()
        .width();
    let resize_start = frame_point_at(
        note_track
            .divider_hit_rect()
            .expect("note column resize hit zone"),
    );
    drop(expanded);
    app.pointer_down_at(window, size, resize_start)
        .expect("note resize should capture");
    let resize_end = geometry::Point::new(resize_start.x() + 36, resize_start.y());
    app.pointer_move_at(window, size, resize_end)
        .expect("note resize should update its source override");
    app.pointer_up_at(window, size, resize_end)
        .expect("note resize should release");
    app.request_redraw(window);

    let rebuilt = app
        .show_scene(window, size)
        .expect("resized expanded table should rebuild");
    let rebuilt_editor = rebuilt
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("rebuilt active editor");
    let rebuilt_track = rebuilt
        .layout()
        .table_tracks()
        .iter()
        .find(|track| {
            track
                .column_identity()
                .is_some_and(|header| header.column() == interaction::Id::new("note"))
        })
        .expect("rebuilt note column track");
    assert_eq!(rebuilt_editor.node_id(), editor_identity);
    let rebuilt_width = rebuilt
        .layout()
        .frames()
        .iter()
        .find(|frame| Some(frame.node_id()) == rebuilt_track.header_node())
        .expect("rebuilt note header frame")
        .rect()
        .width();
    assert_eq!(rebuilt_width, before_width + 36);
    assert_eq!(
        text_draft(&app, window, session::Focus::table_cell(cell)).text(),
        "retained draft"
    );
    assert!(
        rebuilt
            .layout()
            .frames()
            .iter()
            .filter(|frame| frame.table_cell().is_some())
            .count()
            <= 84,
        "toggle/resize work should remain bounded to materialized cells"
    );
}

#[test]
fn table_participation_changes_chrome_without_changing_control_behavior() {
    let mut gallery = control_gallery::app(control_gallery::State::default());
    gallery.start();
    let window = gallery.session().windows()[0].id();
    let size = geometry::Size::new(760, 660);
    let rendered = gallery
        .show_scene(window, size)
        .expect("gallery table should render");
    let theme = Theme::default();

    let sort_header = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_part() == Some(view::TablePart::HeaderControl)
                && frame.label_text() == Some("Record")
        })
        .expect("sortable button should participate as a header control");
    assert_eq!(sort_header.role(), view::Role::Button);
    assert!(sort_header.target().is_some());
    assert!(rendered.scene().quads().iter().any(|quad| {
        quad.rect() == sort_header.rect()
            && quad.fill() == theme.table().header_background
            && quad.rounding() == scene::Rounding::none()
    }));
    assert!(!rendered.scene().quads().iter().any(|quad| {
        quad.rect() == sort_header.rect()
            && quad.fill() == theme.control().button_background
            && quad.rounding() == theme.control().rounding
    }));

    let action = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_part() == Some(view::TablePart::Action)
                && frame.label_text() == Some("Open")
        })
        .expect("intentional cell action should remain a button");
    assert_eq!(action.role(), view::Role::Button);
    assert!(action.target().is_some());
    assert!(rendered.scene().quads().iter().any(|quad| {
        quad.rect() == action.rect()
            && quad.fill() == theme.control().button_background
            && quad.rounding() == theme.control().rounding
    }));

    let toggle = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_part() == Some(view::TablePart::Toggle)
                && frame
                    .table_cell()
                    .is_some_and(|cell| cell.row().value() == 0)
        })
        .expect("typed boolean cell should derive an interactive toggle");
    assert_eq!(toggle.role(), view::Role::Checkbox);
    assert!(toggle.target().is_some());
    assert_eq!(toggle.checked(), Some(true));
    let toggle_rect = toggle.rect();
    let toggle_mark = layout::table_choice_mark_rect(toggle_rect, &theme);
    assert!(
        rendered
            .scene()
            .icons()
            .iter()
            .any(|icon| icon.rect() == toggle_mark)
    );
    assert!(
        rendered
            .scene()
            .quads()
            .iter()
            .any(|quad| quad.rect() == toggle_mark),
        "typed toggles retain interactive checkbox behavior inside table chrome"
    );
    let toggle_point = frame_point_at(toggle_mark);
    drop(rendered);
    gallery
        .pointer_down_at(window, size, toggle_point)
        .expect("first typed-toggle press should select the row");
    gallery
        .pointer_up_at(window, size, toggle_point)
        .expect("selection-only typed-toggle release should be inert");
    assert_eq!(gallery.state().record_enabled.get(&0), None);
    gallery
        .show_scene(window, size)
        .expect("selected toggle row should reproject");
    gallery
        .pointer_down_at(window, size, toggle_point)
        .expect("second typed-toggle press should be handled");
    gallery
        .pointer_up_at(window, size, toggle_point)
        .expect("second typed-toggle release should emit its command");
    assert_eq!(gallery.state().record_enabled.get(&0), Some(&false));

    let mut editable = editable_table_app(EditableTableState {
        records: vec![EditableRecord {
            key: 7,
            name: "Ada".to_owned(),
            count: 4,
        }],
    });
    editable.start();
    let editable_window = editable.session().windows()[0].id();
    let editable_size = geometry::Size::new(320, 124);
    let cell = crate::table::Cell::new(
        interaction::Id::new("editable.table"),
        crate::virtual_list::Key::new(7),
        interaction::Id::new("name"),
    );
    let idle = editable
        .show_scene(editable_window, editable_size)
        .expect("idle editor should render");
    let idle_editor = idle
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("editor frame");
    assert_eq!(idle_editor.table_part(), Some(view::TablePart::Cell));
    assert_eq!(idle_editor.role(), view::Role::TextBox);
    assert!(idle.scene().quads().iter().all(|quad| {
        quad.rect() != idle_editor.rect() || quad.fill() != theme.text_input().field_background
    }));

    editable
        .handle_input(
            editable_window,
            Input::focus(session::Focus::table_cell(cell)),
        )
        .expect("editor should focus");
    let focused_display = editable
        .show_scene(editable_window, editable_size)
        .expect("focused display cell should render");
    let focused_cell = focused_display
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("focused display cell frame");
    assert_eq!(focused_cell.table_part(), Some(view::TablePart::Cell));
    editable
        .handle_input(
            editable_window,
            Input::key_down(input::Key::F2, input::Modifiers::default()),
        )
        .expect("F2 should enter the focused cell");
    let focused = editable
        .show_scene(editable_window, editable_size)
        .expect("active editor should render");
    let focused_editor = focused
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.table_cell() == Some(cell))
        .expect("active editor frame");
    assert_eq!(focused_editor.table_part(), Some(view::TablePart::Cell));
    let inset = geometry::Rect::new(
        focused_editor.rect().x() + 1,
        focused_editor.rect().y() + 1,
        focused_editor.rect().width() - 2,
        focused_editor.rect().height() - 2,
    );
    assert!(focused.scene().outlines().iter().any(|outline| {
        outline.rect() == inset
            && outline.color() == theme.focus().color
            && outline.rounding() == scene::Rounding::none()
    }));
}

#[test]
fn typed_boolean_table_cells_project_value_and_keep_native_toggle_grammar() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 900);
    let initial = app
        .show_scene(window, size)
        .expect("control gallery table should render");
    let enabled = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some_and(|cell| {
                cell.row() == crate::virtual_list::Key::new(0)
                    && cell.column() == interaction::Id::new("enabled")
            })
        })
        .expect("first enabled cell");
    assert!(enabled.checkbox().is_some_and(view::Checkbox::checked));
    let point = frame_point_at(enabled.active_rect());

    app.pointer_down_at(window, size, point)
        .expect("first click should select the checkbox row");
    app.pointer_up_at(window, size, point)
        .expect("selection-only click should finish without activation");
    assert_eq!(app.state().record_enabled.get(&0), None);
    app.show_scene(window, size)
        .expect("selected checkbox row should reproject");
    app.pointer_down_at(window, size, point)
        .expect("second click should press the native checkbox");
    app.pointer_up_at(window, size, point)
        .expect("second click should toggle the native checkbox");
    assert_eq!(app.state().record_enabled.get(&0), Some(&false));

    let unchecked = app
        .show_scene(window, size)
        .expect("unchecked value should reproject");
    assert!(unchecked.layout().frames().iter().any(|frame| {
        frame.table_cell().is_some_and(|cell| {
            cell.row() == crate::virtual_list::Key::new(0)
                && cell.column() == interaction::Id::new("enabled")
        }) && frame.checkbox().is_some_and(|checkbox| !checkbox.checked())
    }));
    app.handle_input(
        window,
        Input::key_down(input::Key::Space, input::Modifiers::default()),
    )
    .expect("Space should activate the focused native checkbox");
    assert_eq!(app.state().record_enabled.get(&0), Some(&true));
}

#[test]
fn control_gallery_choice_labels_are_single_line_row_content() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(760, 520))
        .expect("control gallery should render");

    for (role, label) in [
        (view::Role::Checkbox, "Wrap text"),
        (view::Role::Checkbox, "Show grid"),
        (view::Role::Checkbox, "Advanced"),
        (view::Role::Radio, "Design"),
        (view::Role::Radio, "Inspect"),
        (view::Role::Radio, "Preview"),
    ] {
        let frame = rendered
            .layout()
            .find_role(role)
            .into_iter()
            .find(|frame| frame.label_text() == Some(label))
            .expect("choice frame should be laid out");
        let text = rendered
            .scene()
            .texts()
            .into_iter()
            .find(|text| text.value() == label)
            .expect("choice label should paint");

        assert_eq!(text.wrap(), scene::TextWrap::None);
        assert_eq!(text.rect().y(), frame.rect().y());
        assert_eq!(text.rect().height(), frame.rect().height());
        assert!(text.rect().x() >= frame.active_rect().right());
        assert!(text.rect().right() <= frame.rect().right());
    }
}

#[test]
fn menu_popup_rows_use_row_layout_for_labels_shortcuts_and_separators() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file_menu = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should exist")
        .menu_action()
        .expect("file menu should have an action");

    app.handle_view(window, file_menu)
        .expect("menu action should be handled");

    let rendered = app
        .show_scene_after_overlay_fade(window, geometry::Size::new(800, 600))
        .expect("open file menu should render");
    let exit = rendered
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.role() == view::Role::Binding && frame.label_text() == Some("Close Window")
        })
        .expect("exit row should be laid out");
    let theme = Theme::default();
    let parts = layout::menu_row_parts(exit.rect(), exit.shortcut_width(), &theme);
    let exit_label = rendered
        .scene()
        .texts()
        .into_iter()
        .find(|text| text.value() == "Close Window")
        .expect("exit label should paint");
    let exit_shortcut = rendered
        .scene()
        .texts()
        .into_iter()
        .find(|text| text.value() == "F4" && rect_contains(parts.shortcut, text.rect()))
        .expect("exit shortcut key should paint");
    let exit_shortcut_icon = scene_icon_in_rect(rendered.scene(), "option", parts.shortcut);

    assert_eq!(exit_label.rect(), parts.label);
    assert_eq!(exit_label.align(), scene::TextAlign::Start);
    assert!(rect_contains(parts.shortcut, exit_shortcut.rect()));
    assert_eq!(exit_shortcut.align(), scene::TextAlign::Start);
    assert_eq!(
        exit_shortcut.style().size(),
        theme.typography().interface().size()
    );
    assert_eq!(
        exit_shortcut.style().weight(),
        theme.typography().interface().weight()
    );
    assert_eq!(exit_shortcut.color(), theme.text().muted);
    assert_eq!(exit_shortcut_icon.color(), theme.text().muted);
    assert_eq!(parts.glyph.width(), parts.glyph.height());
    assert_eq!(parts.trailing.width(), parts.trailing.height());

    let separator = rendered
        .layout()
        .find_role(view::Role::Separator)
        .into_iter()
        .next()
        .expect("file menu separator should be laid out");
    let popup = rendered
        .layout()
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("file menu popup should be laid out");
    let separator_parts =
        layout::menu_row_parts(separator.rect(), separator.shortcut_width(), &theme);

    assert_eq!(separator.rect().height(), theme.menu().row_height);
    assert_eq!(
        separator.rect().x(),
        popup.rect().x() + theme.floating_panel().padding
    );
    assert_eq!(
        separator.rect().right(),
        popup.rect().right() - theme.floating_panel().padding
    );
    assert_eq!(separator_parts.separator.x(), separator.rect().x());
    assert_eq!(separator_parts.separator.width(), separator.rect().width());
    assert!(rendered.scene().rules().iter().any(|rule| {
        rule.rect() == separator_parts.separator
            && rule.color() == theme.menu().separator
            && rule.axis() == scene::Axis::Horizontal
            && rule.thickness_px() == 1
    }));
}

#[test]
fn menu_popup_width_uses_active_keymap_profile_for_shortcut_measurement() {
    let text_theme = Theme::from_toml_str(
        r##"
        [shortcuts]
        display = "text"

        [menu]
        panel-min-width = 1
        "##,
    )
    .expect("text shortcut display theme should parse");
    let expected_theme = text_theme.clone();
    let menu_app = || {
        Runtime::new(SourceState::default())
            .commands(|commands| {
                commands.register::<PaletteOne>(command::Spec::new("X").shortcut("Primary+R"));
            })
            .responders(|responders| {
                responders.app().target::<PaletteOne>();
            })
            .started(|cx| {
                cx.open_window(window::Options::new("Shortcut Menu"));
            })
            .view(|_, _| {
                widget::view(|ui| {
                    ui.menu_bar(|ui| {
                        ui.menu("menu.test", "M", |ui| {
                            ui.add(widget::Binding::<PaletteOne>::menu());
                        });
                    });
                })
            })
    };
    let mut windows_app = menu_app().keymap(keymap::Profile::windows()).theme({
        let text_theme = text_theme.clone();
        move |_| text_theme.clone()
    });
    let mut mac_app = menu_app()
        .keymap(keymap::Profile::mac())
        .theme(move |_| text_theme.clone());
    windows_app.start();
    mac_app.start();

    let size = geometry::Size::new(800, 600);
    let windows_window = windows_app.session().windows()[0].id();
    let mac_window = mac_app.session().windows()[0].id();
    let windows_menu = windows_app
        .present(windows_window)
        .expect("windows menu app should present")
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("M"))
        .expect("windows menu should exist")
        .menu_action()
        .expect("windows menu should have an action");
    let mac_menu = mac_app
        .present(mac_window)
        .expect("mac menu app should present")
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("M"))
        .expect("mac menu should exist")
        .menu_action()
        .expect("mac menu should have an action");

    windows_app
        .handle_view(windows_window, windows_menu)
        .expect("windows menu action should open");
    mac_app
        .handle_view(mac_window, mac_menu)
        .expect("mac menu action should open");
    let windows = windows_app
        .show_scene(windows_window, size)
        .expect("windows menu should render");
    let mac = mac_app
        .show_scene(mac_window, size)
        .expect("mac menu should render");
    let windows_popup = windows
        .layout()
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("windows shortcut menu popup should be laid out");
    let mac_popup = mac
        .layout()
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("mac shortcut menu popup should be laid out");
    let mac_row = mac
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.role() == view::Role::Binding && frame.label_text() == Some("X"))
        .expect("mac menu row should be laid out");
    let mac_parts =
        layout::menu_row_parts(mac_row.rect(), mac_row.shortcut_width(), &expected_theme);

    assert!(
        mac_popup.rect().width() > windows_popup.rect().width(),
        "menu panel width should come from the active profile's formatted shortcuts: windows={} mac={} mac_row_shortcut={}",
        windows_popup.rect().width(),
        mac_popup.rect().width(),
        mac_row.shortcut_width()
    );
    assert!(
        mac_parts.shortcut.right()
            <= mac_popup.rect().right() - expected_theme.floating_panel().padding,
        "active-profile shortcut part should fit inside the measured menu popup"
    );
}

#[test]
fn menu_popup_opens_under_its_menu_title() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let view_menu_action = app
        .present(window)
        .expect("control gallery should present")
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("View"))
        .expect("view menu should be projected")
        .menu_action()
        .expect("view menu should have an action");

    app.handle_view(window, view_menu_action)
        .expect("view menu action should open the menu");

    let rendered = app
        .show_scene(window, size)
        .expect("open view menu should render");
    let view_menu = rendered
        .layout()
        .find_role(view::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("View"))
        .expect("view menu should be laid out");
    let popup = rendered
        .layout()
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("view menu popup should be laid out");

    assert!(view_menu.rect().x() > 0);
    assert_eq!(popup.rect().x(), view_menu.rect().x());
    assert_eq!(popup.rect().y(), view_menu.rect().bottom());
}

#[test]
fn menu_titles_paint_hover_pressed_and_active_tints() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let initial = app
        .show_scene(window, size)
        .expect("control gallery should render");
    let menu = initial
        .layout()
        .find_role(view::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Controls"))
        .expect("controls menu should be laid out");
    let point = frame_point(menu);

    assert_no_tint_quad(
        initial.scene(),
        menu.rect(),
        Theme::default().menu().title_background,
    );

    app.pointer_move_at(window, size, point)
        .expect("menu pointer move should be handled");
    let hovered = app
        .show_scene(window, size)
        .expect("hovered menu should render");
    assert_tint_quad(
        hovered.scene(),
        menu.rect(),
        Theme::default().menu().title_hover_tint,
    );

    app.pointer_down_at(window, size, point)
        .expect("menu pointer down should be handled");
    let pressed = app
        .show_scene(window, size)
        .expect("pressed menu should render");
    assert_tint_quad(
        pressed.scene(),
        menu.rect(),
        Theme::default().menu().title_pressed_tint,
    );

    app.pointer_up_at(window, size, point)
        .expect("menu pointer up should be handled");
    let active = app
        .show_scene(window, size)
        .expect("open menu should render active title");
    assert_tint_quad(
        active.scene(),
        menu.rect(),
        Theme::default().menu().title_active_tint,
    );
}

#[test]
fn menu_popup_rows_paint_hover_tint_from_pointer_projection() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let initial = app
        .show_scene(window, size)
        .expect("text editor should render");
    let file = initial
        .layout()
        .find_role(view::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("File"))
        .expect("file menu should be laid out");
    let file_point = frame_point(file);

    app.pointer_down_at(window, size, file_point)
        .expect("file menu pointer down should be handled");
    app.pointer_up_at(window, size, file_point)
        .expect("file menu pointer up should open the menu");
    let opened = app
        .show_scene_after_overlay_fade(window, size)
        .expect("open file menu should render");
    let new_row = opened
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.role() == view::Role::Binding && frame.label_text() == Some("New"))
        .expect("new command row should be laid out");
    let before_hover = app
        .diagnostics(window)
        .expect("diagnostics should exist")
        .frame
        .clone();

    let moved = app
        .pointer_move_at(window, size, frame_point(new_row))
        .expect("popup row pointer move should be handled");

    assert!(moved.is_handled());
    assert_eq!(
        moved.effect().invalidation(),
        Some(response::effect::Invalidation::Paint)
    );
    assert!(moved.effect().contains_invalidation());

    let hovered = app
        .show_scene(window, size)
        .expect("hovered popup row should render");
    let after_hover = app
        .diagnostics(window)
        .expect("diagnostics should exist")
        .frame
        .clone();
    let hovered_row = hovered
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.role() == view::Role::Binding && frame.label_text() == Some("New"))
        .expect("new command row should still be laid out");

    assert_tint_quad(
        hovered.scene(),
        hovered_row.rect(),
        Theme::default().menu().row_hover_tint,
    );
    assert_eq!(after_hover.view_rebuilds, before_hover.view_rebuilds);
    assert_eq!(
        after_hover.layout_recomposes,
        before_hover.layout_recomposes
    );
    assert!(after_hover.layout_reuses > before_hover.layout_reuses);
}

#[test]
fn checked_menu_popup_rows_do_not_paint_active_tint() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let view_menu = app
        .present(window)
        .expect("control gallery should present")
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("View"))
        .expect("view menu should be projected")
        .menu_action()
        .expect("view menu should have an action");

    app.handle_view(window, view_menu)
        .expect("view menu action should open the menu");

    let opened = app
        .show_scene_after_overlay_fade(window, size)
        .expect("open view menu should render");
    let wrap = opened
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.role() == view::Role::Binding && frame.label_text() == Some("Wrap text")
        })
        .expect("checked wrap row should be laid out");
    let theme = Theme::default();
    let parts = layout::menu_row_parts(wrap.rect(), wrap.shortcut_width(), &theme);

    assert_eq!(wrap.checked(), Some(true));
    assert_no_tint_quad(opened.scene(), wrap.rect(), theme.menu().title_active_tint);
    assert!(
        opened
            .scene()
            .icons()
            .iter()
            .any(|icon| { icon.rect() == parts.glyph && icon.icon().id().as_str() == "check" })
    );
}

#[test]
fn focus_outline_uses_frame_rounding() {
    let mut app = control_gallery::app(control_gallery::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    app.show_scene(window, size)
        .expect("control gallery should render before focus");
    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab should focus first menu");

    let focused = app
        .show_scene(window, size)
        .expect("focused menu should render");
    let frame = focused
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.is_focused())
        .expect("focused frame should be present");
    let outline = focused
        .scene()
        .outlines()
        .into_iter()
        .find(|outline| {
            outline.rect() == frame.rect() && outline.color() == Theme::default().focus().color
        })
        .expect("focus outline should paint");

    assert_eq!(outline.rounding(), Theme::default().control().rounding);
    assert_eq!(outline.offset(), Theme::default().focus().offset);
}

#[test]
fn popup_focus_outline_retains_popup_local_rounded_clip() {
    let mut app = text_editor::app(text_editor::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let file_menu = app
        .present(window)
        .expect("text editor should present")
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .and_then(view::Node::menu_action)
        .expect("file menu should expose an open action");
    app.handle_view(window, file_menu)
        .expect("file menu should open");

    let opened = app
        .show_scene_after_overlay_fade(window, size)
        .expect("open menu should render");
    let target = opened
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.role() == view::Role::Binding && frame.label_text() == Some("Open"))
        .and_then(layout::Frame::target)
        .cloned()
        .expect("open binding should expose a focus target");
    assert!(app.focus(window, session::Focus::control(&target).keyboard()));

    let focused = app
        .show_scene_after_overlay_fade(window, size)
        .expect("focused popup should render");
    let frame = focused
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.target() == Some(&target) && frame.is_focused())
        .expect("popup binding should receive keyboard focus");
    let popup = focused
        .layout()
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("focused binding should remain in a popup");
    assert_outline_is_scoped_by_clip(focused.scene(), frame.rect(), popup.rect());

    let outline_index = focused
        .scene()
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(primitive, scene::Primitive::Outline(outline) if outline.rect() == frame.rect())
        })
        .expect("popup focus outline should paint");
    let clip = focused.scene().primitives()[..outline_index]
        .iter()
        .rev()
        .find_map(|primitive| match primitive {
            scene::Primitive::Clip(clip) if clip.rect() == popup.rect() => Some(clip),
            _ => None,
        })
        .expect("popup focus outline should retain its rounded clip");
    assert_eq!(clip.rounding(), Theme::default().floating_panel().rounding);
}

#[test]
fn scene_preserves_popup_paint_order_after_base_content() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file_menu = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should exist")
        .menu_action()
        .expect("file menu should have an action");

    app.handle_view(window, file_menu)
        .expect("menu action should be handled");

    let projected = app.present(window).expect("window should have a view");
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(
        &projected,
        geometry::Size::new(800, 600),
        &mut layout_engine,
    );
    let scene = scene::Scene::paint(&layout);
    let popup = layout
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("file menu popup should be laid out");
    let theme = Theme::default();

    assert!(
        scene
            .shadows()
            .iter()
            .any(|shadow| shadow.color().channels() == (0, 0, 0, 96)),
        "popup paint should include theme-owned elevation"
    );
    let border = scene
        .outlines()
        .into_iter()
        .find(|outline| {
            outline.rect() == popup.rect() && outline.color() == theme.floating_panel().border()
        })
        .expect("menu popup should paint the theme border datum");
    assert_eq!(border.width(), 1.0);
    assert_eq!(border.rounding(), theme.floating_panel().rounding);

    let popup_shadow = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Shadow(shadow) if shadow.rect() == popup.rect()
            )
        })
        .expect("popup shadow should be painted");
    let popup_pane = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Pane(pane)
                    if pane.rect() == popup.rect()
                        && matches!(
                            pane.material(),
                            scene::Material::Glass(glass)
                                if glass.backdrop_layers().iter().any(|layer| {
                                    matches!(layer, scene::BackdropLayer::Blur(blur) if blur.sigma() == 44.55)
                                })
                        )
                        && pane.rounding() == theme.floating_panel().rounding
            )
        })
        .expect("popup pane should be painted");
    let popup_border = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Outline(outline)
                    if outline.rect() == popup.rect()
                        && outline.color() == theme.floating_panel().border()
            )
        })
        .expect("popup border should be painted");
    let file_menu_text = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "File"
            )
        })
        .expect("menu bar file text should be painted");
    let open_command_text = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "Open"
            )
        })
        .expect("popup open command text should be painted");
    let open_command_clip = scene.primitives()[..open_command_text]
        .iter()
        .rposition(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Clip(clip)
                    if clip.rect() == popup.rect()
                        && clip.rounding() == theme.floating_panel().rounding
            )
        })
        .expect("popup row should be clipped to rounded panel");
    let open_command_pop_clip = scene.primitives()[open_command_text..]
        .iter()
        .position(|primitive| matches!(primitive, scene::Primitive::PopClip))
        .map(|index| index + open_command_text)
        .expect("popup row clip should be popped after row content");
    let exit_command_text = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "Close Window"
            )
        })
        .expect("popup exit command text should be painted");
    let exit_shortcut_text = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "F4"
            )
        })
        .expect("popup exit shortcut key text should be painted");

    assert!(popup_shadow < popup_pane);
    assert!(popup_pane < popup_border);
    assert!(popup_border < open_command_text);
    assert!(popup_pane < open_command_text);
    assert!(popup_pane < open_command_clip);
    assert!(open_command_clip < open_command_text);
    assert!(open_command_text < open_command_pop_clip);
    assert!(file_menu_text < open_command_text);
    assert!(file_menu_text < exit_command_text);
    assert!(exit_command_text < exit_shortcut_text);
}

#[test]
fn generic_floating_panel_uses_shared_chrome_before_content() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .overlay()
                .width(view::Dimension::fixed(240))
                .height(view::Dimension::fixed(160))
                .children(|ui| {
                    ui.add(
                        widget::panel::Floating::new("tests.floating")
                            .width(view::Dimension::fixed(180))
                            .height(view::Dimension::fixed(80))
                            .children(|ui| {
                                ui.label("Inside");
                            }),
                    );
                }),
        );
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 160), &mut layout_engine);
    let scene = scene::Scene::paint(&layout);
    let panel = layout
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("floating panel should be laid out");

    let shadow = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Shadow(shadow) if shadow.rect() == panel.rect()
            )
        })
        .expect("floating panel shadow should paint");
    let pane = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Pane(pane) if pane.rect() == panel.rect()
            )
        })
        .expect("floating panel pane should paint");
    let content = scene
        .primitives()
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == "Inside"
            )
        })
        .expect("floating panel content should paint");

    assert!(shadow < pane);
    assert!(pane < content);
}

#[test]
fn floating_panel_offset_places_unanchored_overlay() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .overlay()
                .width(view::Dimension::fixed(240))
                .height(view::Dimension::fixed(160))
                .children(|ui| {
                    ui.add(
                        widget::panel::Floating::new("tests.floating.offset")
                            .offset(24, 18)
                            .width(view::Dimension::fixed(120))
                            .height(view::Dimension::fixed(60))
                            .children(|ui| {
                                ui.label("Offset");
                            }),
                    );
                }),
        );
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(240, 160), &mut layout_engine);
    let panel = layout
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("floating panel should be laid out");

    assert_eq!(panel.rect(), geometry::Rect::new(24, 18, 120, 60));
}

#[test]
fn generic_floating_panel_uses_stack_padding_and_gap_for_content() {
    let view = widget::view(|ui| {
        ui.add(
            widget::panel::Floating::new("tests.floating.layout")
                .column()
                .width(view::Dimension::fixed(220))
                .height(view::Dimension::fixed(140))
                .layout(|layout| layout.gap(7).padding(view::Padding::all(11)))
                .children(|ui| {
                    ui.label("Alpha");
                    ui.label("Beta");
                }),
        );
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(260, 180), &mut layout_engine);
    let theme = Theme::default();
    let panel = layout
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("floating panel should be laid out");
    let alpha = layout
        .find_role(view::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Alpha"))
        .expect("alpha label should be laid out");
    let beta = layout
        .find_role(view::Role::Label)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Beta"))
        .expect("beta label should be laid out");

    assert_eq!(
        alpha.rect().x(),
        panel.rect().x() + theme.floating_panel().padding + 11
    );
    assert_eq!(
        alpha.rect().y(),
        panel.rect().y() + theme.floating_panel().padding + 11
    );
    assert_eq!(beta.rect().y(), alpha.rect().bottom() + 7);
}

#[test]
fn slider_labels_are_single_line_without_default_row_fill() {
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .column()
                .height(view::Dimension::fixed(80))
                .children(|ui| {
                    ui.slider(widget::Slider::new("Feather", 24.0, 0.0..=64.0));
                }),
        );
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(360, 80), &mut layout_engine);
    let scene = scene::Scene::paint(&layout);
    let slider = layout
        .find_role(view::Role::Slider)
        .into_iter()
        .next()
        .expect("slider should be laid out");
    let label = scene
        .texts()
        .into_iter()
        .find(|text| text.value() == "Feather: 24.00")
        .expect("slider label should paint");

    assert_eq!(label.wrap(), scene::TextWrap::None);
    assert!(
        !scene
            .quads()
            .iter()
            .any(|quad| quad.rect() == slider.rect()),
        "default slider row should not paint a filled control background"
    );
}

#[test]
fn overlay_layout_paints_styled_backgrounds_under_floating_panel() {
    let bar = scene::Color::rgb(235, 73, 83);
    let view = widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .overlay()
                .width(view::Dimension::fixed(200))
                .height(view::Dimension::fixed(120))
                .children(|ui| {
                    ui.add(
                        widget::Element::new()
                            .background(scene::Brush::solid(bar))
                            .width(view::Dimension::grow())
                            .height(view::Dimension::grow()),
                    );
                    ui.add(
                        widget::panel::Floating::new("tests.overlay.panel")
                            .width(view::Dimension::fixed(120))
                            .height(view::Dimension::fixed(64))
                            .children(|ui| {
                                ui.label("Panel");
                            }),
                    );
                }),
        );
    });
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose(&view, geometry::Size::new(200, 120), &mut layout_engine);
    let scene = scene::Scene::paint(&layout);
    let panel = layout
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("floating panel should be laid out");

    assert!(
        scene.quads().iter().any(|quad| {
            quad.fill() == bar && quad.rect() == geometry::Rect::new(0, 0, 200, 120)
        })
    );
    assert!(scene.panes().iter().any(|pane| pane.rect() == panel.rect()));
}

#[test]
fn glass_tuner_projects_live_theme_values_and_hit_tests_panel_controls() {
    let mut state = glass_tuner::State::default();
    state.comparison_open = false;
    let mut app = glass_tuner::app(state);

    app.start();

    let window = app.session().windows()[0].id();
    let size = glass_tuner::window_size();
    let initial = app
        .show_scene_after_overlay_fade(window, size)
        .expect("glass tuner should render");
    let panel = initial
        .layout()
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("glass tuner floating panel should be laid out");
    let default_pane = scene_pane_at(initial.scene(), panel.rect())
        .expect("glass tuner panel should paint a pane");
    let scene::Material::Glass(default_glass) = default_pane.material() else {
        panic!("glass tuner pane should carry glass material");
    };
    assert!(default_glass.backdrop_layers().iter().any(|layer| {
        matches!(layer, scene::BackdropLayer::Blur(blur) if *blur == scene::BackdropBlur::new(44.55))
    }));
    assert!(default_glass.surface_layers().iter().any(|layer| {
        matches!(
            layer,
            scene::SurfaceLayer::Tint { brush, opacity }
                if *brush == scene::Brush::solid(scene::Color::rgb(28, 28, 30))
                    && *opacity == 0.40
        )
    }));
    assert!(initial.scene().texts().iter().any(|text| {
        text.value().contains("blur-sigma = 44.55")
            || text.value().contains("noise-opacity = 0.022")
    }));
    assert!(initial.scene().texts().iter().any(|text| {
        text.value() == "Blur sigma: 44.55" && text.wrap() == scene::TextWrap::None
    }));
    assert!(
        !initial
            .scene()
            .texts()
            .iter()
            .any(|text| text.value().contains("refraction")),
        "acrylic tuner should not expose refraction TOML by default"
    );

    app.invoke(app.trigger::<glass_tuner::SetToken>((glass_tuner::AcrylicToken::TintR, 80.0)))
        .output
        .expect("set tint red should succeed");
    app.invoke(
        app.trigger::<glass_tuner::SetToken>((glass_tuner::AcrylicToken::NoiseOpacity, 0.04)),
    )
    .output
    .expect("set noise opacity should succeed");

    let rendered = app
        .show_scene(window, size)
        .expect("glass tuner should render after acrylic tuning");
    let tuned_pane =
        scene_pane_at(rendered.scene(), panel.rect()).expect("tuned panel should paint a pane");
    let scene::Material::Glass(tuned_glass) = tuned_pane.material() else {
        panic!("tuned panel pane should carry glass material");
    };
    assert!(tuned_glass.surface_layers().iter().any(|layer| {
        matches!(
            layer,
            scene::SurfaceLayer::Tint { brush, opacity }
                if *brush == scene::Brush::solid(scene::Color::rgb(80, 28, 30))
                    && *opacity == 0.40
        )
    }));
    assert!(tuned_glass.surface_layers().iter().any(|layer| {
        matches!(layer, scene::SurfaceLayer::Noise(noise) if noise.opacity() == 0.04)
    }));
    assert!(tuned_glass.backdrop_layers().iter().any(|layer| {
        matches!(
            layer,
            scene::BackdropLayer::Luminosity(luminosity)
                if luminosity.color() == scene::Color::rgb(80, 28, 30)
        )
    }));
    assert!(rendered.scene().texts().iter().any(|text| {
        text.value().contains("tint = \"#501c1e\"")
            || text.value().contains("noise-opacity = 0.040")
    }));
    assert!(
        rendered
            .scene()
            .texts()
            .iter()
            .any(|text| { text.value().contains("luminosity-color = \"#501c1e\"") })
    );
    assert!(rendered.scene().texts().iter().any(|text| {
        text.value() == "Noise opacity: 0.04" && text.wrap() == scene::TextWrap::None
    }));

    let slider = rendered
        .layout()
        .find_role(view::Role::Slider)
        .into_iter()
        .next()
        .expect("glass tuner should lay out sliders");
    let slider_rect = slider.active_rect();
    let hit = app
        .hit_test(
            window,
            size,
            geometry::Point::new(
                slider_rect.x().saturating_add(slider_rect.width() / 2),
                slider_rect.y().saturating_add(slider_rect.height() / 2),
            ),
        )
        .expect("slider should be hit testable");
    assert_eq!(hit.frame().role(), view::Role::Slider);

    app.invoke(app.trigger::<glass_tuner::TogglePanel>(()))
        .output
        .expect("toggle panel should succeed");
    let hidden = app
        .show_scene(window, size)
        .expect("hidden glass tuner should render");
    assert!(
        hidden
            .layout()
            .find_role(view::Role::FloatingPanel)
            .is_empty(),
        "hidden glass tuner should remove the live panel immediately"
    );
    let expired = app
        .show_scene_at(
            window,
            size,
            std::time::Instant::now()
                + std::time::Duration::from_millis(Theme::default().overlay().exit_fade_ms + 1),
        )
        .expect("expired glass tuner ghost should render");
    assert!(expired.scene().panes().is_empty());
}

#[test]
fn glass_tuner_force_promoted_comparison_renders_group_at_rest() {
    let mut state = glass_tuner::State::default();
    state.comparison_open = false;
    let mut app = glass_tuner::app(state);

    app.start();

    let window = app.session().windows()[0].id();
    let size = glass_tuner::window_size();
    let initial = app
        .show_scene_after_overlay_fade(window, size)
        .expect("glass tuner should render");
    assert_eq!(top_level_group_count(initial.scene()), 0);

    app.invoke(app.trigger::<glass_tuner::ToggleComparison>(()))
        .output
        .expect("toggle comparison should succeed");
    app.invoke(app.trigger::<glass_tuner::ToggleForcePromoted>(()))
        .output
        .expect("toggle forced promotion should succeed");
    let forced = app
        .show_scene_after_overlay_fade(window, size)
        .expect("forced comparison should render");

    assert!(
        top_level_group_count(forced.scene()) >= 1,
        "forced full-opacity comparison overlay should stay on the group path"
    );
}

#[test]
fn glass_tuner_comparison_emits_half_alpha_popup_witness_quad() {
    let mut state = glass_tuner::State::default();
    state.comparison_open = false;
    let mut app = glass_tuner::app(state);

    app.start();

    let window = app.session().windows()[0].id();
    let size = glass_tuner::window_size();
    app.invoke(app.trigger::<glass_tuner::ToggleComparison>(()))
        .output
        .expect("toggle comparison should succeed");
    let rendered = app
        .show_scene_after_overlay_fade(window, size)
        .expect("comparison should render");

    assert!(
        rendered
            .scene()
            .quads()
            .iter()
            .any(|quad| quad.fill().channels() == (255, 0, 255, 128)),
        "alpha witness must be a real half-alpha primitive, not a clear color"
    );
}

fn top_level_group_count(scene: &Scene) -> usize {
    scene
        .primitives()
        .iter()
        .filter(|primitive| matches!(primitive, scene::Primitive::Group(_)))
        .count()
}

#[test]
fn command_hint_visually_wins_without_erasing_the_independent_description() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 660);
    let initial = app
        .show_scene_after_overlay_fade(window, size)
        .expect("control gallery should render");
    let view = app.present(window).expect("resolved gallery view");
    let binding = |name| {
        view.bindings()
            .into_iter()
            .find(|binding| binding.command_name() == name)
            .expect("gallery command binding")
    };
    let both = binding("control_gallery.increment_clicks");
    assert_eq!(
        both.hint(),
        Some("Adds one using the current gallery state")
    );
    assert_eq!(
        both.description(),
        Some("Increment the gallery click counter")
    );
    let hint_only = binding("control_gallery.toggle_wrap");
    assert!(hint_only.hint().is_some());
    assert_eq!(hint_only.description(), None);
    let description_only = binding("control_gallery.toggle_expanded_rows");
    assert_eq!(description_only.hint(), None);
    assert!(description_only.description().is_some());
    let neither = binding("control_gallery.select_mode");
    assert_eq!(neither.hint(), None);
    assert_eq!(neither.description(), None);
    let click = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.label_text() == Some("Click") && frame.target().is_some())
        .expect("described Click command should be present");
    let point = frame_point(click);

    app.pointer_move_at(window, size, point)
        .expect("hover should be handled");
    let before_delay = app
        .show_scene(window, size)
        .expect("hover scheduling frame should render");
    assert!(
        before_delay
            .layout()
            .frames()
            .iter()
            .all(|frame| frame.interaction_id() != Some(interaction::Id::new("feedback.hover"))),
        "hint panel must not appear before the dwell deadline"
    );
    let crate::animation::Schedule::At(deadline) = app.animation_schedule() else {
        panic!("hover dwell should schedule exactly one deadline");
    };

    app.invalidate_due_animation_frames(deadline);
    let visible = app
        .show_scene_after_overlay_fade(window, size)
        .expect("due hover hint should rebuild through the shared panel path");
    let panel = visible
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.interaction_id() == Some(interaction::Id::new("feedback.hover")))
        .expect("hint should project as a floating hover panel");
    assert_eq!(panel.role(), view::Role::FloatingPanel);
    assert!(panel.target().is_none(), "hover panels never accept input");
    assert_eq!(
        panel
            .popup_placement()
            .expect("hover panel placement request")
            .anchor(),
        geometry::placement::Anchor::Point(point),
        "hover revelation attaches to the pointer snapshot rather than the whole target rectangle"
    );
    let clearance = Theme::default().auxiliary_panel().pointer_clearance;
    assert_eq!(panel.rect().x(), point.x().saturating_add(clearance));
    assert_eq!(panel.rect().y(), point.y().saturating_add(clearance));
    assert!(
        visible.scene().texts().iter().any(|text| {
            text.value() == "Adds one using the current gallery state"
                && text.wrap() == scene::TextWrap::WordOrGlyph
        }),
        "scene text: {:?}",
        visible
            .scene()
            .texts()
            .iter()
            .map(|text| (text.value(), text.wrap()))
            .collect::<Vec<_>>()
    );
    assert!(visible.scene().icons().iter().any(|icon| {
        icon.icon().id().as_str() == "info" && rect_contains(panel.rect(), icon.rect())
    }));

    app.pointer_move_at(window, size, geometry::Point::new(1, 1))
        .expect("leaving the anchor should dismiss the hover panel");
    let dismissed = app
        .show_scene_after_overlay_fade(window, size)
        .expect("dismissal should rebuild through the shared panel path");
    assert!(
        dismissed
            .layout()
            .frames()
            .iter()
            .all(|frame| frame.interaction_id() != Some(interaction::Id::new("feedback.hover")))
    );
}

#[test]
fn confirmed_table_overflow_uses_a_glyphless_shared_hover_panel() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 700);
    let initial = app
        .show_scene_after_overlay_fade(window, size)
        .expect("compact gallery table should render");
    let overflowed = initial
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.table_cell().is_some()
                && frame.target().is_some()
                && frame.overflow_tip().is_some()
        })
        .expect("compact table should expose a confirmed overflow projection");
    let source = overflowed
        .overflow_tip()
        .expect("overflow source")
        .to_owned();
    let point = frame_point(overflowed);
    drop(initial);

    app.pointer_move_at(window, size, point)
        .expect("overflowed cell should hover");
    app.show_scene(window, size)
        .expect("hover dwell should schedule");
    let crate::animation::Schedule::At(deadline) = app.animation_schedule() else {
        panic!("overflow hover should schedule a dwell deadline");
    };
    app.invalidate_due_animation_frames(deadline);
    let visible = app
        .show_scene_after_overlay_fade(window, size)
        .expect("overflow revelation should enter the shared panel path");
    let panel = visible
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.interaction_id() == Some(interaction::Id::new("feedback.hover")))
        .expect("overflow should project one hover panel");

    assert_eq!(panel.auxiliary_hint().and_then(view::Hint::icon), None);
    assert_eq!(
        panel.auxiliary_hint().map(view::Hint::tone),
        Some(view::Tone::Neutral)
    );
    assert!(panel.target().is_none());
    assert!(
        visible
            .scene()
            .texts()
            .iter()
            .any(|text| { text.value() == source && text.wrap() == scene::TextWrap::WordOrGlyph })
    );
    assert!(visible.scene().icons().iter().all(|icon| {
        icon.icon().id().as_str() != "info" || !rect_contains(panel.rect(), icon.rect())
    }));
}

#[test]
fn warning_feedback_uses_shared_panel_without_trapping_focus() {
    let first = session::Focus::text("feedback.first");
    let second = session::Focus::text("feedback.second");
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            let window = cx.open_window(window::Options::new("Feedback warning"));
            assert!(cx.report_feedback(
                window,
                crate::feedback::Severity::Warning,
                "The connection is unstable"
            ));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("First").focus(first));
                ui.text_box(widget::TextBox::new("Second").focus(second));
            })
        });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(420, 180);
    let visible = app
        .show_scene_after_overlay_fade(window, size)
        .expect("window feedback should render");
    let panel = visible
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.interaction_id() == Some(interaction::Id::new("feedback.window")))
        .expect("warning should project through the shared panel path");
    assert_eq!(
        panel.auxiliary_hint().map(view::Hint::tone),
        Some(view::Tone::Warning)
    );
    assert_eq!(
        panel
            .popup_placement()
            .expect("window feedback must enter the common placement-request path")
            .anchor(),
        geometry::placement::Anchor::Point(geometry::Point::new(12, 12))
    );
    assert!(panel.target().is_none());
    assert!(visible.scene().icons().iter().any(|icon| {
        icon.icon().id().as_str() == "warning" && rect_contains(panel.rect(), icon.rect())
    }));

    assert!(app.focus(window, first));
    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("Tab should traverse beneath noninteractive feedback");
    assert!(
        app.session()
            .focused(window)
            .is_some_and(|focus| focus.same_target(&second)),
        "warning severity must neither trap focus nor suppress the underlying focus order"
    );
}

#[test]
fn gallery_runtime_dialogue_projects_warning_and_information_without_authored_panels() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 660);

    app.emit(control_gallery::runtime::Event::Report(
        crate::feedback::Severity::Warning,
    ));
    let warning = app
        .show_scene_after_overlay_fade(window, size)
        .expect("gallery warning should render");
    let warning_panel = warning
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.interaction_id() == Some(interaction::Id::new("feedback.window")))
        .expect("gallery warning panel");
    assert_eq!(
        warning_panel.auxiliary_hint().map(view::Hint::tone),
        Some(view::Tone::Warning)
    );
    assert!(warning.scene().icons().iter().any(|icon| {
        icon.icon().id().as_str() == "warning" && rect_contains(warning_panel.rect(), icon.rect())
    }));

    app.emit(control_gallery::runtime::Event::ClearFeedback);
    app.emit(control_gallery::runtime::Event::Report(
        crate::feedback::Severity::Info,
    ));
    let info = app
        .show_scene_after_overlay_fade(window, size)
        .expect("gallery information should render");
    let info_panel = info
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.interaction_id() == Some(interaction::Id::new("feedback.window")))
        .expect("gallery information panel");
    assert_eq!(
        info_panel.auxiliary_hint().map(view::Hint::tone),
        Some(view::Tone::Neutral)
    );
    assert_eq!(
        info_panel
            .auxiliary_hint()
            .and_then(view::Hint::icon)
            .map(|icon| icon.id().as_str()),
        Some("info")
    );
    assert!(info.scene().icons().iter().any(|icon| {
        icon.icon().id().as_str() == "info" && rect_contains(info_panel.rect(), icon.rect())
    }));
}

#[test]
fn auxiliary_content_wraps_and_caps_before_shared_placement_without_a_scroll_species() {
    let theme = Theme::from_toml_str(
        r#"
        [auxiliary-panel]
        max-width = 140
        max-height = 64
        "#,
    )
    .expect("bounded auxiliary theme");
    let long = "A deliberately long warning whose complete retained truth must wrap before placement and clip only at the themed residual-height boundary.";
    let mut app = Runtime::new(SourceState::default())
        .started(move |cx| {
            let window = cx.open_window(window::Options::new("Bounded feedback"));
            assert!(cx.report_feedback(window, crate::feedback::Severity::Warning, long));
        })
        .theme(move |_| theme.clone())
        .view(|_, _| View::new(view::Node::label("Content")));
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(180, 100);
    let visible = app
        .show_scene_after_overlay_fade(window, size)
        .expect("bounded feedback should render");
    let panel = visible
        .layout()
        .frames()
        .iter()
        .find(|frame| frame.interaction_id() == Some(interaction::Id::new("feedback.window")))
        .expect("bounded feedback panel");

    assert_eq!(panel.rect().width(), 140);
    assert_eq!(panel.rect().height(), 64);
    assert!(panel.popup_placement().is_some());
    assert!(panel.rect().x() >= 0 && panel.rect().right() <= size.width());
    assert!(panel.rect().y() >= 0 && panel.rect().bottom() <= size.height());
    assert!(
        visible.layout().find_role(view::Role::Scroll).is_empty(),
        "residual overflow is clipping policy, not a nested scrollable panel species"
    );
    assert!(
        visible
            .scene()
            .texts()
            .iter()
            .any(|text| { text.value() == long && text.wrap() == scene::TextWrap::WordOrGlyph })
    );
}

fn assert_tint_quad(scene: &Scene, rect: geometry::Rect, color: scene::Color) {
    assert!(
        scene.quads().iter().any(|quad| {
            quad.rect() == rect
                && quad.fill() == color
                && quad.rounding() == Theme::default().control().rounding
        }),
        "expected tint quad for rect {rect:?} and color {:?}",
        color.channels()
    );
}

fn assert_no_tint_quad(scene: &Scene, rect: geometry::Rect, color: scene::Color) {
    assert!(
        !scene
            .quads()
            .iter()
            .any(|quad| quad.rect() == rect && quad.fill() == color),
        "unexpected tint quad for rect {rect:?} and color {:?}",
        color.channels()
    );
}

fn assert_choice_pressed_tint_above_mark_chrome(
    presentation: &scene::Presentation,
    mark: geometry::Rect,
) {
    let primitives = presentation.scene().primitives();
    let mark_color = Theme::default().choice().mark;
    let indicator = Theme::default().choice().indicator;
    let pressed_tint = Theme::default().choice().pressed_tint;
    let mark_index = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad) if quad.rect() == mark && quad.fill() == mark_color
            )
        })
        .expect("choice mark base should be painted");
    let tint_index = primitives
        .iter()
        .position(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Quad(quad) if quad.rect() == mark && quad.fill() == pressed_tint
            )
        })
        .expect("choice pressed tint should be painted");

    assert!(
        mark_index < tint_index,
        "choice pressed tint should paint above the mark base"
    );

    let indicator_index = primitives.iter().position(|primitive| match primitive {
        scene::Primitive::Icon(icon) => icon.icon().id().as_str() == "check" && icon.rect() == mark,
        scene::Primitive::Quad(quad) => {
            quad.fill() == indicator && rect_contains(mark, quad.rect())
        }
        _ => false,
    });

    if let Some(indicator_index) = indicator_index {
        assert!(
            indicator_index < tint_index,
            "choice pressed tint should paint above the selected mark indicator"
        );
    }
}

fn assert_no_choice_label_overlay(presentation: &scene::Presentation, rect: geometry::Rect) {
    let pressed_tint = Theme::default().choice().pressed_tint;

    assert!(
        !presentation
            .scene()
            .quads()
            .iter()
            .any(|quad| quad.rect() == rect && quad.fill() == pressed_tint),
        "choice label region should not paint the pressed tint"
    );
}

fn assert_outline_is_scoped_by_clip(
    scene: &scene::Scene,
    outline_rect: geometry::Rect,
    clip_rect: geometry::Rect,
) {
    let primitives = scene.primitives();
    let outline_index = primitives
        .iter()
        .position(|primitive| {
            matches!(primitive, scene::Primitive::Outline(outline) if outline.rect() == outline_rect)
        })
        .expect("focused frame should retain a deferred outline");
    let clip_index = primitives[..outline_index]
        .iter()
        .rposition(|primitive| {
            matches!(primitive, scene::Primitive::Clip(clip) if clip.rect() == clip_rect)
        })
        .expect("the originating clip should precede the deferred outline");
    let pop_index = primitives[outline_index + 1..]
        .iter()
        .position(|primitive| matches!(primitive, scene::Primitive::PopClip))
        .map(|offset| outline_index + 1 + offset)
        .expect("the originating clip should close after the deferred outline");

    assert!(clip_index < outline_index);
    assert!(outline_index < pop_index);
}

fn assert_quad_is_scoped_by_active_clip(
    scene: &scene::Scene,
    fill: scene::Color,
    bounds: geometry::Rect,
    clip_rect: geometry::Rect,
) {
    let mut clips = Vec::new();
    for primitive in scene.primitives() {
        match primitive {
            scene::Primitive::Clip(clip) => clips.push(clip.rect()),
            scene::Primitive::PopClip => {
                clips.pop();
            }
            scene::Primitive::Quad(quad)
                if quad.fill() == fill && rect_contains(bounds, quad.rect()) =>
            {
                assert!(
                    clips.contains(&clip_rect),
                    "late chrome must repaint under its originating viewport clip"
                );
                return;
            }
            _ => {}
        }
    }
    panic!("expected clipped late-chrome quad");
}

fn frame_point_at(rect: geometry::Rect) -> geometry::Point {
    geometry::Point::new(
        rect.x().saturating_add(rect.width() / 2),
        rect.y().saturating_add(rect.height() / 2),
    )
}

fn rect_top_point(rect: geometry::Rect) -> geometry::Point {
    geometry::Point::new(
        rect.x().saturating_add(rect.width() / 2),
        rect.y().saturating_add(1),
    )
}

fn rect_bottom_point(rect: geometry::Rect) -> geometry::Point {
    geometry::Point::new(
        rect.x().saturating_add(rect.width() / 2),
        rect.bottom().saturating_sub(2),
    )
}

fn cursor_after_move<M: State, E: Send + 'static>(
    app: &mut Runtime<M, E, View>,
    window: window::Id,
    size: geometry::Size,
    point: geometry::Point,
) -> Option<pointer::Cursor> {
    app.pointer_move_at(window, size, point)
        .expect("pointer move should be handled");
    drain_cursor_updates(app, window, size)
        .last()
        .map(|update| update.cursor())
}

fn drain_cursor_updates<M: State, E: Send + 'static>(
    app: &mut Runtime<M, E, View>,
    window: window::Id,
    size: geometry::Size,
) -> Vec<pointer::Update> {
    app.drain_scenes(|id| {
        assert_eq!(id, window);
        size
    })
    .cursor_updates()
    .to_vec()
}

fn first_visible_text_area_surface_y(presentation: &scene::Presentation) -> f32 {
    visible_text_area_surfaces(presentation)
        .into_iter()
        .map(|(_, y, _)| y)
        .min_by(f32::total_cmp)
        .expect("text area should render at least one visible surface")
}

fn visible_text_area_surfaces(presentation: &scene::Presentation) -> Vec<(usize, f32, f32)> {
    presentation
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .flat_map(|frame| {
            let height = frame.rect().height() as f32;
            frame
                .text_area_layout()
                .into_iter()
                .flat_map(|text_area| text_area.render_surfaces())
                .filter(move |surface| surface.y() < height && surface.y() + surface.height() > 0.0)
        })
        .map(|surface| (surface.source_line(), surface.y(), surface.height()))
        .collect()
}

fn scroll_app() -> Runtime<SourceState, (), View> {
    Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Scroll"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.add(
                        widget::Scroll::new()
                            .id("scroll.test")
                            .height(view::Dimension::fixed(72))
                            .children(|ui| {
                                for index in 0..8 {
                                    ui.label(format!("Row {index}"));
                                }
                            }),
                    );
                });
            })
        })
}

fn command_palette_scroll_app() -> Runtime<SourceState, (), View> {
    Runtime::new(SourceState::default())
        .commands(|commands| {
            commands
                .register::<PaletteOne>(command::Spec::new("Palette One").shortcut("Primary+R"))
                .register::<PaletteTwo>(command::Spec::new("Palette Two"))
                .register::<PaletteThree>(command::Spec::new("Palette Three"))
                .register::<PaletteFour>(command::Spec::new("Palette Four"))
                .register::<PaletteFive>(command::Spec::new("Palette Five"))
                .register::<PaletteSix>(command::Spec::new("Palette Six"))
                .register::<PaletteSeven>(command::Spec::new("Palette Seven"))
                .register::<PaletteEight>(command::Spec::new("Palette Eight"))
                .register::<PaletteNine>(command::Spec::new("Palette Nine"))
                .register::<PaletteTen>(command::Spec::new("Palette Ten"))
                .register::<PaletteEleven>(command::Spec::new("Palette Eleven"))
                .register::<PaletteTwelve>(command::Spec::new("Palette Twelve"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<PaletteOne>()
                .target::<PaletteTwo>()
                .target::<PaletteThree>()
                .target::<PaletteFour>()
                .target::<PaletteFive>()
                .target::<PaletteSix>()
                .target::<PaletteSeven>()
                .target::<PaletteEight>()
                .target::<PaletteNine>()
                .target::<PaletteTen>()
                .target::<PaletteEleven>()
                .target::<PaletteTwelve>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Palette Scroll"));
        })
        .view(|_, _| View::new(view::Node::root()))
}

fn nested_clipped_scroll_app() -> Runtime<SourceState, (), View> {
    let focus = session::Focus::text("nested.search");
    Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Nested Scroll"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.text_box(widget::TextBox::new("").focus(focus));
                    ui.add(
                        widget::Scroll::new()
                            .id("scroll.outer")
                            .label("Outer Scroll")
                            .height(view::Dimension::fixed(64))
                            .children(|ui| {
                                ui.label("Outer row 0");
                                ui.label("Outer row 1");
                                ui.add(
                                    widget::Scroll::new()
                                        .id("scroll.inner")
                                        .label("Inner Scroll")
                                        .height(view::Dimension::fixed(54))
                                        .children(|ui| {
                                            for index in 0..8 {
                                                ui.label(format!("Inner row {index}"));
                                            }
                                        }),
                                );
                                for index in 2..10 {
                                    ui.label(format!("Outer row {index}"));
                                }
                            }),
                    );
                });
            })
        })
}

fn scroll_outer_until_inner_overlaps_search(
    app: &mut Runtime<SourceState, (), View>,
    window: window::Id,
    size: geometry::Size,
) {
    let initial = app
        .show_scene(window, size)
        .expect("nested scroll should render");
    let outer = scroll_frame_with_label(&initial, "Outer Scroll");
    app.scroll_at(
        window,
        size,
        frame_point_at(
            outer
                .viewport()
                .expect("outer scroll should expose viewport")
                .rect(),
        ),
        interaction::ScrollDelta::vertical(112),
    )
    .expect("outer scroll should be handled");
    // These tests inspect authored layout geometry rather than the production
    // projected hit-test path, so request a layout baseline at the new offset.
    app.request_redraw(window);
}

fn first_scroll_frame(presentation: &scene::Presentation) -> &layout::Frame {
    presentation
        .layout()
        .find_role(view::Role::Scroll)
        .into_iter()
        .next()
        .expect("scroll should be laid out")
}

fn scroll_frame_with_label<'a>(
    presentation: &'a scene::Presentation,
    label: &str,
) -> &'a layout::Frame {
    presentation
        .layout()
        .find_role(view::Role::Scroll)
        .into_iter()
        .find(|frame| frame.label_text() == Some(label))
        .expect("named scroll should be laid out")
}

fn command_palette_results_frame(presentation: &scene::Presentation) -> &layout::Frame {
    presentation
        .layout()
        .find_role(view::Role::Scroll)
        .into_iter()
        .find(|frame| {
            frame.target().and_then(interaction::Target::element_id)
                == Some(interaction::CommandPalette::results_id())
        })
        .expect("command palette results scroll should be laid out")
}

fn command_palette_panel_frame(presentation: &scene::Presentation) -> &layout::Frame {
    presentation
        .layout()
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .find(|frame| {
            frame.target().and_then(interaction::Target::element_id)
                == Some(interaction::CommandPalette::panel_id())
        })
        .expect("command palette panel should be laid out")
}

fn immediate_scroll_child_frames<'a>(
    layout: &'a layout::Layout,
    scroll: &layout::Frame,
) -> Vec<&'a layout::Frame> {
    let child_depth = scroll.path_depth() + 1;
    layout
        .frames()
        .iter()
        .filter(|frame| frame.path_depth() == child_depth && frame.is_descendant_of(scroll))
        .collect()
}

fn selected_palette_result_frame(presentation: &scene::Presentation) -> &layout::Frame {
    presentation
        .layout()
        .frames()
        .iter()
        .find(|frame| {
            frame.is_selected() && frame.binding_source() == Some(context::Source::Palette)
        })
        .expect("selected command palette result should be laid out")
}

fn first_scrollbar_track(layout: &layout::Layout) -> geometry::Rect {
    layout
        .chrome()
        .iter()
        .map(layout::Chrome::track)
        .next()
        .expect("scrollbar chrome should be projected")
}

fn scene_has_scrollbar_thumb(scene: &Scene, theme: &Theme, bounds: geometry::Rect) -> bool {
    let (thumb_r, thumb_g, thumb_b, _) = theme.scrollbar().appearance.thumb.channels();
    scene.quads().iter().any(|quad| {
        let (r, g, b, a) = quad.fill().channels();
        (r, g, b) == (thumb_r, thumb_g, thumb_b) && a > 0 && rect_contains(bounds, quad.rect())
    })
}

fn scene_text<'a>(scene: &'a Scene, value: &str) -> &'a scene::Text {
    scene
        .texts()
        .into_iter()
        .find(|text| text.value() == value)
        .expect("scene text should exist")
}

fn scene_text_in_rect<'a>(
    scene: &'a Scene,
    value: &str,
    bounds: geometry::Rect,
) -> &'a scene::Text {
    scene
        .texts()
        .into_iter()
        .find(|text| text.value() == value && rect_contains(bounds, text.rect()))
        .expect("scene text should exist inside bounds")
}

fn scene_pane_at(scene: &Scene, rect: geometry::Rect) -> Option<&scene::Pane> {
    scene.panes().into_iter().find(|pane| pane.rect() == rect)
}

fn scene_icon_in_rect<'a>(scene: &'a Scene, icon: &str, bounds: geometry::Rect) -> &'a scene::Icon {
    scene
        .icons()
        .into_iter()
        .find(|candidate| {
            candidate.icon().id().as_str() == icon && rect_contains(bounds, candidate.rect())
        })
        .expect("scene icon should exist inside bounds")
}

fn text_color_channels(color: scene::Color) -> (f32, f32, f32, f32) {
    let (r, g, b, a) = color.channels();

    (
        linear_channel(r),
        linear_channel(g),
        linear_channel(b),
        alpha_channel(a),
    )
}

fn text_color_channels_equal(actual: (f32, f32, f32, f32), expected: (f32, f32, f32, f32)) -> bool {
    (actual.0 - expected.0).abs() < f32::EPSILON
        && (actual.1 - expected.1).abs() < f32::EPSILON
        && (actual.2 - expected.2).abs() < f32::EPSILON
        && (actual.3 - expected.3).abs() < f32::EPSILON
}

fn linear_channel(channel: u8) -> f32 {
    let value = alpha_channel(channel);

    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn alpha_channel(channel: u8) -> f32 {
    channel as f32 / u8::MAX as f32
}
