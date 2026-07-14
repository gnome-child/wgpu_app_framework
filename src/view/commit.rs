use std::{fmt::Display, sync::Arc};

use super::super::command;

#[derive(Clone)]
pub(crate) struct TextCommit {
    build: Arc<dyn Fn(String) -> Result<command::AnyTrigger, String> + Send + Sync>,
}

impl TextCommit {
    pub(crate) fn infallible<C>(map: impl Fn(String) -> C::Args + Send + Sync + 'static) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self {
            build: Arc::new(move |text| Ok(command::AnyTrigger::command::<C>(map(text)))),
        }
    }

    pub(crate) fn fallible<C, E>(
        map: impl Fn(String) -> Result<C::Args, E> + Send + Sync + 'static,
    ) -> Self
    where
        C: command::Command,
        C::Args: Clone,
        E: Display,
    {
        Self {
            build: Arc::new(move |text| {
                map(text)
                    .map(command::AnyTrigger::command::<C>)
                    .map_err(|error| error.to_string())
            }),
        }
    }

    pub(crate) fn formatted<C>(
        map: impl Fn(String) -> Result<C::Args, String> + Send + Sync + 'static,
    ) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self {
            build: Arc::new(move |text| map(text).map(command::AnyTrigger::command::<C>)),
        }
    }

    pub(crate) fn build(&self, text: String) -> Result<command::AnyTrigger, String> {
        (self.build)(text)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fmt,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use super::*;

    struct Commit;

    impl command::Command for Commit {
        type Args = String;
        type Output = ();

        const NAME: &'static str = "test.fallible_text_commit";
    }

    struct Rejection {
        formats: Arc<AtomicUsize>,
    }

    impl fmt::Display for Rejection {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.formats.fetch_add(1, Ordering::SeqCst);
            formatter.write_str("rejected once")
        }
    }

    #[test]
    fn fallible_commit_formats_one_rejection_snapshot() {
        let formats = Arc::new(AtomicUsize::new(0));
        let commit = TextCommit::fallible::<Commit, Rejection>({
            let formats = Arc::clone(&formats);
            move |_| {
                Err(Rejection {
                    formats: Arc::clone(&formats),
                })
            }
        });

        let reason = match commit.build("draft".to_owned()) {
            Ok(_) => panic!("the test recipe should reject"),
            Err(reason) => reason,
        };
        assert_eq!(reason, "rejected once");
        assert_eq!(formats.load(Ordering::SeqCst), 1);
    }
}
