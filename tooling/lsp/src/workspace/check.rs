//! Type-checks the packages of a workspace in one context.

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};

use fm::{FileId, FileManager};
use nargo::{
    package::{Dependency, Package},
    workspace::Workspace,
};
use noirc_driver::{
    CompileOptions, add_dep, check_crate, check_crates, keep_warnings_from, prepare_crate,
};
use noirc_errors::CustomDiagnostic;
use noirc_frontend::{
    graph::CrateId,
    hir::{CheckedContext, Context, LspMode, ParsedFiles},
};

use crate::{LspState, insert_all_files_for_workspace_into_file_manager, parse_diff};

/// The root crate of a workspace's analysis is an empty crate that depends on every member
/// package, so that checking it elaborates all of them into one context. Its file exists only
/// in the file manager.
fn workspace_root_file_path(workspace: &Workspace) -> PathBuf {
    workspace.root_dir.join("__nargo_lsp_workspace_root__.nr")
}

/// Adds the file of the workspace's root crate (see [`workspace_root_file_path`]) to a file
/// manager that already holds the workspace's sources, so that [`prepare_workspace`] finds it.
pub(crate) fn add_workspace_root_file(workspace: &Workspace, file_manager: &mut FileManager) {
    if !workspace.is_assumed {
        file_manager.add_file_with_source_canonical_path(
            &workspace_root_file_path(workspace),
            String::new(),
        );
    }
}

pub(crate) struct PreparedWorkspace {
    pub(crate) context: Context,
    /// The synthetic root crate that depends on every member package once they are added.
    pub(crate) root_crate_id: CrateId,
    /// The crate of each member package, keyed by the package's root directory. Filled in by
    /// [`check_workspace`], except for an assumed workspace whose single package is the root.
    pub(crate) package_crate_ids: HashMap<PathBuf, CrateId>,
}

/// Creates the context a workspace is checked in, with the stdlib and the synthetic root crate.
/// Member packages are added by [`check_workspace`] as they are checked.
pub(crate) fn prepare_workspace(
    file_manager: &Arc<FileManager>,
    parsed_files: &Arc<ParsedFiles>,
    workspace: &Workspace,
) -> PreparedWorkspace {
    let mut package_crate_ids = HashMap::new();

    let (mut context, root_crate_id) = if workspace.is_assumed {
        // An assumed workspace has a single package and no manifest, so there is nothing to share.
        let package = workspace.members.first().expect("an assumed workspace has one package");
        let (context, crate_id) = nargo::prepare_package(file_manager, parsed_files, package);
        package_crate_ids.insert(package.root_dir.clone(), crate_id);
        (context, crate_id)
    } else {
        let mut context = Context::new(Arc::clone(file_manager), Arc::clone(parsed_files));
        let root_crate_id = prepare_crate(&mut context, &workspace_root_file_path(workspace));
        (context, root_crate_id)
    };

    context.activate_lsp_mode(LspMode::Full);
    PreparedWorkspace { context, root_crate_id, package_crate_ids }
}

pub(crate) struct CheckedWorkspace {
    pub(crate) context: CheckedContext,
    pub(crate) file_manager: Arc<FileManager>,
    /// The crate of each member package, keyed by the package's root directory.
    pub(crate) package_crate_ids: HashMap<PathBuf, CrateId>,
    /// Errors from every crate, and warnings from member packages only: an error in a
    /// dependency breaks the packages that use it, but its warnings are not the user's to fix.
    pub(crate) diagnostics: Vec<CustomDiagnostic>,
}

