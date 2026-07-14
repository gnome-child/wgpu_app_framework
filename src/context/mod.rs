mod source;

pub use source::Source;

use std::{cell::RefCell, fmt, rc::Rc};

use super::{clipboard::Clipboard, task, text};

pub struct Context {
    source: Source,
    clipboard: Option<Clipboard>,
    caret_map: Option<Rc<RefCell<dyn text::selection::CaretMap>>>,
    tasks: Option<task::Sink>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            source: Source::Programmatic,
            clipboard: None,
            caret_map: None,
            tasks: None,
        }
    }
}

impl Context {
    pub(super) fn with_clipboard(clipboard: &mut Clipboard) -> Self {
        Self::with_clipboard_source(clipboard, Source::Programmatic)
    }

    pub(super) fn with_clipboard_source(clipboard: &mut Clipboard, source: Source) -> Self {
        Self {
            source,
            clipboard: Some(clipboard.clone()),
            caret_map: None,
            tasks: None,
        }
    }

    pub(super) fn with_services_source(
        clipboard: &mut Clipboard,
        tasks: task::Sink,
        source: Source,
    ) -> Self {
        Self {
            source,
            clipboard: Some(clipboard.clone()),
            caret_map: None,
            tasks: Some(tasks),
        }
    }

    pub(super) fn with_caret_map(
        mut self,
        caret_map: Rc<RefCell<dyn text::selection::CaretMap>>,
    ) -> Self {
        self.caret_map = Some(caret_map);
        self
    }

    pub(super) fn sourced(&self, source: Source) -> Self {
        Self {
            source,
            clipboard: self.clipboard.clone(),
            caret_map: self.caret_map.clone(),
            tasks: self.tasks.clone(),
        }
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn spawn<E: Send + 'static>(&mut self, task: task::Task<E>) -> Option<task::Id> {
        self.tasks
            .as_mut()
            .and_then(|tasks| tasks.spawn(task.into_any()))
    }

    pub(super) fn clipboard(&self) -> Option<&Clipboard> {
        self.clipboard.as_ref()
    }

    pub(super) fn clipboard_mut(&mut self) -> Option<Clipboard> {
        self.clipboard.clone()
    }

    pub(super) fn caret_map(&self) -> Option<Rc<RefCell<dyn text::selection::CaretMap>>> {
        self.caret_map.clone()
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("source", &self.source)
            .field("clipboard", &self.clipboard)
            .field("caret_map", &self.caret_map.is_some())
            .field("tasks", &self.tasks)
            .finish()
    }
}
