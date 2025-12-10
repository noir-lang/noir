fn main(a: [[bool; 2]; 3], mut b: [[bool; 2]; 3]) -> pub bool {
    for _ in 0..2 {
        let idx_e: u32 = (if (0 != (a[2][0] as Field)) { 0 } else { 1 });

        b[2][idx_e] = (!a[0][1]);

        for idx_g in 65_u8..70_u8 {
            let idx_j = ((idx_g as u32) % 2);
            let idx_k = ((idx_g as u32) % 3);
            b[1][idx_j] = a[idx_k][1];
        }
    }
    b[0][0]
}
