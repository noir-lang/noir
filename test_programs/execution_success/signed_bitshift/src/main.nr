fn main() {
    for idx_d in 0..1 {
        if idx_d == 2 {
            // previously the LICM pass would hoist this bitshift out of the loop (and so out from under the predicate)
            // such that it would fail execution.
            let a = (345422577_i32 >> -115035501_i32);
            let _ = 1 >> a;
        }
    }
}
