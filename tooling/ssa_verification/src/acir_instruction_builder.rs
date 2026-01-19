use acvm::{
    FieldElement,
    acir::{
        circuit::{Circuit, Program as AcirProgram},
        native_types::Witness,
    },
};
use std::collections::BTreeSet;

use noirc_evaluator::ssa::{
    SsaEvaluatorOptions, ir::map::Id, optimize_ssa_builder_into_acir, primary_passes,
};
use noirc_evaluator::ssa::{SsaLogging, ir::function::Function};
use noirc_evaluator::ssa::{
    opt::{CONSTANT_FOLDING_MAX_ITER, INLINING_MAX_INSTRUCTIONS},
    ssa_gen::Ssa,
};

use noirc_evaluator::brillig::BrilligOptions;
use noirc_evaluator::ssa::{
    SsaBuilder,
    function_builder::FunctionBuilder,
    ir::{instruction::BinaryOp, types::Type},
};
use serde::{Deserialize, Serialize};

/// Represents artifacts generated from compiling an instruction
#[derive(Serialize, Deserialize)]
pub(crate) struct InstructionArtifacts {
    /// Name of the instruction
    pub(crate) instruction_name: String,

    /// SSA representation formatted as "acir(inline) {...}"
    pub(crate) formatted_ssa: String,

    /// JSON serialized SSA
    pub(crate) serialized_ssa: String,

    /// Gzipped bytes of ACIR program
    pub(crate) serialized_acir: Vec<u8>,
}

/// Represents the type of a variable in the instruction
#[derive(Debug)]
pub(crate) enum VariableType {
    /// Field element type
    Field,
    /// Unsigned integer type
    Unsigned,
    /// Signed integer type
    Signed,
}

/// Represents a variable with its type and size information
pub(crate) struct Variable {
    /// Type of the variable (Field, Unsigned, or Signed)
    pub(crate) variable_type: VariableType,
    /// Bit size of the variable (ignored for Field type)
    pub(crate) variable_size: u32,
}

impl Variable {
    /// Gets a string representation of the variable's type and size
    pub(crate) fn get_name(&self) -> String {
        format!("{:?}_{}", self.variable_type, self.variable_size)
    }
}

impl InstructionArtifacts {
    /// Converts a Variable into its corresponding SSA Type
    fn get_type(variable: &Variable) -> Type {
        match variable.variable_type {
            VariableType::Field => Type::field(),
            VariableType::Signed => Type::signed(variable.variable_size),
            VariableType::Unsigned => Type::unsigned(variable.variable_size),
        }
    }

    /// Creates a new binary operation instruction artifact
    fn new_binary(
        op: BinaryOp,
        instruction_name: String,
        first_variable: &Variable,
        second_variable: &Variable,
    ) -> Self {
        let first_variable_type = Self::get_type(first_variable);
        let second_variable_type = Self::get_type(second_variable);
        let ssa = binary_function(op, first_variable_type, second_variable_type);
        let serialized_ssa = &serde_json::to_string(&ssa).unwrap();
        let formatted_ssa = format!("{}", ssa.print_without_locations());

        let program = ssa_to_acir_program(ssa);
        let serialized_program = AcirProgram::serialize_program(&program);
        let name = format!(
            "{}_{}_{}",
            instruction_name,
            first_variable.get_name(),
            second_variable.get_name()
        );

        Self {
            instruction_name: name,
            formatted_ssa,
            serialized_ssa: serialized_ssa.to_string(),
            serialized_acir: serialized_program,
        }
    }

    /// Creates a new instruction artifact using a provided SSA generation function
    fn new_by_func(
        ssa_generate_function: fn(Type) -> Ssa,
        instruction_name: String,
        variable: &Variable,
    ) -> Self {
        let variable_type = Self::get_type(variable);
        let ssa = ssa_generate_function(variable_type);
        Self::new_by_ssa(ssa, instruction_name, variable)
    }

