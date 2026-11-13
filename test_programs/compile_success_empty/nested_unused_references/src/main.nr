// Each `&` here is an allocation held inside the array one level up. None of the arrays are
// read, so every allocation must be removed before ACIR generation.
fn main() {
    let _nested = &[&[&[&1]]];
}
