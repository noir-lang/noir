use std::test::OracleMock;

struct Point {
    x: Field,
    y: Field,
}

impl Eq for Point {
    fn eq(self, other: Point) -> bool {
        (self.x == other.x) & (self.y == other.y)
    }
}

#[oracle(void_field)]
unconstrained fn void_field_oracle() -> Field {}

unconstrained fn void_field() -> Field {
    void_field_oracle()
}

#[oracle(field_field)]
unconstrained fn field_field_oracle(_x: Field) -> Field {}

unconstrained fn field_field(x: Field) -> Field {
    field_field_oracle(x)
}

#[oracle(struct_field)]
unconstrained fn struct_field_oracle(_point: Point, _array: [Field; 4]) -> Field {}

unconstrained fn struct_field(point: Point, array: [Field; 4]) -> Field {
    struct_field_oracle(point, array)
}

#[test(should_fail)]
fn test_mock_no_returns() {
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        OracleMock::mock("void_field");
        void_field(); // Some return value must be set
    }
}

#[test]
fn test_mock() {
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        OracleMock::mock("void_field").returns(10);
        assert_eq(void_field(), 10);
    }
}

#[test]
fn test_multiple_mock() {
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        let first_mock = OracleMock::mock("void_field").returns(10);
        OracleMock::mock("void_field").returns(42);

        // The mocks are searched for in creation order, so the first one prevents the second from being called.
        assert_eq(void_field(), 10);

        first_mock.clear();
        assert_eq(void_field(), 42);
    }
}

#[test]
fn test_multiple_mock_times() {
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        OracleMock::mock("void_field").returns(10).times(2);
        OracleMock::mock("void_field").returns(42);

        assert_eq(void_field(), 10);
        assert_eq(void_field(), 10);
        assert_eq(void_field(), 42);
    }
}

#[test]
fn test_mock_with_params() {
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        OracleMock::mock("field_field").with_params((5,)).returns(10);
        assert_eq(field_field(5), 10);
    }
}

#[test]
fn test_multiple_mock_with_params() {
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        OracleMock::mock("field_field").with_params((5,)).returns(10);
        OracleMock::mock("field_field").with_params((7,)).returns(14);

        assert_eq(field_field(5), 10);
        assert_eq(field_field(7), 14);
    }
}

#[test]
fn test_mock_last_params() {
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        let mock = OracleMock::mock("field_field").returns(10);
        assert_eq(field_field(5), 10);

        assert_eq(mock.get_last_params(), 5);
    }
}

#[test]
fn test_mock_last_params_many_calls() {
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        let mock = OracleMock::mock("field_field").returns(10);
        assert_eq(field_field(5), 10);
        assert_eq(field_field(7), 10);

        assert_eq(mock.get_last_params(), 7);
    }
}

#[test]
fn test_mock_struct_field() {
    // Combination of simpler test cases
    let array = [1, 2, 3, 4];
    let another_array = [4, 3, 2, 1];
    let point = Point { x: 14, y: 27 };

    /// Safety: testing context
    unsafe {
        //@safety: testing context
        OracleMock::mock("struct_field").returns(42).times(2);
        let timeless_mock = OracleMock::mock("struct_field").returns(0);
        assert_eq(42, struct_field(point, array));
        assert_eq(42, struct_field(point, array));
        // The times(2) mock is now cleared

        assert_eq(0, struct_field(point, array));
        let last_params: (Point, [Field; 4]) = timeless_mock.get_last_params();
        assert_eq(last_params.0, point);
        assert_eq(last_params.1, array);

        // We clear the mock with no times() to allow other mocks to be callable
        timeless_mock.clear();

        OracleMock::mock("struct_field").with_params((point, array)).returns(10);
        OracleMock::mock("struct_field").with_params((point, another_array)).returns(20);
        assert_eq(10, struct_field(point, array));
        assert_eq(20, struct_field(point, another_array));
    }
}

