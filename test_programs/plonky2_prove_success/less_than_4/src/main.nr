fn main(x: pub [u8; 16], y: pub [u8; 16], z: [[bool; 16]; 16]) {
    for i in 0..16 {
        for j in 0..16 {
            assert(z[i][j] == (x[i] < y[j]));
        }
    }
}
