use super::*;

#[test]
fn nearest_responder_target_wins() {
    let mut registry = command::Registry::default();
    registry.register::<Save>(command::Spec::new("Save").shortcut("Ctrl+S"));

    let mut store = state::Store::new(EditorState {
        document: SaveDocument {
            dirty: true,
            save_count: 0,
        },
        project: Project {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    });
    let mut responders = responder::Builder::default();
    responders
        .object("document", |state: &mut EditorState| &mut state.document)
        .target::<Save>();
    responders.app().target::<Save>();

    let mut chain = responders.chain_for(&mut store, Some(session::Focus::text("document")));
    let trigger = command::Trigger::<Save>::command(());

    assert!(
        trigger
            .state(&registry, &mut chain, &Context::default())
            .is_enabled()
    );

    let response = trigger.invoke(&registry, &mut chain, &mut Context::default());
    assert!(response.output.is_ok());
    assert!(response.effect.contains_invalidation());

    drop(chain);
    assert!(!store.model().document.dirty);
    assert_eq!(store.model().document.save_count, 1);
    assert!(store.model().project.dirty);
    assert_eq!(store.model().project.save_count, 0);
}

#[test]
fn hidden_target_falls_through_to_next_responder() {
    let mut registry = command::Registry::default();
    registry.register::<Save>(command::Spec::new("Save").shortcut("Ctrl+S"));

    let mut store = state::Store::new(HiddenSaveState {
        project: Project {
            dirty: true,
            save_count: 0,
        },
        ..HiddenSaveState::default()
    });
    let mut responders = responder::Builder::default();
    responders
        .object("passive", |state: &mut HiddenSaveState| &mut state.passive)
        .target::<Save>();
    responders.app().target::<Save>();

    let mut chain = responders.chain_for(&mut store, Some(session::Focus::text("passive")));
    let trigger = command::Trigger::<Save>::command(());

    assert_eq!(
        trigger
            .state(&registry, &mut chain, &Context::default())
            .label
            .as_deref(),
        Some("Save Project")
    );

    let response = trigger.invoke(&registry, &mut chain, &mut Context::default());
    assert!(response.output.is_ok());

    drop(chain);
    assert!(!store.model().project.dirty);
    assert_eq!(store.model().project.save_count, 1);
}

#[test]
fn disabled_target_claims_and_blocks_invocation() {
    let mut registry = command::Registry::default();
    registry.register::<Save>(command::Spec::new("Save").shortcut("Ctrl+S"));

    let mut store = state::Store::new(EditorState {
        document: SaveDocument {
            dirty: false,
            save_count: 0,
        },
        project: Project {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    });
    let mut responders = responder::Builder::default();
    responders
        .object("document", |state: &mut EditorState| &mut state.document)
        .target::<Save>();
    responders.app().target::<Save>();

    let mut chain = responders.chain_for(&mut store, Some(session::Focus::text("document")));
    let trigger = command::Trigger::<Save>::command(());

    assert!(
        !trigger
            .state(&registry, &mut chain, &Context::default())
            .is_enabled()
    );

    let response = trigger.invoke(&registry, &mut chain, &mut Context::default());
    assert!(matches!(
        response.output,
        Err(Error::Disabled {
            command: "app.save"
        })
    ));

    drop(chain);
    assert_eq!(store.model().document.save_count, 0);
    assert!(store.model().project.dirty);
    assert_eq!(store.model().project.save_count, 0);
}

#[test]
fn same_responder_visible_targets_are_ambiguous() {
    let mut registry = command::Registry::default();
    registry.register::<Save>(command::Spec::new("Save").shortcut("Ctrl+S"));

    let mut store = state::Store::new(EditorState {
        project: Project {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    });
    let mut responders = responder::Builder::default();
    responders.app().target::<Save>().target::<Save>();

    let mut chain = responders.chain(&mut store);
    let trigger = command::Trigger::<Save>::command(());

    assert_eq!(
        trigger
            .state(&registry, &mut chain, &Context::default())
            .tooltip
            .as_deref(),
        Some("multiple targets claim command app.save in responder app")
    );

    let response = trigger.invoke(&registry, &mut chain, &mut Context::default());
    assert!(matches!(
        response.output,
        Err(Error::AmbiguousTarget {
            command: "app.save",
            responder: "app",
        })
    ));

    drop(chain);
    assert!(store.model().project.dirty);
    assert_eq!(store.model().project.save_count, 0);
}
