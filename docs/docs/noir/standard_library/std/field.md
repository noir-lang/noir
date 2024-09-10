# Module `std::field`

## `Field` methods

### assert_max_bit_size

```noir
fn assert_max_bit_size(self, bit_size: u32)
```

Asserts that `self` can be represented in `bit_size` bits.

# Failures
Causes a constraint failure for `Field` values exceeding `2^{bit_size}`.

### __assert_max_bit_size

```noir
fn __assert_max_bit_size(self, bit_size: u32)
```

### to_le_bits

```noir
fn to_le_bits<let N: u32>(self) -> [u1; N]
```

Decomposes `self` into its little endian bit decomposition as a `[u1; N]` array.
This slice will be zero padded should not all bits be necessary to represent `self`.

# Failures
Causes a constraint failure for `Field` values exceeding `2^N` as the resulting slice will not
be able to represent the original `Field`.

# Safety
Values of `N` equal to or greater than the number of bits necessary to represent the `Field` modulus
(e.g. 254 for the BN254 field) allow for multiple bit decompositions. This is due to how the `Field` will
wrap around due to overflow when verifying the decomposition.

### to_be_bits

```noir
fn to_be_bits<let N: u32>(self) -> [u1; N]
```

Decomposes `self` into its big endian bit decomposition as a `[u1; N]` array.
This array will be zero padded should not all bits be necessary to represent `self`.

# Failures
Causes a constraint failure for `Field` values exceeding `2^N` as the resulting slice will not
be able to represent the original `Field`.

# Safety
Values of `N` equal to or greater than the number of bits necessary to represent the `Field` modulus
(e.g. 254 for the BN254 field) allow for multiple bit decompositions. This is due to how the `Field` will
wrap around due to overflow when verifying the decomposition.

### to_le_bytes

```noir
fn to_le_bytes<let N: u32>(self) -> [u8; N]
```

Decomposes `self` into its little endian byte decomposition as a `[u8]` slice of length `byte_size`.
This slice will be zero padded should not all bytes be necessary to represent `self`.

# Failures
Causes a constraint failure for `Field` values exceeding `2^{8*byte_size}` as the resulting slice will not
be able to represent the original `Field`.

# Safety
Values of `byte_size` equal to or greater than the number of bytes necessary to represent the `Field` modulus
(e.g. 32 for the BN254 field) allow for multiple byte decompositions. This is due to how the `Field` will
wrap around due to overflow when verifying the decomposition.

### to_be_bytes

```noir
fn to_be_bytes<let N: u32>(self) -> [u8; N]
```

Decomposes `self` into its big endian byte decomposition as a `[u8]` slice of length `byte_size`.
This slice will be zero padded should not all bytes be necessary to represent `self`.

# Failures
Causes a constraint failure for `Field` values exceeding `2^{8*byte_size}` as the resulting slice will not
be able to represent the original `Field`.

# Safety
Values of `byte_size` equal to or greater than the number of bytes necessary to represent the `Field` modulus
(e.g. 32 for the BN254 field) allow for multiple byte decompositions. This is due to how the `Field` will
wrap around due to overflow when verifying the decomposition.

### to_le_radix

```noir
fn to_le_radix<let N: u32>(self, radix: u32) -> [u8; N]
```

### to_be_radix

```noir
fn to_be_radix<let N: u32>(self, radix: u32) -> [u8; N]
```

### __to_le_radix

```noir
fn __to_le_radix<let N: u32>(self, radix: u32) -> [u8; N]
```

### __to_be_radix

```noir
fn __to_be_radix<let N: u32>(self, radix: u32) -> [u8; N]
```

### pow_32

```noir
fn pow_32(self, exponent: Field) -> Field
```

### sgn0

```noir
fn sgn0(self) -> u1
```

### lt

```noir
fn lt(self, another: Field) -> bool
```

## modulus_num_bits

```noir
fn modulus_num_bits() -> u64
```

## modulus_be_bits

```noir
fn modulus_be_bits() -> [u1]
```

## modulus_le_bits

```noir
fn modulus_le_bits() -> [u1]
```

## modulus_be_bytes

```noir
fn modulus_be_bytes() -> [u8]
```

## modulus_le_bytes

```noir
fn modulus_le_bytes() -> [u8]
```

## bytes32_to_field

```noir
fn bytes32_to_field(bytes32: [u8; 32]) -> Field
```

