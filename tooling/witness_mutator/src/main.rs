//! `noir-witness-mutator`: report second witnesses of a compiled Noir program.

use clap::Parser;
use color_eyre::eyre::{Context, Result, bail};
use noir_artifact_cli::{Artifact, fs::inputs::read_inputs_from_file};
use noir_witness_mutator::{OracleConfig, Report, Severity, search, source::location_of};
use noirc_artifacts::program::CompiledProgram;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the JSON build artifact, as produced by `nargo compile`.
    #[clap(long, short)]
    artifact_path: PathBuf,

    /// Path to the Prover.toml holding the program's inputs.
    #[clap(long, short)]
    prover_file: PathBuf,

    /// Name of the function to search, when the artifact is a contract.
    #[clap(long)]
    contract_fn: Option<String>,

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

    /// Stop after this many candidate witnesses.
    #[clap(long, default_value_t = 50_000)]
    max_candidates: usize,

    /// Print the full witness of each finding.
    #[clap(long, default_value_t = false)]
    verbose: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let artifact = Artifact::read_from_file(&args.artifact_path)
        .with_context(|| format!("reading {}", args.artifact_path.display()))?;

    let program: CompiledProgram = match artifact {
        Artifact::Program(program) => program.into(),
        Artifact::Contract(contract) => {
            let names = || contract.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
            let Some(ref name) = args.contract_fn else {
                bail!(
                    "this is a contract artifact; choose a function with --contract-fn. \
                     Available: {}",
                    names().join(", ")
                );
            };
            let Some(program) = contract.function_as_compiled_program(name) else {
                bail!("no function '{name}' in this contract. Available: {}", names().join(", "));
            };
            program
        }
    };

    let (input_map, _) = read_inputs_from_file(&args.prover_file, &program.abi)
        .with_context(|| format!("reading {}", args.prover_file.display()))?;
    let initial_witness = program.abi.encode(&input_map, None)?;

    let oracles = OracleConfig {
        resolver_url: args.oracle_resolver,
        root_path: args.oracle_root_dir,
        package_name: args.oracle_package_name,
    };

    let report = search(&program.program, initial_witness, args.max_candidates, &oracles)
        .map_err(|error| color_eyre::eyre::eyre!(error))?;

    print(&report, &program, args.verbose);

    // A non-zero status marks a circuit weaker than the program it was compiled from, which is
    // what a corpus run or a CI check wants to act on. A program that returns an unconstrained
    // value is the author's decision and is reported without failing the run.
    if report.compiler_bugs().next().is_some() {
        std::process::exit(1);
    }
    Ok(())
}

fn print(report: &Report, program: &CompiledProgram, verbose: bool) {
    println!(
        "{} hint call site(s), {} candidate witness(es) tried",
        report.sites, report.candidates_tried
    );
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
        println!("no second witness found");
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
