use super::add::handle_add_op;
use crate::Polynomial;

/// This calls the add op under the hood
/// We negate the RHS and send it to the add op
pub fn handle_sub_op(left: Polynomial, right: Polynomial) -> Polynomial {
    let negated_right = match right {
        Polynomial::Null => panic!("Cannot do an operation with the Null Polynomial"),
        Polynomial::Arithmetic(arith) => Polynomial::Arithmetic(-&arith),
        Polynomial::Constants(c) => Polynomial::Constants(-c),
        Polynomial::Linear(linear) => Polynomial::Linear(-&linear),
    };

    handle_add_op(left, negated_right)
}
