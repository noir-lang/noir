use noirc_driver::CompileOptions;

// TODO pass them with FuzzerOptions
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

#[derive(Clone, Debug)]
pub struct SsaBlockOptions {
    pub constrain_idempotent_enabled: bool,
    pub instruction_options: InstructionOptions,
}

#[derive(Clone, Debug)]
pub struct ContextOptions {
    pub idempotent_morphing_enabled: bool,
    pub compile_options: CompileOptions,
    pub max_jumps_num: usize,
    pub max_instructions_num: usize,
    pub instruction_options: InstructionOptions,
    pub fuzzer_command_options: FuzzerCommandOptions,
}

impl From<ContextOptions> for SsaBlockOptions {
    fn from(context_options: ContextOptions) -> Self {
        SsaBlockOptions {
            constrain_idempotent_enabled: context_options.idempotent_morphing_enabled,
            instruction_options: context_options.instruction_options,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FuzzerCommandOptions {
    pub merge_instruction_blocks_enabled: bool,
    pub jmp_if_enabled: bool,
    pub jmp_block_enabled: bool,
    pub switch_to_next_block_enabled: bool,
}

impl Default for FuzzerCommandOptions {
    fn default() -> Self {
        Self {
            merge_instruction_blocks_enabled: true,
            jmp_if_enabled: true,
            jmp_block_enabled: true,
            switch_to_next_block_enabled: true,
        }
    }
}

pub struct FuzzerOptions {
    pub idempotent_morphing_enabled: bool,
    pub constant_execution_enabled: bool,
    pub compile_options: CompileOptions,
    pub max_jumps_num: usize,
    pub max_instructions_num: usize,
    pub instruction_options: InstructionOptions,
    pub fuzzer_command_options: FuzzerCommandOptions,
}

impl Default for FuzzerOptions {
    fn default() -> Self {
        Self {
            idempotent_morphing_enabled: false,
            constant_execution_enabled: false,
            compile_options: CompileOptions::default(),
            max_jumps_num: 30,
            max_instructions_num: 500,
            instruction_options: InstructionOptions::default(),
            fuzzer_command_options: FuzzerCommandOptions::default(),
        }
    }
}
