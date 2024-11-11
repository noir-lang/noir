fn main(x: [[i16; 3]; 8]) {
    for i in 0..7 {
        assert(x[i][0] / x[i][1] == x[i][2]);
    }
}
