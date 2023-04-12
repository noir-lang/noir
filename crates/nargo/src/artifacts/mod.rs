//! This module defines the structure of Nargo's different compilation artifacts.
//!
//! These artifacts are intended to remain independent of any applications being built on top of Noir.
//! Should any projects require/desire a different artifact format, it's expected that they will write a transformer
//! to generate them using these artifacts as a starting point.

use acvm::acir::circuit::Circuit;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod contract;
pub mod program;

// TODO: move these down into ACVM.
fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut circuit_bytes: Vec<u8> = Vec::new();
    circuit.write(&mut circuit_bytes).unwrap();

    circuit_bytes.serialize(s)
}

fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
where
    D: Deserializer<'de>,
{
    let circuit_bytes = Vec::<u8>::deserialize(deserializer)?;
    let circuit = Circuit::read(&*circuit_bytes).unwrap();
    Ok(circuit)
}
