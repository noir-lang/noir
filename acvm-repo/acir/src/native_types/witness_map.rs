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

use crate::{SerializationFormat, native_types::Witness, serialization};

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
        let format = SerializationFormat::from_env()
            .map_err(|err| SerializationError::Serialize(std::io::Error::other(err)))?;
        self.serialize_with_format(format.unwrap_or_default())
    }

    /// Serialize and compress with a given format.
    pub fn serialize_with_format(
        &self,
        format: SerializationFormat,
    ) -> Result<Vec<u8>, WitnessMapError> {
        let buf = serialization::serialize_with_format(self, format)
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

#[cfg(test)]
mod tests {
    use super::*;
    use acir_field::FieldElement;

    #[test]
    fn test_round_trip_serialization() {
        // Create a witness map with several entries
        let mut original = WitnessMap::new();
        original.insert(Witness(0), FieldElement::from(42u128));
        original.insert(Witness(1), FieldElement::from(123u128));
        original.insert(Witness(5), FieldElement::from(999u128));
        original.insert(Witness(10), FieldElement::zero());
        original.insert(Witness(100), FieldElement::one());

        // Serialize
        let serialized = original.serialize().expect("Serialization should succeed");

        // Deserialize
        let deserialized =
            WitnessMap::deserialize(&serialized).expect("Deserialization should succeed");

        // Verify round trip
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_round_trip_empty_witness_map() {
        // Test with an empty witness map
        let original = WitnessMap::<FieldElement>::new();

        let serialized = original.serialize().expect("Serialization should succeed");
        let deserialized =
            WitnessMap::deserialize(&serialized).expect("Deserialization should succeed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_round_trip_single_entry() {
        // Test with a single entry
        let mut original = WitnessMap::new();
        original.insert(Witness(0), FieldElement::from(12345u128));

        let serialized = original.serialize().expect("Serialization should succeed");
        let deserialized =
            WitnessMap::deserialize(&serialized).expect("Deserialization should succeed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_round_trip_large_field_elements() {
        // Test with large field elements
        let mut original = WitnessMap::new();
        original.insert(Witness(0), FieldElement::from(u128::MAX));
        original.insert(Witness(1), -FieldElement::one());
        original.insert(Witness(2), FieldElement::from(u128::MAX / 2));

        let serialized = original.serialize().expect("Serialization should succeed");
        let deserialized =
            WitnessMap::deserialize(&serialized).expect("Deserialization should succeed");

        assert_eq!(original, deserialized);
    }
}
