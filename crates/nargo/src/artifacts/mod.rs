//! This module defines the structure of Nargo's different compilation artifacts.
//!
//! These artifacts are intended to remain independent of any applications being built on top of Noir.
//! Should any projects require/desire a different artifact format, it's expected that they will write a transformer
//! to generate them using these artifacts as a starting point.

use acvm::acir::circuit::Circuit;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod contract;
pub mod program;

#[cfg(feature = "bb_js")]
use {
    acvm_backend_barretenberg::ConstraintSystem, base64, flate2::read::GzDecoder,
    flate2::write::GzEncoder, flate2::Compression, std::io::prelude::*,
};

// TODO: move these down into ACVM.
#[cfg(not(feature = "bb_js"))]
fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut circuit_bytes: Vec<u8> = Vec::new();
    circuit.write(&mut circuit_bytes).unwrap();

    circuit_bytes.serialize(s)
}

#[cfg(not(feature = "bb_js"))]
fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
where
    D: Deserializer<'de>,
{
    let circuit_bytes = Vec::<u8>::deserialize(deserializer)?;
    let circuit = Circuit::read(&*circuit_bytes).unwrap();
    Ok(circuit)
}

#[cfg(feature = "bb_js")]
fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let cs: ConstraintSystem =
        ConstraintSystem::try_from(circuit).expect("should have no malformed bb funcs");
    let circuit_bytes = cs.to_bytes();

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&circuit_bytes).unwrap();
    let compressed_bytes = encoder.finish().unwrap();

    let b64_string = base64::encode(compressed_bytes);
    s.serialize_str(&b64_string)
}

#[cfg(feature = "bb_js")]
fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
where
    D: Deserializer<'de>,
{
    // TODO(#1569): bb.js expects a serialized ConstraintSystem. For bb.js to fully interop with nargo, 
    // it will have to break out of the normal nargo deserialization process for a circuit.
    // Handle this elsewhere when reading in a program for proving/verifying with bb.js
    panic!("Not supported");
}
