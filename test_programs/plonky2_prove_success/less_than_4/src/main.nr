fn main(x1: pub [u8; 16], y1: pub [u8; 16],
        x2: pub [u8; 16], y2: pub [u8; 16],
        x3: pub [u8; 16], y3: pub [u8; 16],
        x4: pub [u8; 16], y4: pub [u8; 16],
        z: [[bool; 16]; 16]) {
    for i in 0..16 {
        for j in 0..16 {
            assert(z[i][j] == (x1[i] < y1[j]));
            assert(z[i][j] == (x2[i] < y2[j]));
            assert(z[i][j] == (x3[i] < y3[j]));
            assert(z[i][j] == (x4[i] < y4[j]));
        }
    }
}
