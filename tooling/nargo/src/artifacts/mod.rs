//! This module defines the structure of Nargo's different compilation artifacts.
//!
//! These artifacts are intended to remain independent of any applications being built on top of Noir.
//! Should any projects require/desire a different artifact format, it's expected that they will write a transformer
//! to generate them using these artifacts as a starting point.
use acvm::acir::circuit::Circuit;
use base64::Engine;
use serde::{
    de::Error as DeserializationError, ser::Error as SerializationError, Deserializer, Serializer,
};

pub mod contract;
pub mod debug;
pub mod program;

// TODO: move these down into ACVM.
fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut circuit_bytes: Vec<u8> = Vec::new();
    circuit.write(&mut circuit_bytes).map_err(S::Error::custom)?;
    let encoded_b64 = base64::engine::general_purpose::STANDARD.encode(circuit_bytes);
    s.serialize_str(&encoded_b64)
}

fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
where
    D: Deserializer<'de>,
{
    let bytecode_b64: String = serde::Deserialize::deserialize(deserializer)?;
    let circuit_bytes =
        base64::engine::general_purpose::STANDARD.decode(bytecode_b64).map_err(D::Error::custom)?;
    let circuit = Circuit::read(&*circuit_bytes).map_err(D::Error::custom)?;
    Ok(circuit)
}
