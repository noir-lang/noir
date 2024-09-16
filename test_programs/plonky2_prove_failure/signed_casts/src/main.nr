fn main(x: [i8; 2], y: [i16; 2], z: [i32; 2]) {
    for i in 0..1 {
        //  i8 -> i16
        assert(x[i] as i16 == y[i]);
        //  i8 -> i32
        assert(x[i] as i32 == z[i]);
        // i16 -> i32
        assert(y[i] as i32 == z[i]);
    }
}
