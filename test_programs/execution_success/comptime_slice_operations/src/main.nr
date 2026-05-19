unconstrained fn run() -> [Field; 5] {
    let s: [Field] = &[1, 2, 3];
    let s = s.push_back(4);
    let (first, s) = s.pop_front();
    let s = s.push_front(10);
    let (s, last) = s.pop_back();
    let s = s.insert(1, 20);
    let (s, removed) = s.remove(0);
    [first, last, removed, s[0], s.len() as Field]
}

fn main() {
    let expected = comptime { run() };
    // Safety: comparing comptime vs brillig
    let result = unsafe { run() };
    assert(result == expected);
}
