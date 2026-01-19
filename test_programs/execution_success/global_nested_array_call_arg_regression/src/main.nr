global G_C: [[bool; 3]; 1] = [[true, false, true]];
fn main(a: bool, zero: u32) -> pub bool {
    // NOTE: If the nested array input is not cloned this test will fail
    // with `--minimal-ssa` as `func_1` will not be inlined and G_C will be mutated directly.
    let b = func_1(G_C[zero])[zero + 1];
    if a {
        G_C[zero][zero + 1]
    } else {
        b
    }
}
fn func_1(mut b: [bool; 3]) -> [bool; 3] {
    b[1] = true;
    b
}
