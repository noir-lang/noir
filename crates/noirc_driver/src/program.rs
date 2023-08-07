use acvm::acir::circuit::Circuit;

use base64::Engine;
use noirc_abi::Abi;
use noirc_errors::debug_info::DebugInfo;
use noirc_frontend::ContractFunctionType;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Describes the types of smart contract functions that are allowed.
/// Unlike the similar enum in noirc_frontend, 'open' and 'unconstrained'
/// are mutually exclusive here. In the case a function is both, 'unconstrained'
/// takes precedence.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum FunctionType {
    /// This function will be executed in a private
    /// context.
    Secret,
    /// This function will be executed in a public
    /// context.
    Open,
    /// This function cannot constrain any values and can use nondeterministic features
    /// like arrays of a dynamic size.
    Unconstrained,
}

impl FunctionType {
    pub(super) fn new(kind: ContractFunctionType, is_unconstrained: bool) -> Self {
        match (kind, is_unconstrained) {
            (_, true) => Self::Unconstrained,
            (ContractFunctionType::Secret, false) => Self::Secret,
            (ContractFunctionType::Open, false) => Self::Open,
        }
    }
}

/// Each function in the contract will be compiled
/// as a separate noir program.
///
/// A contract function unlike a regular Noir program
/// however can have additional properties.
/// One of these being a function type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledFunction {
    pub name: String,

    pub function_type: FunctionType,

    pub is_internal: bool,

    pub abi: Abi,

    #[serde(serialize_with = "serialize_circuit", deserialize_with = "deserialize_circuit")]
    pub bytecode: Circuit,

    #[serde(skip)]
    pub debug: DebugInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompiledProgram {
    pub name: Option<String>,

    /// Each of the contract's functions are compiled into a separate `CompiledProgram`
    /// stored in this `Vector`.
    pub functions: Vec<CompiledFunction>,
}

pub(crate) fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut circuit_bytes: Vec<u8> = Vec::new();
    circuit.write(&mut circuit_bytes).unwrap();

    let encoded_b64 = base64::engine::general_purpose::STANDARD.encode(circuit_bytes);
    s.serialize_str(&encoded_b64)
}

pub(crate) fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
where
    D: Deserializer<'de>,
{
    let bytecode_b64: String = serde::Deserialize::deserialize(deserializer)?;
    let circuit_bytes = base64::engine::general_purpose::STANDARD.decode(bytecode_b64).unwrap();
    let circuit = Circuit::read(&*circuit_bytes).unwrap();
    Ok(circuit)
}
