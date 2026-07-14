use super::*;
use std::time::{Duration, Instant};

struct CurrentTextCommit;

impl Command for CurrentTextCommit {
    type Args = String;
    type Output = ();

    const NAME: &'static str = "test.current_text_commit";
}

#[derive(Clone)]
struct CurrentTextState {
    text: String,
    invocations: usize,
}

impl State for CurrentTextState {}

impl Target<CurrentTextCommit> for CurrentTextState {
    fn state(&self, text: &String, _: &Context) -> command::State {
        if text == "valid" {
            command::State::enabled()
        } else {
            command::State::disabled()
        }
    }

    fn invoke(&mut self, text: String, _: &mut Context) -> Response<()> {
        self.text = text;
        self.invocations += 1;
        Response::changed(())
    }
}

#[test]
fn text_commit_state_uses_current_draft_arguments() {
    let focus = session::Focus::text("current-commit");
    let mut app = Runtime::new(CurrentTextState {
        text: "invalid".to_owned(),
        invocations: 0,
    })
    .commands(|commands| {
        commands
            .install(document::Editing::standard())
            .register::<CurrentTextCommit>(command::Spec::new("Current commit"));
    })
    .responders(|responders| {
        responders.app().target::<CurrentTextCommit>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Current text commit"));
    })
    .view(move |state, _| {
        widget::view(|ui| {
            ui.text_box(
                widget::TextBox::new(state.text.clone())
                    .focus(focus)
                    .on_commit::<CurrentTextCommit>(),
            );
        })
    });
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    app.show_scene(window, size)
        .expect("text box should render");

    app.handle_input(window, Input::focus(focus))
        .expect("text box should focus");
    app.handle_input(window, Input::shortcut("Ctrl+A"))
        .expect("base text should select");
    app.handle_input(window, Input::text_commit("valid"))
        .expect("current valid draft should type");
    app.handle_input(window, Input::focus(non_text_focus("blur")))
        .expect("current valid arguments should commit");
    assert_eq!(app.state().text, "valid");
    assert_eq!(app.state().invocations, 1);

    app.show_scene(window, size)
        .expect("committed base should rebuild");
    app.handle_input(window, Input::focus(focus))
        .expect("text box should refocus");
    app.handle_input(window, Input::shortcut("Ctrl+A"))
        .expect("valid base should select");
    app.handle_input(window, Input::text_commit("invalid"))
        .expect("current invalid draft should type");
    let rejected = app
        .handle_input(window, Input::focus(non_text_focus("second-blur")))
        .expect("disabled current arguments should reject without runtime failure");

    assert!(rejected.is_handled());
    assert_eq!(app.state().text, "valid");
    assert_eq!(app.state().invocations, 1);
    assert_eq!(text_draft(&app, window, focus).text(), "invalid");
    assert!(matches!(
        app.session().text_input_feedback(window, focus),
        Some((crate::feedback::Severity::Error, reason)) if !reason.is_empty()
    ));
    assert!(
        app.session()
            .focused(window)
            .is_some_and(|current| current.same_target(&focus)),
        "a rejected current command must keep the text task active"
    );
}

#[test]
fn text_box_on_commit_invokes_command_with_draft_text() {
    let focus = session::Focus::text("search");
    let mut app = Runtime::new(TextBoxSubmitState {
        submitted: "Mar".to_owned(),
        ..TextBoxSubmitState::default()
    })
    .commands(|commands| {
        commands.register::<SubmitText>(command::Spec::new("Submit Text"));
    })
    .responders(|responders| {
        responders.app().target::<SubmitText>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Text Box"));
    })
    .view(move |state, _| {
        widget::view(|ui| {
            ui.text_box(
                widget::TextBox::new(state.submitted.clone())
                    .placeholder("Search")
                    .focus(focus)
                    .on_commit::<SubmitText>(),
            );
        })
    });

    app.start();

    let window = app.session().windows()[0].id();
    app.show_scene(window, geometry::Size::new(240, 80))
        .expect("text box view should render");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");

    let edited = app
        .handle_input(window, Input::text_commit("s"))
        .expect("text box edit should be handled");

    assert!(edited.is_handled());
    assert!(!edited.changed_state());
    assert_eq!(text_draft(&app, window, focus).text(), "Mars");
    assert_eq!(app.state().submitted, "Mar");
    assert_eq!(app.revision(), state::Revision::initial());

    let submitted = app
        .handle_input(window, Input::focus(non_text_focus("blur")))
        .expect("blur should commit the text box draft");

    assert!(submitted.is_handled());
    assert!(submitted.changed_state());
    assert_eq!(app.state().submitted, "Mars");
    assert_eq!(app.state().source, Some(context::Source::Input));
    assert_eq!(app.revision().get(), 1);
    assert_eq!(text_draft(&app, window, focus).text(), "Mars");
}

