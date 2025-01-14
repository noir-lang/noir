fn main() {
    let field = 1000;
    let be_bits: [u1; 16] = field.to_be_bits();
    let le_bits: [u1; 16] = field.to_le_bits();

    for i in 0..16 {
        let x = be_bits[i];
        let y = le_bits[15 - i];
        assert(x == y);
    }

    let x = 3;
    let be_bits_x: [u1; 4] = x.to_be_bits();
    let le_bits_x: [u1; 4] = x.to_le_bits();

    for i in 0..4 {
        let be_bit = be_bits_x[i];
        let le_bit = le_bits_x[3 - i];
        assert(be_bit == le_bit);
    }
}
