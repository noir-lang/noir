//! Witness Analysis Module
//!
//! This module provides analysis of witness dependencies and connectivity.
//! It builds a dependency graph from all circuit opcodes and can identify
//! witnesses that are not connected to public inputs/outputs.

use acvm::acir::circuit::{Circuit, Opcode};
use acvm::acir::circuit::opcodes::{BlackBoxFuncCall, FunctionInput, MemOp};
use acvm::acir::native_types::{Expression, Witness};
use acvm::FieldElement;
use std::collections::{BTreeSet, HashMap, HashSet};

/// Types of potential issues with witnesses
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WitnessIssue {
    /// Witness is not connected to any public input/output
    Disconnected,
    /// Witness is never used in any constraint
    Unused,
    /// Witness comes from Brillig (unconstrained) output
    BrilligOutput,
    /// Witness comes from hash function output (not cryptographically constrained)
    HashOutput(String), // Name of hash function
    /// Witness comes from cryptographic operation that can't be modeled
    CryptoOutput(String), // Name of operation
}

/// Detailed information about a witness issue
#[derive(Debug, Clone)]
pub struct WitnessIssueReport {
    pub witness: Witness,
    pub issue: WitnessIssue,
    pub opcode_index: Option<usize>,
    pub details: String,
}

/// Analyzer for witness dependencies
pub struct WitnessAnalyzer {
    /// Graph of witness dependencies (bidirectional)
    dependencies: HashMap<Witness, HashSet<Witness>>,
    /// Track witnesses from Brillig calls (unconstrained)
    brillig_outputs: HashSet<Witness>,
    /// Track witnesses from hash function outputs
    hash_outputs: HashMap<Witness, String>,
    /// Track witnesses from crypto operations
    crypto_outputs: HashMap<Witness, String>,
    /// Track which opcodes use each witness
    witness_to_opcodes: HashMap<Witness, Vec<usize>>,
}

