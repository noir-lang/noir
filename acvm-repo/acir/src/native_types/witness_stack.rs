use std::io::Read;

use acir_field::AcirField;
use flate2::Compression;
use flate2::bufread::GzDecoder;
use flate2::bufread::GzEncoder;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::serialization;

use super::WitnessMap;

#[derive(Debug, Error)]
enum SerializationError {
    #[error("error compressing witness stack: {0}")]
    Compress(std::io::Error),

    #[error("error decompressing witness stack: {0}")]
    Decompress(std::io::Error),

    #[error("error serializing witness stack: {0}")]
    Serialize(std::io::Error),

    #[error("error deserializing witness stack: {0}")]
    Deserialize(std::io::Error),
}

/// Native error for serializing/deserializing a witness stack.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct WitnessStackError(#[from] SerializationError);

/// An ordered set of witness maps for separate circuits
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct WitnessStack<F> {
    stack: Vec<StackItem<F>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct StackItem<F> {
    /// Index into a [crate::circuit::Program] function list for which we have an associated witness
    pub index: u32,
    /// A full witness for the respective constraint system specified by the index
    pub witness: WitnessMap<F>,
}

impl<F> WitnessStack<F> {
    /// Append an element to the top of the stack
    pub fn push(&mut self, index: u32, witness: WitnessMap<F>) {
        self.stack.push(StackItem { index, witness });
    }

    /// Removes the top element from the stack and return its
    pub fn pop(&mut self) -> Option<StackItem<F>> {
        self.stack.pop()
    }

    /// Returns the top element of the stack, or `None` if it is empty
    pub fn peek(&self) -> Option<&StackItem<F>> {
        self.stack.last()
    }

    /// Returns the size of the stack
    pub fn length(&self) -> usize {
        self.stack.len()
    }
}

impl<F: AcirField + Serialize> WitnessStack<F> {
    /// Serialize and compress.
    pub fn serialize(&self) -> Result<Vec<u8>, WitnessStackError> {
        let buf = serialization::serialize_with_format_from_env(self)
            .map_err(|e| WitnessStackError(SerializationError::Serialize(e)))?;

        let mut deflater = GzEncoder::new(buf.as_slice(), Compression::best());
        let mut buf = Vec::new();
        deflater
            .read_to_end(&mut buf)
            .map_err(|e| WitnessStackError(SerializationError::Compress(e)))?;

        Ok(buf)
    }
}

impl<F: AcirField + for<'a> Deserialize<'a>> WitnessStack<F> {
    /// Decompress and deserialize.
    pub fn deserialize(buf: &[u8]) -> Result<Self, WitnessStackError> {
        let mut deflater = GzDecoder::new(buf);
        let mut buf = Vec::new();
        deflater
            .read_to_end(&mut buf)
            .map_err(|e| WitnessStackError(SerializationError::Decompress(e)))?;

        serialization::deserialize_any_format(&buf)
            .map_err(|e| WitnessStackError(SerializationError::Deserialize(e)))
    }
}

impl<F> From<WitnessMap<F>> for WitnessStack<F> {
    fn from(witness: WitnessMap<F>) -> Self {
        let stack = vec![StackItem { index: 0, witness }];
        Self { stack }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_types::Witness;
    use acir_field::FieldElement;

    #[test]
    fn test_round_trip_serialization() {
        // Create a witness stack with multiple stack items
        let mut stack = WitnessStack::default();

        // First function call with some witnesses
        let mut witness1 = WitnessMap::new();
        witness1.insert(Witness(0), FieldElement::from(42u128));
        witness1.insert(Witness(1), FieldElement::from(123u128));
        stack.push(0, witness1);

        // Second function call with different witnesses
        let mut witness2 = WitnessMap::new();
        witness2.insert(Witness(0), FieldElement::from(999u128));
        witness2.insert(Witness(5), FieldElement::zero());
        stack.push(1, witness2);

        // Third function call
        let mut witness3 = WitnessMap::new();
        witness3.insert(Witness(10), FieldElement::one());
        witness3.insert(Witness(20), FieldElement::from(u128::MAX));
        stack.push(2, witness3);

        // Serialize
        let serialized = stack.serialize().expect("Serialization should succeed");

        // Deserialize
        let deserialized =
            WitnessStack::deserialize(&serialized).expect("Deserialization should succeed");

        // Verify round trip
        assert_eq!(stack, deserialized);
    }

    #[test]
    fn test_round_trip_empty_witness_stack() {
        // Test with an empty witness stack
        let original = WitnessStack::<FieldElement>::default();

        let serialized = original.serialize().expect("Serialization should succeed");
        let deserialized =
            WitnessStack::deserialize(&serialized).expect("Deserialization should succeed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_round_trip_single_stack_item() {
        // Test with a single stack item
        let mut stack = WitnessStack::default();
        let mut witness = WitnessMap::new();
        witness.insert(Witness(0), FieldElement::from(12345u128));
        witness.insert(Witness(1), FieldElement::from(67890u128));
        stack.push(0, witness);

        let serialized = stack.serialize().expect("Serialization should succeed");
        let deserialized =
            WitnessStack::deserialize(&serialized).expect("Deserialization should succeed");

        assert_eq!(stack, deserialized);
    }

    #[test]
    fn test_round_trip_from_witness_map() {
        // Test conversion from WitnessMap and serialization
        let mut witness = WitnessMap::new();
        witness.insert(Witness(0), FieldElement::from(111u128));
        witness.insert(Witness(1), FieldElement::from(222u128));
        witness.insert(Witness(2), FieldElement::from(333u128));

        let original = WitnessStack::from(witness);

        let serialized = original.serialize().expect("Serialization should succeed");
        let deserialized =
            WitnessStack::deserialize(&serialized).expect("Deserialization should succeed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_round_trip_large_stack() {
        // Test with many stack items
        let mut stack = WitnessStack::default();

        for i in 0..10 {
            let mut witness = WitnessMap::new();
            witness.insert(Witness(i), FieldElement::from(u128::from(i) * 100));
            witness.insert(Witness(i + 100), FieldElement::from(u128::from(i) * 1000));
            stack.push(i, witness);
        }

        let serialized = stack.serialize().expect("Serialization should succeed");
        let deserialized =
            WitnessStack::deserialize(&serialized).expect("Deserialization should succeed");

        assert_eq!(stack, deserialized);
    }

    #[test]
    fn test_stack_operations() {
        // Test stack operations work correctly
        let mut stack = WitnessStack::default();

        let mut witness1 = WitnessMap::new();
        witness1.insert(Witness(0), FieldElement::from(1u128));
        stack.push(0, witness1.clone());

        let mut witness2 = WitnessMap::new();
        witness2.insert(Witness(1), FieldElement::from(2u128));
        stack.push(1, witness2.clone());

        assert_eq!(stack.length(), 2);
        assert_eq!(stack.peek().unwrap().index, 1);

        let popped = stack.pop().unwrap();
        assert_eq!(popped.index, 1);
        assert_eq!(popped.witness, witness2);

        assert_eq!(stack.length(), 1);
        assert_eq!(stack.peek().unwrap().index, 0);
        assert_eq!(stack.length(), 1);
    }
}
