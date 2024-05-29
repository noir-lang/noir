pub mod black_box_functions;
pub mod brillig;
pub mod directives;
pub mod opcodes;

use crate::native_types::{Expression, Witness};
use acir_field::AcirField;
pub use opcodes::Opcode;
use thiserror::Error;

use std::{io::prelude::*, num::ParseIntError, str::FromStr};

use base64::Engine;
use flate2::Compression;
use serde::{de::Error as DeserializationError, Deserialize, Deserializer, Serialize, Serializer};

use std::collections::BTreeSet;

use self::{brillig::BrilligBytecode, opcodes::BlockId};

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

/// A program represented by multiple ACIR circuits. The execution trace of these
/// circuits is dictated by construction of the [crate::native_types::WitnessStack].
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Program<F> {
    pub functions: Vec<Circuit<F>>,
    pub unconstrained_functions: Vec<BrilligBytecode<F>>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Circuit<F> {
    // current_witness_index is the highest witness index in the circuit. The next witness to be added to this circuit
    // will take on this value. (The value is cached here as an optimization.)
    pub current_witness_index: u32,
    pub opcodes: Vec<Opcode<F>>,
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
    /// Maps opcode locations to failed assertion payloads.
    /// The data in the payload is embedded in the circuit to provide useful feedback to users
    /// when a constraint in the circuit is not satisfied.
    ///
    // Note: This should be a BTreeMap, but serde-reflect is creating invalid
    // c++ code at the moment when it is, due to OpcodeLocation needing a comparison
    // implementation which is never generated.
    pub assert_messages: Vec<(OpcodeLocation, AssertionPayload<F>)>,

    /// States whether the backend should use a SNARK recursion friendly prover.
    /// If implemented by a backend, this means that proofs generated with this circuit
    /// will be friendly for recursively verifying inside of another SNARK.
    pub recursive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpressionOrMemory<F> {
    Expression(Expression<F>),
    Memory(BlockId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssertionPayload<F> {
    StaticString(String),
    Dynamic(/* error_selector */ u64, Vec<ExpressionOrMemory<F>>),
}

#[derive(Debug, Copy, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct ErrorSelector(u64);

impl ErrorSelector {
    pub fn new(integer: u64) -> Self {
        ErrorSelector(integer)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Serialize for ErrorSelector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ErrorSelector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let as_u64 = s.parse().map_err(serde::de::Error::custom)?;
        Ok(ErrorSelector(as_u64))
    }
}

/// This selector indicates that the payload is a string.
/// This is used to parse any error with a string payload directly,
/// to avoid users having to parse the error externally to the ACVM.
/// Only non-string errors need to be parsed externally to the ACVM using the circuit ABI.
pub const STRING_ERROR_SELECTOR: ErrorSelector = ErrorSelector(0);

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct RawAssertionPayload<F> {
    pub selector: ErrorSelector,
    pub data: Vec<F>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ResolvedAssertionPayload<F> {
    String(String),
    Raw(RawAssertionPayload<F>),
}

#[derive(Debug, Copy, Clone)]
/// The opcode location for a call to a separate ACIR circuit
/// This includes the function index of the caller within a [program][Program]
/// and the index in the callers ACIR to the specific call opcode.
/// This is only resolved and set during circuit execution.
pub struct ResolvedOpcodeLocation {
    pub acir_function_index: usize,
    pub opcode_location: OpcodeLocation,
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

impl<F: AcirField> Circuit<F> {
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
}

impl<F: Serialize> Program<F> {
    fn write<W: std::io::Write>(&self, writer: W) -> std::io::Result<()> {
        let buf = bincode::serialize(self).unwrap();
        let mut encoder = flate2::write::GzEncoder::new(writer, Compression::default());
        encoder.write_all(&buf)?;
        encoder.finish()?;
        Ok(())
    }

    pub fn serialize_program(program: &Self) -> Vec<u8> {
        let mut program_bytes: Vec<u8> = Vec::new();
        program.write(&mut program_bytes).expect("expected circuit to be serializable");
        program_bytes
    }

    // Serialize and base64 encode program
    pub fn serialize_program_base64<S>(program: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let program_bytes = Program::serialize_program(program);
        let encoded_b64 = base64::engine::general_purpose::STANDARD.encode(program_bytes);
        s.serialize_str(&encoded_b64)
    }
}

impl<F: for<'a> Deserialize<'a>> Program<F> {
    fn read<R: std::io::Read>(reader: R) -> std::io::Result<Self> {
        let mut gz_decoder = flate2::read::GzDecoder::new(reader);
        let mut buf_d = Vec::new();
        gz_decoder.read_to_end(&mut buf_d)?;
        bincode::deserialize(&buf_d)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))
    }

    pub fn deserialize_program(serialized_circuit: &[u8]) -> std::io::Result<Self> {
        Program::read(serialized_circuit)
    }

    // Deserialize and base64 decode program
    pub fn deserialize_program_base64<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytecode_b64: String = serde::Deserialize::deserialize(deserializer)?;
        let program_bytes = base64::engine::general_purpose::STANDARD
            .decode(bytecode_b64)
            .map_err(D::Error::custom)?;
        let circuit = Self::deserialize_program(&program_bytes).map_err(D::Error::custom)?;
        Ok(circuit)
    }
}

impl<F: AcirField> std::fmt::Display for Circuit<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "current witness index : {}", self.current_witness_index)?;

        let write_witness_indices =
            |f: &mut std::fmt::Formatter<'_>, indices: &[u32]| -> Result<(), std::fmt::Error> {
                write!(f, "[")?;
                for (index, witness_index) in indices.iter().enumerate() {
                    write!(f, "{witness_index}")?;
                    if index != indices.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                writeln!(f, "]")
            };

        write!(f, "private parameters indices : ")?;
        write_witness_indices(
            f,
            &self
                .private_parameters
                .iter()
                .map(|witness| witness.witness_index())
                .collect::<Vec<_>>(),
        )?;

        write!(f, "public parameters indices : ")?;
        write_witness_indices(f, &self.public_parameters.indices())?;

        write!(f, "return value indices : ")?;
        write_witness_indices(f, &self.return_values.indices())?;

        for opcode in &self.opcodes {
            writeln!(f, "{opcode}")?;
        }
        Ok(())
    }
}

