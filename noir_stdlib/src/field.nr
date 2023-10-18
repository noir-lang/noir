
impl Field {
    pub fn to_le_bits(self: Self, bit_size: u32) -> [u1] {
        crate::assert_constant(bit_size);
        self.__to_le_bits(bit_size)
    }
    
    pub fn to_be_bits(self: Self, bit_size: u32) -> [u1] {
        crate::assert_constant(bit_size);
        self.__to_be_bits(bit_size)
    }

    #[builtin(to_le_bits)]
    fn __to_le_bits(_self: Self, _bit_size: u32) -> [u1] {}
    
    #[builtin(to_be_bits)]
    fn __to_be_bits(_self: Self, _bit_size: u32) -> [u1] {}

    pub fn to_le_bytes(self: Self, byte_size: u32) -> [u8] {
        self.to_le_radix(256, byte_size)
    }

    pub fn to_be_bytes(self: Self, byte_size: u32) -> [u8] {
        self.to_be_radix(256, byte_size)
    }


    pub fn to_le_radix(self: Self, radix: u32, result_len: u32) -> [u8] {
        crate::assert_constant(radix);
        crate::assert_constant(result_len);
        self.__to_le_radix(radix, result_len)
    }

    pub fn to_be_radix(self: Self, radix: u32, result_len: u32) -> [u8] {
        crate::assert_constant(radix);
        crate::assert_constant(result_len);
        self.__to_be_radix(radix, result_len)
    }



    // decompose `_self` into a `_result_len` vector over the `_radix` basis
    // `_radix` must be less than 256
    #[builtin(to_le_radix)]
    fn __to_le_radix(_self: Self, _radix: u32, _result_len: u32) -> [u8] {}
    
    #[builtin(to_be_radix)]
    fn __to_be_radix(_self: Self, _radix: u32, _result_len: u32) -> [u8] {}


    // Returns self to the power of the given exponent value.
    // Caution: we assume the exponent fits into 32 bits
    // using a bigger bit size impacts negatively the performance and should be done only if the exponent does not fit in 32 bits
    pub fn pow_32(self, exponent: Field) -> Field {
        let mut r: Field = 1;
        let b = exponent.to_le_bits(32);

        for i in 1..33 {
            r *= r;
            r = (b[32-i] as Field) * (r * self) + (1 - b[32-i] as Field) * r;
        }
        r
    }

    // Parity of (prime) Field element, i.e. sgn0(x mod p) = 0 if x ∈ {0, ..., p-1} is even, otherwise sgn0(x mod p) = 1.
    pub fn sgn0(self) -> u1 {
        self as u1
    }
}

#[builtin(modulus_num_bits)]
pub fn modulus_num_bits() -> Field {}

#[builtin(modulus_be_bits)]
pub fn modulus_be_bits() -> [u1] {}

#[builtin(modulus_le_bits)]
pub fn modulus_le_bits() -> [u1] {}

#[builtin(modulus_be_bytes)]
pub fn modulus_be_bytes() -> [u8] {}

#[builtin(modulus_le_bytes)]
pub fn modulus_le_bytes() -> [u8] {}
