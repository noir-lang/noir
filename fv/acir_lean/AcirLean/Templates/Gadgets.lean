/-
PINNED: no review needed. These constraint lists are checked byte-for-byte
against what the Rust gadgets emit, for every width in `Spec.Pin.pinnedWidths`
(`templates.golden`, `fv_templates.rs`). `scripts/check.sh` allows only plain
definitions in this directory.
-/

import AcirLean.Spec.Semantics
import Mathlib.Data.Nat.Size

/-!
The gadget constraint lists, exactly as the Rust gadgets emit them before ACIR
optimization, with witness indices in `AcirContext` allocation order.
-/

namespace AcirLean

/-- Rust `num_bits` / `bit_size_u128`: the bit length. -/
abbrev bits (n : ℕ) : ℕ := Nat.size n

/-! ### `euclidean_division_var(a, b, n, 1)`, non-constant `b`
Witnesses: `0 = a, 1 = b, 2 = inv, 3 = q, 4 = r, 5 = b - r - 1`; at `n = 128`
also `6, 7 = q / 2^64, q % 2^64` and `8, 9 = b / 2^64, b % 2^64`. -/

def divVarGadget (n : ℕ) : List Opcode :=
  [ .assertZero [⟨1, []⟩, ⟨-1, [1, 2]⟩],
    .range 3 n, .range 4 n, .range 3 n, .range 4 n,
    .assertZero [⟨1, []⟩, ⟨-1, [1]⟩, ⟨1, [4]⟩, ⟨1, [5]⟩],
    .range 5 n,
    .assertZero [⟨1, [0]⟩, ⟨-1, [1, 3]⟩, ⟨-1, [4]⟩] ] ++
  if n = 128 then
    [ .range 6 64, .range 7 64, .range 6 64, .range 7 64, .range 7 64,
      .assertZero [⟨1, [3]⟩, ⟨-(2 ^ 64 : ℕ), [6]⟩, ⟨-1, [7]⟩],
      .range 8 64, .range 9 64, .range 8 64, .range 9 64, .range 9 64,
      .assertZero [⟨1, [1]⟩, ⟨-(2 ^ 64 : ℕ), [8]⟩, ⟨-1, [9]⟩],
      .assertZero [⟨1, [6, 8]⟩] ]
  else []

/-! ### `euclidean_division_var(a, b, n, pred)`, non-constant `b` and predicate
Witnesses: `0 = a, 1 = b, 2 = pred, 3 = z, 4 = [b == 0], 5 = q, 6 = r,
7 = b - pred - r, 8 = b*q + r`; at `n = 128` also `9, 10 = q / 2^64, q % 2^64`,
`11 = q % 2^64 + pred + 2^64 - 1`, `12, 13 = b / 2^64, b % 2^64` and
`14 = b % 2^64 + pred + 2^64 - 1`. -/

def divPredGadget (n : ℕ) : List Opcode :=
  [ .assertZero [⟨1, []⟩, ⟨-1, [1, 3]⟩, ⟨-1, [4]⟩],
    .assertZero [⟨1, [1, 4]⟩],
    .assertZero [⟨1, [2, 4]⟩],
    .range 5 n, .range 6 n, .range 5 n, .range 6 n,
    .assertZero [⟨1, [1]⟩, ⟨-1, [2]⟩, ⟨-1, [6]⟩, ⟨-1, [7]⟩],
    .range 7 n,
    .assertZero [⟨1, [1, 5]⟩, ⟨1, [6]⟩, ⟨-1, [8]⟩],
    .assertZero [⟨1, [0, 2]⟩, ⟨-1, [2, 8]⟩] ] ++
  if n = 128 then
    [ .range 9 64, .range 10 64, .range 9 64, .range 10 64,
      .assertZero [⟨(2 ^ 64 - 1 : ℕ), []⟩, ⟨1, [2]⟩, ⟨1, [10]⟩, ⟨-1, [11]⟩],
      .range 11 65,
      .assertZero [⟨1, [2, 5]⟩, ⟨-(2 ^ 64 : ℕ), [2, 9]⟩, ⟨-1, [2, 10]⟩],
      .range 12 64, .range 13 64, .range 12 64, .range 13 64,
      .assertZero [⟨(2 ^ 64 - 1 : ℕ), []⟩, ⟨1, [2]⟩, ⟨1, [13]⟩, ⟨-1, [14]⟩],
      .range 14 65,
      .assertZero [⟨1, [1, 2]⟩, ⟨-(2 ^ 64 : ℕ), [2, 12]⟩, ⟨-1, [2, 13]⟩],
      .assertZero [⟨1, [9, 12]⟩] ]
  else []

