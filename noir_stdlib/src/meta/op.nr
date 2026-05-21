use crate::hash::Hash;

pub struct UnaryOp {
    op: Field,
}

// Cannot derive Eq or Hash since they internally use paths
// starting with std:: which is invalid within the same crate.
// We'd need to use crate:: instead.
impl crate::cmp::Eq for UnaryOp {
    fn eq(self, other: Self) -> bool {
        self.op == other.op
    }
}

impl crate::hash::Hash for UnaryOp {
    fn hash<H>(self, h: &mut H)
    where
        H: crate::hash::Hasher,
    {
        self.op.hash(h);
    }
}

impl UnaryOp {
    // docs:start:is_minus
    pub fn is_minus(self) -> bool {
        // docs:end:is_minus
        self.op == 0
    }

    // docs:start:is_not
    pub fn is_not(self) -> bool {
        // docs:end:is_not
        self.op == 1
    }

    // docs:start:is_mutable_reference
    pub fn is_mutable_reference(self) -> bool {
        // docs:end:is_mutable_reference
        self.op == 2
    }

    // docs:start:is_dereference
    pub fn is_dereference(self) -> bool {
        // docs:end:is_dereference
        self.op == 3
    }

    // docs:start:unary_quoted
    pub comptime fn quoted(self) -> Quoted {
        // docs:end:unary_quoted
        if self.is_minus() {
            quote { - }
        } else if self.is_not() {
            quote { ! }
        } else if self.is_mutable_reference() {
            quote { &mut }
        } else if self.is_dereference() {
            quote { * }
        } else {
            let op = self;
            crate::panic::panic(f"Unexpected unary operator in UnaryOp::quoted: {op}")
        }
    }
}

pub struct BinaryOp {
    op: Field,
}

impl crate::cmp::Eq for BinaryOp {
    fn eq(self, other: Self) -> bool {
        self.op == other.op
    }
}

impl crate::hash::Hash for BinaryOp {
    fn hash<H>(self, h: &mut H)
    where
        H: crate::hash::Hasher,
    {
        self.op.hash(h);
    }
}

impl BinaryOp {
    // docs:start:is_add
    pub fn is_add(self) -> bool {
        // docs:end:is_add
        self.op == 0
    }

    // docs:start:is_subtract
    pub fn is_subtract(self) -> bool {
        // docs:end:is_subtract
        self.op == 1
    }

    // docs:start:is_multiply
    pub fn is_multiply(self) -> bool {
        // docs:end:is_multiply
        self.op == 2
    }

    // docs:start:is_divide
    pub fn is_divide(self) -> bool {
        // docs:end:is_divide
        self.op == 3
    }

    // docs:start:is_equal
    pub fn is_equal(self) -> bool {
        // docs:end:is_equal
        self.op == 4
    }

    // docs:start:is_not_equal
    pub fn is_not_equal(self) -> bool {
        // docs:end:is_not_equal
        self.op == 5
    }

    // docs:start:is_less_than
    pub fn is_less_than(self) -> bool {
        // docs:end:is_less_than
        self.op == 6
    }

    // docs:start:is_less_than_or_equal
    pub fn is_less_than_or_equal(self) -> bool {
        // docs:end:is_less_than_or_equal
        self.op == 7
    }

    // docs:start:is_greater_than
    pub fn is_greater_than(self) -> bool {
        // docs:end:is_greater_than
        self.op == 8
    }

    // docs:start:is_greater_than_or_equal
    pub fn is_greater_than_or_equal(self) -> bool {
        // docs:end:is_greater_than_or_equal
        self.op == 9
    }

    // docs:start:is_and
    pub fn is_and(self) -> bool {
        // docs:end:is_and
        self.op == 10
    }

    // docs:start:is_or
    pub fn is_or(self) -> bool {
        // docs:end:is_or
        self.op == 11
    }

    // docs:start:is_xor
    pub fn is_xor(self) -> bool {
        // docs:end:is_xor
        self.op == 12
    }

    // docs:start:is_shift_right
    pub fn is_shift_right(self) -> bool {
        // docs:end:is_shift_right
        self.op == 13
    }

    // docs:start:is_shift_left
    pub fn is_shift_left(self) -> bool {
        // docs:end:is_shift_left
        self.op == 14
    }

    // docs:start:is_modulo
    pub fn is_modulo(self) -> bool {
        // docs:end:is_modulo
        self.op == 15
    }

