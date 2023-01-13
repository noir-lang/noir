use serde::{Deserialize, Serialize};

use crate::native_types::{Expression, Witness};
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
    Arithmetic(Expression),
    Range(Witness, u32),
    And(AndGate),
    Xor(XorGate),
    GadgetCall(GadgetCall),
    Directive(Directive),
}

impl Gate {
    pub fn name(&self) -> &str {
        match self {
            Gate::Arithmetic(_) => "arithmetic",
            Gate::Range(_, _) => "range",
            Gate::And(_) => "and",
            Gate::Xor(_) => "xor",
            Gate::Directive(Directive::Invert { .. }) => "invert",
            Gate::Directive(Directive::Quotient { .. }) => "quotient",
            Gate::Directive(Directive::ToRadix { .. }) => "to_radix",
            Gate::GadgetCall(g) => g.name.name(),
        }
    }
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, Gate::Arithmetic(_))
    }
    pub fn arithmetic(self) -> Expression {
        match self {
            Gate::Arithmetic(gate) => gate,
            _ => panic!("tried to convert a non arithmetic gate to an Expression struct"),
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
            Gate::Directive(Directive::Quotient(QuotientDirective { a, b, q, r, predicate })) => {
                if let Some(pred) = predicate {
                    write!(
                        f,
                        "Predicate euclidian division: {}*{} = {}*(x{}*{} + x{})",
                        pred,
                        a,
                        pred,
                        q.witness_index(),
                        b,
                        r.witness_index()
                    )
                } else {
                    write!(
                        f,
                        "Euclidian division: {} = x{}*{} + x{}",
                        a,
                        q.witness_index(),
                        b,
                        r.witness_index()
                    )
                }
            }
            Gate::And(g) => write!(f, "{:?}", g),
            Gate::Xor(g) => write!(f, "{:?}", g),
            Gate::GadgetCall(g) => write!(f, "{:?}", g),
            Gate::Directive(Directive::ToRadix { a, b, radix }) => {
                write!(
                    f,
                    "To Radix: {} into base {}; x{}...x{}",
                    a,
                    radix,
                    b.first().unwrap().witness_index(),
                    b.last().unwrap().witness_index(),
                )
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotientDirective {
    pub a: Expression,
    pub b: Expression,
    pub q: Witness,
    pub r: Witness,
    pub predicate: Option<Box<Expression>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Directives do not apply any constraints.
pub enum Directive {
    //Inverts the value of x and stores it in the result variable
    Invert { x: Witness, result: Witness },

    //Performs euclidian division of a / b (as integers) and stores the quotient in q and the rest in r
    Quotient(QuotientDirective),

    //decomposition of a: a=\sum b[i]*radix^i where b is an array of witnesses < radix
    ToRadix { a: Expression, b: Vec<Witness>, radix: u32 },
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
