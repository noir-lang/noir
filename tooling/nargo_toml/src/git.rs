use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::flock::FileLock;

/// The deepest a clone root can sit below the cache root, in path components. Dependencies are
/// downloaded to `<host>/<owner>/<name>/<tag>`, e.g. `github.com/owner/name/v1.0.0`, so a clone
/// root is always exactly 4 components deep.
const MAX_DEPENDENCY_CACHE_DEPTH: usize = 4;

/// Lists every git dependency currently present in the global download cache, as paths relative
/// to the cache root (e.g. `github.com/owner/name/v1.0.0`).
///
/// A directory is treated as a downloaded dependency when it contains a `.git` entry, which
/// `git clone` always creates. This is host-agnostic: it doesn't assume any particular server
/// or a fixed `owner/name` nesting depth.
pub fn list_cached_git_dependencies() -> BTreeSet<PathBuf> {
    collect_cached_git_dependencies(&nargo_crates())
}

/// Walks the dependency cache rooted at `cache_root`, returning the path (relative to `cache_root`)
/// of every directory that contains a `.git` entry. Such a directory is a clone root, so we do not
/// descend into it. Descent also stops at [`MAX_DEPENDENCY_CACHE_DEPTH`], which the contents of a
/// clone never reach.
fn collect_cached_git_dependencies(cache_root: &Path) -> BTreeSet<PathBuf> {
    fn go(cache_root: &Path, dir: &Path, depth: usize, found: &mut BTreeSet<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if path.join(".git").exists() {
                if let Ok(relative) = path.strip_prefix(cache_root) {
                    found.insert(relative.to_path_buf());
                }
            } else if depth + 1 < MAX_DEPENDENCY_CACHE_DEPTH {
                go(cache_root, &path, depth + 1, found);
            }
        }
    }

    let mut found = BTreeSet::new();
    go(cache_root, cache_root, 0, &mut found);
    found
}

/// Creates a unique folder name for a GitHub repo
/// by using its URL and tag
fn resolve_folder_name(base: &url::Url, tag: &str) -> String {
    let mut folder = PathBuf::from("");
    for part in [base.domain().unwrap(), base.path(), tag] {
        folder.push(part.trim_start_matches('/'));
    }
    folder.to_string_lossy().into_owned()
}

/// Path to the `nargo` directory under `$HOME`.
fn nargo_crates() -> PathBuf {
    dirs::home_dir().unwrap().join("nargo")
}

/// Target directory to download dependencies into, e.g.
/// `$HOME/nargo/github.com/noir-lang/noir-bignum/v0.1.2`
fn git_dep_location(base: &url::Url, tag: &str) -> PathBuf {
    let folder_name = resolve_folder_name(base, tag);

    nargo_crates().join(folder_name)
}

pub(crate) fn lock_git_deps() -> std::io::Result<FileLock> {
    FileLock::new(&nargo_crates().join(".package-cache"), "git dependencies cache")
}

/// XXX: I'd prefer to use a GitHub library however, there
/// does not seem to be an easy way to download a repo at a specific
/// tag
/// github-rs looks promising, however it seems to require an API token
///
/// One advantage of using "git clone" is that there is effectively no rate limit
pub(crate) fn clone_git_repo(url: &str, tag: &str) -> Result<PathBuf, String> {
    use std::process::Command;

    let base = match url::Url::parse(url) {
        Ok(base) => base,
        Err(err) => return Err(err.to_string()),
    };

    let loc = git_dep_location(&base, tag);
    if loc.exists() {
        return Ok(loc);
    }

    Command::new("git")
        .arg("-c")
        .arg("advice.detachedHead=false")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--branch")
        .arg(tag)
        .arg(base.as_str())
        .arg(&loc)
        .status()
        .expect("git clone command failed to start");

    Ok(loc)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    use test_case::test_case;
    use url::Url;

    use super::{collect_cached_git_dependencies, resolve_folder_name};

    #[test_case("https://github.com/noir-lang/noir-bignum/"; "with slash")]
    #[test_case("https://github.com/noir-lang/noir-bignum"; "without slash")]
    fn test_resolve_folder_name(url: &str) {
        let tag = "v0.4.2";
        let dir = resolve_folder_name(&Url::parse(url).unwrap(), tag);
        assert_eq!(dir, "github.com/noir-lang/noir-bignum/v0.4.2");
    }

    /// Creates a clone root by making the directory tree `relative` under `cache_root` and
    /// dropping a `.git` directory inside it, mirroring what `git clone` leaves behind.
    fn make_clone_root(cache_root: &Path, relative: &str) {
        let root = cache_root.join(relative);
        fs::create_dir_all(root.join(".git")).unwrap();
        // A real clone also has source files alongside `.git`; include one so the walker has to
        // stop at the clone root rather than descend into its contents.
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.nr"), "").unwrap();
    }

    #[test]
    fn lists_clone_roots_relative_to_cache_and_does_not_descend_into_them() {
        let cache = tempfile::tempdir().unwrap();
        let cache_root = cache.path();

        // Two tags of the same repo, plus a repo under a different host.
        make_clone_root(cache_root, "github.com/noir-lang/keccak256/v0.1.2");
        make_clone_root(cache_root, "github.com/noir-lang/keccak256/v0.1.3");
        make_clone_root(cache_root, "example.org/owner/name/v2.0.0");

        // A lock file at the cache root and an empty intermediate directory must be ignored.
        fs::write(cache_root.join(".package-cache"), "").unwrap();
        fs::create_dir_all(cache_root.join("github.com/noir-lang/not-downloaded-yet")).unwrap();

        let found = collect_cached_git_dependencies(cache_root);

        let expected: BTreeSet<PathBuf> = [
            PathBuf::from("github.com/noir-lang/keccak256/v0.1.2"),
            PathBuf::from("github.com/noir-lang/keccak256/v0.1.3"),
            PathBuf::from("example.org/owner/name/v2.0.0"),
        ]
        .into_iter()
        .collect();

        assert_eq!(found, expected);
    }

    #[test]
    fn does_not_report_clone_roots_below_the_depth_limit() {
        let cache = tempfile::tempdir().unwrap();
        let cache_root = cache.path();

        // Sits one level deeper than `MAX_DEPENDENCY_CACHE_DEPTH` allows, so it is not reported.
        make_clone_root(cache_root, "deep.org/group/subgroup/name/v1.0.0");

        let found = collect_cached_git_dependencies(cache_root);

        assert!(found.is_empty());
    }

    #[test]
    fn lists_nothing_when_cache_is_absent() {
        let cache = tempfile::tempdir().unwrap();
        let missing = cache.path().join("does-not-exist");

        let found = collect_cached_git_dependencies(&missing);

        assert!(found.is_empty());
    }
}
