use super::super::{Backend, NativeError, Window};
use super::{CursorHost, Native, NativeContext};
use crate::{
    diagnostics, ime, notification, overlay, pointer, session, shell, window as app_window,
};

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
        native_window.set_ime_allowed(false);
        native_window.set_visibility(true);

        self.raw_windows.insert(native_window.raw_id(), window.id());
        self.windows.insert(window.id(), native_window);
        self.cursor_hosts.insert(window.id(), CursorHost::Parent);
        self.cursor_values
            .insert(window.id(), pointer::Cursor::Default);

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
        self.cursor_hosts.remove(&window);
        self.cursor_values.remove(&window);
        self.ime_targets.remove(&window);
        <Self as notification::Listener<app_window::Departed>>::notify(self, &window);
        log::debug!("closed native window: {window:?}");
        Ok(())
    }

    fn request_redraw(
        &mut self,
        _context: &mut Self::Context<'_>,
        window: app_window::Id,
    ) -> Result<(), Self::Error> {
        Native::request_redraw(self, window)
    }

    fn present(
        &mut self,
        _context: &mut Self::Context<'_>,
        presentation: &shell::Presentation,
    ) -> Result<diagnostics::RenderReport, Self::Error> {
        self.present_native(presentation)
    }

    #[allow(private_interfaces)]
    fn overlay_capabilities(&self) -> overlay::Capabilities {
        Native::overlay_capabilities()
    }

    #[allow(private_interfaces)]
    fn present_overlay_popups(
        &mut self,
        context: &mut Self::Context<'_>,
        synchronized_parents: &[app_window::Id],
        presentations: &[overlay::PopupPresentation],
    ) -> Result<(), Self::Error> {
        self.present_popup_overlays(context, synchronized_parents, presentations)
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

    #[allow(private_interfaces)]
    fn set_ime(
        &mut self,
        _context: &mut Self::Context<'_>,
        update: ime::Update,
    ) -> Result<(), Self::Error> {
        self.apply_ime_update(update);
        Ok(())
    }

    fn schedule_poll(&mut self, _context: &mut Self::Context<'_>) -> Result<(), Self::Error> {
        self.schedule_poll_request();
        Ok(())
    }

    fn maintain(&mut self, _context: &mut Self::Context<'_>) -> Result<(), Self::Error> {
        let now = std::time::Instant::now();
        let redraw_parents = self.apply_due_popup_accents(now);
        self.apply_due_popup_borders(now);
        self.request_popup_parent_redraws(&redraw_parents);
        Ok(())
    }
}

impl notification::Listener<app_window::Departed> for Native {
    fn notify(&mut self, window: &app_window::Id) -> notification::Reaction {
        let stale = self
            .popups
            .keys()
            .filter(|key| key.parent == *window)
            .copied()
            .collect::<Vec<_>>();
        for key in stale {
            if let Some(popup) = self.popups.remove(&key) {
                self.raw_popups.remove(&popup.window.raw_id());
            }
        }
        self.cursor_hosts.remove(window);
        self.cursor_values.remove(window);
        self.ime_targets.remove(window);

        notification::Reaction::ignored()
    }
}
