#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

use std::{
    fs::{File, OpenOptions},
    path::Path,
};

/// A cross-platform file lock that ensures exclusive access to a resource.
///
/// The lock is automatically released when the `FileLock` is dropped.
///
/// # Example
///
/// ```rust,no_run
/// use file_utils::FileLock;
/// use std::path::Path;
///
/// let lock = FileLock::new(Path::new("/tmp/mylock"), "my resource")?;
/// // Critical section - only one process can be here at a time
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct FileLock {
    file: File,
}

impl FileLock {
    /// Create a new file lock at the specified path.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the lock file
    /// * `lock_name` - Human-readable name for the lock (used in logging)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The parent directory cannot be created
    /// - The lock file cannot be opened or created
    /// - The lock cannot be acquired
    pub fn new(file_path: &Path, lock_name: &str) -> std::io::Result<Self> {
        std::fs::create_dir_all(
            file_path.parent().expect("can't create lock on filesystem root"),
        )?;
        let file = OpenOptions::new().create(true).truncate(false).write(true).open(file_path)?;
        if fs2::FileExt::try_lock_exclusive(&file).is_err() {
            eprintln!("Waiting for lock on {lock_name}...");
        }

        fs2::FileExt::lock_exclusive(&file)?;

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

