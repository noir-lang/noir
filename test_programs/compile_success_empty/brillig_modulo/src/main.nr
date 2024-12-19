// Tests a very simple program.
//
// The features being tested is modulo operations on brillig
fn main() {
    unsafe {
        //@safety: testing context
        assert(modulo(47, 3) == 2);
        assert(modulo(2, 3) == 2);
        assert(signed_modulo(5, 3) == 2);
        assert(signed_modulo(2, 3) == 2);

        let minus_two: i8 = -2; // 254
        let minus_three: i8 = -3; // 253
        let minus_five: i8 = -5; // 251
        // (5 / -3) * -3 + 2 = -1 * -3 + 2 = 3 + 2 = 5
        assert(signed_modulo(5, minus_three) == 2);
        // (-5 / 3) * 3 - 2 = -1 * 3 - 2 = -3 - 2 = -5
        assert(signed_modulo(minus_five, 3) == minus_two);
        // (-5 / -3) * -3 - 2 = 1 * -3 - 2 = -3 - 2 = -5
        assert(signed_modulo(minus_five, minus_three) == minus_two);
    }
}

unconstrained fn modulo(x: u32, y: u32) -> u32 {
    x % y
}

unconstrained fn signed_modulo(x: i8, y: i8) -> i8 {
    x % y
}
