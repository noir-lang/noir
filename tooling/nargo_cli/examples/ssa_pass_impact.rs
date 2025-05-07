//! Run the SSA pipelines on `test_programs/execution_success` and find the ones
//! where a certain SSA pass has the most impact.
//!
//! ```ignore
//! cargo run -p nargo_cli --example ssa_pass_impact -- --ssa-pass "Removing Unreachable Functions"
//! ```
use std::path::{Path, PathBuf};

use acvm::acir::circuit::ExpressionWidth;
use clap::Parser;
use fm::FileManager;
use nargo::{
    insert_all_files_for_workspace_into_file_manager, package::Package, parse_all, prepare_package,
    workspace::Workspace,
};
use nargo_toml::{
    ManifestError, PackageSelection, get_package_manifest, resolve_workspace_from_toml,
};
use noirc_driver::{CompilationResult, CompileOptions, NOIR_ARTIFACT_VERSION_STRING, check_crate};
use noirc_errors::CustomDiagnostic;
use noirc_evaluator::{
    brillig::BrilligOptions,
    errors::RuntimeError,
    ssa::{SsaEvaluatorOptions, SsaLogging, SsaPass, primary_passes, ssa_gen},
};
use noirc_frontend::{
    debug::DebugInstrumenter,
    elaborator::UnstableFeature,
    hir::ParsedFiles,
    monomorphization::{ast::Program, monomorphize},
};

/// SSA rendered to `String` after a certain step.
struct SsaPrint {
    step: usize,
    msg: String,
    ssa: String,
}
struct SsaBeforeAndAfter {
    before: SsaPrint,
    after: SsaPrint,
}

/// Try to find the directory that Cargo sets when it is running;
/// otherwise fallback to assuming the CWD is the root of the repository
/// and append the crate path.
fn test_programs_dir() -> PathBuf {
    let root_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir).parent().unwrap().parent().unwrap().to_path_buf(),
        Err(_) => std::env::current_dir().unwrap(),
    };
    root_dir.join("test_programs")
}

/// Collect the test programs under a sub-directory.
fn read_test_program_dirs(
    test_programs_dir: &Path,
    test_sub_dir: &str,
) -> impl Iterator<Item = PathBuf> + use<> {
    let test_case_dir = test_programs_dir.join(test_sub_dir);
    std::fs::read_dir(test_case_dir)
        .unwrap()
        .flatten()
        .filter(|c| c.path().is_dir())
        .map(|c| c.path())
}

/// Read a given program directory into a workspace.
fn read_workspace(
    program_dir: &Path,
    selection: PackageSelection,
) -> Result<Workspace, ManifestError> {
    let toml_path = get_package_manifest(program_dir)?;

    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_owned()),
    )?;

    Ok(workspace)
}

#[derive(Parser, Debug)]
struct Options {
    /// Name of the SSA pass we want to see the impact of.
    #[arg(long)]
    ssa_pass: String,
    #[arg(long, default_value = "0")]
    inliner_aggressiveness: i64,
}

