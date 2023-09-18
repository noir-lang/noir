use std::{
    collections::{btree_map, BTreeMap},
    io::Read,
    ops::Index,
};

use acir_field::FieldElement;
use flate2::bufread::GzDecoder;
use flate2::bufread::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::native_types::Witness;

#[cfg(feature = "serialize-messagepack")]
#[derive(Debug, Error)]
enum SerializationError {
    #[error(transparent)]
    MsgpackEncode(#[from] rmp_serde::encode::Error),

    #[error(transparent)]
    MsgpackDecode(#[from] rmp_serde::decode::Error),

    #[error(transparent)]
    Deflate(#[from] std::io::Error),
}

#[cfg(not(feature = "serialize-messagepack"))]
#[derive(Debug, Error)]
enum SerializationError {
    #[error(transparent)]
    Deflate(#[from] std::io::Error),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct WitnessMapError(#[from] SerializationError);

/// A map from the witnesses in a constraint system to the field element values
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct WitnessMap(BTreeMap<Witness, FieldElement>);

impl WitnessMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub fn get(&self, witness: &Witness) -> Option<&FieldElement> {
        self.0.get(witness)
    }
    pub fn get_index(&self, index: u32) -> Option<&FieldElement> {
        self.0.get(&index.into())
    }
    pub fn contains_key(&self, key: &Witness) -> bool {
        self.0.contains_key(key)
    }
    pub fn insert(&mut self, key: Witness, value: FieldElement) -> Option<FieldElement> {
        self.0.insert(key, value)
    }
}

impl Index<&Witness> for WitnessMap {
    type Output = FieldElement;

    fn index(&self, index: &Witness) -> &Self::Output {
        &self.0[index]
    }
}

pub struct IntoIter(btree_map::IntoIter<Witness, FieldElement>);

impl Iterator for IntoIter {
    type Item = (Witness, FieldElement);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl IntoIterator for WitnessMap {
    type Item = (Witness, FieldElement);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

impl From<BTreeMap<Witness, FieldElement>> for WitnessMap {
    fn from(value: BTreeMap<Witness, FieldElement>) -> Self {
        Self(value)
    }
}

#[cfg(feature = "serialize-messagepack")]
impl TryFrom<WitnessMap> for Vec<u8> {
    type Error = WitnessMapError;

    fn try_from(val: WitnessMap) -> Result<Self, Self::Error> {
        let buf = rmp_serde::to_vec(&val).map_err(|err| WitnessMapError(err.into()))?;
        let mut deflater = flate2::write::DeflateEncoder::new(buf.as_slice(), Compression::best());
        let mut buf_c = Vec::new();
        deflater.read_to_end(&mut buf_c).map_err(|err| WitnessMapError(err.into()))?;
        Ok(buf_c)
    }
}

#[cfg(not(feature = "serialize-messagepack"))]
impl TryFrom<WitnessMap> for Vec<u8> {
    type Error = WitnessMapError;

    fn try_from(val: WitnessMap) -> Result<Self, Self::Error> {
        let buf = bincode::serialize(&val).unwrap();
        let mut deflater = GzEncoder::new(buf.as_slice(), Compression::best());
        let mut buf_c = Vec::new();
        deflater.read_to_end(&mut buf_c).map_err(|err| WitnessMapError(err.into()))?;
        Ok(buf_c)
    }
}

#[cfg(feature = "serialize-messagepack")]
impl TryFrom<&[u8]> for WitnessMap {
    type Error = WitnessMapError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut deflater = flate2::bufread::DeflateDecoder::new(bytes);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d).map_err(|err| WitnessMapError(err.into()))?;
        let witness_map =
            rmp_serde::from_slice(buf_d.as_slice()).map_err(|err| WitnessMapError(err.into()))?;
        Ok(Self(witness_map))
    }
}

#[cfg(not(feature = "serialize-messagepack"))]
impl TryFrom<&[u8]> for WitnessMap {
    type Error = WitnessMapError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut deflater = GzDecoder::new(bytes);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d).map_err(|err| WitnessMapError(err.into()))?;
        let witness_map = bincode::deserialize(buf_d.as_slice()).unwrap();
        Ok(Self(witness_map))
    }
}
