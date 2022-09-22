use std::path::PathBuf;

pub fn git_dep_location(base: &url::Url, tag: &str) -> PathBuf {
    let folder_name = super::resolver::resolve_folder_name(base, tag);

    super::nargo_crates().join(folder_name)
}

/// XXX: I'd prefer to use a GitHub library however, there
/// does not seem to be an easy way to download a repo at a specific
/// tag
/// github-rs looks promising, however it seems to require an API token
///
/// One advantage of using "git clone" is that there is effectively no rate limit
pub fn clone_git_repo(url: &str, tag: &str) -> Result<PathBuf, String> {
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
