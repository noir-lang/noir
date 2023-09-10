use dep::std::println;

mod utils;

// NOTE: this file is forked from https://github.com/shuklaayush/noir-bigint v0.1.0
// Once Noir supports const generics, we can use the original crate with our custom limbs instead
//
// CHANGELOG:
// - Updated constants for a max 2048 bit RSA. The BigInt lib only supports 5 limbs
// - Added mulmod and powmod functions
// - Use unconstrained division for powmod

// Top-level constants related to the size of BigUint56 limbs and bytes.
global BITS_PER_LIMB: Field = 56; /// Number of bits per limb.
global NUM_LIMBS: Field = 37;      /// Number of limbs.

global BYTES_PER_LIMB: Field = 7; /// Number of bytes per limb (BITS_PER_LIMB / 8).
global MAX_BITS: Field = 2072;     /// Maximum number of bits (BITS_PER_LIMB * NUM_LIMBS).
global MAX_BYTES: Field = 259;     /// Maximum number of bytes (NUM_LIMBS * BYTES_PER_LIMB).

// TODO/NOTES:
// 1. Noir doesn't support expressions on globals so these are hardcoded
// 2. Noir doesn't automatically add `comptime` to all globals, hence strongly typed
// 3. Noir doesn't support const generics, so we can't have a generic limb type
//
// The ideal implementation would be with a generic limb type `T`, but Noir 
// doesn't support const generics so this is non-trivial to implement.
//  struct BigUint<T, NUM_LIMBS> {
//      limbs : [T; NUM_LIMBS],
//  }

/// A structure representing a large unsigned integer using a fixed number of limbs.
/// Each limb is a 56-bit unsigned integer. 
/// 1. 56 is divisible by 8, making byte conversion easier.
/// 2. Multiplication requires a double width value, and u112 is the maximum representable in Noir.
struct BigUint56 {
    limbs: [u56; NUM_LIMBS],
}

impl BigUint56 {
    /// Creates a new BigUint56 initialized to zero.
    fn zero() -> Self {
        Self { limbs: [0 as u56; NUM_LIMBS] }
    }

    /// Creates a new BigUint56 initialized to one.
    fn one() -> Self {
        let mut one = BigUint56::zero();
        one.limbs[0] = 1;
        one
    }
    
    /// Constructs a BigUint56 from a single `u56` value.
    fn from_u56(val: u56) -> Self {
        let mut buint = BigUint56::zero();
        buint.limbs[0] = val;
        buint
    }

    /// Constructs a BigUint56 from a byte array in little-endian format.
    fn from_bytes(bytes: [u8]) -> Self {
        // TODO: Do I need this check?
        assert(bytes.len() as u56 <= MAX_BYTES as u56);

        let mut res = BigUint56::zero();
        for i in 0..bytes.len() {
            let limb_index = (i as u56) / (BYTES_PER_LIMB as u56);
            let byte_index = (i as u56) % (BYTES_PER_LIMB as u56);

            res.limbs[limb_index as Field] |= (bytes[i] as u56) << (byte_index * 8);
        }

        res
    }

    /// Returns the little-endian byte array representation of the BigUint56.
    fn to_bytes(self: Self) -> [u8; MAX_BYTES] {
        let mut res = [0 as u8; MAX_BYTES];
    
        for i in 0..NUM_LIMBS {
            let limb_bytes = (self.limbs[i] as Field).to_le_bytes(BYTES_PER_LIMB as u32);
            for j in 0..BYTES_PER_LIMB {
                let idx = i * BYTES_PER_LIMB + j;
                res[idx] = limb_bytes[j as Field];
            }
        }
    
        res
    }

    /// Returns the bit array representation of the BigUint56, with LSB at index 0.
    fn to_bits(self: Self) -> [u1; MAX_BITS] {
        let mut res = [0 as u1; MAX_BITS];
    
        for i in 0..NUM_LIMBS {
            let limb_bits = (self.limbs[i] as Field).to_le_bits(BITS_PER_LIMB as u32);
            for j in 0..BITS_PER_LIMB {
                let idx = i * (BITS_PER_LIMB as Field) + (j as Field);
                res[idx] = limb_bits[j as Field];
            }
        }
    
        res
    }

    /// Adds two BigUint56 numbers with carry.
    /// Returns a tuple containing the result and the carry.
    fn adc(self: Self, other: Self) -> (Self, u56) {
        let mut res = BigUint56::zero();
        let mut carry = 0 as u56;

        for i in 0..NUM_LIMBS {
            let (sum, new_carry) = utils::adc(self.limbs[i], other.limbs[i], carry);
            res.limbs[i] = sum;
            carry = new_carry;
        };

        (res, carry)
    }

    /// Adds two BigUint56 instances without returning the carry.
    // TODO: Check if carry is 0?
    fn add(self: Self, other: Self) -> Self {
        let (res, _carry) = self.adc(other);
        res
    }

