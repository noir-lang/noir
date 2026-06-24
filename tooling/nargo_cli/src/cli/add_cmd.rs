use std::str::FromStr;

use clap::{ArgGroup, Args};
use nargo::constants::PKG_FILE;
use nargo::package::CrateName;
use nargo::workspace::Workspace;
use nargo_toml::{
    DependencyConfig, PackageSelection, add_dependency_to_manifest, resolve_dependency,
};
use thiserror::Error;
use url::Url;

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand};

/// URL of a git repository for a dependency, validated as it is parsed from the command line.
///
/// Only `https` URLs are accepted. A repository's web page offers an HTTPS and an SSH URL to copy;
/// the SSH one is the scp-style `git@github.com:owner/repo.git`, which is not a URL and cannot be
/// cached by host and path, so it is rejected with guidance to use the HTTPS form. `http` is
/// rejected so credentials are never sent unencrypted.
#[derive(Debug, Clone)]
pub(crate) struct GitUrl(String);

impl GitUrl {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub(crate) enum GitUrlParseError {
    #[error(
        "`{input}` is not a valid git URL: {source}.\n\
         Use the HTTPS URL from the repository's web page, e.g. `https://github.com/owner/repo`.\n\
         SSH URLs like `git@github.com:owner/repo.git` are not supported."
    )]
    InvalidUrl { input: String, source: url::ParseError },
    #[error(
        "git dependencies must use an `https` URL, but `{input}` uses `{scheme}`.\n\
         Use the HTTPS URL from the repository's web page, e.g. `https://github.com/owner/repo`."
    )]
    UnsupportedScheme { input: String, scheme: String },
}

impl FromStr for GitUrl {
    type Err = GitUrlParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s)
            .map_err(|source| GitUrlParseError::InvalidUrl { input: s.to_string(), source })?;
        // `https` is a special scheme, so a successfully parsed URL is guaranteed to have a host
        // (a domain or an IP for a self-hosted server), which is all the cache path needs.
        if url.scheme() != "https" {
            return Err(GitUrlParseError::UnsupportedScheme {
                input: s.to_string(),
                scheme: url.scheme().to_string(),
            });
        }
        Ok(GitUrl(s.to_string()))
    }
}

/// Add a dependency to the package's `Nargo.toml`.
///
/// The dependency is resolved (and, for git dependencies, downloaded) before the manifest is
/// written, so a dependency that cannot be found never ends up in `Nargo.toml`.
#[derive(Debug, Clone, Args)]
#[clap(group(ArgGroup::new("source").required(true).args(["path", "git"])))]
pub(crate) struct AddCommand {
    /// Name to use for the dependency (the key under `[dependencies]`).
    /// Defaults to the package name declared in the dependency's own `Nargo.toml`.
    name: Option<CrateName>,

    /// Path to a local dependency, relative to this package's `Nargo.toml`.
    #[clap(long, conflicts_with = "git")]
    path: Option<String>,

    /// URL of a git repository to depend on.
    #[clap(long, conflicts_with = "path", requires = "tag")]
    git: Option<GitUrl>,

    /// Git branch or tag to use (required with `--git`).
    #[clap(long, requires = "git")]
    tag: Option<String>,

    /// Subdirectory within the git repository that contains the package.
    #[clap(long, requires = "git")]
    directory: Option<String>,

    /// Replace the dependency if one with the same name already exists.
    #[clap(long = "override")]
    overwrite: bool,

    #[clap(flatten)]
    package_options: PackageOptions,
}

impl WorkspaceCommand for AddCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }

    fn lock_type(&self) -> LockType {
        // We edit a workspace member's `Nargo.toml`. `lock_workspace` takes its lock on the
        // manifest file itself, so an exclusive lock serializes concurrent `add` runs.
        LockType::Exclusive
    }
}

