use acvm::FieldElement;
use acvm::acir::native_types::Witness;
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::{
    builder::{FuzzerBuilder, FuzzerBuilderError, InstructionWithOneArg, InstructionWithTwoArgs},
    config::NUMBER_OF_VARIABLES_INITIAL,
    typed_value::{TypedValue, ValueType},
};
use noirc_driver::CompiledProgram;
use std::collections::HashMap;
#[derive(Arbitrary, Debug, Clone, Hash)]
pub(crate) struct Argument {
    /// Index of the argument in the context of stored variables of this type
    /// e.g. if we have variables with ids [0, 1] in u64 vector and variables with ids [5, 8] in fields vector
    /// Argument(Index(0), ValueType::U64) -> id 0
    /// Argument(Index(0), ValueType::Field) -> id 5
    /// Argument(Index(1), ValueType::Field) -> id 8
    index: usize,
    /// Type of the argument
    value_type: ValueType,
}

#[derive(Arbitrary, Debug, Clone, Hash)]
pub(crate) enum Instructions {
    /// Addition of two values
    AddChecked { lhs: Argument, rhs: Argument },
    /// Subtraction of two values
    SubChecked { lhs: Argument, rhs: Argument },
    /// Multiplication of two values
    MulChecked { lhs: Argument, rhs: Argument },
    /// Division of two values
    Div { lhs: Argument, rhs: Argument },
    /// Equality comparison
    Eq { lhs: Argument, rhs: Argument },
    /// Modulo operation
    Mod { lhs: Argument, rhs: Argument },
    /// Bitwise NOT
    Not { lhs: Argument },
    /// Left shift
    Shl { lhs: Argument, rhs: Argument },
    /// Right shift
    Shr { lhs: Argument, rhs: Argument },
    /// Cast into type
    Cast { lhs: Argument, type_: ValueType },
    /// Bitwise AND
    And { lhs: Argument, rhs: Argument },
    /// Bitwise OR
    Or { lhs: Argument, rhs: Argument },
    /// Bitwise XOR
    Xor { lhs: Argument, rhs: Argument },
}

/// Main context for the fuzzer containing both ACIR and Brillig builders and their state
/// It works with indices of variables Ids, because it cannot handle Ids logic for ACIR and Brillig
pub(crate) struct FuzzerContext {
    /// ACIR builder
    acir_builder: FuzzerBuilder,
    /// Brillig builder
    brillig_builder: FuzzerBuilder,
    /// Ids of ACIR witnesses stored as TypedValue separated by type
    acir_ids: HashMap<ValueType, Vec<TypedValue>>,
    /// Ids of Brillig witnesses stored as TypedValue separated by type
    brillig_ids: HashMap<ValueType, Vec<TypedValue>>,
    /// ACIR and Brillig last changed value
    last_value_acir: Option<TypedValue>,
    last_value_brillig: Option<TypedValue>,
    /// Whether the context is constant execution
    is_constant: bool,
}

/// Returns a typed value from the map
/// Variables are stored in a map with type as key and vector of typed values as value
/// We use modulo to wrap index around the length of the vector, because fuzzer can produce index that is greater than the length of the vector
fn get_typed_value_from_map(
    map: &HashMap<ValueType, Vec<TypedValue>>,
    type_: ValueType,
    idx: usize,
) -> Option<TypedValue> {
    let arr = map.get(&type_);
    arr?;
    let arr = arr.unwrap();
    let value = arr.get(idx % arr.len());
    value?;
    Some(value.unwrap().clone())
}

fn append_typed_value_to_map(
    map: &mut HashMap<ValueType, Vec<TypedValue>>,
    type_: ValueType,
    value: TypedValue,
) {
    map.entry(type_.clone()).or_default().push(value);
}

impl FuzzerContext {
    /// Creates a new fuzzer context with the given types
    /// It creates a new variable for each type and stores it in the map
    ///
    /// For example, if we have types [u64, u64, field], it will create 3 variables
    /// and store them in the map as {u64: [0, 1], field: [2]} for both ACIR and Brillig
    pub(crate) fn new(types: Vec<ValueType>) -> Self {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();
        let mut acir_ids = HashMap::new();
        let mut brillig_ids = HashMap::new();
        for type_ in types {
            let acir_id = acir_builder.insert_variable(type_.to_ssa_type());
            let brillig_id = brillig_builder.insert_variable(type_.to_ssa_type());
            acir_ids.entry(type_.clone()).or_insert(Vec::new()).push(acir_id);
            brillig_ids.entry(type_).or_insert(Vec::new()).push(brillig_id);
        }

        Self {
            acir_builder,
            brillig_builder,
            acir_ids,
            brillig_ids,
            last_value_acir: None,
            last_value_brillig: None,
            is_constant: false,
        }
    }

