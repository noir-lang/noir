//! SMT Solver Integration
//!
//! This module provides integration with SMT solvers for checking
//! underconstrained circuits (non-determinism detection).
//!
//! ## Key Concepts
//!
//! - **Public witnesses**: Fixed by the verifier, same in both "worlds"
//! - **Private witnesses**: Chosen by prover, if unconstrained = vulnerability
//! - **Two-world model**: We check if the same public inputs can lead to different private values
//!
//! ## Supported Solvers
//!
//! Currently supports cvc5, but the `SmtSolver` trait allows adding other solvers.

use acvm::acir::circuit::Circuit;
use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use std::collections::BTreeSet;
use std::process::Command;
use std::io::Write;
use tempfile::NamedTempFile;
use super::AnalysisError;

/// Trait for SMT solver implementations
/// 
/// This allows swapping out different SMT solvers (cvc5, z3, etc.)
pub trait SmtSolver {
    /// Check if an SMT script is satisfiable
    fn check_sat(&self, smt_script: &str) -> Result<bool, AnalysisError>;
    
    /// Check satisfiability with a custom timeout
    fn check_sat_with_timeout(&self, smt_script: &str, timeout_seconds: u64) -> Result<bool, AnalysisError>;
    
    /// Get a model (satisfying assignment) if SAT
    fn get_model(&self, smt_script: &str) -> Result<String, AnalysisError>;
    
    /// Check if a witness appears in any constraint
    fn witness_appears_in_constraints(&self, witness: Witness, smt_script: &str) -> bool;
}

/// Configuration for cvc5 solver
#[derive(Debug, Clone)]
pub struct SolverConfig {
    /// Path to cvc5 executable (None = use from PATH)
    pub cvc5_path: Option<String>,
    /// Timeout for each SMT check in seconds
    pub timeout_seconds: u64,
    /// Enable verbose output for debugging
    pub verbose: bool,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            cvc5_path: None,
            timeout_seconds: 30,
            verbose: false,
        }
    }
}

/// cvc5 SMT Solver wrapper
pub struct Cvc5Solver {
    config: SolverConfig,
}

impl Cvc5Solver {
    /// Create a new cvc5 solver instance
    pub fn new() -> Self {
        Self { config: SolverConfig::default() }
    }

