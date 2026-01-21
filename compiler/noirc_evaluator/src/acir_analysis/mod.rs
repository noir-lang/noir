//! ACIR Analysis Module for Underconstrained Circuit Detection
//!
//! This module provides analysis of ACIR circuits to detect underconstrained
//! circuits and non-determinism using formal methods, particularly cvc5 SMT solver.
//!
//! ## How it works
//!
//! An underconstrained circuit is one where a malicious prover can choose
//! different values for private inputs while keeping the same public inputs,
//! and still produce a valid proof.
//!
//! We check this by:
//! 1. Converting ACIR to SMT-LIB format
//! 2. For each PRIVATE witness, checking if it can have multiple valid values
//!    given FIXED public inputs
//! 3. If yes → the circuit is underconstrained (vulnerability!)
//!
//! ## Additional Checks
//!
//! - **Brillig outputs**: Witnesses from unconstrained functions
//! - **Hash outputs**: Witnesses from hash functions that may not be properly constrained
//! - **Disconnected witnesses**: Witnesses not reachable from public inputs/outputs
//! - **Unused witnesses**: Witnesses not used in any constraint

mod cvc5_solver;
mod acir_to_smt;
mod witness_analysis;

use acvm::acir::circuit::Circuit;
use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use noirc_artifacts::debug::DebugInfo;
use noirc_artifacts::ssa::{InternalBug, SsaReport};
use noirc_errors::call_stack::CallStack;
use std::collections::{BTreeMap, BTreeSet};

pub use cvc5_solver::{Cvc5Solver, SolverConfig, SmtSolver};
pub use acir_to_smt::{AcirToSmtConverter, SmtConfig};
pub use witness_analysis::{WitnessAnalyzer, WitnessIssue, WitnessIssueReport};

/// Configuration for ACIR analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Enable SMT-based freedom check (most thorough but slowest)
    pub enable_smt_check: bool,
    /// Enable Brillig output analysis
    pub enable_brillig_check: bool,
    /// Enable hash output analysis
    pub enable_hash_check: bool,
    /// Enable connectivity analysis
    pub enable_connectivity_check: bool,
    /// Timeout for each witness check in seconds
    pub timeout_per_witness: u64,
    /// Maximum number of witnesses to check with SMT (0 = unlimited)
    pub max_witnesses_to_check: usize,
    /// Enable verbose output
    pub verbose: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            enable_smt_check: true,
            enable_brillig_check: true,
            enable_hash_check: true,
            enable_connectivity_check: true,
            timeout_per_witness: 30,
            max_witnesses_to_check: 0, // unlimited
            verbose: false,
        }
    }
}

/// Main analyzer for ACIR circuits
pub struct AcirAnalyzer {
    /// cvc5 solver instance
    solver: Cvc5Solver,
    /// Converter from ACIR to SMT-LIB
    converter: AcirToSmtConverter,
    /// Witness analyzer
    witness_analyzer: WitnessAnalyzer,
    /// Analysis configuration
    config: AnalysisConfig,
}

impl AcirAnalyzer {
    /// Create a new ACIR analyzer
    pub fn new() -> Self {
        Self {
            solver: Cvc5Solver::new(),
            converter: AcirToSmtConverter::new(),
            witness_analyzer: WitnessAnalyzer::new(),
            config: AnalysisConfig::default(),
        }
    }