impl WitnessAnalyzer {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            brillig_outputs: HashSet::new(),
            hash_outputs: HashMap::new(),
            crypto_outputs: HashMap::new(),
            witness_to_opcodes: HashMap::new(),
        }
    }

    /// Full analysis - returns all issues found
    pub fn analyze_circuit(
        &mut self,
        circuit: &Circuit<FieldElement>,
    ) -> Vec<WitnessIssueReport> {
        // Build dependency graph from all opcodes
        self.build_full_dependency_graph(circuit);

        let mut reports = Vec::new();

        // Find all public witnesses
        let public: BTreeSet<Witness> = circuit
            .public_parameters
            .0
            .iter()
            .chain(circuit.return_values.0.iter())
            .copied()
            .collect();

        // Find witnesses reachable from public inputs/outputs
        let reachable = self.reachable_from(&public);

        // Check all witnesses
        for i in 0..=circuit.current_witness_index {
            let witness = Witness::from(i);

            // Skip public witnesses
            if public.contains(&witness) {
                continue;
            }

            // Check for Brillig outputs
            if self.brillig_outputs.contains(&witness) {
                // Check if this Brillig output is constrained elsewhere
                if !self.is_witness_constrained_after_brillig(&witness) {
                    reports.push(WitnessIssueReport {
                        witness,
                        issue: WitnessIssue::BrilligOutput,
                        opcode_index: self.witness_to_opcodes.get(&witness).and_then(|v| v.first().copied()),
                        details: format!(
                            "Witness w{} is an output from Brillig (unconstrained function). \
                            Prover can set arbitrary value unless constrained by other opcodes!",
                            witness.witness_index()
                        ),
                    });
                }
            }

            // Check for hash outputs
            if let Some(hash_name) = self.hash_outputs.get(&witness) {
                if !self.is_witness_constrained_after_hash(&witness) {
                    reports.push(WitnessIssueReport {
                        witness,
                        issue: WitnessIssue::HashOutput(hash_name.clone()),
                        opcode_index: self.witness_to_opcodes.get(&witness).and_then(|v| v.first().copied()),
                        details: format!(
                            "Witness w{} is an output from {} hash function. \
                            Without additional constraints, prover can potentially manipulate this value!",
                            witness.witness_index(), hash_name
                        ),
                    });
                }
            }

            // Check for crypto outputs
            if let Some(op_name) = self.crypto_outputs.get(&witness) {
                reports.push(WitnessIssueReport {
                    witness,
                    issue: WitnessIssue::CryptoOutput(op_name.clone()),
                    opcode_index: self.witness_to_opcodes.get(&witness).and_then(|v| v.first().copied()),
                    details: format!(
                        "Witness w{} is an output from {} operation. \
                        This cannot be fully modeled in SMT - manual review recommended.",
                        witness.witness_index(), op_name
                    ),
                });
            }

            // Check if disconnected from public witnesses
            if !reachable.contains(&witness) && circuit.private_parameters.contains(&witness) {
                reports.push(WitnessIssueReport {
                    witness,
                    issue: WitnessIssue::Disconnected,
                    opcode_index: None,
                    details: format!(
                        "Private witness w{} is not reachable from public inputs/outputs",
                        witness.witness_index()
                    ),
                });
            }

            // Check if unused in any constraint
            if !self.dependencies.contains_key(&witness) 
               && !circuit.return_values.0.contains(&witness) 
               && !self.brillig_outputs.contains(&witness) {
                reports.push(WitnessIssueReport {
                    witness,
                    issue: WitnessIssue::Unused,
                    opcode_index: None,
                    details: format!(
                        "Witness w{} is not used in any constraint at all",
                        witness.witness_index()
                    ),
                });
            }
        }

        reports
    }

    /// Find witnesses that are not connected to public inputs/outputs
    pub fn find_disconnected_witnesses(
        &mut self,
        circuit: &Circuit<FieldElement>,
    ) -> Vec<Witness> {
        // Build dependency graph
        self.build_full_dependency_graph(circuit);

        // Find all public inputs and outputs
        let public: BTreeSet<Witness> = circuit
            .public_parameters
            .0
            .iter()
            .chain(circuit.return_values.0.iter())
            .copied()
            .collect();

        // Find all witnesses reachable from public inputs/outputs
        let reachable = self.reachable_from(&public);

        // Return private witnesses that are not reachable
        circuit
            .private_parameters
            .iter()
            .filter(|w| !reachable.contains(w))
            .copied()
            .collect()
    }

    /// Get all Brillig output witnesses
    pub fn get_brillig_outputs(&self) -> &HashSet<Witness> {
        &self.brillig_outputs
    }

    /// Get all hash output witnesses
    pub fn get_hash_outputs(&self) -> &HashMap<Witness, String> {
        &self.hash_outputs
    }

    /// Build dependency graph from ALL circuit opcodes
    fn build_full_dependency_graph(&mut self, circuit: &Circuit<FieldElement>) {
        self.dependencies.clear();
        self.brillig_outputs.clear();
        self.hash_outputs.clear();
        self.crypto_outputs.clear();
        self.witness_to_opcodes.clear();

        for (idx, opcode) in circuit.opcodes.iter().enumerate() {
            match opcode {
                Opcode::AssertZero(expr) => {
                    let witnesses = self.collect_expression_witnesses(expr);
                    self.add_mutual_dependencies(&witnesses);
                    self.record_opcode_for_witnesses(&witnesses, idx);
                }
                
                Opcode::BlackBoxFuncCall(bb_call) => {
                    self.process_black_box_call(bb_call, idx);
                }
                
                Opcode::MemoryOp { block_id: _, op } => {
                    self.process_memory_op(op, idx);
                }
                
                Opcode::MemoryInit { block_id: _, init, .. } => {
                    // Memory init creates dependencies between all init witnesses
                    let witnesses: Vec<Witness> = init.iter().copied().collect();
                    self.add_mutual_dependencies(&witnesses);
                    self.record_opcode_for_witnesses(&witnesses, idx);
                }
                
                Opcode::BrilligCall { outputs, .. } => {
                    // Mark all Brillig outputs as potentially unconstrained
                    // We can't easily inspect the structure here, so we use a simplified approach
                    // The actual constraint check happens through SMT solver
                    self.process_brillig_outputs(outputs, idx);
                }
                
                Opcode::Call { outputs, inputs, .. } => {
                    // Circuit call - outputs depend on inputs
                    let input_witnesses: Vec<Witness> = inputs.iter().copied().collect();
                    let output_witnesses: Vec<Witness> = outputs.iter().copied().collect();
                    
                    // Outputs depend on inputs
                    for output in &output_witnesses {
                        for input in &input_witnesses {
                            self.add_dependency(*output, *input);
                        }
                    }
                    
                    self.record_opcode_for_witnesses(&input_witnesses, idx);
                    self.record_opcode_for_witnesses(&output_witnesses, idx);
                }
            }
        }
    }

    /// Process Brillig outputs - simplified version that works with the opaque structure
    fn process_brillig_outputs(&mut self, outputs: &[acvm::acir::circuit::brillig::BrilligOutputs], opcode_idx: usize) {
        use acvm::acir::circuit::brillig::BrilligOutputs;
        
        for output in outputs {
            match output {
                BrilligOutputs::Simple(w) => {
                    self.brillig_outputs.insert(*w);
                    self.record_opcode_for_witness(*w, opcode_idx);
                }
                BrilligOutputs::Array(witnesses) => {
                    for w in witnesses.iter() {
                        self.brillig_outputs.insert(*w);
                        self.record_opcode_for_witness(*w, opcode_idx);
                    }
                }
            }
        }
    }

    /// Process a BlackBoxFuncCall opcode
    fn process_black_box_call(&mut self, bb_call: &BlackBoxFuncCall<FieldElement>, opcode_idx: usize) {
        match bb_call {
            BlackBoxFuncCall::RANGE { input, .. } => {
                if let FunctionInput::Witness(w) = input {
                    self.add_self_dependency(*w);
                    self.record_opcode_for_witness(*w, opcode_idx);
                }
            }
            
            BlackBoxFuncCall::AND { lhs, rhs, output, .. } |
            BlackBoxFuncCall::XOR { lhs, rhs, output, .. } => {
                let mut inputs = Vec::new();
                if let FunctionInput::Witness(w) = lhs { inputs.push(*w); }
                if let FunctionInput::Witness(w) = rhs { inputs.push(*w); }
                
                for input in &inputs {
                    self.add_dependency(*output, *input);
                }
                inputs.push(*output);
                self.record_opcode_for_witnesses(&inputs, opcode_idx);
            }
            
            // Hash functions - outputs are not cryptographically constrained
            BlackBoxFuncCall::Blake2s { inputs, outputs } => {
                let input_witnesses: Vec<Witness> = inputs.iter()
                    .filter_map(|i| if let FunctionInput::Witness(w) = i { Some(*w) } else { None })
                    .collect();
                self.process_hash_function_with_inputs("Blake2s", &input_witnesses, outputs.as_ref(), opcode_idx);
            }
            BlackBoxFuncCall::Blake3 { inputs, outputs } => {
                let input_witnesses: Vec<Witness> = inputs.iter()
                    .filter_map(|i| if let FunctionInput::Witness(w) = i { Some(*w) } else { None })
                    .collect();
                self.process_hash_function_with_inputs("Blake3", &input_witnesses, outputs.as_ref(), opcode_idx);
            }
            BlackBoxFuncCall::Sha256Compression { inputs, hash_values, outputs } => {
                let all_inputs: Vec<Witness> = inputs.iter()
                    .chain(hash_values.iter())
                    .filter_map(|i| if let FunctionInput::Witness(w) = i { Some(*w) } else { None })
                    .collect();
                self.process_hash_function_with_inputs("SHA256", &all_inputs, outputs.as_ref(), opcode_idx);
            }
            BlackBoxFuncCall::Keccakf1600 { inputs, outputs } => {
                let input_witnesses: Vec<Witness> = inputs.iter()
                    .filter_map(|i| if let FunctionInput::Witness(w) = i { Some(*w) } else { None })
                    .collect();
                self.process_hash_function_with_inputs("Keccak-f1600", &input_witnesses, outputs.as_ref(), opcode_idx);
            }
            
            // Poseidon is a ZK-friendly hash, but still needs care
            BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs, .. } => {
                let input_witnesses: Vec<Witness> = inputs.iter()
                    .filter_map(|i| if let FunctionInput::Witness(w) = i { Some(*w) } else { None })
                    .collect();
                self.process_hash_function_with_inputs("Poseidon2", &input_witnesses, outputs.as_ref(), opcode_idx);
            }
            
            // ECDSA and other crypto operations
            BlackBoxFuncCall::EcdsaSecp256k1 { output, .. } => {
                self.crypto_outputs.insert(*output, "ECDSA-secp256k1".to_string());
                self.record_opcode_for_witness(*output, opcode_idx);
            }
            
            BlackBoxFuncCall::EcdsaSecp256r1 { output, .. } => {
                self.crypto_outputs.insert(*output, "ECDSA-secp256r1".to_string());
                self.record_opcode_for_witness(*output, opcode_idx);
            }
            
            // Elliptic curve operations
            BlackBoxFuncCall::MultiScalarMul { outputs, .. } => {
                let (x, y, inf) = outputs;
                self.crypto_outputs.insert(*x, "MSM".to_string());
                self.crypto_outputs.insert(*y, "MSM".to_string());
                self.crypto_outputs.insert(*inf, "MSM".to_string());
                self.record_opcode_for_witness(*x, opcode_idx);
                self.record_opcode_for_witness(*y, opcode_idx);
                self.record_opcode_for_witness(*inf, opcode_idx);
            }
            
            BlackBoxFuncCall::EmbeddedCurveAdd { outputs, .. } => {
                let (x, y, inf) = outputs;
                self.crypto_outputs.insert(*x, "CurveAdd".to_string());
                self.crypto_outputs.insert(*y, "CurveAdd".to_string());
                self.crypto_outputs.insert(*inf, "CurveAdd".to_string());
                self.record_opcode_for_witness(*x, opcode_idx);
                self.record_opcode_for_witness(*y, opcode_idx);
                self.record_opcode_for_witness(*inf, opcode_idx);
            }
            
            BlackBoxFuncCall::RecursiveAggregation { .. } => {
                // Recursive aggregation is complex - treated as opaque
            }
            
            BlackBoxFuncCall::AES128Encrypt { outputs, .. } => {
                for output in outputs.iter() {
                    self.crypto_outputs.insert(*output, "AES128".to_string());
                    self.record_opcode_for_witness(*output, opcode_idx);
                }
            }
        }
    }

    fn process_hash_function_with_inputs(
        &mut self,
        name: &str,
        input_witnesses: &[Witness],
        outputs: &[Witness],
        opcode_idx: usize
    ) {
        // Mark outputs as hash outputs
        for output in outputs {
            self.hash_outputs.insert(*output, name.to_string());
            // Add dependency from output to inputs
            for input in input_witnesses {
                self.add_dependency(*output, *input);
            }
        }
        
        let mut all_witnesses: Vec<Witness> = input_witnesses.to_vec();
        all_witnesses.extend(outputs.iter().copied());
        self.record_opcode_for_witnesses(&all_witnesses, opcode_idx);
    }

    /// Process memory operation
    fn process_memory_op(&mut self, op: &MemOp<FieldElement>, opcode_idx: usize) {
        let mut witnesses = Vec::new();
        
        // Collect witnesses from index expression
        witnesses.extend(self.collect_expression_witnesses(&op.index));
        
        // Collect witnesses from value expression
        witnesses.extend(self.collect_expression_witnesses(&op.value));
        
        // Collect witnesses from operation expression
        witnesses.extend(self.collect_expression_witnesses(&op.operation));
        
        self.add_mutual_dependencies(&witnesses);
        self.record_opcode_for_witnesses(&witnesses, opcode_idx);
    }

    /// Collect all witnesses from an expression
    fn collect_expression_witnesses(&self, expr: &Expression<FieldElement>) -> Vec<Witness> {
        let mut witnesses = Vec::new();

        // Collect from multiplication terms
        for (_, w1, w2) in &expr.mul_terms {
            witnesses.push(*w1);
            witnesses.push(*w2);
        }

        // Collect from linear terms
        for (_, witness) in &expr.linear_combinations {
            witnesses.push(*witness);
        }

        witnesses
    }

    /// Add mutual dependencies between all witnesses
    fn add_mutual_dependencies(&mut self, witnesses: &[Witness]) {
        for i in 0..witnesses.len() {
            for j in (i + 1)..witnesses.len() {
                let w1 = witnesses[i];
                let w2 = witnesses[j];
                self.add_dependency(w1, w2);
            }
        }
    }

    /// Add bidirectional dependency between two witnesses
    fn add_dependency(&mut self, w1: Witness, w2: Witness) {
        self.dependencies.entry(w1).or_default().insert(w2);
        self.dependencies.entry(w2).or_default().insert(w1);
    }

    /// Add self-dependency (marks witness as used)
    fn add_self_dependency(&mut self, w: Witness) {
        self.dependencies.entry(w).or_default();
    }

    /// Record which opcode uses a witness
    fn record_opcode_for_witness(&mut self, witness: Witness, opcode_idx: usize) {
        self.witness_to_opcodes.entry(witness).or_default().push(opcode_idx);
    }

    fn record_opcode_for_witnesses(&mut self, witnesses: &[Witness], opcode_idx: usize) {
        for w in witnesses {
            self.record_opcode_for_witness(*w, opcode_idx);
        }
    }

    /// Check if a Brillig output is constrained by subsequent opcodes
    fn is_witness_constrained_after_brillig(&self, witness: &Witness) -> bool {
        // Check if this witness appears in any constraint (not just dependency)
        // A witness is properly constrained if it appears in at least one non-Brillig opcode
        if let Some(opcodes) = self.witness_to_opcodes.get(witness) {
            // If it appears in more than one opcode, it's likely constrained
            opcodes.len() > 1
        } else {
            false
        }
    }

    /// Check if a hash output is constrained
    fn is_witness_constrained_after_hash(&self, witness: &Witness) -> bool {
        // Same logic as Brillig
        if let Some(opcodes) = self.witness_to_opcodes.get(witness) {
            opcodes.len() > 1
        } else {
            false
        }
    }

    /// Find all witnesses reachable from a set of starting witnesses
    fn reachable_from(&self, start: &BTreeSet<Witness>) -> BTreeSet<Witness> {
        let mut visited = BTreeSet::new();
        let mut queue: Vec<Witness> = start.iter().copied().collect();

        while let Some(current) = queue.pop() {
            if visited.insert(current) {
                // Add all witnesses that depend on current
                if let Some(deps) = self.dependencies.get(&current) {
                    queue.extend(deps.iter());
                }
            }
        }

        visited
    }

    /// Find all input witnesses that don't appear in any constraint
    pub fn find_unused_input_witnesses(
        &mut self,
        circuit: &Circuit<FieldElement>,
    ) -> Vec<Witness> {
        self.build_full_dependency_graph(circuit);

        let used_witnesses: HashSet<Witness> = self.dependencies.keys().copied().collect();

        let all_inputs: BTreeSet<Witness> = circuit
            .public_parameters
            .0
            .iter()
            .chain(circuit.private_parameters.iter())
            .copied()
            .collect();

        all_inputs
            .iter()
            .filter(|w| !used_witnesses.contains(w))
            .copied()
            .collect()
    }

    /// Find all witnesses that don't appear in any constraint
    pub fn find_all_unused_witnesses(
        &mut self,
        circuit: &Circuit<FieldElement>,
    ) -> Vec<Witness> {
        self.build_full_dependency_graph(circuit);

        let used_witnesses: HashSet<Witness> = self.dependencies.keys().copied().collect();

        let mut unused = Vec::new();
        for i in 0..=circuit.current_witness_index {
            let witness = Witness::from(i);
            
            if circuit.return_values.0.contains(&witness) {
                continue;
            }
            
            if !used_witnesses.contains(&witness) {
                unused.push(witness);
            }
        }
        unused
    }
}

