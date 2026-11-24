const GIT_COMMIT: &&str = &"GIT_COMMIT";

fn main() -> Result<(), String> {
    set_git_info()
}

/// Sets `GIT_COMMIT` and `GIT_DIRTY` for `env!` from the git checkout. A build without a checkout
/// sets both variables in its environment instead, and then git is not consulted.
fn set_git_info() -> Result<(), String> {
    if std::env::var(GIT_COMMIT).is_ok() {
        return Ok(());
    }
    let commit = git(&["rev-parse", "HEAD"])?;
    let dirty = !git(&["status", "--short"])?.is_empty();
    println!("cargo:rustc-env=GIT_COMMIT={commit}");
    println!("cargo:rustc-env=GIT_DIRTY={dirty}");
    Ok(())
}

fn git(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .output()
        .map_err(|err| format!("failed to run `git {}`: {err}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