    fn new_by_ssa(ssa: Ssa, instruction_name: String, variable: &Variable) -> Self {
        let serialized_ssa = &serde_json::to_string(&ssa).unwrap();
        let formatted_ssa = format!("{}", ssa.print_without_locations());

        let program = ssa_to_acir_program(ssa);
        let serialized_program = AcirProgram::serialize_program(&program);
        let name = format!("{}_{}", instruction_name, variable.get_name());

        Self {
            instruction_name: name,
            formatted_ssa,
            serialized_ssa: serialized_ssa.to_string(),
            serialized_acir: serialized_program,
        }
    }

    /// Creates a new constrain instruction artifact
    pub(crate) fn new_constrain(variable: &Variable) -> Self {
        Self::new_by_func(constrain_function, "Constrain".into(), variable)
    }

    /// Creates a new NOT operation instruction artifact
    pub(crate) fn new_not(variable: &Variable) -> Self {
        Self::new_by_func(not_function, "Not".into(), variable)
    }

    /// Creates a new range check instruction artifact
    pub(crate) fn new_range_check(variable: &Variable, bit_size: u32) -> Self {
        let ssa = range_check_function(Self::get_type(variable), bit_size);
        Self::new_by_ssa(ssa, "RangeCheck".into(), variable)
    }

    /// Creates a new truncate instruction artifact
    pub(crate) fn new_truncate(variable: &Variable, bit_size: u32, max_bit_size: u32) -> Self {
        let ssa = truncate_function(Self::get_type(variable), bit_size, max_bit_size);
        Self::new_by_ssa(ssa, "Truncate".into(), variable)
    }

    /// Creates a new ADD operation instruction artifact
    pub(crate) fn new_add(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(
            BinaryOp::Add { unchecked: false },
            "Binary::Add".into(),
            first_variable,
            second_variable,
        )
    }

    /// Creates a new SUB operation instruction artifact
    pub(crate) fn new_sub(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(
            BinaryOp::Sub { unchecked: false },
            "Binary::Sub".into(),
            first_variable,
            second_variable,
        )
    }

    /// Creates a new XOR operation instruction artifact
    pub(crate) fn new_xor(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(BinaryOp::Xor, "Binary::Xor".into(), first_variable, second_variable)
    }

    /// Creates a new AND operation instruction artifact
    pub(crate) fn new_and(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(BinaryOp::And, "Binary::And".into(), first_variable, second_variable)
    }

    /// Creates a new OR operation instruction artifact
    pub(crate) fn new_or(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(BinaryOp::Or, "Binary::Or".into(), first_variable, second_variable)
    }

    /// Creates a new less than operation instruction artifact
    pub(crate) fn new_lt(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(BinaryOp::Lt, "Binary::Lt".into(), first_variable, second_variable)
    }

    /// Creates a new equals operation instruction artifact
    pub(crate) fn new_eq(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(BinaryOp::Eq, "Binary::Eq".into(), first_variable, second_variable)
    }

    /// Creates a new modulo operation instruction artifact
    pub(crate) fn new_mod(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(BinaryOp::Mod, "Binary::Mod".into(), first_variable, second_variable)
    }

    /// Creates a new multiply operation instruction artifact
    pub(crate) fn new_mul(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(
            BinaryOp::Mul { unchecked: false },
            "Binary::Mul".into(),
            first_variable,
            second_variable,
        )
    }

    /// Creates a new divide operation instruction artifact
    pub(crate) fn new_div(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(BinaryOp::Div, "Binary::Div".into(), first_variable, second_variable)
    }

    /// Creates a new shift left operation instruction artifact
    pub(crate) fn new_shl(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(BinaryOp::Shl, "Binary::Shl".into(), first_variable, second_variable)
    }

    /// Creates a new shift right operation instruction artifact
    pub(crate) fn new_shr(first_variable: &Variable, second_variable: &Variable) -> Self {
        Self::new_binary(BinaryOp::Shr, "Binary::Shr".into(), first_variable, second_variable)
    }
}

