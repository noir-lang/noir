pub mod black_box_functions;
pub mod brillig;
pub mod directives;
pub mod opcodes;

use crate::native_types::Witness;
pub use opcodes::Opcode;
use thiserror::Error;

use std::{io::prelude::*, num::ParseIntError, str::FromStr};

use base64::Engine;
use flate2::Compression;
use serde::{de::Error as DeserializationError, Deserialize, Deserializer, Serialize, Serializer};

use std::collections::BTreeSet;

/// Specifies the maximum width of the expressions which will be constrained.
///
/// Unbounded Expressions are useful if you are eventually going to pass the ACIR
/// into a proving system which supports R1CS.
///
/// Bounded Expressions are useful if you are eventually going to pass the ACIR
/// into a proving system which supports PLONK, where arithmetic expressions have a
/// finite fan-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExpressionWidth {
    #[default]
    Unbounded,
    Bounded {
        width: usize,
    },
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Circuit {
    // current_witness_index is the highest witness index in the circuit. The next witness to be added to this circuit
    // will take on this value. (The value is cached here as an optimization.)
    pub current_witness_index: u32,
    pub opcodes: Vec<Opcode>,
    pub expression_width: ExpressionWidth,

    /// The set of private inputs to the circuit.
    pub private_parameters: BTreeSet<Witness>,
    // ACIR distinguishes between the public inputs which are provided externally or calculated within the circuit and returned.
    // The elements of these sets may not be mutually exclusive, i.e. a parameter may be returned from the circuit.
    // All public inputs (parameters and return values) must be provided to the verifier at verification time.
    /// The set of public inputs provided by the prover.
    pub public_parameters: PublicInputs,
    /// The set of public inputs calculated within the circuit.
    pub return_values: PublicInputs,
    /// Maps opcode locations to failed assertion messages.
    /// These messages are embedded in the circuit to provide useful feedback to users
    /// when a constraint in the circuit is not satisfied.
    ///
    // Note: This should be a BTreeMap, but serde-reflect is creating invalid
    // c++ code at the moment when it is, due to OpcodeLocation needing a comparison
    // implementation which is never generated.
    //
    // TODO: These are only used for constraints that are explicitly created during code generation (such as index out of bounds on slices)
    // TODO: We should move towards having all the checks being evaluated in the same manner
    // TODO: as runtime assert messages specified by the user. This will also be a breaking change as the `Circuit` structure will change.
    pub assert_messages: Vec<(OpcodeLocation, String)>,

    /// States whether the backend should use a SNARK recursion friendly prover.
    /// If implemented by a backend, this means that proofs generated with this circuit
    /// will be friendly for recursively verifying inside of another SNARK.
    pub recursive: bool,
}

impl Circuit {
    /// Returns the assert message associated with the provided [`OpcodeLocation`].
    /// Returns `None` if no such assert message exists.
    pub fn get_assert_message(&self, opcode_location: OpcodeLocation) -> Option<&str> {
        self.assert_messages
            .iter()
            .find(|(loc, _)| *loc == opcode_location)
            .map(|(_, message)| message.as_str())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
/// Opcodes are locatable so that callers can
/// map opcodes to debug information related to their context.
pub enum OpcodeLocation {
    Acir(usize),
    Brillig { acir_index: usize, brillig_index: usize },
}

impl std::fmt::Display for OpcodeLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpcodeLocation::Acir(index) => write!(f, "{index}"),
            OpcodeLocation::Brillig { acir_index, brillig_index } => {
                write!(f, "{acir_index}.{brillig_index}")
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum OpcodeLocationFromStrError {
    #[error("Invalid opcode location string: {0}")]
    InvalidOpcodeLocationString(String),
}

/// The implementation of display and FromStr allows serializing and deserializing a OpcodeLocation to a string.
/// This is useful when used as key in a map that has to be serialized to JSON/TOML, for example when mapping an opcode to its metadata.
impl FromStr for OpcodeLocation {
    type Err = OpcodeLocationFromStrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split('.').collect();

        if parts.is_empty() || parts.len() > 2 {
            return Err(OpcodeLocationFromStrError::InvalidOpcodeLocationString(s.to_string()));
        }

        fn parse_components(parts: Vec<&str>) -> Result<OpcodeLocation, ParseIntError> {
            match parts.len() {
                1 => {
                    let index = parts[0].parse()?;
                    Ok(OpcodeLocation::Acir(index))
                }
                2 => {
                    let acir_index = parts[0].parse()?;
                    let brillig_index = parts[1].parse()?;
                    Ok(OpcodeLocation::Brillig { acir_index, brillig_index })
                }
                _ => unreachable!("`OpcodeLocation` has too many components"),
            }
        }

        parse_components(parts)
            .map_err(|_| OpcodeLocationFromStrError::InvalidOpcodeLocationString(s.to_string()))
    }
}

impl Circuit {
    pub fn num_vars(&self) -> u32 {
        self.current_witness_index + 1
    }

    /// Returns all witnesses which are required to execute the circuit successfully.
    pub fn circuit_arguments(&self) -> BTreeSet<Witness> {
        self.private_parameters.union(&self.public_parameters.0).cloned().collect()
    }

    /// Returns all public inputs. This includes those provided as parameters to the circuit and those
    /// computed as return values.
    pub fn public_inputs(&self) -> PublicInputs {
        let public_inputs =
            self.public_parameters.0.union(&self.return_values.0).cloned().collect();
        PublicInputs(public_inputs)
    }

    fn write<W: std::io::Write>(&self, writer: W) -> std::io::Result<()> {
        let buf = bincode::serialize(self).unwrap();
        let mut encoder = flate2::write::GzEncoder::new(writer, Compression::default());
        encoder.write_all(&buf)?;
        encoder.finish()?;
        Ok(())
    }

    fn read<R: std::io::Read>(reader: R) -> std::io::Result<Self> {
        let mut gz_decoder = flate2::read::GzDecoder::new(reader);
        let mut buf_d = Vec::new();
        gz_decoder.read_to_end(&mut buf_d)?;
        bincode::deserialize(&buf_d)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))
    }

