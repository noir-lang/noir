//! Run the SSA pipelines on `test_programs/execution_success` and find the ones
//! where a certain SSA pass has the most impact.
//!
//! ```ignore
//! cargo run -p nargo_cli --example ssa_pass_impact -- --ssa-pass "Removing Unreachable Functions"
//! ```
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use clap::Parser;
use fm::FileManager;
use nargo::{
    insert_all_files_for_workspace_into_file_manager, package::Package, parse_all, prepare_package,
    workspace::Workspace,
};
use nargo_toml::{
    ManifestError, PackageSelection, get_package_manifest, resolve_workspace_from_toml,
};
use noirc_driver::{
    CompilationResult, CompileOptions, CrateName, NOIR_ARTIFACT_VERSION_STRING, check_crate,
};
use noirc_errors::CustomDiagnostic;
use noirc_evaluator::{
    errors::RuntimeError,
    ssa::{SsaPass, primary_passes, ssa_gen},
};
use noirc_frontend::{
    debug::DebugInstrumenter,
    elaborator::UnstableFeature,
    hir::ParsedFiles,
    monomorphization::{ast::Program, monomorphize},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use regex::Regex;
use similar::{ChangeTag, DiffableStr, DiffableStrRef, TextDiff};

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

    /// Inliner aggressiveness to use in SSA passes.
    #[arg(long, default_value = "0")]
    inliner_aggressiveness: i64,

    /// Show the top N most impacted program passes.
    #[arg(long, default_value = "50")]
    top_impact_count: usize,
}

fn main() {
    let opts = Options::parse();
    let sel = PackageSelection::DefaultOrAll;

    let test_workspaces = read_test_program_dirs(&test_programs_dir(), "execution_success")
        .filter_map(|dir| read_workspace(&dir, sel.clone()).ok())
        .collect::<Vec<_>>();

    let compile_options = CompileOptions {
        // Keep this up to date with whatever features are required by the integration tests.
        unstable_features: vec![UnstableFeature::Enums],
        silence_warnings: true,
        skip_underconstrained_check: true,
        skip_brillig_constraints_check: true,
        inliner_aggressiveness: opts.inliner_aggressiveness,
        ..Default::default()
    };

    let ssa_options = compile_options.as_ssa_options(PathBuf::new());

    let last_pass = primary_passes(&ssa_options)
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.msg().contains(&opts.ssa_pass).then_some(i))
        .max()
        .expect("cannot find a pass with the given name");

    // Note that instead of compiling the code and running SSA passes one by one,
    // we could work with the snapshots exported in https://github.com/noir-lang/noir/pull/7853 (currently draft),
    // and focus on just the string comparison part. That would have the benefit
    // of doing 100% what the normal compilation pipeline does, and that once the
    // snapshots are prepared, we can compare any pairs at will, rather than have
    // to recompile to look at another pass.

    let ssa_pairs: Vec<Vec<(CrateName, Vec<SsaBeforeAndAfter>)>> = test_workspaces
        .into_par_iter()
        .map(|workspace| {
            let mut workspace_pairs = Vec::new();

            let mut file_manager = workspace.new_file_manager();
            insert_all_files_for_workspace_into_file_manager(&workspace, &mut file_manager);
            let parsed_files = parse_all(&file_manager);
            let binary_packages = workspace.into_iter().filter(|package| package.is_binary());

            // Cannot share the boxed closures between threads.
            let ssa_passes = primary_passes(&ssa_options);

            for package in binary_packages {
                let program = match compile_into_program(
                    &file_manager,
                    &parsed_files,
                    &workspace,
                    package,
                    &compile_options,
                ) {
                    Ok((Some(program), _)) => program,
                    Ok((None, _)) => continue,
                    Err(_) => {
                        eprintln!("failed to compile {}", package.name);
                        continue;
                    }
                };

                let package_pairs = collect_ssa_before_and_after(
                    program,
                    &ssa_passes[..=last_pass],
                    &opts.ssa_pass,
                )
                .unwrap_or_else(|e| panic!("failed to run SSA passes on {}: {e}", package.name));

                if !package_pairs.is_empty() {
                    workspace_pairs.push((package.name.clone(), package_pairs));
                }
            }

            workspace_pairs
        })
        .collect();

    let ssa_pairs = ssa_pairs.into_iter().flatten().collect();

    show_report(ssa_pairs, opts.top_impact_count);
}

