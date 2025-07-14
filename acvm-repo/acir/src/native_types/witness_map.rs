use std::{
    collections::{BTreeMap, btree_map},
    io::Read,
    ops::Index,
};

use acir_field::AcirField;
use flate2::Compression;
use flate2::bufread::GzDecoder;
use flate2::bufread::GzEncoder;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{native_types::Witness, serialization};

#[derive(Debug, Error)]
enum SerializationError {
    #[error("error compressing witness map: {0}")]
    Compress(std::io::Error),

    #[error("error decompressing witness map: {0}")]
    Decompress(std::io::Error),

    #[error("error serializing witness map: {0}")]
    Serialize(std::io::Error),

    #[error("error deserializing witness map: {0}")]
    Deserialize(std::io::Error),
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

impl<F: AcirField + Serialize> WitnessMap<F> {
    /// Serialize and compress.
    pub fn serialize(&self) -> Result<Vec<u8>, WitnessMapError> {
        let buf = serialization::serialize_with_format_from_env(self)
            .map_err(|e| WitnessMapError(SerializationError::Serialize(e)))?;

        let mut deflater = GzEncoder::new(buf.as_slice(), Compression::best());
        let mut buf = Vec::new();
        deflater
            .read_to_end(&mut buf)
            .map_err(|e| WitnessMapError(SerializationError::Compress(e)))?;

        Ok(buf)
    }
}

impl<F: AcirField + for<'a> Deserialize<'a>> WitnessMap<F> {
    /// Decompress and deserialize.
    pub fn deserialize(buf: &[u8]) -> Result<Self, WitnessMapError> {
        let mut deflater = GzDecoder::new(buf);
        let mut buf = Vec::new();
        deflater
            .read_to_end(&mut buf)
            .map_err(|e| WitnessMapError(SerializationError::Decompress(e)))?;

        serialization::deserialize_any_format(&buf)
            .map_err(|e| WitnessMapError(SerializationError::Deserialize(e)))
    }
}
