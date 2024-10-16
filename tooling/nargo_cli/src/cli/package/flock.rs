use std::fs::{File, OpenOptions};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::{Arc, Weak};
use std::{fmt, io};

use anyhow::{ensure, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use fs4::tokio::AsyncFileExt;
use fs4::{lock_contended_error};
use fs4::fs_std::FileExt;
use tokio::sync::Mutex;
use nargo_fmt::Config;

pub const OK_FILE: &str = ".scarb-ok";

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FileLockKind {
    Shared,
    Exclusive,
}

#[derive(Debug)]
pub struct FileLockGuard {
    file: Option<File>,
    path: Utf8PathBuf,
    lock_kind: FileLockKind,
}

impl FileLockGuard {
    pub fn path(&self) -> &Utf8Path {
        self.path.as_path()
    }

    pub fn lock_kind(&self) -> FileLockKind {
        self.lock_kind
    }

    pub fn rename(&mut self, to: impl AsRef<Path>) -> Result<&mut Self> {
        ensure!(
            self.lock_kind == FileLockKind::Exclusive,
            "cannot rename shared file: {}",
            self.path,
        );
        // let to = to.as_ref().try_to_utf8()?;
        let to = to.as_ref().try_to_utf8()?;
        fsx::rename(&self.path, &to)?;
        self.path = to;
        Ok(self)
    }

    pub fn into_async(mut self) -> AsyncFileLockGuard {
        AsyncFileLockGuard {
            file: self.file.take().map(tokio::fs::File::from_std),
            path: std::mem::take(&mut self.path),
            lock_kind: self.lock_kind,
        }
    }
}

impl Deref for FileLockGuard {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        self.file.as_ref().unwrap()
    }
}

impl DerefMut for FileLockGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.file.as_mut().unwrap()
    }
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            let _ = file.unlock();
        }
    }
}

#[derive(Debug)]
pub struct AsyncFileLockGuard {
    file: Option<tokio::fs::File>,
    path: Utf8PathBuf,
    lock_kind: FileLockKind,
}

impl AsyncFileLockGuard {
    pub fn path(&self) -> &Utf8Path {
        self.path.as_path()
    }

    pub fn lock_kind(&self) -> FileLockKind {
        self.lock_kind
    }

    pub async fn into_sync(mut self) -> FileLockGuard {
        FileLockGuard {
            file: match self.file.take() {
                None => None,
                Some(file) => Some(file.into_std().await),
            },
            path: std::mem::take(&mut self.path),
            lock_kind: self.lock_kind,
        }
    }
}

impl Deref for AsyncFileLockGuard {
    type Target = tokio::fs::File;

    fn deref(&self) -> &Self::Target {
        self.file.as_ref().unwrap()
    }
}

impl DerefMut for AsyncFileLockGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.file.as_mut().unwrap()
    }
}

impl Drop for AsyncFileLockGuard {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            let _ = file.unlock();
        }
    }
}

/// An exclusive lock over a global entity identified by a path within a [`Filesystem`].
pub struct AdvisoryLock<'f> {
    path: Utf8PathBuf,
    description: String,
    file_lock: Mutex<
        // This Arc is shared between all guards within the process.
        // Here it is Weak, because AdvisoryLock itself does not keep the lock
        // (only guards do).
        Weak<FileLockGuard>,
    >,
    filesystem: &'f Filesystem,
    config: &'f Config,
}

pub struct AdvisoryLockGuard {
    _inner: Arc<FileLockGuard>,
}

impl<'f> AdvisoryLock<'f> {
    /// Acquires this advisory lock in an async manner.
    ///
    /// This lock is global per-process and can be acquired recursively.
    /// An RAII structure is returned to release the lock, and if this process abnormally
    /// terminates the lock is also released.
    pub async fn acquire_async(&self) -> Result<AdvisoryLockGuard> {
        let mut slot = self.file_lock.lock().await;

        let file_lock_arc = match slot.upgrade() {
            Some(arc) => arc,
            None => {
                let arc = Arc::new(self.filesystem.create_rw(
                    &self.path,
                    &self.description,
                )?);
                *slot = Arc::downgrade(&arc);
                arc
            }
        };
        Ok(AdvisoryLockGuard {
            _inner: file_lock_arc,
        })
    }
}

