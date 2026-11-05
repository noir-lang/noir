//! `noir-witness-mutator`: report second witnesses of a compiled Noir program.

use clap::Parser;
use color_eyre::eyre::{Context, Result, bail};
use noir_artifact_cli::{Artifact, fs::inputs::read_inputs_from_file};
use noir_witness_mutator::{
    OracleConfig, Report, Severity, search, source::location_of, source::opcode_count,
};
use noirc_abi::InputMap;
use noirc_artifacts::program::CompiledProgram;
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the JSON build artifact, as produced by `nargo compile`.
    #[clap(long, short)]
    artifact_path: PathBuf,

    /// Path to a Prover.toml holding the program's inputs. Repeatable.
    ///
    /// Optional: a program whose `main` takes no parameters has nothing to supply.
    #[clap(long, short)]
    prover_file: Vec<PathBuf>,

    /// Directory of `.toml` input files, each searched in turn.
    ///
    /// One input proves little on its own — a second witness often exists only for some values —
    /// so checking several is the difference between a weak result and a useful one.
    #[clap(long)]
    inputs_dir: Option<PathBuf>,

    /// Name of the function to search, when the artifact is a contract.
    #[clap(long)]
    contract_fn: Option<String>,

    /// Search only this call site, as reported in a finding (`f0:op12`). Repeatable.
    #[clap(long)]
    site: Vec<String>,

    /// Search only call sites at this source location (`src/main.nr:42`). Repeatable.
    #[clap(long)]
    source_line: Vec<String>,

    /// JSON-RPC url of a host that answers the program's oracle calls.
    ///
    /// The host is asked only during the honest run; the search answers from a recording of it.
    #[clap(long)]
    oracle_resolver: Option<String>,

    /// Root directory reported to the oracle host.
    #[clap(long)]
    oracle_root_dir: Option<PathBuf>,

    /// Package name reported to the oracle host.
    #[clap(long)]
    oracle_package_name: Option<String>,

    /// Stop after this many candidate witnesses, per input.
    #[clap(long, default_value_t = 50_000)]
    max_candidates: usize,

    /// Emit machine-readable JSON instead of prose.
    #[clap(long, default_value_t = false)]
    json: bool,

    /// Print the full witness of each finding.
    #[clap(long, default_value_t = false)]
    verbose: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let program = load_program(&args)?;
    let targets = resolve_targets(&args, &program)?;
    let inputs = collect_inputs(&args)?;

    let oracles = OracleConfig {
        resolver_url: args.oracle_resolver.clone(),
        root_path: args.oracle_root_dir.clone(),
        package_name: args.oracle_package_name.clone(),
    };

    let mut runs: Vec<(String, Result<Report, String>)> = Vec::new();
    for input in &inputs {
        let label = input.as_ref().map_or("(no inputs)".to_string(), |p| p.display().to_string());
        let report = encode_inputs(&program, input.as_deref())
            .map_err(|error| error.to_string())
            .and_then(|initial_witness| {
                search(
                    &program.program,
                    initial_witness,
                    args.max_candidates,
                    &oracles,
                    targets.as_ref(),
                )
            });
        runs.push((label, report));
    }

    if args.json {
        println!("{:#}", json_report(&args, &program, &runs));
    } else {
        print_runs(&program, &runs, args.verbose);
    }

    // A non-zero status marks a circuit weaker than the program it was compiled from, which is
    // what a corpus run or a CI check wants to act on. A program that returns an unconstrained
    // value is the author's decision and is reported without failing the run.
    let found_bug = runs
        .iter()
        .any(|(_, report)| report.as_ref().is_ok_and(|r| r.compiler_bugs().next().is_some()));
    if found_bug {
        std::process::exit(1);
    }
    Ok(())
}

fn load_program(args: &Args) -> Result<CompiledProgram> {
    let artifact = Artifact::read_from_file(&args.artifact_path)
        .with_context(|| format!("reading {}", args.artifact_path.display()))?;

    match artifact {
        Artifact::Program(program) => Ok(program.into()),
        Artifact::Contract(contract) => {
            let names = || contract.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
            let Some(ref name) = args.contract_fn else {
                bail!(
                    "this is a contract artifact; choose a function with --contract-fn. \
                     Available: {}",
                    names().join(", ")
                );
            };
            contract.function_as_compiled_program(name).ok_or_else(|| {
                color_eyre::eyre::eyre!(
                    "no function '{name}' in this contract. Available: {}",
                    names().join(", ")
                )
            })
        }
    }
}

