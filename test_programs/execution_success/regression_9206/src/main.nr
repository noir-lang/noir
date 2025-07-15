global G_A: [[u128; 0]; 2] = [[], []];
global G_S: [[u128; 0]] = [[], []];
fn main(i: u32) -> pub u32 {
    let _a = unsafe { func_3(G_A[i])[1] };
    let _s = unsafe { func_3(G_S[i])[1] };
    i
}
unconstrained fn func_3(_a: [u128; 0]) -> [[u128; 0]; 2] {
    G_A
}
