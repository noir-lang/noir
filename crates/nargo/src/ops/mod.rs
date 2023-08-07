pub use self::codegen_verifier::codegen_verifier;
pub use self::execute::execute_function;
pub use self::execute::SolvedFunction;
pub use self::optimize::optimize_program;
pub use self::optimize::OptimizedProgram;
pub use self::preprocess::preprocess_function;
// TODO: Just expose the mods
pub use self::preprocess::PreprocessedProgram;
pub use self::prove::prove_execution;
pub use self::verify::verify_proof;

mod codegen_verifier;
mod execute;
mod foreign_calls;
mod optimize;
mod preprocess;
mod prove;
mod verify;
