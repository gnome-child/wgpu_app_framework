#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(pub(super) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    Canceled,
    Completed,
}
