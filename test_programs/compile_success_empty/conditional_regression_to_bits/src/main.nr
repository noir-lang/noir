fn main() {
    //Regression for to_le_bits() constant evaluation
    // binary array representation of u8 1
    let arr: [u8; 2] = [1, 2];
    let as_bits_hardcode_1 = [1, 0];
    let mut c1 = 0;
    for i in 0..2 {
        let mut as_bits: [u1; 2] = (arr[i] as Field).to_le_bits();
        c1 = c1 + as_bits[0] as Field;

        if i == 0 {
            assert(arr[i] == 1); // 1
            for k in 0..2 {
                assert(as_bits_hardcode_1[k] == as_bits[k]);
            }
        }
        if i == 1 {
            assert(arr[i] == 2); //2
            for k in 0..2 {
                assert(as_bits_hardcode_1[k] != as_bits[k]);
            }
        }
    }
    assert(c1 == 1);
}