    // docs:start:binary_quoted
    pub comptime fn quoted(self) -> Quoted {
        // docs:end:binary_quoted
        if self.is_add() {
            quote { + }
        } else if self.is_subtract() {
            quote { - }
        } else if self.is_multiply() {
            quote { * }
        } else if self.is_divide() {
            quote { / }
        } else if self.is_equal() {
            quote { == }
        } else if self.is_not_equal() {
            quote { != }
        } else if self.is_less_than() {
            quote { < }
        } else if self.is_less_than_or_equal() {
            quote { <= }
        } else if self.is_greater_than() {
            quote { > }
        } else if self.is_greater_than_or_equal() {
            quote { >= }
        } else if self.is_and() {
            quote { & }
        } else if self.is_or() {
            quote { | }
        } else if self.is_xor() {
            quote { ^ }
        } else if self.is_shift_right() {
            quote { >> }
        } else if self.is_shift_left() {
            quote { << }
        } else if self.is_modulo() {
            quote { % }
        } else {
            let op = self;
            crate::panic::panic(f"Unexpected binary operator in BinaryOp::quoted: {op}")
        }
    }
}

mod tests {
    use crate::meta::op::{BinaryOp, UnaryOp};

    comptime fn unary_op(quoted: Quoted) -> UnaryOp {
        let (op, _) = quoted.as_expr().unwrap().as_unary_op().unwrap();
        op
    }

    comptime fn binary_op(quoted: Quoted) -> BinaryOp {
        let (_, op, _) = quoted.as_expr().unwrap().as_binary_op().unwrap();
        op
    }

    #[test]
    fn unary_op_is_methods() {
        comptime {
            assert(unary_op(quote { -x }).is_minus());
            assert(unary_op(quote { !x }).is_not());
            assert(unary_op(quote { &mut x }).is_mutable_reference());
            assert(unary_op(quote { *x }).is_dereference());
        }
    }

    #[test]
    fn unary_op_quoted() {
        comptime {
            assert_eq(unary_op(quote { -x }).quoted(), quote { - });
            assert_eq(unary_op(quote { !x }).quoted(), quote { ! });
            assert_eq(unary_op(quote { &mut x }).quoted(), quote { &mut });
            assert_eq(unary_op(quote { *x }).quoted(), quote { * });
        }
    }

    #[test]
    fn unary_op_eq() {
        comptime {
            assert(unary_op(quote { -x }) == unary_op(quote { -x }));
            assert(unary_op(quote { -x }) != unary_op(quote { !x }));
        }
    }

    #[test]
    fn binary_op_is_methods() {
        comptime {
            assert(binary_op(quote { x + y }).is_add());
            assert(binary_op(quote { x - y }).is_subtract());
            assert(binary_op(quote { x * y }).is_multiply());
            assert(binary_op(quote { x / y }).is_divide());
            assert(binary_op(quote { x == y }).is_equal());
            assert(binary_op(quote { x != y }).is_not_equal());
            assert(binary_op(quote { x < y }).is_less_than());
            assert(binary_op(quote { x <= y }).is_less_than_or_equal());
            assert(binary_op(quote { x > y }).is_greater_than());
            assert(binary_op(quote { x >= y }).is_greater_than_or_equal());
            assert(binary_op(quote { x & y }).is_and());
            assert(binary_op(quote { x | y }).is_or());
            assert(binary_op(quote { x ^ y }).is_xor());
            assert(binary_op(quote { x >> y }).is_shift_right());
            assert(binary_op(quote { x << y }).is_shift_left());
            assert(binary_op(quote { x % y }).is_modulo());
        }
    }

    #[test]
    fn binary_op_quoted() {
        comptime {
            assert_eq(binary_op(quote { x + y }).quoted(), quote { + });
            assert_eq(binary_op(quote { x - y }).quoted(), quote { - });
            assert_eq(binary_op(quote { x * y }).quoted(), quote { * });
            assert_eq(binary_op(quote { x / y }).quoted(), quote { / });
            assert_eq(binary_op(quote { x == y }).quoted(), quote { == });
            assert_eq(binary_op(quote { x != y }).quoted(), quote { != });
            assert_eq(binary_op(quote { x < y }).quoted(), quote { < });
            assert_eq(binary_op(quote { x <= y }).quoted(), quote { <= });
            assert_eq(binary_op(quote { x > y }).quoted(), quote { > });
            assert_eq(binary_op(quote { x >= y }).quoted(), quote { >= });
            assert_eq(binary_op(quote { x & y }).quoted(), quote { & });
            assert_eq(binary_op(quote { x | y }).quoted(), quote { | });
            assert_eq(binary_op(quote { x ^ y }).quoted(), quote { ^ });
            assert_eq(binary_op(quote { x >> y }).quoted(), quote { >> });
            assert_eq(binary_op(quote { x << y }).quoted(), quote { << });
            assert_eq(binary_op(quote { x % y }).quoted(), quote { % });
        }
    }

    #[test]
    fn binary_op_eq() {
        comptime {
            assert(binary_op(quote { x + y }) == binary_op(quote { x + y }));
            assert(binary_op(quote { x + y }) != binary_op(quote { x - y }));
        }
    }

}