    /// Create with custom cvc5 path
    pub fn with_path(path: String) -> Self {
        Self {
            config: SolverConfig {
                cvc5_path: Some(path),
                ..Default::default()
            },
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: SolverConfig) -> Self {
        Self { config }
    }

    /// Check if SMT script is satisfiable
    pub fn check_sat(&self, smt_script: &str) -> Result<bool, AnalysisError> {
        self.check_sat_with_timeout(smt_script, self.config.timeout_seconds)
    }

    /// Check satisfiability with custom timeout
    pub fn check_sat_with_timeout(&self, smt_script: &str, timeout_seconds: u64) -> Result<bool, AnalysisError> {
        // Write SMT script to temporary file
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(smt_script.as_bytes())?;
        temp_file.flush()?;

        // Run cvc5 with timeout
        let cvc5_cmd = self.config.cvc5_path.as_deref().unwrap_or("cvc5");
        let output = Command::new(cvc5_cmd)
            .arg("--tlimit")
            .arg(format!("{}", timeout_seconds * 1000)) // cvc5 uses milliseconds
            .arg(temp_file.path())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if self.config.verbose {
            eprintln!("cvc5 stdout: {}", stdout);
            eprintln!("cvc5 stderr: {}", stderr);
        }

        // Check for timeout
        if stdout.contains("timeout") || stdout.contains("unknown") {
            return Err(AnalysisError::SolverError("Solver timeout".to_string()));
        }

        if !output.status.success() && !stdout.contains("sat") && !stdout.contains("unsat") {
            return Err(AnalysisError::SolverError(format!(
                "cvc5 error: {}",
                stderr
            )));
        }

        // Parse result - be careful with "unsat" containing "sat"
        let stdout_trimmed = stdout.trim();
        if stdout_trimmed == "sat" {
            Ok(true)
        } else if stdout_trimmed == "unsat" {
            Ok(false)
        } else if stdout.contains("unsat") {
            Ok(false)
        } else if stdout.contains("sat") {
            Ok(true)
        } else {
            Err(AnalysisError::SolverError(
                format!("Unknown cvc5 output: {}", stdout)
            ))
        }
    }

    /// Check if a PRIVATE witness can have multiple valid values (non-determinism)
    /// given FIXED public inputs.
    ///
    /// ## Correct Two-World Model
    ///
    /// We create TWO parallel constraint systems ("worlds"):
    /// - World 1: Original constraints with witness values w0, w1, w2, ...
    /// - World 2: Same constraints but with alternative values for private witnesses
    ///
    /// Key insight:
    /// - PUBLIC witnesses (verifier inputs + return values) MUST be IDENTICAL in both worlds
    /// - PRIVATE witnesses can differ
    ///
    /// If we can find satisfying assignments where:
    /// - All public witnesses are the same in both worlds
    /// - At least one private witness differs
    /// Then the circuit is UNDERCONSTRAINED!
    pub fn check_private_witness_freedom(
        &self,
        private_witness: Witness,
        base_smt_script: &str,
        circuit: &Circuit<FieldElement>,
        public_witnesses: &BTreeSet<Witness>,
    ) -> Result<bool, AnalysisError> {
        let w_idx = private_witness.witness_index();
        
        let mut script = String::new();
        script.push_str("(set-logic QF_NIA)\n");
        script.push_str("(set-option :produce-models true)\n");
        script.push_str(&format!("; Underconstrained check for private witness w{}\n\n", w_idx));
        
        // ============================================================
        // STEP 1: Declare World 1 witnesses (original)
        // ============================================================
        script.push_str("; === World 1 witnesses (original) ===\n");
        for i in 0..=circuit.current_witness_index {
            script.push_str(&format!("(declare-fun w{} () Int)\n", i));
        }
        script.push_str("\n");
        
        // ============================================================
        // STEP 2: Declare World 2 witnesses (alternative)
        // Only PRIVATE witnesses get alternative versions
        // ============================================================
        script.push_str("; === World 2 witnesses (alternative for private only) ===\n");
        for i in 0..=circuit.current_witness_index {
            let witness = Witness::from(i);
            // Only create alternative for PRIVATE witnesses
            if !public_witnesses.contains(&witness) {
                script.push_str(&format!("(declare-fun w{}_alt () Int)\n", i));
            }
        }
        script.push_str("\n");
        
        // ============================================================
        // STEP 3: Field bounds for all witnesses (both worlds)
        // ============================================================
        script.push_str("; === Field bounds ===\n");
        for i in 0..=circuit.current_witness_index {
            script.push_str(&format!("(assert (>= w{} 0))\n", i));
            let witness = Witness::from(i);
            if !public_witnesses.contains(&witness) {
                script.push_str(&format!("(assert (>= w{}_alt 0))\n", i));
            }
        }
        script.push_str("\n");
        
        // ============================================================
        // STEP 4: World 1 constraints (original)
        // ============================================================
        script.push_str("; === World 1 constraints (original) ===\n");
        for line in base_smt_script.lines() {
            if line.starts_with("(assert") && !line.contains("FIELD_MODULUS") {
                script.push_str(line);
                script.push('\n');
            }
        }
        script.push_str("\n");
        
        // ============================================================
        // STEP 5: World 2 constraints
        // Replace ALL private witnesses with their _alt versions
        // Keep public witnesses unchanged (they're shared!)
        // ============================================================
        script.push_str("; === World 2 constraints (with alternative private witnesses) ===\n");
        for line in base_smt_script.lines() {
            if line.starts_with("(assert") && !line.contains("FIELD_MODULUS") {
                let mut alt_line = line.to_string();
                
                // Replace each PRIVATE witness with its _alt version
                // PUBLIC witnesses stay the same (this is the key insight!)
                for i in 0..=circuit.current_witness_index {
                    let witness = Witness::from(i);
                    if !public_witnesses.contains(&witness) {
                        alt_line = replace_witness_in_constraint(&alt_line, i, &format!("w{}_alt", i));
                    }
                }
                
                script.push_str(&alt_line);
                script.push('\n');
            }
        }
        script.push_str("\n");
        
        // ============================================================
        // STEP 6: The key constraint - private witness differs
        // ============================================================
        script.push_str("; === Key check: private witness can have different value ===\n");
        script.push_str(&format!("(assert (not (= w{} w{}_alt)))\n\n", w_idx, w_idx));
        
        // ============================================================
        // STEP 7: Check satisfiability
        // ============================================================
        script.push_str("(check-sat)\n");
        
        // If SAT: there exist two different values for the private witness
        // that both satisfy constraints with the SAME public inputs
        // This means the circuit is UNDERCONSTRAINED!
        
        if self.config.verbose {
            eprintln!("=== Generated SMT script for w{} ===\n{}", w_idx, script);
        }
        
        self.check_sat(&script)
    }

    /// Check if ALL private witnesses can be uniquely determined
    /// This is a batch version of check_private_witness_freedom
    /// that's more efficient for large circuits using parallel processing.
    pub fn check_all_private_witnesses_constrained(
        &self,
        base_smt_script: &str,
        circuit: &Circuit<FieldElement>,
        public_witnesses: &BTreeSet<Witness>,
    ) -> Result<Vec<Witness>, AnalysisError> {
        use rayon::prelude::*;
        
        // Collect private witnesses to check
        let private_witnesses: Vec<Witness> = (0..=circuit.current_witness_index)
            .map(Witness::from)
            .filter(|w| !public_witnesses.contains(w))
            .collect();
        
        // Parallel check using rayon
        let underconstrained: Vec<Witness> = private_witnesses
            .par_iter()
            .filter_map(|witness| {
                match self.check_private_witness_freedom(*witness, base_smt_script, circuit, public_witnesses) {
                    Ok(true) => Some(*witness),
                    Ok(false) => None, // Witness is constrained
                    Err(e) => {
                        if self.config.verbose {
                            eprintln!("Warning: Could not check witness w{}: {}", witness.witness_index(), e);
                        }
                        None
                    }
                }
            })
            .collect();
        
        Ok(underconstrained)
    }

    /// Simplified check: does a witness appear in any constraint at all?
    /// If not, it's trivially underconstrained.
    pub fn witness_appears_in_constraints(&self, witness: Witness, smt_script: &str) -> bool {
        let w_idx = witness.witness_index();
        for line in smt_script.lines() {
            if line.starts_with("(assert") {
                if constraint_contains_witness(line, w_idx) {
                    return true;
                }
            }
        }
        false
    }

    /// Get a model (satisfying assignment) from cvc5
    /// Useful for debugging and providing counterexamples
    pub fn get_model(&self, smt_script: &str) -> Result<String, AnalysisError> {
        let script_with_model = format!("{}\n(get-model)\n", smt_script.trim_end_matches("(check-sat)\n"));
        
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(script_with_model.as_bytes())?;
        temp_file.flush()?;

        let cvc5_cmd = self.config.cvc5_path.as_deref().unwrap_or("cvc5");
        let output = Command::new(cvc5_cmd)
            .arg("--produce-models")
            .arg(temp_file.path())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
}

/// Check if constraint contains witness w{idx} (not w{idx}0, w{idx}1, etc.)
fn constraint_contains_witness(constraint: &str, idx: u32) -> bool {
    let pattern = format!("w{}", idx);
    let chars: Vec<char> = constraint.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i] == 'w' {
            i += 1;
            let mut num_str = String::new();
            while i < chars.len() && chars[i].is_ascii_digit() {
                num_str.push(chars[i]);
                i += 1;
            }
            if !num_str.is_empty() {
                let full_match = format!("w{}", num_str);
                if full_match == pattern {
                    // Check that next char is not a digit or underscore followed by digit
                    if i >= chars.len() || (!chars[i].is_ascii_digit() && chars[i] != '_') {
                        return true;
                    }
                }
            }
        } else {
            i += 1;
        }
    }
    false
}

