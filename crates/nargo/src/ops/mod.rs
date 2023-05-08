pub use self::codegen_verifier::codegen_verifier;
pub use self::execute::execute_circuit;
pub use self::preprocess::{preprocess_contract, preprocess_program};
pub use self::prove::prove_execution;
pub use self::verify::verify_proof;
pub use self::recursion::{proof_as_fields, vk_as_fields};

mod codegen_verifier;
mod execute;
mod preprocess;
mod prove;
mod verify;
mod recursion;
