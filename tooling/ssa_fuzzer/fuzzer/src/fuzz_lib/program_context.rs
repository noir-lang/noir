use super::{
    function_context::{FunctionData, FuzzerFunctionCommand, FuzzerFunctionContext},
    instruction::{FunctionInfo, InstructionBlock},
    options::{FunctionContextOptions, FuzzerMode, FuzzerOptions},
};
use acvm::FieldElement;
use noir_ssa_fuzzer::{
    builder::{FuzzerBuilder, FuzzerBuilderError},
    typed_value::Type,
};
use noirc_artifacts::program::CompiledProgram;
use noirc_evaluator::ssa::ir::{
    function::{Function, RuntimeType},
    map::Id,
};
use std::collections::BTreeMap;

struct StoredFunction {
    id: Id<Function>,
    function: FunctionData,
    types: Vec<Type>,
}

/// FuzzerProgramContext is a context for storing and processing SSA functions
pub(crate) struct FuzzerProgramContext {
    /// Builder for the program
    builder: FuzzerBuilder,
    /// Options for the program context
    program_context_options: FunctionContextOptions, // TODO
    /// Whether the program is executed in constants
    is_constant: bool,
    /// Function information
    function_information: BTreeMap<Id<Function>, FunctionInfo>,
    /// Stored functions
    stored_functions: Vec<StoredFunction>,
    /// Current function id
    current_function_id: Id<Function>,
    /// Instruction blocks
    instruction_blocks: Vec<InstructionBlock>,
    /// Main initialized
    is_main_initialized: bool,
    /// Values of the inputs
    ///
    /// Used for the constant mode (to replace variables in the main function with constants)
    values: Vec<FieldElement>,

    mode: FuzzerMode,
}

impl FuzzerProgramContext {
    /// Creates a new FuzzerProgramContext
    fn new(
        program_context_options: FunctionContextOptions,
        runtime: RuntimeType,
        instruction_blocks: Vec<InstructionBlock>,
        values: Vec<FieldElement>,
        mode: FuzzerMode,
    ) -> Self {
        let builder =
            FuzzerBuilder::new_by_runtime(runtime, program_context_options.simplifying_enabled);
        Self {
            builder,
            program_context_options,
            is_constant: false,
            function_information: BTreeMap::new(),
            stored_functions: Vec::new(),
            current_function_id: Id::new(0),
            instruction_blocks,
            is_main_initialized: false,
            values,
            mode,
        }
    }

    /// Creates a new FuzzerProgramContext where all inputs are constants
    fn new_constant_context(
        program_context_options: FunctionContextOptions,
        runtime: RuntimeType,
        instruction_blocks: Vec<InstructionBlock>,
        values: Vec<FieldElement>,
        mode: FuzzerMode,
    ) -> Self {
        let builder =
            FuzzerBuilder::new_by_runtime(runtime, program_context_options.simplifying_enabled);
        Self {
            builder,
            program_context_options,
            is_constant: true,
            function_information: BTreeMap::new(),
            stored_functions: Vec::new(),
            current_function_id: Id::new(0),
            instruction_blocks,
            is_main_initialized: false,
            values,
            mode,
        }
    }

    /// Stores function and its signature
    pub(crate) fn process_function(&mut self, function: FunctionData, types: Vec<Type>) {
        // leaving max_unrolled_size = 0 for now
        //
        let signature = FunctionInfo {
            input_types: types.clone(),
            return_type: function.return_type.clone(),
            max_unrolled_size: 0,
        };
        self.function_information.insert(self.current_function_id, signature);
        let stored_function = StoredFunction { id: self.current_function_id, function, types };
        self.stored_functions.push(stored_function);
        self.current_function_id = Id::new(self.current_function_id.to_u32() + 1);
    }

