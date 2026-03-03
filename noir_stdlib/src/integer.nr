impl u1 {
    /// Returns the maximum value representable by `u1`: 1.
    pub fn max_value() -> u1 {
        1
    }

    /// Returns the minimum value representable by `u1`: 0.
    pub fn min_value() -> u1 {
        0
    }

    /// Returns the number of bits in `u1`: 1.
    pub fn bits() -> u32 {
        1
    }
}

impl u8 {
    /// Returns the maximum value representable by `u8`: 255.
    pub fn max_value() -> u8 {
        255
    }

    /// Returns the minimum value representable by `u8`: 0.
    pub fn min_value() -> u8 {
        0
    }

    /// Returns the number of bits in `u8`: 8.
    pub fn bits() -> u32 {
        8
    }
}

impl u16 {
    /// Returns the maximum value representable by `u16`: 65535.
    pub fn max_value() -> u16 {
        65535
    }

    /// Returns the minimum value representable by `u16`: 0.
    pub fn min_value() -> u16 {
        0
    }

    /// Returns the number of bits in `u16`: 16.
    pub fn bits() -> u32 {
        16
    }
}

impl u32 {
    /// Returns the maximum value representable by `u32`: 4294967295.
    pub fn max_value() -> u32 {
        4294967295
    }

    /// Returns the minimum value representable by `u32`: 0.
    pub fn min_value() -> u32 {
        0
    }

    /// Returns the number of bits in `u32`: 32.
    pub fn bits() -> u32 {
        32
    }
}

impl u64 {
    /// Returns the maximum value representable by `u64`: 18446744073709551615.
    pub fn max_value() -> u64 {
        18446744073709551615
    }

    /// Returns the minimum value representable by `u64`: 0.
    pub fn min_value() -> u64 {
        0
    }

    /// Returns the number of bits in `u64`: 64.
    pub fn bits() -> u32 {
        64
    }
}

impl i8 {
    /// Returns the maximum value representable by `i8`: 127.
    pub fn max_value() -> i8 {
        127
    }

    /// Returns the minimum value representable by `i8`: -128.
    pub fn min_value() -> i8 {
        -128
    }

    /// Returns the number of bits in `i8`: 8.
    pub fn bits() -> u32 {
        8
    }
}

impl i16 {
    /// Returns the maximum value representable by `i16`: 32767.
    pub fn max_value() -> i16 {
        32767
    }

    /// Returns the minimum value representable by `i16`: -32768.
    pub fn min_value() -> i16 {
        -32768
    }

    /// Returns the number of bits in `i16`: 16.
    pub fn bits() -> u32 {
        16
    }
}

impl i32 {
    /// Returns the maximum value representable by `i32`: 2147483647.
    pub fn max_value() -> i32 {
        2147483647
    }

    /// Returns the minimum value representable by `i32`: -2147483648.
    pub fn min_value() -> i32 {
        -2147483648
    }

    /// Returns the number of bits in `i32`: 32.
    pub fn bits() -> u32 {
        32
    }
}

impl i64 {
    /// Returns the maximum value representable by `i64`: 9223372036854775807.
    pub fn max_value() -> i64 {
        9223372036854775807
    }

    /// Returns the minimum value representable by `i64`: -9223372036854775808.
    pub fn min_value() -> i64 {
        -9223372036854775808
    }

    /// Returns the number of bits in `i64`: 64.
    pub fn bits() -> u32 {
        64
    }
}

mod tests {
    #[test]
    fn test_u1_limits() {
        assert_eq(u1::max_value(), 1);
        assert_eq(u1::min_value(), 0);
        assert_eq(u1::bits(), 1);
    }

    #[test]
    fn test_u8_limits() {
        assert_eq(u8::max_value(), 255);
        assert_eq(u8::min_value(), 0);
        assert_eq(u8::bits(), 8);
    }

    #[test]
    fn test_u16_limits() {
        assert_eq(u16::max_value(), 65535);
        assert_eq(u16::min_value(), 0);
        assert_eq(u16::bits(), 16);
    }

    #[test]
    fn test_u32_limits() {
        assert_eq(u32::max_value(), 4294967295);
        assert_eq(u32::min_value(), 0);
        assert_eq(u32::bits(), 32);
    }

    #[test]
    fn test_u64_limits() {
        assert_eq(u64::max_value(), 18446744073709551615);
        assert_eq(u64::min_value(), 0);
        assert_eq(u64::bits(), 64);
    }

    #[test]
    fn test_i8_limits() {
        assert_eq(i8::max_value(), 127);
        assert_eq(i8::min_value(), -128);
        assert_eq(i8::bits(), 8);
    }

    #[test]
    fn test_i16_limits() {
        assert_eq(i16::max_value(), 32767);
        assert_eq(i16::min_value(), -32768);
        assert_eq(i16::bits(), 16);
    }

    #[test]
    fn test_i32_limits() {
        assert_eq(i32::max_value(), 2147483647);
        assert_eq(i32::min_value(), -2147483648);
        assert_eq(i32::bits(), 32);
    }

    #[test]
    fn test_i64_limits() {
        assert_eq(i64::max_value(), 9223372036854775807);
        assert_eq(i64::min_value(), -9223372036854775808);
        assert_eq(i64::bits(), 64);
    }
}