    pub fn serialize_circuit(circuit: &Circuit) -> Vec<u8> {
        let mut circuit_bytes: Vec<u8> = Vec::new();
        circuit.write(&mut circuit_bytes).expect("expected circuit to be serializable");
        circuit_bytes
    }

    pub fn deserialize_circuit(serialized_circuit: &[u8]) -> std::io::Result<Self> {
        Circuit::read(serialized_circuit)
    }

    // Serialize and base64 encode circuit
    pub fn serialize_circuit_base64<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let circuit_bytes = Circuit::serialize_circuit(circuit);
        let encoded_b64 = base64::engine::general_purpose::STANDARD.encode(circuit_bytes);
        s.serialize_str(&encoded_b64)
    }

    // Deserialize and base64 decode circuit
    pub fn deserialize_circuit_base64<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytecode_b64: String = serde::Deserialize::deserialize(deserializer)?;
        let circuit_bytes = base64::engine::general_purpose::STANDARD
            .decode(bytecode_b64)
            .map_err(D::Error::custom)?;
        let circuit = Self::deserialize_circuit(&circuit_bytes).map_err(D::Error::custom)?;
        Ok(circuit)
    }
}

impl std::fmt::Display for Circuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "current witness index : {}", self.current_witness_index)?;

        let write_public_inputs = |f: &mut std::fmt::Formatter<'_>,
                                   public_inputs: &PublicInputs|
         -> Result<(), std::fmt::Error> {
            write!(f, "[")?;
            let public_input_indices = public_inputs.indices();
            for (index, public_input) in public_input_indices.iter().enumerate() {
                write!(f, "{public_input}")?;
                if index != public_input_indices.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            writeln!(f, "]")
        };

        write!(f, "public parameters indices : ")?;
        write_public_inputs(f, &self.public_parameters)?;

        write!(f, "return value indices : ")?;
        write_public_inputs(f, &self.return_values)?;

        for opcode in &self.opcodes {
            writeln!(f, "{opcode}")?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Circuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PublicInputs(pub BTreeSet<Witness>);

impl PublicInputs {
    /// Returns the witness index of each public input
    pub fn indices(&self) -> Vec<u32> {
        self.0.iter().map(|witness| witness.witness_index()).collect()
    }

    pub fn contains(&self, index: usize) -> bool {
        self.0.contains(&Witness(index as u32))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Circuit, Compression, Opcode, PublicInputs,
    };
    use crate::{circuit::ExpressionWidth, native_types::Witness};
    use acir_field::FieldElement;

    fn and_opcode() -> Opcode {
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
            lhs: FunctionInput { witness: Witness(1), num_bits: 4 },
            rhs: FunctionInput { witness: Witness(2), num_bits: 4 },
            output: Witness(3),
        })
    }
    fn range_opcode() -> Opcode {
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness: Witness(1), num_bits: 8 },
        })
    }
    fn keccakf1600_opcode() -> Opcode {
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::Keccakf1600 {
            inputs: vec![
                FunctionInput { witness: Witness(1), num_bits: 64 },
                FunctionInput { witness: Witness(2), num_bits: 64 },
                FunctionInput { witness: Witness(3), num_bits: 64 },
                FunctionInput { witness: Witness(4), num_bits: 64 },
                FunctionInput { witness: Witness(5), num_bits: 64 },
                FunctionInput { witness: Witness(6), num_bits: 64 },
                FunctionInput { witness: Witness(7), num_bits: 64 },
                FunctionInput { witness: Witness(8), num_bits: 64 },
                FunctionInput { witness: Witness(9), num_bits: 64 },
                FunctionInput { witness: Witness(10), num_bits: 64 },
                FunctionInput { witness: Witness(11), num_bits: 64 },
                FunctionInput { witness: Witness(12), num_bits: 64 },
                FunctionInput { witness: Witness(13), num_bits: 64 },
                FunctionInput { witness: Witness(14), num_bits: 64 },
                FunctionInput { witness: Witness(15), num_bits: 64 },
                FunctionInput { witness: Witness(16), num_bits: 64 },
                FunctionInput { witness: Witness(17), num_bits: 64 },
                FunctionInput { witness: Witness(18), num_bits: 64 },
                FunctionInput { witness: Witness(19), num_bits: 64 },
                FunctionInput { witness: Witness(20), num_bits: 64 },
                FunctionInput { witness: Witness(21), num_bits: 64 },
                FunctionInput { witness: Witness(22), num_bits: 64 },
                FunctionInput { witness: Witness(23), num_bits: 64 },
                FunctionInput { witness: Witness(24), num_bits: 64 },
                FunctionInput { witness: Witness(25), num_bits: 64 },
            ],
            outputs: vec![
                Witness(26),
                Witness(27),
                Witness(28),
                Witness(29),
                Witness(30),
                Witness(31),
                Witness(32),
                Witness(33),
                Witness(34),
                Witness(35),
                Witness(36),
                Witness(37),
                Witness(38),
                Witness(39),
                Witness(40),
                Witness(41),
                Witness(42),
                Witness(43),
                Witness(44),
                Witness(45),
                Witness(46),
                Witness(47),
                Witness(48),
                Witness(49),
                Witness(50),
            ],
        })
    }

    #[test]
    fn serialization_roundtrip() {
        let circuit = Circuit {
            current_witness_index: 5,
            expression_width: ExpressionWidth::Unbounded,
            opcodes: vec![and_opcode(), range_opcode()],
            private_parameters: BTreeSet::new(),
            public_parameters: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(12)])),
            return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(4), Witness(12)])),
            assert_messages: Default::default(),
            recursive: false,
        };

        fn read_write(circuit: Circuit) -> (Circuit, Circuit) {
            let bytes = Circuit::serialize_circuit(&circuit);
            let got_circuit = Circuit::deserialize_circuit(&bytes).unwrap();
            (circuit, got_circuit)
        }

        let (circ, got_circ) = read_write(circuit);
        assert_eq!(circ, got_circ);
    }

    #[test]
    fn test_serialize() {
        let circuit = Circuit {
            current_witness_index: 0,
            expression_width: ExpressionWidth::Unbounded,
            opcodes: vec![
                Opcode::AssertZero(crate::native_types::Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![],
                    q_c: FieldElement::from(8u128),
                }),
                range_opcode(),
                and_opcode(),
                keccakf1600_opcode(),
            ],
            private_parameters: BTreeSet::new(),
            public_parameters: PublicInputs(BTreeSet::from_iter(vec![Witness(2)])),
            return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2)])),
            assert_messages: Default::default(),
            recursive: false,
        };

        let json = serde_json::to_string_pretty(&circuit).unwrap();

        let deserialized = serde_json::from_str(&json).unwrap();
        assert_eq!(circuit, deserialized);
    }

    #[test]
    fn does_not_panic_on_invalid_circuit() {
        use std::io::Write;

        let bad_circuit = "I'm not an ACIR circuit".as_bytes();

        // We expect to load circuits as compressed artifacts so we compress the junk circuit.
        let mut zipped_bad_circuit = Vec::new();
        let mut encoder =
            flate2::write::GzEncoder::new(&mut zipped_bad_circuit, Compression::default());
        encoder.write_all(bad_circuit).unwrap();
        encoder.finish().unwrap();

        let deserialization_result = Circuit::deserialize_circuit(&zipped_bad_circuit);
        assert!(deserialization_result.is_err());
    }
}
