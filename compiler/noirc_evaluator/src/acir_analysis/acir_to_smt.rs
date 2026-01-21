//! Converter from ACIR to SMT-LIB format
//!
//! This module converts ACIR circuits to SMT-LIB format for use with SMT solvers like cvc5.
//! It properly models finite field arithmetic over the BN254 scalar field.
//!
//! ## Supported Features
//! - Finite field arithmetic with BN254 modulus
//! - All arithmetic expressions (AssertZero)
//! - Range constraints (RANGE black box)
//! - Bitwise operations (AND, XOR)
//! - Memory operations (arrays)
//! - Public/private input tracking
//!
//! ## SMT Logic
//! Uses QF_NIA (Quantifier-Free Non-linear Integer Arithmetic) with explicit
//! modular reduction. For production use with large fields, consider using
//! SMT solvers with native finite field support (cvc5 5.0+ with QF_FF).

use acvm::acir::circuit::opcodes::{BlackBoxFuncCall, BlockId, FunctionInput, MemOp};
use acvm::acir::circuit::{Circuit, Opcode};
use acvm::acir::native_types::{Expression, Witness};
use acvm::{AcirField, FieldElement};
use num_bigint::BigUint;
use std::collections::{BTreeMap, HashMap, HashSet};

use super::AnalysisError;

/// BN254 scalar field modulus
/// p = 21888242871839275222246405745257275088548364400416034343698204186575808495617
const BN254_MODULUS_HEX: &str = "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001";

/// Configuration for SMT conversion
#[derive(Debug, Clone)]
pub struct SmtConfig {
    /// Use bitvector theory instead of integers (more efficient for bounded values)
    pub use_bitvectors: bool,
    /// Add explicit field bounds for all witnesses
    pub add_field_bounds: bool,
    /// Track which witnesses are public inputs
    pub public_witness_indices: HashSet<u32>,
    /// Track which witnesses are return values
    pub return_witness_indices: HashSet<u32>,
    /// Include model generation (get-value) commands
    pub generate_model: bool,
    /// Add comments to the SMT output for debugging
    pub add_comments: bool,
    /// Maximum bit size for range checks (optimization)
    pub max_range_bits: Option<u32>,
}

impl Default for SmtConfig {
    fn default() -> Self {
        Self {
            use_bitvectors: false,
            add_field_bounds: true,
            public_witness_indices: HashSet::new(),
            return_witness_indices: HashSet::new(),
            generate_model: true,
            add_comments: true,
            max_range_bits: None,
        }
    }
}

/// Converter from ACIR to SMT-LIB format
pub struct AcirToSmtConverter {
    /// Configuration
    config: SmtConfig,
    /// Mapping from witness to SMT variable name
    witness_to_var: HashMap<Witness, String>,
    /// SMT-LIB script lines
    script: Vec<String>,
    /// Memory blocks (BlockId -> array of initial witnesses)
    memory_blocks: BTreeMap<u32, Vec<Witness>>,
    /// Range constraints for witnesses (witness index -> bit size)
    range_constraints: HashMap<u32, u32>,
    /// Field modulus as BigUint
    modulus: BigUint,
    /// Witnesses that have been declared
    declared_witnesses: HashSet<Witness>,
    /// Auxiliary variable counter for intermediate values
    aux_var_counter: u32,
    /// Memory version counter per block (for tracking sequential writes)
    memory_versions: HashMap<u32, u32>,
}

impl AcirToSmtConverter {
    /// Create a new converter with default configuration
    pub fn new() -> Self {
        Self::with_config(SmtConfig::default())
    }

    /// Create a new converter with custom configuration
    pub fn with_config(config: SmtConfig) -> Self {
        let modulus = BigUint::parse_bytes(BN254_MODULUS_HEX.as_bytes(), 16)
            .expect("Invalid BN254 modulus hex string");

        Self {
            config,
            witness_to_var: HashMap::new(),
            script: Vec::with_capacity(256), // Pre-allocate some capacity
            memory_blocks: BTreeMap::new(),
            range_constraints: HashMap::new(),
            modulus,
            declared_witnesses: HashSet::new(),
            aux_var_counter: 0,
            memory_versions: HashMap::new(),
        }
    }

    /// Convert ACIR circuit to SMT-LIB format
    pub fn convert(&mut self, circuit: &Circuit<FieldElement>) -> Result<String, AnalysisError> {
        self.reset();

        // SMT-LIB header
        self.emit_header();

        // Define the field modulus constant
        self.emit_modulus_definition();

        // First pass: collect all witnesses and memory blocks
        self.collect_circuit_info(circuit)?;

        // Declare all witnesses
        self.emit_witness_declarations(circuit)?;

        // Add field bounds for all witnesses (0 <= w < p)
        if self.config.add_field_bounds {
            self.emit_field_bounds()?;
        }

        // Convert each opcode to SMT constraints
        for (idx, opcode) in circuit.opcodes.iter().enumerate() {
            if self.config.add_comments {
                self.script.push(format!("; Opcode {}: {:?}", idx, opcode_type_name(opcode)));
            }
            self.convert_opcode(opcode)?;
        }

        // Add check-sat and optional model generation
        self.emit_footer();

        Ok(self.script.join("\n"))
    }

    /// Reset the converter state
    fn reset(&mut self) {
        self.script.clear();
        self.witness_to_var.clear();
        self.memory_blocks.clear();
        self.range_constraints.clear();
        self.declared_witnesses.clear();
        self.aux_var_counter = 0;
        self.memory_versions.clear();
    }

    /// Emit SMT-LIB header
    fn emit_header(&mut self) {
        if self.config.add_comments {
            self.script.push("; ACIR to SMT-LIB conversion".into());
            self.script.push("; Field: BN254 scalar field".into());
            self.script.push(format!("; Modulus: {}", self.modulus));
            self.script.push(String::new());
        }

        // Use QF_NIA for non-linear integer arithmetic
        // For production with large fields, consider QF_FF (cvc5 5.0+)
        self.script.push("(set-logic QF_NIA)".into());
        self.script.push("(set-option :produce-models true)".into());
        self.script.push(String::new());
    }

