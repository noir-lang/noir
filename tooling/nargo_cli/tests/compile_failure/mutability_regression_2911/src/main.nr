// Expect 'Variable must be mutable to be assigned to' error
fn main() {
    let slice : &mut [Field] = &mut [];
    slice = &mut (*slice).push_back(1);
}
