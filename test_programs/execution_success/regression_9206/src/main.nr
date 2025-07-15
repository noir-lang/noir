global G_A: [[u128; 0]; 2] = [[], []];
fn main(b: u8) -> pub u8 {
    let _d = unsafe { func_3(func_3([], b)[1], 0) };
    b
}
unconstrained fn func_3(_a: [u128; 0], _b: u8) -> [[u128; 0]; 2] {
    G_A
}
