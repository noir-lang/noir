//! This module defines the structure of Nargo's different compilation artifacts.
//!
//! These artifacts are intended to remain independent of any applications being built on top of Noir.
//! Should any projects require/desire a different artifact format, it's expected that they will write a transformer
//! to generate them using these artifacts as a starting point.

use acvm::acir::circuit::Circuit;
use base64;
use flate2::write::GzEncoder;
use serde::{Deserializer, Serializer};
// use flate2::read::GzDecoder;
use flate2::Compression;
use std::io::prelude::*;

use self::barretenberg_structures::ConstraintSystem;

mod barretenberg_structures;
pub mod contract;
pub mod program;

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

fn deserialize_circuit<'de, D>(_deserializer: D) -> Result<Circuit, D::Error>
where
    D: Deserializer<'de>,
{
    panic!("Not supported");
    // let b64_string = String::deserialize(deserializer)?;
    // let compressed_bytes = base64::decode(b64_string).map_err(serde::de::Error::custom)?;

    // let mut decoder = GzDecoder::new(&compressed_bytes[..]);
    // let mut circuit_bytes = Vec::new();
    // decoder.read_to_end(&mut circuit_bytes).unwrap();

    // let circuit = Circuit::read(&*circuit_bytes).unwrap();
    // Ok(circuit)
}

// fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     let mut circuit_bytes: Vec<u8> = Vec::new();
//     circuit.write(&mut circuit_bytes).unwrap();

//     let encoded_circuit_bytes = run_length_encode_zeroes(circuit_bytes);
//     let b64_string = base64::encode(encoded_circuit_bytes);
//     s.serialize_str(&b64_string)
// }

// fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let b64_string = String::deserialize(deserializer)?;
//     let encoded_circuit_bytes = base64::decode(b64_string).map_err(serde::de::Error::custom)?;
//     let circuit_bytes = run_length_decode_zeroes(encoded_circuit_bytes);
//     let circuit = Circuit::read(&*circuit_bytes).unwrap();
//     Ok(circuit)
// }

// TODO: move these down into ACVM.
// fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     let mut circuit_bytes: Vec<u8> = Vec::new();
//     circuit.write(&mut circuit_bytes).unwrap();

//     // circuit_bytes.serialize(s)
//     let hex_string = hex::encode(circuit_bytes);
//     s.serialize_str(&hex_string)
// }

// fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     // let circuit_bytes = Vec::<u8>::deserialize(deserializer)?;
//     let hex_string = String::deserialize(deserializer)?;
//     let circuit_bytes = hex::decode(hex_string).map_err(serde::de::Error::custom)?;
//     let circuit = Circuit::read(&*circuit_bytes).unwrap();
//     Ok(circuit)
// }

// fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     let mut circuit_bytes: Vec<u8> = Vec::new();
//     circuit.write(&mut circuit_bytes).unwrap();

//     let encoded_circuit_bytes = run_length_encode_zeroes(circuit_bytes);
//     let hex_string = hex::encode(encoded_circuit_bytes);

//     s.serialize_str(&hex_string)
// }

// fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let hex_string = String::deserialize(deserializer)?;
//     let encoded_circuit_bytes = hex::decode(hex_string).map_err(serde::de::Error::custom)?;
//     let circuit_bytes = run_length_decode_zeroes(encoded_circuit_bytes);

//     let circuit = Circuit::read(&*circuit_bytes).unwrap();
//     Ok(circuit)
// }

// fn run_length_encode_zeroes(input: Vec<u8>) -> Vec<u8> {
//     let mut output: Vec<u8> = Vec::new();
//     let mut count = 0;
//     for &byte in input.iter() {
//         if byte == 0 {
//             count += 1;
//             // Skip the loop, don't push anything yet.
//             continue;
//         }

//         // If we've encountered a sequence of zeroes, write it out.
//         if count > 0 {
//             output.push(0);
//             output.push(count);
//             count = 0;
//         }

//         output.push(byte);
//     }

//     // Edge case for if the last sequence is zeroes.
//     if count > 0 {
//         output.push(0);
//         output.push(count);
//     }

//     output
// }

// fn run_length_decode_zeroes(input: Vec<u8>) -> Vec<u8> {
//     let mut output: Vec<u8> = Vec::new();
//     let mut i = 0;
//     while i < input.len() {
//         if input[i] == 0 {
//             // We have a sequence of zeroes. The next byte is the count.
//             let count = input[i+1];
//             for _ in 0..count {
//                 output.push(0);
//             }
//             i += 2; // Skip the count.
//         } else {
//             output.push(input[i]);
//             i += 1;
//         }
//     }

//     output
// }
