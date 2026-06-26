pub trait Output: Send + 'static {}

impl Output for () {}
impl Output for bool {}
impl Output for f64 {}
impl Output for i32 {}
impl Output for usize {}
impl Output for String {}

impl<T: Send + 'static> Output for Option<T> {}
impl<T: Send + 'static> Output for Vec<T> {}
