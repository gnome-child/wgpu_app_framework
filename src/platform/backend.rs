use super::super::{geometry, pointer, scene, session, shell, window};

pub trait Backend {
    type Error;
    type Context<'a>;

    fn open_window(
        &mut self,
        context: &mut Self::Context<'_>,
        window: &Window,
    ) -> Result<(), Self::Error>;

    fn close_window(
        &mut self,
        context: &mut Self::Context<'_>,
        window: window::Id,
    ) -> Result<(), Self::Error>;

    fn present(
        &mut self,
        context: &mut Self::Context<'_>,
        presentation: &shell::Presentation,
    ) -> Result<(), Self::Error>;

    fn request(
        &mut self,
        context: &mut Self::Context<'_>,
        request: session::Request,
    ) -> Result<(), Self::Error>;

    fn set_cursor(
        &mut self,
        _context: &mut Self::Context<'_>,
        _window: window::Id,
        _cursor: pointer::Cursor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn schedule_poll(&mut self, context: &mut Self::Context<'_>) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    id: window::Id,
    title: String,
    size: geometry::Size,
    canvas_color: scene::Color,
}

impl Window {
    pub(super) fn from_shell(window: &shell::Window) -> Self {
        Self {
            id: window.id(),
            title: window.title().to_owned(),
            size: window.size(),
            canvas_color: window.canvas_color(),
        }
    }

    pub fn id(&self) -> window::Id {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn size(&self) -> geometry::Size {
        self.size
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.canvas_color
    }
}