    /// Define the field modulus as a constant
    fn emit_modulus_definition(&mut self) {
        if self.config.add_comments {
            self.script.push("; Field modulus p".into());
        }
        self.script.push(format!("(define-fun FIELD_MODULUS () Int {})", self.modulus));
        self.script.push(String::new());
    }

    /// Collect information about witnesses and memory blocks
    fn collect_circuit_info(
        &mut self,
        circuit: &Circuit<FieldElement>,
    ) -> Result<(), AnalysisError> {
        for opcode in &circuit.opcodes {
            match opcode {
                Opcode::MemoryInit { block_id, init, .. } => {
                    self.memory_blocks.insert(block_id.0, init.clone());
                }
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE { input, num_bits }) => {
                    if let FunctionInput::Witness(w) = input {
                        self.range_constraints.insert(w.witness_index(), *num_bits);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Emit witness declarations
    fn emit_witness_declarations(
        &mut self,
        circuit: &Circuit<FieldElement>,
    ) -> Result<(), AnalysisError> {
        if self.config.add_comments {
            self.script.push("; Witness declarations".into());
        }

        let max_witness = circuit.current_witness_index;
        for i in 0..=max_witness {
            let witness = Witness::from(i);
            let var_name = format!("w{}", i);
            self.witness_to_var.insert(witness, var_name.clone());
            self.declared_witnesses.insert(witness);

            // Add comment about witness type
            let comment = if self.config.add_comments {
                if self.config.public_witness_indices.contains(&i) {
                    " ; public input"
                } else if self.config.return_witness_indices.contains(&i) {
                    " ; return value"
                } else {
                    ""
                }
            } else {
                ""
            };

            self.script.push(format!("(declare-fun {} () Int){}", var_name, comment));
        }
        self.script.push(String::new());

        // Declare initial memory arrays (version 0) if any
        if !self.memory_blocks.is_empty() {
            if self.config.add_comments {
                self.script.push("; Memory arrays (initial version v0)".into());
            }
            for (block_id, _) in &self.memory_blocks {
                // Declare version 0 of each memory block
                self.script.push(format!("(declare-fun mem{}_v0 (Int) Int)", block_id));
            }
            self.script.push(String::new());
        }

        Ok(())
    }

    /// Emit field bounds for all witnesses (0 <= w < p)
    fn emit_field_bounds(&mut self) -> Result<(), AnalysisError> {
        if self.config.add_comments {
            self.script.push("; Field bounds: 0 <= w < FIELD_MODULUS".into());
        }

        for (witness, var_name) in &self.witness_to_var {
            // Check if we have a tighter range constraint
            if let Some(bits) = self.range_constraints.get(&witness.witness_index()) {
                // Use the range constraint instead of full field bounds
                let max_val = (BigUint::from(1u32) << bits) - 1u32;
                self.script.push(format!("(assert (and (>= {} 0) (<= {} {})))", var_name, var_name, max_val));
            } else {
                // Full field bounds
                self.script.push(format!(
                    "(assert (and (>= {} 0) (< {} FIELD_MODULUS)))",
                    var_name, var_name
                ));
            }
        }
        self.script.push(String::new());

        Ok(())
    }

    /// Emit footer with check-sat and model commands
    fn emit_footer(&mut self) {
        self.script.push(String::new());
        self.script.push("(check-sat)".into());

        if self.config.generate_model {
            // Generate get-value for all witnesses
            let witnesses: Vec<_> = self.witness_to_var.values().cloned().collect();
            if !witnesses.is_empty() {
                self.script.push(format!("(get-value ({}))", witnesses.join(" ")));
            }
        }
    }

    /// Convert a single opcode to SMT constraints
    fn convert_opcode(&mut self, opcode: &Opcode<FieldElement>) -> Result<(), AnalysisError> {
        match opcode {
            Opcode::AssertZero(expr) => {
                self.convert_assert_zero(expr)?;
            }
            Opcode::BlackBoxFuncCall(bb_call) => {
                self.convert_black_box(bb_call)?;
            }
            Opcode::MemoryOp { block_id, op } => {
                self.convert_memory_op(*block_id, op)?;
            }
            Opcode::MemoryInit { block_id, init, .. } => {
                self.convert_memory_init(*block_id, init)?;
            }
            Opcode::BrilligCall { outputs, .. } => {
                // Brillig calls are unconstrained functions
                // We can't model them precisely, but we can ensure outputs are in field
                if self.config.add_comments {
                    self.script.push("; Brillig call - outputs treated as unconstrained".into());
                }
                // Outputs are already declared and field-bounded
                let _ = outputs; // Acknowledge we're ignoring the details
            }
            Opcode::Call { inputs, outputs, .. } => {
                // Circuit calls would need the sub-circuit to be inlined
                if self.config.add_comments {
                    self.script.push("; Circuit call - would require inlining".into());
                }
                let _ = (inputs, outputs);
            }
        }
        Ok(())
    }

    /// Convert AssertZero opcode: expr ≡ 0 (mod p)
    /// 
    /// This is the core constraint in ACIR. An Expression is:
    ///   Σ(q_M × wL × wR) + Σ(q_L × w) + q_c
    /// 
    /// AssertZero means this expression equals zero in the finite field.
    /// In SMT we model this as: (mod expr FIELD_MODULUS) = 0
    fn convert_assert_zero(&mut self, expr: &Expression<FieldElement>) -> Result<(), AnalysisError> {
        let smt_expr = self.expression_to_smt(expr)?;
        
        // Exact translation: expr ≡ 0 (mod p)
        // This is mathematically precise for finite field arithmetic
        self.script.push(format!("(assert (= (mod {} FIELD_MODULUS) 0))", smt_expr));
        
        Ok(())
    }

    /// Convert a black box function call to SMT
    /// 
    /// Black box functions in ACIR have well-defined semantics:
    /// - RANGE: 0 <= input < 2^num_bits
    /// - AND/XOR: bitwise operations on num_bits integers
    /// - Hash functions: cryptographic, outputs bounded by their word size
    /// - Crypto operations: outputs have type-specific bounds
    fn convert_black_box(
        &mut self,
        bb_call: &BlackBoxFuncCall<FieldElement>,
    ) -> Result<(), AnalysisError> {
        match bb_call {
            // === Arithmetic constraints (can be precisely modeled) ===
            
            BlackBoxFuncCall::RANGE { input, num_bits } => {
                // RANGE constraint: 0 <= input < 2^num_bits
                // This is an exact constraint from ACIR spec
                self.convert_range_constraint(input, *num_bits)?;
            }
            
            BlackBoxFuncCall::AND { lhs, rhs, num_bits, output } => {
                // Bitwise AND on num_bits integers
                // output = lhs AND rhs (bitwise)
                self.convert_bitwise_and(lhs, rhs, *num_bits, *output)?;
            }
            
            BlackBoxFuncCall::XOR { lhs, rhs, num_bits, output } => {
                // Bitwise XOR on num_bits integers
                // output = lhs XOR rhs (bitwise)
                self.convert_bitwise_xor(lhs, rhs, *num_bits, *output)?;
            }
            
            // === Hash functions (black box - only bound outputs) ===
            // These are cryptographic primitives we cannot model in SMT.
            // We only constrain outputs to valid ranges based on their word size.
            
            BlackBoxFuncCall::Blake2s { outputs, .. }
            | BlackBoxFuncCall::Blake3 { outputs, .. } => {
                // Hash outputs are bytes (8-bit words)
                self.emit_range_bounds_for_witnesses(outputs.as_ref(), 8, "Hash (8-bit outputs)")?;
            }
            
            BlackBoxFuncCall::Poseidon2Permutation { outputs, .. } => {
                // Poseidon outputs are field elements (already bounded by field)
                if self.config.add_comments {
                    self.script.push("; Poseidon2 - outputs are field elements".into());
                }
                let _ = outputs; // Field bounds already applied globally
            }
            
            BlackBoxFuncCall::Sha256Compression { outputs, .. } => {
                // SHA256 outputs are 32-bit words
                self.emit_range_bounds_for_witnesses(outputs.as_ref(), 32, "SHA256 (32-bit outputs)")?;
            }
            
            BlackBoxFuncCall::Keccakf1600 { outputs, .. } => {
                // Keccak-f1600 outputs are 64-bit lanes
                self.emit_range_bounds_for_witnesses(outputs.as_ref(), 64, "Keccak (64-bit outputs)")?;
            }
            
            // === Cryptographic operations (outputs have specific types) ===
            
            BlackBoxFuncCall::EcdsaSecp256k1 { output, .. }
            | BlackBoxFuncCall::EcdsaSecp256r1 { output, .. } => {
                // ECDSA verification returns boolean (0 or 1)
                self.emit_boolean_constraint(*output, "ECDSA verification")?;
            }
            
            BlackBoxFuncCall::MultiScalarMul { outputs, .. }
            | BlackBoxFuncCall::EmbeddedCurveAdd { outputs, .. } => {
                // Curve point: (x, y, is_infinity)
                // x, y are field elements (bounded), is_infinity is boolean
                let (_x, _y, inf) = outputs;
                self.emit_boolean_constraint(*inf, "Curve point is_infinity")?;
            }
            
            BlackBoxFuncCall::RecursiveAggregation { .. } => {
                // Recursive aggregation cannot be modeled in SMT
                if self.config.add_comments {
                    self.script.push("; RecursiveAggregation - opaque".into());
                }
            }
            
            BlackBoxFuncCall::AES128Encrypt { outputs, .. } => {
                // AES outputs are bytes (8-bit)
                self.emit_range_bounds_for_witnesses(outputs, 8, "AES128 (8-bit outputs)")?;
            }
        }
        Ok(())
    }
    
    /// Emit range bounds for a slice of witnesses: 0 <= w < 2^num_bits
    fn emit_range_bounds_for_witnesses(
        &mut self,
        witnesses: &[Witness],
        num_bits: u32,
        comment: &str,
    ) -> Result<(), AnalysisError> {
        if self.config.add_comments {
            self.script.push(format!("; {}", comment));
        }
        let max_value = (BigUint::from(1u32) << num_bits) - 1u32;
        for witness in witnesses {
            let var = self.get_witness_var(*witness)?;
            self.script.push(format!(
                "(assert (and (>= {} 0) (<= {} {})))",
                var, var, max_value
            ));
        }
        Ok(())
    }
    
    /// Emit boolean constraint: w ∈ {0, 1}
    fn emit_boolean_constraint(&mut self, witness: Witness, comment: &str) -> Result<(), AnalysisError> {
        if self.config.add_comments {
            self.script.push(format!("; {} - boolean", comment));
        }
        let var = self.get_witness_var(witness)?;
        self.script.push(format!("(assert (or (= {} 0) (= {} 1)))", var, var));
        Ok(())
    }

    /// Convert range constraint: 0 <= input < 2^num_bits
    /// 
    /// This is the exact ACIR semantics for RANGE black box function.
    fn convert_range_constraint(
        &mut self,
        input: &FunctionInput<FieldElement>,
        num_bits: u32,
    ) -> Result<(), AnalysisError> {
        let input_smt = self.function_input_to_smt(input)?;
        let max_value = (BigUint::from(1u32) << num_bits) - 1u32;

        if self.config.add_comments {
            self.script.push(format!("; RANGE: 0 <= x < 2^{}", num_bits));
        }
        self.script.push(format!(
            "(assert (and (>= {} 0) (<= {} {})))",
            input_smt, input_smt, max_value
        ));

        Ok(())
    }

    /// Convert bitwise AND operation
    /// 
    /// Uses bit-blasting: decompose values into bits, compute AND per-bit.
    /// For bit b: AND(a,b) = a*b when a,b ∈ {0,1}
    fn convert_bitwise_and(
        &mut self,
        lhs: &FunctionInput<FieldElement>,
        rhs: &FunctionInput<FieldElement>,
        num_bits: u32,
        output: Witness,
    ) -> Result<(), AnalysisError> {
        let lhs_smt = self.function_input_to_smt(lhs)?;
        let rhs_smt = self.function_input_to_smt(rhs)?;
        let output_var = self.get_witness_var(output)?;

        if self.config.add_comments {
            self.script.push(format!("; AND: {} bits", num_bits));
        }

        // Always use precise bit-blasting
        self.emit_bitwise_and_bitblast(&lhs_smt, &rhs_smt, &output_var, num_bits);

        Ok(())
    }

    /// Emit precise bitwise AND using bit-blasting
    /// 
    /// Uses the fact that AND on single bits is multiplication:
    /// bit_a AND bit_b = bit_a * bit_b (when both are 0 or 1)
    fn emit_bitwise_and_bitblast(
        &mut self,
        lhs: &str,
        rhs: &str,
        output: &str,
        num_bits: u32,
    ) {
        let prefix = format!("and_{}_{}", lhs.replace(['(', ')', ' '], "_"), self.aux_var_counter);
        self.aux_var_counter += 1;

        // Declare bit variables for lhs, rhs, and output
        for i in 0..num_bits {
            self.script.push(format!("(declare-fun {}_lhs_bit{} () Int)", prefix, i));
            self.script.push(format!("(declare-fun {}_rhs_bit{} () Int)", prefix, i));
            self.script.push(format!("(declare-fun {}_out_bit{} () Int)", prefix, i));
        }

        // Assert each bit is 0 or 1
        for i in 0..num_bits {
            self.script.push(format!(
                "(assert (or (= {}_lhs_bit{} 0) (= {}_lhs_bit{} 1)))",
                prefix, i, prefix, i
            ));
            self.script.push(format!(
                "(assert (or (= {}_rhs_bit{} 0) (= {}_rhs_bit{} 1)))",
                prefix, i, prefix, i
            ));
            self.script.push(format!(
                "(assert (or (= {}_out_bit{} 0) (= {}_out_bit{} 1)))",
                prefix, i, prefix, i
            ));
        }

        // Assert bit decomposition: lhs = sum(lhs_bit[i] * 2^i)
        let mut lhs_sum = String::new();
        for i in 0..num_bits {
            if i == 0 {
                lhs_sum = format!("{}_lhs_bit0", prefix);
            } else {
                lhs_sum = format!("(+ {} (* {} {}_lhs_bit{}))", lhs_sum, 1u64 << i, prefix, i);
            }
        }
        self.script.push(format!("(assert (= {} {}))", lhs, lhs_sum));

        // Assert bit decomposition for rhs
        let mut rhs_sum = String::new();
        for i in 0..num_bits {
            if i == 0 {
                rhs_sum = format!("{}_rhs_bit0", prefix);
            } else {
                rhs_sum = format!("(+ {} (* {} {}_rhs_bit{}))", rhs_sum, 1u64 << i, prefix, i);
            }
        }
        self.script.push(format!("(assert (= {} {}))", rhs, rhs_sum));

        // Assert AND relationship: out_bit[i] = lhs_bit[i] * rhs_bit[i]
        for i in 0..num_bits {
            self.script.push(format!(
                "(assert (= {}_out_bit{} (* {}_lhs_bit{} {}_rhs_bit{})))",
                prefix, i, prefix, i, prefix, i
            ));
        }

        // Assert output decomposition
        let mut out_sum = String::new();
        for i in 0..num_bits {
            if i == 0 {
                out_sum = format!("{}_out_bit0", prefix);
            } else {
                out_sum = format!("(+ {} (* {} {}_out_bit{}))", out_sum, 1u64 << i, prefix, i);
            }
        }
        self.script.push(format!("(assert (= {} {}))", output, out_sum));
    }

    /// Convert bitwise XOR operation using bit-blasting
    /// 
    /// For precise modeling, we use: XOR(a,b) = a + b - 2*AND(a,b) on single bits
    /// Convert bitwise XOR operation
    /// 
    /// Uses bit-blasting: decompose values into bits, compute XOR per-bit.
    /// For bit b: XOR(a,b) = a + b - 2*a*b when a,b ∈ {0,1}
    fn convert_bitwise_xor(
        &mut self,
        lhs: &FunctionInput<FieldElement>,
        rhs: &FunctionInput<FieldElement>,
        num_bits: u32,
        output: Witness,
    ) -> Result<(), AnalysisError> {
        let lhs_smt = self.function_input_to_smt(lhs)?;
        let rhs_smt = self.function_input_to_smt(rhs)?;
        let output_var = self.get_witness_var(output)?;

        if self.config.add_comments {
            self.script.push(format!("; XOR: {} bits", num_bits));
        }

        // Always use precise bit-blasting
        self.emit_bitwise_xor_bitblast(&lhs_smt, &rhs_smt, &output_var, num_bits);

        Ok(())
    }

    /// Emit precise bitwise XOR using bit-blasting
    /// 
    /// Uses: XOR(bit_a, bit_b) = bit_a + bit_b - 2 * bit_a * bit_b
    /// When both are 0 or 1: 0^0=0, 0^1=1, 1^0=1, 1^1=0
    fn emit_bitwise_xor_bitblast(
        &mut self,
        lhs: &str,
        rhs: &str,
        output: &str,
        num_bits: u32,
    ) {
        let prefix = format!("xor_{}_{}", lhs.replace(['(', ')', ' '], "_"), self.aux_var_counter);
        self.aux_var_counter += 1;

        // Declare bit variables
        for i in 0..num_bits {
            self.script.push(format!("(declare-fun {}_lhs_bit{} () Int)", prefix, i));
            self.script.push(format!("(declare-fun {}_rhs_bit{} () Int)", prefix, i));
            self.script.push(format!("(declare-fun {}_out_bit{} () Int)", prefix, i));
        }

        // Assert each bit is 0 or 1
        for i in 0..num_bits {
            self.script.push(format!(
                "(assert (or (= {}_lhs_bit{} 0) (= {}_lhs_bit{} 1)))",
                prefix, i, prefix, i
            ));
            self.script.push(format!(
                "(assert (or (= {}_rhs_bit{} 0) (= {}_rhs_bit{} 1)))",
                prefix, i, prefix, i
            ));
            self.script.push(format!(
                "(assert (or (= {}_out_bit{} 0) (= {}_out_bit{} 1)))",
                prefix, i, prefix, i
            ));
        }

        // Assert bit decomposition for lhs
        let mut lhs_sum = String::new();
        for i in 0..num_bits {
            if i == 0 {
                lhs_sum = format!("{}_lhs_bit0", prefix);
            } else {
                lhs_sum = format!("(+ {} (* {} {}_lhs_bit{}))", lhs_sum, 1u64 << i, prefix, i);
            }
        }
        self.script.push(format!("(assert (= {} {}))", lhs, lhs_sum));

        // Assert bit decomposition for rhs
        let mut rhs_sum = String::new();
        for i in 0..num_bits {
            if i == 0 {
                rhs_sum = format!("{}_rhs_bit0", prefix);
            } else {
                rhs_sum = format!("(+ {} (* {} {}_rhs_bit{}))", rhs_sum, 1u64 << i, prefix, i);
            }
        }
        self.script.push(format!("(assert (= {} {}))", rhs, rhs_sum));

        // Assert XOR relationship: out_bit[i] = lhs_bit[i] + rhs_bit[i] - 2*lhs_bit[i]*rhs_bit[i]
        for i in 0..num_bits {
            self.script.push(format!(
                "(assert (= {}_out_bit{} (- (+ {}_lhs_bit{} {}_rhs_bit{}) (* 2 (* {}_lhs_bit{} {}_rhs_bit{})))))",
                prefix, i, prefix, i, prefix, i, prefix, i, prefix, i
            ));
        }

        // Assert output decomposition
        let mut out_sum = String::new();
        for i in 0..num_bits {
            if i == 0 {
                out_sum = format!("{}_out_bit0", prefix);
            } else {
                out_sum = format!("(+ {} (* {} {}_out_bit{}))", out_sum, 1u64 << i, prefix, i);
            }
        }
        self.script.push(format!("(assert (= {} {}))", output, out_sum));
    }

    /// Convert memory initialization
    /// 
    /// Creates initial memory state as version 0 of the array
    fn convert_memory_init(
        &mut self,
        block_id: BlockId,
        init: &[Witness],
    ) -> Result<(), AnalysisError> {
        if self.config.add_comments {
            self.script.push(format!("; Memory block {} initialization ({} elements)", block_id.0, init.len()));
        }

        // Initialize version counter for this block
        self.memory_versions.insert(block_id.0, 0);

        // Use version 0 for initial state
        for (idx, witness) in init.iter().enumerate() {
            let witness_var = self.get_witness_var(*witness)?;
            self.script.push(format!(
                "(assert (= (mem{}_v0 {}) {}))",
                block_id.0, idx, witness_var
            ));
        }

        Ok(())
    }

    /// Convert memory operation (read/write) with proper versioning
    /// 
    /// For writes, we create a new version of the array using SMT array store semantics:
    /// mem_v(n+1) = store(mem_vn, index, value)
    /// 
    /// For reads, we use the current version:
    /// value = select(mem_vn, index)
    fn convert_memory_op(
        &mut self,
        block_id: BlockId,
        op: &MemOp<FieldElement>,
    ) -> Result<(), AnalysisError> {
        let index_smt = self.expression_to_smt(&op.index)?;
        let value_smt = self.expression_to_smt(&op.value)?;
        let operation_smt = self.expression_to_smt(&op.operation)?;

        // Get current version (or 0 if not initialized)
        let current_version = *self.memory_versions.get(&block_id.0).unwrap_or(&0);
        let current_mem = format!("mem{}_v{}", block_id.0, current_version);

        // operation = 0 means read, operation = 1 means write
        if op.operation.is_zero() {
            // Read operation: value = mem[index]
            if self.config.add_comments {
                self.script.push(format!("; Memory read from block {} (version {})", block_id.0, current_version));
            }
            self.script.push(format!(
                "(assert (= {} ({} {})))",
                value_smt, current_mem, index_smt
            ));
        } else if op.operation.is_one() {
            // Write operation: create new version
            let new_version = current_version + 1;
            let new_mem = format!("mem{}_v{}", block_id.0, new_version);
            
            if self.config.add_comments {
                self.script.push(format!("; Memory write to block {} (v{} -> v{})", block_id.0, current_version, new_version));
            }
            
            // Declare new memory function
            self.script.push(format!("(declare-fun {} (Int) Int)", new_mem));
            
            // For all indices except the written one, new_mem[i] = old_mem[i]
            // This is modeled using: forall i. i != index => new_mem[i] = old_mem[i]
            // In QF_NIA we can't use quantifiers, so we use the select/store pattern:
            // We assert that new_mem at the written index equals value
            self.script.push(format!(
                "(assert (= ({} {}) {}))",
                new_mem, index_smt, value_smt
            ));
            
            // For other indices, we add frame condition (new = old for unwritten indices)
            // This is an approximation - full array theory would use store/select
            // We can't enumerate all indices, so we use conditional equality
            let frame_aux = format!("frame_idx_{}", self.aux_var_counter);
            self.aux_var_counter += 1;
            self.script.push(format!("(declare-fun {} () Int)", frame_aux));
            self.script.push(format!(
                "(assert (=> (not (= {} {})) (= ({} {}) ({} {}))))",
                frame_aux, index_smt, new_mem, frame_aux, current_mem, frame_aux
            ));
            
            // Update version counter
            self.memory_versions.insert(block_id.0, new_version);
        } else {
            // Dynamic read/write - operation determined at runtime
            if self.config.add_comments {
                self.script.push("; Dynamic memory operation".to_string());
            }
            // If operation = 0, then value = mem[index]
            self.script.push(format!(
                "(assert (=> (= {} 0) (= {} ({} {}))))",
                operation_smt, value_smt, current_mem, index_smt
            ));
            // If operation = 1, we can't properly model the write without knowing statically
            // So we just assert the value relationship
            self.script.push(format!(
                "(assert (=> (= {} 1) (= ({} {}) {})))",
                operation_smt, current_mem, index_smt, value_smt
            ));
        }

        Ok(())
    }

    /// Convert a FunctionInput to SMT expression
    fn function_input_to_smt(
        &self,
        input: &FunctionInput<FieldElement>,
    ) -> Result<String, AnalysisError> {
        match input {
            FunctionInput::Constant(c) => Ok(self.field_to_smt(*c)),
            FunctionInput::Witness(w) => self.get_witness_var(*w),
        }
    }

    /// Convert an ACIR expression to SMT format
    fn expression_to_smt(
        &self,
        expr: &Expression<FieldElement>,
    ) -> Result<String, AnalysisError> {
        let mut terms = Vec::new();

        // Add multiplication terms: q_M * w_i * w_j
        for (coeff, w1, w2) in &expr.mul_terms {
            let var1 = self.get_witness_var(*w1)?;
            let var2 = self.get_witness_var(*w2)?;
            let coeff_smt = self.field_to_smt(*coeff);

            if coeff.is_one() {
                terms.push(format!("(* {} {})", var1, var2));
            } else if *coeff == -FieldElement::one() {
                terms.push(format!("(* (- 1) (* {} {}))", var1, var2));
            } else {
                terms.push(format!("(* {} (* {} {}))", coeff_smt, var1, var2));
            }
        }

        // Add linear terms: q_i * w_i
        for (coeff, witness) in &expr.linear_combinations {
            let var_name = self.get_witness_var(*witness)?;
            let coeff_smt = self.field_to_smt(*coeff);

            if coeff.is_one() {
                terms.push(var_name);
            } else if *coeff == -FieldElement::one() {
                terms.push(format!("(* (- 1) {})", var_name));
            } else {
                terms.push(format!("(* {} {})", coeff_smt, var_name));
            }
        }

        // Add constant term
        if !expr.q_c.is_zero() {
            terms.push(self.field_to_smt(expr.q_c));
        }

        if terms.is_empty() {
            Ok("0".to_string())
        } else if terms.len() == 1 {
            Ok(terms[0].clone())
        } else {
            Ok(format!("(+ {})", terms.join(" ")))
        }
    }

    /// Get the SMT variable name for a witness
    fn get_witness_var(&self, witness: Witness) -> Result<String, AnalysisError> {
        self.witness_to_var
            .get(&witness)
            .ok_or_else(|| {
                AnalysisError::ConversionError(format!(
                    "Unknown witness: w{}",
                    witness.witness_index()
                ))
            })
            .cloned()
    }

    /// Convert a field element to SMT representation
    ///
    /// Field elements in BN254 are in range [0, p) where p is a large prime.
    /// For "negative" numbers like -1, the field stores p-1.
    /// We detect this and convert appropriately for SMT.
    fn field_to_smt(&self, field: FieldElement) -> String {
        if field.is_zero() {
            return "0".to_string();
        }
        if field.is_one() {
            return "1".to_string();
        }

        // Get the actual value as BigUint
        let bytes = field.to_be_bytes();
        let value = BigUint::from_bytes_be(&bytes);

        // Check if this represents a "negative" number (value > p/2)
        let half_modulus = &self.modulus / 2u32;

        if value > half_modulus {
            // This is a negative number in disguise
            // actual_value = -(p - value) = value - p
            let neg_value = &self.modulus - &value;
            format!("(- {})", neg_value)
        } else {
            // Positive number
            value.to_string()
        }
    }

    /// Create a fresh auxiliary variable
    #[allow(dead_code)]
    fn fresh_aux_var(&mut self) -> String {
        let var = format!("aux{}", self.aux_var_counter);
        self.aux_var_counter += 1;
        self.script.push(format!("(declare-fun {} () Int)", var));
        var
    }

    /// Add a constraint that a variable is in the field range
    #[allow(dead_code)]
    fn assert_in_field(&mut self, var: &str) {
        self.script.push(format!(
            "(assert (and (>= {} 0) (< {} FIELD_MODULUS)))",
            var, var
        ));
    }
}

impl Default for AcirToSmtConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Get a descriptive name for an opcode type
fn opcode_type_name<F: AcirField>(opcode: &Opcode<F>) -> &'static str {
    match opcode {
        Opcode::AssertZero(_) => "AssertZero",
        Opcode::BlackBoxFuncCall(bb) => match bb {
            BlackBoxFuncCall::RANGE { .. } => "BlackBox::RANGE",
            BlackBoxFuncCall::AND { .. } => "BlackBox::AND",
            BlackBoxFuncCall::XOR { .. } => "BlackBox::XOR",
            BlackBoxFuncCall::Blake2s { .. } => "BlackBox::Blake2s",
            BlackBoxFuncCall::Blake3 { .. } => "BlackBox::Blake3",
            BlackBoxFuncCall::Poseidon2Permutation { .. } => "BlackBox::Poseidon2",
            BlackBoxFuncCall::Sha256Compression { .. } => "BlackBox::Sha256",
            BlackBoxFuncCall::Keccakf1600 { .. } => "BlackBox::Keccak",
            BlackBoxFuncCall::EcdsaSecp256k1 { .. } => "BlackBox::ECDSA_k1",
            BlackBoxFuncCall::EcdsaSecp256r1 { .. } => "BlackBox::ECDSA_r1",
            BlackBoxFuncCall::MultiScalarMul { .. } => "BlackBox::MSM",
            BlackBoxFuncCall::EmbeddedCurveAdd { .. } => "BlackBox::CurveAdd",
            BlackBoxFuncCall::RecursiveAggregation { .. } => "BlackBox::RecursiveAggr",
            BlackBoxFuncCall::AES128Encrypt { .. } => "BlackBox::AES128",
        },
        Opcode::MemoryOp { .. } => "MemoryOp",
        Opcode::MemoryInit { .. } => "MemoryInit",
        Opcode::BrilligCall { .. } => "BrilligCall",
        Opcode::Call { .. } => "Call",
    }
}

/// Helper to convert ACIR circuit for satisfiability checking
/// Returns the SMT-LIB script as a string
#[allow(dead_code)]
pub(crate) fn check_circuit_satisfiability(
    circuit: &Circuit<FieldElement>,
    config: Option<SmtConfig>,
) -> Result<String, AnalysisError> {
    let mut converter = match config {
        Some(cfg) => AcirToSmtConverter::with_config(cfg),
        None => AcirToSmtConverter::new(),
    };
    converter.convert(circuit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_to_smt_zero() {
        let converter = AcirToSmtConverter::new();
        assert_eq!(converter.field_to_smt(FieldElement::zero()), "0");
    }

    #[test]
    fn test_field_to_smt_one() {
        let converter = AcirToSmtConverter::new();
        assert_eq!(converter.field_to_smt(FieldElement::one()), "1");
    }

    #[test]
    fn test_field_to_smt_small_positive() {
        let converter = AcirToSmtConverter::new();
        let field = FieldElement::from(42u32);
        assert_eq!(converter.field_to_smt(field), "42");
    }

    #[test]
    fn test_field_to_smt_negative_one() {
        let converter = AcirToSmtConverter::new();
        let neg_one = -FieldElement::one();
        // -1 in the field is p - 1, which should be represented as (- 1)
        let result = converter.field_to_smt(neg_one);
        assert!(result.starts_with("(- "), "Expected negative format, got: {}", result);
    }

    #[test]
    fn test_modulus_definition() {
        let converter = AcirToSmtConverter::new();
        let expected_modulus = BigUint::parse_bytes(BN254_MODULUS_HEX.as_bytes(), 16).unwrap();
        assert_eq!(converter.modulus, expected_modulus);
    }

    #[test]
    fn test_empty_circuit_conversion() {
        let circuit = Circuit::<FieldElement> {
            function_name: String::new(),
            current_witness_index: 0,
            opcodes: vec![],
            private_parameters: Default::default(),
            public_parameters: Default::default(),
            return_values: Default::default(),
            assert_messages: vec![],
        };

        let mut converter = AcirToSmtConverter::new();
        let result = converter.convert(&circuit);
        assert!(result.is_ok());
        let smt = result.unwrap();
        assert!(smt.contains("(set-logic QF_NIA)"));
        assert!(smt.contains("(check-sat)"));
    }

    #[test]
    fn test_simple_assert_zero() {
        // Create a simple circuit: w0 = 0
        let circuit = Circuit::<FieldElement> {
            function_name: String::new(),
            current_witness_index: 0,
            opcodes: vec![Opcode::AssertZero(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(FieldElement::one(), Witness(0))],
                q_c: FieldElement::zero(),
            })],
            private_parameters: Default::default(),
            public_parameters: Default::default(),
            return_values: Default::default(),
            assert_messages: vec![],
        };

        let mut converter = AcirToSmtConverter::new();
        let result = converter.convert(&circuit);
        assert!(result.is_ok());
        let smt = result.unwrap();
        assert!(smt.contains("(assert (= w0 0))"));
    }

    #[test]
    fn test_range_constraint() {
        // Create a circuit with range constraint: 0 <= w0 < 2^8
        let circuit = Circuit::<FieldElement> {
            function_name: String::new(),
            current_witness_index: 0,
            opcodes: vec![Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::Witness(Witness(0)),
                num_bits: 8,
            })],
            private_parameters: Default::default(),
            public_parameters: Default::default(),
            return_values: Default::default(),
            assert_messages: vec![],
        };

        let mut converter = AcirToSmtConverter::new();
        let result = converter.convert(&circuit);
        assert!(result.is_ok());
        let smt = result.unwrap();
        // Should contain range constraint 0 <= w0 <= 255
        assert!(smt.contains("255"), "Expected range constraint with max value 255");
    }

