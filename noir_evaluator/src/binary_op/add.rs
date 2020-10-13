use crate::{Arithmetic, Environment, Evaluator, FieldElement,  Linear, Object};

pub fn handle_add_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Object {
    match (left, right) {
        (Object::Null, _) => handle_null_add(),
        (Object::Arithmetic(arith), y) => handle_arithmetic_add(arith, y),
        (Object::Constants(c), y) => handle_constant_add(c, y),
        (Object::Linear(lin), y) => handle_linear_add(lin, y),
        (Object::Integer(x), y) => Object::Integer(x.add(y, env, evaluator)),
        (x, y) => panic!("{:?} and {:?} operations are unsupported"),
    }
}

fn handle_null_add() -> Object {
    panic!("Cannot do an operation with the Null Object")
}
fn handle_arithmetic_add(arith: Arithmetic, polynomial: Object) -> Object {
    match polynomial {
        Object::Arithmetic(arith_rhs) => Object::Arithmetic(&arith + &arith_rhs),
        Object::Linear(linear) => Object::Arithmetic(&arith + &linear.into()),
        Object::Constants(constant) => {
            Object::Arithmetic(&arith + &Linear::from(constant).into())
        }
        Object::Null => handle_null_add(),
        Object::Integer(_) => panic!("Can only add an integer to an integer"),
        x => super::unsupported_error(vec![x]),
    }
}
fn handle_constant_add(constant: FieldElement, polynomial: Object) -> Object {
    match polynomial {
        Object::Arithmetic(arith) => {
            Object::Arithmetic(&Linear::from(constant).into() + &arith)
        }
        Object::Linear(linear) => Object::Linear(&linear + &constant),
        Object::Constants(constant_rhs) => Object::Constants(constant + constant_rhs),
        Object::Null => handle_null_add(),
        Object::Integer(_) => panic!("Can only add an integer to an integer"),
        x => super::unsupported_error(vec![x]),
    }
}
fn handle_linear_add(linear: Linear, polynomial: Object) -> Object {
    match polynomial {
        Object::Arithmetic(arith) => Object::Arithmetic(&arith + &linear.into()),
        Object::Linear(linear_rhs) => Object::Arithmetic(&linear + &linear_rhs),
        Object::Constants(constant) => Object::Linear(&linear + &constant),
        Object::Null => handle_null_add(),
        Object::Integer(_) => panic!("Can only add an integer to an integer"),
        x => super::unsupported_error(vec![x]),
    }
}
