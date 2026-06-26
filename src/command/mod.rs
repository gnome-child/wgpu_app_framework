pub mod args;
pub mod binding;
pub mod call;
pub mod definition;
pub mod effect;
mod key;
mod output;
pub mod registry;
mod response;
pub mod shortcut;
pub mod state;
pub mod target;

pub use args::Args;
pub use call::Call;
pub use effect::Effect;
pub(crate) use key::Key;
pub use output::Output;
pub use registry::Registry;
pub use response::Response;
pub use state::State;
pub use target::Target;

#[cfg(test)]
pub(crate) struct TestTarget;

#[cfg(test)]
impl<C> Target<C> for TestTarget
where
    C: Command,
    C::Output: Default,
{
    fn invoke(&mut self, _args: C::Args, _invocation: call::Invocation<C>) -> Response<C::Output> {
        Response::output(C::Output::default())
    }
}

pub trait Command: 'static + Sized {
    type Args: args::Args;
    type Output: output::Output;

    const NAME: &'static str;
    const DISPLAY: &'static str;

    fn hint() -> Option<&'static str> {
        None
    }

    fn repeatable() -> bool {
        false
    }

    fn target() -> target::Kind {
        target::Kind::command(key::Key::of::<Self>())
    }
}

#[macro_export]
macro_rules! command {
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident {
            name: $command_name:literal,
            display: $display:literal
            $(, args: $args:ty)?
            $(, output: $output:ty)?
            $(, hint: $hint:literal)?
            $(, repeatable: $repeatable:expr)?
            $(, target: $target:ty)?
            $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name;

        impl $crate::Command for $name {
            type Args = $crate::command!(@type_or_unit $($args)?);
            type Output = $crate::command!(@type_or_unit $($output)?);

            const NAME: &'static str = $command_name;
            const DISPLAY: &'static str = $display;

            fn hint() -> Option<&'static str> {
                $crate::command!(@option_str $($hint)?)
            }

            fn repeatable() -> bool {
                $crate::command!(@bool $($repeatable)?)
            }

            $(
                fn target() -> $crate::command::target::Kind {
                    <$target as $crate::command::target::Category>::kind()
                }
            )?
        }
    };

    (
        $(#[$meta:meta])*
        $vis:vis $name:ident;
    ) => {
        $crate::command! {
            $(#[$meta])*
            $vis $name {
                name: stringify!($name),
                display: stringify!($name),
            }
        }
    };

    (@type_or_unit) => {
        ()
    };

    (@type_or_unit $ty:ty) => {
        $ty
    };

    (@option_str) => {
        None
    };

    (@option_str $value:literal) => {
        Some($value)
    };

    (@bool) => {
        false
    };

    (@bool $value:expr) => {
        $value
    };
}

#[cfg(test)]
mod tests;