    /// Initializes unrolled sizes for all functions in the program
    ///
    /// Unrolled size is the number of instructions that will be executed in a loop
    ///
    /// It is calculated by multiplying the size of the instruction block by the number of iterations
    /// in the loop
    ///
    /// It is used to limit the number of instructions in the program
    fn initialize_unrolled_sizes(&mut self) {
        // design of the fuzzer allows to call functions only defined after the current one
        // go from the last function to the first one
        for function in self.stored_functions.iter_mut().rev() {
            let mut max_unrolled_size = 1;
            let mut cycle_sizes = Vec::new();
            let mut iterations_before = 1;
            for command in &function.function.commands {
                match command {
                    FuzzerFunctionCommand::InsertCycle { block_body_idx, start_iter, end_iter } => {
                        let instruction_block_size = self.instruction_blocks
                            [*block_body_idx % self.instruction_blocks.len()]
                        .instructions
                        .len();
                        let cycle_iterations_count = if end_iter > start_iter {
                            (end_iter - start_iter).saturating_add(1)
                        } else {
                            1
                        } as usize;
                        cycle_sizes.push(cycle_iterations_count);
                        iterations_before *= cycle_iterations_count;
                        max_unrolled_size += instruction_block_size * iterations_before;
                    }
                    FuzzerFunctionCommand::InsertSimpleInstructionBlock {
                        instruction_block_idx,
                    } => {
                        let instruction_block_size = self.instruction_blocks
                            [*instruction_block_idx % self.instruction_blocks.len()]
                        .instructions
                        .len();
                        max_unrolled_size += instruction_block_size * iterations_before;
                    }
                    FuzzerFunctionCommand::InsertJmpIfBlock { block_then_idx, block_else_idx } => {
                        let instruction_block_size_then = self.instruction_blocks
                            [*block_then_idx % self.instruction_blocks.len()]
                        .instructions
                        .len();
                        let instruction_block_size_else = self.instruction_blocks
                            [*block_else_idx % self.instruction_blocks.len()]
                        .instructions
                        .len();
                        max_unrolled_size += instruction_block_size_then * iterations_before
                            + instruction_block_size_else * iterations_before;
                    }
                    FuzzerFunctionCommand::InsertJmpBlock { block_idx } => {
                        let instruction_block_size = self.instruction_blocks
                            [*block_idx % self.instruction_blocks.len()]
                        .instructions
                        .len();
                        // InsertJmpBlock breaks the loop, so we need to divide by the size of the previous cycle
                        let previous_cycle_size = cycle_sizes.pop().unwrap_or(1);
                        iterations_before /= previous_cycle_size;
                        max_unrolled_size += instruction_block_size * iterations_before;
                    }
                    FuzzerFunctionCommand::SwitchToNextBlock => {
                        // SwitchToNextBlock breaks the loop
                        iterations_before /= cycle_sizes.pop().unwrap_or(1);
                    }
                    FuzzerFunctionCommand::InsertFunctionCall { function_idx, .. } => {
                        let defined_functions = self
                            .function_information
                            .clone()
                            .into_iter()
                            .filter(|(id, _)| id.to_u32() > function.id.to_u32())
                            .collect::<Vec<_>>();
                        if defined_functions.is_empty() {
                            continue;
                        }
                        let function_to_call = defined_functions
                            .get(*function_idx % defined_functions.len())
                            .unwrap()
                            .0;
                        let function_to_call_signature =
                            self.function_information.get(&function_to_call).unwrap();
                        let function_to_call_max_unrolled_size =
                            function_to_call_signature.max_unrolled_size;
                        if function_to_call_max_unrolled_size == 0 {
                            unreachable!("Encountered a function with no unrolled size");
                        }
                        max_unrolled_size += function_to_call_max_unrolled_size * iterations_before;
                    }
                }
            }
            let final_block_size = self.instruction_blocks
                [function.function.return_instruction_block_idx % self.instruction_blocks.len()]
            .instructions
            .len();
            max_unrolled_size += final_block_size;
            self.function_information.get_mut(&function.id).unwrap().max_unrolled_size =
                max_unrolled_size;
        }
    }

    /// Initializes information of all functions in the program
    fn initialize_function_info(&mut self) {
        self.initialize_unrolled_sizes();
    }

