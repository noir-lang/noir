# Bugs found in Noir stdlib

## U128

### decode_ascii
Old **decode_ascii** function didn't check that the values of individual bytes in the string were just in the range of [0-9a-f-A-F].
```rust
fn decode_ascii(ascii: u8) -> Field {
    if ascii < 58 {
        ascii - 48
    } else if ascii < 71 {
        ascii - 55
    } else {
        ascii - 87
    } as Field
}
```
Since the function used the assumption that decode_ascii returns values in range [0,15] to construct **lo** and **hi** it was possible to overflow these 64-bit limbs.

### unconstrained_div
```rust
 unconstrained fn unconstrained_div(self: Self, b: U128) -> (U128, U128) {
        if self < b {
            (U128::from_u64s_le(0, 0), self)
        } else {
            //TODO check if this can overflow?
            let (q,r) = self.unconstrained_div(b * U128::from_u64s_le(2, 0));
            let q_mul_2 = q * U128::from_u64s_le(2, 0);
            if r < b {
                (q_mul_2, r)
            } else {
                (q_mul_2 + U128::from_u64s_le(1, 0), r - b)
            }
        }
    }
```
There were 2 issues in unconstrained_div:
1) Attempting to divide by zero resulted in an infinite loop, because there was no check.
2) $a >= 2^{127}$ cause the function to multiply b to such power of 2 that the result would be more than $2^{128}$ and lead to assertion failure even though it was a legitimate input

N.B. initial fix by Rumata888 also had an edgecase missing for when a==b and b >= (1<<127).

### wrapping_mul
```rust
fn wrapping_mul(self: Self, b: U128) -> U128 {
        let low = self.lo * b.lo;
        let lo = low as u64 as Field;
        let carry = (low - lo) / pow64;
        let high = if crate::field::modulus_num_bits() as u32 > 196 {
            (self.lo + self.hi) * (b.lo + b.hi) - low + carry // Bug
        } else {
            self.lo * b.hi + self.hi * b.lo + carry
        };
        let hi = high as u64 as Field;
        U128 { lo, hi }
    }
```
Wrapping mul had the code copied from regular mul barring the assertion that the product of high limbs is zero. Because that check was removed, the optimized path for moduli > 196 bits was incorrect, since it included their product (as at least one of them was supposed to be zero originally, but not for wrapping multiplication)



