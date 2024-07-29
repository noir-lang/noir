// Based on the code https://github.com/shuklaayush/noir-bigint
use crate::wrapping_sub;
use crate::bigint::Secpk1Fq;

// Symbols and notation
/*
  N = NUM_BLOCKS
  BYTES_PER_BLOCK = 4, because of 4 bytes in u32
  NUM_BYTES = 4*N  (NUM_BLOCKS * BYTES_PER_BLOCK)
  BITS_PER_BLOCK = 32
*/
struct BigUint<let N: u32> {
    blocks: [u32; N],
}

// All conversion operations with bytes and bits are performed in little endian format
impl<let N: u32> BigUint<N> {
    fn utils_adc(_self: Self, a: u32, b: u32, carry: u32) -> (u32, u32) {
        let ret = a as Field + b as Field + carry as Field;
        (ret as u32, (ret as u64 >> 32 as u8) as u32)
    }

    fn utils_sbb(_self: Self, a: u32, b: u32, borrow: u32) -> (u32, u32) {
        // 31 = number bits -1
        let ret = wrapping_sub(a as u64, b as u64 + (borrow as u64 >> 31 as u8));
        (ret as u32, (ret >> 32 as u8) as u32)
    }

    // Compute a + (b * c) + carry. Return (res, new_carry)
    fn utils_mac(_self: Self, a: u32, b: u32, c: u32, carry: u32) -> (u32, u32) {
        let ret = (a as Field) + (b as Field) * (c as Field) + (carry as Field);
        (ret as u32, (ret as u64 >> 32 as u8) as u32)
    }

    fn to_u8_arr(arr: [u8; 32]) -> [u8] {
        arr
    }
    fn to_u8_32(arr: [u8]) -> [u8; 32] {
        let mut res = [0 as u8; 32];
        for i in 0..arr.len() {
            res[i] = arr[i];
        }
        res
    }

    pub fn zero() -> Self {
        Self { blocks: [0 as u32; N] }
    }

    pub fn one() -> Self {
        let mut one = BigUint::zero();
        one.blocks[0] = 1;
        one
    }

    pub fn from_le_bytes(bytes: [u8]) -> Self {
        assert(bytes.len() as u32 <= 4 * N);

        let mut res = BigUint::zero();
        for i in 0..bytes.len() {
            let block_ind = (i as u32) / (4 as u32);
            let byte_ind = (i as u32) % (4 as u32);

            res.blocks[block_ind as Field] |= (bytes[i] as u32) << (byte_ind * 8) as u8;
        }
        res
    }

    pub fn from_le_bytes_32(bytes: [u8; 32]) -> Self {
        let mut res = BigUint::zero();
        for i in 0..32 {
            let block_ind = i / 4;
            let byte_ind = i % 4;

            res.blocks[block_ind] |= (bytes[i] as u32) << (byte_ind * 8) as u8;
        }
        res
    }

    pub fn from_Secpk1Fq(num: Secpk1Fq) -> Self {
        BigUint::from_le_bytes_32(BigUint::to_u8_32(num.to_le_bytes()))
    }

    pub fn to_le_blocks(self: Self) -> [u32; N] {
        self.blocks
    }

    pub fn to_Secpk1Fq(self: Self) -> Secpk1Fq {
        Secpk1Fq::from_le_bytes(BigUint::to_u8_arr(self.to_le_bytes_32()))
    }

    pub fn to_le_bytes_32(self: Self) -> [u8; 32] {
        let mut res = [0 as u8; 32];
        let blocks = 8; // 32 / 4

        for i in 0..blocks {
            let block_bytes = (self.blocks[i] as Field).to_le_bytes(4 as u32);

            for j in 0..4 {
                let idx = i * 4 + j;
                res[idx] = block_bytes[j as Field];
            }
        }
        res
    }

    fn adc(self: Self, other: Self) -> (Self, u32) {
        let mut res = BigUint::zero();
        let mut carry = 0 as u32;

        for i in 0..N {
            let (sum, new_carry) = self.utils_adc(self.blocks[i], other.blocks[i], carry);
            res.blocks[i] = sum;
            carry = new_carry;
        }
        (res, carry)
    }

    pub fn add(self: Self, other: Self) -> Self {
        self.adc(other).0
    }

    fn sbb(self: Self, other: Self) -> (Self, u32) {
        let mut res = BigUint::zero();
        let mut borrow = 0 as u32;

        for i in 0..N {
            let (diff, new_borrow) = self.utils_sbb(self.blocks[i], other.blocks[i], borrow);
            res.blocks[i] = diff;
            borrow = new_borrow;
        }
        (res, borrow)
    }

    pub fn sub(self: Self, other: Self) -> Self {
        self.sbb(other).0
    }