    /// Creates a new fuzzer context with the given values for a constant folding checking
    ///
    /// For example, if we have values [1, 2, 3] and types [u64, u64, field], it will create 3 constants
    /// and store them in the map as {u64: [0, 1], field: [2]} for both ACIR and Brillig
    pub(crate) fn new_constant_context(
        values: Vec<impl Into<FieldElement>>,
        types: Vec<ValueType>,
    ) -> Self {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();
        let mut acir_ids = HashMap::new();
        let mut brillig_ids = HashMap::new();

        for (value, type_) in values.into_iter().zip(types) {
            let field_element = value.into();
            acir_ids
                .entry(type_.clone())
                .or_insert(Vec::new())
                .push(acir_builder.insert_constant(field_element, type_.clone()));
            brillig_ids
                .entry(type_.clone())
                .or_insert(Vec::new())
                .push(brillig_builder.insert_constant(field_element, type_.clone()));
        }

        Self {
            acir_builder,
            brillig_builder,
            acir_ids,
            brillig_ids,
            last_value_acir: None,
            last_value_brillig: None,
            is_constant: true,
        }
    }

    /// Inserts an instruction that takes a single argument
    pub(crate) fn insert_instruction_with_single_arg(
        &mut self,
        arg: Argument,
        instruction: InstructionWithOneArg,
    ) {
        let acir_arg = get_typed_value_from_map(&self.acir_ids, arg.value_type.clone(), arg.index);
        let brillig_arg =
            get_typed_value_from_map(&self.brillig_ids, arg.value_type.clone(), arg.index);
        let (acir_arg, brillig_arg) = match (acir_arg, brillig_arg) {
            (Some(acir_arg), Some(brillig_arg)) => (acir_arg, brillig_arg),
            _ => return,
        };
        let acir_result = instruction(&mut self.acir_builder, acir_arg);
        let brillig_result = instruction(&mut self.brillig_builder, brillig_arg);
        self.last_value_acir = Some(acir_result.clone());
        self.last_value_brillig = Some(brillig_result.clone());
        append_typed_value_to_map(&mut self.acir_ids, acir_result.to_value_type(), acir_result);
        append_typed_value_to_map(
            &mut self.brillig_ids,
            brillig_result.to_value_type(),
            brillig_result,
        );
    }

    /// Inserts an instruction that takes two arguments
    pub(crate) fn insert_instruction_with_double_args(
        &mut self,
        lhs: Argument,
        rhs: Argument,
        instruction: InstructionWithTwoArgs,
    ) {
        let acir_lhs = get_typed_value_from_map(&self.acir_ids, lhs.value_type.clone(), lhs.index);
        let acir_rhs = get_typed_value_from_map(&self.acir_ids, rhs.value_type.clone(), rhs.index);
        let (acir_lhs, acir_rhs) = match (acir_lhs, acir_rhs) {
            (Some(acir_lhs), Some(acir_rhs)) => (acir_lhs, acir_rhs),
            _ => return,
        };
        let acir_result = instruction(&mut self.acir_builder, acir_lhs, acir_rhs);
        let brillig_lhs =
            get_typed_value_from_map(&self.brillig_ids, lhs.value_type.clone(), lhs.index);
        let brillig_rhs =
            get_typed_value_from_map(&self.brillig_ids, rhs.value_type.clone(), rhs.index);
        let (brillig_lhs, brillig_rhs) = match (brillig_lhs, brillig_rhs) {
            (Some(brillig_lhs), Some(brillig_rhs)) => (brillig_lhs, brillig_rhs),
            _ => return,
        };
        let brillig_result = instruction(&mut self.brillig_builder, brillig_lhs, brillig_rhs);
        self.last_value_acir = Some(acir_result.clone());
        self.last_value_brillig = Some(brillig_result.clone());
        append_typed_value_to_map(&mut self.acir_ids, acir_result.to_value_type(), acir_result);
        append_typed_value_to_map(
            &mut self.brillig_ids,
            brillig_result.to_value_type(),
            brillig_result,
        );
    }

