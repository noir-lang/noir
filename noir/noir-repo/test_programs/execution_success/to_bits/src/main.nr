use dep::std;

fn main() {
    let field = 1000;
    let be_bits = field.to_be_bits(16);
    let le_bits = field.to_le_bits(16);

    for i in 0..16 {
        let x = be_bits[i];
        let y = le_bits[15 - i];
        assert(x == y);
    }

    let x = 3;
    let be_bits_x = x.to_be_bits(4);
    let le_bits_x = x.to_le_bits(4);

    for i in 0..4 {
        let be_bit = be_bits_x[i];
        let le_bit = le_bits_x[3 - i];
        assert(be_bit == le_bit);
    }
}
