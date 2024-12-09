use std::path::PathBuf;

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
    use test_case::test_case;
    use url::Url;

    use super::resolve_folder_name;

    #[test_case("https://github.com/noir-lang/noir-bignum/"; "with slash")]
    #[test_case("https://github.com/noir-lang/noir-bignum"; "without slash")]
    fn test_resolve_folder_name(url: &str) {
        let tag = "v0.4.2";
        let dir = resolve_folder_name(&Url::parse(url).unwrap(), tag);
        assert_eq!(dir, "github.com/noir-lang/noir-bignum/v0.4.2");
    }
}
