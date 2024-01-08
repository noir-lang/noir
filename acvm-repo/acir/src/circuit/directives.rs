use crate::native_types::{Expression, Witness};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Directives do not apply any constraints.
/// You can think of them as opcodes that allow one to use non-determinism
/// In the future, this can be replaced with asm non-determinism blocks
pub enum Directive {
    //decomposition of a: a=\sum b[i]*radix^i where b is an array of witnesses < radix in little endian form
    ToLeRadix { a: Expression, b: Vec<Witness>, radix: u32 },
}

impl Directive {
    pub fn get_outputs_vec(&self) -> Vec<Witness> {
        match self {
            Directive::ToLeRadix { b, .. } => b.to_owned(),
            Directive::PermutationSort { bits, .. } => bits.to_owned(),
        }
    }
}
