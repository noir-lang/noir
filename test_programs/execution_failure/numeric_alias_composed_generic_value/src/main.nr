global N: u32 = 2;
type Double<let N: u32>: u32 = N * 2;
type Quad<let N: u32>: u32 = Double::<N> + Double::<N>;

// `Quad::<3>` is 12. The `N` passed to `Double` is `Quad`'s own generic, not the global `N`
// (which would make it 8). An input of 8 must be rejected.
fn main(x: pub u32) {
    assert(x == Quad::<3>);
}
