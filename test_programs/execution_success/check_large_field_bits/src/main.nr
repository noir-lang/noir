// 2^32 + 1
global A: Field = 4294967297;
global B: Field = 4294967297;

// 2^33 + 2
global C: Field = A + B;

fn main() {
    // 2 * (2^32 + 1) == 2^33 + 2
    assert(C == 8589934594);

    let mut leading_zeroes = 0;
    let mut stop = false;
    let bits: [u1; 64] = C.to_be_bits();
    for i in 0..64 {
        if (bits[i] == 0) & !stop {
            leading_zeroes += 1;
        } else {
            stop = true;
        }
    }
    let size = 64 - leading_zeroes;

    // 8589934594 has 34 bits
    assert(size == 34);
    C.assert_max_bit_size::<34>();

    assert(
        C.to_be_bits()
            == [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 1, 0,
            ],
    );

    // leading big-endian bits past 34 are 0's
    assert(
        C.to_be_bits()
            == [
                0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 1, 0,
            ],
    );
    assert(
        C.to_be_bits()
            == [
                0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 1, 0,
            ],
    );
}