    /// Create with custom cvc5 path
    pub fn with_cvc5_path(cvc5_path: String) -> Self {
        Self {
            solver: Cvc5Solver::with_path(cvc5_path),
            converter: AcirToSmtConverter::new(),
            witness_analyzer: WitnessAnalyzer::new(),
            config: AnalysisConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(cvc5_path: Option<String>, config: AnalysisConfig) -> Self {
        let solver = match cvc5_path {
            Some(path) => Cvc5Solver::with_config(SolverConfig {
                cvc5_path: Some(path),
                timeout_seconds: config.timeout_per_witness,
                verbose: config.verbose,
            }),
            None => Cvc5Solver::with_config(SolverConfig {
                cvc5_path: None,
                timeout_seconds: config.timeout_per_witness,
                verbose: config.verbose,
            }),
        };
        
        Self {
            solver,
            converter: AcirToSmtConverter::new(),
            witness_analyzer: WitnessAnalyzer::new(),
            config,
        }
    }

    /// Analyze a circuit for underconstrained witnesses
    ///
    /// This is the main entry point for ACIR analysis.
    /// It checks if any PRIVATE witnesses can have arbitrary values
    /// while PUBLIC inputs remain fixed.
    ///
    /// If `debug_info` is provided, the reports will include proper source locations.
    pub fn analyze_underconstrained(
        &mut self,
        circuit: &Circuit<FieldElement>,
        debug_info: Option<&DebugInfo>,
        output_dir: Option<&std::path::Path>,
    ) -> Result<Vec<AcirAnalysisReport>, AnalysisError> {
        // Build witness -> call_stack mapping from debug info
        let witness_call_stacks = debug_info
            .map(|di| build_witness_call_stack_map(circuit, di))
            .unwrap_or_default();
        let mut reports = Vec::new();

        // 1. Convert ACIR to SMT-LIB format
        let smt_script = self.converter.convert(circuit)?;
        
        // Save intermediate files if output directory is provided
        self.save_intermediate_files(circuit, &smt_script, output_dir);

        // 2. Identify PUBLIC and PRIVATE witnesses
        let public_witnesses: BTreeSet<Witness> = circuit
            .public_parameters
            .0
            .iter()
            .chain(circuit.return_values.0.iter())
            .copied()
            .collect();
        
        let return_witnesses: BTreeSet<Witness> = circuit
            .return_values
            .0
            .iter()
            .copied()
            .collect();

        // 3. Run witness analyzer for Brillig, Hash, and connectivity checks
        if self.config.enable_brillig_check || self.config.enable_hash_check || self.config.enable_connectivity_check {
            let witness_reports = self.witness_analyzer.analyze_circuit(circuit);
            
            for report in witness_reports {
                // Skip public witnesses
                if public_witnesses.contains(&report.witness) {
                    continue;
                }
                
                let should_report = match &report.issue {
                    WitnessIssue::BrilligOutput => self.config.enable_brillig_check,
                    WitnessIssue::HashOutput(_) => self.config.enable_hash_check,
                    WitnessIssue::CryptoOutput(_) => self.config.enable_hash_check,
                    WitnessIssue::Disconnected => self.config.enable_connectivity_check,
                    WitnessIssue::Unused => true,
                };
                
                if should_report {
                    let call_stack = witness_call_stacks
                        .get(&report.witness)
                        .cloned()
                        .unwrap_or_default();
                    reports.push(AcirAnalysisReport::from_witness_issue(report, call_stack));
                }
            }
        }

        // 4. Find private witnesses that don't appear in any constraint at all
        // (quick check before expensive SMT)
        for i in 0..=circuit.current_witness_index {
            let witness = Witness::from(i);
            
            if public_witnesses.contains(&witness) {
                continue;
            }
            
            if !self.solver.witness_appears_in_constraints(witness, &smt_script) {
                // Check if already reported
                if !reports.iter().any(|r| r.witness() == Some(witness)) {
                    let call_stack = witness_call_stacks
                        .get(&witness)
                        .cloned()
                        .unwrap_or_default();
                    reports.push(AcirAnalysisReport::UnconstrainedWitness {
                        witness,
                        details: format!(
                            "Witness w{} is not used in any constraint at all",
                            witness.witness_index()
                        ),
                        call_stack,
                    });
                }
            }
        }

        // Collect already-reported witnesses to avoid duplicates
        let reported_witnesses: BTreeSet<Witness> = reports
            .iter()
            .filter_map(|r| r.witness())
            .collect();

        // 5. Check if circuit is satisfiable at all
        if self.config.enable_smt_check {
            match self.solver.check_sat(&smt_script) {
                Ok(true) => {} // Satisfiable, continue with checks
                Ok(false) => {
                    return Ok(vec![AcirAnalysisReport::Unsatisfiable]);
                }
                Err(e) => {
                    if self.config.verbose {
                        eprintln!("Warning: Could not check satisfiability: {}", e);
                    }
                }
            }
        }

        // 6. For each PRIVATE witness (that wasn't already reported),
        // check if it can have multiple values given FIXED public inputs
        if self.config.enable_smt_check {
            let mut checked_count = 0;
            
            for i in 0..=circuit.current_witness_index {
                // Check limit
                if self.config.max_witnesses_to_check > 0 && checked_count >= self.config.max_witnesses_to_check {
                    break;
                }
                
                let witness = Witness::from(i);
                
                // Skip PUBLIC witnesses
                if public_witnesses.contains(&witness) {
                    continue;
                }
                
                // Skip already reported
                if reported_witnesses.contains(&witness) {
                    continue;
                }
                
                // Skip return values
                if return_witnesses.contains(&witness) {
                    continue;
                }
                
                checked_count += 1;
                
                // THE KEY CHECK: Can this private witness have multiple valid values
                // while keeping public inputs fixed?
                match self.solver.check_private_witness_freedom(
                    witness,
                    &smt_script,
                    circuit,
                    &public_witnesses,
                ) {
                    Ok(true) => {
                        let call_stack = witness_call_stacks
                            .get(&witness)
                            .cloned()
                            .unwrap_or_default();
                        reports.push(AcirAnalysisReport::UnconstrainedWitness {
                            witness,
                            details: format!(
                                "Private witness w{} can have multiple valid values for same public inputs - \
                                prover can choose arbitrary value!",
                                witness.witness_index()
                            ),
                            call_stack,
                        });
                    }
                    Ok(false) => {
                        // UNSAT means witness is uniquely determined by public inputs - good!
                    }
                    Err(e) => {
                        if self.config.verbose {
                            eprintln!("Warning: Could not check witness w{}: {}", witness.witness_index(), e);
                        }
                    }
                }
            }
        }

        // 7. Check witness dependency graph for disconnected components
        if self.config.enable_connectivity_check {
            let disconnected = self.witness_analyzer.find_disconnected_witnesses(circuit);
            let already_reported: BTreeSet<Witness> = reports
                .iter()
                .filter_map(|r| r.witness())
                .collect();
                
            for witness in disconnected {
                if public_witnesses.contains(&witness) || already_reported.contains(&witness) {
                    continue;
                }
                let call_stack = witness_call_stacks
                    .get(&witness)
                    .cloned()
                    .unwrap_or_default();
                reports.push(AcirAnalysisReport::DisconnectedWitness {
                    witness,
                    details: format!(
                        "Private witness w{} is not reachable from public inputs/outputs",
                        witness.witness_index()
                    ),
                    call_stack,
                });
            }
        }

        Ok(reports)
    }

    /// Save intermediate files for debugging
    /// 
    /// Errors are logged but don't fail the analysis
    fn save_intermediate_files(
        &self,
        circuit: &Circuit<FieldElement>,
        smt_script: &str,
        output_dir: Option<&std::path::Path>,
    ) {
        let save_to = |dir: &std::path::Path| -> Result<(), std::io::Error> {
            let analysis_dir = dir.join("acir_analysis");
            std::fs::create_dir_all(&analysis_dir)?;
            
            let smt_file_path = analysis_dir.join(format!("{}.smt2", circuit.function_name));
            std::fs::write(&smt_file_path, smt_script)?;
            
            let acir_file_path = analysis_dir.join(format!("{}_acir.txt", circuit.function_name));
            let acir_info = format!(
                "Function: {}\n\
                Current witness index: {}\n\
                Opcodes: {}\n\
                Public parameters: {:?}\n\
                Return values: {:?}\n\
                Private parameters: {:?}\n\
                \n=== Opcodes ===\n{}",
                circuit.function_name,
                circuit.current_witness_index,
                circuit.opcodes.len(),
                circuit.public_parameters.0,
                circuit.return_values.0,
                circuit.private_parameters,
                circuit.opcodes.iter().enumerate()
                    .map(|(i, op)| format!("{}: {:?}", i, op))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            std::fs::write(&acir_file_path, acir_info)?;
            Ok(())
        };

        let target_dir = output_dir
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        
        if let Err(e) = save_to(&target_dir) {
            if self.config.verbose {
                eprintln!("Warning: Could not save intermediate files to {:?}: {}", target_dir, e);
            }
        }
    }
}

impl Default for AcirAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Analysis report for ACIR analysis
#[derive(Debug, Clone)]
pub enum AcirAnalysisReport {
    /// Witness that can have arbitrary values
    UnconstrainedWitness {
        witness: Witness,
        details: String,
        call_stack: CallStack,
    },
    /// Witness not connected to public inputs/outputs
    DisconnectedWitness {
        witness: Witness,
        details: String,
        call_stack: CallStack,
    },
    /// Witness from Brillig (unconstrained function) output
    BrilligOutput {
        witness: Witness,
        details: String,
        call_stack: CallStack,
    },
    /// Witness from hash function output
    HashOutput {
        witness: Witness,
        hash_function: String,
        details: String,
        call_stack: CallStack,
    },
    /// Witness from cryptographic operation
    CryptoOutput {
        witness: Witness,
        operation: String,
        details: String,
        call_stack: CallStack,
    },
    /// Circuit is unsatisfiable
    Unsatisfiable,
}

impl AcirAnalysisReport {
    /// Get the witness associated with this report (if any)
    pub fn witness(&self) -> Option<Witness> {
        match self {
            AcirAnalysisReport::UnconstrainedWitness { witness, .. } => Some(*witness),
            AcirAnalysisReport::DisconnectedWitness { witness, .. } => Some(*witness),
            AcirAnalysisReport::BrilligOutput { witness, .. } => Some(*witness),
            AcirAnalysisReport::HashOutput { witness, .. } => Some(*witness),
            AcirAnalysisReport::CryptoOutput { witness, .. } => Some(*witness),
            AcirAnalysisReport::Unsatisfiable => None,
        }
    }

    /// Get the call stack associated with this report (if any)
    /// Returns a reference to avoid unnecessary cloning
    pub fn call_stack(&self) -> Option<&CallStack> {
        match self {
            AcirAnalysisReport::UnconstrainedWitness { call_stack, .. } => Some(call_stack),
            AcirAnalysisReport::DisconnectedWitness { call_stack, .. } => Some(call_stack),
            AcirAnalysisReport::BrilligOutput { call_stack, .. } => Some(call_stack),
            AcirAnalysisReport::HashOutput { call_stack, .. } => Some(call_stack),
            AcirAnalysisReport::CryptoOutput { call_stack, .. } => Some(call_stack),
            AcirAnalysisReport::Unsatisfiable => None,
        }
    }

    /// Create from WitnessIssueReport with call stack
    pub fn from_witness_issue(report: WitnessIssueReport, call_stack: CallStack) -> Self {
        match report.issue {
            WitnessIssue::BrilligOutput => AcirAnalysisReport::BrilligOutput {
                witness: report.witness,
                details: report.details,
                call_stack,
            },
            WitnessIssue::HashOutput(name) => AcirAnalysisReport::HashOutput {
                witness: report.witness,
                hash_function: name,
                details: report.details,
                call_stack,
            },
            WitnessIssue::CryptoOutput(name) => AcirAnalysisReport::CryptoOutput {
                witness: report.witness,
                operation: name,
                details: report.details,
                call_stack,
            },
            WitnessIssue::Disconnected => AcirAnalysisReport::DisconnectedWitness {
                witness: report.witness,
                details: report.details,
                call_stack,
            },
            WitnessIssue::Unused => AcirAnalysisReport::UnconstrainedWitness {
                witness: report.witness,
                details: report.details,
                call_stack,
            },
        }
    }

    /// Convert to SsaReport for integration with compiler
    pub fn to_ssa_report(&self) -> SsaReport {
        match self {
            AcirAnalysisReport::UnconstrainedWitness { witness, details, call_stack } => {
                SsaReport::Bug(InternalBug::UnconstrainedWitness {
                    witness: *witness,
                    details: details.clone(),
                    call_stack: call_stack.clone(),
                })
            }
            AcirAnalysisReport::DisconnectedWitness { witness, details, call_stack } => {
                SsaReport::Bug(InternalBug::DisconnectedWitness {
                    witness: *witness,
                    details: details.clone(),
                    call_stack: call_stack.clone(),
                })
            }
            AcirAnalysisReport::BrilligOutput { witness, details, call_stack } => {
                // Use UnconstrainedWitness with descriptive details for Brillig outputs
                SsaReport::Bug(InternalBug::UnconstrainedWitness {
                    witness: *witness,
                    details: format!("[Brillig Output] {}", details),
                    call_stack: call_stack.clone(),
                })
            }
            AcirAnalysisReport::HashOutput { witness, hash_function, details, call_stack } => {
                // Use UnconstrainedWitness with descriptive details for hash outputs
                SsaReport::Bug(InternalBug::UnconstrainedWitness {
                    witness: *witness,
                    details: format!("[{} Hash Output] {}", hash_function, details),
                    call_stack: call_stack.clone(),
                })
            }
            AcirAnalysisReport::CryptoOutput { witness, operation, details, call_stack } => {
                // Use UnconstrainedWitness with descriptive details for crypto outputs
                SsaReport::Bug(InternalBug::UnconstrainedWitness {
                    witness: *witness,
                    details: format!("[{} Output] {}", operation, details),
                    call_stack: call_stack.clone(),
                })
            }
            AcirAnalysisReport::Unsatisfiable => {
                SsaReport::Bug(InternalBug::UnsatisfiableCircuit {
                    call_stack: CallStack::default(),
                })
            }
        }
    }
}

/// Analysis errors
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("SMT solver error: {0}")]
    SolverError(String),
    #[error("Conversion error: {0}")]
    ConversionError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Build a mapping from witness to call stack using debug info
///
/// For each witness, we find the first opcode that uses it and get
/// the call stack for that opcode from debug_info.
fn build_witness_call_stack_map(
    circuit: &Circuit<FieldElement>,
    debug_info: &DebugInfo,
) -> BTreeMap<Witness, CallStack> {
    use acvm::acir::circuit::opcodes::{BlackBoxFuncCall, FunctionInput};
    use acvm::acir::circuit::AcirOpcodeLocation;

    let mut result = BTreeMap::new();

    // Helper to get call stack for opcode index
    let get_call_stack = |opcode_idx: usize| -> CallStack {
        let loc = AcirOpcodeLocation::new(opcode_idx);
        debug_info
            .acir_opcode_location(&loc)
            .map(|locations| CallStack::from_iter(locations.into_iter()))
            .unwrap_or_default()
    };

    // Helper to collect witnesses from expression
    let witnesses_from_expr = |expr: &acvm::acir::native_types::Expression<FieldElement>| -> Vec<Witness> {
        let mut witnesses = Vec::new();
        for (_, w1, w2) in &expr.mul_terms {
            witnesses.push(*w1);
            witnesses.push(*w2);
        }
        for (_, w) in &expr.linear_combinations {
            witnesses.push(*w);
        }
        witnesses
    };

    // Process each opcode
    for (opcode_idx, opcode) in circuit.opcodes.iter().enumerate() {
        let witnesses: Vec<Witness> = match opcode {
            acvm::acir::circuit::Opcode::AssertZero(expr) => {
                witnesses_from_expr(expr)
            }
            acvm::acir::circuit::Opcode::BlackBoxFuncCall(bb) => {
                let mut ws = Vec::new();
                match bb {
                    BlackBoxFuncCall::RANGE { input, .. } => {
                        if let FunctionInput::Witness(w) = input {
                            ws.push(*w);
                        }
                    }
                    BlackBoxFuncCall::AND { lhs, rhs, output, .. }
                    | BlackBoxFuncCall::XOR { lhs, rhs, output, .. } => {
                        if let FunctionInput::Witness(w) = lhs { ws.push(*w); }
                        if let FunctionInput::Witness(w) = rhs { ws.push(*w); }
                        ws.push(*output);
                    }
                    BlackBoxFuncCall::Blake2s { inputs, outputs, .. }
                    | BlackBoxFuncCall::Blake3 { inputs, outputs, .. } => {
                        for input in inputs {
                            if let FunctionInput::Witness(w) = input { ws.push(*w); }
                        }
                        ws.extend(outputs.iter().copied());
                    }
                    BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs, .. } => {
                        for input in inputs {
                            if let FunctionInput::Witness(w) = input { ws.push(*w); }
                        }
                        ws.extend(outputs.iter().copied());
                    }
                    BlackBoxFuncCall::Sha256Compression { inputs, outputs, .. } => {
                        for input in inputs.iter() {
                            if let FunctionInput::Witness(w) = input { ws.push(*w); }
                        }
                        ws.extend(outputs.iter().copied());
                    }
                    BlackBoxFuncCall::Keccakf1600 { inputs, outputs, .. } => {
                        for input in inputs.iter() {
                            if let FunctionInput::Witness(w) = input { ws.push(*w); }
                        }
                        ws.extend(outputs.iter().copied());
                    }
                    BlackBoxFuncCall::EcdsaSecp256k1 { output, .. }
                    | BlackBoxFuncCall::EcdsaSecp256r1 { output, .. } => {
                        ws.push(*output);
                    }
                    BlackBoxFuncCall::MultiScalarMul { outputs, .. }
                    | BlackBoxFuncCall::EmbeddedCurveAdd { outputs, .. } => {
                        ws.push(outputs.0);
                        ws.push(outputs.1);
                        ws.push(outputs.2);
                    }
                    BlackBoxFuncCall::AES128Encrypt { outputs, .. } => {
                        ws.extend(outputs.iter().copied());
                    }
                    _ => {}
                }
                ws
            }
            acvm::acir::circuit::Opcode::MemoryOp { op, .. } => {
                let mut ws = witnesses_from_expr(&op.index);
                ws.extend(witnesses_from_expr(&op.value));
                ws.extend(witnesses_from_expr(&op.operation));
                ws
            }
            acvm::acir::circuit::Opcode::MemoryInit { init, .. } => {
                init.clone()
            }
            acvm::acir::circuit::Opcode::BrilligCall { inputs, outputs, .. } => {
                let mut ws = Vec::new();
                // Inputs
                for input in inputs {
                    match input {
                        acvm::acir::circuit::brillig::BrilligInputs::Single(expr) => {
                            ws.extend(witnesses_from_expr(expr));
                        }
                        acvm::acir::circuit::brillig::BrilligInputs::Array(exprs) => {
                            for expr in exprs {
                                ws.extend(witnesses_from_expr(expr));
                            }
                        }
                        acvm::acir::circuit::brillig::BrilligInputs::MemoryArray(_) => {}
                    }
                }
                // Outputs
                for output in outputs {
                    match output {
                        acvm::acir::circuit::brillig::BrilligOutputs::Simple(w) => {
                            ws.push(*w);
                        }
                        acvm::acir::circuit::brillig::BrilligOutputs::Array(arr) => {
                            ws.extend(arr.iter().copied());
                        }
                    }
                }
                ws
            }
            acvm::acir::circuit::Opcode::Call { inputs, outputs, .. } => {
                let mut ws: Vec<Witness> = inputs.iter().copied().collect();
                ws.extend(outputs.iter().copied());
                ws
            }
        };

        // For each witness in this opcode, if not already mapped, add mapping
        let call_stack = get_call_stack(opcode_idx);
        for witness in witnesses {
            result.entry(witness).or_insert_with(|| call_stack.clone());
        }
    }

    result
}
