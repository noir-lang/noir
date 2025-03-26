use acvm::FieldElement;
use acvm::acir::native_types::Witness;
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::{
    builder::{FuzzerBuilder, FuzzerBuilderError},
    config,
    config::NUMBER_OF_VARIABLES_INITIAL,
    helpers::{id_to_int, u32_to_id_value},
};
use noirc_driver::CompiledProgram;
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::types::Type;
use noirc_evaluator::ssa::ir::value::Value;

#[derive(Arbitrary, Debug, Clone, Hash)]
pub(crate) enum Instructions {
    /// Addition of two values
    AddChecked { lhs: u32, rhs: u32 },
    /// Addition of two values without checking for overflow
    AddUnchecked { lhs: u32, rhs: u32 },
    /// Subtraction of two values
    SubChecked { lhs: u32, rhs: u32 },
    /// Subtraction of two values without checking for overflow
    SubUnchecked { lhs: u32, rhs: u32 },
    /// Multiplication of two values
    MulChecked { lhs: u32, rhs: u32 },
    /// Multiplication of two values without checking for overflow
    MulUnchecked { lhs: u32, rhs: u32 },
    /// Division of two values
    Div { lhs: u32, rhs: u32 },
    /// Equality comparison
    Eq { lhs: u32, rhs: u32 },
    /// Modulo operation
    Mod { lhs: u32, rhs: u32 },
    /// Bitwise NOT
    Not { lhs: u32 },
    /// Left shift
    Shl { lhs: u32, rhs: u32 },
    /// Right shift
    Shr { lhs: u32, rhs: u32 },
    /// Simple type cast
    SimpleCast { lhs: u32 },
    /// Cast to bigger type and back
    BigCastAndBack { lhs: u32, size: u32 },
    /// Array element access
    ArrayGet { array: u32, index: u32 },
    /// Array element assignment
    ArraySet { array: u32, index: u32, value: u32 },
    /// Array creation
    MakeArray { elements: Vec<u32> },
    /// Bitwise AND
    And { lhs: u32, rhs: u32 },
    /// Bitwise OR
    Or { lhs: u32, rhs: u32 },
    /// Bitwise XOR
    Xor { lhs: u32, rhs: u32 },
}
/// Represents an array in the SSA
#[derive(Copy, Clone)]
struct Array {
    id: Id<Value>,
    length: u32,
}

/// Main context for the fuzzer containing both ACIR and Brillig builders and their state
/// It works with indices of variables Ids, because it cannot handle Ids logic for ACIR and Brillig
pub(crate) struct FuzzerContext {
    /// ACIR builder
    acir_builder: FuzzerBuilder,
    /// Brillig builder
    brillig_builder: FuzzerBuilder,
    /// Ids of ACIR witnesses stored as u32
    acir_ids: Vec<u32>,
    /// Ids of Brillig witnesses stored as u32
    brillig_ids: Vec<u32>,
    /// ACIR arrays
    acir_arrays: Vec<Array>,
    /// Brillig arrays
    brillig_arrays: Vec<Array>,
    /// Whether the context is constant execution
    is_constant: bool,
}