/// A [`Filesystem`] is intended to be a globally shared, hence locked, resource in Scarb.
///
/// The [`Utf8Path`] of a file system cannot be learned unless it's done in a locked fashion,
/// and otherwise functions on this structure are prepared to handle concurrent invocations across
/// multiple instances of Scarb and its extensions.
///
/// All paths within a [`Filesystem`] must be UTF-8 encoded.
#[derive(Clone)]
pub struct Filesystem {
    root: Arc<LazyDirectoryCreator>,
}

impl Filesystem {
    /// Creates a new [`Filesystem`] to be rooted at the given path.
    pub fn new(root: Utf8PathBuf) -> Self {
        Self {
            root: LazyDirectoryCreator::new(root, false),
        }
    }

    /// Creates a new [`Filesystem`] to be rooted at the given path.
    ///
    /// This variant uses [`create_output_dir::create_output_dir`] function to create root
    /// directory.
    pub fn new_output_dir(root: Utf8PathBuf) -> Self {
        Self {
            root: LazyDirectoryCreator::new(root, true),
        }
    }

    /// Like [`Utf8Path::join`], creates a new [`Filesystem`] rooted at a subdirectory of this one.
    pub fn child(&self, path: impl AsRef<Utf8Path>) -> Filesystem {
        Filesystem {
            root: self.root.child(path),
        }
    }

    /// Like [`Utf8Path::join`], creates a new [`Filesystem`] rooted at a subdirectory of this one.
    ///
    /// Unlike [`Filesystem::child`], this method consumes the current [`Filesystem`].
    pub fn into_child(self, path: impl AsRef<Utf8Path>) -> Filesystem {
        Filesystem {
            root: self.root.into_child(path),
        }
    }

    /// Get path to this [`Filesystem`] root without ensuring the path exists.
    pub fn path_unchecked(&self) -> &Utf8Path {
        self.root.as_unchecked()
    }

    /// Get path to this [`Filesystem`] root, ensuring the path exists.
    pub fn path_existent(&self) -> Result<&Utf8Path> {
        self.root.as_existent()
    }

    /// Returns `true` if this [`Filesystem`] already exists on the disk.
    pub fn exists(&self) -> bool {
        self.path_unchecked().exists()
    }

    /// Opens exclusive access to a [`File`], returning the locked version of it.
    ///
    /// This function will create a file at `path` if it doesn't already exist (including
    /// intermediate directories) else if it does exist, it will be truncated. It will then acquire
    /// an exclusive lock on `path`. If the process must block waiting for the lock, the
    /// `description` annotated with _blocking_ status message is printed to [`Config::ui`].
    ///
    /// The returned file can be accessed to look at the path and also has read/write access to
    /// the underlying file.
    pub fn create_rw(
        &self,
        path: impl AsRef<Utf8Path>,
        description: &str,
    ) -> Result<FileLockGuard> {
        self.open(
            path.as_ref(),
            OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .create(true),
            FileLockKind::Exclusive,
            description,
        )
    }

    /// Opens shared access to a [`File`], returning the locked version of it.
    ///
    /// This function will fail if `path` doesn't already exist, but if it does then it will
    /// acquire a shared lock on `path`.
    /// If the process must block waiting for the lock, the `description` annotated with _blocking_
    /// status message is printed to [`Config::ui`].
    ///
    /// The returned file can be accessed to look at the path and also has read
    /// access to the underlying file.
    /// Any writes to the file will return an error.
    pub fn open_ro(
        &self,
        path: impl AsRef<Utf8Path>,
        description: &str,
    ) -> Result<FileLockGuard> {
        self.open(
            path.as_ref(),
            OpenOptions::new().read(true),
            FileLockKind::Shared,
            description,
        )
    }

    fn open(
        &self,
        path: &Utf8Path,
        opts: &OpenOptions,
        lock_kind: FileLockKind,
        description: &str,
    ) -> Result<FileLockGuard> {
        let path = self.root.as_existent()?.join(path);

        let file = opts
            .open(&path)
            .with_context(|| format!("failed to open: {path}"))?;

        match lock_kind {
            FileLockKind::Exclusive => {
                acquire(
                    &file,
                    &path,
                    description,
                    &FileExt::try_lock_exclusive,
                    &FileExt::lock_exclusive,
                )?;
            }
            FileLockKind::Shared => {
                acquire(
                    &file,
                    &path,
                    description,
                    &FileExt::try_lock_shared,
                    &FileExt::lock_shared,
                )?;
            }
        }

        Ok(FileLockGuard {
            file: Some(file),
            path,
            lock_kind,
        })
    }

