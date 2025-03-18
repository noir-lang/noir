//! This module implements a fuzzer for testing and comparing ACIR and Brillig SSA implementations.
//! It generates random sequences of arithmetic and logical operations and ensures both implementations
//! produce identical results.
//! Main fuzz steps:
//! 0) Generate random witness
//! 1) Generate random sequence of instructions
//! 2) Insert instructions into ACIR and Brillig builders
//! 3) Get programs, and compile them
//! 4) Run and compare
//! if programs returned different results, then we have a bug
//! if one of the programs failed to compile, then we just execute the other one
//! and if the other one executed successfully, it's a bug

#![no_main]

use acvm::FieldElement;
use acvm::acir::native_types::{Witness, WitnessMap};
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noirc_driver::{CompileError, CompiledProgram};
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::types::Type;
use noirc_evaluator::ssa::ir::value::Value;
use ssa_fuzzer::{
    builder::FuzzerBuilder,
    config,
    config::NUMBER_OF_VARIABLES_INITIAL,
    helpers::{id_to_int, u32_to_id_value},
    runner::{execute_single, run_and_compare},
};

/// Represents the different types of instructions that can be fuzzed
#[derive(Arbitrary, Debug, Clone, Hash)]
enum Instructions {
    /// Addition of two values
    AddChecked {
        lhs: u32,
        rhs: u32,
    },
    AddUnchecked {
        lhs: u32,
        rhs: u32,
    },
    /// Subtraction of two values
    SubChecked {
        lhs: u32,
        rhs: u32,
    },
    SubUnchecked {
        lhs: u32,
        rhs: u32,
    },
    /// Multiplication of two values
    MulChecked {
        lhs: u32,
        rhs: u32,
    },
    MulUnchecked {
        lhs: u32,
        rhs: u32,
    },
    /// Division of two values
    Div {
        lhs: u32,
        rhs: u32,
    },
    /// Equality comparison
    Eq {
        lhs: u32,
        rhs: u32,
    },
    /// Modulo operation
    Mod {
        lhs: u32,
        rhs: u32,
    },
    /// Bitwise NOT
    Not {
        lhs: u32,
    },
    /// Left shift
    Shl {
        lhs: u32,
        rhs: u32,
    },
    /// Right shift
    Shr {
        lhs: u32,
        rhs: u32,
    },
    /// Simple type cast
    SimpleCast {
        lhs: u32,
    },
    /// Cast to bigger type and back
    BigCastAndBack {
        lhs: u32,
        size: u32,
    },
    /// Array element access
    ArrayGet {
        array: u32,
        index: u32,
    },
    /// Array element assignment
    ArraySet {
        array: u32,
        index: u32,
        value: u32,
    },
    /// Array creation
    MakeArray {
        elements: Vec<u32>,
    },
}
/// Represents an array in the SSA
#[derive(Copy, Clone)]
struct Array {
    id: Id<Value>,
    length: u32,
}

/// Main context for the fuzzer containing both ACIR and Brillig builders and their state
/// It works with indices of variables Ids, because it cannot handle Ids logic for ACIR and Brillig
struct FuzzerContext {
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
}

