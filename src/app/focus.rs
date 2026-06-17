use crate::ui;

#[derive(Debug, Default)]
pub struct State {
    current: Option<Focus>,
    transient_scopes: Vec<Scope>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Focus {
    pub path: ui::Path,
    pub reason: ui::focus::Reason,
    pub visibility: ui::focus::Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransientScope {
    Menu,
    Submenu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scope {
    id: TransientScope,
    roots: Vec<ui::Path>,
    restore: Option<Focus>,
}

impl State {
    #[cfg(test)]
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn focused(focus: Focus) -> Self {
        Self {
            current: Some(focus),
            transient_scopes: Vec::new(),
        }
    }

    pub fn as_ref(&self) -> Option<&Focus> {
        self.current.as_ref()
    }

    pub fn path(&self) -> Option<ui::Path> {
        self.current.as_ref().map(|focus| focus.path.clone())
    }

    pub fn visibility(&self) -> ui::focus::Visibility {
        self.current
            .as_ref()
            .map(Focus::visibility)
            .unwrap_or(ui::focus::Visibility::Hidden)
    }

    pub fn restores_to(&self, path: &ui::Path) -> bool {
        self.transient_scopes.iter().rev().any(|scope| {
            scope
                .restore
                .as_ref()
                .is_some_and(|focus| focus.path == *path)
        })
    }

    pub fn set(&mut self, focus: Focus) -> bool {
        if self.current.as_ref() == Some(&focus) {
            return false;
        }

        self.current = Some(focus);
        true
    }

    pub fn clear(&mut self) -> bool {
        let changed = self.current.is_some();
        self.current = None;
        changed
    }

    pub fn clear_stale(&mut self, is_focusable: impl FnOnce(&ui::Path) -> bool) -> bool {
        let Some(path) = self.current.as_ref().map(|focus| &focus.path) else {
            return false;
        };

        if is_focusable(path) {
            return false;
        }

        self.clear()
    }

    pub fn begin_transient(&mut self, id: TransientScope, root: ui::Path) -> bool {
        if let Some(scope) = self
            .transient_scopes
            .iter_mut()
            .find(|scope| scope.id == id)
        {
            return scope.include_root(root);
        }

        self.transient_scopes.push(Scope {
            id,
            roots: vec![root],
            restore: self.current.clone(),
        });
        true
    }

    pub fn include_transient_root(&mut self, id: TransientScope, root: ui::Path) -> bool {
        self.transient_scopes
            .iter_mut()
            .find(|scope| scope.id == id)
            .is_some_and(|scope| scope.include_root(root))
    }

    pub fn close_transient(
        &mut self,
        id: TransientScope,
        visibility: Option<ui::focus::Visibility>,
    ) -> bool {
        let Some(index) = self
            .transient_scopes
            .iter()
            .rposition(|scope| scope.id == id)
        else {
            return false;
        };

        let removed: Vec<_> = self.transient_scopes.drain(index..).collect();
        let restore = removed
            .first()
            .and_then(|scope| scope.restore.clone())
            .map(|mut focus| {
                if let Some(visibility) = visibility {
                    focus.visibility = visibility;
                }
                focus
            });

        let current_inside_scope = self
            .current
            .as_ref()
            .is_some_and(|focus| removed.iter().any(|scope| scope.contains(&focus.path)));

        if !current_inside_scope {
            return true;
        }

        match restore {
            Some(focus) => {
                self.set(focus);
                true
            }
            None => {
                self.clear();
                true
            }
        }
    }
}

impl Focus {
    pub fn new(
        path: ui::Path,
        reason: ui::focus::Reason,
        visibility: ui::focus::Visibility,
    ) -> Self {
        Self {
            path,
            reason,
            visibility,
        }
    }

    pub fn visibility(&self) -> ui::focus::Visibility {
        self.visibility
    }
}

impl Scope {
    fn include_root(&mut self, root: ui::Path) -> bool {
        if self.roots.iter().any(|existing| existing == &root) {
            return false;
        }

        self.roots.push(root);
        true
    }

    fn contains(&self, path: &ui::Path) -> bool {
        self.roots.iter().any(|root| path.is_descendant_of(root))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIELD: ui::Id = ui::Id::new("field");
    const MENU: ui::Id = ui::Id::new("menu");
    const MENU_ROW: ui::Id = ui::Id::new("menu_row");
    const SUBMENU: ui::Id = ui::Id::new("submenu");
    const SUBMENU_ROW: ui::Id = ui::Id::new("submenu_row");
    const OUTSIDE: ui::Id = ui::Id::new("outside");

    fn focus(id: ui::Id) -> Focus {
        Focus::new(
            ui::Path::from(id),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        )
    }

    #[test]
    fn set_clear_and_clear_stale_manage_current_focus() {
        let mut state = State::new();

        assert!(state.set(focus(FIELD)));
        assert_eq!(state.path(), Some(ui::Path::from(FIELD)));
        assert!(!state.clear_stale(|_| true));
        assert!(state.clear_stale(|_| false));
        assert_eq!(state.path(), None);

        assert!(!state.clear());
    }

    #[test]
    fn closing_transient_scope_restores_captured_focus_when_current_is_inside_scope() {
        let mut state = State::focused(focus(FIELD));
        let menu_root = ui::Path::from(MENU);

        assert!(state.begin_transient(TransientScope::Menu, menu_root.clone()));
        assert!(state.restores_to(&ui::Path::from(FIELD)));
        assert!(state.set(Focus::new(
            menu_root.child(MENU_ROW),
            ui::focus::Reason::Pointer,
            ui::focus::Visibility::Hidden,
        )));

        assert!(state.close_transient(TransientScope::Menu, None));
        assert_eq!(state.path(), Some(ui::Path::from(FIELD)));
        assert_eq!(state.visibility(), ui::focus::Visibility::Visible);
        assert!(!state.restores_to(&ui::Path::from(FIELD)));
    }

    #[test]
    fn closing_transient_scope_preserves_focus_that_moved_outside_scope() {
        let mut state = State::focused(focus(FIELD));

        assert!(state.begin_transient(TransientScope::Menu, ui::Path::from(MENU)));
        assert!(state.set(focus(OUTSIDE)));

        assert!(state.close_transient(TransientScope::Menu, None));
        assert_eq!(state.path(), Some(ui::Path::from(OUTSIDE)));
    }

    #[test]
    fn nested_transient_scopes_restore_in_order() {
        let mut state = State::focused(focus(FIELD));
        let menu_root = ui::Path::from(MENU);
        let menu_row = menu_root.child(MENU_ROW);
        let submenu_root = menu_root.child(SUBMENU);

        assert!(state.begin_transient(TransientScope::Menu, menu_root.clone()));
        assert!(state.set(Focus::new(
            menu_row.clone(),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        )));
        assert!(state.begin_transient(TransientScope::Submenu, submenu_root.clone(),));
        assert!(state.set(Focus::new(
            submenu_root.child(SUBMENU_ROW),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        )));

        assert!(state.close_transient(TransientScope::Submenu, None));
        assert_eq!(state.path(), Some(menu_row));

        assert!(state.close_transient(TransientScope::Menu, None));
        assert_eq!(state.path(), Some(ui::Path::from(FIELD)));
    }

    #[test]
    fn closing_transient_scope_can_override_restored_visibility() {
        let mut state = State::focused(focus(FIELD));
        let menu_root = ui::Path::from(MENU);

        assert!(state.begin_transient(TransientScope::Menu, menu_root.clone()));
        assert!(state.set(Focus::new(
            menu_root.child(MENU_ROW),
            ui::focus::Reason::Pointer,
            ui::focus::Visibility::Hidden,
        )));

        assert!(state.close_transient(TransientScope::Menu, Some(ui::focus::Visibility::Hidden),));
        assert_eq!(state.path(), Some(ui::Path::from(FIELD)));
        assert_eq!(state.visibility(), ui::focus::Visibility::Hidden);
    }
}