impl Default for WitnessAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acvm::acir::native_types::Expression;
    use acvm::acir::circuit::PublicInputs;
    use acvm::AcirField;

    /// Create a simple test circuit with given opcodes
    fn make_circuit(
        current_witness_index: u32,
        opcodes: Vec<Opcode<FieldElement>>,
        public_params: BTreeSet<Witness>,
        return_vals: BTreeSet<Witness>,
    ) -> Circuit<FieldElement> {
        Circuit {
            function_name: "test".to_string(),
            current_witness_index,
            opcodes,
            private_parameters: BTreeSet::new(),
            public_parameters: PublicInputs(public_params),
            return_values: PublicInputs(return_vals),
            assert_messages: vec![],
        }
    }

    #[test]
    fn test_empty_circuit() {
        let circuit = make_circuit(0, vec![], BTreeSet::new(), BTreeSet::new());
        let mut analyzer = WitnessAnalyzer::new();
        let reports = analyzer.analyze_circuit(&circuit);
        
        // Empty circuit should have no issues
        assert!(reports.is_empty());
    }

    #[test]
    fn test_simple_connected_circuit() {
        // w0 (public) + w1 = w2 (return)
        let mut public_params = BTreeSet::new();
        public_params.insert(Witness(0));
        let mut return_vals = BTreeSet::new();
        return_vals.insert(Witness(2));
        
        let circuit = make_circuit(
            2,
            vec![Opcode::AssertZero(Expression {
                mul_terms: vec![],
                linear_combinations: vec![
                    (FieldElement::one(), Witness(0)),
                    (FieldElement::one(), Witness(1)),
                    (-FieldElement::one(), Witness(2)),
                ],
                q_c: FieldElement::zero(),
            })],
            public_params,
            return_vals,
        );

        let mut analyzer = WitnessAnalyzer::new();
        let reports = analyzer.analyze_circuit(&circuit);
        
        // All witnesses are connected, so no issues
        assert!(reports.is_empty(), "Expected no issues, got: {:?}", reports);
    }

    #[test]
    fn test_disconnected_witness_detected() {
        // w0 is public, w1 connected to w0, w2 is disconnected
        let mut public_params = BTreeSet::new();
        public_params.insert(Witness(0));
        let mut return_vals = BTreeSet::new();
        return_vals.insert(Witness(1));
        
        let circuit = make_circuit(
            2,
            vec![
                // w0 + w1 = 0 (connected to public)
                Opcode::AssertZero(Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![
                        (FieldElement::one(), Witness(0)),
                        (-FieldElement::one(), Witness(1)),
                    ],
                    q_c: FieldElement::zero(),
                }),
                // w2 = 42 (disconnected - not reachable from public)
                Opcode::AssertZero(Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), Witness(2))],
                    q_c: -FieldElement::from(42u32),
                }),
            ],
            public_params,
            return_vals,
        );

        let mut analyzer = WitnessAnalyzer::new();
        let reports = analyzer.analyze_circuit(&circuit);
        
        // w2 should be reported as disconnected
        let w2_issues: Vec<_> = reports.iter().filter(|r| r.witness == Witness(2)).collect();
        assert!(!w2_issues.is_empty(), "Expected w2 to be reported as disconnected");
    }

    #[test]
    fn test_range_constraint_processed() {
        let mut public_params = BTreeSet::new();
        public_params.insert(Witness(0));
        
        let circuit = make_circuit(
            0,
            vec![Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::Witness(Witness(0)),
                num_bits: 8,
            })],
            public_params,
            BTreeSet::new(),
        );

        let mut analyzer = WitnessAnalyzer::new();
        let reports = analyzer.analyze_circuit(&circuit);
        
        // Public witness with range constraint should not be flagged
        assert!(reports.is_empty());
    }
}
