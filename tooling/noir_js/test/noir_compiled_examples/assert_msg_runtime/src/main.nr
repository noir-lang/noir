fn main(x: u64, y: pub u64) {
    // A fmtstr assertion message is used to show that noirJS will decode the error payload as a string.
    assert(x < y, f"Expected x < y but got {x} < {y}");
}
