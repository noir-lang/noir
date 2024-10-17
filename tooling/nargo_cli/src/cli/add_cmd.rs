use crate::cli::manifest_editor::add::AddDependency;
use crate::cli::manifest_editor::{DepId, DepType, EditManifestOptions, Op};
use crate::cli::{manifest_editor, NargoConfig};
use crate::errors::CliError;
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, Parser};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{CompileOptions, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::graph::CrateName;
use crate::cli::compile_cmd::compile_workspace_full;

/// Add dependencies to a Nargo.toml manifest file
#[derive(Debug, Clone, Args)]
pub(crate) struct AddCommand {
    /// Reference to a package to add as a dependency
    ///
    /// You can reference a package by:
    /// - `<name>`, like `scarb add alexandria_math` (the latest version will be used)
    /// - `<name>@<version-req>`, like `scarb add alexandria_math@1` or `scarb add alexandria_math@=0.1.0`
    #[arg(value_name = "DEP_ID", verbatim_doc_comment)]
    pub packages: Vec<DepId>,

    /// The name of the package to compile
    // #[clap(long, conflicts_with = "workspace")]
    #[clap(long)]
    package: Option<CrateName>,
    // /// Do not actually write the manifest.
    // #[arg(long)]
    // pub dry_run: bool,

    // #[command(flatten)]
    // pub packages_filter: PackagesFilter,
    //
    /// _Source_ section.
    #[command(flatten, next_help_heading = "Source")]
    pub source: AddSourceArgs,
    //
    // /// _Section_ section.
    // #[command(flatten, next_help_heading = "Section")]
    // pub section: AddSectionArgs,
}

/// _Source_ section of [`AddArgs`].
#[derive(Parser, Clone, Debug)]
pub struct AddSourceArgs {
    /// Filesystem path to local package to add.
    #[arg(long, conflicts_with_all = ["git", "GitRefGroup"])]
    pub path: Option<Utf8PathBuf>,

    /// Git repository location
    ///
    /// Without any other information, Scarb will use the latest commit on the default branch.
    #[arg(long, value_name = "URI")]
    pub git: Option<String>,

    /// Git reference args for `--git`.
    #[command(flatten)]
    pub git_ref: GitRefGroup,
}

/// Git reference specification arguments.
#[derive(Parser, Clone, Debug)]
#[group(requires = "git", multiple = false)]
pub struct GitRefGroup {
    /// Git branch to download the package from.
    #[arg(long)]
    pub branch: Option<String>,

    /// Git tag to download the package from.
    #[arg(long)]
    pub tag: Option<String>,

    /// Git reference to download the package from
    ///
    /// This is the catch-all, handling hashes to named references in remote repositories.
    #[arg(long)]
    pub rev: Option<String>,
}

/// _Section_ section of [`AddArgs`].
#[derive(Parser, Clone, Debug)]
pub struct AddSectionArgs {
    /// Add as development dependency.
    ///
    /// Dev-dependencies are only used when compiling tests.
    ///
    /// These dependencies are not propagated to other packages which depend on this package.
    #[arg(long)]
    pub dev: bool,
}

pub(crate) fn run(args: AddCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection = PackageSelection::DefaultOrAll;
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);

    let toml_path_utf = Utf8Path::new(toml_path.to_str().unwrap());
    manifest_editor::edit(
        toml_path_utf,
        build_ops(args.packages.clone(), args.source.clone()),
        EditManifestOptions {
            dry_run: false,
        },
    ).unwrap();
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_owned()),
    )?;
    let compile_options = CompileOptions::default();
    compile_workspace_full(&workspace, &compile_options)?;

    Ok(())
}


fn build_ops(
    packages: Vec<DepId>,
    source: AddSourceArgs,
) -> Vec<Box<dyn Op>> {
    //todo fix for aztec, currently no support for adding packages to dev section
    let dep_type = DepType::Normal;

    let template = AddDependency {
        dep: DepId::unspecified(),
        path: source.path,
        git: source.git,
        branch: source.git_ref.branch,
        tag: source.git_ref.tag,
        rev: source.git_ref.rev,
        dep_type,
    };

    if packages.is_empty() {
        vec![Box::new(template)]
    } else {
        packages
            .into_iter()
            .map(|dep| -> Box<dyn Op> {
                Box::new(AddDependency {
                    dep,
                    ..template.clone()
                })
            })
            .collect()
    }
}