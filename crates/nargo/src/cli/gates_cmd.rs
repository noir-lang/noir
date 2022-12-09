use std::collections::HashMap;

use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use std::path::Path;

use crate::cli::compile_cmd::compile_circuit;
use crate::errors::CliError;

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("gates").unwrap();
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");
    count_gates(show_ssa, allow_warnings)
}

pub fn count_gates(show_ssa: bool, allow_warnings: bool) -> Result<(), CliError> {
    let curr_dir = std::env::current_dir().unwrap();
    count_gates_with_path(curr_dir, show_ssa, allow_warnings)
}

pub fn count_gates_with_path<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<(), CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
    let gates = compiled_program.circuit.gates.clone();

    // Store counts of each gate type into hashmap.
    let mut gate_counts: HashMap<&str, u32> = HashMap::new();
    for gate in gates.iter() {
        *gate_counts.entry(gate.name()).or_default() += 1;
    }

    // Sort gates by name alphabetically for consistent display.
    let mut sorted_gate_counts: Vec<(&str, u32)> = gate_counts.into_iter().collect();
    sorted_gate_counts.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    println!("Gates successfully counted\n");

    println!("Total gates: {}\n", gates.len());

    println!("By type:");

    for (gate_type, count) in sorted_gate_counts {
        println!("{}: {}", gate_type, count);
    }

    let backend = crate::backends::ConcreteBackend;
    let exact_circuit_size = backend.get_exact_circuit_size(compiled_program.circuit);
    println!("\nExact constraint size: {}\n", exact_circuit_size);

    Ok(())
}
