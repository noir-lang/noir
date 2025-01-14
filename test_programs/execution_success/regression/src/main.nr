global NIBBLE_LENGTH: u32 = 16;

struct U4 {
    inner: u8,
}

impl U4 {
    fn zero() -> U4 {
        U4 { inner: 0 }
    }

    fn from_u8(x: u8) -> U4 {
        U4 { inner: x % 16 }
    }
}

impl Eq for U4 {
    fn eq(self, other: Self) -> bool {
        self.inner == other.inner
    }
}

fn compact_decode<let N: u32>(input: [u8; N], length: Field) -> ([U4; NIBBLE_LENGTH], Field) {
    assert(2 * input.len() <= NIBBLE_LENGTH);
    assert(length as u32 <= input.len());

    let mut nibble = [U4::zero(); NIBBLE_LENGTH];

    let first_nibble = U4::from_u8(input[0] >> 4);
    let parity = first_nibble.inner as u1;

    if parity == 1 {
        nibble[0] = U4::from_u8(input[0] & 0x0f);
        for i in 1..input.len() {
            if i as u32 < length as u32 {
                let x = input[i];
                nibble[2 * i - 1] = U4::from_u8(x >> 4);
                nibble[2 * i] = U4::from_u8(x & 0x0f);
            }
        }
    } else {
        for i in 0..2 {
            if (i as u32) < length as u32 - 1 {
                let x = input[i + 1];
                nibble[2 * i] = U4::from_u8(x >> 4);
                nibble[2 * i + 1] = U4::from_u8(x & 0x0f);
            }
        }
    }

    let out = (nibble, 2 * length + (parity as Field) - 2);

    out
}

fn enc<let N: u32>(value: [u8; N], value_length: Field) -> ([u8; 32], Field) {
    assert(value.len() as u8 >= value_length as u8);
    let mut out_value = [0; 32];
    if value_length == 0 {
        let out = (out_value, value_length);
        out
    } else if value_length as u8 < 31 {
        out_value[0] = 0x80 + value_length as u8;

        for i in 1..value.len() {
            out_value[i] = value[i - 1];
        }

        let out = (out_value, value_length + 1);

        out
    } else {
        let out = (out_value, 32);
        out
    }
}

fn bitshift_literal_0() -> u64 {
    let mut bits: u64 = 0;
    bits |= 1 << 0;

    bits
}
fn bitshift_literal_4() -> u64 {
    let mut bits: u64 = 0;
    bits |= 1 << 4;

    bits
}
fn bitshift_variable(idx: u8) -> u64 {
    let mut bits: u64 = 0;
    bits |= 1 << idx;

    bits
}

fn main(x: [u8; 5], z: Field, u: i16, v: i16) {
    //Issue 1144
    let (nib, len) = compact_decode(x, z);
    assert(len == 5);
    assert(
        [nib[0], nib[1], nib[2], nib[3], nib[4]]
            == [U4::from_u8(15), U4::from_u8(1), U4::from_u8(12), U4::from_u8(11), U4::from_u8(8)],
    );
    // Issue 1169
    let val1 = [
        0xb8, 0x8f, 0x61, 0xe6, 0xfb, 0xda, 0x83, 0xfb, 0xff, 0xfa, 0xbe, 0x36, 0x41, 0x12, 0x13,
        0x74, 0x80, 0x39, 0x80, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    let val1_length = 20;

    let enc_val1 = enc(val1, val1_length);

    assert(
        enc_val1.0
            == [
                0x94, 0xb8, 0x8f, 0x61, 0xe6, 0xfb, 0xda, 0x83, 0xfb, 0xff, 0xfa, 0xbe, 0x36, 0x41,
                0x12, 0x13, 0x74, 0x80, 0x39, 0x80, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ],
    );
    assert(enc_val1.1 == 21);
    // Issue 2399
    let result_0 = bitshift_literal_0();
    assert(result_0 == 1);
    let result_4 = bitshift_literal_4();
    assert(result_4 == 16);
    let result_0 = bitshift_variable(0);
    assert(result_0 == 1);
    let result_4 = bitshift_variable(4);
    assert(result_4 == 16);

    // Issue 6609
    assert(u % -13 == 0);
    assert(u % v == 0);
    assert(u % -11 == 4);
    assert(-u % -11 == -4);
    assert(u % -11 == u % (v + 2));
    assert(-u % -11 == -u % (v + 2));
}
