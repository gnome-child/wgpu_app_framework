use winit::event_loop::ActiveEventLoop;

pub struct NativeContext<'a> {
    event_loop: &'a ActiveEventLoop,
}

impl<'a> NativeContext<'a> {
    pub fn new(event_loop: &'a ActiveEventLoop) -> Self {
        NativeContext { event_loop }
    }

    pub fn event_loop(&self) -> &ActiveEventLoop {
        self.event_loop
    }
}
