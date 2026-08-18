fn main(
    a: [Field; 4],
    outer: [[Field; 4]; 2],
    i: u32,
    j: u32,
    k: u32,
) -> pub ([Field; 4], [Field; 4]) {
    let mut a = a;
    let mut outer = outer;
    a[i] = 7;
    outer[0] = a;
    a[j] = 9;
    let snapshot = outer[k];
    assert(snapshot[j] == 0, "snapshot was mutated");
    (snapshot, a)
}
