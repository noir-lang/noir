use crate::constants::TWO_POW_64;

// This is a quick struct made to pack 32 bytes into 4 u64s
// and then pack those into two u128s.
//
// Creating a u256 was made for convenience.
//
// This is needed because in the cpp code, we have accumulate_sha256
// which returns 2 field elements, one for the high and low limb.
pub struct U256 {
    // This is in big-endian order, typically because
    // sha256 is usually in big endian order.
    // Note: this means that inner[0] has the most significant 64 bits.
    inner: [u64; 4],
}

impl U256 {
    pub fn from_bytes32(bytes: [u8; 32]) -> U256 {
        // We use addition rather than a bitwise OR as the bitshifts ensure that none of the bytes overlap each other.
        let high_0 = ((bytes[0] as u64) << 56)
            + ((bytes[1] as u64) << 48)
            + ((bytes[2] as u64) << 40)
            + ((bytes[3] as u64) << 32)
            + ((bytes[4] as u64) << 24)
            + ((bytes[5] as u64) << 16)
            + ((bytes[6] as u64) << 8)
            + (bytes[7] as u64);

        let high_1 = ((bytes[8] as u64) << 56)
            + ((bytes[9] as u64) << 48)
            + ((bytes[10] as u64) << 40)
            + ((bytes[11] as u64) << 32)
            + ((bytes[12] as u64) << 24)
            + ((bytes[13] as u64) << 16)
            + ((bytes[14] as u64) << 8)
            + (bytes[15] as u64);

        let low_0 = ((bytes[16] as u64) << 56)
            + ((bytes[17] as u64) << 48)
            + ((bytes[18] as u64) << 40)
            + ((bytes[19] as u64) << 32)
            + ((bytes[20] as u64) << 24)
            + ((bytes[21] as u64) << 16)
            + ((bytes[22] as u64) << 8)
            + (bytes[23] as u64);

        let low_1 = ((bytes[24] as u64) << 56)
            + ((bytes[25] as u64) << 48)
            + ((bytes[26] as u64) << 40)
            + ((bytes[27] as u64) << 32)
            + ((bytes[28] as u64) << 24)
            + ((bytes[29] as u64) << 16)
            + ((bytes[30] as u64) << 8)
            + (bytes[31] as u64);

        U256 { inner: [high_0, high_1, low_0, low_1] }
    }

    // We cannot represent u128 in the type system
    // so we cannot return a u128 type.
    //
    // This as conversion assumes that Field can store 128 bits of information
    // or else the conversion is lossy.
    //
    // TODO: Add a test for this.
    pub fn to_u128_limbs(self) -> [Field; 2] {
        let high = (self.inner[0] as Field) * TWO_POW_64 + self.inner[1] as Field;
        let low = (self.inner[2] as Field) * TWO_POW_64 + self.inner[3] as Field;

        [high, low]
    }
}

#[test]
fn smoke_u256_from_bytes32_all_zeroes() {
    let input = [0; 32];
    let result = U256::from_bytes32(input);
    assert(result.inner[0] == 0);
    assert(result.inner[1] == 0);
    assert(result.inner[2] == 0);
    assert(result.inner[3] == 0);
}

#[test]
fn smoke_u256_from_bytes32_one_zero_zero_zero() {
    // We want to output [1,0,0,0]
    let input = [
        0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0,
    ];
    let result = U256::from_bytes32(input);

    assert(result.inner[0] == 1);
    assert(result.inner[1] == 0);
    assert(result.inner[2] == 0);
    assert(result.inner[3] == 0);
}

#[test]
fn smoke_u256_from_bytes32_test() {
    /*
    input:  [   0xAA, 0xBB, 0xCC, 0xDD,
                0xEE, 0xFF, 0x00, 0x11,
                0x22, 0x33, 0x44, 0x55,
                0x66, 0x77, 0x88, 0x99,
                0x11, 0x22, 0x33, 0x44,
                0x55, 0x66, 0x77, 0x88,
                0x99, 0xAA, 0xBB, 0xCC,
                0xDD, 0xEE, 0xFF, 0x00
            ]
    output: inner[0]: 0xAABBCCDDEEFF0011
            inner[1]: 0x2233445566778899
            inner[2]: 0x1122334455667788
            inner[3]: 0x99AABBCCDDEEFF00
    */
    let input: [u8; 32] = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
        0xFF, 0x00,
    ];
    let result = U256::from_bytes32(input);

    assert(result.inner[0] == 0xAABBCCDDEEFF0011);
    assert(result.inner[1] == 0x2233445566778899);
    assert(result.inner[2] == 0x1122334455667788);
    assert(result.inner[3] == 0x99AABBCCDDEEFF00);
}
