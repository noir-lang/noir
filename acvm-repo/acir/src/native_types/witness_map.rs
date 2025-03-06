use std::{
    collections::{BTreeMap, btree_map},
    io::Read,
    ops::Index,
};

use acir_field::AcirField;
use flate2::Compression;
use flate2::bufread::GzDecoder;
use flate2::bufread::GzEncoder;
use noir_protobuf::ProtoCodec as _;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{native_types::Witness, proto::convert::ProtoSchema};

#[derive(Debug, Error)]
enum SerializationError {
    #[error(transparent)]
    Deflate(#[from] std::io::Error),

    #[allow(dead_code)]
    #[error("error deserializing witness map: {0}")]
    Deserialize(String),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct WitnessMapError(#[from] SerializationError);

/// A map from the witnesses in a constraint system to the field element values
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct WitnessMap<F>(BTreeMap<Witness, F>);

impl<F> WitnessMap<F> {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub fn get(&self, witness: &Witness) -> Option<&F> {
        self.0.get(witness)
    }
    pub fn get_index(&self, index: u32) -> Option<&F> {
        self.0.get(&index.into())
    }
    pub fn contains_key(&self, key: &Witness) -> bool {
        self.0.contains_key(key)
    }
    pub fn insert(&mut self, key: Witness, value: F) -> Option<F> {
        self.0.insert(key, value)
    }
}

impl<F> Index<&Witness> for WitnessMap<F> {
    type Output = F;

    fn index(&self, index: &Witness) -> &Self::Output {
        &self.0[index]
    }
}

pub struct IntoIter<F>(btree_map::IntoIter<Witness, F>);

impl<F> Iterator for IntoIter<F> {
    type Item = (Witness, F);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<F> IntoIterator for WitnessMap<F> {
    type Item = (Witness, F);
    type IntoIter = IntoIter<F>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

impl<F> From<BTreeMap<Witness, F>> for WitnessMap<F> {
    fn from(value: BTreeMap<Witness, F>) -> Self {
        Self(value)
    }
}

impl<F: Serialize + AcirField> WitnessMap<F> {
    pub(crate) fn bincode_serialize(&self) -> Result<Vec<u8>, WitnessMapError> {
        bincode::serialize(self).map_err(|e| SerializationError::Deserialize(e.to_string()).into())
    }
}

impl<F: AcirField + for<'a> Deserialize<'a>> WitnessMap<F> {
    pub(crate) fn bincode_deserialize(buf: &[u8]) -> Result<Self, WitnessMapError> {
        bincode::deserialize(buf).map_err(|e| SerializationError::Deserialize(e.to_string()).into())
    }
}

#[allow(dead_code)]
impl<F: AcirField> WitnessMap<F> {
    pub(crate) fn proto_serialize(&self) -> Vec<u8> {
        ProtoSchema::<F>::serialize_to_vec(self)
    }

    pub(crate) fn proto_deserialize(buf: &[u8]) -> Result<Self, WitnessMapError> {
        ProtoSchema::<F>::deserialize_from_vec(buf)
            .map_err(|e| SerializationError::Deserialize(e.to_string()).into())
    }
}

impl<F: Serialize + AcirField> TryFrom<WitnessMap<F>> for Vec<u8> {
    type Error = WitnessMapError;

    fn try_from(val: WitnessMap<F>) -> Result<Self, Self::Error> {
        let buf = val.bincode_serialize()?;
        let mut deflater = GzEncoder::new(buf.as_slice(), Compression::best());
        let mut buf = Vec::new();
        deflater.read_to_end(&mut buf).map_err(|err| WitnessMapError(err.into()))?;
        Ok(buf)
    }
}

impl<F: AcirField + for<'a> Deserialize<'a>> TryFrom<&[u8]> for WitnessMap<F> {
    type Error = WitnessMapError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut deflater = GzDecoder::new(bytes);
        let mut buf = Vec::new();
        deflater.read_to_end(&mut buf).map_err(|err| WitnessMapError(err.into()))?;
        let witness_map = Self::bincode_deserialize(&buf)?;
        Ok(witness_map)
    }
}
