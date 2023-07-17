use dep::std;



fn main(a: u32, mut c: [u32; 4], x: [u8; 5], result: pub [u8; 32]){
    if a == 0 {
        c[0] = 3;
    } else {
        c[0] = 1;
        c[1] = c[2] / a + 11 % a;
        let f1 = a as Field;
        assert(10/f1 != 0);
    }
    assert(c[0] == 3);
}
