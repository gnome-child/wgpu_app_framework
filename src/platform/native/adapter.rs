use super::super::{Backend, NativeError, Window};
use super::{Native, NativeContext};
use crate::{pointer, session, shell, window as app_window};

impl Backend for Native {
    type Error = NativeError;
    type Context<'a> = NativeContext<'a>;

    fn open_window(
        &mut self,
        context: &mut Self::Context<'_>,
        window: &Window,
    ) -> Result<(), Self::Error> {
        let mut native_window = self.create_native_window(context, window)?;
        self.clear_window(&mut native_window)?;
        native_window.set_ime_allowed(true);
        native_window.set_visibility(true);

        self.raw_windows.insert(native_window.raw_id(), window.id());
        self.windows.insert(window.id(), native_window);

        Ok(())
    }

    fn close_window(
        &mut self,
        _context: &mut Self::Context<'_>,
        window: app_window::Id,
    ) -> Result<(), Self::Error> {
        let Some(native_window) = self.windows.remove(&window) else {
            log::warn!("cannot close missing native window: {window:?}");
            return Err(NativeError::MissingWindow { window });
        };
        self.raw_windows.remove(&native_window.raw_id());
        log::debug!("closed native window: {window:?}");
        Ok(())
    }

    fn present(
        &mut self,
        _context: &mut Self::Context<'_>,
        presentation: &shell::Presentation,
    ) -> Result<(), Self::Error> {
        self.present_native(presentation)
    }

    fn request(
        &mut self,
        _context: &mut Self::Context<'_>,
        request: session::Request,
    ) -> Result<(), Self::Error> {
        self.request_once(request);
        Ok(())
    }

    fn set_cursor(
        &mut self,
        _context: &mut Self::Context<'_>,
        window: app_window::Id,
        cursor: pointer::Cursor,
    ) -> Result<(), Self::Error> {
        Native::set_cursor(self, window, cursor)
    }

    fn schedule_poll(&mut self, _context: &mut Self::Context<'_>) -> Result<(), Self::Error> {
        self.schedule_poll_request();
        Ok(())
    }
}
