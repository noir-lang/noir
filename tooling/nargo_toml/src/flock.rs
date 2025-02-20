use fs2::FileExt;
use std::{
    fs::{File, OpenOptions},
    path::Path,
};

// TODO: move this to some utils crate.

pub(crate) struct FileLock {
    file: File,
}

impl FileLock {
    pub(crate) fn new(file_path: &Path, lock_name: &str) -> std::io::Result<Self> {
        std::fs::create_dir_all(file_path.parent().expect("can't create lock on filesystem root"))?;
        let file = OpenOptions::new().create(true).truncate(false).write(true).open(file_path)?;
        if file.try_lock_exclusive().is_err() {
            eprintln!("Waiting for lock on {lock_name}...");
        }

        file.lock_exclusive()?;

        Ok(Self { file })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Err(e) = self.file.unlock() {
            tracing::warn!("failed to release lock: {e:?}");
        }
    }
}