    /// Construct an [`AdvisoryLock`] within this file system.
    pub fn advisory_lock<'a>(
        &'a self,
        path: impl AsRef<Utf8Path>,
        description: impl ToString,
        config: &'a Config,
    ) -> AdvisoryLock<'a> {
        AdvisoryLock {
            path: path.as_ref().to_path_buf(),
            description: description.to_string(),
            file_lock: Mutex::new(Weak::new()),
            filesystem: self,
            config,
        }
    }

    /// Remove the directory underlying this filesystem and create it again.
    ///
    /// # Safety
    /// This is very simple internal method meant to be used in very specific use-cases, so its
    /// implementation does not handle all cases.
    /// 1. Panics if this is an output filesystem.
    /// 2. Child filesystems will stop working properly after recreation.
    pub(crate) fn recreate(&self) -> Result<()> {
        if self.root.is_output_dir() {
            panic!("cannot recreate output filesystems");
        }

        let path = self.root.as_unchecked();
        if path.exists() {
            fsx::remove_dir_all(path)?;
        }
        fsx::create_dir_all(path)?;
        Ok(())
    }

    /// Checks if this filesystem has a valid `.scarb-ok` file.
    pub fn is_ok(&self) -> bool {
        self.path_unchecked().join(OK_FILE).exists()
    }

    /// Marks this filesystem as being properly set up (whatever this means is up to user),
    /// by creating a `.scarb-ok` file.
    pub fn mark_ok(&self) -> Result<()> {
        let _ = fsx::create(self.path_existent()?.join(OK_FILE))?;
        Ok(())
    }
}

impl fmt::Display for Filesystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root)
    }
}

impl fmt::Debug for Filesystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Filesystem")
            .field(self.root.deref())
            .finish()
    }
}

/// The following sequence of if statements & advisory locks implements a file system-based
/// mutex, that synchronizes extraction logic. The first condition checks if extraction has
/// happened in the past. If not, then we acquire the advisory lock (which means waiting for
/// our time slice to do the job). Successful lock acquisition does not mean though that we
/// still have to perform the extraction! While we waited for our time slice, another process
/// could just do the extraction! The second condition prevents repeating the work.
///
/// This is actually very important for correctness. The another process that performed
/// the extraction, will highly probably soon try to read the extracted files. If we recreate
/// the filesystem now, we will cause that process to crash. That's what happened on Windows
/// in examples tests, when the second condition was missing.
macro_rules! protected_run_if_not_ok {
    ($fs:expr, $lock:expr, $body:block) => {{
        let fs: &$crate::flock::Filesystem = $fs;
        let lock: &$crate::flock::AdvisoryLock<'_> = $lock;
        if !fs.is_ok() {
            let _lock = lock.acquire_async().await?;
            if !fs.is_ok() {
                $body
                fs.mark_ok()?;
            }
        }
    }};
}

pub(crate) use protected_run_if_not_ok;
use crate::cli::internal::fsx;
use crate::cli::internal::fsx::PathUtf8Ext;
use crate::cli::package::lazy_directory_creator::LazyDirectoryCreator;

fn acquire(
    file: &File,
    path: &Utf8Path,
    description: &str,
    lock_try: &dyn Fn(&File) -> io::Result<()>,
    lock_block: &dyn Fn(&File) -> io::Result<()>,
) -> Result<()> {
    match lock_try(file) {
        Ok(()) => return Ok(()),
        Err(err) if err.kind() == io::ErrorKind::Unsupported => {
            // Ignore locking on filesystems that look like they don't implement file locking.
            return Ok(());
        }
        Err(err) if is_lock_contended_error(&err) => {
            // Pass-through
        }
        Err(err) => {
            Err(err).with_context(|| format!("failed to lock file: {path}"))?;
        }
    }

    println!("\x1b[36mBlocking\x1b[0m: waiting for file lock on {}", description);

    lock_block(file).with_context(|| format!("failed to lock file: {path}"))?;

    Ok(())
}

fn is_lock_contended_error(err: &io::Error) -> bool {
    let t = lock_contended_error();
    err.raw_os_error() == t.raw_os_error() || err.kind() == t.kind()
}
