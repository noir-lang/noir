#[inline_never]
unconstrained fn get_slice() -> [u32] {
    @[100]
}

unconstrained fn main() -> pub u32 {
    let _ = get_slice().pop_front();
    get_slice()[0]
}
