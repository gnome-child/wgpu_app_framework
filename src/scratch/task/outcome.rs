use super::{Id, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    id: Id,
    status: Status,
    changed_state: bool,
}

impl Outcome {
    pub(in crate::scratch) fn completed(id: Id, changed_state: bool) -> Self {
        Self {
            id,
            status: Status::Completed,
            changed_state,
        }
    }

    pub fn id(self) -> Id {
        self.id
    }

    pub fn status(self) -> Status {
        self.status
    }

    pub fn changed_state(self) -> bool {
        self.changed_state
    }
}
