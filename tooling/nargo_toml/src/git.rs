use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use crate::errors::GitError;
use crate::flock::FileLock;

/// The deepest a clone root can sit below the cache root, in path components. Dependencies are
/// downloaded to `<host>/<owner>/<name>/<tag>`, e.g. `github.com/owner/name/v1.0.0`, so a clone
/// root is always exactly 4 components deep.
const MAX_DEPENDENCY_CACHE_DEPTH: usize = 4;

/// Directory under the cache root that `git clone` writes into. It is only moved to its final
/// `<host>/<owner>/<name>/<tag>` location once the clone has succeeded, so an interrupted clone can
/// never be mistaken for a downloaded dependency. Hostnames cannot start with `.`, so this can't
/// collide with a dependency's host directory.
const CLONE_STAGING_DIR: &str = ".clone-staging";

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
            if !path.is_dir() || (depth == 0 && entry.file_name() == CLONE_STAGING_DIR) {
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

/// Creates a unique folder name for a git repository by using its URL and tag.
///
/// The host (a domain like `github.com`, or an IP for a self-hosted server) is used as-is, so
/// repositories on the same host share a parent directory in the cache.
///
/// The result is always a relative path made of plain components, so it cannot point outside the
/// cache: a tag such as `../../x` is rejected. Git never accepts such a tag name anyway.
fn resolve_folder_name(base: &url::Url, tag: &str) -> Result<PathBuf, GitError> {
    let host = base.host_str().ok_or_else(|| GitError::MissingHost { url: base.to_string() })?;
    let mut folder = PathBuf::from("");
    for part in [host, base.path(), tag] {
        folder.push(part.trim_start_matches('/'));
    }
    if !folder.components().all(|component| matches!(component, Component::Normal(_))) {
        return Err(GitError::InvalidTag { url: base.to_string(), tag: tag.to_string() });
    }
    Ok(folder)
}

/// Path to the `nargo` directory under `$HOME`.
fn nargo_crates() -> PathBuf {
    dirs::home_dir().unwrap().join("nargo")
}

pub(crate) fn lock_git_deps() -> Result<FileLock, GitError> {
    FileLock::new(&nargo_crates().join(".package-cache"), "git dependencies cache")
        .map_err(GitError::Lock)
}

/// Downloads the repository at `url` checked out at `tag` into the global dependency cache,
/// returning the directory it was downloaded to, e.g.
/// `$HOME/nargo/github.com/noir-lang/noir-bignum/v0.1.2`. Does nothing if it is already cached.
///
/// The caller must hold the lock returned by [`lock_git_deps`].
///
/// XXX: I'd prefer to use a GitHub library however, there
/// does not seem to be an easy way to download a repo at a specific
/// tag
/// github-rs looks promising, however it seems to require an API token
///
/// One advantage of using "git clone" is that there is effectively no rate limit
pub(crate) fn clone_git_repo(url: &str, tag: &str) -> Result<PathBuf, GitError> {
    clone_git_repo_into(&nargo_crates(), url, tag)
}

fn clone_git_repo_into(cache_root: &Path, url: &str, tag: &str) -> Result<PathBuf, GitError> {
    let base = url::Url::parse(url)
        .map_err(|source| GitError::InvalidUrl { url: url.to_string(), source })?;
    let loc = cache_root.join(resolve_folder_name(&base, tag)?);

    // `.git` marks a complete download, as in `collect_cached_git_dependencies`. Anything else at
    // `loc` is left over from an interrupted download and is replaced.
    if loc.join(".git").exists() {
        return Ok(loc);
    }
    remove_dir_if_exists(&loc)?;

    // A previous clone may have been killed before it could be moved into place.
    let staging = cache_root.join(CLONE_STAGING_DIR);
    remove_dir_if_exists(&staging)?;
    if let Some(parent) = loc.parent() {
        std::fs::create_dir_all(parent).map_err(|source| cache_error(parent, source))?;
    }

    let output = Command::new("git")
        .arg("-c")
        .arg("advice.detachedHead=false")
        .arg("clone")
        .arg("--quiet")
        .arg("--depth")
        .arg("1")
        .arg("--branch")
        .arg(tag)
        .arg(base.as_str())
        .arg(&staging)
        // stdin and stdout are the JSON-RPC channel when running as a language server, so git must
        // not touch them. Credential prompts still work as git reads those from the terminal.
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|source| {
            let (url, tag) = (url.to_string(), tag.to_string());
            if source.kind() == std::io::ErrorKind::NotFound {
                GitError::GitNotFound { url, tag }
            } else {
                GitError::SpawnFailed { url, tag, source }
            }
        })?;

    if !output.status.success() {
        return Err(GitError::CloneFailed {
            url: url.to_string(),
            tag: tag.to_string(),
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    std::fs::rename(&staging, &loc).map_err(|source| cache_error(&loc, source))?;

    Ok(loc)
}

fn remove_dir_if_exists(path: &Path) -> Result<(), GitError> {
    match std::fs::remove_dir_all(path) {
        Err(err) if err.kind() != std::io::ErrorKind::NotFound => Err(cache_error(path, err)),
        _ => Ok(()),
    }
}

fn cache_error(path: &Path, source: std::io::Error) -> GitError {
    GitError::Cache { path: path.to_path_buf(), source }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use test_case::test_case;
    use url::Url;

    use super::{
        CLONE_STAGING_DIR, clone_git_repo_into, collect_cached_git_dependencies,
        resolve_folder_name,
    };
    use crate::errors::GitError;

    #[test_case("https://github.com/noir-lang/noir-bignum/"; "with slash")]
    #[test_case("https://github.com/noir-lang/noir-bignum"; "without slash")]
    fn test_resolve_folder_name(url: &str) {
        let tag = "v0.4.2";
        let dir = resolve_folder_name(&Url::parse(url).unwrap(), tag).unwrap();
        assert_eq!(dir, Path::new("github.com/noir-lang/noir-bignum/v0.4.2"));
    }

    /// A self-hosted repository can be served from an IP address, which has no registrable domain.
    #[test]
    fn test_resolve_folder_name_with_ip_host() {
        let dir =
            resolve_folder_name(&Url::parse("https://192.168.1.10/me/repo").unwrap(), "v1.0.0")
                .unwrap();
        assert_eq!(dir, Path::new("192.168.1.10/me/repo/v1.0.0"));
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

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(["-c", "user.name=test", "-c", "user.email=test@example.com"])
            .args(args)
            .current_dir(dir)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }

    /// Creates a repository under `dir` with a single commit tagged `v1.0.0`, returning a URL for
    /// it. Dependency URLs must have a host, so the URL names one; git ignores it for `file://`.
    /// (`localhost` would not do, as URL parsing drops it from `file://` URLs.)
    fn make_repository(dir: &Path) -> String {
        let repo = dir.join("repo");
        fs::create_dir(&repo).unwrap();
        git(&repo, &["init", "--quiet"]);
        fs::write(repo.join("Nargo.toml"), "").unwrap();
        git(&repo, &["add", "Nargo.toml"]);
        git(&repo, &["commit", "--quiet", "-m", "initial"]);
        git(&repo, &["tag", "v1.0.0"]);
        format!("file://127.0.0.1{}", repo.display())
    }

    #[test]
    fn clones_a_tag_into_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let url = make_repository(dir.path());
        let cache_root = dir.path().join("cache");

        let loc = clone_git_repo_into(&cache_root, &url, "v1.0.0").unwrap();

        assert!(loc.starts_with(&cache_root));
        assert!(loc.join("Nargo.toml").exists());
        assert!(!cache_root.join(CLONE_STAGING_DIR).exists());
        // A second call is served from the cache.
        assert_eq!(clone_git_repo_into(&cache_root, &url, "v1.0.0").unwrap(), loc);
    }

    #[test]
    fn failed_clone_reports_the_dependency_and_git_output() {
        let dir = tempfile::tempdir().unwrap();
        let url = make_repository(dir.path());
        let cache_root = dir.path().join("cache");

        let error = clone_git_repo_into(&cache_root, &url, "v9.9.9").unwrap_err();

        assert!(matches!(error, GitError::CloneFailed { .. }), "{error:?}");
        let message = error.to_string();
        assert!(message.contains(&url), "{message}");
        assert!(message.contains("`v9.9.9`"), "{message}");
        assert!(message.contains("v9.9.9 not found"), "{message}");
        assert!(collect_cached_git_dependencies(&cache_root).is_empty());
    }

    #[test]
    fn replaces_a_partial_download_left_in_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let url = make_repository(dir.path());
        let cache_root = dir.path().join("cache");
        let url_parsed = Url::parse(&url).unwrap();
        let loc = cache_root.join(resolve_folder_name(&url_parsed, "v1.0.0").unwrap());
        fs::create_dir_all(&loc).unwrap();
        fs::write(loc.join("partial"), "").unwrap();
        fs::create_dir_all(cache_root.join(CLONE_STAGING_DIR).join(".git")).unwrap();

        assert!(collect_cached_git_dependencies(&cache_root).is_empty());

        assert_eq!(clone_git_repo_into(&cache_root, &url, "v1.0.0").unwrap(), loc);
        assert!(loc.join("Nargo.toml").exists());
        assert!(!loc.join("partial").exists());
    }

    #[test_case("../../../../../victim"; "parent directory")]
    #[test_case("v1/../../../../../../victim"; "nested parent directory")]
    fn rejects_tags_that_escape_the_cache(tag: &str) {
        let dir = tempfile::tempdir().unwrap();
        let victim = dir.path().join("victim");
        fs::create_dir(&victim).unwrap();
        let cache_root = dir.path().join("cache");

        let error =
            clone_git_repo_into(&cache_root, "https://example.com/owner/name", tag).unwrap_err();

        assert!(matches!(error, GitError::InvalidTag { .. }), "{error:?}");
        assert!(victim.exists());
    }

    #[test]
    fn rejects_urls_without_a_host() {
        let dir = tempfile::tempdir().unwrap();

        let error = clone_git_repo_into(dir.path(), "file:///owner/name", "v1.0.0").unwrap_err();

        assert!(matches!(error, GitError::MissingHost { .. }), "{error:?}");
    }
}
