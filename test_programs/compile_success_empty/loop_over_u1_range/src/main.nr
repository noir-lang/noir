// u1 has been removed; this test now verifies that looping over small
// unsigned ranges (u8) still works correctly.
fn main() {
    let zero: u8 = 0;
    for _ in zero..zero {}

    let one: u8 = 1;
    for _ in one..one {}

    for _ in zero..one {}
}
