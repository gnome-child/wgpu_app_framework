use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_DOCUMENT_IDENTITY: AtomicU64 = AtomicU64::new(1);
static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identity(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Version {
    identity: Identity,
    revision: u64,
}

#[derive(Debug, Clone)]
pub struct SaveSnapshot {
    version: Version,
    text: String,
}

impl Identity {
    pub(super) fn next() -> Self {
        Self(NEXT_DOCUMENT_IDENTITY.fetch_add(1, Ordering::Relaxed))
    }
}

impl Version {
    pub(super) fn new(identity: Identity, revision: u64) -> Self {
        Self { identity, revision }
    }

    pub fn identity(self) -> Identity {
        self.identity
    }

    pub fn revision(self) -> u64 {
        self.revision
    }
}

impl SaveSnapshot {
    pub(super) fn new(version: Version, text: String) -> Self {
        Self { version, text }
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn write_to(&self, path: impl AsRef<Path>) -> io::Result<()> {
        write_atomic(path.as_ref(), self.text.as_bytes())
    }
}

fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    let (temporary_path, mut temporary) = create_temporary_sibling(path)?;
    let write_result = temporary
        .write_all(contents)
        .and_then(|()| temporary.sync_all());
    drop(temporary);

    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&temporary_path);
        return Err(error);
    }

    if let Err(error) = replace_file(&temporary_path, path) {
        let _ = std::fs::remove_file(&temporary_path);
        return Err(error);
    }

    Ok(())
}

fn create_temporary_sibling(path: &Path) -> io::Result<(PathBuf, File)> {
    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "save path must name a file"))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    for _ in 0..128 {
        let sequence = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
        let mut temporary_name = OsString::from(".");
        temporary_name.push(file_name);
        temporary_name.push(format!(
            ".wgpu_l3-save-{}-{sequence}.tmp",
            std::process::id()
        ));
        let temporary_path = parent.join(temporary_name);

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
        {
            Ok(file) => return Ok((temporary_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a temporary save file",
    ))
}

#[cfg(not(target_os = "windows"))]
fn replace_file(temporary: &Path, destination: &Path) -> io::Result<()> {
    std::fs::rename(temporary, destination)
}

#[cfg(target_os = "windows")]
fn replace_file(temporary: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let temporary = temporary
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let moved = unsafe {
        MoveFileExW(
            temporary.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };

    if moved == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
