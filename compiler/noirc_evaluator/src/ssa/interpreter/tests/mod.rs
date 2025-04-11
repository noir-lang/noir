#![cfg(test)]

use crate::errors::RuntimeError;

use super::{IResult, Ssa, Value};

mod instructions;

fn executes_with_no_errors(src: &str) {
    let ssa = Ssa::from_str(src).unwrap();
    assert!(super::interpret(&ssa).is_some())
}

fn expect_values(src: &str) -> Vec<Value> {
    let ssa = Ssa::from_str(src).unwrap();
    super::interpret(&ssa).unwrap()
}

fn expect_value(src: &str) -> Value {
    let mut results = expect_values(src);
    assert_eq!(results.len(), 1);
    results.pop().unwrap()
}

fn expect_error(src: &str) -> RuntimeError {
    let ssa = Ssa::from_str(src).unwrap();
    super::interpret(&ssa).unwrap_err()
}

#[test]
fn empty_program() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            return
        }
    ";
    executes_with_no_errors(src);
}
