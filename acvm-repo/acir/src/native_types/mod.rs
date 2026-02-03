//! Low-level native types used within the [crate::circuit] module for representing ACIR.

mod expression;
mod witness;
mod witness_map;
mod witness_stack;

pub use expression::Expression;
pub(crate) use expression::display_expression;
pub use witness::Witness;
pub use witness_map::WitnessMap;
pub use witness_stack::WitnessStack;
pub use witness_stack::WitnessStackError;