/// Opcode indices the search should be restricted to, or `None` to search everything.
fn resolve_targets(args: &Args, program: &CompiledProgram) -> Result<Option<HashSet<usize>>> {
    if args.site.is_empty() && args.source_line.is_empty() {
        return Ok(None);
    }

    let mut targets = HashSet::new();
    for index in 0..opcode_count(&program.program, 0) {
        let label = format!("f0:op{index}");
        let location = location_of(program, 0, index);

        let wanted = args.site.contains(&label)
            || args.source_line.iter().any(|selector| {
                location.as_ref().is_some_and(|found| found.ends_with(selector.as_str()))
            });
        if wanted {
            targets.insert(index);
        }
    }

    if targets.is_empty() {
        bail!("no opcode matches the requested --site / --source-line");
    }
    Ok(Some(targets))
}

/// Every input set to search, in order. `None` means "this program takes no parameters".
fn collect_inputs(args: &Args) -> Result<Vec<Option<PathBuf>>> {
    let mut inputs: Vec<Option<PathBuf>> = args.prover_file.iter().cloned().map(Some).collect();

    if let Some(dir) = &args.inputs_dir {
        let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
            .with_context(|| format!("reading {}", dir.display()))?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        found.sort();
        if found.is_empty() {
            bail!("no .toml input files in {}", dir.display());
        }
        inputs.extend(found.into_iter().map(Some));
    }

    if inputs.is_empty() {
        inputs.push(None);
    }
    Ok(inputs)
}

fn encode_inputs(
    program: &CompiledProgram,
    prover_file: Option<&Path>,
) -> Result<acir::native_types::WitnessMap<acir::FieldElement>> {
    let input_map = match prover_file {
        Some(path) => {
            read_inputs_from_file(path, &program.abi)
                .with_context(|| format!("reading {}", path.display()))?
                .0
        }
        None if program.abi.parameters.is_empty() => InputMap::new(),
        None => bail!(
            "this program takes {} input(s); pass --prover-file",
            program.abi.parameters.len()
        ),
    };
    Ok(program.abi.encode(&input_map, None)?)
}

