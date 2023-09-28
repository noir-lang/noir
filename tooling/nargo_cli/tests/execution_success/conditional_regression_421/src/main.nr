fn main(a: u32, mut c: [u32; 4]){
    //Issue reported in #421
    if a == c[0] {
        assert(c[0] == 0);
    } else {
        if a == c[1] {
            assert(c[1] == 0);
        } else  {
            if a == c[2] {
                assert(c[2] == 0);
            }
        }
    }
}