fn main() {
    let opts = Options::parse();
    let sel = PackageSelection::DefaultOrAll;

    let test_workspaces = read_test_program_dirs(&test_programs_dir(), "execution_success")
        .filter_map(|dir| read_workspace(&dir, sel.clone()).ok())
        .collect::<Vec<_>>();

    let compile_options = CompileOptions {
        unstable_features: vec![UnstableFeature::Enums],
        silence_warnings: true,
        ..Default::default()
    };

    let ssa_options = SsaEvaluatorOptions {
        ssa_logging: SsaLogging::None,
        brillig_options: BrilligOptions::default(),
        print_codegen_timings: false,
        expression_width: ExpressionWidth::default(),
        emit_ssa: None,
        skip_underconstrained_check: true,
        skip_brillig_constraints_check: true,
        enable_brillig_constraints_check_lookback: false,
        inliner_aggressiveness: opts.inliner_aggressiveness,
        max_bytecode_increase_percent: None,
    };

    let ssa_passes = primary_passes(&ssa_options);
    let last_pass = ssa_passes
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.msg().contains(&opts.ssa_pass).then_some(i))
        .max()
        .expect("cannot find a pass with the given name");

    let mut ssa_pairs = Vec::new();

    // Note that instead of compiling the code and running SSA passes one by one,
    // we could work with the snapshots exported in https://github.com/noir-lang/noir/pull/7853 (currently draft),
    // and focus on just the string comparison part. That would have the benefit
    // of doing 100% what the normal compilation pipeline does, and that once the
    // snapshots are prepared, we can compare any pairs at will, rather than have
    // to recompile to look at another pass.

    for workspace in test_workspaces {
        let mut file_manager = workspace.new_file_manager();
        insert_all_files_for_workspace_into_file_manager(&workspace, &mut file_manager);
        let parsed_files = parse_all(&file_manager);
        let binary_packages = workspace.into_iter().filter(|package| package.is_binary());

        for package in binary_packages {
            let Ok((Some(program), _)) = compile_into_program(
                &file_manager,
                &parsed_files,
                &workspace,
                package,
                &compile_options,
            ) else {
                continue;
            };

            let pairs =
                collect_ssa_before_and_after(program, &ssa_passes[..=last_pass], &opts.ssa_pass)
                    .unwrap_or_else(|e| {
                        panic!("failed to run SSA passes on {}: {e}", package.name)
                    });

            if !pairs.is_empty() {
                ssa_pairs.push((package.name.clone(), pairs));
            }
        }
    }

    let package_cnt = ssa_pairs.len();
    let ssa_pair_cnt = ssa_pairs.iter().map(|(_, pairs)| pairs.len()).sum::<usize>();
    let no_impact_cnt = ssa_pairs
        .iter()
        .map(|(_, pairs)| pairs.iter().filter(|p| p.before.ssa == p.after.ssa).count())
        .sum::<usize>();

    println!("Packages: {package_cnt}");
    println!("SSA pairs: {ssa_pair_cnt}");
    println!("No impact: {no_impact_cnt}");
}

/// Compile a package into a monomorphized [Program].
///
/// If the package has no `main` function then `None` is returend.
fn compile_into_program(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    workspace: &Workspace,
    package: &Package,
    options: &CompileOptions,
) -> CompilationResult<Option<Program>> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    context.disable_comptime_printing();
    context.debug_instrumenter = DebugInstrumenter::default();
    context.package_build_path = workspace.package_build_path(package);
    let (_, warnings) = check_crate(&mut context, crate_id, &options)?;
    let Some(main) = context.get_main_function(&crate_id) else {
        return Ok((None, warnings));
    };
    let program = monomorphize(main, &mut context.def_interner, false)
        .map_err(|error| vec![CustomDiagnostic::from(error)])?;
    Ok((Some(program), warnings))
}

/// Run the SSA passes on a program until a certain named one in the pipeline.
fn collect_ssa_before_and_after(
    program: Program,
    passes: &[SsaPass],
    name: &str,
) -> Result<Vec<SsaBeforeAndAfter>, RuntimeError> {
    let mut pairs = Vec::new();
    let mut ssa = ssa_gen::generate_ssa(program)?;
    let mut last_msg = "Initial";

    for (i, pass) in passes.iter().enumerate() {
        let before = pass.msg().contains(name).then(|| format!("{ssa}"));
        ssa = pass.run(ssa)?;
        if let Some(before) = before {
            pairs.push(SsaBeforeAndAfter {
                before: SsaPrint { step: i, msg: last_msg.to_string(), ssa: before },
                after: SsaPrint { step: i + 1, msg: pass.msg().to_string(), ssa: format!("{ssa}") },
            });
        }
        last_msg = pass.msg()
    }

    Ok(pairs)
}
