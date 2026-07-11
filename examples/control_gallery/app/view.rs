use super::{
    Mode, State,
    command::{
        EditRecordCount, EditRecordCountArgs, EditRecordNote, EditRecordNoteArgs, IncrementClicks,
        ResetControls, SelectMode, SetLevel, SortRecords, SubmitQuery, ToggleAdvanced, ToggleGrid,
        ToggleWrap,
    },
};
use wgpu_l3::{
    Table, View, document, geometry, interaction, scene, table, text, timeline,
    view::{Align, Context as ViewContext, Dimension, Padding},
    virtual_list, widget, window,
};

const MENU_CONTROLS: interaction::Id = interaction::Id::new("control_gallery.menu.controls");
const MENU_EDIT: interaction::Id = interaction::Id::new("control_gallery.menu.edit");
const MENU_VIEW: interaction::Id = interaction::Id::new("control_gallery.menu.view");
const RECORD_COUNT: usize = 1_000_000;
pub(super) const QUERY_FOCUS: interaction::Id = interaction::Id::new("control_gallery.query");

pub const WINDOW_TITLE: &str = "wgpu_l3 Control Gallery";
pub const CANVAS_COLOR: scene::Color = window::DEFAULT_CANVAS_COLOR;

pub fn window_size() -> geometry::Size {
    geometry::Size::new(760, 660)
}

#[derive(Clone)]
struct GalleryRecords {
    descending: bool,
    notes: std::collections::HashMap<u64, String>,
    counts: std::collections::HashMap<u64, i64>,
}

impl table::Provider for GalleryRecords {
    fn len(&self) -> usize {
        RECORD_COUNT
    }

    fn key(&self, index: usize) -> virtual_list::Key {
        virtual_list::Key::new(self.record(index) as u64)
    }

    fn index_of(&self, key: virtual_list::Key) -> Option<usize> {
        let record = key.value() as usize;
        (record < self.len()).then(|| {
            if self.descending {
                self.len() - record - 1
            } else {
                record
            }
        })
    }

    fn cell(&self, row: usize, cell: table::Cell) -> wgpu_l3::view::Node {
        let record = self.record(row);
        match cell.column().as_str() {
            "record" => wgpu_l3::Widget::into_node(widget::Label::world(
                format!("Record {record}"),
                text::Overflow::EllipsisEnd,
            )),
            "detail" => wgpu_l3::Widget::into_node(widget::Label::world(
                format!(
                    "Application-owned detail for record {record} with a deliberately long value"
                ),
                text::Overflow::EllipsisMiddle,
            )),
            "note" => wgpu_l3::Widget::into_node(
                table::TextEditor::new(
                    cell,
                    self.notes
                        .get(&(record as u64))
                        .cloned()
                        .unwrap_or_default(),
                )
                .placeholder("Note")
                .validate(|value| {
                    (value.chars().count() <= 40)
                        .then_some(())
                        .ok_or_else(|| "Note must be 40 characters or fewer".to_owned())
                })
                .on_commit::<EditRecordNote>(|cell, value| EditRecordNoteArgs { cell, value }),
            ),
            "count" => wgpu_l3::Widget::into_node(
                table::NumberEditor::new(
                    cell,
                    self.counts.get(&(record as u64)).copied().unwrap_or(0),
                )
                .validate(|value| {
                    (0..=999)
                        .contains(&value)
                        .then_some(())
                        .ok_or_else(|| "Count must be from 0 to 999".to_owned())
                })
                .on_commit::<EditRecordCount>(|cell, value| EditRecordCountArgs { cell, value }),
            ),
            "enabled" => {
                wgpu_l3::Widget::into_node(widget::Checkbox::new("Enabled", record % 2 == 0))
            }
            "action" => wgpu_l3::Widget::into_node(
                widget::Button::new("Open").trigger::<IncrementClicks>(()),
            ),
            _ => unreachable!("gallery table declares every provider column"),
        }
    }
}

impl GalleryRecords {
    fn record(&self, index: usize) -> usize {
        if self.descending {
            RECORD_COUNT - index - 1
        } else {
            index
        }
    }
}

pub fn view(state: &State, _: ViewContext) -> View {
    widget::view(|ui| {
        ui.column(|ui| {
            ui.menu_bar(|ui| {
                ui.menu(MENU_CONTROLS, "Controls", |ui| {
                    ui.add(widget::Binding::<IncrementClicks>::menu());
                    ui.add(widget::Binding::<ResetControls>::menu());
                });
                ui.menu(MENU_EDIT, "Edit", |ui| {
                    ui.add(widget::Binding::<timeline::Undo>::menu());
                    ui.add(widget::Binding::<timeline::Redo>::menu());
                    ui.separator();
                    ui.add(widget::Binding::<document::Cut>::menu());
                    ui.add(widget::Binding::<document::Copy>::menu());
                    ui.add(widget::Binding::<document::Paste>::menu());
                    ui.add(widget::Binding::<document::Delete>::menu());
                    ui.separator();
                    ui.add(widget::Binding::<document::SelectAll>::menu());
                });
                ui.menu(MENU_VIEW, "View", |ui| {
                    ui.add(widget::Binding::<ToggleWrap>::menu());
                    ui.add(widget::Binding::<ToggleGrid>::menu());
                    ui.add(widget::Binding::<ToggleAdvanced>::menu());
                });
            });

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
                        ui.add(
                            Table::new(
                                "control_gallery.records",
                                24,
                                [
                                    table::Column::new("record", "Record", Dimension::fixed(110))
                                        .header(
                                            widget::Button::new("Record ↕")
                                                .trigger::<SortRecords>(()),
                                        ),
                                    table::Column::new("detail", "Detail", Dimension::weight(2)),
                                    table::Column::new("note", "Note", Dimension::weight(1)),
                                    table::Column::new("count", "Count", Dimension::fixed(72)),
                                    table::Column::new("enabled", "Enabled", Dimension::fixed(100)),
                                    table::Column::new("action", "Action", Dimension::fixed(72)),
                                ],
                                GalleryRecords {
                                    descending: state.records_descending,
                                    notes: state.record_notes.clone(),
                                    counts: state.record_counts.clone(),
                                },
                            )
                            .width(Dimension::grow())
                            .height(Dimension::fixed(136)),
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
