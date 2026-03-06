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

#[oracle(func_field)]
unconstrained fn func_field_oracle(_x: u32, _f: fn(u32, u64) -> Field, _y: u64) -> Field {}

unconstrained fn func_field(x: u32, f: fn(u32, u64) -> Field, y: u64) -> Field {
    func_field_oracle(x, f, y)
}

#[oracle(field_func)]
unconstrained fn field_func_oracle(_x: Field) -> fn(u32, u64) -> Field {}

unconstrained fn field_func(x: Field) -> fn(u32, u64) -> Field {
    field_func_oracle(x)
}

#[oracle(brillig_func_field)]
unconstrained fn brillig_func_field_oracle(
    _x: u8,
    _f: unconstrained fn(u8, u128) -> Field,
    _y: u128,
) -> Field {}

unconstrained fn brillig_func_field(
    x: u8,
    f: unconstrained fn(u8, u128) -> Field,
    y: u128,
) -> Field {
    brillig_func_field_oracle(x, f, y)
}

#[test(should_fail)]
fn test_mock_no_returns() {
    // Safety: testing context
    unsafe {
        OracleMock::mock("void_field");
        void_field(); // Some return value must be set
    }
}

#[test]
fn test_mock() {
    // Safety: testing context
    unsafe {
        OracleMock::mock("void_field").returns(10);
        assert_eq(void_field(), 10);
    }
}

#[test]
fn test_multiple_mock() {
    // Safety: testing context
    unsafe {
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
    // Safety: testing context
    unsafe {
        OracleMock::mock("void_field").returns(10).times(2);
        OracleMock::mock("void_field").returns(42);

        assert_eq(void_field(), 10);
        assert_eq(void_field(), 10);
        assert_eq(void_field(), 42);
    }
}

#[test]
fn test_mock_with_params() {
    // Safety: testing context
    unsafe {
        OracleMock::mock("field_field").with_params((5,)).returns(10);
        assert_eq(field_field(5), 10);
    }
}

#[test]
fn test_multiple_mock_with_params() {
    // Safety: testing context
    unsafe {
        OracleMock::mock("field_field").with_params((5,)).returns(10);
        OracleMock::mock("field_field").with_params((7,)).returns(14);

        assert_eq(field_field(5), 10);
        assert_eq(field_field(7), 14);
    }
}

#[test]
fn test_mock_last_params() {
    // Safety: testing context
    unsafe {
        let mock = OracleMock::mock("field_field").returns(10);
        assert_eq(field_field(5), 10);

        assert_eq(mock.get_last_params(), 5);
    }
}

#[test]
fn test_mock_last_params_many_calls() {
    // Safety: testing context
    unsafe {
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

    // Safety: testing context
    unsafe {
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

#[test]
fn test_times_called() {
    // Safety: testing context
    unsafe {
        let mock = OracleMock::mock("void_field").returns(10);
        assert_eq(mock.times_called(), 0);

        call_void_field_twice(mock, 0);

        assert_eq(void_field(), 10);
        assert_eq(void_field(), 10);
        assert_eq(void_field(), 10);
        assert_eq(void_field(), 10);
        assert_eq(void_field(), 10);

        call_void_field_twice(mock, 7);

        assert_eq(mock.times_called(), 9);
    }
}

unconstrained fn call_void_field_twice(mock: OracleMock, initial_times_called: Field) {
    assert_eq(mock.times_called(), initial_times_called);
    assert_eq(void_field(), 10);
    assert_eq(void_field(), 10);
    assert_eq(mock.times_called(), initial_times_called + 2);
}

#[test]
unconstrained fn test_function_pointer() {
    // Create a lambda, which will become its own global function with an ID.
    let add = |x: u32, y: u64| (x as Field) + (y as Field);
    let mul = |x: u32, y: u64| (x as Field) * (y as Field);

    // Testing that and oracle can take and return a function as an ID.

    // Set the expectation that we will pass the `add` lambda, surrounded by other numbers
    // to make sure deserialization does not affect them.
    let mock = OracleMock::mock("func_field").with_params((10, add, 20)).returns(30);

    // Pass another lambda just so it's used as a value and we should see the dispatch function handle it.
    let _mock = OracleMock::mock("func_field").with_params((10, mul, 20)).returns(200);

    // Call the oracle with the expected parameters.
    let ret = func_field(10, add, 20);
    // It should match the parameters and return the expected result.
    // This is trivial as the mocker just stores the inputs as-is.
    assert_eq(ret, 30);

    // Get the last params back and parse whatever we passed for `add` into a
    // function type with the correct signature. It will only be an ID, but
    // giving it type should allow us to call it.
    let (x, f, y): (u32, fn(u32, u64) -> Field, u64) = mock.get_last_params();

    // Check that receiving a function did not affect the values before/after it.
    assert_eq(x, 10);
    assert_eq(y, 20);

    // Now call the function and see that it returns the same result as `add`.
    // Note that `f` _is_ `add`, as it is nothing but an ID of a function in SSA.
    let r = f(x, y);
    assert_eq(r, add(x, y));
    assert(r != mul(x, y));
}

#[test]
unconstrained fn test_brillig_function_pointer() {
    // Safety: test that we can pass a function to the oracle and back, when this is the only use of the function,
    // it is never directly called from Noir, and so it will not have a dispatch function.
    // Because it is a pointer to an unconstrained function, it won't have 2 different versions of created;
    // instead it will be passed as a tuple of the same ID on both sides, which is a bit inconsistent with
    // the type that indicates a constrained and an unconstrained function, but since the constrained one
    // is never called, and even if it was it would have the same result, it doesn't matter.
    let mock =
        OracleMock::mock("brillig_func_field").with_params((10, brillig_add_8_128, 20)).returns(30);
    // Call the oracle with the expected parameters.
    let _ = brillig_func_field(10, brillig_add_8_128, 20);
    // Get the function back.
    let (x, f, y): (u8, unconstrained fn(u8, u128) -> Field, u128) = mock.get_last_params();
    // Now call the function and see that it returns the expected result.
    // Since we don't have a dispatch function, we will actually call the function directly.
    assert_eq(f(x, y), 30);
}

unconstrained fn brillig_add_8_128(x: u8, y: u128) -> Field {
    (x as Field) + (y as Field)
}

#[test(should_fail)]
unconstrained fn test_invalid_function_pointer() {
    // Create a lambda, so we have a dispatch function to its constrained/unconstrained versions.
    let add = |x: u32, y: u64| (x as Field) + (y as Field);
    let sub = |x: u32, y: u64| (x as Field) - (y as Field);

    // Test that returning an arbitrary function pointer cannot be invoked.

    // Return the ID of the main entry point itself, rather than the ID of `add`.
    // Need to return a tuple: the IDs of the constrained and unconstrained version.
    let _ = OracleMock::mock("field_func").with_params((0,)).returns((0, 0));
    // Also make it possible to return the valid lambdas, so we get dispatch for them.
    let _ = OracleMock::mock("field_func").with_params((1,)).returns(add);
    let _ = OracleMock::mock("field_func").with_params((2,)).returns(sub);
    // Call the oracle with the sneaky parameters that return an invalid ID.
    let f: fn(u32, u64) -> Field = field_func(0);
    // Trying to call the function should result in an error, since the ID 0 is not handled by the dispatch.
    let _ = f(10, 20);
}
