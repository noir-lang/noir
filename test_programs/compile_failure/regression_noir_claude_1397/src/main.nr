// Regression for noir-lang/noir-claude#1397.
//
// This `loop` exits when `i == 4`, so its body sits on the `else` branch of an `eq` header: a
// `!=` guarded loop that runs while `i != 4`. Starting at `i = 5`, the body executes (5 != 4)
// and reaches the failing `static_assert(false)`, so this program must fail to compile.
//
// The `static_assert` pass decided whether the body executes via `upper > lower`. For this loop
// the inferred bounds are `lower = 5`, `upper = 4`, so `4 > 5` is false and the body was treated
// as an empty loop, deleting the live assertion and letting the program compile successfully.
unconstrained fn main() {
    let mut i: u32 = 5;
    loop {
        if i == 4 {
            break;
        }
        std::static_assert(false, "should fail");
        i -= 1;
    }
}