impl FuzzerContext {
    /// Creates a new fuzzer context with the given type
    pub(crate) fn new(type_: Type) -> Self {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();
        acir_builder.insert_variables(type_.clone());
        brillig_builder.insert_variables(type_.clone());
        let mut acir_ids = vec![];
        let mut brillig_ids = vec![];
        // by default private variables ids are indexed from 0 to NUMBER_OF_VARIABLES_INITIAL - 1
        for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
            acir_ids.push(i);
            brillig_ids.push(i);
        }
        Self {
            acir_builder,
            brillig_builder,
            acir_ids,
            brillig_ids,
            acir_arrays: vec![],
            brillig_arrays: vec![],
            is_constant: false,
        }
    }

    /// Creates a new fuzzer context with the given values for a constant folding checking
    pub(crate) fn new_constant(values: Vec<impl Into<FieldElement>>, type_: Type) -> Self {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();
        acir_builder.set_type(type_.clone());
        brillig_builder.set_type(type_.clone());
        let mut acir_ids = vec![];
        let mut brillig_ids = vec![];
        for value in values {
            let field_element = value.into();
            acir_ids.push(id_to_int(acir_builder.insert_constant(field_element)));
            brillig_ids.push(id_to_int(brillig_builder.insert_constant(field_element)));
        }
        Self {
            acir_builder,
            brillig_builder,
            acir_ids,
            brillig_ids,
            acir_arrays: vec![],
            brillig_arrays: vec![],
            is_constant: true,
        }
    }

    /// Creates a new array from a vector of indices of variables
    /// Skips non-presented variables
    pub(crate) fn insert_array(&mut self, elements: Vec<u32>) {
        let mut acir_values_ids = vec![];
        let mut brillig_values_ids = vec![];
        for elem in elements {
            let acir_len = self.acir_ids.len();
            let brillig_len = self.brillig_ids.len();
            acir_values_ids.push(elem % acir_len as u32);
            brillig_values_ids.push(elem % brillig_len as u32);
        }
        let acir_len = acir_values_ids.len();
        let brillig_len = brillig_values_ids.len();
        let acir_array = self.acir_builder.insert_make_array(acir_values_ids);
        let brillig_array = self.brillig_builder.insert_make_array(brillig_values_ids);
        self.acir_arrays.push(Array { id: acir_array, length: acir_len as u32 });
        self.brillig_arrays.push(Array { id: brillig_array, length: brillig_len as u32 });
    }

    /// Gets an element from an array at the given index
    pub(crate) fn insert_array_get(&mut self, array_idx: u32, index: u32) {
        if self.acir_arrays.is_empty() {
            // no arrays created
            return;
        }
        // choose array by index
        let acir_arrays_len = self.acir_arrays.len() as u32;
        let brillig_arrays_len = self.brillig_arrays.len() as u32;
        let acir_array = self.acir_arrays[(array_idx % acir_arrays_len) as usize];
        let brillig_array = self.brillig_arrays[(array_idx % brillig_arrays_len) as usize];
        let acir_array_id = acir_array.id;
        let brillig_array_id = brillig_array.id;

        let acir_id = self.acir_ids[(index % acir_array.length) as usize];
        let brillig_id = self.brillig_ids[(index % brillig_array.length) as usize];
        let acir_result = self.acir_builder.insert_array_get(acir_array_id, acir_id);
        let brillig_result = self.brillig_builder.insert_array_get(brillig_array_id, brillig_id);
        self.acir_ids.push(id_to_int(acir_result));
        self.brillig_ids.push(id_to_int(brillig_result));
    }

    /// Sets an element in an array at the given index
    pub(crate) fn insert_array_set(&mut self, array_idx: u32, index: u32, value: u32) {
        if self.acir_arrays.is_empty() {
            // no arrays created
            return;
        }
        // choose array by index
        let acir_arrays_len = self.acir_arrays.len() as u32;
        let brillig_arrays_len = self.brillig_arrays.len() as u32;
        let acir_array = self.acir_arrays[(array_idx % acir_arrays_len) as usize];
        let brillig_array = self.brillig_arrays[(array_idx % brillig_arrays_len) as usize];
        let acir_array_id = acir_array.id;
        let brillig_array_id = brillig_array.id;

        let acir_id = self.acir_ids[(index % acir_array.length) as usize];
        let brillig_id = self.brillig_ids[(index % brillig_array.length) as usize];
        let acir_value = u32_to_id_value(value % (self.acir_ids.len() as u32));
        let brillig_value = u32_to_id_value(value % (self.brillig_ids.len() as u32));

        let acir_result = self.acir_builder.insert_array_set(acir_array_id, acir_id, acir_value);
        let brillig_result =
            self.brillig_builder.insert_array_set(brillig_array_id, brillig_id, brillig_value);
        self.acir_arrays.push(Array { id: acir_result, length: acir_array.length });
        self.brillig_arrays.push(Array { id: brillig_result, length: brillig_array.length });
    }

    /// Inserts an instruction that takes a single argument
    pub(crate) fn insert_instruction_with_single_arg(
        &mut self,
        arg: u32,
        f: fn(&mut FuzzerBuilder, Id<Value>) -> Id<Value>,
    ) {
        let acir_len = self.acir_ids.len() as u32;
        let brillig_len = self.brillig_ids.len() as u32;
        let acir_arg = u32_to_id_value(self.acir_ids[(arg % acir_len) as usize]);
        let brillig_arg = u32_to_id_value(self.brillig_ids[(arg % brillig_len) as usize]);
        let acir_result = f(&mut self.acir_builder, acir_arg);
        let brillig_result = f(&mut self.brillig_builder, brillig_arg);
        self.acir_ids.push(id_to_int(acir_result));
        self.brillig_ids.push(id_to_int(brillig_result));
    }

    /// Inserts an instruction that takes two arguments
    pub(crate) fn insert_instruction_with_double_args(
        &mut self,
        lhs: u32,
        rhs: u32,
        f: fn(&mut FuzzerBuilder, Id<Value>, Id<Value>) -> Id<Value>,
    ) {
        let acir_len = self.acir_ids.len() as u32;
        let brillig_len = self.brillig_ids.len() as u32;
        let acir_lhs = u32_to_id_value(self.acir_ids[(lhs % acir_len) as usize]);
        let acir_rhs = u32_to_id_value(self.acir_ids[(rhs % acir_len) as usize]);
        let brillig_lhs = u32_to_id_value(self.brillig_ids[(lhs % brillig_len) as usize]);
        let brillig_rhs = u32_to_id_value(self.brillig_ids[(rhs % brillig_len) as usize]);
        let acir_result = f(&mut self.acir_builder, acir_lhs, acir_rhs);
        let brillig_result = f(&mut self.brillig_builder, brillig_lhs, brillig_rhs);
        self.acir_ids.push(id_to_int(acir_result));
        self.brillig_ids.push(id_to_int(brillig_result));
    }

    /// Inserts an instruction into both ACIR and Brillig programs
    pub(crate) fn insert_instruction(&mut self, instruction: Instructions) {
        match instruction {
            Instructions::AddChecked { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_add_instruction_checked(lhs, rhs)
                });
            }
            Instructions::AddUnchecked { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_add_instruction_unchecked(lhs, rhs)
                });
            }
            Instructions::SubChecked { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_sub_instruction_checked(lhs, rhs)
                });
            }
            Instructions::SubUnchecked { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_sub_instruction_unchecked(lhs, rhs)
                });
            }
            Instructions::MulChecked { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_mul_instruction_checked(lhs, rhs)
                });
            }
            Instructions::MulUnchecked { lhs, rhs } => {
                self.insert_instruction_with_double_args(lhs, rhs, |builder, lhs, rhs| {
                    builder.insert_mul_instruction_unchecked(lhs, rhs)
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
            Instructions::SimpleCast { lhs } => {
                // TODO(sn): makearray([simplecast]) causes panic in compiler
                self.insert_instruction_with_single_arg(lhs, |builder, lhs| {
                    builder.insert_simple_cast(lhs)
                });
            }
            Instructions::MakeArray { elements } => {
                self.insert_array(elements);
            }
            Instructions::ArrayGet { array, index } => {
                self.insert_array_get(array, index);
            }
            Instructions::ArraySet { array, index, value } => {
                self.insert_array_set(array, index, value);
            }
            Instructions::BigCastAndBack { lhs, size } => {
                let acir_len = self.acir_ids.len() as u32;
                let brillig_len = self.brillig_ids.len() as u32;
                let acir_lhs = u32_to_id_value(self.acir_ids[(lhs % acir_len) as usize]);
                let brillig_lhs = u32_to_id_value(self.brillig_ids[(lhs % brillig_len) as usize]);
                let acir_result = self.acir_builder.insert_cast_bigger_and_back(acir_lhs, size);
                let brillig_result =
                    self.brillig_builder.insert_cast_bigger_and_back(brillig_lhs, size);
                self.acir_ids.push(id_to_int(acir_result));
                self.brillig_ids.push(id_to_int(brillig_result));
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
        let acir_result_index = *self.acir_ids.last().unwrap();
        let brillig_result_index = *self.brillig_ids.last().unwrap();
        self.acir_builder.finalize_function(u32_to_id_value(acir_result_index));
        self.brillig_builder.finalize_function(u32_to_id_value(brillig_result_index));
    }

    /// Returns witnesses for ACIR and Brillig
    /// If program does not have any instruction, it terminated with the last one witness
    /// Resulting WitnessStack of programs contains only variables and return value
    /// If we did not insert any instructions, WitnessStack contains only variables, so we return the last one
    /// If we inserted some instructions, WitnessStack contains return value, so we return the last one
    /// If we are in constant execution, we in witness stack only return value, so we return the Witness(0)
    pub(crate) fn get_return_witnesses(&self) -> (Witness, Witness) {
        if self.is_constant {
            (Witness(0), Witness(0))
        } else if self.acir_ids.len() as u32 != NUMBER_OF_VARIABLES_INITIAL {
            (Witness(NUMBER_OF_VARIABLES_INITIAL), Witness(NUMBER_OF_VARIABLES_INITIAL))
        } else {
            (Witness(NUMBER_OF_VARIABLES_INITIAL - 1), Witness(NUMBER_OF_VARIABLES_INITIAL - 1))
        }
    }

    /// Returns programs for ACIR and Brillig
    pub(crate) fn get_programs(self) -> (Result<CompiledProgram, FuzzerBuilderError>, Result<CompiledProgram, FuzzerBuilderError>) {
        (self.acir_builder.compile(), self.brillig_builder.compile())
    }
}
