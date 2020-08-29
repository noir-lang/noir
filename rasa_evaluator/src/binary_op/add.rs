use crate::{Arithmetic, FieldElement, Linear, Polynomial};

pub fn handle_add_op(left: Polynomial, right: Polynomial) -> Polynomial {
    match (left, right) {
        (Polynomial::Null, _) => handle_null_add(),
        (Polynomial::Arithmetic(arith), y) => handle_arithmetic_add(arith, y),
        (Polynomial::Constants(c), y) => handle_constant_add(c, y),
        (Polynomial::Linear(lin), y) => handle_linear_add(lin, y),
    }
}

fn handle_null_add() -> Polynomial {
    panic!("Cannot do an operation with the Null Polynomial")
}
fn handle_arithmetic_add(arith: Arithmetic, polynomial: Polynomial) -> Polynomial {
    match polynomial {
        Polynomial::Arithmetic(arith_rhs) => Polynomial::Arithmetic(&arith + &arith_rhs),
        Polynomial::Linear(linear) => Polynomial::Arithmetic(&arith + &linear.into()),
        Polynomial::Constants(constant) => {
            Polynomial::Arithmetic(&arith + &Linear::from(constant).into())
        }
        Polynomial::Null => handle_null_add(),
    }
}
fn handle_constant_add(constant: FieldElement, polynomial: Polynomial) -> Polynomial {
    match polynomial {
        Polynomial::Arithmetic(arith) => {
            Polynomial::Arithmetic(&Linear::from(constant).into() + &arith)
        }
        Polynomial::Linear(linear) => Polynomial::Linear(&linear + &constant),
        Polynomial::Constants(constant_rhs) => Polynomial::Constants(constant + constant_rhs),
        Polynomial::Null => handle_null_add(),
    }
}
fn handle_linear_add(linear: Linear, polynomial: Polynomial) -> Polynomial {
    match polynomial {
        Polynomial::Arithmetic(arith) => Polynomial::Arithmetic(&arith + &linear.into()),
        Polynomial::Linear(linear_rhs) => Polynomial::Arithmetic(&linear + &linear_rhs),
        Polynomial::Constants(constant) => Polynomial::Linear(&linear + &constant),
        Polynomial::Null => handle_null_add(),
    }
}
