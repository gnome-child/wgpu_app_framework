use std::path::PathBuf;
use wgpu_l3::document;

pub enum Event {
    FileSaved {
        version: document::Version,
        generation: u64,
        path: PathBuf,
        result: Result<(), String>,
    },
}
