struct Empty {}

#[recursive]
fn main(_inputs: Empty) -> pub Empty {
    assert(true);
    Empty {}
}
