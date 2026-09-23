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
Witnesses: `0 = a, 1 = b, 2 = inv, 3 = q, 4 = r, 5 = b - r - 1`. -/

def divVarT (n : ℕ) : List Cstr :=
  [ .zero [⟨1, []⟩, ⟨-1, [1, 2]⟩],
    .range 3 n, .range 4 n, .range 3 n, .range 4 n,
    .zero [⟨1, []⟩, ⟨-1, [1]⟩, ⟨1, [4]⟩, ⟨1, [5]⟩],
    .range 5 n,
    .zero [⟨1, [0]⟩, ⟨-1, [1, 3]⟩, ⟨-1, [4]⟩] ]

/-! ### `truncate_var(x, k, 254)`
Witnesses: `0 = x, 1 = q, 2 = r, 3 = q0 - q, 4 = z, 5 = y, 6 = u`. -/

def q0 (k : ℕ) : ℕ := p / 2 ^ k
def M (k : ℕ) : ℕ := p % 2 ^ k
def N' (k : ℕ) : ℕ := bits (M k - 1)
def R' (k : ℕ) : ℕ := 2 ^ N' k - M k

def truncT (k : ℕ) : List Cstr :=
  [ .range 1 (254 - k), .range 2 k, .range 1 (254 - k), .range 2 k, .range 2 k,
    .zero [⟨1, [0]⟩, ⟨-(2 ^ k : ℕ), [1]⟩, ⟨-1, [2]⟩],
    .zero [⟨(q0 k : ℤ), []⟩, ⟨-1, [1]⟩, ⟨-1, [3]⟩],
    .range 3 (254 - k),
    .zero [⟨1, []⟩, ⟨1, [1, 4]⟩, ⟨-(q0 k : ℤ), [4]⟩, ⟨-1, [5]⟩],
    .zero [⟨1, [1, 5]⟩, ⟨-(q0 k : ℤ), [5]⟩],
    .zero [⟨1, [2, 5]⟩, ⟨(R' k : ℤ), [5]⟩, ⟨-1, [6]⟩],
    if N' k = 0 then .zero [⟨1, [6]⟩] else .range 6 (N' k) ]

end AcirLean
