use crate::scratch::{session, state};

use super::Runtime;

#[derive(Clone)]
pub struct Snapshot<M: state::State> {
    pub(super) state: state::Snapshot<M>,
    pub(super) session: session::Snapshot,
}

pub trait Persistence<M: state::State> {
    type Error;

    fn save(&mut self, snapshot: &Snapshot<M>) -> Result<(), Self::Error>;

    fn load(&mut self) -> Result<Snapshot<M>, Self::Error>;
}

impl<M: state::State> Snapshot<M> {
    pub fn new(state: state::Snapshot<M>, session: session::Snapshot) -> Self {
        Self { state, session }
    }

    pub fn state(&self) -> &state::Snapshot<M> {
        &self.state
    }

    pub fn session(&self) -> &session::Snapshot {
        &self.session
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn snapshot(&self) -> Snapshot<M> {
        Snapshot {
            state: self.store.snapshot(),
            session: self.session.snapshot(),
        }
    }

    pub fn restore(&mut self, snapshot: Snapshot<M>) -> state::Change {
        self.restore_with_reason(snapshot, state::Reason::Restore)
    }

    pub fn save<P>(&mut self, persistence: &mut P) -> Result<state::Revision, P::Error>
    where
        P: Persistence<M>,
    {
        let snapshot = self.snapshot();
        persistence.save(&snapshot)?;
        self.store.mark_saved();
        self.request_all_redraws();
        Ok(self.revision())
    }

    pub fn load<P>(&mut self, persistence: &mut P) -> Result<state::Change, P::Error>
    where
        P: Persistence<M>,
    {
        let snapshot = persistence.load()?;
        Ok(self.restore_with_reason(snapshot, state::Reason::Load))
    }

    fn restore_with_reason(
        &mut self,
        snapshot: Snapshot<M>,
        reason: state::Reason,
    ) -> state::Change {
        let change = self.store.restore(snapshot.state, reason);
        self.store.mark_saved();
        self.session.restore(snapshot.session);
        self.composition.clear();
        self.timeline.clear();
        self.gesture = None;
        self.history_group = None;
        self.tasks.clear();
        self.diagnostics.restore_windows(self.session.windows());
        change
    }
}
