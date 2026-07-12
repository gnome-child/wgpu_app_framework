use super::super::{diagnostics, geometry, ime, overlay, pointer, scene, session, shell, window};

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

    fn request_redraw(
        &mut self,
        _context: &mut Self::Context<'_>,
        _window: window::Id,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn present(
        &mut self,
        context: &mut Self::Context<'_>,
        presentation: &shell::Presentation,
    ) -> Result<diagnostics::RenderReport, Self::Error>;

    #[allow(private_interfaces)]
    fn overlay_capabilities(&self) -> overlay::Capabilities {
        overlay::Capabilities::default()
    }

    #[allow(private_interfaces)]
    fn present_overlay_popups(
        &mut self,
        _context: &mut Self::Context<'_>,
        _synchronized_parents: &[window::Id],
        _presentations: &[overlay::PopupPresentation],
    ) -> Result<(), Self::Error> {
        Ok(())
    }

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

    #[allow(private_interfaces)]
    fn set_ime(
        &mut self,
        _context: &mut Self::Context<'_>,
        _update: ime::Update,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn schedule_poll(&mut self, context: &mut Self::Context<'_>) -> Result<(), Self::Error>;

    fn maintain(&mut self, _context: &mut Self::Context<'_>) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    facts: window::Facts,
}

impl Window {
    pub(super) fn from_shell(window: &shell::Window) -> Self {
        Self {
            facts: window.facts().clone(),
        }
    }

    pub fn id(&self) -> window::Id {
        self.facts.id()
    }

    pub fn title(&self) -> &str {
        self.facts.title()
    }

    pub fn size(&self) -> geometry::Size {
        self.facts.inner_size()
    }

    pub fn canvas_color(&self) -> scene::Color {
        self.facts.canvas_color()
    }

    pub fn kind(&self) -> window::Kind {
        self.facts.kind()
    }
}
