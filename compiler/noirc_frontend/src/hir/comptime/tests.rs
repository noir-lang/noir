#![cfg(test)]

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use fm::{FileId, FileManager};
use noirc_arena::Index;
use noirc_errors::Location;

use super::errors::InterpreterError;
use super::value::Value;
use super::Interpreter;
use crate::elaborator::Elaborator;
use crate::hir::def_collector::dc_crate::{CompilationError, DefCollector};
use crate::hir::def_collector::dc_mod::collect_defs;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleData};
use crate::hir::{Context, ParsedFiles};
use crate::node_interner::FuncId;
use crate::parse_program;

/// Create an interpreter for a code snippet and pass it to a test function.
///
/// The stdlib is not made available as a dependency.
pub(crate) fn with_interpreter<T>(
    src: &str,
    f: impl FnOnce(&mut Interpreter, FuncId, &[(CompilationError, FileId)]) -> T,
) -> T {
    let file = FileId::default();

    // Can't use Index::test_new here for some reason, even with #[cfg(test)].
    let module_id = LocalModuleId(Index::unsafe_zeroed());
    let mut modules = noirc_arena::Arena::default();
    let location = Location::new(Default::default(), file);
    let root = LocalModuleId(modules.insert(ModuleData::new(
        None,
        location,
        Vec::new(),
        Vec::new(),
        false, // is contract
        false, // is struct
    )));
    assert_eq!(root, module_id);

    let file_manager = FileManager::new(&PathBuf::new());
    let parsed_files = ParsedFiles::new();
    let mut context = Context::new(file_manager, parsed_files);
    context.def_interner.populate_dummy_operator_traits();

    let krate = context.crate_graph.add_crate_root(FileId::dummy());

    let (module, errors) = parse_program(src);
    assert_eq!(errors.len(), 0);
    let ast = module.into_sorted();

    let def_map = CrateDefMap { root: module_id, modules, krate, extern_prelude: BTreeMap::new() };
    let mut collector = DefCollector::new(def_map);

    collect_defs(&mut collector, ast, FileId::dummy(), module_id, krate, &mut context);
    context.def_maps.insert(krate, collector.def_map);

    let main = context.get_main_function(&krate).expect("Expected 'main' function");

    let pedantic_solving = true;
    let mut elaborator = Elaborator::elaborate_and_return_self(
        &mut context,
        krate,
        collector.items,
        None,
        pedantic_solving,
    );

    let errors = elaborator.errors.clone();

    let mut interpreter = elaborator.setup_interpreter();

    f(&mut interpreter, main, &errors)
}

/// Evaluate a code snippet by calling the `main` function.
fn interpret_helper(src: &str) -> Result<Value, InterpreterError> {
    with_interpreter(src, |interpreter, main, errors| {
        assert_eq!(errors.len(), 0);
        let no_location = Location::dummy();
        interpreter.call_function(main, Vec::new(), HashMap::new(), no_location)
    })
}

fn interpret(src: &str) -> Value {
    interpret_helper(src).unwrap_or_else(|error| {
        panic!("Expected interpreter to exit successfully, but found {error:?}")
    })
}

fn interpret_expect_error(src: &str) -> InterpreterError {
    interpret_helper(src).expect_err("Expected interpreter to error")
}

#[test]
fn interpreter_works() {
    let program = "comptime fn main() -> pub Field { 3 }";
    let result = interpret(program);
    assert_eq!(result, Value::Field(3u128.into()));
}

#[test]
fn interpreter_type_checking_works() {
    let program = "comptime fn main() -> pub u8 { 3 }";
    let result = interpret(program);
    assert_eq!(result, Value::U8(3u8));
}

#[test]
fn let_statement_works() {
    let program = "comptime fn main() -> pub i8 {
        let x = 4;
        x
    }";
    let result = interpret(program);
    assert_eq!(result, Value::I8(4));
}

#[test]
fn mutation_works() {
    let program = "comptime fn main() -> pub i8 {
        let mut x = 3;
        x = 4;
        x
    }";
    let result = interpret(program);
    assert_eq!(result, Value::I8(4));
}

