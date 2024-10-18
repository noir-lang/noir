// A simple program to test that SSA array values elements
// aren't disconnected from their instruction results, and
// that dead instruction elemination looks inside of arrays
// when deciding whether of not an instruction should be
// retained.
fn main(x: Field) -> pub [Field; 1] {
    [x + 1]
}
