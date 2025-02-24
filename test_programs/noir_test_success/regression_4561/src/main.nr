// Regression test for issue #4561
use std::test::OracleMock;

type TReturnElem = [Field; 3];
type TReturn = [TReturnElem; 2];

#[oracle(simple_nested_return)]
unconstrained fn simple_nested_return_oracle() -> TReturn {}

unconstrained fn simple_nested_return_unconstrained() -> TReturn {
    simple_nested_return_oracle()
}

#[test]
unconstrained fn test_simple_nested_return() {
    OracleMock::mock("simple_nested_return").returns([1, 2, 3, 4, 5, 6]);
    assert_eq(simple_nested_return_unconstrained(), [[1, 2, 3], [4, 5, 6]]);
}

#[oracle(nested_with_fields_return)]
unconstrained fn nested_with_fields_return_oracle() -> (Field, TReturn, Field) {}

unconstrained fn nested_with_fields_return_unconstrained() -> (Field, TReturn, Field) {
    nested_with_fields_return_oracle()
}

#[test]
unconstrained fn test_nested_with_fields_return() {
    OracleMock::mock("nested_with_fields_return").returns((0, [1, 2, 3, 4, 5, 6], 7));
    assert_eq(nested_with_fields_return_unconstrained(), (0, [[1, 2, 3], [4, 5, 6]], 7));
}

#[oracle(two_nested_return)]
unconstrained fn two_nested_return_oracle() -> (Field, TReturn, Field, TReturn) {}

unconstrained fn two_nested_return_unconstrained() -> (Field, TReturn, Field, TReturn) {
    two_nested_return_oracle()
}

#[test]
unconstrained fn two_nested_return() {
    OracleMock::mock("two_nested_return").returns((0, [1, 2, 3, 4, 5, 6], 7, [1, 2, 3, 4, 5, 6]));
    assert_eq(
        two_nested_return_unconstrained(),
        (0, [[1, 2, 3], [4, 5, 6]], 7, [[1, 2, 3], [4, 5, 6]]),
    );
}

#[oracle(foo_return)]
unconstrained fn foo_return() -> (Field, TReturn, TestTypeFoo) {}
unconstrained fn foo_return_unconstrained() -> (Field, TReturn, TestTypeFoo) {
    foo_return()
}

struct TestTypeFoo {
    a: Field,
    b: [[[Field; 3]; 4]; 2],
    c: [TReturnElem; 2],
    d: TReturnElem,
}

#[test]
unconstrained fn complexe_struct_return() {
    OracleMock::mock("foo_return").returns((
        0, [1, 2, 3, 4, 5, 6], 7,
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24],
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], [1, 2, 3, 4, 5, 6],
    ));
    let foo_x = foo_return_unconstrained();
    assert_eq((foo_x.0, foo_x.1), (0, [[1, 2, 3], [4, 5, 6]]));
    assert_eq(foo_x.2.a, 7);
    assert_eq(
        foo_x.2.b,
        [
            [[1, 2, 3], [4, 5, 6], [7, 8, 9], [10, 11, 12]],
            [[13, 14, 15], [16, 17, 18], [19, 20, 21], [22, 23, 24]],
        ],
    );
    let a: TReturnElem = [1, 2, 3];
    let b: TReturnElem = [4, 5, 6];
    assert_eq(foo_x.2.c, [a, b]);
    assert_eq(foo_x.2.d, a);
}
