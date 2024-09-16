fn main(x: [i32; 2], y: [i32; 2]) {
    for i in 0..1 {
        assert(-x[i] == y[i]);
        assert(-(-x[i]) == x[i]);
    }
}
