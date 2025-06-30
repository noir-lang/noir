use noirc_driver::CompileOptions;

/// Options for the instructions that can be used in the SSA blocks
#[derive(Clone, Copy, Debug)]
pub struct InstructionOptions {
    pub cast_enabled: bool,
    pub xor_enabled: bool,
    pub and_enabled: bool,
    pub or_enabled: bool,
    pub not_enabled: bool,
    pub add_enabled: bool,
    pub sub_enabled: bool,
    pub mul_enabled: bool,
    pub mod_enabled: bool,
    pub div_enabled: bool,
    pub shl_enabled: bool,
    pub shr_enabled: bool,
    pub eq_enabled: bool,
    pub lt_enabled: bool,
    pub load_enabled: bool,
    pub store_enabled: bool,
    pub alloc_enabled: bool,
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
pub struct SsaBlockOptions {
    /// If false, we don't add constraints for idempotent morphing results
    pub constrain_idempotent_enabled: bool,
    /// Options for the instructions that can be used in the SSA block
    pub instruction_options: InstructionOptions,
}

/// Options of the program context
#[derive(Clone, Debug)]
pub struct FunctionContextOptions {
    /// If false, we don't add constraints for idempotent morphing results
    pub idempotent_morphing_enabled: bool,
    /// Options for the program compilation
    pub compile_options: CompileOptions,
    /// Maximum number of SSA blocks in the program
    pub max_ssa_blocks_num: usize,
    /// Maximum number of instructions inserted in the program
    pub max_instructions_num: usize,
    /// Options for the instructions that can be used in the SSA block
    pub instruction_options: InstructionOptions,
    /// Options for the fuzzer commands that can be used in the SSA block
    pub fuzzer_command_options: FuzzerCommandOptions,

    /// Maximum number of iterations in the program
    pub max_iterations_num: usize,
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
pub struct FuzzerCommandOptions {
    /// If false, we don't insert jmp_if
    pub jmp_if_enabled: bool,
    /// If false, we don't insert jmp command
    pub jmp_block_enabled: bool,
    /// If false, we don't switch to the next block
    pub switch_to_next_block_enabled: bool,
}

impl Default for FuzzerCommandOptions {
    fn default() -> Self {
        Self { jmp_if_enabled: true, jmp_block_enabled: true, switch_to_next_block_enabled: true }
    }
}

pub struct FuzzerOptions {
    pub constrain_idempotent_morphing_enabled: bool,
    pub constant_execution_enabled: bool,
    pub compile_options: CompileOptions,
    pub max_ssa_blocks_num: usize,
    pub max_instructions_num: usize,
    pub max_iterations_num: usize,
    pub instruction_options: InstructionOptions,
    pub fuzzer_command_options: FuzzerCommandOptions,
}

impl Default for FuzzerOptions {
    fn default() -> Self {
        let mut compile_options = CompileOptions::default();
        compile_options.show_ssa = true;
        Self {
            constrain_idempotent_morphing_enabled: false,
            constant_execution_enabled: false,
            compile_options, //: CompileOptions::default(),
            max_ssa_blocks_num: 100,
            max_instructions_num: 1000,
            max_iterations_num: 1000,
            instruction_options: InstructionOptions::default(),
            fuzzer_command_options: FuzzerCommandOptions::default(),
        }
    }
}