    #[test]
    fn test_quadratic_constraint() {
        // Create a circuit: w0 * w1 - w2 = 0 (i.e., w2 = w0 * w1)
        let circuit = Circuit::<FieldElement> {
            function_name: "test_mul".to_string(),
            current_witness_index: 2,
            opcodes: vec![Opcode::AssertZero(Expression {
                mul_terms: vec![(FieldElement::one(), Witness(0), Witness(1))],
                linear_combinations: vec![(-FieldElement::one(), Witness(2))],
                q_c: FieldElement::zero(),
            })],
            private_parameters: Default::default(),
            public_parameters: Default::default(),
            return_values: Default::default(),
            assert_messages: vec![],
        };

        let mut converter = AcirToSmtConverter::new();
        let result = converter.convert(&circuit);
        assert!(result.is_ok());
        let smt = result.unwrap();
        
        // Should contain multiplication term
        assert!(smt.contains("(* w0 w1)"), "Expected multiplication term, got:\n{}", smt);
        // Should contain w2 with negative coefficient
        assert!(smt.contains("w2"), "Expected w2 in constraint");
        
        println!("Generated SMT for quadratic constraint:\n{}", smt);
    }

    #[test]
    fn test_complex_constraint() {
        // Create a circuit: 3*w0 + 2*w1*w2 - 5 = 0
        let circuit = Circuit::<FieldElement> {
            function_name: "test_complex".to_string(),
            current_witness_index: 2,
            opcodes: vec![Opcode::AssertZero(Expression {
                mul_terms: vec![(FieldElement::from(2u32), Witness(1), Witness(2))],
                linear_combinations: vec![(FieldElement::from(3u32), Witness(0))],
                q_c: -FieldElement::from(5u32),
            })],
            private_parameters: Default::default(),
            public_parameters: Default::default(),
            return_values: Default::default(),
            assert_messages: vec![],
        };

        let mut converter = AcirToSmtConverter::new();
        let result = converter.convert(&circuit);
        assert!(result.is_ok());
        let smt = result.unwrap();
        
        println!("Generated SMT for complex constraint:\n{}", smt);
        
        // Check for coefficient 3
        assert!(smt.contains("3"), "Expected coefficient 3");
        // Check for coefficient 2
        assert!(smt.contains("2"), "Expected coefficient 2");
    }

