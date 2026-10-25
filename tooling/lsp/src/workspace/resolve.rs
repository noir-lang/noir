//! Finds the set of packages to analyze together for a source file.

use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use async_lsp::LanguageClient;
use async_lsp::lsp_types::{LogMessageParams, MessageType};
use nargo::{
    constants::PKG_FILE,
    package::{Package, PackageType},
    workspace::Workspace,
};
use nargo_toml::{PackageSelection, find_file_manifest, resolve_workspace_from_toml};
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;
use noirc_frontend::graph::CrateName;

use crate::{LspError, LspState};

/// Packages under the editor's root are analyzed together up to this many. Past it, each
/// Nargo workspace is analyzed on its own.
const MAX_ROOT_PACKAGES: usize = 200;

/// What is known about the packages under the editor's root (see [`discover_root_workspace`]).
pub(crate) enum RootWorkspace {
    /// The root has not been searched yet.
    Unknown,
    /// The root holds no packages, or more than root analysis takes.
    Unusable,
    Found {
        workspace: Arc<Workspace>,
        /// The canonical root directory of every package in `workspace`.
        package_dirs: HashSet<PathBuf>,
    },
}

/// Finds the set of packages to analyze together for a source file:
/// * every package found under the editor's root, when the file belongs to one of them and
///   root analysis is enabled
/// * otherwise the outermost Nargo workspace containing the file (see [`enclosing_workspace`])
/// * otherwise, without a `Nargo.toml`, the parent directory as a dummy workspace
///
/// Only a manifest change alters the answer, so it is cached per manifest until then.
pub(crate) fn resolve_workspace_for_source_path(
    state: &mut LspState,
    file_path: &Path,
) -> Result<Arc<Workspace>, LspError> {
    let Some(toml_path) = find_file_manifest(file_path) else {
        return assumed_workspace(file_path).map(Arc::new);
    };
    if let Some(workspace) = state.resolved_workspaces.get(&toml_path) {
        return Ok(Arc::clone(workspace));
    }

    let workspace = resolve_workspace_for_manifest(state, &toml_path)?;
    state.resolved_workspaces.insert(toml_path, Arc::clone(&workspace));
    Ok(workspace)
}

fn resolve_workspace_for_manifest(
    state: &mut LspState,
    toml_path: &Path,
) -> Result<Arc<Workspace>, LspError> {
    if state.options.enable_root_analysis
        && let Some(package_dir) = toml_path.parent()
        && let RootWorkspace::Found { workspace, package_dirs } = root_workspace(state)
        && package_dirs.contains(&canonical_or_same(package_dir))
    {
        return Ok(Arc::clone(workspace));
    }

    match resolve_workspace_from_toml(
        toml_path,
        PackageSelection::All,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    ) {
        Ok(workspace) => Ok(Arc::new(enclosing_workspace(workspace, toml_path))),
        Err(error) => Err(LspError::ManifestError(toml_path.to_path_buf(), error)),
    }
}

fn root_workspace(state: &mut LspState) -> &RootWorkspace {
    if matches!(state.root_workspace, RootWorkspace::Unknown)
        && let Some(root_path) = state.root_path.clone()
    {
        state.root_workspace = match discover_root_workspace(&root_path, MAX_ROOT_PACKAGES) {
            RootDiscovery::Found(workspace) => {
                let package_dirs =
                    workspace.members.iter().map(|member| canonical_or_same(&member.root_dir));
                RootWorkspace::Found {
                    package_dirs: package_dirs.collect(),
                    workspace: Arc::new(workspace),
                }
            }
            RootDiscovery::Empty => RootWorkspace::Unusable,
            RootDiscovery::TooManyPackages => {
                let _ = state.client.log_message(LogMessageParams {
                    typ: MessageType::INFO,
                    message: format!(
                        "{} holds more than {MAX_ROOT_PACKAGES} Noir packages, so each Nargo \
                         workspace is analyzed on its own and references do not cross between \
                         them",
                        root_path.display()
                    ),
                });
                RootWorkspace::Unusable
            }
        };
    }
    &state.root_workspace
}

enum RootDiscovery {
    Found(Workspace),
    Empty,
    TooManyPackages,
}

