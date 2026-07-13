use std::marker::PhantomData;

use crate::{command, view};

use super::Widget;

pub struct Binding<C: command::Command> {
    args: C::Args,
    form: Form,
    _command: PhantomData<C>,
}

#[derive(Clone, Copy)]
enum Form {
    Button,
    Menu,
}

impl<C> Binding<C>
where
    C: command::Command<Args = ()>,
{
    pub fn button() -> Self {
        Self::button_with_args(())
    }

    pub fn menu() -> Self {
        Self::menu_with_args(())
    }
}

impl<C> Binding<C>
where
    C: command::Command,
{
    pub fn button_with_args(args: C::Args) -> Self {
        Self::new(args, Form::Button)
    }

    pub fn menu_with_args(args: C::Args) -> Self {
        Self::new(args, Form::Menu)
    }

    fn new(args: C::Args, form: Form) -> Self {
        Self {
            args,
            form,
            _command: PhantomData,
        }
    }
}

impl<C> Widget for Binding<C>
where
    C: command::Command,
    C::Args: Clone,
{
    fn into_node(self) -> view::Node {
        match self.form {
            Form::Button => view::Node::bound_with_args::<C>(self.args),
            Form::Menu => view::Node::menu_bound_with_args::<C>(self.args),
        }
    }
}
