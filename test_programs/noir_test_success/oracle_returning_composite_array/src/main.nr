use std::test::OracleMock;

fn main() {}

#[oracle(composite_array)]
unconstrained fn composite_array() -> [(u32, bool); 4] {}

#[test]
unconstrained fn test_composite_array() {
    let array = [(1, true); 4];
    let _ = OracleMock::mock("composite_array").returns(array);
    let ret = composite_array();
    assert_eq(ret, array);
}

#[oracle(nested_tuple_array)]
unconstrained fn nested_tuple_array() -> [[(u8, u8); 3]; 4] {}

#[test]
unconstrained fn test_nested_tuple_array() {
    let array: [[(u8, u8); 3]; 4] = [
        [(1, 2), (3, 4), (5, 6)],
        [(7, 8), (9, 10), (11, 12)],
        [(13, 14), (15, 16), (17, 18)],
        [(19, 20), (21, 22), (23, 24)],
    ];
    let _ = OracleMock::mock("nested_tuple_array").returns(array);
    assert_eq(nested_tuple_array(), array);
}
