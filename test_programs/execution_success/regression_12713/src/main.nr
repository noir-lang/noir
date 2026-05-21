global G_A: [[u32; 3]] = @[[10, 20, 30]; 2];
unconstrained fn main() {
    let mut idx_a: u32 = 0;
    loop {
        if idx_a == 3 {
            break
        } else {
            idx_a = idx_a + 1;
            let mut b = G_A.pop_front();
            println(b);
            b.0[0] = 40;
        }
    }
}