/// Type-checks every member package of `workspace` in one context, reading open documents
/// from their editor contents.
///
/// Each member is checked as the root of its own compilation (see `Context::check_crates`),
/// smallest dependency closure first, so a member is checked before any member that depends on
/// it. A dependency is shared between members unless its own elaboration writes into another
/// crate's comptime state, or it depends on such a crate: those get an instance per member, as
/// in a standalone compilation, so that each member's macros see exactly the registrations of
/// its own dependency closure and the types of its instances agree.
pub(crate) fn check_workspace(state: &mut LspState, workspace: &Workspace) -> CheckedWorkspace {
    let mut file_manager = workspace.new_file_manager();
    if workspace.is_assumed {
        let package = workspace.members.first().unwrap();
        file_manager.add_file_with_source_canonical_path(&package.entry_path, String::new());
    } else {
        insert_all_files_for_workspace_into_file_manager(state, workspace, &mut file_manager);
    }
    add_workspace_root_file(workspace, &mut file_manager);

    let parsed_files = Arc::new(parse_diff(&file_manager, state));
    let file_manager = Arc::new(file_manager);

    let PreparedWorkspace { context, root_crate_id, mut package_crate_ids } =
        prepare_workspace(&file_manager, &parsed_files, workspace);
    let options = CompileOptions::default();

    // A file outside any package has nothing to share with other packages.
    if workspace.is_assumed {
        let (context, result) = check_crate(context, root_crate_id, &options);
        let diagnostics = match result {
            Ok(((), diagnostics)) | Err(diagnostics) => diagnostics,
        };
        return CheckedWorkspace { context, file_manager, package_crate_ids, diagnostics };
    }

    let mut members: Vec<&Package> = workspace.members.iter().collect();
    members.sort_by_key(|package| (package_closure_size(package), package.root_dir.clone()));
    let mut members = members.into_iter();
    let mut root_pending = true;
    let mut shared: HashMap<PathBuf, CrateId> = HashMap::new();

    let (context, errors) =
        check_crates(context, &options, &mut options.comptime_io(), |context| {
            if let Some(package) = members.next() {
                let mut this_run = HashMap::new();
                let crate_id = instantiate_package(context, package, &mut shared, &mut this_run);
                add_dep(context, root_crate_id, crate_id, package.name.clone());
                package_crate_ids.insert(package.root_dir.clone(), crate_id);
                Some(crate_id)
            } else if root_pending {
                root_pending = false;
                Some(root_crate_id)
            } else {
                None
            }
        });

    let member_crates: Vec<CrateId> = package_crate_ids.values().copied().collect();
    let diagnostics = keep_warnings_from(&context, &errors, &member_crates, &options);
    let diagnostics = without_repeats(diagnostics);

    CheckedWorkspace { context, file_manager, package_crate_ids, diagnostics }
}

/// What tells two diagnostics apart for [`without_repeats`]: file, primary span and message.
type DiagnosticKey = (FileId, Option<(u32, u32)>, String);

/// A crate elaborated once per dependent reports the same diagnostic once per instance; this
/// keeps the first of each.
fn without_repeats(diagnostics: Vec<CustomDiagnostic>) -> Vec<CustomDiagnostic> {
    let mut seen: HashSet<DiagnosticKey> = HashSet::new();
    diagnostics
        .into_iter()
        .filter(|diagnostic| {
            let span = diagnostic
                .secondaries
                .first()
                .map(|label| (label.location.span.start(), label.location.span.end()));
            seen.insert((diagnostic.file, span, diagnostic.message.clone()))
        })
        .collect()
}

/// The number of distinct packages `package` depends on, directly or not.
fn package_closure_size(package: &Package) -> usize {
    fn walk(package: &Package, seen: &mut HashSet<PathBuf>) {
        for dependency in package.dependencies.values() {
            let dependency = dependency_package(dependency);
            if seen.insert(dependency.entry_path.clone()) {
                walk(dependency, seen);
            }
        }
    }
    let mut seen = HashSet::new();
    walk(package, &mut seen);
    seen.len()
}

fn dependency_package(dependency: &Dependency) -> &Package {
    match dependency {
        Dependency::Local { package } | Dependency::Remote { package } => package,
    }
}

/// Whether `existing`, the crate a package was first elaborated as, can serve the member being
/// checked too. It cannot when it is stateful, as the member's macros need its registrations
/// made again, nor when one of its dependencies is not shared, as its items would then name the
/// types of a different instance of that dependency than the member's own.
fn can_share(existing: CrateId, dependencies_shared: bool, context: &Context) -> bool {
    dependencies_shared && !context.def_interner.is_stateful_crate(existing)
}

/// Adds `package` and its dependencies to the crate graph for the member being checked and
/// returns the crate to use for it: the crate the package was first elaborated as when
/// [`can_share`] allows, and a fresh instance otherwise. `shared` holds those first crates.
/// `this_run` memoizes the crate chosen for each package within one member, so that a package
/// reached along two paths is one crate.
fn instantiate_package(
    context: &mut Context,
    package: &Package,
    shared: &mut HashMap<PathBuf, CrateId>,
    this_run: &mut HashMap<PathBuf, CrateId>,
) -> CrateId {
    if let Some(crate_id) = this_run.get(&package.entry_path) {
        return *crate_id;
    }

    let mut dependencies = Vec::new();
    let mut dependencies_shared = true;
    for (name, dependency) in &package.dependencies {
        let dependency = dependency_package(dependency);
        let crate_id = instantiate_package(context, dependency, shared, this_run);
        dependencies_shared &= shared.get(&dependency.entry_path) == Some(&crate_id);
        dependencies.push((name.clone(), crate_id));
    }

    let existing = shared.get(&package.entry_path).copied();
    let crate_id = match existing {
        Some(existing) if can_share(existing, dependencies_shared, context) => existing,
        _ => {
            let file_id =
                context.file_manager.name_to_id(package.entry_path.clone()).unwrap_or_else(|| {
                    panic!("{} is not in the file manager", package.entry_path.display())
                });
            let crate_id = if existing.is_some() {
                context.crate_graph.add_crate_instance(file_id)
            } else {
                let crate_id = context.crate_graph.add_crate(file_id);
                shared.insert(package.entry_path.clone(), crate_id);
                crate_id
            };
            let std_crate_id = *context.stdlib_crate_id();
            add_dep(context, crate_id, std_crate_id, "std".parse().unwrap());
            context
                .required_unstable_features
                .insert(crate_id, package.compiler_required_unstable_features.clone());
            for (name, dependency) in dependencies {
                add_dep(context, crate_id, dependency, name);
            }
            crate_id
        }
    };
    this_run.insert(package.entry_path.clone(), crate_id);
    crate_id
}

