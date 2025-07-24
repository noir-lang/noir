use noirc_driver::CompileOptions;

/// Options for the instructions that can be used in the SSA blocks
#[derive(Clone, Copy, Debug)]
pub(crate) struct InstructionOptions {
    pub(crate) cast_enabled: bool,
    pub(crate) xor_enabled: bool,
    pub(crate) and_enabled: bool,
    pub(crate) or_enabled: bool,
    pub(crate) not_enabled: bool,
    pub(crate) add_enabled: bool,
    pub(crate) sub_enabled: bool,
    pub(crate) mul_enabled: bool,
    pub(crate) mod_enabled: bool,
    pub(crate) div_enabled: bool,
    pub(crate) shl_enabled: bool,
    pub(crate) shr_enabled: bool,
    pub(crate) eq_enabled: bool,
    pub(crate) lt_enabled: bool,
    pub(crate) load_enabled: bool,
    pub(crate) store_enabled: bool,
    pub(crate) alloc_enabled: bool,
}

impl Default for InstructionOptions {
    fn default() -> Self {
        Self {
            cast_enabled: true,
            xor_enabled: true,
            and_enabled: true,
            or_enabled: true,
            not_enabled: true,
            add_enabled: true,
            sub_enabled: true,
            mul_enabled: true,
            mod_enabled: true,
            div_enabled: true,
            shl_enabled: true,
            shr_enabled: true,
            eq_enabled: true,
            lt_enabled: true,
            load_enabled: true,
            store_enabled: true,
            alloc_enabled: true,
        }
    }
}

/// Options for the SSA block
#[derive(Clone, Debug)]
pub(crate) struct SsaBlockOptions {
    /// If false, we don't add constraints for idempotent morphing results
    pub(crate) constrain_idempotent_enabled: bool,
    /// Options for the instructions that can be used in the SSA block
    pub(crate) instruction_options: InstructionOptions,
}

/// Options of the program context
#[derive(Clone, Debug)]
pub(crate) struct FunctionContextOptions {
    /// If false, we don't add constraints for idempotent morphing results
    pub(crate) idempotent_morphing_enabled: bool,
    /// Options for the program compilation
    pub(crate) compile_options: CompileOptions,
    /// Maximum number of SSA blocks in the program
    pub(crate) max_ssa_blocks_num: usize,
    /// Maximum number of instructions inserted in the program
    pub(crate) max_instructions_num: usize,
    /// Options for the instructions that can be used in the SSA block
    pub(crate) instruction_options: InstructionOptions,
    /// Options for the fuzzer commands that can be used in the SSA block
    pub(crate) fuzzer_command_options: FuzzerCommandOptions,

    /// Maximum number of iterations in the program
    pub(crate) max_iterations_num: usize,
}

impl From<FunctionContextOptions> for SsaBlockOptions {
    fn from(context_options: FunctionContextOptions) -> Self {
        SsaBlockOptions {
            constrain_idempotent_enabled: context_options.idempotent_morphing_enabled,
            instruction_options: context_options.instruction_options,
        }
    }
}

/// Options for the fuzzer commands that can be used in the program context
#[derive(Clone, Copy, Debug)]
pub(crate) struct FuzzerCommandOptions {
    /// If false, we don't insert jmp_if
    pub(crate) jmp_if_enabled: bool,
    /// If false, we don't insert jmp command
    pub(crate) jmp_block_enabled: bool,
    /// If false, we don't switch to the next block
    pub(crate) switch_to_next_block_enabled: bool,
    pub(crate) loops_enabled: bool,
}

impl Default for FuzzerCommandOptions {
    fn default() -> Self {
        Self {
            jmp_if_enabled: true,
            jmp_block_enabled: true,
            switch_to_next_block_enabled: true,
            loops_enabled: true,
        }
    }
}

#[derive(Clone)]
pub(crate) struct FuzzerOptions {
    pub(crate) constrain_idempotent_morphing_enabled: bool,
    pub(crate) constant_execution_enabled: bool,
    pub(crate) compile_options: CompileOptions,
    pub(crate) max_ssa_blocks_num: usize,
    pub(crate) max_instructions_num: usize,
    pub(crate) max_iterations_num: usize,
    pub(crate) instruction_options: InstructionOptions,
    pub(crate) fuzzer_command_options: FuzzerCommandOptions,
}

impl Default for FuzzerOptions {
    fn default() -> Self {
        Self {
            constrain_idempotent_morphing_enabled: false,
            constant_execution_enabled: false,
            compile_options: CompileOptions { show_ssa: false, ..Default::default() },
            max_ssa_blocks_num: 100,
            max_instructions_num: 1000,
            max_iterations_num: 1000,
            instruction_options: InstructionOptions::default(),
            fuzzer_command_options: FuzzerCommandOptions::default(),
        }
    }
}

impl From<&FuzzerOptions> for FunctionContextOptions {
    fn from(options: &FuzzerOptions) -> FunctionContextOptions {
        FunctionContextOptions {
            idempotent_morphing_enabled: false,
            compile_options: options.compile_options.clone(),
            max_ssa_blocks_num: options.max_ssa_blocks_num,
            max_instructions_num: options.max_instructions_num,
            instruction_options: options.instruction_options,
            fuzzer_command_options: options.fuzzer_command_options,
            max_iterations_num: options.max_iterations_num,
        }
    }
}
