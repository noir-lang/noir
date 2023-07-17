use acvm::acir::circuit::Circuit;

use base64::Engine;
use noirc_errors::debug_info::DebugInfo;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompiledProgram {
    #[serde(serialize_with = "serialize_circuit", deserialize_with = "deserialize_circuit")]
    pub circuit: Circuit,
    pub abi: noirc_abi::Abi,
    pub debug: DebugInfo,
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
