use crate::{
    hir_def::{expr::HirExpression, stmt::HirStatement},
    node_interner::{NodeInterner, StmtId},
    test_utils::get_program,
    tests::{assert_no_errors, check_errors},
};

#[test]
fn infers_lambda_argument_from_method_call_function_type() {
    let src = r#"
    struct Foo {
        value: Field,
    }

    impl Foo {
        fn foo(self) -> Field {
            self.value
        }
    }

    struct Box<T> {
        value: T,
    }

    impl<T> Box<T> {
        fn map<U>(self, f: fn(T) -> U) -> Box<U> {
            Box { value: f(self.value) }
        }
    }

    fn main() {
        let box = Box { value: Foo { value: 1 } };
        let _ = box.map(|foo| foo.foo());
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_call_function_type() {
    let src = r#"
    struct Foo {
        value: Field,
    }

    fn call(f: fn(Foo) -> Field) -> Field {
        f(Foo { value: 1 })
    }

    fn main() {
        let _ = call(|foo| foo.value);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_call_function_tuple_type() {
    let src = r#"
    struct Foo {
        value: Field,
    }

    fn call(f: (fn(Foo) -> Field, fn(Foo) -> Field)) -> (Field, Field) {
        let v = Foo { value: 1 };
        let (fa, fb) = f;
        let a = fa(v);
        let b = fb(v);
        (a, b)
    }

    fn main() {
        let _ = call((|foo| foo.value - 1, |foo| foo.value + 1));
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_call_function_type_in_generic_call() {
    let src = r#"
    struct Foo {
        value: Field,
    }

    fn call<T>(t: T, f: fn(T) -> Field) -> Field {
        f(t)
    }

    fn main() {
        let _ = call(Foo { value: 1 }, |foo| foo.value);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_call_function_type_as_alias() {
    let src = r#"
    struct Foo {
        value: Field,
    }

    type MyFn = fn(Foo) -> Field;

    fn call(f: MyFn) -> Field {
        f(Foo { value: 1 })
    }

    fn main() {
        let _ = call(|foo| foo.value);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_function_return_type() {
    let src = r#"
    pub struct Foo {
        value: Field,
    }

    pub fn func() -> fn(Foo) -> Field {
        |foo| foo.value
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_function_return_type_multiple_statements() {
    let src = r#"
    pub struct Foo {
        value: Field,
    }

    pub fn func() -> fn(Foo) -> Field {
        let _ = 1;
        |foo| foo.value
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_function_return_type_when_inside_if() {
    let src = r#"
    pub struct Foo {
        value: Field,
    }

    pub fn func() -> fn(Foo) -> Field {
        if true {
            |foo| foo.value
        } else {
            |foo| foo.value
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_variable_type() {
    let src = r#"
    pub struct Foo {
        value: Field,
    }

    fn main() {
      let _: fn(Foo) -> Field = |foo| foo.value;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_variable_alias_type() {
    let src = r#"
    pub struct Foo {
        value: Field,
    }

    type FooFn = fn(Foo) -> Field;

    fn main() {
      let _: FooFn = |foo| foo.value;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_variable_double_alias_type() {
    let src = r#"
    pub struct Foo {
        value: Field,
    }

    type FooFn = fn(Foo) -> Field;
    type FooFn2 = FooFn;

    fn main() {
      let _: FooFn2 = |foo| foo.value;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_variable_tuple_type() {
    let src = r#"
    pub struct Foo {
        value: Field,
    }

    fn main() {
      let _: (fn(Foo) -> Field, _) = (|foo| foo.value, 1);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn infers_lambda_argument_from_variable_tuple_type_aliased() {
    let src = r#"
    pub struct Foo {
        value: Field,
    }

    type Alias = (fn(Foo) -> Field, Field);

    fn main() {
      let _: Alias = (|foo| foo.value, 1);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn regression_7088() {
    // A test for code that initially broke when implementing inferring
    // lambda parameter types from the function type related to the call
    // the lambda is in (PR #7088).
    let src = r#"
    struct U60Repr<let N: u32, let NumSegments: u32> {}

    impl<let N: u32, let NumSegments: u32> U60Repr<N, NumSegments> {
        fn new<let NumFieldSegments: u32>(_: [Field; N * NumFieldSegments]) -> Self {
            U60Repr {}
        }
    }

    fn main() {
        let input: [Field; 6] = [0; 6];
        let _: U60Repr<3, 6> = U60Repr::new(input);
    }
    "#;
    assert_no_errors(src);
}

fn get_program_captures(src: &str) -> Vec<Vec<String>> {
    let (program, context, _errors) = get_program(src);
    let interner = context.def_interner;
    let mut all_captures: Vec<Vec<String>> = Vec::new();
    for func in program.into_sorted().functions {
        let func_id = interner.find_function(func.item.name()).unwrap();
        let hir_func = interner.function(&func_id);
        // Iterate over function statements and apply filtering function
        find_lambda_captures(hir_func.block(&interner).statements(), &interner, &mut all_captures);
    }
    all_captures
}

fn find_lambda_captures(stmts: &[StmtId], interner: &NodeInterner, result: &mut Vec<Vec<String>>) {
    for stmt_id in stmts.iter() {
        let hir_stmt = interner.statement(stmt_id);
        let expr_id = match hir_stmt {
            HirStatement::Expression(expr_id) => expr_id,
            HirStatement::Let(let_stmt) => let_stmt.expression,
            HirStatement::Assign(assign_stmt) => assign_stmt.expression,
            HirStatement::Semi(semi_expr) => semi_expr,
            HirStatement::For(for_loop) => for_loop.block,
            HirStatement::Loop(block) => block,
            HirStatement::While(_, block) => block,
            HirStatement::Error => panic!("Invalid HirStatement!"),
            HirStatement::Break => panic!("Unexpected break"),
            HirStatement::Continue => panic!("Unexpected continue"),
            HirStatement::Comptime(_) => panic!("Unexpected comptime"),
        };
        let expr = interner.expression(&expr_id);

        get_lambda_captures(expr, interner, result); // TODO: dyn filter function as parameter
    }
}

fn get_lambda_captures(
    expr: HirExpression,
    interner: &NodeInterner,
    result: &mut Vec<Vec<String>>,
) {
    if let HirExpression::Lambda(lambda_expr) = expr {
        let mut cur_capture = Vec::new();

        for capture in lambda_expr.captures.iter() {
            cur_capture.push(interner.definition(capture.ident.id).name.clone());
        }
        result.push(cur_capture);

        // Check for other captures recursively within the lambda body
        let hir_body_expr = interner.expression(&lambda_expr.body);
        if let HirExpression::Block(block_expr) = hir_body_expr {
            find_lambda_captures(block_expr.statements(), interner, result);
        }
    }
}

#[test]
fn resolve_basic_closure() {
    let src = r#"
        fn main(x : Field) -> pub Field {
            let closure = |y| y + x;
            closure(x)
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolve_simplified_closure() {
    // based on bug https://github.com/noir-lang/noir/issues/1088
    let src = r#"
      fn do_closure(x: Field) -> Field {
        let y = x;
        let ret_capture = || {
          y
        };
        ret_capture()
      }

      fn main(x: Field) {
          assert(do_closure(x) == 100);
      }

      "#;
    let parsed_captures = get_program_captures(src);
    let expected_captures = vec![vec!["y".to_string()]];
    assert_eq!(expected_captures, parsed_captures);
}

#[test]
fn resolve_complex_closures() {
    let src = r#"
        fn main(x: Field) -> pub Field {
            let closure_without_captures = |x: Field| -> Field { x + x };
            let a = closure_without_captures(1);

            let closure_capturing_a_param = |y: Field| -> Field { y + x };
            let b = closure_capturing_a_param(2);

            let closure_capturing_a_local_var = |y: Field| -> Field { y + b };
            let c = closure_capturing_a_local_var(3);

            let closure_with_transitive_captures = |y: Field| -> Field {
                let d = 5;
                let nested_closure = |z: Field| -> Field {
                    let doubly_nested_closure = |w: Field| -> Field { w + x + b };
                    a + z + y + d + x + doubly_nested_closure(4) + x + y
                };
                let res = nested_closure(5);
                res
            };

            a + b + c + closure_with_transitive_captures(6)
        }
    "#;
    assert_no_errors(src);

    let expected_captures = vec![
        vec![],
        vec!["x".to_string()],
        vec!["b".to_string()],
        vec!["x".to_string(), "b".to_string(), "a".to_string()],
        vec!["x".to_string(), "b".to_string(), "a".to_string(), "y".to_string(), "d".to_string()],
        vec!["x".to_string(), "b".to_string()],
    ];

    let parsed_captures = get_program_captures(src);

    assert_eq!(expected_captures, parsed_captures);
}

#[test]
fn mutate_with_reference_in_lambda() {
    let src = r#"
    fn main() {
        let x = &mut 3;
        let f = || {
            *x += 2;
        };
        f();
        assert(*x == 5);
    }
    "#;

    assert_no_errors(src);
}

#[test]
fn mutate_with_reference_marked_mutable_in_lambda() {
    let src = r#"
    fn main() {
        let mut x = &mut 3;
        let f = || {
            *x += 2;
        };
        f();
        assert(*x == 5);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn deny_capturing_mut_variable_without_reference_in_lambda() {
    let src = r#"
    fn main() {
        let mut x = 3;
        let f = || {
            x += 2;
            ^ Mutable variable x captured in lambda must be a mutable reference
            ~ Use '&mut' instead of 'mut' to capture a mutable variable.
        };
        f();
        assert(x == 5);
    }
    "#;
    check_errors(src);
}

#[test]
fn deny_capturing_mut_variable_without_reference_in_nested_lambda() {
    let src = r#"
    fn main() {
        let mut x = 3;
        let f = || {
            let inner = || {
                x += 2;
                ^ Mutable variable x captured in lambda must be a mutable reference
                ~ Use '&mut' instead of 'mut' to capture a mutable variable.
            };
            inner();
        };
        f();
        assert(x == 5);
    }
    "#;
    check_errors(src);
}

#[test]
fn allow_capturing_mut_variable_only_used_immutably() {
    let src = r#"
    fn main() {
        let mut x = 3;
        let f = || x;
        let _x2 = f();
        assert(x == 3);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn deny_capturing_mut_var_as_param_to_function() {
    let src = r#"
    fn main() {
        let mut x = 3;
        let f = || mutate(&mut x);
                               ^ Mutable variable x captured in lambda must be a mutable reference
                               ~ Use '&mut' instead of 'mut' to capture a mutable variable.
        f();
        assert(x == 3);
    }

    fn mutate(x: &mut Field) {
        *x = 5;
    }
    "#;
    check_errors(src);
}

#[test]
fn deny_capturing_mut_var_as_param_to_function_in_nested_lambda() {
    let src = r#"
    fn main() {
        let mut x = 3;
        let f = || {
            let inner = || mutate(&mut x);
                                       ^ Mutable variable x captured in lambda must be a mutable reference
                                       ~ Use '&mut' instead of 'mut' to capture a mutable variable.
            inner();
        };
        f();
        assert(x == 3);
    }

    fn mutate(x: &mut Field) {
        *x = 5;
    }
    "#;
    check_errors(src);
}

#[test]
fn deny_capturing_mut_var_as_param_to_impl_method() {
    let src = r#"
    struct Foo {
        value: Field,
    }

    impl Foo {
        fn mutate(&mut self) {
            self.value = 2;
        }
    }

    fn main() {
        let mut foo = Foo { value: 1 };
        let f = || foo.mutate();
                   ^^^ Mutable variable foo captured in lambda must be a mutable reference
                   ~~~ Use '&mut' instead of 'mut' to capture a mutable variable.
        f();
        assert(foo.value == 2);
    }
    "#;
    check_errors(src);
}

#[test]
fn deny_attaching_mut_ref_to_immutable_object() {
    let src = r#"
    struct Foo {
        value: Field,
    }

    impl Foo {
        fn mutate(&mut self) {
            self.value = 2;
        }
    }

    fn main() {
        let foo = Foo { value: 1 };
        let f = || foo.mutate();
                   ^^^ Cannot mutate immutable variable `foo`
        f();
        assert(foo.value == 2);
    }
    "#;
    check_errors(src);
}
