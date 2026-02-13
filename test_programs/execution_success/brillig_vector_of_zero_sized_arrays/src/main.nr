// Regression test: passing a vector of zero-sized arrays from ACIR to Brillig
// previously panicked in the Brillig VM because the vector's semantic length
// was incorrectly computed as 0 (since the flattened element size is 0).
fn main() {
    let a: [[Field; 0]] = &[[], []];
    // Safety: testing ACIR-to-Brillig vector passing
    let l = unsafe { func_1(a) };
    assert(l == []);
}

unconstrained fn func_1(a: [[Field; 0]]) -> [Field; 0] {
    a[0]
}
