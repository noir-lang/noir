use acvm::acir::circuit::Circuit;
use noirc_abi::Abi;
use noirc_driver::ContractFunctionType;
use serde::{Deserialize, Serialize};

/// `PreprocessedContract` represents a Noir contract which has been preprocessed by a particular backend proving system.
///
/// This differs from a generic Noir contract artifact in that:
/// - The ACIR bytecode has had an optimization pass applied to tailor it for the backend.
/// - Proving and verification keys have been pregenerated based on this ACIR.
#[derive(Serialize, Deserialize)]
pub(crate) struct PreprocessedContract {
    /// The name of the contract.
    pub(crate) name: String,
    /// The identifier of the proving backend which this contract has been compiled for.
    pub(crate) backend: String,
    /// Each of the contract's functions are compiled into a separate program stored in this `Vec`.
    pub(crate) functions: Vec<PreprocessedContractFunction>,
}

/// Each function in the contract will be compiled as a separate noir program.
///
/// A contract function unlike a regular Noir program however can have additional properties.
/// One of these being a function type.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PreprocessedContractFunction {
    pub(crate) name: String,

    pub(crate) function_type: ContractFunctionType,

    pub(crate) abi: Abi,

    #[serde(
        serialize_with = "super::serialize_circuit",
        deserialize_with = "super::deserialize_circuit"
    )]
    pub(crate) bytecode: Circuit,

    pub(crate) proving_key: Vec<u8>,
    pub(crate) verification_key: Vec<u8>,
}
