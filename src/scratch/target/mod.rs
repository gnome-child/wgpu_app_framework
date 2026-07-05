use std::rc::Rc;

mod any;

pub(in crate::scratch) use any::AnyTarget;

use super::{
    command::{Command, State},
    context::Context,
    response::Response,
};

/// This is the role. A concrete value opts into a command by implementing this.
pub trait Target<C: Command> {
    fn state(&self, args: &C::Args, cx: &Context) -> State;

    fn invoke(&mut self, args: C::Args, cx: &mut Context) -> Response<C::Output>;
}

pub(in crate::scratch) type Selector<M, T> = Rc<dyn for<'a> Fn(&'a mut M) -> &'a mut T>;
