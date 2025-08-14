global G_B: [[str<2>; 1]; 2] = [["DS"], ["EN"]];
fn main() -> pub [[str<2>; 1]; 2] {
    unsafe { func_2(G_B) };
    G_B
}
unconstrained fn func_2(mut b: [[str<2>; 1]; 2]) {
    let mut idx_d: u32 = 0;
    loop {
        if (idx_d == 1) {
            break
        } else {
            idx_d = (idx_d + 1);
            b[0][0] = G_B[1][0];
        }
    }
}
