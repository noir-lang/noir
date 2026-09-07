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
        if fs2::FileExt::try_lock_exclusive(&file).is_err() {
            eprintln!("Waiting for lock on {lock_name}...");
            fs2::FileExt::lock_exclusive(&file)?;
        }

        Ok(Self { file })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Err(e) = fs2::FileExt::unlock(&self.file) {
            tracing::warn!("failed to release lock: {e:?}");
        }
    }
}
