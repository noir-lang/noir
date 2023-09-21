use crate::native_types::{Expression, Witness};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotientDirective {
    pub a: Expression,
    pub b: Expression,
    pub q: Witness,
    pub r: Witness,
    pub predicate: Option<Expression>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Directives do not apply any constraints.
/// You can think of them as opcodes that allow one to use non-determinism
/// In the future, this can be replaced with asm non-determinism blocks
pub enum Directive {
    //Performs euclidian division of a / b (as integers) and stores the quotient in q and the rest in r
    Quotient(QuotientDirective),

    //decomposition of a: a=\sum b[i]*radix^i where b is an array of witnesses < radix in little endian form
    ToLeRadix {
        a: Expression,
        b: Vec<Witness>,
        radix: u32,
    },

    // Sort directive, using a sorting network
    // This directive is used to generate the values of the control bits for the sorting network such that its outputs are properly sorted according to sort_by
    PermutationSort {
        inputs: Vec<Vec<Expression>>, // Array of tuples to sort
        tuple: u32, // tuple size; if 1 then inputs is a single array [a0,a1,..], if 2 then inputs=[(a0,b0),..] is [a0,b0,a1,b1,..], etc..
        bits: Vec<Witness>, // control bits of the network which permutes the inputs into its sorted version
        sort_by: Vec<u32>, // specify primary index to sort by, then the secondary,... For instance, if tuple is 2 and sort_by is [1,0], then a=[(a0,b0),..] is sorted by bi and then ai.
    },
}

impl Directive {
    pub fn name(&self) -> &str {
        match self {
            Directive::Quotient(_) => "quotient",
            Directive::ToLeRadix { .. } => "to_le_radix",
            Directive::PermutationSort { .. } => "permutation_sort",
        }
    }
}
