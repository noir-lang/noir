mod array;
mod integer;

pub use array::Array;
pub use integer::Integer;

use acvm::acir::circuit::gate::Gate;
use acvm::acir::native_types::{Arithmetic, Linear, Witness};
use noir_field::FieldElement;

use crate::Evaluator;

use super::errors::RuntimeErrorKind;

#[derive(Clone, Debug)]
pub enum Object {
    Null,
    Integer(Integer),
    Array(Array),
    Arithmetic(Arithmetic),
    Constants(FieldElement),
    Linear(Linear), // These will be selector * witness(var_name) + selector // Note that this is not a gate e.g `5x+6` does not apply a gate
}

impl Object {
    pub fn r#type(&self) -> &'static str {
        match self {
            Object::Integer(_) | Object::Arithmetic(_) | Object::Linear(_) => "witness",
            Object::Array(_) => "collection",
            Object::Constants(_) => "constant",
            Object::Null => "()",
        }
    }

    pub fn constrain_zero(&self, evaluator: &mut Evaluator) {
        match self {
            Object::Null => unreachable!(),
            Object::Constants(_) => unreachable!("cannot constrain a constant to be zero"),
            Object::Integer(integer) => integer.constrain_zero(evaluator),
            Object::Array(arr) => arr.constrain_zero(evaluator),
            Object::Arithmetic(arith) => evaluator.gates.push(Gate::Arithmetic(arith.clone())),
            Object::Linear(linear) => evaluator.gates.push(Gate::Arithmetic((*linear).into())),
        }
    }
    pub fn negate(self) -> Self {
        match self {
            Object::Null => {
                unreachable!("ice: before calling negate, you should check if the object is null")
            }
            Object::Integer(integer) => {
                // When negating the integer, we do not need
                // to maintain the range invariant.
                //
                // This behavior is not uniform. If you use the subtract method on Integer
                // it will apply a range constraint.
                let linear = -&Linear::from_witness(integer.witness);
                Object::Linear(linear)
            }
            Object::Array(arr) => {
                let negated_contents: Vec<_> = arr
                    .contents
                    .into_iter()
                    .map(|element| element.negate())
                    .collect();

                Object::Array(Array {
                    contents: negated_contents,
                    length: arr.length,
                })
            }
            Object::Arithmetic(arith) => Object::Arithmetic(-&arith),
            Object::Constants(constant) => Object::Constants(-constant),
            Object::Linear(linear) => Object::Linear(-&linear),
        }
    }
    // Converts a Object into an arithmetic object
    pub fn to_arithmetic(&self) -> Option<Arithmetic> {
        match self {
            Object::Null => None,
            Object::Integer(integer) => Some(Linear::from_witness(integer.witness).into()),
            Object::Array(_) => None,
            Object::Arithmetic(arith) => Some(arith.clone()),
            Object::Constants(constant) => Some(constant.into()),
            Object::Linear(linear) => Some(linear.into()),
        }
    }
    pub fn is_gate(&self) -> bool {
        matches!(self, Object::Arithmetic(_))
    }
    pub fn constant(&self) -> Result<FieldElement, RuntimeErrorKind> {
        match self {
            Object::Constants(x) => Ok(*x),
            _ => Err(RuntimeErrorKind::expected_type("constant", self.r#type())),
        }
    }
    pub fn is_constant(&self) -> bool {
        matches!(self, Object::Constants(_))
    }

    pub fn arithmetic(&self) -> Option<&Arithmetic> {
        match self {
            Object::Arithmetic(x) => Some(x),
            _ => None,
        }
    }
    pub fn extract_private_witness(self) -> Option<Arithmetic> {
        match self {
            Object::Arithmetic(x) => Some(x),
            Object::Linear(x) => Some(x.into()),
            Object::Integer(x) => Some((&x.witness).into()),
            Object::Array(_) => None,
            Object::Constants(_) => None,
            Object::Null => None,
        }
    }

    pub fn can_defer_constraint(&self) -> bool {
        match self {
            Object::Arithmetic(x) => x.can_defer_constraint(),
            Object::Linear(x) => x.can_defer_constraint(),
            Object::Integer(x) => x.witness.can_defer_constraint(),
            Object::Array(_) => false,
            Object::Constants(_) => false,
            Object::Null => false,
        }
    }

    pub fn witness(&self) -> Option<Witness> {
        if !self.is_unit_witness() {
            return None;
        };
        match self {
            Object::Linear(linear) => {
                assert!(linear.mul_scale == FieldElement::one());
                assert!(linear.add_scale == FieldElement::zero());
                Some(linear.witness)
            }
            _ => None,
        }
    }
    pub fn linear(&self) -> Option<Linear> {
        match self {
            Object::Linear(linear) => Some(*linear),
            Object::Integer(integer) => Some(Linear::from(integer.witness)),
            _ => None,
        }
    }
    pub fn integer(&self) -> Option<Integer> {
        match self {
            Object::Integer(integer) => Some(*integer),
            _ => None,
        }
    }
    pub fn array(&self) -> Option<Array> {
        match self {
            Object::Array(arr) => Some(arr.clone()),
            _ => None,
        }
    }
    // Returns true if the linear Object is a regular witness that has not been scaled
    pub fn is_unit_witness(&self) -> bool {
        match self {
            Object::Linear(linear) => linear.is_unit(),
            _ => false,
        }
    }
    // Returns true if the Object is linear
    pub fn is_linear(&self) -> bool {
        matches!(
            self,
            Object::Linear(_) | Object::Constants(_) | Object::Integer(_)
        )
    }
    pub fn from_witness(witness: Witness) -> Object {
        Object::Linear(Linear::from_witness(witness))
    }

    // XXX: It is possible to make this into a Mul trait, but it seems to hurt readability
    // Could we move this into the Mul file itself?
    pub fn mul_constant(&self, constant: FieldElement) -> Option<Object> {
        let obj = match self {
            Object::Null => return None,
            Object::Array(arr) => {
                let mut result = Vec::with_capacity(arr.length as usize);
                for element in arr.contents.iter() {
                    result.push(element.mul_constant(constant)?);
                }

                Object::Array(Array {
                    contents: result,
                    length: arr.length,
                })
            }
            Object::Linear(lin) => Object::Linear(lin * &constant),
            Object::Integer(integer) => {
                let result = &Linear::from_witness(integer.witness) * &constant;
                Object::Linear(result)
            }
            Object::Constants(lhs) => Object::Constants(*lhs * constant),
            Object::Arithmetic(lhs) => Object::Arithmetic(lhs * &constant),
        };
        Some(obj)
    }
}

impl From<Object> for Gate {
    fn from(poly: Object) -> Gate {
        match poly {
            Object::Arithmetic(arith) => Gate::Arithmetic(arith),
            // XXX: Arriving here means we have an internal error/bug, so we abort
            _ => unimplemented!("Only Arithmetic Objects can be converted into gates"),
        }
    }
}

// (selector_id, selector as an i128 , We don't have big int yet)
#[derive(Clone, Debug)]
pub struct Selector(pub String, pub Object); //XXX(med) I guess we know it's going to be a FieldElement, so we should probably find a way to give it FieldElement directly instead of Polynomial

impl Default for Selector {
    fn default() -> Selector {
        Selector("zero".to_string(), Object::Constants(FieldElement::zero()))
    }
}

pub struct RangedObject {
    pub(crate) start: FieldElement,
    pub(crate) end: FieldElement,
}

impl RangedObject {
    pub fn new(start: FieldElement, end: FieldElement) -> Result<Self, RuntimeErrorKind> {
        // We will move these checks into the analyser once
        // we have Private and Public integers, so they are only checked once

        // For now, we allow start and end ranges to be in the range u252
        // 252 is arbitrary and holds no weight, I simply chose it to be close to bn254
        let start_bits = start.num_bits();
        if start_bits > 252 {
            let message = format!("Currently, we only allow integers to be represented by a u252, start range needs {} bits to be represented", start_bits);
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message,
            });
        };

        let end_bits = end.num_bits();
        if end_bits > 252 {
            let message = format!("Currently, we only allow integers to be represented by a u252, end range needs {} bits to be represented", end_bits);
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message,
            });
        };

        // We only allow ascending ranges
        if (end - start).num_bits() > 252 {
            let message = "We currently only allow ranges to be ascending. For example `0..10` is  valid, however `10..0` is not".to_string();
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message,
            });
        };

        Ok(RangedObject { start, end })
    }
}

impl Iterator for RangedObject {
    type Item = FieldElement;

    #[inline]
    fn next(&mut self) -> Option<FieldElement> {
        if self.start != self.end {
            let return_val = self.start;
            self.start += FieldElement::one();
            Some(return_val)
        } else {
            None
        }
    }
}