/// Show the impact on the console.
fn show_report(pairs: Vec<(CrateName, Vec<SsaBeforeAndAfter>)>, top_impact_count: usize) {
    let package_cnt = pairs.len();
    let mut total_cnt = 0;
    let mut equals_cnt = 0;
    let mut passes_by_name: BTreeMap<String, Vec<(f64, CrateName, SsaBeforeAndAfter)>> =
        Default::default();

    for (package, passes) in pairs {
        total_cnt += passes.len();
        for pass in passes {
            if pass.before.ssa == pass.after.ssa {
                equals_cnt += 1;
            } else {
                let sim = ssa_similarity(&pass.before.ssa, &pass.after.ssa);
                let passes = passes_by_name.entry(pass.after.msg.clone()).or_default();
                passes.push((sim, package.clone(), pass));
            }
        }
    }

    for passes in passes_by_name.values_mut() {
        passes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    }

    println!("Packages: {package_cnt}");
    println!("Passes total: {total_cnt}");
    println!("Passes with no impact: {equals_cnt}");

    for (name, passes) in passes_by_name {
        println!("Passes most impacted by '{name}' (top {top_impact_count}):");
        for (sim, package, pass) in passes.into_iter().take(top_impact_count) {
            println!(
                "\t{:.3} impact: step {} following '{}' in {package}",
                1.0 - sim,
                pass.after.step,
                pass.before.msg,
            );
        }
    }
}

/// Compile a package into a monomorphized [Program].
///
/// If the package has no `main` function then `None` is returned.
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
    let (_, warnings) = check_crate(&mut context, crate_id, options)?;
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
        let before =
            pass.msg().contains(name).then(|| format!("{}", ssa.print_without_locations()));
        ssa = pass.run(ssa)?;
        if let Some(before) = before {
            pairs.push(SsaBeforeAndAfter {
                before: SsaPrint { step: i, msg: last_msg.to_string(), ssa: before },
                after: SsaPrint {
                    step: i + 1,
                    msg: pass.msg().to_string(),
                    ssa: format!("{}", ssa.print_without_locations()),
                },
            });
        }
        last_msg = pass.msg();
    }

    Ok(pairs)
}

