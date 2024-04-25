fn main(z: u8, matrix: pub [[u8; 8]; 8]) {
    let mut x: u8 = 0;
    for i in 0..8 {
        for j in 0..8 {
            x = x ^ matrix[i][j];
        }
    } 
    assert(z == x);
}
