fn main(y: [[i16; 3]; 8]) {
    for i in 0..7 {
        assert(y[i][0] % y[i][1] == y[i][2]);
    }
}