    // low, high results
    pub fn mul(self: Self, other: Self) -> (Self, Self) {
        let mut lo = BigUint::zero();
        let mut hi = BigUint::zero();

        for i in 0..N {
            let mut carry = 0 as u32;

            for j in 0..N {
                let k = i + j;

                if k >= N {
                    let (n, c) = self.utils_mac(hi.blocks[k - N], self.blocks[i], other.blocks[j], carry);
                    hi.blocks[k - N] = n;
                    carry = c;
                } else {
                    let (n, c) = self.utils_mac(lo.blocks[k], self.blocks[i], other.blocks[j], carry);
                    lo.blocks[k] = n;
                    carry = c;
                }
            }
            hi.blocks[i] = carry;
        }
        (lo, hi)
    }

    // most significant bit
    pub fn msb(self: Self) -> u64 {
        let mut res = 0;

        for i in 0..N {
            let bits = (self.blocks[i] as Field).to_le_bits(32);
            for j in 0..32 {
                if bits[j] == 1 {
                    res = i*32 + j + 1;
                }
            }
        }
        res as u64
    }

    // Shift by 0 <= n < BITS_PER_BLOCK bits
    fn shl_block(self: Self, n: u8) -> (Self, u32) {
        assert(n < 32 as u8);

        let mut res = self;
        let rshift = 32 - n;
        let carry = if n == 0 {
            0
        } else {
            self.blocks[N - 1] >> rshift
        };

        if (n > 0) {
            res.blocks[0] = self.blocks[0] << n;
            for i in 1..N {
                res.blocks[i] = (self.blocks[i] << n) | (self.blocks[i - 1] >> rshift);
            }
        }

        // Otherwise nargo fmt will panick
        let res = res;
        (res, carry)
    }

    // Simplify shl_byte function
    fn shl1(self: Self) -> Self {
        let mut res = self;
        let rshift = (32 - 1) as u8;

        res.blocks[0] = self.blocks[0] << 1;
        for i in 1..N {
            res.blocks[i] = {self.blocks[i] << 1} | (self.blocks[i - 1] >> rshift);
        }
        res
    }

    // Shift Left by n bits
    pub fn shl(self: Self, n: u32) -> Self {
        let mut res = BigUint::zero();

        if n < 32 * N {
            let shift_num = n / (32);
            let rem = n % (32);

            for i in 0..N {
                if i >= shift_num {
                    res.blocks[i] = self.blocks[i as u32 - shift_num];
                }
            }
            res = res.shl_block(rem as u8).0;
        }
        res
    }

    fn shr1(self: Self) -> Self {
        let mut res = self;
        let lshift = 31 as u8; // 32-1

        for i in 0..N - 1 {
            res.blocks[i] = (self.blocks[i] >> 1) | (self.blocks[i + 1] << lshift);
        }
        res.blocks[N - 1] = self.blocks[N - 1] >> 1;
        res
    }

    // (quotient, remainder)
    pub fn div(self: Self, other: Self) -> (Self, Self) {
        assert(!BigUint::zero().eq(other));

        if self.lt(other) {
            (BigUint::zero(), self)
        } else {
            let mut rem = self;
            let mut quo = BigUint::zero();

            let bit_diff = self.msb() - other.msb();
            let mut c = other.shl(bit_diff as u32);

            for i in 0..32 * N + 1 {
                if i <= bit_diff as u32 {
                    if rem.lt(c) {
                        quo = quo.shl1();
                    } else {
                        rem = rem.sub(c);
                        quo = quo.shl1().add(BigUint::one());
                    }
                    c = c.shr1();
                }
            }
            (quo, rem)
        }
    }

    // Use simple binary search algorithm
    // https://programmercave.com/blog/2023/03/03/Efficiently-Finding-the-Square-Root-of-a-Number-Linear-Search-vs-Binary-Search
    pub fn sqrt(self: Self) -> Self {
        let mut mid = self;
        let zero = BigUint::zero();
        let one  = BigUint::one();

        if self.eq(zero) {
            mid
        } else {
            let mut beg = zero;
            let mut end = mid;
            let mut done = false;

            // Max number of bits
            for _ in 0..32 * N {
                if !done {
                    // shr1 replace .div(2) operation
                    mid = beg.add(end).shr1();
                    let (low, high) = mid.mul(mid);

                    if (!high.eq(zero) | low.gt(self)) {
                        end = mid.sub(one);
                    } else if (low.lt(self)) {
                        beg = mid.add(one);
                    } else {
                        done = true;
                        beg = mid;
                    }
                }
            }
            if end.gt(beg) { beg } else { end }
        }
    }

    pub fn eq(self: Self, other: Self) -> bool {
        self.blocks == other.blocks
    }

    pub fn gt(self: Self, other: Self) -> bool {
        let (diff, borrow) = self.sbb(other);
        (borrow == 0) & !diff.eq(BigUint::zero())
    }

    pub fn gte(self: Self, other: Self) -> bool {
        self.sbb(other).1 == 0
    }

    pub fn lt(self: Self, other: Self) -> bool {
        other.gt(self)
    }
}