/// Remove identifiers from the SSA, so we can compare the structure without
/// worrying about trivial differences like changing IDs of the same variable
/// between one pass to the next.
fn sanitize_ssa(ssa: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Capture function ID, value IDs, global IDs.
    let re = RE.get_or_init(|| Regex::new(r#"(f|b|v|g)\d+"#).expect("ID regex failed"));
    re.replace_all(ssa, "${1}_").into_owned()
}

/// Calculate a similarity metric between two SSA strings, ignoring the difference in ID allocation.
fn ssa_similarity(ssa1: &str, ssa2: &str) -> f64 {
    if ssa1.is_empty() && ssa2.is_empty() {
        return 1.0;
    }
    let ssa1 = sanitize_ssa(ssa1);
    let ssa2 = sanitize_ssa(ssa2);

    let equals = TextDiff::from_lines(&ssa1, &ssa2)
        .iter_all_changes()
        .filter(|c| c.tag() == ChangeTag::Equal)
        .count() as f64;

    let lines1 = ssa1.as_diffable_str().tokenize_lines().len();
    let lines2 = ssa2.as_diffable_str().tokenize_lines().len();

    (2.0 * equals) / ((lines1 + lines2) as f64)
}

/// These tests can be executed with:
/// ```ignore
/// cargo test -p nargo_cli --example ssa_pass_impact
/// ```
#[cfg(test)]
mod tests {
    use crate::{sanitize_ssa, ssa_similarity};

    const SAMPLE_SSA: &str = r#"
        g0 = i8 114
        g1 = make_array [i8 114, u32 2354179802, i8 37, i8 179, u32 1465519558, i8 87] : [(i8, u32, i8); 2]

        acir(inline) fn main f0 {
        b0(v7: i8, v8: u32, v9: i8, v10: [(i8, i8, u1, u1, [u8; 0]); 2]):
            v17 = allocate -> &mut u32
            store u32 25 at v17
            v19 = cast v9 as i64
            v21 = array_get v10, index u32 5 -> i8
            v23 = array_get v10, index u32 6 -> i8
            v25 = array_get v10, index u32 7 -> u1
            v27 = array_get v10, index u32 8 -> u1
            v29 = array_get v10, index u32 9 -> [u8; 0]
            v30 = cast v23 as i64
            v31 = lt v30, v19
            v32 = not v31
            jmpif v32 then: b1, else: b2
        "#;

    #[test]
    fn test_sanitize_ssa() {
        let ssa = sanitize_ssa(SAMPLE_SSA);

        similar_asserts::assert_eq!(
            ssa,
            r#"
        g_ = i8 114
        g_ = make_array [i8 114, u32 2354179802, i8 37, i8 179, u32 1465519558, i8 87] : [(i8, u32, i8); 2]

        acir(inline) fn main f_ {
        b_(v_: i8, v_: u32, v_: i8, v_: [(i8, i8, u1, u1, [u8; 0]); 2]):
            v_ = allocate -> &mut u32
            store u32 25 at v_
            v_ = cast v_ as i64
            v_ = array_get v_, index u32 5 -> i8
            v_ = array_get v_, index u32 6 -> i8
            v_ = array_get v_, index u32 7 -> u1
            v_ = array_get v_, index u32 8 -> u1
            v_ = array_get v_, index u32 9 -> [u8; 0]
            v_ = cast v_ as i64
            v_ = lt v_, v_
            v_ = not v_
            jmpif v_ then: b_, else: b_
        "#
        )
    }

    #[test]
    fn test_ssa_similarity() {
        assert_eq!(1.0, ssa_similarity(SAMPLE_SSA, SAMPLE_SSA), "similar to self");
        assert_eq!(1.0, ssa_similarity("", ""), "empty is similar");

        let s = ssa_similarity(
            SAMPLE_SSA,
            r#"
        g0 = i8 114
        g1 = make_array [i8 114, u32 2354179802, i8 37, i8 179, u32 1465519558, i8 87] : [(i8, u32, i8); 2]

        acir(inline) fn main f0 {
        b0(v7: i8, v8: u32, v9: i8, v10: [(i8, i8, u1, u1, [u8; 0]); 2]):
            v18 = array_get v10, index u32 0 -> i8
            v20 = array_get v10, index u32 1 -> i8
            v22 = array_get v10, index u32 2 -> u1
            v24 = array_get v10, index u32 3 -> u1
            v26 = array_get v10, index u32 4 -> [u8; 0]
            v28 = array_get v10, index u32 5 -> i8
            v30 = array_get v10, index u32 6 -> i8
            v32 = array_get v10, index u32 7 -> u1
            v34 = array_get v10, index u32 8 -> u1
            v36 = array_get v10, index u32 9 -> [u8; 0]
            v37 = make_array [v18, v20, v22, v24, v28, v30, v32, v34] : [Field; 8]
            v38 = allocate -> &mut u32
            store u32 25 at v38
            v40 = cast v9 as i64
            v41 = array_get v10, index u32 5 -> i8
            v42 = array_get v10, index u32 6 -> i8
            v43 = array_get v10, index u32 7 -> u1
            v44 = array_get v10, index u32 8 -> u1
            v45 = array_get v10, index u32 9 -> [u8; 0]
            v46 = cast v42 as i64
            v47 = lt v46, v40
            v48 = not v47
            jmpif v48 then: b1, else: b2
        "#,
        );
        assert!(0.0 < s && s < 1.0, "somewhat similar with insertions")
    }
}
