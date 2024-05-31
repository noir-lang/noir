struct Empty {}

unconstrained fn main(_inputs: Empty) -> pub Empty {
    assert(true);
    Empty {}
}
