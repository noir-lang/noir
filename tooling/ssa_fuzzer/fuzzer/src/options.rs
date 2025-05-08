#[derive(Clone, Debug)]
pub struct SsaBlockOptions {
    pub idempotent_morphing_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct ContextOptions {
    pub idempotent_morphing_enabled: bool,
}

impl From<ContextOptions> for SsaBlockOptions {
    fn from(context_options: ContextOptions) -> Self {
        SsaBlockOptions { idempotent_morphing_enabled: context_options.idempotent_morphing_enabled }
    }
}

impl Into<ContextOptions> for SsaBlockOptions {
    fn into(self) -> ContextOptions {
        ContextOptions { idempotent_morphing_enabled: self.idempotent_morphing_enabled }
    }
}

pub struct FuzzerOptions {
    pub constant_execution_enabled: bool,
    pub idempotent_morphing_enabled: bool,
}

impl FuzzerOptions {
    pub fn new(constant_execution_enabled: bool, idempotent_morphing_enabled: bool) -> Self {
        Self { constant_execution_enabled, idempotent_morphing_enabled }
    }
}