    /// Subtracts two BigUint56 numbers with borrow.
    /// Returns a tuple containing the result and the borrow.
    fn sbb(self: Self, other: Self) -> (Self, u56) {
        let mut res = BigUint56::zero();
        let mut borrow = 0 as u56;

        for i in 0..NUM_LIMBS {
            let (diff, new_borrow) = utils::sbb(self.limbs[i], other.limbs[i], borrow);
            res.limbs[i] = diff;
            borrow = new_borrow;
        };
        
        (res, borrow)
    }

    /// Subtracts two BigUint56 instances without returning the borrow.
    // TODO: Check if borrow is 0?
    fn sub(self: Self, other: Self) -> Self {
        let (res, _borrow) = self.sbb(other);
        res
    }

    /// Multiplies two BigUint56 instances using long multiplication.
    /// Returns a tuple containing the lower and higher parts of the multiplication result.
    fn mul(self: Self, other: Self) -> (Self, Self) {
        let mut lo = BigUint56::zero();
        let mut hi = BigUint56::zero();

        for i in 0..NUM_LIMBS {
            let mut carry = 0 as u56;

            for j in 0..NUM_LIMBS {
                let k = i + j;

                if k as u56 >= NUM_LIMBS as u56 {
                    let (n, c) = utils::mac(hi.limbs[k - NUM_LIMBS], self.limbs[i], other.limbs[j], carry);
                    hi.limbs[k - NUM_LIMBS] = n;
                    carry = c;
                } else {
                    let (n, c) = utils::mac(lo.limbs[k], self.limbs[i], other.limbs[j], carry);
                    lo.limbs[k] = n;
                    carry = c;
                }
            };

            hi.limbs[i] = carry;
        };

        (lo, hi)
    }

    /// Shifts the BigUint56 instance to the left by a specified number of bits `n`.
    /// where `0 <= n < Limb::BITS`,
    /// Returns the shifted result and the carry.
    // TODO: Should I return early if n == 0?
    fn shl_limb(self: Self, n: u56) -> (Self, u56) {
        assert(n < BITS_PER_LIMB as u56);

        let mut res = BigUint56::zero();

        let rshift = BITS_PER_LIMB as u56 - n;
        let carry = self.limbs[NUM_LIMBS - 1] >> rshift;

        res.limbs[0] = self.limbs[0] << n;
        for i in 1..NUM_LIMBS {
            res.limbs[i] = (self.limbs[i] << n) | (self.limbs[i - 1] >> rshift);
        // BUG: Panics without this semi-colon.
        };

        (res, carry)
    }

    /// Shifts the BigUint56 instance to the left by 1 bit.
    fn shl1(self: Self) -> Self {
        let (res, _carry) = self.shl_limb(1);
        res
    }

    /// Shifts the BigUint56 instance to the left by a specified number of bits `n`.
    // TODO: Should I return early if n == 0?
    fn shl(self: Self, n: u56) -> Self {
        let mut res = BigUint56::zero();

        if n < MAX_BITS as u56 {
            let shift_num = n / (BITS_PER_LIMB as u56);
            let rem = n % (BITS_PER_LIMB as u56);

            // for i in shift_num..NUM_LIMBS {
            for i in 0..NUM_LIMBS {
                if i as u56 >= shift_num {
                    // BUG: This line panics with Expected array index to fit in u64
                    // res.limbs[i] = self.limbs[i - shift_num as Field];
                    res.limbs[i as u56] = self.limbs[i as u56 - shift_num];
                }
            }

            let (new_lower, _carry) = res.shl_limb(rem);
            res = new_lower;
        }

        res
    }

    /// Shifts the BigUint56 instance to the right by a specified number of bits `n`.
    /// where `0 <= n < Limb::BITS`,
    // TODO: Should I return early if n == 0?
    fn shr_limb(self: Self, n: u56) -> Self {
        assert(n < BITS_PER_LIMB as u56);

        let mut res = BigUint56::zero();
        let lshift = BITS_PER_LIMB as u56 - n;

        for i in 0..NUM_LIMBS-1 {
            res.limbs[i] = (self.limbs[i] >> n) | (self.limbs[i + 1] << lshift);
        }
        res.limbs[NUM_LIMBS - 1] = self.limbs[NUM_LIMBS - 1] >> n;

        res
    }

    /// Shifts the BigUint56 instance to the right by 1 bit.
    fn shr1(self: Self) -> Self {
        let res = self.shr_limb(1);
        res
    }

    /// Shifts the BigUint56 instance to the right by a specified number of bits.
    // TODO: Should I return early if n == 0?
    fn shr(self: Self, n: u56) -> Self {
        let mut res = BigUint56::zero();

        if n < MAX_BITS as u56 {
            let shift_num = n / (BITS_PER_LIMB as u56);
            let rem = n % (BITS_PER_LIMB as u56);

            // for i in 0..shift_num {
            for i in 0..NUM_LIMBS {
                if i as u56 + shift_num < NUM_LIMBS as u56 {
                    res.limbs[i] = self.limbs[i + shift_num as Field];
                }
            }

            res = res.shr_limb(rem);
        }

        res
    }

