global G_A: [[u32; 3]] = @[[10, 20, 30]; 3];
unconstrained fn main() {
    for op in 0..3 {
        let mut idx_a: u32 = 0;
        loop {
            if idx_a == 3 {
                break
            } else {
                idx_a = idx_a + 1;
                if op == 0 {
                    let mut b = G_A.pop_front();
                    println(b);
                    b.0[0] = 40;
                } else if op == 1 {
                    let mut b = G_A.pop_back();
                    println(b);
                    b.1[0] = 50;
                } else if op == 2 {
                    // `remove(1)` targets the middle element so its underlying memory is
                    // disjoint from the first/last elements bumped by the pop_front/pop_back
                    // fixes; otherwise those bumps would mask a regression here.
                    let mut b = G_A.remove(1);
                    println(b);
                    b.1[0] = 60;
                }
            }
        }
    }
}
