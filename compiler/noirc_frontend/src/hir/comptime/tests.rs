#![cfg(test)]

use noirc_errors::Location;

use super::errors::InterpreterError;
use super::interpreter::Interpreter;
use super::value::Value;
use crate::hir::type_check::test::type_check_src_code;

fn interpret_helper(src: &str, func_namespace: Vec<String>) -> Result<Value, InterpreterError> {
    let (mut interner, main_id) = type_check_src_code(src, func_namespace);
    let mut interpreter = Interpreter::new(&mut interner);

    let no_location = Location::dummy();
    interpreter.call_function(main_id, Vec::new(), no_location)
}

fn interpret(src: &str, func_namespace: Vec<String>) -> Value {
    interpret_helper(src, func_namespace).unwrap_or_else(|error| {
        panic!("Expected interpreter to exit successfully, but found {error:?}")
    })
}

fn interpret_expect_error(src: &str, func_namespace: Vec<String>) -> InterpreterError {
    interpret_helper(src, func_namespace).expect_err("Expected interpreter to error")
}

#[test]
fn interpreter_works() {
    let program = "fn main() -> pub Field { 3 }";
    let result = interpret(program, vec!["main".into()]);
    assert_eq!(result, Value::Field(3u128.into()));
}

#[test]
fn mutation_works() {
    let program = "fn main() -> pub i8 {
        let mut x = 3;
        x = 4;
        x
    }";
    let result = interpret(program, vec!["main".into()]);
    assert_eq!(result, Value::I8(4));
}

#[test]
fn mutating_references() {
    let program = "fn main() -> pub i32 {
        let x = &mut 3;
        *x = 4;
        *x
    }";
    let result = interpret(program, vec!["main".into()]);
    assert_eq!(result, Value::I32(4));
}

#[test]
fn mutating_mutable_references() {
    let program = "fn main() -> pub i64 {
        let mut x = &mut 3;
        *x = 4;
        *x
    }";
    let result = interpret(program, vec!["main".into()]);
    assert_eq!(result, Value::I64(4));
}

#[test]
fn mutating_arrays() {
    let program = "fn main() -> pub u8 {
        let mut a1 = [1, 2, 3, 4];
        a1[1] = 22;
        a1[1]
    }";
    let result = interpret(program, vec!["main".into()]);
    assert_eq!(result, Value::U8(22));
}

#[test]
fn mutate_in_new_scope() {
    let program = "fn main() -> pub u8 {
        let mut x = 0;
        x += 1;
        {
            x += 1;
        }
        x
    }";
    let result = interpret(program, vec!["main".into()]);
    assert_eq!(result, Value::U8(2));
}

#[test]
fn for_loop() {
    let program = "fn main() -> pub u8 {
        let mut x = 0;
        for i in 0 .. 6 {
            x += i;
        }
        x
    }";
    let result = interpret(program, vec!["main".into()]);
    assert_eq!(result, Value::U8(15));
}

#[test]
fn for_loop_u16() {
    let program = "fn main() -> pub u16 {
        let mut x = 0;
        for i in 0 .. 6 {
            x += i;
        }
        x
    }";
    let result = interpret(program, vec!["main".into()]);
    assert_eq!(result, Value::U16(15));
}

#[test]
fn for_loop_with_break() {
    let program = "unconstrained fn main() -> pub u32 {
        let mut x = 0;
        for i in 0 .. 6 {
            if i == 4 {
                break;
            }
            x += i;
        }
        x
    }";
    let result = interpret(program, vec!["main".into()]);
    assert_eq!(result, Value::U32(6));
}

#[test]
fn for_loop_with_continue() {
    let program = "unconstrained fn main() -> pub u64 {
        let mut x = 0;
        for i in 0 .. 6 {
            if i == 4 {
                continue;
            }
            x += i;
        }
        x
    }";
    let result = interpret(program, vec!["main".into()]);
    assert_eq!(result, Value::U64(11));
}

#[test]
fn assert() {
    let program = "fn main() {
        assert(1 == 1);
    }";
    let result = interpret(program, vec!["main".into()]);
    assert_eq!(result, Value::Unit);
}

#[test]
fn assert_fail() {
    let program = "fn main() {
        assert(1 == 2);
    }";
    let result = interpret_expect_error(program, vec!["main".into()]);
    assert!(matches!(result, InterpreterError::FailingConstraint { .. }));
}

#[test]
fn lambda() {
    let program = "fn main() -> pub u8 {
        let f = |x: u8| x + 1;
        f(1)
    }";
    let result = interpret(program, vec!["main".into()]);
    assert!(matches!(result, Value::U8(2)));
}

#[test]
fn non_deterministic_recursion() {
    let program = "
    fn main() -> pub u64 {
        fib(10)
    }

    fn fib(x: u64) -> u64 {
        if x <= 1 {
            x
        } else {
            fib(x - 1) + fib(x - 2)
        }
    }";
    let result = interpret(program, vec!["main".into(), "fib".into()]);
    assert_eq!(result, Value::U64(55));
}