pub(crate) fn run(args: AddCommand, workspace: Workspace) -> Result<(), CliError> {
    // Adding to several manifests at once is ambiguous, so we require a single selected package.
    // A workspace root without a `default-member` (or `--workspace`) leaves no package selected.
    let index = workspace.selected_package_index.ok_or(CliError::AddRequiresPackageSelection)?;
    let package = &workspace.members[index];
    let manifest_path = package.root_dir.join(PKG_FILE);

    let dependency = match (&args.path, &args.git) {
        (Some(path), _) => DependencyConfig::Path { path: path.clone() },
        (None, Some(git)) => DependencyConfig::Git {
            git: git.as_str().to_string(),
            // `clap` guarantees `--tag` is present whenever `--git` is.
            tag: args.tag.clone().expect("--git requires --tag"),
            directory: args.directory.clone(),
        },
        // `clap`'s required `source` group guarantees exactly one of `--path`/`--git`.
        (None, None) => unreachable!("clap requires either --path or --git"),
    };

    // Resolve the dependency before touching the manifest: this validates that it exists and is
    // not a binary, downloads git dependencies into the cache, and tells us where it landed.
    let resolved = resolve_dependency(&package.root_dir, &dependency)?;

    let name = match args.name {
        Some(name) => name,
        None => resolved.package_name().clone(),
    };

    add_dependency_to_manifest(&manifest_path, &name.to_string(), &dependency, args.overwrite)?;

    let location = resolved.package().root_dir.display();
    noirc_errors::println_to_stdout!("Added dependency `{name}`");
    match &dependency {
        DependencyConfig::Git { git, tag, .. } => {
            noirc_errors::println_to_stdout!("    source: {git} (tag {tag})");
        }
        DependencyConfig::Path { path } => {
            noirc_errors::println_to_stdout!("    source: {path}");
        }
    }
    noirc_errors::println_to_stdout!("  location: {location}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use nargo_toml::{PackageSelection, resolve_workspace_from_toml};

    use super::*;

    /// Writes a package `Nargo.toml` and its entry file under `root/dir`.
    fn write_pkg(root: &Path, dir: &str, name: &str, package_type: &str) {
        let pkg_dir = root.join(dir);
        std::fs::create_dir_all(pkg_dir.join("src")).unwrap();
        std::fs::write(
            pkg_dir.join(PKG_FILE),
            format!("[package]\nname = \"{name}\"\ntype = \"{package_type}\"\nauthors = [\"\"]\n\n[dependencies]\n"),
        )
        .unwrap();
        let entry = if package_type == "lib" { "lib.nr" } else { "main.nr" };
        std::fs::write(pkg_dir.join("src").join(entry), "").unwrap();
    }

    /// Resolves the package at `root/dir` into a single-package workspace.
    fn workspace_for(root: &Path, dir: &str) -> Workspace {
        resolve_workspace_from_toml(
            &root.join(dir).join(PKG_FILE),
            PackageSelection::DefaultOrAll,
            None,
        )
        .unwrap()
    }

    fn add_command(path: &str, overwrite: bool) -> AddCommand {
        AddCommand {
            name: None,
            path: Some(path.to_string()),
            git: None,
            tag: None,
            directory: None,
            overwrite,
            package_options: PackageOptions::default(),
        }
    }

    #[test]
    fn add_path_dependency_infers_name_and_guards_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_pkg(root, "the_bin", "demo", "bin");
        write_pkg(root, "the_lib", "cool_lib", "lib");

        let manifest = root.join("the_bin").join(PKG_FILE);

        // First add: the name is inferred from the dependency's own `Nargo.toml`.
        run(add_command("../the_lib", false), workspace_for(root, "the_bin")).unwrap();
        let contents = std::fs::read_to_string(&manifest).unwrap();
        assert!(
            contents.contains(r#"cool_lib = { path = "../the_lib" }"#),
            "expected the inferred dependency entry, got:\n{contents}"
        );

        // Re-adding the same dependency without `--override` is rejected.
        let err = run(add_command("../the_lib", false), workspace_for(root, "the_bin"))
            .expect_err("re-adding an existing dependency should fail");
        assert!(matches!(
            err,
            CliError::ManifestError(nargo_toml::ManifestError::DependencyAlreadyExists(_))
        ));

        // With `--override` it succeeds.
        run(add_command("../the_lib", true), workspace_for(root, "the_bin")).unwrap();
    }

    #[test]
    fn git_url_accepts_https_and_preserves_the_original_string() {
        let url: GitUrl = "https://github.com/noir-lang/noir-bignum".parse().unwrap();
        assert_eq!(url.as_str(), "https://github.com/noir-lang/noir-bignum");
    }

    #[test]
    fn git_url_rejects_http_to_avoid_sending_credentials_in_the_clear() {
        let err = "http://github.com/noir-lang/noir-bignum".parse::<GitUrl>().unwrap_err();
        assert!(
            matches!(err, GitUrlParseError::UnsupportedScheme { scheme, .. } if scheme == "http")
        );
    }

    #[test]
    fn git_url_rejects_ssh_scheme() {
        let err = "ssh://git@github.com/noir-lang/noir-bignum".parse::<GitUrl>().unwrap_err();
        assert!(
            matches!(err, GitUrlParseError::UnsupportedScheme { scheme, .. } if scheme == "ssh")
        );
    }

    #[test]
    fn git_url_rejects_scp_style_ssh_url() {
        // This is the form that produced the bare "relative URL without a base" error.
        let err = "git@github.com:noir-lang/sha256.git".parse::<GitUrl>().unwrap_err();
        assert!(matches!(err, GitUrlParseError::InvalidUrl { .. }));
    }

    #[test]
    fn git_url_accepts_https_with_ip_host() {
        // A self-hosted repository may be served from an IP address rather than a domain.
        let url: GitUrl = "https://192.168.1.10/me/repo".parse().unwrap();
        assert_eq!(url.as_str(), "https://192.168.1.10/me/repo");
    }
}