impl<F: AcirField> std::fmt::Debug for Circuit<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl<F: AcirField> std::fmt::Display for Program<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (func_index, function) in self.functions.iter().enumerate() {
            writeln!(f, "func {}", func_index)?;
            writeln!(f, "{}", function)?;
        }
        for (func_index, function) in self.unconstrained_functions.iter().enumerate() {
            writeln!(f, "unconstrained func {}", func_index)?;
            writeln!(f, "{:?}", function.bytecode)?;
        }
        Ok(())
    }
}

impl<F: AcirField> std::fmt::Debug for Program<F> {
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
    use crate::{
        circuit::{ExpressionWidth, Program},
        native_types::Witness,
    };
    use acir_field::{AcirField, FieldElement};
    use serde::{Deserialize, Serialize};

    fn and_opcode<F: AcirField>() -> Opcode<F> {
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
            lhs: FunctionInput { witness: Witness(1), num_bits: 4 },
            rhs: FunctionInput { witness: Witness(2), num_bits: 4 },
            output: Witness(3),
        })
    }
    fn range_opcode<F: AcirField>() -> Opcode<F> {
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness: Witness(1), num_bits: 8 },
        })
    }
    fn keccakf1600_opcode<F: AcirField>() -> Opcode<F> {
        let inputs: Box<[FunctionInput; 25]> = Box::new(std::array::from_fn(|i| FunctionInput {
            witness: Witness(i as u32 + 1),
            num_bits: 8,
        }));
        let outputs: Box<[Witness; 25]> = Box::new(std::array::from_fn(|i| Witness(i as u32 + 26)));

        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::Keccakf1600 { inputs, outputs })
    }
    fn schnorr_verify_opcode<F: AcirField>() -> Opcode<F> {
        let public_key_x =
            FunctionInput { witness: Witness(1), num_bits: FieldElement::max_num_bits() };
        let public_key_y =
            FunctionInput { witness: Witness(2), num_bits: FieldElement::max_num_bits() };
        let signature: Box<[FunctionInput; 64]> = Box::new(std::array::from_fn(|i| {
            FunctionInput { witness: Witness(i as u32 + 3), num_bits: 8 }
        }));
        let message: Vec<FunctionInput> = vec![FunctionInput { witness: Witness(67), num_bits: 8 }];
        let output = Witness(68);

        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::SchnorrVerify {
            public_key_x,
            public_key_y,
            signature,
            message,
            output,
        })
    }

    #[test]
    fn serialization_roundtrip() {
        let circuit = Circuit {
            current_witness_index: 5,
            expression_width: ExpressionWidth::Unbounded,
            opcodes: vec![and_opcode::<FieldElement>(), range_opcode(), schnorr_verify_opcode()],
            private_parameters: BTreeSet::new(),
            public_parameters: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(12)])),
            return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(4), Witness(12)])),
            assert_messages: Default::default(),
            recursive: false,
        };
        let program = Program { functions: vec![circuit], unconstrained_functions: Vec::new() };

        fn read_write<F: AcirField + Serialize + for<'a> Deserialize<'a>>(
            program: Program<F>,
        ) -> (Program<F>, Program<F>) {
            let bytes = Program::serialize_program(&program);
            let got_program = Program::deserialize_program(&bytes).unwrap();
            (program, got_program)
        }

        let (circ, got_circ) = read_write(program);
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
                schnorr_verify_opcode(),
            ],
            private_parameters: BTreeSet::new(),
            public_parameters: PublicInputs(BTreeSet::from_iter(vec![Witness(2)])),
            return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2)])),
            assert_messages: Default::default(),
            recursive: false,
        };
        let program = Program { functions: vec![circuit], unconstrained_functions: Vec::new() };

        let json = serde_json::to_string_pretty(&program).unwrap();

        let deserialized = serde_json::from_str(&json).unwrap();
        assert_eq!(program, deserialized);
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

        let deserialization_result: Result<Program<FieldElement>, _> =
            Program::deserialize_program(&zipped_bad_circuit);
        assert!(deserialization_result.is_err());
    }
}
