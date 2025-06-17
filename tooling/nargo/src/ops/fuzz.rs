use alloy_primitives::U256;
use noirc_abi::Abi;
use noirc_driver::CompileOptions;
use noirc_errors::CustomDiagnostic;
use noirc_frontend::hir::{Context, def_map::FuzzingHarness};

/// Configuration for fuzzing loop execution
pub struct FuzzExecutionConfig {
    /// Number of threads to use for fuzzing
    pub num_threads: usize,
    /// How long to run the fuzzing loop
    pub cases_or_max_total_time: Option<u64>,
    /// Corpus folder to load inputs from and save new inputs to
    pub corpus_folder: Option<String>,
    /// Corpus folder minimization
    pub corpus_minimization_folder: Option<String>,
    /// Where to save the input that's causing failure
    pub failure_folder: Option<String>,
}

/// Configuration for where to persist fuzzing artefacts
pub struct FuzzFolderConfig {
    /// Corpus folder to load inputs from and save new inputs to
    pub corpus_folder: Option<String>,
    /// Corpus folder minimization
    pub corpus_minimization_folder: Option<String>,
    /// Where to save the input that's causing failure
    pub failure_folder: Option<String>,
}

impl From<FuzzExecutionConfig> for FuzzFolderConfig {
    fn from(config: FuzzExecutionConfig) -> Self {
        Self {
            corpus_folder: config.corpus_folder,
            corpus_minimization_folder: config.corpus_minimization_folder,
            failure_folder: config.failure_folder,
        }
    }
}

/// Stub implementation - fuzzing requires ACVM backend
pub fn run_fuzzing_harness(
    _context: &Context,
    _fn_name: String,
    _fuzzing_harness: FuzzingHarness,
    _fuzzing_config: FuzzExecutionConfig,
    _compile_options: &CompileOptions,
) -> Result<Option<(Vec<CustomDiagnostic>, String)>, CustomDiagnostic> {
    Err(CustomDiagnostic::simple_error(
        "Fuzzing is not available in Sensei (requires ZK backend)".to_string(),
        "Fuzzing functionality has been removed with ACVM".to_string(),
        noirc_errors::Location::dummy(),
    ))
}