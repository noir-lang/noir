use std::{
    collections::{btree_map, BTreeMap},
    io::Read,
    ops::Index,
};

use flate2::bufread::GzDecoder;
use flate2::bufread::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::native_types::Witness;

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

impl<F: Serialize> TryFrom<WitnessMap<F>> for Vec<u8> {
    type Error = WitnessMapError;

    fn try_from(val: WitnessMap<F>) -> Result<Self, Self::Error> {
        let buf = bincode::serialize(&val).unwrap();
        let mut deflater = GzEncoder::new(buf.as_slice(), Compression::best());
        let mut buf_c = Vec::new();
        deflater.read_to_end(&mut buf_c).map_err(|err| WitnessMapError(err.into()))?;
        Ok(buf_c)
    }
}

impl<F: for<'a> Deserialize<'a>> TryFrom<&[u8]> for WitnessMap<F> {
    type Error = WitnessMapError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut deflater = GzDecoder::new(bytes);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d).map_err(|err| WitnessMapError(err.into()))?;
        let witness_map = bincode::deserialize(&buf_d).unwrap();
        Ok(Self(witness_map))
    }
}
