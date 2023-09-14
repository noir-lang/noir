fn main(a: u32, mut c: [u32; 4]){
    if a == c[0] {
        assert(c[0] == 0);
    } else if a == c[1] {
        assert(c[1] == 0);
    } else if a == c[2] {
        assert(c[2] == 0);
    } else if a == c[3] {
        // expect to match this case
        assert(c[3] == 0);
    } else {
       assert(c[0] == 10);
    }
}

