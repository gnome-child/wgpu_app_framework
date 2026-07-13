use super::{
    Mode, State,
    command::{
        EditRecordCount, EditRecordCountArgs, EditRecordNote, EditRecordNoteArgs, IncrementClicks,
        OpenRecord, ResetControls, SelectMode, SetLevel, SetRecordEnabled, SetRecordEnabledArgs,
        SubmitQuery, ToggleAdvanced, ToggleExpandedRows, ToggleGrid, ToggleWrap,
    },
    state::{RECORD_COUNT, RecordOrder},
};
use wgpu_l3::{
    Table, View, geometry, interaction, scene, table, text,
    view::{Align, Context as ViewContext, Dimension, Padding},
    virtual_list, widget, window,
};

pub(super) const QUERY_FOCUS: interaction::Id = interaction::Id::new("control_gallery.query");

pub const WINDOW_TITLE: &str = "wgpu_l3 Control Gallery";
pub const CANVAS_COLOR: scene::Color = window::DEFAULT_CANVAS_COLOR;

pub fn window_size() -> geometry::Size {
    geometry::Size::new(760, 660)
}

#[derive(Clone)]
struct GalleryRecord {
    number: RecordNumber,
    detail: String,
    note: String,
    count: i64,
    enabled: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RecordNumber(usize);

impl std::fmt::Display for RecordNumber {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Record {}", self.0)
    }
}

fn record_at(index: usize, descending: bool, order: Option<&RecordOrder>) -> usize {
    if let Some(order) = order {
        order.row(index)
    } else if descending {
        RECORD_COUNT - index - 1
    } else {
        index
    }
}

pub fn view(state: &State, _: ViewContext) -> View {
    widget::view(|ui| {
        ui.column(|ui| {
            ui.standard_menu_bar();

            ui.add(
                widget::Element::new()
                    .column()
                    .layout(|layout| {
                        layout
                            .gap(10)
                            .padding(Padding::all(16))
                            .align_items(Align::Stretch)
                    })
                    .width(Dimension::grow())
                    .height(Dimension::grow())
                    .children(|ui| {
                        ui.label("Interactive Controls");
                        ui.add(summary_panel(state));
                        ui.add(toggle_panel(state));
                        ui.add(mode_panel(state));
                        ui.add(input_panel(state));
                        ui.label(format!("Status: {}", state.last_status));

                        if state.show_advanced {
                            ui.add(advanced_panel(state));
                        }

                        ui.label("One million provided records");
                        ui.checkbox(
                            widget::Checkbox::new("Expanded rows", state.expanded_rows)
                                .trigger::<ToggleExpandedRows>(()),
                        );
                        let descending = state.record_sort.column().as_str() == "record"
                            && state.record_sort.direction()
                                == table::SortDirection::Descending;
                        let key_order = state.record_order.clone();
                        let index_order = state.record_order.clone();
                        let record_order = state.record_order.clone();
                        let source = table::Source::new(
                            RECORD_COUNT,
                            move |index| {
                                virtual_list::Key::new(
                                    record_at(index, descending, key_order.as_ref()) as u64,
                                )
                            },
                            move |key| {
                                let record = key.value() as usize;
                                (record < RECORD_COUNT).then(|| {
                                    if let Some(order) = index_order.as_ref() {
                                        order
                                            .index_of(record)
                                            .expect("record order indexes every logical record")
                                    } else if descending {
                                        RECORD_COUNT - record - 1
                                    } else {
                                        record
                                    }
                                })
                            },
                            {
                                let notes = state.record_notes.clone();
                                let counts = state.record_counts.clone();
                                let enabled = state.record_enabled.clone();
                                move |index| {
                                    let record =
                                        record_at(index, descending, record_order.as_ref());
                                    let key = record as u64;
                                    GalleryRecord {
                                        number: RecordNumber(record),
                                        detail: format!(
                                            "Application-owned detail for record {record} with a deliberately long value"
                                        ),
                                        note: notes.get(&key).cloned().unwrap_or_default(),
                                        count: counts.get(&key).copied().unwrap_or(0),
                                        enabled: enabled
                                            .get(&key)
                                            .copied()
                                            .unwrap_or(record % 2 == 0),
                                    }
                                }
                            },
                        );
                        let columns: Vec<table::TypedColumn<GalleryRecord>> = vec![
                            table::Column::text(
                                "record",
                                "Record",
                                Dimension::fixed(110),
                                |record: &GalleryRecord| &record.number,
                            )
                            .build(),
                            table::Column::text(
                                "detail",
                                "Detail",
                                Dimension::weight(2),
                                |record: &GalleryRecord| &record.detail,
                            )
                            .overflow(text::Overflow::EllipsisMiddle)
                            .build(),
                            table::Column::text(
                                "note",
                                "Note",
                                Dimension::weight(1),
                                |record: &GalleryRecord| &record.note,
                            )
                            .validate(|value| {
                                (value.chars().count() <= 40)
                                    .then_some(())
                                    .ok_or_else(|| {
                                        "Note must be 40 characters or fewer".to_owned()
                                    })
                            })
                            .editable::<EditRecordNote>(|cell, value| EditRecordNoteArgs {
                                cell,
                                value,
                            })
                            .build(),
                            table::Column::text(
                                "count",
                                "Count",
                                Dimension::fixed(72),
                                |record: &GalleryRecord| &record.count,
                            )
                            .align(Align::End)
                            .input(text::Input::signed_integer())
                            .validate(|value| {
                                (0..=999).contains(value).then_some(()).ok_or_else(|| {
                                    "Count must be from 0 to 999".to_owned()
                                })
                            })
                            .editable::<EditRecordCount>(|cell, value| EditRecordCountArgs {
                                cell,
                                value,
                            })
                            .build(),
                            table::Column::boolean(
                                "enabled",
                                "Enabled",
                                Dimension::fixed(100),
                                |record: &GalleryRecord| &record.enabled,
                            )
                            .toggle::<SetRecordEnabled>(|cell, value| SetRecordEnabledArgs {
                                cell,
                                value,
                            })
                            .build(),
                            table::Column::custom(
                                "action",
                                "Action",
                                Dimension::fixed(72),
                                |_: &GalleryRecord, _| {
                                    wgpu_l3::Widget::into_node(
                                        widget::Button::new("Open")
                                            .trigger::<IncrementClicks>(()),
                                    )
                                },
                            ),
                        ];
                        ui.add(
                            Table::typed(
                                "control_gallery.records",
                                24,
                                columns,
                                source,
                            )
                            .sorted_by(
                                state.record_sort.column(),
                                state.record_sort.direction(),
                            )
                            .presentation(if state.expanded_rows {
                                table::Presentation::Expanded
                            } else {
                                table::Presentation::Compact
                            })
                            .context_rows::<OpenRecord>(|key| key)
                            .width(Dimension::grow())
                            .height(Dimension::fixed(500)),
                        );
                    }),
            );
        });
    })
}

