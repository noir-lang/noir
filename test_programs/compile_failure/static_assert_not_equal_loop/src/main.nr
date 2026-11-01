// Regression test for noir-lang/noir-claude#1397.
//
// The induction variable starts ABOVE the guard and counts down, so the loop body runs
// (5 != 4). The header is a single `eq` with the body on the `else` branch — a `NotEqual`
// loop with `lower (5) > upper (4)`. `LoopBounds::loop_executes` checks `upper > lower`
// (i.e. `4 > 5 == false`) and wrongly classifies the loop as empty, so
// `evaluate_static_assert_and_assert_constant` deletes the reachable `static_assert(false)`
// and the program compiles. It must instead fail to compile.
unconstrained fn main() {
    let mut i: u8 = 5;
    loop {
        if i == 4 {
            break;
        }
        std::static_assert(false, "static_assert in a live NotEqual loop body must fire");
        i -= 1;
    }
}
