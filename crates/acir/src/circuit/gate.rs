use serde::{Deserialize, Serialize};

use crate::native_types::{Arithmetic, Witness};
use crate::OPCODE;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AndGate {
    pub a: Witness,
    pub b: Witness,
    pub result: Witness,
    pub num_bits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct XorGate {
    pub a: Witness,
    pub b: Witness,
    pub result: Witness,
    pub num_bits: u32,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
// XXX: Gate does not capture what this is anymore. I think IR/OPCODE would be a better name
pub enum Gate {
    Arithmetic(Arithmetic),
    Range(Witness, u32),
    And(AndGate),
    Xor(XorGate),
    GadgetCall(GadgetCall),
    Directive(Directive),
}

impl Gate {
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, Gate::Arithmetic(_))
    }
    pub fn arithmetic(self) -> Arithmetic {
        match self {
            Gate::Arithmetic(gate) => gate,
            _ => panic!("tried to convert a non arithmetic gate to an Arithmetic struct"),
        }
    }
}

impl std::fmt::Debug for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gate::Arithmetic(a) => {
                for i in &a.mul_terms {
                    write!(f, "{:?}x{}*x{} + ", i.0, i.1.witness_index(), i.2.witness_index())?;
                }
                for i in &a.linear_combinations {
                    write!(f, "{:?}x{} + ", i.0, i.1.witness_index())?;
                }
                write!(f, "{:?} = 0", a.q_c)
            }
            Gate::Range(w, s) => {
                write!(f, "x{} is {} bits", w.witness_index(), s)
            }
            Gate::Directive(Directive::Invert { x, result: r }) => {
                write!(f, "x{}=1/x{}, or 0", r.witness_index(), x.witness_index())
            }
            Gate::Directive(Directive::Truncate { a, b, c: _c, bit_size }) => {
                write!(
                    f,
                    "Truncate: x{} is x{} truncated to {} bits",
                    b.witness_index(),
                    a.witness_index(),
                    bit_size
                )
            }
            Gate::Directive(Directive::Quotient { a, b, q, r }) => {
                write!(
                    f,
                    "Euclidian division: {} = x{}*{} + x{}",
                    a,
                    q.witness_index(),
                    b,
                    r.witness_index()
                )
            }
            Gate::Directive(Directive::Oddrange { a, b, r, bit_size }) => {
                write!(
                    f,
                    "Oddrange: x{} = x{}*2^{} + x{}",
                    a.witness_index(),
                    b.witness_index(),
                    bit_size,
                    r.witness_index()
                )
            }
            Gate::And(g) => write!(f, "{:?}", g),
            Gate::Xor(g) => write!(f, "{:?}", g),
            Gate::GadgetCall(g) => write!(f, "{:?}", g),
            Gate::Directive(Directive::Split { a, b, bit_size: _ }) => {
                write!(
                    f,
                    "Split: x{} into x{}...x{}",
                    a.witness_index(),
                    b.first().unwrap().witness_index(),
                    b.last().unwrap().witness_index(),
                )
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Directives do not apply any constraints.
pub enum Directive {
    //Inverts the value of x and stores it in the result variable
    Invert { x: Witness, result: Witness },

    //Performs euclidian division of a / b (as integers) and stores the quotient in q and the rest in r
    Quotient { a: Arithmetic, b: Arithmetic, q: Witness, r: Witness },

    //Reduces the value of a modulo 2^bit_size and stores the result in b: a= c*2^bit_size + b
    Truncate { a: Witness, b: Witness, c: Witness, bit_size: u32 },

    //Computes the highest bit b of a: a = b*2^(bit_size-1) + r, where a<2^bit_size, b is 0 or 1 and r<2^(bit_size-1)
    Oddrange { a: Witness, b: Witness, r: Witness, bit_size: u32 },

    //bit decomposition of a: a=\sum b[i]*2^i
    Split { a: Witness, b: Vec<Witness>, bit_size: u32 },
}

// Note: Some gadgets will not use all of the witness
// So we need to supply how many bits of the witness is needed
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GadgetInput {
    pub witness: Witness,
    pub num_bits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GadgetCall {
    pub name: OPCODE,
    pub inputs: Vec<GadgetInput>,
    pub outputs: Vec<Witness>,
}