/// Collects every package found under `root_path` into one workspace, the way rust-analyzer
/// loads every Cargo workspace under the opened folder. A package listed both by its own
/// manifest and by a workspace manifest is included once. Manifests that fail to resolve are
/// skipped; they get their diagnostics when a file of theirs is opened.
///
/// The search stops as soon as it finds more than `max_packages` packages.
fn discover_root_workspace(root_path: &Path, max_packages: usize) -> RootDiscovery {
    let mut members: Vec<Package> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();

    let walker = walkdir::WalkDir::new(root_path).into_iter().filter_entry(|entry| {
        if entry.depth() == 0 || !entry.file_type().is_dir() {
            return true;
        }
        let name = entry.file_name().to_string_lossy();
        name != "target" && name != "node_modules" && !name.starts_with('.')
    });
    for entry in walker.filter_map(Result::ok) {
        if !entry.file_type().is_file() || entry.file_name() != PKG_FILE {
            continue;
        }
        // A package already listed by a workspace manifest needs no second resolution.
        if entry.path().parent().is_some_and(|dir| seen.contains(&canonical_or_same(dir))) {
            continue;
        }
        let Ok(workspace) = resolve_workspace_from_toml(
            entry.path(),
            PackageSelection::All,
            Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
        ) else {
            continue;
        };
        for package in workspace.members {
            if seen.insert(canonical_or_same(&package.root_dir)) {
                members.push(package);
            }
        }
        if members.len() > max_packages {
            return RootDiscovery::TooManyPackages;
        }
    }

    if members.is_empty() {
        return RootDiscovery::Empty;
    }
    RootDiscovery::Found(Workspace {
        root_dir: root_path.to_path_buf(),
        target_dir: None,
        members,
        selected_package_index: None,
        is_assumed: false,
    })
}

/// Treats the parent directory of a file outside any Nargo package as a dummy workspace with
/// a single package whose entry point is the file.
fn assumed_workspace(file_path: &Path) -> Result<Workspace, LspError> {
    let Some(parent_folder) = file_path
        .parent()
        .and_then(|f| f.file_name())
        .and_then(|file_name_os_str| file_name_os_str.to_str())
    else {
        return Err(LspError::WorkspaceResolutionError(format!(
            "Could not resolve parent folder for file: {}",
            file_path.display()
        )));
    };

    let crate_name = match CrateName::from_str(parent_folder) {
        Ok(name) => name,
        Err(error) => {
            eprintln!("{error}");
            CrateName::from_str("root").unwrap()
        }
    };

    let assumed_package = Package {
        version: None,
        compiler_required_version: Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
        compiler_required_unstable_features: Vec::new(),
        root_dir: PathBuf::from(parent_folder),
        package_type: PackageType::Binary,
        entry_path: PathBuf::from(file_path),
        name: crate_name,
        dependencies: BTreeMap::new(),
    };
    let workspace = Workspace {
        root_dir: PathBuf::from(parent_folder),
        members: vec![assumed_package],
        selected_package_index: Some(0),
        is_assumed: true,
        target_dir: None,
    };
    Ok(workspace)
}

/// Widens `workspace`, resolved from the manifest at `toml_path`, to the outermost ancestor
/// workspace that lists it as a member. Analyzing that workspace is what lets a package see the
/// packages that depend on it.
fn enclosing_workspace(workspace: Workspace, toml_path: &Path) -> Workspace {
    let Some(package_dir) = toml_path.parent() else {
        return workspace;
    };
    let package_dir = canonical_or_same(package_dir);

    let mut workspace = workspace;
    for ancestor in package_dir.ancestors().skip(1) {
        let candidate_toml = ancestor.join(PKG_FILE);
        if !candidate_toml.is_file() {
            continue;
        }
        let Ok(candidate) = resolve_workspace_from_toml(
            &candidate_toml,
            PackageSelection::All,
            Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
        ) else {
            continue;
        };
        let is_member = candidate
            .members
            .iter()
            .any(|member| canonical_or_same(&member.root_dir) == package_dir);
        if is_member && canonical_or_same(&candidate.root_dir) != package_dir {
            workspace = candidate;
        }
    }
    workspace
}