    /// Inserts an instruction into both ACIR and Brillig programs
    pub(crate) fn insert_instruction(&mut self, instruction: Instructions) {
        match instruction {
            Instructions::AddChecked { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_add_instruction_checked(lhs, rhs)
                });
            }
            Instructions::SubChecked { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_sub_instruction_checked(lhs, rhs)
                });
            }
            Instructions::MulChecked { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_mul_instruction_checked(lhs, rhs)
                });
            }
            Instructions::Div { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_div_instruction(lhs, rhs)
                });
            }
            Instructions::Eq { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_eq_instruction(lhs, rhs)
                });
            }
            Instructions::Cast { lhs, type_ } => {
                let acir_lhs =
                    get_typed_value_from_map(&self.acir_ids, lhs.value_type.clone(), lhs.index);
                let brillig_lhs =
                    get_typed_value_from_map(&self.brillig_ids, lhs.value_type.clone(), lhs.index);
                let (acir_lhs, brillig_lhs) = match (acir_lhs, brillig_lhs) {
                    (Some(acir_lhs), Some(brillig_lhs)) => (acir_lhs, brillig_lhs),
                    _ => return,
                };
                let acir_result = self.acir_builder.insert_cast(acir_lhs, type_.clone());
                let brillig_result = self.brillig_builder.insert_cast(brillig_lhs, type_.clone());
                self.last_value_acir = Some(acir_result.clone());
                self.last_value_brillig = Some(brillig_result.clone());
                append_typed_value_to_map(
                    &mut self.acir_ids,
                    acir_result.to_value_type(),
                    acir_result,
                );
                append_typed_value_to_map(
                    &mut self.brillig_ids,
                    brillig_result.to_value_type(),
                    brillig_result,
                );
            }
            Instructions::Mod { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_mod_instruction(lhs, rhs)
                });
            }
            Instructions::Not { lhs } => {
                self.insert_instruction_with_single_arg(lhs, |builder, lhs| {
                    builder.insert_not_instruction(lhs)
                });
            }
            Instructions::Shl { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_shl_instruction(lhs, rhs)
                });
            }
            Instructions::Shr { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_shr_instruction(lhs, rhs)
                });
            }
            Instructions::And { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_and_instruction(lhs, rhs)
                });
            }
            Instructions::Or { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_or_instruction(lhs, rhs)
                });
            }
            Instructions::Xor { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_xor_instruction(lhs, rhs)
                });
            }
        }
    }

    /// Finalizes the function by setting the return value
    pub(crate) fn finalize_function(&mut self) {
        match (self.last_value_acir.clone(), self.last_value_brillig.clone()) {
            (Some(acir_result), Some(brillig_result)) => {
                self.acir_builder.finalize_function(acir_result);
                self.brillig_builder.finalize_function(brillig_result);
            }
            _ => {
                // If no last value was set, use the first value from the first type in each map
                let first_type =
                    self.acir_ids.keys().next().expect("Should have at least one type");

                let acir_result = self
                    .acir_ids
                    .get(first_type)
                    .and_then(|values| values.first().cloned())
                    .expect("Should have at least one value");

                let brillig_result = self
                    .brillig_ids
                    .get(first_type)
                    .and_then(|values| values.first().cloned())
                    .expect("Should have at least one value");

                self.acir_builder.finalize_function(acir_result);
                self.brillig_builder.finalize_function(brillig_result);
            }
        }
    }

    /// Returns witnesses for ACIR and Brillig
    /// If program does not have any instructions, it terminated with the last witness
    /// Resulting WitnessStack of programs contains only variables and return value
    /// If we inserted some instructions, WitnessStack contains return value, so we return the last one
    /// If we are checking constant folding, the witness stack will only contain the return value, so we return Witness(0)
    pub(crate) fn get_return_witnesses(&self) -> (Witness, Witness) {
        if self.is_constant {
            return (Witness(0), Witness(0));
        }
        match (self.last_value_acir.clone(), self.last_value_brillig.clone()) {
            (Some(_acir_result), Some(_brillig_result)) => {
                (Witness(NUMBER_OF_VARIABLES_INITIAL - 1), Witness(NUMBER_OF_VARIABLES_INITIAL - 1))
            }
            _ => (Witness(NUMBER_OF_VARIABLES_INITIAL), Witness(NUMBER_OF_VARIABLES_INITIAL)),
        }
    }

    /// Returns programs for ACIR and Brillig
    pub(crate) fn get_programs(
        self,
    ) -> (Result<CompiledProgram, FuzzerBuilderError>, Result<CompiledProgram, FuzzerBuilderError>)
    {
        (self.acir_builder.compile(), self.brillig_builder.compile())
    }
}
