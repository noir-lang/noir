use std::sync::LazyLock;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use ignore::{DirEntry, WalkBuilder};
use nargo::package::Package;

pub const SCARB_ENV: &str = "SCARB";
pub const MANIFEST_FILE_NAME: &str = "Nargo.toml";
pub const ORIGINAL_MANIFEST_FILE_NAME: &str = "Nargo.orig.toml";
pub const VCS_INFO_FILE_NAME: &str = "VCS.json";
pub const LOCK_FILE_NAME: &str = "Nargo.lock";
pub const DEFAULT_MODULE_MAIN_FILE: &str = "lib.cairo";
pub const DEFAULT_TESTS_PATH: &str = "tests";
pub const DEFAULT_TARGET_DIR_NAME: &str = "target";
pub const SCARB_IGNORE_FILE_NAME: &str = ".nargoignore";
pub const DEFAULT_README_FILE_NAME: &str = "README.md";
pub static DEFAULT_SOURCE_PATH: LazyLock<Utf8PathBuf> =
    LazyLock::new(|| ["src", "lib.cairo"].iter().collect());
pub const DEFAULT_LICENSE_FILE_NAME: &str = "LICENSE";

/// List all files relevant to building this package inside this source.
///
/// The basic assumption is that all files in the package directory are relevant for building this
/// package, provided that they potentially can be committed to the source directory. The following
/// rules hold:
/// * Look for any `.scarbignore`, `.gitignore` or `.ignore`-like files, using the [`ignore`] crate.
/// * Skip `.git` directory.
/// * Skip any subdirectories containing `Scarb.toml`.
/// * Skip `<root>/target` directory.
/// * Skip `Scarb.lock` file.
/// * Skip README and LICENSE files.
/// * **Skip `Scarb.toml` file, as users of this function may want to generate it themselves.**
/// * Symlinks within the package directory are followed, while symlinks outside are just skipped.
/// * Avoid crossing file system boundaries, because it can complicate our lives.
pub fn list_source_files(pkg: &Package) -> Result<Vec<Utf8PathBuf>> {
    let mut ret = Vec::new();
    push_worktree_files(pkg, &mut ret)
        .with_context(|| format!("failed to list source files in: {}", pkg.root_dir.to_str().unwrap()))?;
    Ok(ret)
}

fn push_worktree_files(pkg: &Package, ret: &mut Vec<Utf8PathBuf>) -> Result<()> {
    let filter = {
        let pkg = pkg.clone();
        // let readme = pkg.;
        // let license_file = pkg
        //     .manifest
        //     .metadata
        //     .license_file
        //     .clone()
        //     .unwrap_or_default();

        move |entry: &DirEntry| -> bool {
            let path = entry.path();
            let is_root = entry.depth() == 0;

            // Ignore symlinks pointing outside the package directory.
            if path.strip_prefix(&pkg.root_dir).is_err() {
                return false;
            };

            // Skip any subdirectories containing `Scarb.toml`.
            if !is_root && path.join(MANIFEST_FILE_NAME).exists() {
                return false;
            }

            // Skip `Scarb.toml`, `Scarb.lock` and `target` directory.
            if entry.depth() == 1
                && ({
                    let f = entry.file_name();
                    f == MANIFEST_FILE_NAME || f == LOCK_FILE_NAME || f == DEFAULT_TARGET_DIR_NAME
                })
            {
                return false;
            }

            // Skip README and LICENSE files
            // if path == readme || path == license_file {
            //     return false;
            // }

            true
        }
    };

    WalkBuilder::new(&pkg.root_dir)
        .follow_links(true)
        .standard_filters(true)
        .parents(false)
        .require_git(true)
        .same_file_system(true)
        .add_custom_ignore_filename(SCARB_IGNORE_FILE_NAME)
        .filter_entry(filter)
        .build()
        .try_for_each(|entry| {
            let entry = entry?;
            if !is_dir(&entry) {
                ret.push(entry.into_path().try_into()?);
            }
            Ok(())
        })
}

fn is_dir(entry: &DirEntry) -> bool {
    entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
}