#[test]
fn bound_text_box_scene_paints_text_box_text_not_command_label() {
    let focus = session::Focus::text("search");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .commands(|commands| {
            commands.register::<SubmitText>(command::Spec::new("Submit Text"));
        })
        .responders(|responders| {
            responders.app().target::<SubmitText>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Scene"));
        })
        .view(move |state, _| {
            widget::view(|ui| {
                ui.text_box(
                    widget::TextBox::new(state.submitted.clone())
                        .placeholder("Search")
                        .focus(focus)
                        .on_commit::<SubmitText>(),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let initial = app
        .show_scene(window, geometry::Size::new(240, 80))
        .expect("text box view should render");

    assert!(scene_contains_text(initial.scene(), "Search"));
    assert!(!scene_contains_text(initial.scene(), "Submit Text"));

    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");
    app.handle_input(window, Input::text_commit("q"))
        .expect("text box commit should be handled");

    let typed = app
        .show_scene(window, geometry::Size::new(240, 80))
        .expect("updated text box view should render");

    assert!(scene_contains_text_surface(typed.scene(), "q"));
    assert!(!scene_contains_text(typed.scene(), "Submit Text"));
}

#[test]
fn bound_text_box_pointer_target_stays_text_input_target() {
    let focus = session::Focus::text("search");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .commands(|commands| {
            commands.register::<SubmitText>(command::Spec::new("Submit Text"));
        })
        .responders(|responders| {
            responders.app().target::<SubmitText>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Bound Text Box Target"));
        })
        .view(move |state, _| {
            widget::view(|ui| {
                ui.text_box(
                    widget::TextBox::new(state.submitted.clone())
                        .focus(focus)
                        .on_commit::<SubmitText>(),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let rendered = app
        .show_scene(window, geometry::Size::new(240, 80))
        .expect("text box view should render");
    let text_box = rendered
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let target = text_box
        .target()
        .expect("text box should have an interaction target");

    assert_eq!(target, &interaction::Target::text_area(focus));
    assert!(target.captures());
}

#[test]
fn text_box_commit_with_maps_committed_text_into_custom_command_args() {
    let focus = session::Focus::text("filter");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .commands(|commands| {
            commands.register::<SubmitMappedText>(command::Spec::new("Submit Mapped Text"));
        })
        .responders(|responders| {
            responders.app().target::<SubmitMappedText>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Mapped Text Box"));
        })
        .view(move |state, _| {
            widget::view(|ui| {
                ui.text_box(
                    widget::TextBox::new(state.submitted.clone())
                        .focus(focus)
                        .commit_with::<SubmitMappedText>(|text| TextSubmitArgs {
                            normalized: text.trim().to_ascii_lowercase(),
                            raw: text,
                        }),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.show_scene(window, geometry::Size::new(240, 80))
        .expect("mapped text box view should render");
    app.handle_input(window, Input::focus(focus))
        .expect("mapped text box focus should be handled");

    let edited = app
        .handle_input(window, Input::text_commit("  Rust  "))
        .expect("mapped text box edit should be handled");

    assert!(edited.is_handled());
    assert!(!edited.changed_state());
    assert_eq!(text_draft(&app, window, focus).text(), "  Rust  ");
    assert_eq!(app.state().submitted, "");
    assert_eq!(app.revision(), state::Revision::initial());

    let submitted = app
        .handle_input(window, Input::focus(non_text_focus("blur")))
        .expect("blur should commit the mapped text box draft");

    assert!(submitted.is_handled());
    assert!(submitted.changed_state());
    assert_eq!(app.state().submitted, "  Rust  ");
    assert_eq!(app.state().normalized, "rust");
    assert_eq!(app.state().source, Some(context::Source::Input));
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn unbound_text_box_commit_updates_framework_owned_draft() {
    let focus = session::Focus::text("plain-text-box");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Plain Text Box"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.show_scene(window, geometry::Size::new(240, 80))
        .expect("plain text box view should render");
    app.handle_input(window, Input::focus(focus))
        .expect("plain text box focus should be handled");

    let submitted = app
        .handle_input(window, Input::text_commit("ignored"))
        .expect("plain text box commit should not error");

    assert!(submitted.is_handled());
    assert!(!submitted.changed_state());
    assert!(submitted.effect().contains_invalidation());
    assert_eq!(app.state().submitted, "");
    assert_eq!(app.revision(), state::Revision::initial());

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box input should own a draft");

    assert_eq!(draft.text(), "ignored");
    assert_eq!(draft.cursor(), "ignored".len());

    let projected = app
        .present(window)
        .expect("text box draft should project into the view");
    let text_box = text_box_node(projected.root()).expect("text box should be in the view");
    let text_box = text_box
        .text_box_model()
        .expect("node should contain text box state");

    assert_eq!(text_box.text(), "ignored");
    assert_eq!(text_box.cursor(), Some("ignored".len()));
}

#[test]
fn character_key_updates_focused_text_box_draft_and_focus_outline() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Key Input"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before key input");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");

    let typed = app
        .handle_input(
            window,
            Input::key_down(input::Key::Character('q'), input::Modifiers::default()),
        )
        .expect("character key should update text box draft");

    assert!(typed.is_handled());
    assert!(!typed.changed_state());
    assert!(typed.effect().contains_invalidation());
    assert_eq!(app.revision(), state::Revision::initial());

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box should retain a draft");

    assert_eq!(draft.text(), "q");
    assert_eq!(draft.cursor(), 1);

    let rendered = app
        .show_scene(window, geometry::Size::new(240, 80))
        .expect("focused text box should render");
    let text_box = rendered
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");

    assert!(text_box.is_focused());
    assert!(
        rendered
            .scene()
            .outlines()
            .iter()
            .any(|outline| outline.rect() == text_box.rect()
                && outline.color().channels() == (10, 132, 255, 255))
    );
}

#[test]
fn tab_key_moves_focus_through_current_view_order() {
    let first = session::Focus::text("first");
    let document = session::Focus::text("document");
    let second = session::Focus::text("second");
    let mut app = Runtime::new(text_editor::State {
        document: TextDocument::from_multiline_text("body"),
        ..text_editor::State::default()
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Tab Navigation"));
    })
    .view(move |state, _| {
        widget::view(|ui| {
            ui.column(|ui| {
                ui.text_box(widget::TextBox::new("first").focus(first));
                ui.text_area(widget::TextArea::from_document(&state.document).focus(document));
                ui.text_box(widget::TextBox::new("second").focus(second));
            });
        })
    });

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before key navigation");

    let focused_first = app
        .handle_input(
            window,
            Input::key_down(input::Key::Tab, input::Modifiers::default()),
        )
        .expect("tab should navigate to first focusable node");

    assert!(focused_first.is_handled());
    assert!(!focused_first.changed_state());
    assert!(focused_first.effect().contains_invalidation());
    assert_keyboard_focus(app.session().focused(window), first);

    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab should navigate to the document");

    assert_keyboard_focus(app.session().focused(window), document);

    let document_text = app.state().document.text();
    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab from the document should navigate instead of editing text");

    assert_keyboard_focus(app.session().focused(window), second);
    assert_eq!(app.state().document.text(), document_text);

    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab should wrap to first focusable node");

    assert_keyboard_focus(app.session().focused(window), first);

    app.handle_input(
        window,
        Input::key_down(
            input::Key::Tab,
            input::Modifiers::new(true, false, false, false),
        ),
    )
    .expect("shift-tab should wrap backward");

    assert_keyboard_focus(app.session().focused(window), second);

    app.handle_input(
        window,
        Input::key_down(
            input::Key::Tab,
            input::Modifiers::new(true, false, false, false),
        ),
    )
    .expect("shift-tab should navigate backward through focus order");

    assert_keyboard_focus(app.session().focused(window), document);
}

#[test]
fn text_box_drafts_keep_independent_history_across_focus_changes() {
    let first = session::Focus::text("first");
    let second = session::Focus::text("second");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Drafts"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.text_box(widget::TextBox::new("").focus(first));
                    ui.text_box(widget::TextBox::new("").focus(second));
                });
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before text input");

    app.handle_input(window, Input::focus(first))
        .expect("first field should focus");
    app.handle_input(window, Input::text_commit("ab"))
        .expect("first field should accept text");
    app.handle_input(window, Input::focus(second))
        .expect("second field should focus");
    app.handle_input(window, Input::text_commit("xy"))
        .expect("second field should accept text");

    let first_target = interaction::Target::text_area(first);
    let second_target = interaction::Target::text_area(second);
    let input = app
        .session()
        .interaction(window)
        .expect("window should keep interaction state")
        .text_input();
    assert_eq!(
        input
            .draft_for(&first_target)
            .expect("first draft should survive blur")
            .text(),
        "ab"
    );
    assert_eq!(
        input
            .draft_for(&second_target)
            .expect("second draft should be active")
            .text(),
        "xy"
    );

    app.handle_input(window, Input::focus(first))
        .expect("first field should refocus");
    let first_undo = app
        .handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("first field undo should be local");
    assert!(first_undo.is_handled());
    assert!(!first_undo.changed_state());

    let input = app
        .session()
        .interaction(window)
        .expect("window should keep interaction state")
        .text_input();
    assert_eq!(
        input
            .draft_for(&first_target)
            .expect("first draft should remain")
            .text(),
        ""
    );
    assert_eq!(
        input
            .draft_for(&second_target)
            .expect("second draft should be untouched")
            .text(),
        "xy"
    );

    app.handle_input(window, Input::focus(second))
        .expect("second field should refocus");
    app.handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("second field undo should be local");

    let input = app
        .session()
        .interaction(window)
        .expect("window should keep interaction state")
        .text_input();
    assert_eq!(
        input
            .draft_for(&second_target)
            .expect("second draft should remain")
            .text(),
        ""
    );
    assert_eq!(app.revision(), state::Revision::initial());
    assert!(!app.is_dirty());
}

#[test]
fn runtime_retention_bounds_inactive_text_box_drafts() {
    let first = session::Focus::text("first");
    let second = session::Focus::text("second");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .retention(runtime::Retention::new().drafts(1))
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Draft Retention"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.text_box(widget::TextBox::new("").focus(first));
                    ui.text_box(widget::TextBox::new("").focus(second));
                });
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before text input");

    app.handle_input(window, Input::focus(first))
        .expect("first field should focus");
    app.handle_input(window, Input::text_commit("ab"))
        .expect("first field should accept text");
    app.handle_input(window, Input::focus(second))
        .expect("second field should focus");
    app.handle_input(window, Input::text_commit("xy"))
        .expect("second field should accept text");

    let first_target = interaction::Target::text_area(first);
    let second_target = interaction::Target::text_area(second);
    let input = app
        .session()
        .interaction(window)
        .expect("window should keep interaction state")
        .text_input();

    assert!(
        input.draft_for(&first_target).is_none(),
        "inactive first draft should be pruned by runtime retention"
    );
    assert_eq!(
        input
            .draft_for(&second_target)
            .expect("active second draft should be retained")
            .text(),
        "xy"
    );
}

fn assert_keyboard_focus(actual: Option<session::Focus>, expected: session::Focus) {
    let actual = actual.expect("window should have focus");
    assert!(actual.same_target(&expected));
    assert_eq!(actual.reason(), session::Reason::Keyboard);
    assert_eq!(actual.visibility(), session::Visibility::Visible);
}

#[test]
fn text_box_cursor_moves_before_next_text_commit() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Cursor"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("ab").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.show_scene(window, geometry::Size::new(240, 80))
        .expect("text box view should render");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");
    app.handle_input(window, Input::text_commit("c"))
        .expect("text box commit should be handled");
    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowLeft, input::Modifiers::default()),
    )
    .expect("left arrow should be handled by the text box");
    app.handle_input(window, Input::text_commit("X"))
        .expect("second text box commit should be handled");

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box input should retain a draft");

    assert_eq!(draft.text(), "abXc");
    assert_eq!(draft.cursor(), "abX".len());
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn text_box_ctrl_a_then_cut_updates_bound_text_and_clipboard() {
    let focus = session::Focus::text("search");
    let mut app = Runtime::new(TextBoxSubmitState {
        submitted: "alpha".to_owned(),
        ..TextBoxSubmitState::default()
    })
    .commands(|commands| {
        commands
            .install(document::Editing::standard())
            .register::<SubmitText>(command::Spec::new("Submit Text"));
    })
    .responders(|responders| {
        responders.app().target::<SubmitText>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Text Box Cut"));
    })
    .view(move |state, _| {
        widget::view(|ui| {
            ui.text_box(
                widget::TextBox::new(state.submitted.clone())
                    .focus(focus)
                    .on_commit::<SubmitText>(),
            );
        })
    });

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before shortcut input");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");

    let selected = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('a'),
                input::Modifiers::new(false, true, false, false),
            ),
        )
        .expect("ctrl-a should be handled by focused text box");

    assert!(selected.is_handled());
    assert!(!selected.changed_state());

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("select all should create a text box draft");

    assert_eq!(draft.selection(), Some(0.."alpha".len()));

    let cut = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('x'),
                input::Modifiers::new(false, true, false, false),
            ),
        )
        .expect("ctrl-x should cut the focused text box selection");

    assert!(cut.is_handled());
    assert!(!cut.changed_state());
    assert_eq!(
        app.clipboard()
            .text()
            .expect("clipboard read should succeed")
            .as_deref(),
        Some("alpha")
    );
    assert_eq!(app.state().submitted, "alpha");
    assert_eq!(app.revision(), state::Revision::initial());

    let draft = app
        .session()
        .interaction(window)
        .expect("window should keep interaction state")
        .text_input()
        .draft_for(&target)
        .expect("cut should keep the text box draft");

    assert_eq!(draft.text(), "");
    assert_eq!(draft.cursor(), 0);
    assert_eq!(draft.selection(), None);

    let committed = app
        .handle_input(window, Input::focus(non_text_focus("blur")))
        .expect("blur should commit the cut draft");

    assert!(committed.is_handled());
    assert!(committed.changed_state());
    assert_eq!(app.state().submitted, "");
    assert_eq!(app.revision().get(), 1);
    assert_eq!(text_draft(&app, window, focus).text(), "");
}

#[test]
fn text_box_cut_waits_for_confirmed_clipboard_write() {
    let focus = session::Focus::text("search");
    let mut app = Runtime::new(TextBoxSubmitState {
        submitted: "alpha".to_owned(),
        ..TextBoxSubmitState::default()
    })
    .with_clipboard(Clipboard::unavailable_system())
    .commands(|commands| {
        commands.install(document::Editing::standard());
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Failed Text Box Cut"));
    })
    .view(move |state, _| {
        widget::view(|ui| {
            ui.text_box(widget::TextBox::new(state.submitted.clone()).focus(focus));
        })
    });

    app.start();
    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before command input");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");
    app.invoke_focused(window, app.trigger::<document::SelectAll>(()))
        .output
        .expect("select all should succeed");

    let cut = app
        .invoke_focused(window, app.trigger::<document::Cut>(()))
        .output
        .expect("cut should report its clipboard outcome");

    assert!(cut.unavailable());
    assert!(!cut.clipboard_changed());
    assert!(!cut.buffer_changed());
    assert_eq!(text_draft(&app, window, focus).text(), "alpha");

    let paste = app
        .invoke_focused(window, app.trigger::<document::Paste>(()))
        .output
        .expect("paste failure should remain an outcome");
    assert!(paste.unavailable());
    assert_eq!(text_draft(&app, window, focus).text(), "alpha");
}

#[test]
fn text_box_paste_replaces_selection_and_truncates_to_first_line() {
    let focus = session::Focus::text("search");
    let mut app = Runtime::new(TextBoxSubmitState {
        submitted: "alpha".to_owned(),
        ..TextBoxSubmitState::default()
    })
    .commands(|commands| {
        commands
            .install(document::Editing::standard())
            .register::<SubmitText>(command::Spec::new("Submit Text"));
    })
    .responders(|responders| {
        responders.app().target::<SubmitText>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Text Box Paste"));
    })
    .view(move |state, _| {
        widget::view(|ui| {
            ui.text_box(
                widget::TextBox::new(state.submitted.clone())
                    .focus(focus)
                    .on_commit::<SubmitText>(),
            );
        })
    });

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before shortcut input");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");
    app.handle_input(
        window,
        Input::key_down(
            input::Key::Character('a'),
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("ctrl-a should select the focused text box");

    app.clipboard()
        .put(&clipboard::Text::new("beta\ngamma"))
        .expect("clipboard write should succeed");

    let pasted = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('v'),
                input::Modifiers::new(false, true, false, false),
            ),
        )
        .expect("ctrl-v should paste into the focused text box");

    assert!(pasted.is_handled());
    assert!(!pasted.changed_state());
    assert_eq!(app.state().submitted, "alpha");
    assert_eq!(app.revision(), state::Revision::initial());

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should keep interaction state")
        .text_input()
        .draft_for(&target)
        .expect("paste should keep the text box draft");

    assert_eq!(draft.text(), "beta");
    assert_eq!(draft.cursor(), "beta".len());
    assert_eq!(draft.selection(), None);

    let committed = app
        .handle_input(window, Input::focus(non_text_focus("blur")))
        .expect("blur should commit the pasted draft");

    assert!(committed.is_handled());
    assert!(committed.changed_state());
    assert_eq!(app.state().submitted, "beta");
    assert_eq!(app.revision().get(), 1);
    assert_eq!(text_draft(&app, window, focus).text(), "beta");
}

#[test]
fn numeric_text_box_normalizes_paste_and_keeps_undo_redo_on_the_final_draft() {
    let focus = session::Focus::text("unsigned-number");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .commands(|commands| {
            commands.install(document::Editing::standard());
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Numeric Text Box Paste"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(
                    widget::TextBox::new("7")
                        .input(text::Input::unsigned_integer())
                        .focus(focus),
                );
            })
        });
    app.start();
    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("numeric text box should present");
    app.handle_input(window, Input::focus(focus))
        .expect("numeric text box should focus");
    app.handle_input(
        window,
        Input::key_down(
            input::Key::Character('a'),
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("ctrl-a should select the numeric draft");
    app.clipboard()
        .put(&clipboard::Text::new(" 42 "))
        .expect("clipboard write should succeed");
    app.handle_input(
        window,
        Input::key_down(
            input::Key::Character('v'),
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("paste should run through the whole-draft policy");
    assert_eq!(text_draft(&app, window, focus).text(), "42");

    app.handle_input(
        window,
        Input::key_down(
            input::Key::Character('z'),
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("undo should restore the pre-paste draft");
    assert_eq!(text_draft(&app, window, focus).text(), "7");
    app.handle_input(
        window,
        Input::key_down(
            input::Key::Character('y'),
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("redo should restore the normalized draft");
    assert_eq!(text_draft(&app, window, focus).text(), "42");

    app.handle_input(
        window,
        Input::key_down(
            input::Key::Character('a'),
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("ctrl-a should select the normalized draft");
    app.clipboard()
        .put(&clipboard::Text::new("-"))
        .expect("clipboard write should succeed");
    app.handle_input(
        window,
        Input::key_down(
            input::Key::Character('v'),
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("rejected paste is still handled by the focused text box");
    assert_eq!(text_draft(&app, window, focus).text(), "42");
}

#[test]
fn numeric_text_box_never_filters_preedit_and_evaluates_ime_commit_once() {
    let focus = session::Focus::text("signed-number");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Numeric Text Box IME"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(
                    widget::TextBox::new("")
                        .input(text::Input::signed_integer())
                        .focus(focus),
                );
            })
        });
    app.start();
    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("numeric text box should present");
    app.handle_input(window, Input::focus(focus))
        .expect("numeric text box should focus");

    app.handle_input(
        window,
        Input::text_preedit(text::view::Preedit::new("composition", None)),
    )
    .expect("preedit should remain uninterrupted by numeric policy");
    let target = interaction::Target::text_area(focus);
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.text_input().preedit_for(&target))
            .map(text::view::Preedit::text),
        Some("composition")
    );

    app.handle_input(window, Input::text_commit(" -42 "))
        .expect("committed IME text should evaluate as one proposed draft");
    assert_eq!(text_draft(&app, window, focus).text(), "-42");
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.text_input().preedit_for(&target))
            .is_none()
    );
}

#[test]
fn text_box_undo_redo_uses_focused_draft_history() {
    let focus = session::Focus::text("search");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .commands(|commands| {
            commands.register::<SubmitText>(command::Spec::new("Submit Text"));
        })
        .responders(|responders| {
            responders.app().target::<SubmitText>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Undo"));
        })
        .view(move |state, _| {
            widget::view(|ui| {
                ui.text_box(
                    widget::TextBox::new(state.submitted.clone())
                        .focus(focus)
                        .on_commit::<SubmitText>(),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before shortcut input");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");

    let empty_undo = app
        .handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("focused text box undo should be a local no-op when history is empty");
    assert!(empty_undo.is_handled());
    assert!(!empty_undo.changed_state());

    for character in "abc".chars() {
        app.handle_input(window, Input::text_commit(character.to_string()))
            .expect("text commit should edit focused text box");
    }
    let target = interaction::Target::text_area(focus);
    assert_eq!(text_draft(&app, window, focus).text(), "abc");
    assert_eq!(app.state().submitted, "");

    app.handle_input(
        window,
        Input::key_down(input::Key::Backspace, input::Modifiers::default()),
    )
    .expect("backspace should edit focused text box");
    assert_eq!(text_draft(&app, window, focus).text(), "ab");
    assert_eq!(app.state().submitted, "");
    assert_eq!(app.timeline().undo_depth(), 0);

    let undo_delete = app
        .handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("text box undo should restore deleted character");
    assert!(undo_delete.is_handled());
    assert!(!undo_delete.changed_state());
    assert_eq!(text_draft(&app, window, focus).text(), "abc");
    assert_eq!(app.state().submitted, "");
    assert_eq!(app.timeline().undo_depth(), 0);
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.text_input().draft_for(&target))
            .is_some_and(|draft| draft.can_undo()),
        "undoing the delete should leave the older typing entry available"
    );

    let redo_delete = app
        .handle_input(window, Input::shortcut("Ctrl+Shift+Z"))
        .expect("text box redo should reapply deleted character");
    assert!(redo_delete.is_handled());
    assert!(!redo_delete.changed_state());
    assert_eq!(text_draft(&app, window, focus).text(), "ab");

    app.handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("text box undo should restore deleted character again");
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.text_input().draft_for(&target))
            .is_some_and(|draft| draft.can_undo()),
        "undoing the delete again should still leave the older typing entry available"
    );
    app.handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("coalesced typing should undo as one text box entry");
    assert_eq!(text_draft(&app, window, focus).text(), "");
    assert_eq!(app.state().submitted, "");
    assert_eq!(app.timeline().undo_depth(), 0);
}

#[test]
fn text_box_native_key_typing_undoes_as_one_text_chunk() {
    let focus = session::Focus::text("search");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .commands(|commands| {
            commands.register::<SubmitText>(command::Spec::new("Submit Text"));
        })
        .responders(|responders| {
            responders.app().target::<SubmitText>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Native Typing"));
        })
        .view(move |state, _| {
            widget::view(|ui| {
                ui.text_box(
                    widget::TextBox::new(state.submitted.clone())
                        .focus(focus)
                        .on_commit::<SubmitText>(),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before key input");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");

    for character in "abc".chars() {
        app.handle_input(
            window,
            Input::key_down_with_text(
                input::Key::Character(character),
                input::Modifiers::default(),
                Some(character.to_string()),
            ),
        )
        .expect("native-style text key should edit focused text box");
    }

    assert_eq!(text_draft(&app, window, focus).text(), "abc");
    assert_eq!(app.state().submitted, "");
    assert_eq!(app.timeline().undo_depth(), 0);

    let undo = app
        .handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("text box undo should resolve");

    assert!(undo.is_handled());
    assert!(!undo.changed_state());
    assert_eq!(
        text_draft(&app, window, focus).text(),
        "",
        "typing a word through native key events should undo as one chunk"
    );
    assert_eq!(app.state().submitted, "");

    let redo = app
        .handle_input(window, Input::shortcut("Ctrl+Shift+Z"))
        .expect("text box redo should resolve");

    assert!(redo.is_handled());
    assert!(!redo.changed_state());
    assert_eq!(text_draft(&app, window, focus).text(), "abc");
    assert_eq!(app.state().submitted, "");
}

#[test]
fn text_box_shift_arrow_selection_is_replaced_by_typing() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Selection"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("abcd").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before key input");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");
    app.handle_input(
        window,
        Input::key_down(
            input::Key::ArrowLeft,
            input::Modifiers::new(true, false, false, false),
        ),
    )
    .expect("shift-left should extend the text box selection");

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("shift-left should create a text box draft");

    assert_eq!(draft.selection(), Some(3..4));

    app.handle_input(window, Input::text_commit("Z"))
        .expect("typing should replace the text box selection");

    let draft = app
        .session()
        .interaction(window)
        .expect("window should keep interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box draft should remain active");

    assert_eq!(draft.text(), "abcZ");
    assert_eq!(draft.cursor(), 4);
    assert_eq!(draft.selection(), None);
}

#[test]
fn text_box_selection_and_caret_are_painted_as_widget_chrome() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .commands(|commands| {
            commands.install(document::Editing::standard());
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Paint"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("abcd").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.show_scene(window, geometry::Size::new(240, 80))
        .expect("initial render should install a composition");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");

    let caret_scene = app
        .show_scene(window, geometry::Size::new(240, 80))
        .expect("focused text box should render a caret");
    let text_box = caret_scene
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");

    let field = text_box
        .text_box_layout()
        .expect("focused text box should have text layout");
    let caret = field
        .layout()
        .caret()
        .expect("visible blink phase should include a caret");
    let text_rect = text_box.text_box_text_rect();
    let expected = geometry::Rect::new(
        text_rect.x().saturating_add(caret.x().floor() as i32),
        text_rect.y().saturating_add(caret.y().floor() as i32),
        1,
        caret.height().ceil().max(0.0) as i32,
    );
    let caret_rule = caret_scene
        .scene()
        .rules()
        .into_iter()
        .find(|rule| rule.rect() == expected)
        .expect("focused text box should paint the shaped field caret as a rule");

    assert_eq!(caret_rule.axis(), scene::Axis::Vertical);
    assert_eq!(caret_rule.thickness_px(), 2);

    app.handle_input(
        window,
        Input::key_down(
            input::Key::Character('a'),
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("ctrl-a should select text");

    let selected_scene = app
        .show_scene(window, geometry::Size::new(240, 80))
        .expect("selected text box should render selection");

    assert!(
        selected_scene
            .scene()
            .quads()
            .iter()
            .any(|quad| quad.fill().channels() == (10, 132, 255, 96)),
        "selected text box should paint a selection highlight"
    );
}

#[test]
fn text_box_caret_blinks_from_interaction_epoch() {
    let focus = session::Focus::text("find");
    let initial = Instant::now();
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Blink"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("abcd").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    app.show_scene_at(window, size, initial)
        .expect("initial render should install a composition");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");
    let target = interaction::Target::text_area(focus);
    let epoch = app
        .session()
        .interaction(window)
        .and_then(|interaction| interaction.text_input().caret_epoch_for(&target))
        .expect("text box focus should store a caret blink epoch");

    let visible = app
        .show_scene_at(window, size, epoch)
        .expect("epoch frame should render");
    let hidden = app
        .show_scene_at(window, size, epoch + Duration::from_millis(500))
        .expect("hidden blink frame should render");
    let visible_again = app
        .show_scene_at(window, size, epoch + Duration::from_millis(1000))
        .expect("second visible blink frame should render");

    assert!(text_box_caret_visible(&visible));
    assert!(!text_box_caret_visible(&hidden));
    assert!(text_box_caret_visible(&visible_again));
}

#[test]
fn text_box_pointer_click_positions_framework_owned_draft_caret() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Pointer"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("abcd").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    let presentation = app
        .show_scene(window, size)
        .expect("text box view should render");
    let text_box = presentation
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let rect = text_box.rect();
    let left_edge = geometry::Point::new(rect.x() + 1, rect.y() + rect.height() / 2);

    let clicked = app
        .pointer_down_at(window, size, left_edge)
        .expect("text box pointer down should be handled");

    assert!(clicked.is_handled());
    assert!(!clicked.changed_state());

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box pointer click should create a draft");

    assert_eq!(draft.text(), "abcd");
    assert_eq!(draft.cursor(), 0);

    let committed = app
        .handle_input(window, Input::text_commit("X"))
        .expect("text box commit should be handled");

    assert!(committed.is_handled());
    assert!(!committed.changed_state());

    let draft = app
        .session()
        .interaction(window)
        .expect("window should keep interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box draft should remain active");

    assert_eq!(draft.text(), "Xabcd");
    assert_eq!(draft.cursor(), "X".len());

    let projected = app
        .present(window)
        .expect("text box draft should project into the view");
    let text_box = text_box_node(projected.root()).expect("text box should be in the view");
    let text_box = text_box
        .text_box_model()
        .expect("node should contain text box state");

    assert_eq!(text_box.text(), "Xabcd");
    assert_eq!(text_box.cursor(), Some("X".len()));
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn text_box_pointer_down_to_focus_paints_activation_tint() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Pointer Activation"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("abcd").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    let presentation = app
        .show_scene(window, size)
        .expect("text box view should render");
    let text_box = presentation
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let rect = text_box.rect();
    let left_edge = geometry::Point::new(rect.x() + 1, rect.y() + rect.height() / 2);

    app.pointer_down_at(window, size, left_edge)
        .expect("text box pointer down should be handled");
    let pressed = app
        .show_scene(window, size)
        .expect("text box pointer down should render");

    assert_eq!(
        app.session()
            .focused(window)
            .expect("text box should be pointer focused")
            .visibility(),
        session::Visibility::Hidden
    );
    assert_text_box_focus_outline(&pressed);
    assert_text_box_control_pressed_tint(&pressed);

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box pointer click should create a draft");

    assert_eq!(draft.cursor(), 0);
}

#[test]
fn text_box_pointer_up_after_focus_activation_clears_activation_tint() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Pointer Activation Release"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("abcd").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    let presentation = app
        .show_scene(window, size)
        .expect("text box view should render");
    let text_box = presentation
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let rect = text_box.rect();
    let left_edge = geometry::Point::new(rect.x() + 1, rect.y() + rect.height() / 2);

    app.pointer_down_at(window, size, left_edge)
        .expect("text box pointer down should be handled");
    let pressed = app
        .show_scene(window, size)
        .expect("text box pointer down should render");
    assert_text_box_control_pressed_tint(&pressed);

    let released = app
        .pointer_up_at(window, size, left_edge)
        .expect("text box pointer up should be handled");
    assert!(released.effect().contains(&response::Effect::Paint));
    let released = app
        .show_scene(window, size)
        .expect("text box pointer up should render");

    assert_no_text_box_control_pressed_tint(&released);
}

#[test]
fn focused_text_box_pointer_down_positions_caret_without_control_pressed_tint() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Focused Text Box Pointer Paint"));
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
        .expect("text box view should render");
    app.handle_input(window, Input::focus(focus))
        .expect("text box should focus");
    let presentation = app
        .show_scene(window, size)
        .expect("focused text box should render");
    let text_box = presentation
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let rect = text_box.rect();
    let left_edge = geometry::Point::new(rect.x() + 1, rect.y() + rect.height() / 2);

    app.pointer_down_at(window, size, left_edge)
        .expect("text box pointer down should be handled");
    let pressed = app
        .show_scene(window, size)
        .expect("text box pointer down should render");

    assert_no_text_box_control_pressed_tint(&pressed);

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box pointer click should create a draft");

    assert_eq!(draft.cursor(), 0);
}

#[test]
fn text_box_pointer_drag_extends_selection_from_click_anchor() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Drag Selection"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("abcd").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    let presentation = app
        .show_scene(window, size)
        .expect("text box view should render");
    let text_box = presentation
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let rect = text_box.rect();
    let press_y = rect.y() + rect.height() / 2;
    let left_edge = geometry::Point::new(rect.x() + 1, press_y);
    let wobbled_right_edge = geometry::Point::new(rect.right() - 1, rect.y() + 2);

    pointer_down_then_present(&mut app, window, size, left_edge);
    app.pointer_move_at(window, size, wobbled_right_edge)
        .expect("text box pointer drag should be handled");
    let dragging = app
        .show_scene(window, size)
        .expect("text box drag should render");

    assert_no_text_box_control_pressed_tint(&dragging);

    pointer_up_then_present(&mut app, window, size, wobbled_right_edge);

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box pointer drag should retain a draft");

    assert_eq!(draft.text(), "abcd");
    assert_eq!(draft.cursor(), "abcd".len());
    assert_eq!(draft.selection(), Some(0.."abcd".len()));

    let selected = app
        .show_scene(window, size)
        .expect("selected text box should render");
    let selected_text_box = selected
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("selected text box should be laid out");
    let field = selected_text_box
        .text_box_layout()
        .expect("selected text box should have field layout");
    let selection = field
        .layout()
        .selection_spans()
        .first()
        .expect("selected text box should expose a shaped selection span");
    let text_rect = selected_text_box.text_box_text_rect();
    let expected = geometry::Rect::new(
        text_rect.x().saturating_add(selection.x().floor() as i32),
        text_rect.y().saturating_add(selection.y().floor() as i32),
        selection.width().ceil().max(0.0) as i32,
        selection.height().ceil().max(0.0) as i32,
    );

    assert!(
        selected.scene().quads().iter().any(|quad| {
            quad.fill().channels() == (10, 132, 255, 96) && quad.rect() == expected
        }),
        "text box drag should paint the shaped field selection span"
    );
}

fn scene_contains_text(scene: &Scene, value: &str) -> bool {
    scene.texts().iter().any(|text| text.value() == value)
}

fn scene_contains_text_surface(scene: &Scene, value: &str) -> bool {
    scene.text_viewports().iter().any(|viewport| {
        viewport.surfaces().iter().any(|surface| {
            surface
                .buffer()
                .borrow()
                .lines
                .first()
                .is_some_and(|line| line.text() == value)
        })
    })
}

fn text_box_caret_visible(presentation: &scene::Presentation) -> bool {
    let Some(text_box) = presentation
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
    else {
        return false;
    };
    let Some(caret) = text_box
        .text_box_layout()
        .and_then(|field| field.layout().caret())
    else {
        return false;
    };
    let rect = text_box.text_box_text_rect();
    let expected = geometry::Rect::new(
        rect.x().saturating_add(caret.x().floor() as i32),
        rect.y().saturating_add(caret.y().floor() as i32),
        1,
        caret.height().ceil().max(0.0) as i32,
    );

    presentation.scene().rules().into_iter().any(|rule| {
        rule.rect() == expected && rule.axis() == scene::Axis::Vertical && rule.thickness_px() == 2
    })
}

fn assert_no_text_box_control_pressed_tint(presentation: &scene::Presentation) {
    let text_box = presentation
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let pressed_tint = Theme::default().control().pressed_tint;

    assert!(
        !presentation
            .scene()
            .quads()
            .iter()
            .any(|quad| quad.rect() == text_box.rect() && quad.fill() == pressed_tint),
        "text box pointer editing should not paint the generic control pressed tint"
    );
}

fn assert_text_box_control_pressed_tint(presentation: &scene::Presentation) {
    let text_box = presentation
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let pressed_tint = Theme::default().control().pressed_tint;

    assert!(
        presentation
            .scene()
            .quads()
            .iter()
            .any(|quad| quad.rect() == text_box.rect() && quad.fill() == pressed_tint),
        "text box focus acquisition should paint the generic control pressed tint"
    );
}

fn assert_text_box_focus_outline(presentation: &scene::Presentation) {
    let text_box = presentation
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let focus = Theme::default().focus().color;

    assert!(
        presentation
            .scene()
            .outlines()
            .iter()
            .any(|outline| outline.rect() == text_box.rect() && outline.color() == focus),
        "pointer-focused editable text box should retain editor chrome"
    );
}
