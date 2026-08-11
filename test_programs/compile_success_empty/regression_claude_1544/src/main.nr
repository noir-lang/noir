fn main() {
    for i in 0..1 {
        let x: i8 = -1 + (i as i8);
        let r = (x as u8) as i16;
        assert_eq(r, 255);
    }
}
