//! `noir-witness-mutator`: report second witnesses of a compiled Noir program.

use clap::Parser;
use color_eyre::eyre::{Context, Result, bail};
use noir_artifact_cli::{Artifact, fs::inputs::read_inputs_from_file};
use noir_witness_mutator::{Report, Severity, search};
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
    let Artifact::Program(program) = artifact else {
        bail!("contract artifacts are not supported yet");
    };
    let program: CompiledProgram = program.into();

    let (input_map, _) = read_inputs_from_file(&args.prover_file, &program.abi)
        .with_context(|| format!("reading {}", args.prover_file.display()))?;
    let initial_witness = program.abi.encode(&input_map, None)?;

    let report = search(&program.program, initial_witness, args.max_candidates)
        .map_err(|error| color_eyre::eyre::eyre!(error))?;

    print(&report, args.verbose);

    // A non-zero status marks a circuit weaker than the program it was compiled from, which is
    // what a corpus run or a CI check wants to act on. A program that returns an unconstrained
    // value is the author's decision and is reported without failing the run.
    if report.compiler_bugs().next().is_some() {
        std::process::exit(1);
    }
    Ok(())
}

fn print(report: &Report, verbose: bool) {
    println!(
        "{} hint call site(s), {} candidate witness(es) tried",
        report.sites, report.candidates_tried
    );
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
            Severity::Intermediate => println!("  only intermediate witnesses change"),
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
