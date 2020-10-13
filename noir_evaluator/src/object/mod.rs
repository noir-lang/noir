mod array;
mod integer;

pub use array::Array;
pub use integer::Integer;

pub use acir::native_types::{Witness, Linear, Arithmetic};
use acir::circuit::gate::Gate;
use noir_field::FieldElement;

#[derive(Clone, Debug)]
pub enum Object {
    Null,
    Integer(Integer),
    Array(Array),
    Arithmetic(Arithmetic),
    Constants(FieldElement), // These will mostly be the selectors
    Linear(Linear), // These will be selector * witness(var_name) + selector // Note that this is not a gate Eg `5x+6` does not apply a gate
}

impl Object {
    // Converts a Object into an arithmetic object
    pub fn into_arithmetic(&self) -> Arithmetic {
        match self {
            Object::Null => panic!("Cannot convert null into a Object"),
            Object::Integer(integer) => Linear::from_witness(integer.witness.clone()).into(),
            Object::Array(_) => panic!("Cannot convert an array into an arithmetic object"),
            Object::Arithmetic(arith) => arith.clone(),
            Object::Constants(constant) => constant.into(),
            Object::Linear(linear) => linear.into()
        }
    }
    pub fn is_gate(&self) -> bool {
        match self {
            Object::Arithmetic(_) => true,
            _ => false,
        }
    }
    pub fn constant(&self) -> Option<FieldElement> {
        match self {
            Object::Constants(x) => Some(*x),
            _ => None,
        }
    }
    pub fn is_constant(&self) -> bool {
        match self {
            Object::Constants(_) => true,
            _ => false,
        }
    }
    pub fn arithmetic(&self) -> Option<&Arithmetic> {
        match self {
            Object::Arithmetic(x) => Some(x),
            _ => None,
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
                Some(linear.witness.clone())
            }
            _ => None,
        }
    }
    pub fn linear(&self) -> Option<Linear> {
        match self {
            Object::Linear(linear) => Some(linear.clone()),
            Object::Integer(integer) => Some(Linear::from(integer.witness.clone())),
            _ => None,
        }
    }
    pub fn integer(&self) -> Option<Integer> {
        match self {
            Object::Integer(integer) => Some(integer.clone()),
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
        match self {
            Object::Linear(_) => true,
            Object::Constants(_) => true,
            Object::Integer(_) => true,
            _ => false,
        }
    }
    pub fn from_witness(witness: Witness) -> Object {
        Object::Linear(Linear::from_witness(witness))
    }
}

impl From<Object> for Gate {
    fn from(poly: Object) -> Gate {
        match poly {
            Object::Arithmetic(arith) => Gate::Arithmetic(arith),
            _ => unimplemented!("Only Arithmetic Objects can be converted into gates"),
        }
    }
}

// (selector_id, selector as an i128 , We don't have big int yet)
#[derive(Clone, Debug)]
pub struct Selector(pub String, pub Object); //XXX(med) I guess we know it's going to be a FieldElement, so we should probably find a way to give it FieldElement directly instead of Polynomial

impl Default for Selector {
    fn default() -> Selector {
        Selector(
            "zero".to_string(),
            Object::Constants(FieldElement::zero()),
        )
    }
}
