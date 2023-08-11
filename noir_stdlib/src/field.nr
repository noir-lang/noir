
impl Field {
    #[builtin(to_le_bits)]
    fn to_le_bits(_x : Field, _bit_size: u32) -> [u1] {}
    #[builtin(to_be_bits)]
    fn to_be_bits(_x : Field, _bit_size: u32) -> [u1] {}

    fn to_le_bytes(x : Field, byte_size: u32) -> [u8] {
        x.to_le_radix(256, byte_size)
    }
    fn to_be_bytes(x : Field, byte_size: u32) -> [u8] {
        x.to_be_radix(256, byte_size)
    }

    #[builtin(to_le_radix)]
    //decompose _x into a _result_len vector over the _radix basis
    //_radix must be less than 256
    fn to_le_radix(_x : Field, _radix: u32, _result_len: u32) -> [u8] {}
    #[builtin(to_be_radix)]
    fn to_be_radix(_x : Field, _radix: u32, _result_len: u32) -> [u8] {}

    // Returns self to the power of the given exponent value.
    // Caution: we assume the exponent fits into 32 bits
    // using a bigger bit size impacts negatively the performance and should be done only if the exponent does not fit in 32 bits
    fn pow_32(self, exponent: Field) -> Field {
        let mut r: Field = 1;
        let b = exponent.to_le_bits(32);

        for i in 1..33 {
            r *= r;
            r = (b[32-i] as Field) * (r * self) + (1 - b[32-i] as Field) * r;
        }
        r
    }

    // Parity of (prime) Field element, i.e. sgn0(x mod p) = 0 if x ∈ {0, ..., p-1} is even, otherwise sgn0(x mod p) = 1.
    fn sgn0(self) -> u1 {
        self as u1
    }
}

#[builtin(modulus_num_bits)]
fn modulus_num_bits() -> Field {}

#[builtin(modulus_be_bits)]
fn modulus_be_bits() -> [u1] {}

#[builtin(modulus_le_bits)]
fn modulus_le_bits() -> [u1] {}

#[builtin(modulus_be_bytes)]
fn modulus_be_bytes() -> [u8] {}

#[builtin(modulus_le_bytes)]
fn modulus_le_bytes() -> [u8] {}
