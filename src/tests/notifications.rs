use super::*;

struct TestNotice;

impl notification::Notification for TestNotice {
    type Payload = &'static str;

    const NAME: &'static str = "test.notice";
}

#[derive(Default)]
struct NotificationPane {
    events: Rc<RefCell<Vec<String>>>,
}

struct NotificationState {
    events: Rc<RefCell<Vec<String>>>,
    pane: NotificationPane,
}

impl Default for NotificationState {
    fn default() -> Self {
        let events = Rc::new(RefCell::new(Vec::new()));
        Self {
            events: Rc::clone(&events),
            pane: NotificationPane { events },
        }
    }
}

impl Clone for NotificationState {
    fn clone(&self) -> Self {
        Self {
            events: Rc::clone(&self.events),
            pane: NotificationPane {
                events: Rc::clone(&self.pane.events),
            },
        }
    }
}

impl State for NotificationState {}

impl notification::Listener<TestNotice> for NotificationPane {
    fn notify(&mut self, payload: &&'static str) -> notification::Reaction {
        self.events.borrow_mut().push(format!("focused:{payload}"));
        notification::Reaction::changed().with_effect(response::Effect::OpenFileDialog)
    }
}

impl notification::Listener<TestNotice> for NotificationState {
    fn notify(&mut self, payload: &&'static str) -> notification::Reaction {
        self.events.borrow_mut().push(format!("app:{payload}"));
        notification::Reaction::changed().with_effect(response::Effect::SaveFileDialog)
    }
}

#[derive(Clone, Default)]
struct DepartedState {
    windows: Vec<window::Id>,
}

impl State for DepartedState {}

impl notification::Listener<window::Departed> for DepartedState {
    fn notify(&mut self, window: &window::Id) -> notification::Reaction {
        self.windows.push(*window);
        notification::Reaction::changed()
    }
}

fn notification_app() -> Runtime<NotificationState> {
    Runtime::new(NotificationState::default())
        .responders(|responders| {
            responders
                .object("pane", |state: &mut NotificationState| &mut state.pane)
                .listen::<TestNotice>();
            responders.app().listen::<TestNotice>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Notifications"));
        })
}

#[test]
fn notification_with_no_listeners_is_silent_success() {
    let mut app = Runtime::new(NotificationState::default()).started(|cx| {
        cx.open_window(window::Options::new("Notifications"));
    });
    app.start();
    let window = app.session().windows()[0].id();

    let reaction = app.notify_focused::<TestNotice>(window, "quiet", context::Source::Programmatic);

    assert!(!reaction.changed_state());
    assert_eq!(reaction.effect(), &response::Effect::None);
    assert_eq!(app.revision(), state::Revision::initial());
    assert!(app.state().events.borrow().is_empty());
}

#[test]
fn notification_listeners_all_run_in_chain_order_and_merge_effects() {
    let mut app = notification_app();
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("pane")));

    let reaction = app.notify_focused::<TestNotice>(window, "news", context::Source::Programmatic);

    assert!(reaction.changed_state());
    assert_eq!(
        *app.state().events.borrow(),
        vec!["focused:news".to_owned(), "app:news".to_owned()]
    );
    assert_eq!(
        reaction.effect(),
        &response::Effect::OpenFileDialog.then(response::Effect::SaveFileDialog)
    );
}

#[test]
fn notification_changes_commit_notification_reason_without_undo_snapshot() {
    let mut app = notification_app();
    app.start();
    let window = app.session().windows()[0].id();

    let reaction =
        app.notify_focused::<TestNotice>(window, "change", context::Source::Programmatic);

    assert!(reaction.changed_state());
    assert_eq!(app.revision().get(), 1);
    assert_eq!(
        app.store().changes()[0].reason(),
        &state::Reason::Notification(<TestNotice as notification::Notification>::NAME)
    );
    assert!(!app.undo());
    assert_eq!(*app.state().events.borrow(), vec!["app:change".to_owned()]);
}

#[test]
fn departed_is_delivered_to_registered_application_listeners() {
    let mut app = Runtime::new(DepartedState::default())
        .responders(|responders| {
            responders.app().listen::<window::Departed>();
        })
        .started(|cx| {
            let window = cx.open_window(window::Options::new("Ephemeral"));
            assert!(cx.close_window(window));
        });

    app.start();

    assert_eq!(app.state().windows.len(), 1);
    assert!(app.session().windows().is_empty());
    assert_eq!(
        app.store().changes()[0].reason(),
        &state::Reason::Notification(<window::Departed as notification::Notification>::NAME,)
    );
}