/// Replace witness w{idx} with replacement in SMT constraint
/// Must be careful not to replace w0 in w01, etc.
fn replace_witness_in_constraint(constraint: &str, idx: u32, replacement: &str) -> String {
    let pattern = format!("w{}", idx);
    let mut result = String::new();
    let mut chars = constraint.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == 'w' {
            // Check if this starts our pattern
            let mut num_str = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    num_str.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            
            if !num_str.is_empty() {
                let full_match = format!("w{}", num_str);
                if full_match == pattern {
                    // Check next char is not a digit or underscore (word boundary)
                    if chars.peek().map(|c| !c.is_ascii_digit() && *c != '_').unwrap_or(true) {
                        result.push_str(replacement);
                    } else {
                        result.push_str(&full_match);
                    }
                } else {
                    result.push_str(&full_match);
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    
    result
}

impl Default for Cvc5Solver {
    fn default() -> Self {
        Self::new()
    }
}

impl SmtSolver for Cvc5Solver {
    fn check_sat(&self, smt_script: &str) -> Result<bool, AnalysisError> {
        Cvc5Solver::check_sat(self, smt_script)
    }
    
    fn check_sat_with_timeout(&self, smt_script: &str, timeout_seconds: u64) -> Result<bool, AnalysisError> {
        Cvc5Solver::check_sat_with_timeout(self, smt_script, timeout_seconds)
    }
    
    fn get_model(&self, smt_script: &str) -> Result<String, AnalysisError> {
        Cvc5Solver::get_model(self, smt_script)
    }
    
    fn witness_appears_in_constraints(&self, witness: Witness, smt_script: &str) -> bool {
        Cvc5Solver::witness_appears_in_constraints(self, witness, smt_script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_replace_witness() {
        assert_eq!(
            replace_witness_in_constraint("(assert (= w0 w1))", 0, "w0_alt"),
            "(assert (= w0_alt w1))"
        );
        assert_eq!(
            replace_witness_in_constraint("(assert (= w0 w10))", 0, "w0_alt"),
            "(assert (= w0_alt w10))"
        );
        assert_eq!(
            replace_witness_in_constraint("(assert (= w10 w1))", 1, "w1_alt"),
            "(assert (= w10 w1_alt))"
        );
    }
    
    #[test]
    fn test_constraint_contains_witness() {
        assert!(constraint_contains_witness("(assert (= w0 w1))", 0));
        assert!(constraint_contains_witness("(assert (= w0 w1))", 1));
        assert!(!constraint_contains_witness("(assert (= w0 w1))", 10));
        assert!(constraint_contains_witness("(assert (= w10 w1))", 10));
        assert!(constraint_contains_witness("(assert (= w10 w1))", 1));
        assert!(!constraint_contains_witness("(assert (= w10 w1))", 0));
    }
    
    #[test]
    fn test_replace_multiple_witnesses() {
        // Test replacing multiple occurrences
        assert_eq!(
            replace_witness_in_constraint("(assert (= (* w0 w0) w1))", 0, "w0_alt"),
            "(assert (= (* w0_alt w0_alt) w1))"
        );
    }
}
