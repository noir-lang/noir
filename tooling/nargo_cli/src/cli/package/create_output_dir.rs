//! This crate provides only one function, `create_output_dir` which creates an excluded from cache
//! directory atomically with its parents as needed.
//!
//! The source code of this crate has been almost verbatim copy-pasted from
//! [`cargo_util::paths::create_dir_all_excluded_from_backups_atomic`][cargo-util-fn].
//!
//! [cargo-util-fn]: https://docs.rs/cargo-util/latest/cargo_util/paths/fn.create_dir_all_excluded_from_backups_atomic.html

use std::ffi::OsStr;
use std::path::Path;
use std::{env, fs};

use anyhow::{Context, Result};

/// Creates an excluded from cache directory atomically with its parents as needed.
///
/// The atomicity only covers creating the leaf directory and exclusion from cache. Any missing
/// parent directories will not be created in an atomic manner.
///
/// This function is idempotent and in addition to that it won't exclude `path` from cache if it
/// already exists.
pub fn create_output_dir(path: &Path) -> Result<()> {
    if path.is_dir() {
        return Ok(());
    }

    let parent = path.parent().unwrap();
    let base = path.file_name().unwrap();
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory `{}`", parent.display()))?;

    // We do this in two steps: first create a temporary directory and exclude it from backups,
    // then rename it to the desired name.
    // If we created the directory directly where it should be and then excluded it from backups
    // we would risk a situation where the application is interrupted right after the directory
    // creation, but before the exclusion, and the directory would remain non-excluded from backups.
    //
    // We need a temporary directory created in `parent` instead of `$TMP`, because only then we
    // can be easily sure that `fs::rename()` will succeed (the new name needs to be on the same
    // mount point as the old one).
    let tempdir = tempfile::Builder::new().prefix(base).tempdir_in(parent)?;
    exclude_from_backups(tempdir.path());
    exclude_from_content_indexing(tempdir.path());

    // Previously `fs::create_dir_all()` was used here to create the directory directly and
    // `fs::create_dir_all()` explicitly treats the directory being created concurrently by another
    // thread or process as success, hence the check below to follow the existing behavior.
    // If we get an error at `fs::rename()` and suddenly the directory (which didn't exist a moment
    // earlier) exists we can infer from it that another application process is doing work here.
    if let Err(e) = fs::rename(tempdir.path(), path) {
        if !path.exists() {
            return Err(e.into());
        }
    }

    Ok(())
}

/// Marks the directory as excluded from archives/backups.
///
/// This is recommended to prevent derived/temporary files from bloating backups.
/// There are two mechanisms used to achieve this right now:
/// * A dedicated resource property excluding from Time Machine backups on macOS.
/// * `CACHEDIR.TAG` files supported by various tools in a platform-independent way.
fn exclude_from_backups(path: &Path) {
    exclude_from_time_machine(path);
    let _ = fs::write(
        path.join("CACHEDIR.TAG"),
        format!(
            "Signature: 8a477f597d28d172789f06886806bc55
# This file is a cache directory tag{}.
# For information about cache directory tags see https://bford.info/cachedir/
",
            match guess_application_name() {
                None => String::new(),
                Some(name) => format!(" created by {name}"),
            }
        ),
    );
    // Similarly to exclude_from_time_machine() we ignore errors here as it's an optional feature.
}

/// Marks the directory as excluded from content indexing.
///
/// This is recommended to prevent the content of derived/temporary files from being indexed.
/// This is very important for Windows users, as the live content indexing may significantly slow
/// I/O operations or compilers etc.
///
/// This is currently a no-op on non-Windows platforms.
fn exclude_from_content_indexing(path: &Path) {
    #[cfg(windows)]
    {
        use std::iter::once;
        use std::os::windows::prelude::OsStrExt;
        use winapi::um::fileapi::{GetFileAttributesW, SetFileAttributesW};
        use winapi::um::winnt::FILE_ATTRIBUTE_NOT_CONTENT_INDEXED;

        let path: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
        unsafe {
            SetFileAttributesW(
                path.as_ptr(),
                GetFileAttributesW(path.as_ptr()) | FILE_ATTRIBUTE_NOT_CONTENT_INDEXED,
            );
        }
    }
    #[cfg(not(windows))]
    {
        let _ = path;
    }
}

#[cfg(not(target_os = "macos"))]
fn exclude_from_time_machine(_: &Path) {}

/// Marks files or directories as excluded from Time Machine on macOS.
#[cfg(target_os = "macos")]
fn exclude_from_time_machine(path: &Path) {
    use core_foundation::base::TCFType;
    use core_foundation::{number, string, url};
    use std::ptr;

    // For compatibility with 10.7 a string is used instead of global kCFURLIsExcludedFromBackupKey
    let is_excluded_key: std::result::Result<string::CFString, _> =
        "NSURLIsExcludedFromBackupKey".parse();
    let path = url::CFURL::from_path(path, false);
    if let (Some(path), Ok(is_excluded_key)) = (path, is_excluded_key) {
        unsafe {
            url::CFURLSetResourcePropertyForKey(
                path.as_concrete_TypeRef(),
                is_excluded_key.as_concrete_TypeRef(),
                number::kCFBooleanTrue as *const _,
                ptr::null_mut(),
            );
        }
    }
    // Errors are ignored, since it's an optional feature and failure
    // shouldn't prevent applications from working.
}

fn guess_application_name() -> Option<String> {
    let exe = env::current_exe().ok()?;
    let file_name = if exe.extension() == Some(OsStr::new(env::consts::EXE_EXTENSION)) {
        exe.file_stem()
    } else {
        exe.file_name()
    }?;
    Some(file_name.to_string_lossy().into_owned())
}
