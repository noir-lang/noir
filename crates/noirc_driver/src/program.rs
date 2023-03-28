use acvm::acir::circuit::Circuit;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompiledProgram {
    #[serde(serialize_with = "serialize_circuit", deserialize_with = "deserialize_circuit")]
    pub circuit: Circuit,
    pub abi: noirc_abi::Abi,
}

pub(crate) fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut circuit_bytes: Vec<u8> = Vec::new();
    circuit.write(&mut circuit_bytes).unwrap();

    circuit_bytes.serialize(s)
}

pub(crate) fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
where
    D: Deserializer<'de>,
{
    let circuit_bytes = Vec::<u8>::deserialize(deserializer)?;
    let circuit = Circuit::read(&*circuit_bytes).unwrap();
    Ok(circuit)
}
