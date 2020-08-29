use super::sub::handle_sub_op;
use crate::Polynomial;

/// XXX(med) : So at the moment, Equals is the same as SUB
/// Most likely we will need to check if it is a predicate equal or infix equal

/// This calls the sub op under the hood
/// We negate the RHS and send it to the add op
pub fn handle_equal_op(left: Polynomial, right: Polynomial) -> Polynomial {
    handle_sub_op(left, right)
}
