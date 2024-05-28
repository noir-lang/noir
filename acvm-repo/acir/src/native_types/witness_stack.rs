use std::io::Read;

use flate2::bufread::GzDecoder;
use flate2::bufread::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::WitnessMap;

#[derive(Debug, Error)]
enum SerializationError {
    #[error(transparent)]
    Deflate(#[from] std::io::Error),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct WitnessStackError(#[from] SerializationError);

/// An ordered set of witness maps for separate circuits
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct WitnessStack<F> {
    stack: Vec<StackItem<F>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct StackItem<F> {
    /// Index into a [crate::circuit::Program] function list for which we have an associated witness
    pub index: u32,
    /// A full witness for the respective constraint system specified by the index
    pub witness: WitnessMap<F>,
}

impl<F> WitnessStack<F> {
    pub fn push(&mut self, index: u32, witness: WitnessMap<F>) {
        self.stack.push(StackItem { index, witness });
    }

    pub fn pop(&mut self) -> Option<StackItem<F>> {
        self.stack.pop()
    }

    pub fn peek(&self) -> Option<&StackItem<F>> {
        self.stack.last()
    }

    pub fn length(&self) -> usize {
        self.stack.len()
    }
}

impl<F> From<WitnessMap<F>> for WitnessStack<F> {
    fn from(witness: WitnessMap<F>) -> Self {
        let stack = vec![StackItem { index: 0, witness }];
        Self { stack }
    }
}

impl<F: Serialize> TryFrom<WitnessStack<F>> for Vec<u8> {
    type Error = WitnessStackError;

    fn try_from(val: WitnessStack<F>) -> Result<Self, Self::Error> {
        let buf = bincode::serialize(&val).unwrap();
        let mut deflater = GzEncoder::new(buf.as_slice(), Compression::best());
        let mut buf_c = Vec::new();
        deflater.read_to_end(&mut buf_c).map_err(|err| WitnessStackError(err.into()))?;
        Ok(buf_c)
    }
}

impl<F: for<'a> Deserialize<'a>> TryFrom<&[u8]> for WitnessStack<F> {
    type Error = WitnessStackError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut deflater = GzDecoder::new(bytes);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d).map_err(|err| WitnessStackError(err.into()))?;
        let witness_stack = bincode::deserialize(&buf_d).unwrap();
        Ok(witness_stack)
    }
}
