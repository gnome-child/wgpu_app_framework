mod source;

pub use source::Source;

use super::{clipboard::Clipboard, layout, task};

#[derive(Debug)]
pub struct Context {
    source: Source,
    clipboard: Option<Clipboard>,
    text: Option<layout::text::Service>,
    tasks: Option<task::Sink>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            source: Source::Programmatic,
            clipboard: None,
            text: None,
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
            text: None,
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
            text: None,
            tasks: Some(tasks),
        }
    }

    pub(super) fn with_text_service(mut self, text: layout::text::Service) -> Self {
        self.text = Some(text);
        self
    }

    pub(super) fn sourced(&self, source: Source) -> Self {
        Self {
            source,
            clipboard: self.clipboard.clone(),
            text: self.text.clone(),
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

    pub(super) fn text_service(&self) -> Option<layout::text::Service> {
        self.text.clone()
    }
}