    /// Creates new ACIR and Brillig functions for each of stored functions and finalizes them
    pub(crate) fn finalize_program(&mut self) {
        self.initialize_function_info();
        for i in 0..self.stored_functions.len() {
            let stored_function = &self.stored_functions[i];
            // use only functions defined after this one to avoid recursion
            let defined_functions: BTreeMap<Id<Function>, FunctionInfo> = self
                .function_information
                .clone()
                .into_iter()
                .filter(|func_id| func_id.0.to_u32() > stored_function.id.to_u32())
                .collect();
            let mut function_context = if self.is_constant && !self.is_main_initialized {
                FuzzerFunctionContext::new_constant_context(
                    self.values
                        .iter()
                        .zip(stored_function.types.iter())
                        .map(|(value, type_)| (*value, type_.unwrap_numeric()))
                        .collect(),
                    &self.instruction_blocks,
                    self.program_context_options.clone(),
                    stored_function.function.return_type.clone(),
                    defined_functions,
                    &mut self.builder,
                )
            } else {
                FuzzerFunctionContext::new(
                    stored_function.types.to_vec(),
                    &self.instruction_blocks,
                    self.program_context_options.clone(),
                    stored_function.function.return_type.clone(),
                    defined_functions,
                    &mut self.builder,
                )
            };
            self.is_main_initialized = true;
            for command in &stored_function.function.commands {
                function_context.process_fuzzer_command(command);
            }
            function_context.finalize(stored_function.function.return_instruction_block_idx);
            // do not create a new function if it's last one
            if i != self.stored_functions.len() - 1 {
                let current_id = stored_function.id;
                let new_id = Id::<Function>::new(current_id.to_u32() + 1);
                self.builder.new_function(format!("f{}", new_id.to_u32()), new_id);
            }
        }
    }

    /// Returns program compiled from the builder
    pub(crate) fn get_program(self) -> Result<CompiledProgram, FuzzerBuilderError> {
        self.builder.compile(self.program_context_options.compile_options.clone())
    }

    pub(crate) fn get_mode(&self) -> FuzzerMode {
        self.mode.clone()
    }
}

/// Creates [`FuzzerProgramContext`] from [`FuzzerMode`] and [`RuntimeType`]
pub(crate) fn program_context_by_mode(
    mode: FuzzerMode,
    runtime: RuntimeType,
    instruction_blocks: Vec<InstructionBlock>,
    values: Vec<FieldElement>,
    options: FuzzerOptions,
) -> FuzzerProgramContext {
    match mode {
        FuzzerMode::Constant => FuzzerProgramContext::new_constant_context(
            FunctionContextOptions {
                idempotent_morphing_enabled: false,
                ..FunctionContextOptions::from(&options)
            },
            runtime,
            instruction_blocks,
            values,
            mode,
        ),
        FuzzerMode::NonConstant => FuzzerProgramContext::new(
            FunctionContextOptions {
                idempotent_morphing_enabled: false,
                ..FunctionContextOptions::from(&options)
            },
            runtime,
            instruction_blocks,
            values,
            mode,
        ),
        FuzzerMode::NonConstantWithIdempotentMorphing => FuzzerProgramContext::new(
            FunctionContextOptions {
                idempotent_morphing_enabled: true,
                ..FunctionContextOptions::from(&options)
            },
            runtime,
            instruction_blocks,
            values,
            mode,
        ),
        FuzzerMode::NonConstantWithoutDIE => {
            let mut options = options;
            options.compile_options.skip_ssa_pass =
                vec!["Dead Instruction Elimination".to_string()];
            FuzzerProgramContext::new(
                FunctionContextOptions {
                    idempotent_morphing_enabled: true,
                    ..FunctionContextOptions::from(&options)
                },
                runtime,
                instruction_blocks,
                values,
                mode,
            )
        }
        FuzzerMode::NonConstantWithoutSimplifying => {
            let mut options = options;
            options.simplifying_enabled = false;
            FuzzerProgramContext::new(
                FunctionContextOptions::from(&options),
                runtime,
                instruction_blocks,
                values,
                mode,
            )
        }
    }
}
