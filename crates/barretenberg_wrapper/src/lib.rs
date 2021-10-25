extern crate hex;

mod bindings;
pub mod blake2s;
pub mod composer;
pub mod pedersen;
pub mod pippenger;
pub mod schnorr;

//pub use blake2s::hash_to_field;

#[macro_use]
extern crate slice_as_array;

#[cfg(test)]
mod tests {
    //    use super::*;
}
