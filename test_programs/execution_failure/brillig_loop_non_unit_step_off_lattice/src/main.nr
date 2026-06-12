// Regression for noir-lang/noir-claude#102.
//
// The `if i == 4 { break }` guard makes LICM record the induction variable's bounds as `[0, 4)`,
// but the step is 3, so `i` runs `0, 3, 6, ...` and skips the guard value entirely. The bounds are
// therefore not an upper bound on `i`. LICM must not use them to delete `assert(i < 5)` (which must
// fail at `i == 6`) nor to turn the checked `i + 3` into an unchecked add.
unconstrained fn main() -> pub u8 {
    let mut i: u8 = 0;
    let mut counter: u8 = 0;

    loop {
        if i == 4 {
            break;
        }

        assert(i < 5);

        if counter == 3 {
            break;
        }

        i = i + 3;
        counter = counter + 1;
    }

    i
}
