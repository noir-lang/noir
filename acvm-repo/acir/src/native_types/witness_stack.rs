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

/// Native error for serializing/deserializating a witness stack.
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
