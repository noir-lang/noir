// Regression test for issue #4561
use dep::std::test::OracleMock;

type TReturnElem = [Field; 3];
type TReturn = [TReturnElem; 2];

#[oracle(simple_nested_return)]
unconstrained fn simple_nested_return_oracle() -> TReturn {}

unconstrained fn simple_nested_return_unconstrained() -> TReturn {
    simple_nested_return_oracle()
}

#[test]
fn test_simple_nested_return() {
    OracleMock::mock("simple_nested_return").returns([1, 2, 3, 4, 5, 6]);
    assert_eq(simple_nested_return_unconstrained(), [[1, 2, 3], [4, 5, 6]]);
}

#[oracle(nested_with_fields_return)]
unconstrained fn nested_with_fields_return_oracle() -> (Field, TReturn, Field) {}

unconstrained fn nested_with_fields_return_unconstrained() -> (Field, TReturn, Field) {
    nested_with_fields_return_oracle()
}

#[test]
fn test_nested_with_fields_return() {
    OracleMock::mock("nested_with_fields_return").returns((0, [1, 2, 3, 4, 5, 6], 7));
    assert_eq(nested_with_fields_return_unconstrained(), (0, [[1, 2, 3], [4, 5, 6]], 7));
}

#[oracle(two_nested_return)]
unconstrained fn two_nested_return_oracle() -> (Field, TReturn, Field, TReturn) {}

unconstrained fn two_nested_return_unconstrained() -> (Field, TReturn, Field, TReturn) {
    two_nested_return_oracle()
}

#[test]
fn two_nested_return() {
    OracleMock::mock("two_nested_return").returns((0, [1, 2, 3, 4, 5, 6], 7, [1, 2, 3, 4, 5, 6]));
    assert_eq(two_nested_return_unconstrained(), (0, [[1, 2, 3], [4, 5, 6]], 7, [[1, 2, 3], [4, 5, 6]]));
}