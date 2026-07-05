use std::path::PathBuf;

pub enum Event {
    FileSaved {
        path: PathBuf,
        result: Result<(), String>,
    },
}
