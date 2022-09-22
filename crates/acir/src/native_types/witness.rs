use std::io::Read;

use flate2::{
    bufread::{DeflateDecoder, DeflateEncoder},
    Compression,
};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub struct Witness(pub u32);

impl Witness {
    pub fn new(witness_index: u32) -> Witness {
        Witness(witness_index)
    }
    pub fn witness_index(&self) -> u32 {
        self.0
    }
    pub fn as_usize(&self) -> usize {
        // This is safe as long as the architecture is 32bits minimum
        self.0 as usize
    }

    pub const fn can_defer_constraint(&self) -> bool {
        true
    }

    pub fn to_unknown(self) -> UnknownWitness {
        UnknownWitness(self.0)
    }

    pub fn to_bytes(
        witnesses: &std::collections::BTreeMap<Witness, noir_field::FieldElement>,
    ) -> Vec<u8> {
        let buf = rmp_serde::to_vec(witnesses).unwrap();
        let mut deflater = DeflateEncoder::new(buf.as_slice(), Compression::best());
        let mut buf_c = Vec::new();
        deflater.read_to_end(&mut buf_c).unwrap();
        buf_c
    }

    pub fn from_bytes(
        bytes: &[u8],
    ) -> std::collections::BTreeMap<Witness, noir_field::FieldElement> {
        let mut deflater = DeflateDecoder::new(bytes);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d).unwrap();
        rmp_serde::from_slice(buf_d.as_slice()).unwrap()
    }
}

// This is a witness that is unknown relative to the rest of the witnesses in the arithmetic gate
// We use this, so that they are pushed to the beginning of the array
//
// When they are pushed to the beginning of the array, they are less likely to be used in an intermediate gate
// by the optimiser, which would mean two unknowns in an equation.
// See Issue #109
pub struct UnknownWitness(pub u32);

impl UnknownWitness {
    pub fn as_witness(&self) -> Witness {
        Witness(self.0)
    }
}