impl FuzzerContext {
    /// Creates a new fuzzer context with the given type
    fn new(type_: Type) -> Self {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();
        acir_builder.insert_variables(type_.clone());
        brillig_builder.insert_variables(type_.clone());
        let mut acir_ids = vec![];
        let mut brillig_ids = vec![];
        // by default private variables ids are indexed from 0 to NUMBER_OF_VARIABLES_INITIAL
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
        }
    }

    /// Creates a new array from a vector of indices of variables
    /// Skips non-presented variables
    fn insert_array(&mut self, elements: Vec<u32>) {
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
    fn insert_array_get(&mut self, array_idx: u32, index: u32) {
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
    fn insert_array_set(&mut self, array_idx: u32, index: u32, value: u32) {
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
        let value = u32_to_id_value(value);
        let acir_result = self.acir_builder.insert_array_set(acir_array_id, acir_id, value);
        let brillig_result =
            self.brillig_builder.insert_array_set(brillig_array_id, brillig_id, value);
        self.acir_arrays.push(Array { id: acir_result, length: acir_array.length });
        self.brillig_arrays.push(Array { id: brillig_result, length: brillig_array.length });
    }

    /// Inserts an instruction that takes a single argument
    fn insert_instruction_with_single_arg(
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
    fn insert_instruction_with_double_args(
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
    fn insert_instruction(&mut self, instruction: Instructions) {
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
            _ => {}
        }
    }

    /// Finalizes the function by setting the return value
    fn finalize_function(&mut self) {
        let acir_result_index = *self.acir_ids.last().unwrap();
        let brillig_result_index = *self.brillig_ids.last().unwrap();
        self.acir_builder.finalize_function(u32_to_id_value(acir_result_index));
        self.brillig_builder.finalize_function(u32_to_id_value(brillig_result_index));
    }

    /// Returns witnesses for ACIR and Brillig
    /// Only one witness added as return value is the last variable
    /// If acir_witnesses_indeces or brillig_witnesses_indeces are not equal to NUMBER_OF_VARIABLES_INITIAL,
    /// then it means that no instructions were added, so we just return the last variable set
    fn get_return_witnesses(&mut self) -> (Witness, Witness) {
        let acir_result_index = *self.acir_ids.last().unwrap();
        let brillig_result_index = *self.brillig_ids.last().unwrap();
        let mut acir_result_witness = Witness(acir_result_index);
        let mut brillig_result_witness = Witness(brillig_result_index);

        if self.acir_ids.len() as u32 != config::NUMBER_OF_VARIABLES_INITIAL {
            acir_result_witness = Witness(NUMBER_OF_VARIABLES_INITIAL);
            brillig_result_witness = Witness(NUMBER_OF_VARIABLES_INITIAL);
        }
        (acir_result_witness, brillig_result_witness)
    }

    /// Returns programs for ACIR and Brillig
    fn get_programs(
        self,
    ) -> (Result<CompiledProgram, CompileError>, Result<CompiledProgram, CompileError>) {
        (self.acir_builder.compile(), self.brillig_builder.compile())
    }
}

/// Represents the data for the fuzzer
/// `methods` - sequence of instructions to be added to the program
/// `initial_witness` - initial witness values for the program as String
#[derive(Arbitrary, Debug, Clone, Hash)]
struct FuzzerData {
    methods: Vec<Instructions>,
    initial_witness: [String; config::NUMBER_OF_VARIABLES_INITIAL as usize],
}

// main fuzz loop
libfuzzer_sys::fuzz_target!(|data: FuzzerData| {
    // init logger and initialize witness map
    let _ = env_logger::try_init();
    let type_ = Type::field();
    let mut witness_map = WitnessMap::new();
    for i in 0..config::NUMBER_OF_VARIABLES_INITIAL {
        let witness = Witness(i);
        // difference from uint.rs, we use try_from_str here
        let value = FieldElement::try_from_str(data.initial_witness.get(i as usize).unwrap());
        match value {
            Some(value) => {
                witness_map.insert(witness, value);
            }
            None => {
                return;
            }
        }
    }
    let initial_witness = witness_map;
    log::debug!("instructions: {:?}", data.methods.clone());
    log::debug!("initial_witness: {:?}", initial_witness);

    let mut fuzzer_context = FuzzerContext::new(type_.clone());
    for method in data.methods {
        fuzzer_context.insert_instruction(method);
    }
    fuzzer_context.finalize_function();
    let (acir_result_witness, brillig_result_witness) = fuzzer_context.get_return_witnesses();

    let (acir_program, brillig_program) = fuzzer_context.get_programs();
    let (acir_program, brillig_program) = match (acir_program, brillig_program) {
        (Ok(acir), Ok(brillig)) => (acir, brillig),
        (Err(_), Err(_)) => {
            return;
        }
        (Ok(acir), Err(e)) => {
            let acir_result = execute_single(&acir.program, initial_witness, acir_result_witness);
            match acir_result {
                Ok(result) => {
                    println!(
                        "ACIR compiled and successfully executed. Execution result of acir only {:?}",
                        result
                    );
                    panic!(
                        "ACIR compiled and successfully executed, 
                    but brillig compilation failed. Execution result of 
                    acir only {:?}. Brillig compilation failed with: {:?}",
                        result, e
                    );
                }
                Err(_e) => {
                    // if acir compiled, but didn't execute and brillig didn't compile, it's ok
                    return;
                }
            }
        }
        (Err(e), Ok(brillig)) => {
            let brillig_result =
                execute_single(&brillig.program, initial_witness, brillig_result_witness);
            match brillig_result {
                Ok(result) => {
                    println!(
                        "Brillig compiled and successfully executed. Execution result of brillig only {:?}",
                        result
                    );
                    panic!(
                        "Brillig compiled and successfully executed, 
                    but acir compilation failed. Execution result of 
                    brillig only {:?}. Acir compilation failed with: {:?}",
                        result, e
                    );
                }
                Err(_e) => {
                    // if brillig compiled, but didn't execute and acir didn't compile, it's ok
                    return;
                }
            }
        }
    };

    let (result, acir_result, brillig_result) = run_and_compare(
        &acir_program.program,
        &brillig_program.program,
        initial_witness,
        acir_result_witness,
        brillig_result_witness,
    );
    log::debug!("result: {:?}", result);
    log::debug!("acir_result: {:?}", acir_result);
    log::debug!("brillig_result: {:?}", brillig_result);

    assert!(result);
});
