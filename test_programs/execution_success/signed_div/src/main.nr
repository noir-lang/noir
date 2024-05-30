struct SignedDivOp {
    lhs: i8,
    rhs: i8,
    result: i8,
}

fn main(ops: [SignedDivOp; 15]) {
    unsafe_div(ops);
    test_div(ops);
}

unconstrained fn unsafe_div(ops: [SignedDivOp; 15]) {
    test_div(ops);
}

fn test_div(ops: [SignedDivOp; 15]) {
    for i in 0..15 {
        assert_eq(ops[i].lhs / ops[i].rhs, ops[i].result);
    }
}