fn canonical_or_same(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

pub(crate) fn workspace_package_for_file<'a>(
    workspace: &'a Workspace,
    file_path: &Path,
) -> Option<&'a Package> {
    if workspace.is_assumed {
        workspace.members.first()
    } else {
        // Packages can be nested, so the innermost one containing the file wins.
        workspace
            .members
            .iter()
            .filter(|package| file_path.starts_with(&package.root_dir))
            .max_by_key(|package| package.root_dir.as_os_str().len())
    }
}

#[cfg(test)]
mod workspace_resolution_tests {
    use std::sync::Arc;

    use async_lsp::lsp_types::{DidChangeWatchedFilesParams, FileChangeType, FileEvent, Url};

    use super::{RootDiscovery, discover_root_workspace, resolve_workspace_for_source_path};
    use crate::{notifications::on_did_change_watched_files, test_utils};

    fn member_names(workspace: &nargo::workspace::Workspace) -> Vec<String> {
        let mut names: Vec<_> =
            workspace.members.iter().map(|member| member.name.to_string()).collect();
        names.sort();
        names
    }

    #[test]
    fn analyzes_every_package_under_the_root_together() {
        let (mut state, root_marker) = test_utils::init_lsp_server("sibling_workspaces");
        let root_dir =
            root_marker.to_file_path().unwrap().parent().unwrap().parent().unwrap().to_path_buf();

        let workspace =
            resolve_workspace_for_source_path(&mut state, &root_dir.join("a/src/lib.nr")).unwrap();

        assert_eq!(member_names(&workspace), vec!["a", "b"]);
    }

    #[test]
    fn analyzes_only_the_nargo_workspace_without_root_analysis() {
        let (mut state, root_marker) = test_utils::init_lsp_server("sibling_workspaces");
        state.options.enable_root_analysis = false;
        let root_dir =
            root_marker.to_file_path().unwrap().parent().unwrap().parent().unwrap().to_path_buf();

        let workspace =
            resolve_workspace_for_source_path(&mut state, &root_dir.join("a/src/lib.nr")).unwrap();

        assert_eq!(member_names(&workspace), vec!["a"]);
    }

    #[test]
    fn widens_a_member_package_to_its_workspace_without_root_analysis() {
        let (mut state, root_marker) = test_utils::init_lsp_server("workspace");
        state.options.enable_root_analysis = false;
        let root_dir =
            root_marker.to_file_path().unwrap().parent().unwrap().parent().unwrap().to_path_buf();

        let workspace =
            resolve_workspace_for_source_path(&mut state, &root_dir.join("one/src/lib.nr"))
                .unwrap();

        assert_eq!(member_names(&workspace), vec!["one", "two"]);
    }

    #[test]
    fn stops_discovering_past_the_package_limit() {
        let (_, root_marker) = test_utils::init_lsp_server("sibling_workspaces");
        let root_dir =
            root_marker.to_file_path().unwrap().parent().unwrap().parent().unwrap().to_path_buf();

        assert!(matches!(discover_root_workspace(&root_dir, 1), RootDiscovery::TooManyPackages));
        let RootDiscovery::Found(workspace) = discover_root_workspace(&root_dir, 2) else {
            panic!("Expected both packages to be found");
        };
        assert_eq!(member_names(&workspace), vec!["a", "b"]);
    }

    #[test]
    fn resolves_a_workspace_again_only_after_a_manifest_change() {
        let (mut state, root_marker) = test_utils::init_lsp_server("sibling_workspaces");
        let root_dir =
            root_marker.to_file_path().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
        let a_lib = root_dir.join("a/src/lib.nr");

        let first = resolve_workspace_for_source_path(&mut state, &a_lib).unwrap();
        let second = resolve_workspace_for_source_path(&mut state, &a_lib).unwrap();
        assert!(Arc::ptr_eq(&first, &second));

        let manifest = Url::from_file_path(root_dir.join("b/Nargo.toml")).unwrap();
        on_did_change_watched_files(
            &mut state,
            DidChangeWatchedFilesParams {
                changes: vec![FileEvent { uri: manifest, typ: FileChangeType::CHANGED }],
            },
        )
        .unwrap();

        let third = resolve_workspace_for_source_path(&mut state, &a_lib).unwrap();
        assert!(!Arc::ptr_eq(&first, &third));
        assert_eq!(member_names(&third), vec!["a", "b"]);
    }
}