    /// Returns the number of bits needed to represent the BigUint56 instance.
    fn nbits(self: Self) -> u56 {
        let bits = BigUint56::to_bits(self);
        let mut res = 0;
        let mut done = false;
        for i in 0..MAX_BITS {
            if !done {
                if bits[MAX_BITS - i - 1] != 0 {
                    res = (MAX_BITS - i - 1) as u56 + 1;
                    done = true;
                }
            }
        }

        res
    }


    /// Divides the BigUint56 instance by another, returning the quotient and remainder using long division.
    // WARNING: This is a simple binary long division. More efficient algorithms should be considered.
    // TODO: Maybe https://github.com/okuyiga/noir-bigint/blob/d60cc5246c8b0d175c4d6b1f4aaceed7fb725695/bigint/src/division.nr
    unconstrained fn div(self: Self, other: Self) -> (Self, Self) {
        assert(!other.is_zero());

        if self.lt(other) {
            (BigUint56::zero(), self)
        } else {
            let mut rem = self;
            let mut quo = BigUint56::zero();

            let bit_diff = self.nbits() - other.nbits();
            let mut c = other.shl(bit_diff);

            for i in 0..MAX_BITS+1 {
                if i as u56 <= bit_diff {
                    if rem.gte(c) {
                        rem = rem.sub(c);
                        quo = quo.shl1().add(BigUint56::one());
                    } else {
                        quo = quo.shl1();
                    }
                    c = c.shr1();
                }
            };

            (quo, rem)
        }
    }

    /// Checks if the two BigUint56 instances are equal.
    fn eq(self: Self, other: Self) -> bool {
        let mut is_eq = true;
        for i in 0..NUM_LIMBS {
            is_eq = is_eq & (self.limbs[i] == other.limbs[i]);
        }
        is_eq
    }

    /// Checks if the BigUint56 instance is greater than or equal to another.
    fn gte(self: Self, other: Self) -> bool {
        let (_diff, borrow) = self.sbb(other);
        borrow == 0
    }

    /// Checks if the BigUint56 instance is strictly greater than another.
    fn gt(self: Self, other: Self) -> bool {
        let (diff, borrow) = self.sbb(other);
        (borrow == 0) & !diff.eq(BigUint56::zero())
    }

    /// Checks if the BigUint56 instance is less than or equal to another.
    fn lte(self: Self, other: Self) -> bool {
        other.gte(self)
    }

    /// Checks if the BigUint56 instance is strictly less than another.
    fn lt(self: Self, other: Self) -> bool {
        other.gt(self)
    }

    /// Checks if the BigUint56 instance is zero.
    fn is_zero(self: Self) -> bool {
        self.eq(BigUint56::zero())
    }

    /// Returns self + other % modulus.
    /// Assumes `self + other` as unbounded integer is `< 2*modulus`.
    fn addmod(self: Self, other: Self, modulus: Self) -> Self {
        let (sum1, carry) = self.adc(other);

        // Attempt to subtract the modulus, to ensure the result is in the field.
        let (sum2, borrow1) = sum1.sbb(modulus);
        let (_diff, borrow2) = utils::sbb(carry, 0 as u56, borrow1);

        if borrow2 == 0 {
            sum2
        } else {
            sum2.add(modulus)
        }
    }
    
    // Returns self * other % modulus.
    fn mulmod(self: Self, other: Self, modulus: Self) -> Self {
        // We assume that the mul is < 2^MAX_BITS, so we can just mod the low part.
        let (lo, _hi) = self.mul(other);
        assert(_hi.is_zero());

        let (quo, rem) = lo.div_unsafe(modulus);
        let (quo_modulus, _hi) = quo.mul(modulus);
        let reconstructed_lo = quo_modulus.add(rem);

        // Range check to prevent malicious prover from assigning random values for quo and rem
        assert(rem.lt(modulus));
        assert(quo.lte(lo));

        // Constrain that quo * mod + rem == original
        assert(reconstructed_lo.eq(lo));

        rem
    }

    unconstrained fn div_unsafe(self: Self, other: Self) -> (Self, Self) {
        self.div(other)
    }

    // Returns self * self % modulus.
    fn squaremod(self: Self, modulus: Self) -> Self {
        let rem = self.mulmod(self, modulus);
        
        rem
    }

    // Returns self ^ e % modulus.
    fn powmod(self: Self, e: Self, modulus: Self) -> Self {
        // Check e is less than 2^17, since function will only iterate through first 17 bits
        // Enable the most common exponents e.g. 65537
        assert(e.lt(BigUint56::from_u56(131072 as u56)));
        let e_bits = e.to_bits();

        let mut res = BigUint56::one();
        let mut pow = self;

        // TODO: Enable e up to MAX_BITS for generalization
        for i in 0..1 {           
            pow = pow.mulmod(pow, modulus);
        }

        res
    }

    // Print as bytes
    fn println(self: Self) {
        let bytes = self.to_bytes();
        println(bytes);
    }
}
