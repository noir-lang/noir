# BigUint56 API

## Overview

The `BigUint56` structure represents a large unsigned integer using a fixed number of limbs. Each limb is a 56-bit unsigned integer.

## Constants

### `BITS_PER_LIMB`
Number of bits per limb.
- Type: `comptime Field`
- Value: `56`

### `NUM_LIMBS`
Number of limbs.
- Type: `comptime Field`
- Value: `5`

### `BYTES_PER_LIMB`
Number of bytes per limb, which is calculated as `BITS_PER_LIMB / 8`.
- Type: `comptime Field`
- Value: `7`

### `MAX_BITS`
Maximum number of bits, calculated as `BITS_PER_LIMB * NUM_LIMBS`.
- Type: `comptime Field`
- Value: `280`

### `MAX_BYTES`
Maximum number of bytes, calculated as `NUM_LIMBS * BYTES_PER_LIMB`.
- Type: `comptime Field`
- Value: `35`

## Structures

### `BigUint56`
This structure represents a large unsigned integer with 56-bit limbs.

- Field:
    - `limbs: [u56; NUM_LIMBS]` - An array of 56-bit unsigned integers.

## Methods

### `zero() -> Self`
Creates a new `BigUint56` initialized to zero.
- Returns: An instance of `BigUint56` with all limbs set to zero.

### `one() -> Self`
Creates a new `BigUint56` initialized to one.
- Returns: An instance of `BigUint56` with the first limb set to one.

### `from_u56(val: u56) -> Self`
Constructs a `BigUint56` from a single `u56` value.
- Parameters:
    - `val: u56` - A 56-bit unsigned integer.
- Returns: An instance of `BigUint56` with the first limb set to `val`.

### `from_bytes(bytes: [u8]) -> Self`
Constructs a `BigUint56` from a byte array in little-endian format.
- Parameters:
    - `bytes: [u8]` - A byte array.
- Returns: An instance of `BigUint56` converted from the provided byte array.

### `to_bytes(self: Self) -> [u8; MAX_BYTES]`
Returns the little-endian byte array representation of the `BigUint56`.
- Returns: A byte array representation of the `BigUint56`.

### `to_bits(self: Self) -> [u1; MAX_BITS]`
Returns the bit array representation of the `BigUint56`, with LSB at index 0.
- Returns: A bit array representation of the `BigUint56`.

### `adc(self: Self, other: Self) -> (Self, u56)`
Adds two `BigUint56` numbers with carry.
- Parameters:
    - `other: Self` - Another `BigUint56` instance.
- Returns: A tuple containing the result and the carry.

### `add(self: Self, other: Self) -> Self`
Adds two `BigUint56` instances without returning the carry.
- Parameters:
    - `other: Self` - Another `BigUint56` instance.
- Returns: The result of the addition.

### `sbb(self: Self, other: Self) -> (Self, u56)`
Subtracts two `BigUint56` numbers with borrow.
- Parameters:
    - `other: Self` - Another `BigUint56` instance.
- Returns: A tuple containing the result and the borrow.

### `sub(self: Self, other: Self) -> Self`
Subtracts two `BigUint56` instances without returning the borrow.
- Parameters:
    - `other: Self` - Another `BigUint56` instance.
- Returns: The result of the subtraction.

### `mul(self: Self, other: Self) -> (Self, Self)`
Multiplies two `BigUint56` instances using long multiplication.

- Parameters:
    - `other: Self` - Another `BigUint56` instance.
- Returns: A tuple containing the lower and higher parts of the multiplication result.

### `shl_limb(self: Self, n: u56) -> (Self, u56)`
Shifts the `BigUint56` instance to the left by a specified number of bits `n`.

- Parameters:
    - `n: u56` - Number of bits to shift to the left.
- Returns: A tuple containing the shifted result and the carry.

### `shl1(self: Self) -> Self`
Shifts the `BigUint56` instance to the left by 1 bit.

- Returns: The shifted result.

### `shl(self: Self, n: u56) -> Self`
Shifts the `BigUint56` instance to the left by a specified number of bits `n`.

- Parameters:
    - `n: u56` - Number of bits to shift to the left.
- Returns: The shifted result.

### `shr_limb(self: Self, n: u56) -> Self`
Shifts the `BigUint56` instance to the right by a specified number of bits `n`.

- Parameters:
    - `n: u56` - Number of bits to shift to the right.
- Returns: The shifted result.

### `shr1(self: Self) -> Self`
Shifts the `BigUint56` instance to the right by 1 bit.

- Returns: The shifted result.

### `shr(self: Self, n: u56) -> Self`
Shifts the `BigUint56` instance to the right by a specified number of bits.

- Parameters:
    - `n: u56` - Number of bits to shift to the right.
- Returns: The shifted result.

### `nbits(self: Self) -> u56`
Returns the number of bits needed to represent the `BigUint56` instance.

- Returns: Number of bits.

### `div(self: Self, other: Self) -> (Self, Self)`
Divides the `BigUint56` instance by another, returning the quotient and remainder using long division.

- Parameters:
    - `other: Self` - The divisor `BigUint56` instance.
- Returns: A tuple containing the quotient and the remainder.

### `eq(self: Self, other: Self) -> bool`
Compares two `BigUint56` instances for equality.
- Parameters:
    - `other: Self` - Another `BigUint56` instance.
- Returns: `true` if both instances are equal, `false` otherwise.

### `gte(self: Self, other: Self) -> bool`
Checks if a `BigUint56` instance is greater than or equal to another.
- Parameters:
    - `other: Self` - Another `BigUint56` instance to compare against.
- Returns: `true` if the instance is greater than or equal to `other`, `false` otherwise.

### `gt(self: Self, other: Self) -> bool`
Checks if a `BigUint56` instance is strictly greater than another.
- Parameters:
    - `other: Self` - Another `BigUint56` instance to compare against.
- Returns: `true` if the instance is strictly greater than `other`, `false` otherwise.

### `lte(self: Self, other: Self) -> bool`
Checks if a `BigUint56` instance is less than or equal to another.
- Parameters:
    - `other: Self` - Another `BigUint56` instance to compare against.
- Returns: `true` if the instance is less than or equal to `other`, `false` otherwise.

### `lt(self: Self, other: Self) -> bool`
Checks if a `BigUint56` instance is strictly less than another.
- Parameters:
    - `other: Self` - Another `BigUint56` instance to compare against.
- Returns: `true` if the instance is strictly less than `other`, `false` otherwise.

### `is_zero(self: Self) -> bool`
Checks if the `BigUint56` instance is equal to zero.
- Returns: A boolean value indicating if the `BigUint56` instance is zero.

### `add_mod(self: Self, other: Self, modulus: Self) -> Self`
Adds two `BigUint56` instances with modulo operation.
- Parameters:
    - `other: Self` - Another `BigUint56` instance.
    - `modulus: Self` - The modulus value to use for the operation.
- Returns: The result of the addition modulo operation.

### `println(self: Self)`
Prints the `BigUint56` instance.
