use crate::{
    errors::CliError, find_package_manifest, manifest::resolve_workspace_from_toml, prepare_package,
};
use acvm::Backend;
use clap::Args;
use iter_extended::btree_map;
use nargo::package::Package;
use noirc_abi::{AbiParameter, AbiType, MAIN_RETURN_NAME};
use noirc_driver::{check_crate, compute_function_signature, CompileOptions};
use noirc_errors::reporter::ReportedErrors;
use noirc_frontend::{
    graph::{CrateId, CrateName},
    hir::Context,
};

use super::NargoConfig;

/// Checks the constraint system for errors
#[derive(Debug, Clone, Args)]
pub(crate) struct CheckCommand {
    /// The name of the package to check
    #[clap(long)]
    package: Option<CrateName>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    _backend: &B,
    args: CheckCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = find_package_manifest(&config.program_dir)?;
    let workspace = resolve_workspace_from_toml(&toml_path, args.package)?;

    for package in &workspace {
        check_package(package, &args.compile_options)?;
        println!("[{}] Finished!", package.name);
    }
    Ok(())
}

fn check_package(
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<(), ReportedErrors> {
    let (mut context, crate_id) = prepare_package(package);
    check_crate_and_report_errors(&mut context, crate_id, compile_options.deny_warnings)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use noirc_driver::CompileOptions;
    use crate::{find_package_manifest, manifest::resolve_workspace_from_toml};
    use super::create_input_toml_template;

    const TEST_DATA_DIR: &str = "tests/target_tests_data";

    #[test]
    fn pass() {
        let pass_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{TEST_DATA_DIR}/pass"));

        let config = CompileOptions::default();
        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            let toml_path = find_package_manifest(&path).unwrap();
            let workspace = resolve_workspace_from_toml(&toml_path, None).unwrap();
            for package in &workspace {
                assert!(super::check_package(package, &config).is_ok(), "path: {}", path.display());
            }
        }
    }

    #[test]
    #[ignore = "This test fails because the reporter exits the process with 1"]
    fn fail() {
        let fail_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{TEST_DATA_DIR}/fail"));

        let config = CompileOptions::default();
        let paths = std::fs::read_dir(fail_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            let toml_path = find_package_manifest(&path).unwrap();
            let workspace = resolve_workspace_from_toml(&toml_path, None).unwrap();
            for package in &workspace {
                assert!(
                    super::check_package(package, &config).is_err(),
                    "path: {}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn pass_with_warnings() {
        let pass_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("{TEST_DATA_DIR}/pass_dev_mode"));

        let config = CompileOptions { deny_warnings: false, ..Default::default() };

        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            let toml_path = find_package_manifest(&path).unwrap();
            let workspace = resolve_workspace_from_toml(&toml_path, None).unwrap();
            for package in &workspace {
                assert!(super::check_package(package, &config).is_ok(), "path: {}", path.display());
            }
        }
    }
}

/// Run the lexing, parsing, name resolution, and type checking passes and report any warnings
/// and errors found.
pub(crate) fn check_crate_and_report_errors(
    context: &mut Context,
    crate_id: CrateId,
    deny_warnings: bool,
) -> Result<(), ReportedErrors> {
    let result = check_crate(context, crate_id, deny_warnings).map(|warnings| ((), warnings));
    super::compile_cmd::report_errors(result, context, deny_warnings)
}
