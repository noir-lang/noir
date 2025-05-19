use noirc_driver::CompileOptions;

// TODO pass them with FuzzerOptions
#[derive(Clone, Debug)]
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
            cast_enabled: false,
            xor_enabled: true,
            and_enabled: true,
            or_enabled: true,
            not_enabled: true,
            add_enabled: true,
            sub_enabled: true,
            mul_enabled: true,
            mod_enabled: false,
            div_enabled: true,
            shl_enabled: false,
            shr_enabled: false,
            eq_enabled: false,
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
}

impl From<ContextOptions> for SsaBlockOptions {
    fn from(context_options: ContextOptions) -> Self {
        SsaBlockOptions {
            constrain_idempotent_enabled: context_options.idempotent_morphing_enabled,
            instruction_options: InstructionOptions::default(),
        }
    }
}

pub struct FuzzerOptions {
    pub idempotent_morphing_enabled: bool,
    pub constant_execution_enabled: bool,
    pub compile_options: CompileOptions,
    pub max_jumps_num: usize,
    pub max_instructions_num: usize,
}

impl Default for FuzzerOptions {
    fn default() -> Self {
        let mut compile_options = CompileOptions::default();
        compile_options.show_ssa = true;
        Self {
            idempotent_morphing_enabled: false,
            constant_execution_enabled: false,
            compile_options,
            max_jumps_num: 30,
            max_instructions_num: 500,
        }
    }
}
