use super::super::buffer::{Buffer, Position};
use super::{Motion, State};

pub trait CaretMap {
    fn position_for_motion(
        &mut self,
        buffer: &Buffer,
        state: State,
        motion: Motion,
    ) -> Option<Position>;
}
