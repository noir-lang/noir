struct UnaryOp {
    op: Field
}

impl UnaryOp {
    // docs:start:is_minus
    pub fn is_minus(self) -> bool {
        self.op == 0
    }
    // docs:end:is_minus

    // docs:start:is_not
    pub fn is_not(self) -> bool {
        self.op == 1
    }
    // docs:end:is_not

    // docs:start:is_mutable_reference
    pub fn is_mutable_reference(self) -> bool {
        self.op == 2
    }
    // docs:end:is_mutable_reference

    // docs:start:is_dereference
    pub fn is_dereference(self) -> bool {
        self.op == 3
    }
    // docs:end:is_dereference
}

struct BinaryOp {
    op: Field
}

impl BinaryOp {
    // docs:start:is_add
    pub fn is_add(self) -> bool {
        self.op == 0
    }
    // docs:end:is_add

    // docs:start:is_subtract
    pub fn is_subtract(self) -> bool {
        self.op == 1
    }
    // docs:end:is_subtract

    // docs:start:is_multiply
    pub fn is_multiply(self) -> bool {
        self.op == 2
    }
    // docs:end:is_multiply

    // docs:start:is_divide
    pub fn is_divide(self) -> bool {
        self.op == 3
    }
    // docs:end:is_divide

    // docs:start:is_equal
    pub fn is_equal(self) -> bool {
        self.op == 4
    }
    // docs:end:is_equal

    // docs:start:is_not_equal
    pub fn is_not_equal(self) -> bool {
        self.op == 5
    }
    // docs:end:is_not_equal

    // docs:start:is_less
    pub fn is_less(self) -> bool {
        self.op == 6
    }
    // docs:end:is_less

    // docs:start:is_less_equal
    pub fn is_less_equal(self) -> bool {
        self.op == 7
    }
    // docs:end:is_less_equal

    // docs:start:is_greater
    pub fn is_greater(self) -> bool {
        self.op == 8
    }
    // docs:end:is_greater

    // docs:start:is_greater_equal
    pub fn is_greater_equal(self) -> bool {
        self.op == 9
    }
    // docs:end:is_greater_equal

    // docs:start:is_and
    pub fn is_and(self) -> bool {
        self.op == 10
    }
    // docs:end:is_and

    // docs:start:is_or
    pub fn is_or(self) -> bool {
        self.op == 11
    }
    // docs:end:is_or

    // docs:start:is_xor
    pub fn is_xor(self) -> bool {
        self.op == 12
    }
    // docs:end:is_xor

    // docs:start:is_shift_right
    pub fn is_shift_right(self) -> bool {
        self.op == 13
    }
    // docs:end:is_shift_right

    // docs:start:is_shift_left
    pub fn is_shift_left(self) -> bool {
        self.op == 14
    }
    // docs:end:is_shift_left

    // docs:start:is_modulo
    pub fn is_modulo(self) -> bool {
        self.op == 15
    }
    // docs:end:is_modulo
}