fn summary_panel(state: &State) -> widget::Element {
    widget::Element::new()
        .row()
        .layout(|layout| {
            layout
                .gap(8)
                .padding(Padding::all(8))
                .align_items(Align::Center)
        })
        .height(Dimension::fixed(44))
        .children(|ui| {
            ui.button(widget::Button::new("Click").trigger::<IncrementClicks>(()));
            ui.button(widget::Button::new("Reset").trigger::<ResetControls>(()));
            ui.label(format!(
                "Clicks: {} | Mode: {} | Level: {:.0}",
                state.clicks,
                state.mode.label(),
                state.level
            ));
        })
}

fn toggle_panel(state: &State) -> widget::Element {
    widget::Element::new()
        .row()
        .layout(|layout| {
            layout
                .gap(8)
                .padding(Padding::all(8))
                .align_items(Align::Center)
        })
        .height(Dimension::fixed(44))
        .children(|ui| {
            ui.checkbox(widget::Checkbox::new("Wrap text", state.wrap).trigger::<ToggleWrap>(()));
            ui.checkbox(widget::Checkbox::new("Show grid", state.grid).trigger::<ToggleGrid>(()));
            ui.checkbox(
                widget::Checkbox::new("Advanced", state.show_advanced)
                    .trigger::<ToggleAdvanced>(()),
            );
        })
}

fn mode_panel(state: &State) -> widget::Element {
    widget::Element::new()
        .row()
        .layout(|layout| {
            layout
                .gap(8)
                .padding(Padding::all(8))
                .align_items(Align::Center)
        })
        .height(Dimension::fixed(44))
        .children(|ui| {
            for mode in [Mode::Design, Mode::Inspect, Mode::Preview] {
                ui.radio(
                    widget::Radio::new(mode.label(), state.mode == mode)
                        .trigger::<SelectMode>(mode),
                );
            }
        })
}

fn input_panel(state: &State) -> widget::Element {
    widget::Element::new()
        .row()
        .layout(|layout| {
            layout
                .gap(8)
                .padding(Padding::all(8))
                .align_items(Align::Center)
        })
        .height(Dimension::fixed(44))
        .children(|ui| {
            ui.label("Search");
            ui.text_box(
                widget::TextBox::new(state.query.clone())
                    .placeholder("Type to search")
                    .focus(wgpu_l3::session::Focus::text(QUERY_FOCUS))
                    .on_submit::<SubmitQuery>(),
            );
        })
}

fn advanced_panel(state: &State) -> widget::Element {
    widget::Element::new()
        .column()
        .layout(|layout| {
            layout
                .gap(8)
                .padding(Padding::all(8))
                .align_items(Align::Stretch)
        })
        .height(Dimension::fixed(96))
        .children(|ui| {
            ui.slider(
                widget::Slider::new("Level", state.level, 0.0..=100.0).on_change::<SetLevel>(),
            );
            ui.label("Drag the slider to exercise captured pointer input and coalesced history.");
        })
}
