use crate::{errors::CliError, resolver::resolve_root_manifest};
use acvm::Backend;
use clap::Args;

use noirc_driver::{check_crate, CompileOptions};
use noirc_errors::reporter::ReportedErrors;
use noirc_frontend::{graph::CrateId, hir::Context};
use std::path::Path;

use super::NargoConfig;

/// Checks the constraint system for errors
#[derive(Debug, Clone, Args)]
pub(crate) struct CheckCommand {
    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: CheckCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    check_from_path(backend, &config.program_dir, &args.compile_options)?;
    println!("Finished!");
    Ok(())
}

fn check_from_path<B: Backend>(
    // Backend isn't used but keeping it in the signature allows for better type inference
    // TODO: This function doesn't need to exist but requires a little more refactoring
    _backend: &B,
    program_dir: &Path,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let (mut context, crate_id) = resolve_root_manifest(program_dir, None)?;
    check_crate_and_report_errors(
        &mut context,
        crate_id,
        compile_options.deny_warnings,
        compile_options.experimental_ssa,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use noirc_driver::CompileOptions;
    use std::path::PathBuf;

    const TEST_DATA_DIR: &str = "tests/target_tests_data";

    #[test]
    fn pass() {
        let pass_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{TEST_DATA_DIR}/pass"));

        let backend = crate::backends::ConcreteBackend::default();
        let config = CompileOptions::default();
        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(
                super::check_from_path(&backend, &path, &config).is_ok(),
                "path: {}",
                path.display()
            );
        }
    }

    #[test]
    #[ignore = "This test fails because the reporter exits the process with 1"]
    fn fail() {
        let fail_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{TEST_DATA_DIR}/fail"));

        let backend = crate::backends::ConcreteBackend::default();
        let config = CompileOptions::default();
        let paths = std::fs::read_dir(fail_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(
                super::check_from_path(&backend, &path, &config).is_err(),
                "path: {}",
                path.display()
            );
        }
    }

    #[test]
    fn pass_with_warnings() {
        let pass_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("{TEST_DATA_DIR}/pass_dev_mode"));

        let backend = crate::backends::ConcreteBackend::default();
        let config = CompileOptions { deny_warnings: false, ..Default::default() };

        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(
                super::check_from_path(&backend, &path, &config).is_ok(),
                "path: {}",
                path.display()
            );
        }
    }
}

/// Run the lexing, parsing, name resolution, and type checking passes and report any warnings
/// and errors found.
pub(crate) fn check_crate_and_report_errors(
    context: &mut Context,
    crate_id: CrateId,
    deny_warnings: bool,
    experimental_ssa: bool,
) -> Result<(), ReportedErrors> {
    let result = check_crate(context, crate_id, deny_warnings, experimental_ssa)
        .map(|warnings| ((), warnings));
    super::compile_cmd::report_errors(result, context, deny_warnings)
}
