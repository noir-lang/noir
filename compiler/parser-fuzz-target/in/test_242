use dep::std::test::OracleMock;

struct Point {
    x: Field,
    y: Field,
}

#[oracle(foo)]
unconstrained fn foo_oracle(_point: Point, _array: [Field; 4]) -> Field {}

unconstrained fn main() {
    let array = [1, 2, 3, 4];
    let another_array = [4, 3, 2, 1];
    let point = Point { x: 14, y: 27 };

    OracleMock::mock("foo").returns(42).times(1);
    let mock = OracleMock::mock("foo").returns(0);
    assert_eq(42, foo_oracle(point, array));
    assert_eq(0, foo_oracle(point, array));
    mock.clear();

    OracleMock::mock("foo").with_params((point, array)).returns(10);
    OracleMock::mock("foo").with_params((point, another_array)).returns(20);
    assert_eq(10, foo_oracle(point, array));
    assert_eq(20, foo_oracle(point, another_array));
}

