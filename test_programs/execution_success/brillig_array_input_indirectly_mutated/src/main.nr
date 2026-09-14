unconstrained fn main() -> pub [bool; 1] {
    let _ = f([true]);
    [true]
}

#[inline_never]
unconstrained fn f(mut b: [bool; 1]) -> [bool; 1] {
    for _ in 0_u32..=0_u32 {
        b[0] = false;
    }
    b
}
