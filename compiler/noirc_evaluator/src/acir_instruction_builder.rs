use std::collections::BTreeSet;
use acvm::{
    acir::{
        circuit::{
            Circuit, ExpressionWidth,
            Program as AcirProgram
        },
        native_types::Witness,
    },
    FieldElement,
};

use crate::ssa::ssa_gen::Ssa;
use crate::ssa::ir::map::Id;
use crate::ssa::ir::function::Function;

use crate::ssa::{
    function_builder::FunctionBuilder,
    ir::{instruction::BinaryOp, types::Type},
};
use crate::brillig::Brillig;
use serde::{Deserialize, Serialize};

/// Represents artifacts generated from compiling an instruction
#[derive(Serialize, Deserialize)]
pub struct InstructionArtifacts {
    /// Name of the instruction
    pub instruction_name: String,

    /// SSA representation formatted as "acir(inline) {...}"
    pub formatted_ssa: String,

    /// JSON serialized SSA
    pub serialized_ssa: String,

    /// Gzipped bytes of ACIR program
    pub serialized_acir: Vec<u8>,
}

/// Represents the type of a variable in the instruction
#[derive(Debug)]
pub enum VariableType {
    /// Field element type
    Field,
    /// Unsigned integer type
    Unsigned,
    /// Signed integer type
    Signed
}

/// Represents a variable with its type and size information
pub struct Variable {
    /// Type of the variable (Field, Unsigned, or Signed)
    pub variable_type: VariableType,
    /// Bit size of the variable (ignored for Field type)
    pub variable_size: u32,
}

impl Variable {
    /// Gets a string representation of the variable's type and size
    pub fn get_name(&self) -> String {
        return format!("{:?}_{}", self.variable_type, self.variable_size)
    }
}

impl InstructionArtifacts {
    /// Converts a Variable into its corresponding SSA Type
    fn get_type(variable: &Variable) -> Type {
        match variable.variable_type {
            VariableType::Field => Type::field(),
            VariableType::Signed => Type::signed(variable.variable_size),
            VariableType::Unsigned => Type::unsigned(variable.variable_size)
        }
    }

    /// Creates a new binary operation instruction artifact
    fn new_binary(op: BinaryOp, instruction_name: String, first_variable: &Variable, second_variable: &Variable) -> Self {
        let first_variable_type = Self::get_type(first_variable);
        let second_variable_type = Self::get_type(second_variable);
        let ssa = binary_function(op, first_variable_type, second_variable_type);
        let serialized_ssa = &serde_json::to_string(&ssa).unwrap();
        let formatted_ssa = format!("{}", ssa);

        let program = ssa_to_acir_program(ssa);
        let serialized_program = AcirProgram::serialize_program(&program);
        let name = format!("{}_{}_{}", instruction_name, first_variable.get_name(), second_variable.get_name());

        Self {
            instruction_name: name,
            formatted_ssa: formatted_ssa,
            serialized_ssa: serialized_ssa.to_string(),
            serialized_acir: serialized_program
        }
    }

    /// Creates a new instruction artifact using a provided SSA generation function
    fn new_by_func(ssa_generate_function: fn(Type) -> Ssa, instruction_name: String, variable: &Variable) -> Self {
        let variable_type = Self::get_type(variable);
        let ssa = ssa_generate_function(variable_type);
        let serialized_ssa = &serde_json::to_string(&ssa).unwrap();
        let formatted_ssa = format!("{}", ssa);

        let program = ssa_to_acir_program(ssa);
        let serialized_program = AcirProgram::serialize_program(&program);
        let name = format!("{}_{}", instruction_name, variable.get_name());

        Self {
            instruction_name: name,
            formatted_ssa: formatted_ssa,
            serialized_ssa: serialized_ssa.to_string(),
            serialized_acir: serialized_program
        }
    }

    /// Creates a new constrain instruction artifact
    pub fn new_constrain(variable: &Variable) -> Self {
        return Self::new_by_func(constrain_function, "Constrain".into(), variable)
    }

    /// Creates a new NOT operation instruction artifact
    pub fn new_not(variable: &Variable) -> Self {
        return Self::new_by_func(not_function, "Not".into(), variable)
    }

    /// Creates a new range check instruction artifact
    pub fn new_range_check(variable: &Variable) -> Self {
        return Self::new_by_func(range_check_function, "RangeCheck".into(), variable)
    }

    /// Creates a new truncate instruction artifact
    pub fn new_truncate(variable: &Variable) -> Self {
        return Self::new_by_func(truncate_function, "Truncate".into(), variable)
    }

