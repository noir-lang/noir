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

struct BinaryOp {
    op: Field
}

impl BinaryOp {
    fn is_add(self) -> bool {
        self.op == 0
    }

    fn is_subtract(self) -> bool {
        self.op == 1
    }

    fn is_multiply(self) -> bool {
        self.op == 2
    }

    fn is_divide(self) -> bool {
        self.op == 3
    }

    fn is_equal(self) -> bool {
        self.op == 4
    }

    fn is_not_equal(self) -> bool {
        self.op == 5
    }

    fn is_less(self) -> bool {
        self.op == 6
    }

    fn is_less_equal(self) -> bool {
        self.op == 7
    }

    fn is_greater(self) -> bool {
        self.op == 8
    }

    fn is_greater_or_equal(self) -> bool {
        self.op == 9
    }

    fn is_and(self) -> bool {
        self.op == 10
    }

    fn is_or(self) -> bool {
        self.op == 11
    }

    fn is_xor(self) -> bool {
        self.op == 12
    }

    fn is_shift_right(self) -> bool {
        self.op == 13
    }

    fn is_shift_left(self) -> bool {
        self.op == 14
    }

    fn is_modulo(self) -> bool {
        self.op == 15
    }
}

