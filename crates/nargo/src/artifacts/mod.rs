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
    println!("CUR WIT IDX: {:?}", circuit.current_witness_index);
    let cs: ConstraintSystem =
        ConstraintSystem::try_from(circuit).expect("should have no malformed bb funcs");
    let circuit_bytes = cs.to_bytes();
    println!("{:?}", circuit_bytes.capacity());

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
    // panic!("Not supported");
    let b64_string = String::deserialize(deserializer)?;
    let compressed_bytes = base64::decode(b64_string).map_err(serde::de::Error::custom)?;

    let mut decoder = GzDecoder::new(&compressed_bytes[..]);
    let mut circuit_bytes = Vec::new();
    decoder.read_to_end(&mut circuit_bytes).unwrap();

    let circuit = Circuit::read(&*circuit_bytes).unwrap();
    Ok(circuit)
}