/-! ### `truncate_var(x, k, 254)`
Witnesses for `k < 128`: `0 = x, 1 = q, 2 = r, 3 = q0 - q, 4 = z, 5 = y, 6 = u`.
At `k = 128` the remainder and quotient bounds take other branches of
`bound_constraint_with_offset`: `0 = x, 1 = q, 2 = r, 3 = 2^128 - 1 - r,
4 = q + (2^126 - 1 - q0), 5 = z, 6 = y, 7 = u`. -/

def q0 (k : ℕ) : ℕ := p / 2 ^ k
def M (k : ℕ) : ℕ := p % 2 ^ k
def N' (k : ℕ) : ℕ := bits (M k - 1)
def R' (k : ℕ) : ℕ := 2 ^ N' k - M k

/-- The slack that moves `q ≤ q0` to a range check when `q0 < 2^128`. -/
def Rq (k : ℕ) : ℕ := 2 ^ bits (q0 k) - 1 - q0 k

def truncateGadget (k : ℕ) : List Opcode :=
  if k = 128 then
    [ .range 1 126, .range 2 128, .range 1 126, .range 2 128,
      .assertZero [⟨(2 ^ 128 - 1 : ℕ), []⟩, ⟨-1, [2]⟩, ⟨-1, [3]⟩],
      .range 3 128,
      .assertZero [⟨1, [0]⟩, ⟨-(2 ^ 128 : ℕ), [1]⟩, ⟨-1, [2]⟩],
      .assertZero [⟨(Rq 128 : ℤ), []⟩, ⟨1, [1]⟩, ⟨-1, [4]⟩],
      .range 4 (bits (q0 128)),
      .assertZero [⟨1, []⟩, ⟨1, [1, 5]⟩, ⟨-(q0 128 : ℤ), [5]⟩, ⟨-1, [6]⟩],
      .assertZero [⟨1, [1, 6]⟩, ⟨-(q0 128 : ℤ), [6]⟩],
      .assertZero [⟨1, [2, 6]⟩, ⟨(R' 128 : ℤ), [6]⟩, ⟨-1, [7]⟩],
      .range 7 (N' 128) ]
  else
    [ .range 1 (254 - k), .range 2 k, .range 1 (254 - k), .range 2 k, .range 2 k,
      .assertZero [⟨1, [0]⟩, ⟨-(2 ^ k : ℕ), [1]⟩, ⟨-1, [2]⟩],
      .assertZero [⟨(q0 k : ℤ), []⟩, ⟨-1, [1]⟩, ⟨-1, [3]⟩],
      .range 3 (254 - k),
      .assertZero [⟨1, []⟩, ⟨1, [1, 4]⟩, ⟨-(q0 k : ℤ), [4]⟩, ⟨-1, [5]⟩],
      .assertZero [⟨1, [1, 5]⟩, ⟨-(q0 k : ℤ), [5]⟩],
      .assertZero [⟨1, [2, 5]⟩, ⟨(R' k : ℤ), [5]⟩, ⟨-1, [6]⟩],
      if N' k = 0 then .assertZero [⟨1, [6]⟩] else .range 6 (N' k) ]

/-! ### `more_than_eq_var(a, b, m)`: divide `2^m + a - b` by `2^m`
Witnesses: `0 = a, 1 = b, 2 = q, 3 = r`; at `m = 128` also `4 = 2^128 - 1 - r`. -/

def moreThanEqGadget (m : ℕ) : List Opcode :=
  if m = 128 then
    [ .range 2 1, .range 3 128, .range 2 1, .range 3 128,
      .assertZero [⟨(2 ^ 128 - 1 : ℕ), []⟩, ⟨-1, [3]⟩, ⟨-1, [4]⟩],
      .range 4 128,
      .assertZero [⟨(2 ^ 128 : ℕ), []⟩, ⟨1, [0]⟩, ⟨-1, [1]⟩, ⟨-(2 ^ 128 : ℕ), [2]⟩, ⟨-1, [3]⟩] ]
  else
    [ .range 2 1, .range 3 m, .range 2 1, .range 3 m, .range 3 m,
      .assertZero [⟨(2 ^ m : ℕ), []⟩, ⟨1, [0]⟩, ⟨-1, [1]⟩, ⟨-(2 ^ m : ℕ), [2]⟩, ⟨-1, [3]⟩] ]

end AcirLean
