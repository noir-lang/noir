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