#[cfg(test)]
mod check_workspace_tests {
    use async_lsp::lsp_types::Url;

    use super::check_workspace;
    use crate::notifications::workspace_from_document_uri;
    use crate::test_utils::{self, crate_dependency as dependency};

    /// In `test_programs/comptime_state`, `notelib` registers into `macros`' state, `wrapper` is
    /// a plain library over `notelib`, `x` is a plain library, and `p1`, `p2`, `p4` and `q`
    /// depend on them. Each dependent must get its own `notelib`, and with it its own `wrapper`,
    /// while `macros` and `x` are shared, and a member's instances agree with each other.
    #[test]
    fn shares_plain_crates_and_instantiates_stateful_ones_per_dependent() {
        let mut fixture = test_utils::Fixture::new("comptime_state");
        let workspace = fixture.workspace_for("p1/src/lib.nr");
        let checked = check_workspace(&mut fixture.state, &workspace);
        let graph = &checked.context.crate_graph;
        let member = |name: &str| checked.package_crate_ids[&fixture.path(name)];

        // `macros` and `x` write no other crate's state, so everyone shares them.
        assert_eq!(dependency(graph, member("p1"), "macros"), member("macros"));
        assert_eq!(dependency(graph, member("p2"), "macros"), member("macros"));
        assert_eq!(dependency(graph, member("p3"), "x"), member("x"));
        assert_eq!(dependency(graph, member("q"), "x"), member("x"));

        // `notelib` registers into `macros`, so each dependent gets an instance of its own.
        let notelib_in_p1 = dependency(graph, member("p1"), "notelib");
        let notelib_in_p2 = dependency(graph, member("p2"), "notelib");
        let notelib_in_p4 = dependency(graph, member("p4"), "notelib");
        assert_ne!(notelib_in_p1, member("notelib"));
        assert_ne!(notelib_in_p1, notelib_in_p2);
        assert_ne!(notelib_in_p2, notelib_in_p4);
        for instance in [notelib_in_p1, notelib_in_p2, notelib_in_p4] {
            assert_eq!(graph[instance].root_file_id, graph[member("notelib")].root_file_id);
        }

        // `wrapper` depends on `notelib`, so `p4` gets its own `wrapper` too, built over the
        // same `notelib` instance `p4` uses directly.
        let wrapper_in_p4 = dependency(graph, member("p4"), "wrapper");
        assert_ne!(wrapper_in_p4, member("wrapper"));
        assert_eq!(dependency(graph, wrapper_in_p4, "notelib"), notelib_in_p4);
        assert_ne!(dependency(graph, member("wrapper"), "notelib"), member("notelib"));
    }

    #[test]
    fn names_crates_relative_to_the_package_with_the_error() {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("similarly_named_types");
        let root_dir = noir_text_document.to_file_path().unwrap();
        let root_dir = root_dir.parent().unwrap().parent().unwrap();
        let a_lib = Url::from_file_path(root_dir.join("a/src/lib.nr")).unwrap();

        let workspace = workspace_from_document_uri(&mut state, a_lib).unwrap().unwrap();
        let checked = check_workspace(&mut state, &workspace);

        let mut notes: Vec<&str> = checked
            .diagnostics
            .iter()
            .flat_map(|diagnostic| &diagnostic.secondaries)
            .map(|label| label.message.as_str())
            .filter(|message| message.starts_with("Note: "))
            .collect();
        notes.sort();
        assert_eq!(
            notes,
            vec![
                "Note: `Foo` and `lib::Foo` have similar names, but are actually distinct types",
                "Note: `Foo` is defined in the current crate",
                "Note: `lib::Foo` is defined in crate `lib`",
            ]
        );
    }
}