/// Converts SSA to ACIR program
fn ssa_to_acir_program(ssa: Ssa) -> AcirProgram<FieldElement> {
    // third brillig names, fourth errors
    let builder = SsaBuilder::from_ssa(ssa, SsaLogging::None, false, None);
    let ssa_evaluator_options = SsaEvaluatorOptions {
        ssa_logging: SsaLogging::None,
        print_codegen_timings: false,
        emit_ssa: { None },
        skip_underconstrained_check: true,
        skip_brillig_constraints_check: true,
        inliner_aggressiveness: 0,
        constant_folding_max_iter: CONSTANT_FOLDING_MAX_ITER,
        small_function_max_instruction: INLINING_MAX_INSTRUCTIONS,
        max_bytecode_increase_percent: None,
        brillig_options: BrilligOptions::default(),
        enable_brillig_constraints_check_lookback: false,
        skip_passes: vec![],
    };
    let (acir_functions, brillig, _) = match optimize_ssa_builder_into_acir(
        builder,
        &ssa_evaluator_options,
        &primary_passes(&ssa_evaluator_options),
    ) {
        Ok(artifacts_and_warnings) => artifacts_and_warnings.0,
        Err(_) => panic!("Should compile manually generated SSA into acir"),
    };

    let mut functions: Vec<Circuit<FieldElement>> = Vec::new();

    for acir_func in acir_functions.iter() {
        let mut private_params: BTreeSet<Witness> =
            acir_func.input_witnesses.clone().into_iter().collect();
        let ret_values: BTreeSet<Witness> =
            acir_func.return_witnesses.clone().into_iter().collect();

        private_params.extend(ret_values.iter().cloned());
        let circuit: Circuit<FieldElement> = Circuit {
            current_witness_index: acir_func.current_witness_index().witness_index(),
            opcodes: acir_func.opcodes.clone(),
            private_parameters: private_params.clone(),
            ..Circuit::<FieldElement>::default()
        };
        functions.push(circuit);
    }
    AcirProgram { functions, unconstrained_functions: brillig }
}

/// Creates an SSA function for binary operations
fn binary_function(op: BinaryOp, first_variable_type: Type, second_variable_type: Type) -> Ssa {
    // returns v0 op v1
    let main_id: Id<Function> = Id::new(0);
    let mut builder = FunctionBuilder::new("main".into(), main_id);
    let v0 = builder.add_parameter(first_variable_type);
    let v1 = builder.add_parameter(second_variable_type);
    let v2 = builder.insert_binary(v0, op, v1);
    builder.terminate_with_return(vec![v2]);

    builder.finish()
}

/// Creates an SSA function for constraint operations
fn constrain_function(variable_type: Type) -> Ssa {
    // constrains v0 == v1, returns v1
    let main_id: Id<Function> = Id::new(0);
    let mut builder = FunctionBuilder::new("main".into(), main_id);

    let v0 = builder.add_parameter(variable_type.clone());
    let v1 = builder.add_parameter(variable_type);
    builder.insert_constrain(v0, v1, None);
    builder.terminate_with_return(vec![v1]);

    builder.finish()
}

/// Creates an SSA function for range check operations
fn range_check_function(variable_type: Type, bit_size: u32) -> Ssa {
    let main_id: Id<Function> = Id::new(0);
    let mut builder = FunctionBuilder::new("main".into(), main_id);

    let v0 = builder.add_parameter(variable_type);
    builder.insert_range_check(v0, bit_size, Some("Range Check failed".to_string()));
    builder.terminate_with_return(vec![v0]);

    builder.finish()
}

/// Creates an SSA function for truncate operations
fn truncate_function(variable_type: Type, bit_size: u32, max_bit_size: u32) -> Ssa {
    let main_id: Id<Function> = Id::new(0);
    let mut builder = FunctionBuilder::new("main".into(), main_id);

    let v0 = builder.add_parameter(variable_type);
    let v1 = builder.insert_truncate(v0, bit_size, max_bit_size);
    builder.terminate_with_return(vec![v1]);

    builder.finish()
}

/// Creates an SSA function for NOT operations
fn not_function(variable_type: Type) -> Ssa {
    // returns not v0
    let main_id: Id<Function> = Id::new(0);
    let mut builder = FunctionBuilder::new("main".into(), main_id);

    let v0 = builder.add_parameter(variable_type);
    let v1 = builder.insert_not(v0);
    builder.terminate_with_return(vec![v1]);

    builder.finish()
}