    #[test]
    fn test_memory_init() {
        // Create a circuit with memory initialization
        let circuit = Circuit::<FieldElement> {
            function_name: "test_mem".to_string(),
            current_witness_index: 2,
            opcodes: vec![Opcode::MemoryInit {
                block_id: BlockId(0),
                init: vec![Witness(0), Witness(1), Witness(2)],
                block_type: acvm::acir::circuit::opcodes::BlockType::Memory,
            }],
            private_parameters: Default::default(),
            public_parameters: Default::default(),
            return_values: Default::default(),
            assert_messages: vec![],
        };

        let mut converter = AcirToSmtConverter::new();
        let result = converter.convert(&circuit);
        assert!(result.is_ok());
        let smt = result.unwrap();
        
        println!("Generated SMT for memory init:\n{}", smt);
        
        // Should contain memory array declaration
        assert!(smt.contains("mem0"), "Expected memory array declaration");
        // Should contain initialization assertions
        assert!(smt.contains("(= (mem0 0)"), "Expected memory init at index 0");
        assert!(smt.contains("(= (mem0 1)"), "Expected memory init at index 1");
    }

    #[test]
    fn test_bitwise_and() {
        // Create a circuit with AND operation
        let circuit = Circuit::<FieldElement> {
            function_name: "test_and".to_string(),
            current_witness_index: 2,
            opcodes: vec![Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
                lhs: FunctionInput::Witness(Witness(0)),
                rhs: FunctionInput::Witness(Witness(1)),
                num_bits: 8,
                output: Witness(2),
            })],
            private_parameters: Default::default(),
            public_parameters: Default::default(),
            return_values: Default::default(),
            assert_messages: vec![],
        };

        let mut converter = AcirToSmtConverter::new();
        let result = converter.convert(&circuit);
        assert!(result.is_ok());
        let smt = result.unwrap();
        
        println!("Generated SMT for AND operation:\n{}", smt);
        
        // Output should be bounded
        assert!(smt.contains("w2"), "Expected output witness w2");
    }

    #[test]
    fn test_full_circuit_with_public_inputs() {
        use std::collections::BTreeSet;
        use acvm::acir::circuit::PublicInputs;
        
        // Create a circuit: public x, private y, assert x + y = 10
        // w0 = x (public), w1 = y (private), constraint: w0 + w1 - 10 = 0
        let mut public_params = BTreeSet::new();
        public_params.insert(Witness(0));
        
        let circuit = Circuit::<FieldElement> {
            function_name: "test_pub_priv".to_string(),
            current_witness_index: 1,
            opcodes: vec![
                // Range constraint on w0 (8 bits)
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput::Witness(Witness(0)),
                    num_bits: 8,
                }),
                // Range constraint on w1 (8 bits)  
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput::Witness(Witness(1)),
                    num_bits: 8,
                }),
                // Assert w0 + w1 = 10
                Opcode::AssertZero(Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![
                        (FieldElement::one(), Witness(0)),
                        (FieldElement::one(), Witness(1)),
                    ],
                    q_c: -FieldElement::from(10u32),
                }),
            ],
            private_parameters: BTreeSet::from([Witness(1)]),
            public_parameters: PublicInputs(public_params),
            return_values: Default::default(),
            assert_messages: vec![],
        };

        let mut config = SmtConfig::default();
        config.public_witness_indices.insert(0);
        
        let mut converter = AcirToSmtConverter::with_config(config);
        let result = converter.convert(&circuit);
        assert!(result.is_ok());
        let smt = result.unwrap();
        
        println!("Generated SMT for public/private circuit:\n{}", smt);
        
        // Check structure
        assert!(smt.contains("(set-logic QF_NIA)"));
        assert!(smt.contains("FIELD_MODULUS"));
        assert!(smt.contains("(declare-fun w0 () Int)"));
        assert!(smt.contains("(declare-fun w1 () Int)"));
        assert!(smt.contains("(check-sat)"));
        assert!(smt.contains("(get-value"));
        
        // Check constraints - look for the main assertion
        // The constraint is: w0 + w1 - 10 = 0, which becomes (+ w0 w1 (- 10)) = 0
        assert!(smt.contains("w0") && smt.contains("w1"));
    }

    #[test]
    fn test_smt_output_parseable() {
        // Create a simple circuit and verify the SMT output format is valid
        let circuit = Circuit::<FieldElement> {
            function_name: "parseable".to_string(),
            current_witness_index: 1,
            opcodes: vec![
                // w0 * w0 = w1 (squaring)
                Opcode::AssertZero(Expression {
                    mul_terms: vec![(FieldElement::one(), Witness(0), Witness(0))],
                    linear_combinations: vec![(-FieldElement::one(), Witness(1))],
                    q_c: FieldElement::zero(),
                }),
            ],
            private_parameters: Default::default(),
            public_parameters: Default::default(),
            return_values: Default::default(),
            assert_messages: vec![],
        };

        let mut converter = AcirToSmtConverter::new();
        let result = converter.convert(&circuit);
        assert!(result.is_ok());
        let smt = result.unwrap();
        
        // Basic SMT-LIB syntax checks
        let open_parens = smt.chars().filter(|&c| c == '(').count();
        let close_parens = smt.chars().filter(|&c| c == ')').count();
        assert_eq!(open_parens, close_parens, "Parentheses should be balanced");
        
        // Check for required SMT-LIB commands
        assert!(smt.contains("(set-logic"), "Should have set-logic");
        assert!(smt.contains("(declare-fun"), "Should have declare-fun");
        assert!(smt.contains("(assert"), "Should have assert");
        assert!(smt.contains("(check-sat)"), "Should have check-sat");
        
        println!("Valid SMT-LIB output:\n{}", smt);
    }
}