fn json_report(
    args: &Args,
    program: &CompiledProgram,
    runs: &[(String, Result<Report, String>)],
) -> Value {
    let runs_json: Vec<Value> = runs
        .iter()
        .map(|(input, report)| match report {
            Err(error) => json!({ "input": input, "status": "failed", "error": error }),
            Ok(report) => json!({
                "input": input,
                "status": report.status.label(),
                "hint_sites": report.sites,
                "candidates_tried": report.candidates_tried,
                "oracle_calls": report.oracle_calls,
                "has_return_values": report.has_return_values,
                "findings": report.findings.iter().map(|finding| json!({
                    "grade": finding.severity().label(),
                    "site": finding.site.label(),
                    "hint": finding.site.kind.name(),
                    "source": location_of(program, finding.site.function_index,
                                          finding.site.opcode_index),
                    "strategy": finding.strategy,
                    "changes_return_value": finding.changes_return,
                    "blast_radius": finding.blast_radius,
                    "oracle_divergences": finding.oracle_divergences,
                    "changed_outputs": finding.changed_outputs.iter().map(|(w, honest, second)| json!({
                        "witness": w.0,
                        "honest": honest.to_string(),
                        "second_witness": second.to_string(),
                    })).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
                "coverage": report.coverage.iter().map(|site| json!({
                    "site": site.label,
                    "hint": site.hint,
                    "outputs": site.outputs,
                    "candidates_tried": site.candidates_tried,
                })).collect::<Vec<_>>(),
            }),
        })
        .collect();

    let truncated: Vec<&String> = runs
        .iter()
        .filter(|(_, report)| {
            report.as_ref().is_ok_and(|r| r.status == noir_witness_mutator::RunStatus::Truncated)
        })
        .map(|(input, _)| input)
        .collect();
    let unexercised: usize = runs
        .iter()
        .filter_map(|(_, report)| report.as_ref().ok())
        .map(|report| report.coverage.iter().filter(|s| s.candidates_tried == 0).count())
        .sum();

    json!({
        "tool": "noir-witness-mutator",
        "artifact": args.artifact_path.display().to_string(),
        "runs": runs_json,
        // Stated as data, not prose: a caller summarising this run will repeat whatever structure
        // it is given, and the limits of the result are part of the result.
        "caveats": {
            "absence_not_proven": true,
            "explanation": "a finding is a witness the solver accepted, so it is conclusive; \
                            finding nothing is not, because the candidate values are a heuristic \
                            and only the given inputs were searched",
            "inputs_checked": runs.len(),
            "truncated_runs": truncated,
            "sites_never_exercised": unexercised,
            "max_candidates": args.max_candidates,
        }
    })
}

fn print_runs(program: &CompiledProgram, runs: &[(String, Result<Report, String>)], verbose: bool) {
    for (input, report) in runs {
        if runs.len() > 1 {
            println!("\n=== {input}");
        }
        match report {
            Err(error) => println!("could not search: {error}"),
            Ok(report) => print(report, program, verbose),
        }
    }

    if runs.len() > 1 {
        let found = runs
            .iter()
            .filter(|(_, report)| report.as_ref().is_ok_and(|r| !r.findings.is_empty()))
            .count();
        println!("\n{found} of {} input(s) produced a finding", runs.len());
    }
}

fn print(report: &Report, program: &CompiledProgram, verbose: bool) {
    println!(
        "{} hint call site(s), {} candidate witness(es) tried ({})",
        report.sites,
        report.candidates_tried,
        report.status.label()
    );
    if report.status == noir_witness_mutator::RunStatus::Truncated {
        println!(
            "  the candidate budget ran out before the candidates did: this run has ruled nothing \
             out. Raise --max-candidates or narrow the search with --site"
        );
    }
    let unexercised = report.coverage.iter().filter(|site| site.candidates_tried == 0).count();
    if unexercised > 0 {
        println!(
            "  {unexercised} of {} site(s) had no candidate tried, so nothing is known about them",
            report.sites
        );
    }
    if report.oracle_calls > 0 {
        println!(
            "{} oracle call(s) recorded during the honest run and replayed for every candidate",
            report.oracle_calls
        );
    }
    if !report.has_return_values {
        println!(
            "this circuit returns nothing, so a verifier has no value to compare: every finding \
             below is a question about what the circuit claims to prove"
        );
    }
    if report.findings.is_empty() {
        println!("no second witness found among the candidates tried");
        return;
    }

    for finding in &report.findings {
        println!(
            "\n{} second witness at {} in {} (strategy: {})",
            finding.severity().label(),
            finding.site.label(),
            finding.site.kind.name(),
            finding.strategy
        );
        match finding.severity() {
            Severity::CompilerBug => println!(
                "  a return value changes: the constraints the compiler emitted for this hint do \
                 not pin its outputs down"
            ),
            Severity::ProgramUnderconstrained => println!(
                "  a return value changes: the program returns this unconstrained call's output \
                 without constraining it"
            ),
            Severity::WitnessNotUnique => println!(
                "  no return value changes, but {} other witness(es) do: the proof does not pin \
                 down what the program computed. Whether that is exploitable depends on what this \
                 circuit is meant to prove",
                finding.blast_radius
            ),
            Severity::Inert => println!(
                "  only this call's own outputs change, and nothing else in the witness follows: \
                 the value is not read"
            ),
        }
        if finding.oracle_divergences > 0 {
            println!(
                "  {} oracle call(s) would be made with different arguments, so the host sees \
                 different traffic for the same proof",
                finding.oracle_divergences
            );
        }
        if let Some(location) =
            location_of(program, finding.site.function_index, finding.site.opcode_index)
        {
            println!("  at {location}");
        }
        for (witness, honest, mutated) in &finding.changed_outputs {
            println!("  w{}: honest {} -> {}", witness.0, honest, mutated);
        }
        if verbose {
            let mut entries: Vec<_> = finding.witness.clone().into_iter().collect();
            entries.sort_by_key(|(witness, _)| witness.0);
            for (witness, value) in entries {
                println!("    w{} = {}", witness.0, value);
            }
        }
    }
}
