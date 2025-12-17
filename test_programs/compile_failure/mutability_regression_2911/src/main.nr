// Expect 'Variable must be mutable to be assigned to' error
fn main() {
    let list : &mut [Field] = &mut [];
    list = &mut (*list).push_back(1);
}
