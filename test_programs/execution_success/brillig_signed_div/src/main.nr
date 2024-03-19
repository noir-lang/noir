struct SignedDivOp {
    lhs: i8,
    rhs: i8,
    result: i8,
}

unconstrained fn main(ops: [SignedDivOp; 15]) {
    for i in 0..15 {
        assert_eq(ops[i].lhs / ops[i].rhs, ops[i].result);
    }
}
