// Normal globals can be evaluated in a comptime context too,
// but comptime globals can only be evaluated in a comptime context.
comptime global FOO: Field = foo();

// Due to this function's mutability and branching, SSA currently fails
// to fold this function into a constant before the assert_constant check
// is evaluated before loop unrolling.
comptime fn foo() -> Field {
    let mut three = 3;
    if three == 3 {
        5
    } else {
        6
    }
}

#[test]
fn foo_global_constant() {
    assert_constant(FOO);
}

#[test(should_fail)]
fn foo_function_not_constant() {
    assert_constant(foo());
}
