//! Mostly [`fs`] extensions with extra error messaging.

use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};

/// Equivalent to [`fs::canonicalize`] with better error messages.
///
/// Uses [`dunce`] to generate more familiar paths on Windows.
pub fn canonicalize(p: impl AsRef<Path>) -> Result<PathBuf> {
    return inner(p.as_ref());

    fn inner(p: &Path) -> Result<PathBuf> {
        dunce::canonicalize(p)
            .with_context(|| format!("failed to get absolute path of `{}`", p.display()))
    }
}

/// Equivalent to [`fs::canonicalize`], but for Utf-8 paths, with better error messages.
pub fn canonicalize_utf8(p: impl AsRef<Path>) -> Result<Utf8PathBuf> {
    canonicalize(p)?.try_into_utf8()
}

/// Equivalent to [`fs::create_dir_all`] with better error messages.
pub fn create_dir_all(p: impl AsRef<Path>) -> Result<()> {
    return inner(p.as_ref());

    fn inner(p: &Path) -> Result<()> {
        fs::create_dir_all(p)
            .with_context(|| format!("failed to create directory `{}`", p.display()))?;
        Ok(())
    }
}

/// Equivalent to [`fs::remove_file`] with better error messages.
pub fn remove_file(p: impl AsRef<Path>) -> Result<()> {
    return inner(p.as_ref());

    fn inner(p: &Path) -> Result<()> {
        fs::remove_file(p).with_context(|| format!("failed to remove file `{}`", p.display()))?;
        Ok(())
    }
}

/// Equivalent to [`fs::remove_dir_all`] with better error messages.
pub fn remove_dir_all(p: impl AsRef<Path>) -> Result<()> {
    return inner(p.as_ref());

    fn inner(p: &Path) -> Result<()> {
        fs::remove_dir_all(p)
            .with_context(|| format!("failed to remove directory `{}`", p.display()))?;
        Ok(())
    }
}

/// Equivalent to [`fs::write`] with better error messages.
pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    return inner(path.as_ref(), contents.as_ref());

    fn inner(path: &Path, contents: &[u8]) -> Result<()> {
        fs::write(path, contents).with_context(|| format!("failed to write `{}`", path.display()))
    }
}

/// Equivalent to [`File::create`] with better error messages.
pub fn create(path: impl AsRef<Path>) -> Result<File> {
    return inner(path.as_ref());

    fn inner(path: &Path) -> Result<File> {
        File::create(path).with_context(|| format!("failed to create `{}`", path.display()))
    }
}

/// Equivalent to [`fs::read`] with better error messages.
pub fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    return inner(path.as_ref());

    fn inner(path: &Path) -> Result<Vec<u8>> {
        fs::read(path).with_context(|| format!("failed to read `{}`", path.display()))
    }
}

/// Equivalent to [`fs::read_to_string`] with better error messages.
pub fn read_to_string(path: impl AsRef<Path>) -> Result<String> {
    return inner(path.as_ref());

    fn inner(path: &Path) -> Result<String> {
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))
    }
}

/// Equivalent to [`fs::rename`] with better error messages.
pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    return inner(from.as_ref(), to.as_ref());

    fn inner(from: &Path, to: &Path) -> Result<()> {
        fs::rename(from, to).with_context(|| format!("failed to rename file: {}", from.display()))
    }
}

/// Equivalent to [`fs::copy`] with better error messages.
pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    return inner(from.as_ref(), to.as_ref());

    fn inner(from: &Path, to: &Path) -> Result<u64> {
        fs::copy(from, to)
            .with_context(|| format!("failed to copy file {} to {}", from.display(), to.display()))
    }
}

#[cfg(unix)]
pub fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    use std::os::unix::prelude::*;
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
pub fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

#[cfg(unix)]
pub fn is_hidden(entry: impl AsRef<Path>) -> bool {
    is_hidden_by_dot(entry)
}

#[cfg(windows)]
pub fn is_hidden(entry: impl AsRef<Path>) -> bool {
    use std::os::windows::prelude::*;

    let is_hidden = fs::metadata(entry.as_ref())
        .ok()
        .map(|metadata| metadata.file_attributes())
        .map(|attributes| (attributes & 0x2) > 0)
        .unwrap_or(false);

    is_hidden || is_hidden_by_dot(entry)
}

fn is_hidden_by_dot(entry: impl AsRef<Path>) -> bool {
    entry
        .as_ref()
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

pub trait PathUtf8Ext {
    fn try_as_utf8(&'_ self) -> Result<&'_ Utf8Path>;

    fn try_to_utf8(&self) -> Result<Utf8PathBuf> {
        self.try_as_utf8().map(|p| p.to_path_buf())
    }
}

pub trait PathBufUtf8Ext {
    fn try_into_utf8(self) -> Result<Utf8PathBuf>;
}

impl PathUtf8Ext for Path {
    fn try_as_utf8(&'_ self) -> Result<&'_ Utf8Path> {
        Utf8Path::from_path(self)
            .ok_or_else(|| anyhow!("path `{}` is not UTF-8 encoded", self.display()))
    }
}

impl PathUtf8Ext for PathBuf {
    fn try_as_utf8(&'_ self) -> Result<&'_ Utf8Path> {
        self.as_path().try_as_utf8()
    }
}

impl PathBufUtf8Ext for PathBuf {
    fn try_into_utf8(self) -> Result<Utf8PathBuf> {
        Utf8PathBuf::from_path_buf(self)
            .map_err(|path| anyhow!("path `{}` is not UTF-8 encoded", path.display()))
    }
}
