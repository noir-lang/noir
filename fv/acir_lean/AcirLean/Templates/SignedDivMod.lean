/-
PINNED: no review needed. These functions are checked byte-for-byte against
the optimized circuits `nargo compile` produces for signed `div` and `mod`
(`expand_signed_math`, ACIR generation, `acvm::compiler::optimize`), for the
signed widths 8, 16, 32 and 64 (`templates.golden`, `fv_templates.rs`).
`scripts/check.sh` allows only plain definitions in this directory.
-/

import AcirLean.Templates.Gadgets

/-!
Witnesses, with `N = 2^(n-1)`: `0 = a, 1 = b, 2 = result`, `3, 4` the
`a == N` check (`z`, result), `5, 6` the `b == 2^n - 1` check, `7, 8` and
`9, 10` the sign bits and low parts of `a` and `b`, `11` the inverse of `|b|`,
`12 = |b|`, `13, 14, 15` the unsigned division of `|a|` by `|b|`
(quotient, remainder, `|b| - r - 1`). For `div`: `16 = N - q`, `17` the result
sign, `18, 19` the `q == 0` check, `20` the signed quotient, `21 = [q != 0]`.
For `mod`: `16, 17` the `r == 0` check, `18` the signed remainder,
`19 = [r != 0]`.
-/

namespace AcirLean

/-- The constraints `div` and `mod` share: the overflow check, the sign bits,
`|b|`, and the unsigned division of `|a|` by `|b|`. -/
def signedDivModShared (n : ℕ) : List Opcode :=
  [ .range 0 n, .range 1 n,
    .assertZero [⟨1, []⟩, ⟨-1, [0, 3]⟩, ⟨(2 ^ (n - 1) : ℕ), [3]⟩, ⟨-1, [4]⟩],
    .assertZero [⟨1, [0, 4]⟩, ⟨-(2 ^ (n - 1) : ℕ), [4]⟩],
    .assertZero [⟨1, []⟩, ⟨-1, [1, 5]⟩, ⟨(2 ^ n - 1 : ℕ), [5]⟩, ⟨-1, [6]⟩],
    .assertZero [⟨1, [1, 6]⟩, ⟨-(2 ^ n - 1 : ℕ), [6]⟩],
    .assertZero [⟨1, [4, 6]⟩],
    .range 7 1, .range 8 (n - 1),
    .assertZero [⟨1, [0]⟩, ⟨-(2 ^ (n - 1) : ℕ), [7]⟩, ⟨-1, [8]⟩],
    .range 9 1, .range 10 (n - 1),
    .assertZero [⟨1, [1]⟩, ⟨-(2 ^ (n - 1) : ℕ), [9]⟩, ⟨-1, [10]⟩],
    .assertZero [⟨1, [1]⟩, ⟨-2, [1, 9]⟩, ⟨(2 ^ n : ℕ), [9]⟩, ⟨-1, [12]⟩],
    .assertZero [⟨1, []⟩, ⟨-1, [11, 12]⟩],
    .range 13 n, .range 14 n,
    .assertZero [⟨1, []⟩, ⟨-1, [12]⟩, ⟨1, [14]⟩, ⟨1, [15]⟩],
    .range 15 n,
    .assertZero [⟨1, [0]⟩, ⟨-2, [0, 7]⟩, ⟨(2 ^ n : ℕ), [7]⟩, ⟨-1, [12, 13]⟩, ⟨-1, [14]⟩] ]

/-- `fn main(v0: i<n>, v1: i<n>) -> i<n> { v0 / v1 }`, as shipped. -/
def shippedSignedDiv (n : ℕ) : Circuit where
  opcodes := signedDivModShared n ++
    [ .assertZero [⟨(2 ^ (n - 1) : ℕ), []⟩, ⟨-1, [13]⟩, ⟨-1, [16]⟩],
      .assertZero [⟨1, [7]⟩, ⟨-2, [7, 9]⟩, ⟨1, [9]⟩, ⟨-1, [17]⟩],
      .assertZero [⟨1, []⟩, ⟨-1, [13, 18]⟩, ⟨-1, [19]⟩],
      .assertZero [⟨1, [13, 19]⟩],
      .assertZero [⟨1, [13]⟩, ⟨2, [16, 17]⟩, ⟨-1, [20]⟩],
      .assertZero [⟨1, []⟩, ⟨-1, [19]⟩, ⟨-1, [21]⟩],
      .assertZero [⟨1, [2]⟩, ⟨-1, [20, 21]⟩] ]
  parameters := [0, 1]
  returnValues := [2]

/-- `fn main(v0: i<n>, v1: i<n>) -> i<n> { v0 % v1 }`, as shipped. -/
def shippedSignedMod (n : ℕ) : Circuit where
  opcodes := signedDivModShared n ++
    [ .assertZero [⟨1, []⟩, ⟨-1, [14, 16]⟩, ⟨-1, [17]⟩],
      .assertZero [⟨1, [14, 17]⟩],
      .assertZero [⟨(2 ^ n : ℕ), [7]⟩, ⟨-2, [7, 14]⟩, ⟨1, [14]⟩, ⟨-1, [18]⟩],
      .assertZero [⟨1, []⟩, ⟨-1, [17]⟩, ⟨-1, [19]⟩],
      .assertZero [⟨1, [2]⟩, ⟨-1, [18, 19]⟩] ]
  parameters := [0, 1]
  returnValues := [2]

end AcirLean
