unconstrained fn main(x: u32) -> pub bool {
    if x == 0 {
        true
    } else {
        main(x - 1)
    }
}