#[test]
fn mutating_references() {
    let program = "comptime fn main() -> pub i32 {
        let x = &mut 3;
        *x = 4;
        *x
    }";
    let result = interpret(program);
    assert_eq!(result, Value::I32(4));
}

#[test]
fn mutating_mutable_references() {
    let program = "comptime fn main() -> pub i64 {
        let mut x = &mut 3;
        *x = 4;
        *x
    }";
    let result = interpret(program);
    assert_eq!(result, Value::I64(4));
}

#[test]
fn mutation_leaks() {
    let program = "comptime fn main() -> pub i8 {
        let mut x = 3;
        let y = &mut x;
        *y = 5;
        x
    }";
    let result = interpret(program);
    assert_eq!(result, Value::I8(5));
}

#[test]
fn mutating_arrays() {
    let program = "comptime fn main() -> pub u8 {
        let mut a1 = [1, 2, 3, 4];
        a1[1] = 22;
        a1[1]
    }";
    let result = interpret(program);
    assert_eq!(result, Value::U8(22));
}

#[test]
fn mutate_in_new_scope() {
    let program = "comptime fn main() -> pub u8 {
        let mut x = 0;
        x += 1;
        {
            x += 1;
        }
        x
    }";
    let result = interpret(program);
    assert_eq!(result, Value::U8(2));
}

#[test]
fn for_loop() {
    let program = "comptime fn main() -> pub u8 {
        let mut x = 0;
        for i in 0 .. 6 {
            x += i;
        }
        x
    }";
    let result = interpret(program);
    assert_eq!(result, Value::U8(15));
}

#[test]
fn for_loop_inclusive() {
    let program = "comptime fn main() -> pub u8 {
        let mut x = 0;
        for i in 0 ..= 6 {
            x += i;
        }
        x
    }";
    let result = interpret(program);
    assert_eq!(result, Value::U8(21));
}

#[test]
fn for_loop_u16() {
    let program = "comptime fn main() -> pub u16 {
        let mut x = 0;
        for i in 0 .. 6 {
            x += i;
        }
        x
    }";
    let result = interpret(program);
    assert_eq!(result, Value::U16(15));
}

#[test]
fn for_loop_with_break() {
    let program = "unconstrained comptime fn main() -> pub u32 {
        let mut x = 0;
        for i in 0 .. 6 {
            if i == 4 {
                break;
            }
            x += i;
        }
        x
    }";
    let result = interpret(program);
    assert_eq!(result, Value::U32(6));
}

#[test]
fn for_loop_with_continue() {
    let program = "unconstrained comptime fn main() -> pub u64 {
        let mut x = 0;
        for i in 0 .. 6 {
            if i == 4 {
                continue;
            }
            x += i;
        }
        x
    }";
    let result = interpret(program);
    assert_eq!(result, Value::U64(11));
}

#[test]
fn assert() {
    let program = "comptime fn main() {
        assert(1 == 1);
    }";
    let result = interpret(program);
    assert_eq!(result, Value::Unit);
}

#[test]
fn assert_fail() {
    let program = "comptime fn main() {
        assert(1 == 2);
    }";
    let result = interpret_expect_error(program);
    assert!(matches!(result, InterpreterError::FailingConstraint { .. }));
}

#[test]
fn lambda() {
    let program = "comptime fn main() -> pub u8 {
        let f = |x: u8| x + 1;
        f(1)
    }";
    let result = interpret(program);
    assert!(matches!(result, Value::U8(2)));
}

#[test]
fn non_deterministic_recursion() {
    let program = "
    comptime fn main() -> pub u64 {
        fib(10)
    }

    comptime fn fib(x: u64) -> u64 {
        if x <= 1 {
            x
        } else {
            fib(x - 1) + fib(x - 2)
        }
    }";
    let result = interpret(program);
    assert_eq!(result, Value::U64(55));
}

#[test]
fn generic_functions() {
    let program = "
    comptime fn main() -> pub u8 {
        apply(1, |x| x + 1)
    }

    comptime fn apply<T, Env, U>(x: T, f: fn[Env](T) -> U) -> U {
        f(x)
    }
    ";
    let result = interpret(program);
    assert_eq!(result, Value::U8(2));
}