    /// Creates a new ADD operation instruction artifact
    pub fn new_add(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::Add, "Binary::Add".into(), first_variable, second_variable);
    }

    /// Creates a new SUB operation instruction artifact
    pub fn new_sub(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::Sub, "Binary::Sub".into(), first_variable, second_variable);
    }

    /// Creates a new XOR operation instruction artifact
    pub fn new_xor(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::Xor, "Binary::Xor".into(), first_variable, second_variable);
    }

    /// Creates a new AND operation instruction artifact
    pub fn new_and(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::And, "Binary::And".into(), first_variable, second_variable);
    }

    /// Creates a new OR operation instruction artifact
    pub fn new_or(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::Or, "Binary::Or".into(), first_variable, second_variable);
    }

    /// Creates a new less than operation instruction artifact
    pub fn new_lt(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::Lt, "Binary::Lt".into(), first_variable, second_variable);
    }

    /// Creates a new equals operation instruction artifact
    pub fn new_eq(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::Eq, "Binary::Eq".into(), first_variable, second_variable);
    }

    /// Creates a new modulo operation instruction artifact
    pub fn new_mod(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::Mod, "Binary::Mod".into(), first_variable, second_variable);
    }

    /// Creates a new multiply operation instruction artifact
    pub fn new_mul(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::Mul, "Binary::Mul".into(), first_variable, second_variable);
    }

    /// Creates a new divide operation instruction artifact
    pub fn new_div(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::Div, "Binary::Div".into(), first_variable, second_variable);
    }

    /// Creates a new shift left operation instruction artifact
    pub fn new_shl(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::Shl, "Binary::Shl".into(), first_variable, second_variable);
    }

    /// Creates a new shift right operation instruction artifact
    pub fn new_shr(first_variable: &Variable, second_variable: &Variable) -> Self {
        return Self::new_binary(BinaryOp::Shr, "Binary::Shr".into(), first_variable, second_variable);
    }
}

/// Converts SSA to ACIR program
fn ssa_to_acir_program(ssa: Ssa) -> AcirProgram<FieldElement> {
    // third brillig names, fourth errors
    let (acir_functions, brillig, _, _) = ssa
        .into_acir(&Brillig::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");

    let mut functions: Vec<Circuit<FieldElement>> = Vec::new();

    for acir_func in acir_functions.iter() {
        let mut private_params: BTreeSet<Witness> = acir_func.input_witnesses.clone().into_iter().collect();
        let ret_values: BTreeSet<Witness> = acir_func.return_witnesses.clone().into_iter().collect();
        let circuit: Circuit<FieldElement>;
        private_params.extend(ret_values.iter().cloned());
        circuit = Circuit {
            current_witness_index: acir_func.current_witness_index().witness_index(),
            opcodes: acir_func.opcodes().to_vec(),
            private_parameters: private_params.clone(),
            ..Circuit::<FieldElement>::default()
        };
        functions.push(circuit);
    }
    return AcirProgram { functions: functions, unconstrained_functions: brillig };
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

    let func = builder.finish();
    // remove_bit_shifts replaces bit shifts with equivalent arithmetic operations
    let cleared_func = func.remove_bit_shifts();
    return cleared_func;
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

    return builder.finish();
}

/// Creates an SSA function for range check operations
fn range_check_function(variable_type: Type) -> Ssa {
    let main_id: Id<Function> = Id::new(0);
    let mut builder = FunctionBuilder::new("main".into(), main_id);

    let v0 = builder.add_parameter(variable_type);
    builder.insert_range_check(v0, 64, Some("Range Check failed".to_string()));
    builder.terminate_with_return(vec![v0]);

    return builder.finish()
}

/// Creates an SSA function for truncate operations
fn truncate_function(variable_type: Type) -> Ssa {
    // truncate v0: field to bit size 10 with max bit size 20.
    let main_id: Id<Function> = Id::new(0);
    let mut builder = FunctionBuilder::new("main".into(), main_id);

    let v0 = builder.add_parameter(variable_type);
    let v1 = builder.insert_truncate(v0, 10, 20);
    builder.terminate_with_return(vec![v1]);

    return builder.finish();
}

/// Creates an SSA function for NOT operations
fn not_function(variable_type: Type) -> Ssa {
    // returns not v0
    let main_id: Id<Function> = Id::new(0);
    let mut builder = FunctionBuilder::new("main".into(), main_id);

    let v0 = builder.add_parameter(variable_type);
    let v1 = builder.insert_not(v0);
    builder.terminate_with_return(vec![v1]);

    return builder.finish()
}
