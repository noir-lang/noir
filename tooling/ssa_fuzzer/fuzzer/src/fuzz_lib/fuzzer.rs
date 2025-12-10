//! This module implements a fuzzer for testing and comparing ACIR and Brillig SSA implementations.
//! It generates random sequences of arithmetic and logical operations and ensures both implementations
//! produce identical results. Also it runs the fuzzer with constant values and checks if the results are the same.
//!
//! Main fuzz steps:
//!    1. Generate random witness
//!    2. Generate random sequence of instructions
//!    3. Insert instructions into ACIR and Brillig builders
//!    4. Get programs, and compile them
//!    5. Run and compare
//!
//! A bug is detected in two cases:
//!    - If programs return different results
//!    - If one program fails to compile but the other executes successfully

use super::{
    function_context::FunctionData,
    initial_witness::WitnessValue,
    instruction::InstructionBlock,
    options::{FuzzerMode, FuzzerOptions},
    program_context::{FuzzerProgramContext, program_context_by_mode},
};
use acvm::FieldElement;
use acvm::acir::native_types::{WitnessMap, WitnessStack};
use noir_ssa_fuzzer::{runner::execute, typed_value::Type};
use noirc_driver::CompiledProgram;
use noirc_evaluator::ssa::ir::function::RuntimeType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FuzzerData {
    pub(crate) functions: Vec<FunctionData>,
    pub(crate) initial_witness: Vec<WitnessValue>,
    pub(crate) instruction_blocks: Vec<InstructionBlock>,
}

impl Default for FuzzerData {
    fn default() -> Self {
        FuzzerData {
            functions: vec![FunctionData::default()],
            initial_witness: vec![WitnessValue::default()],
            instruction_blocks: vec![],
        }
    }
}

pub(crate) struct Fuzzer {
    pub(crate) contexts: Vec<FuzzerProgramContext>,
}

#[derive(Clone, Debug)]
pub(crate) struct FuzzerOutput {
    pub(crate) witness_stack: WitnessStack<FieldElement>,
    // None if the program failed to compile
    pub(crate) program: Option<CompiledProgram>,
}

pub(crate) enum CompareResults {
    Agree(Vec<FieldElement>),
    Disagree(Vec<FieldElement>, Vec<FieldElement>),
    LeftCompilationFailed,
    RightCompilationFailed,
    LeftExecutionFailed,
    RightExecutionFailed,
    BothFailed,
}

// TODO(sn): https://github.com/noir-lang/noir/issues/9743
impl FuzzerOutput {
    pub(crate) fn get_return_witnesses(&self) -> Vec<FieldElement> {
        // program failed to compile
        if self.program.is_none() {
            return vec![];
        }
        let return_witnesses = &self.program.as_ref().unwrap().program.functions[0].return_values.0;
        if return_witnesses.is_empty() {
            return vec![];
        }
        let max_return_witness_index = return_witnesses.iter().max().unwrap();
        let witness_vec = &self.witness_stack.peek().unwrap().witness;

        // program failed to execute
        if !witness_vec.contains_key(max_return_witness_index) {
            return vec![];
        }
        return_witnesses.iter().map(|witness| witness_vec[witness]).collect()
    }

    #[allow(dead_code)] // TODO(sn): used in fuzzer_output_to_json
    pub(crate) fn get_input_witnesses(&self) -> Vec<FieldElement> {
        // program failed to compile
        if self.program.is_none() {
            return vec![];
        }
        let input_witnesses =
            &self.program.as_ref().unwrap().program.functions[0].private_parameters;
        let witness_vec = &self.witness_stack.peek().unwrap().witness;
        input_witnesses.iter().map(|witness| witness_vec[witness]).collect()
    }

    pub(crate) fn compare_results(&self, other: &Self) -> CompareResults {
        match (self.is_program_compiled(), other.is_program_compiled()) {
            (false, false) => CompareResults::BothFailed,
            (true, false) => {
                if self.get_return_witnesses().is_empty() {
                    CompareResults::BothFailed
                } else {
                    CompareResults::RightCompilationFailed
                }
            }
            (false, true) => {
                if other.get_return_witnesses().is_empty() {
                    CompareResults::BothFailed
                } else {
                    CompareResults::LeftCompilationFailed
                }
            }
            (true, true) => {
                // both programs compiled successfully
                let left_return_witnesses = self.get_return_witnesses();
                let right_return_witnesses = other.get_return_witnesses();
                match (left_return_witnesses.is_empty(), right_return_witnesses.is_empty()) {
                    (true, true) => CompareResults::BothFailed,
                    (true, false) => CompareResults::LeftExecutionFailed,
                    (false, true) => CompareResults::RightExecutionFailed,
                    (false, false) => {
                        if left_return_witnesses != right_return_witnesses {
                            return CompareResults::Disagree(
                                left_return_witnesses.clone(),
                                right_return_witnesses.clone(),
                            );
                        }
                        CompareResults::Agree(left_return_witnesses)
                    }
                }
            }
        }
    }

    pub(crate) fn is_program_compiled(&self) -> bool {
        self.program.is_some()
    }
}

impl Fuzzer {
    pub(crate) fn new(
        runtime: RuntimeType,
        instruction_blocks: Vec<InstructionBlock>,
        values: Vec<FieldElement>,
        options: FuzzerOptions,
    ) -> Self {
        let mut contexts = vec![];
        for mode in &options.modes {
            contexts.push(program_context_by_mode(
                mode.clone(),
                runtime,
                instruction_blocks.clone(),
                values.clone(),
                options.clone(),
            ));
        }
        Self { contexts }
    }

    pub(crate) fn process_function(&mut self, function_data: FunctionData, types: Vec<Type>) {
        for context in &mut self.contexts {
            context.process_function(function_data.clone(), types.clone());
        }
    }

    /// Finalizes the function for contexts, executes and compares the results
    pub(crate) fn finalize_and_run(
        self,
        initial_witness: WitnessMap<FieldElement>,
    ) -> FuzzerOutput {
        let mut execution_results: HashMap<FuzzerMode, FuzzerOutput> = HashMap::new();
        for mut context in self.contexts {
            context.finalize_program();
            execution_results.insert(
                context.get_mode(),
                Self::execute_and_compare(context, initial_witness.clone()),
            );
        }
        let results_set = execution_results
            .values()
            .map(|result| result.get_return_witnesses())
            .collect::<HashSet<_>>();

        if results_set.len() != 1 {
            let mut panic_string = String::new();
            for (mode, result) in execution_results {
                if !result.get_return_witnesses().is_empty() {
                    panic_string
                        .push_str(&format!("Mode {mode:?}: {:?}\n", result.get_return_witnesses()));
                } else {
                    panic_string.push_str(&format!("Mode {mode:?} failed\n"));
                }
            }
            panic!("Fuzzer modes returned different results:\n{panic_string}");
        }
        execution_results.values().next().unwrap().clone()
    }

    fn execute_and_compare(
        context: FuzzerProgramContext,
        initial_witness: WitnessMap<FieldElement>,
    ) -> FuzzerOutput {
        let program = context.get_program();
        let input_witness_stack = WitnessStack::from(initial_witness.clone());
        if program.is_err() {
            return FuzzerOutput { witness_stack: input_witness_stack, program: None };
        }
        let witness_stack = execute(&program.as_ref().unwrap().program, initial_witness);
        if witness_stack.is_err() {
            return FuzzerOutput {
                witness_stack: input_witness_stack,
                program: Some(program.unwrap()),
            };
        }
        FuzzerOutput { witness_stack: witness_stack.unwrap(), program: Some(program.unwrap()) }
    }
}
