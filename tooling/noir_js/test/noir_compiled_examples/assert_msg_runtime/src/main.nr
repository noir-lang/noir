fn main(x: u64, y: pub u64) {
    // A dynamic assertion message is used to show that noirJS will ignore the call and continue execution
    // We need this assertion to fail as the `assert_message` oracle in Noir is only called
    // upon a failing condition in an assert.
    assert(x < y, f"Expected x < y but got {x} < {y}");
}
