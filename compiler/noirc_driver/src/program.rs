use std::collections::BTreeMap;

use acvm::acir::circuit::Circuit;
use fm::FileId;

use base64::Engine;
use noirc_errors::debug_info::DebugInfo;
use serde::{de::Error as DeserializationError, ser::Error as SerializationError};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::debug::DebugFile;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompiledProgram {
    /// Hash of the [`Program`][noirc_frontend::monomorphization::ast::Program] from which this [`CompiledProgram`]
    /// was compiled.
    ///
    /// Used to short-circuit compilation in the case of the source code not changing since the last compilation.
    pub hash: u64,

    #[serde(serialize_with = "serialize_circuit", deserialize_with = "deserialize_circuit")]
    pub circuit: Circuit,
    pub abi: noirc_abi::Abi,
    pub debug: DebugInfo,
    pub file_map: BTreeMap<FileId, DebugFile>,
}

pub(crate) fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut circuit_bytes: Vec<u8> = Vec::new();
    circuit.write(&mut circuit_bytes).map_err(S::Error::custom)?;

    let encoded_b64 = base64::engine::general_purpose::STANDARD.encode(circuit_bytes);
    s.serialize_str(&encoded_b64)
}

pub(crate) fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
where
    D: Deserializer<'de>,
{
    let bytecode_b64: String = serde::Deserialize::deserialize(deserializer)?;
    let circuit_bytes =
        base64::engine::general_purpose::STANDARD.decode(bytecode_b64).map_err(D::Error::custom)?;
    let circuit = Circuit::read(&*circuit_bytes).map_err(D::Error::custom)?;
    Ok(circuit)
}
