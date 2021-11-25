pub mod blake2s;
pub mod composer;
mod crs;
pub mod pedersen;
mod pippenger;
pub mod scalar_mul;
pub mod schnorr;

use noir_field::FieldElement;
use std::convert::TryInto;

pub struct Barretenberg {}

// XXX: It may be better to use this global mutex, since we do not need to
// keep state around. However, for this we need to make sure
// that mem_free is being called at appropriate times
use once_cell::sync::Lazy;
use std::sync::Mutex;
pub static BARRETENBERG: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub fn field_to_array(f: &FieldElement) -> [u8; 32] {
    let v = f.to_bytes();
    let result: [u8; 32] = v.try_into().unwrap_or_else(|v: Vec<u8>| {
        panic!("Expected a Vec of length {} but it was {}", 32, v.len())
    });
    result
}

impl Default for Barretenberg {
    fn default() -> Self {
        Self::new()
    }
}

impl Barretenberg {
    pub fn new() -> Barretenberg {
        Barretenberg {}
    }
}

#[derive(Clone)]
pub struct Env {}
