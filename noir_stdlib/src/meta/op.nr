struct UnaryOp {
    op: Field
}

impl UnaryOp {
    fn is_minus(self) -> bool {
        self.op == 0
    }

    fn is_not(self) -> bool {
        self.op == 1
    }

    fn is_mutable_reference(self) -> bool {
        self.op == 2
    }

    fn is_dereference(self) -> bool {
        self.op == 3
    }
}
