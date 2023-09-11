use std::path::PathBuf;

/// Creates a unique folder name for a GitHub repo
/// by using its URL and tag
fn resolve_folder_name(base: &url::Url, tag: &str) -> String {
    let mut folder_name = base.domain().unwrap().to_owned();
    folder_name.push_str(base.path());
    folder_name.push_str(tag);
    folder_name
}

fn nargo_crates() -> PathBuf {
    dirs::home_dir().unwrap().join("nargo")
}

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
