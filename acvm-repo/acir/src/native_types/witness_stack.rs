use std::io::Read;

use acir_field::AcirField;
use flate2::Compression;
use flate2::bufread::GzDecoder;
use flate2::bufread::GzEncoder;
use noir_protobuf::ProtoCodec as _;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::proto::convert::ProtoSchema;

use super::WitnessMap;

#[derive(Debug, Error)]
enum SerializationError {
    #[error(transparent)]
    Deflate(#[from] std::io::Error),

    #[error(transparent)]
    BincodeError(#[from] bincode::Error),

    #[allow(dead_code)]
    #[error("error deserializing witness stack: {0}")]
    Deserialize(String),
}

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

impl<F: Serialize> WitnessStack<F> {
    pub(crate) fn bincode_serialize(&self) -> Result<Vec<u8>, WitnessStackError> {
        bincode::serialize(self).map_err(|e| WitnessStackError(e.into()))
    }
}

impl<F: for<'a> Deserialize<'a>> WitnessStack<F> {
    pub(crate) fn bincode_deserialize(buf: &[u8]) -> Result<Self, WitnessStackError> {
        bincode::deserialize(buf).map_err(|e| WitnessStackError(e.into()))
    }
}

#[allow(dead_code)]
impl<F: AcirField> WitnessStack<F> {
    pub(crate) fn proto_serialize(&self) -> Vec<u8> {
        ProtoSchema::<F>::serialize_to_vec(self)
    }

    pub(crate) fn proto_deserialize(buf: &[u8]) -> Result<Self, WitnessStackError> {
        ProtoSchema::<F>::deserialize_from_vec(buf)
            .map_err(|e| SerializationError::Deserialize(e.to_string()).into())
    }
}

impl<F: Serialize + AcirField> TryFrom<&WitnessStack<F>> for Vec<u8> {
    type Error = WitnessStackError;

    fn try_from(val: &WitnessStack<F>) -> Result<Self, Self::Error> {
        let buf = val.bincode_serialize()?;
        let mut deflater = GzEncoder::new(buf.as_slice(), Compression::best());
        let mut buf_c = Vec::new();
        deflater.read_to_end(&mut buf_c).map_err(|err| WitnessStackError(err.into()))?;
        Ok(buf_c)
    }
}

impl<F: Serialize + AcirField> TryFrom<WitnessStack<F>> for Vec<u8> {
    type Error = WitnessStackError;

    fn try_from(val: WitnessStack<F>) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl<F: for<'a> Deserialize<'a>> TryFrom<&[u8]> for WitnessStack<F> {
    type Error = WitnessStackError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut deflater = GzDecoder::new(bytes);
        let mut buf = Vec::new();
        deflater.read_to_end(&mut buf).map_err(|err| WitnessStackError(err.into()))?;
        let witness_stack = Self::bincode_deserialize(&buf)?;
        Ok(witness_stack)
    }
}
