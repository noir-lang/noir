// Expect 'Variable must be mutable to be assigned to' error
fn main() {
    let vector : &mut [Field] = &mut [];
    vector = &mut (*vector).push_back(1);
}
