struct UnaryOp {
    op: Field
}

impl UnaryOp {
    pub fn is_minus(self) -> bool {
        self.op == 0
    }

    pub fn is_not(self) -> bool {
        self.op == 1
    }

    pub fn is_mutable_reference(self) -> bool {
        self.op == 2
    }

    pub fn is_dereference(self) -> bool {
        self.op == 3
    }
}

struct BinaryOp {
    op: Field
}

impl BinaryOp {
    pub fn is_add(self) -> bool {
        self.op == 0
    }

    pub fn is_subtract(self) -> bool {
        self.op == 1
    }

    pub fn is_multiply(self) -> bool {
        self.op == 2
    }

    pub fn is_divide(self) -> bool {
        self.op == 3
    }

    pub fn is_equal(self) -> bool {
        self.op == 4
    }

    pub fn is_not_equal(self) -> bool {
        self.op == 5
    }

    pub fn is_less(self) -> bool {
        self.op == 6
    }

    pub fn is_less_equal(self) -> bool {
        self.op == 7
    }

    pub fn is_greater(self) -> bool {
        self.op == 8
    }

    pub fn is_greater_equal(self) -> bool {
        self.op == 9
    }

    pub fn is_and(self) -> bool {
        self.op == 10
    }

    pub fn is_or(self) -> bool {
        self.op == 11
    }

    pub fn is_xor(self) -> bool {
        self.op == 12
    }

    pub fn is_shift_right(self) -> bool {
        self.op == 13
    }

    pub fn is_shift_left(self) -> bool {
        self.op == 14
    }

    pub fn is_modulo(self) -> bool {
        self.op == 15
    }
}

