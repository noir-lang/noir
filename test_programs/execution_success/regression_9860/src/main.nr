fn main(mut a: pub [[u32; 4]; 4]) -> pub [[u32; 4]; 4] {
    let idx_b = a[0][0];
    let idx_f = a[2][3];
    a[2][idx_f] = a[2][2];
    a[idx_b][1] = 0;
    a
}
