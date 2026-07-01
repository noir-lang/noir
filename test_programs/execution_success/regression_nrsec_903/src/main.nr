#[inline_never]
unconstrained fn get_vector() -> [u32] {
    @[100]
}

unconstrained fn main() -> pub u32 {
    let _ = get_vector().pop_front();
    get_vector()[0]
}
