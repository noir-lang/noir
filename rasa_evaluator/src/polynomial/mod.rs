mod arithmetic;
mod linear;

pub use arithmetic::Arithmetic;
pub use linear::Linear;

use super::circuit::Witness;
use crate::circuit::gate::Gate;
use rasa_field::FieldElement;

#[derive(Clone, Debug)]
pub enum Polynomial {
    Null,
    Arithmetic(Arithmetic),
    Constants(FieldElement), // These will mostly be the selectors
    Linear(Linear), // These will be selector * witness(var_name) + selector // Note that this is not a gate Eg `5x+6` does not apply a gate
}

impl Polynomial {
    pub fn is_gate(&self) -> bool {
        match self {
            Polynomial::Arithmetic(_) => true,
            _ => false,
        }
    }
    pub fn constant(&self) -> Option<FieldElement> {
        match self {
            Polynomial::Constants(x) => Some(*x),
            _ => None,
        }
    }
    pub fn is_constant(&self) -> bool {
        match self {
            Polynomial::Constants(x) => true,
            _ => false,
        }
    }
    pub fn arithmetic(&self) -> Option<&Arithmetic> {
        match self {
            Polynomial::Arithmetic(x) => Some(x),
            _ => None,
        }
    }

    pub fn witness(&self) -> Option<Witness> {
        if !self.is_unit_witness() {
            return None;
        };
        match self {
            Polynomial::Linear(linear) => {
                assert!(linear.mul_scale == FieldElement::one());
                assert!(linear.add_scale == FieldElement::zero());
                Some(linear.witness.clone())
            }
            _ => None,
        }
    }
    pub fn linear(&self) -> Option<Linear> {
        match self {
            Polynomial::Linear(linear) => Some(linear.clone()),
            _ => None,
        }
    }
    // Returns true if the linear polynomial is a regular witness that has not been scaled
    pub fn is_unit_witness(&self) -> bool {
        match self {
            Polynomial::Linear(linear) => linear.is_unit(),
            _ => false,
        }
    }
    // Returns true if the polynomial is linear
    pub fn is_linear(&self) -> bool {
        match self {
            Polynomial::Linear(_) => true,
            Polynomial::Constants(_) => true,
            _ => false,
        }
    }
    pub fn from_witness(witness: Witness) -> Polynomial {
        Polynomial::Linear(Linear::from_witness(witness))
    }
}

impl From<Polynomial> for Gate {
    fn from(poly: Polynomial) -> Gate {
        match poly {
            Polynomial::Arithmetic(arith) => Gate::Arithmetic(arith),
            _ => unimplemented!("Only Arithmetic Polynomials can be converted into gates"),
        }
    }
}
