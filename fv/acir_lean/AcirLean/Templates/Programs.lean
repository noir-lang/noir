/-
PINNED: no review needed. These functions are checked byte-for-byte against
what ACIR generation emits, for every width in `Spec.Pin.pinnedWidths`
(`templates.golden`, `fv_templates.rs`). `scripts/check.sh` allows only plain
definitions in this directory.
-/

import AcirLean.Templates.Gadgets

/-!
Whole SSA functions compiled by ACIR generation. Each is the gadget templates
placed at fresh witness indices (`Opcode.rename`), the range checks ACIR
generation puts on the parameters, and a few linking constraints.
-/

namespace AcirLean

/-- Move every witness `i` to `f i`. -/
def Term.rename (f : ℕ → ℕ) (t : Term) : Term := ⟨t.coef, t.witnesses.map f⟩

def Opcode.rename (f : ℕ → ℕ) : Opcode → Opcode
  | .assertZero ts => .assertZero (ts.map (Term.rename f))
  | .range w k => .range (f w) k

/-- Witness `i` goes to position `i` of `m`. -/
def witnessAt (m : List ℕ) (i : ℕ) : ℕ := m.getD i 0

/-- `euclidean_division_var(a, 2^j, n, 1)`: dividing an `n`-bit value by a
power of two. Witnesses: `0 = a, 1 = q, 2 = r`. -/
def divPow2Gadget (n j : ℕ) : List Opcode :=
  [ .range 1 (n - j), .range 2 j, .range 1 (n - j), .range 2 j, .range 2 j,
    .assertZero [⟨1, [0]⟩, ⟨-(2 ^ j : ℕ), [1]⟩, ⟨-1, [2]⟩] ]

/-- `fn main(v0: u<n>, v1: u<n>) -> u<n> { div v0, v1 }`. -/
def acirGenDiv (n : ℕ) : Circuit where
  opcodes := [.range 0 n, .range 1 n] ++
    (divVarGadget n).map (Opcode.rename (witnessAt [0, 1, 3, 4, 5, 6, 7, 8, 9, 10])) ++
    [.assertZero [⟨1, [2]⟩, ⟨-1, [4]⟩]]
  parameters := [0, 1]
  returnValues := [2]

/-- `fn main(v0: u<n>, v1: u<n>) -> u1 { lt v0, v1 }`: `1 - (v0 >= v1)`. -/
def acirGenLt (n : ℕ) : Circuit where
  opcodes := [.range 0 n, .range 1 n] ++
    (moreThanEqGadget n).map (Opcode.rename (witnessAt [0, 1, 3, 4, 5])) ++
    [.assertZero [⟨1, []⟩, ⟨-1, [2]⟩, ⟨-1, [3]⟩]]
  parameters := [0, 1]
  returnValues := [2]

/-- `fn main(v0: Field) -> u<n> { cast (truncate v0 to n bits) as u<n> }`. -/
def acirGenTruncate (n : ℕ) : Circuit where
  opcodes := (truncateGadget n).map (Opcode.rename (witnessAt [0, 2, 3, 4, 5, 6, 7, 8])) ++
    [.assertZero [⟨1, [1]⟩, ⟨-1, [3]⟩]]
  parameters := [0]
  returnValues := [1]

/-- The SSA `signedLtSsa n` compiled: the sign bits `v0 / 2^(n-1)` and
`v1 / 2^(n-1)`, the unsigned comparison, and the two `u1` xors
(`a ^ b = a + b - 2ab`), with `lt = 1 - (v0 >= v1)`. -/
def acirGenSignedLt (n : ℕ) : Circuit :=
  let x := if n = 128 then 10 else 9
  let y := x + 1
  { opcodes := [.range 0 n, .range 1 n] ++
      (divPow2Gadget n (n - 1)).map (Opcode.rename (witnessAt [0, 3, 4])) ++
      (divPow2Gadget n (n - 1)).map (Opcode.rename (witnessAt [1, 5, 6])) ++
      (moreThanEqGadget n).map (Opcode.rename (witnessAt [0, 1, 7, 8, 9])) ++
      [ .assertZero [⟨1, [3]⟩, ⟨-2, [3, 5]⟩, ⟨1, [5]⟩, ⟨-1, [x]⟩],
        .assertZero [⟨1, []⟩, ⟨-1, [7]⟩, ⟨-1, [y]⟩],
        .assertZero [⟨1, [2]⟩, ⟨-1, [x]⟩, ⟨2, [x, y]⟩, ⟨-1, [y]⟩] ]
    parameters := [0, 1]
    returnValues := [2] }

/-- `fn main(v0: u<n>, v1: u<n>) -> u1 { eq v0, v1 }`: `d = v0 - v1`, a
witness `z` with `1 - d z - e = 0` and `d e = 0`, and the result `e`. -/
def acirGenEq (n : ℕ) : Circuit where
  opcodes := [.range 0 n, .range 1 n,
    .assertZero [⟨1, [0]⟩, ⟨-1, [1]⟩, ⟨-1, [3]⟩],
    .assertZero [⟨1, []⟩, ⟨-1, [3, 4]⟩, ⟨-1, [5]⟩],
    .assertZero [⟨1, [3, 5]⟩],
    .assertZero [⟨1, [2]⟩, ⟨-1, [5]⟩]]
  parameters := [0, 1]
  returnValues := [2]

/-- `fn main(v0: u<n>) -> u<n> { not v0 }`: `2^n - 1 - v0`. -/
def acirGenNot (n : ℕ) : Circuit where
  opcodes := [.range 0 n, .assertZero [⟨(2 ^ n - 1 : ℕ), []⟩, ⟨-1, [0]⟩, ⟨-1, [1]⟩]]
  parameters := [0]
  returnValues := [1]

/-- `fn main(v0: Field, v1: Field) -> Field { div v0, v1 }`: a witness `z` with
`v1 z = 1`, and the result `v0 z`. -/
def acirGenFieldDiv : Circuit where
  opcodes := [.assertZero [⟨1, []⟩, ⟨-1, [1, 3]⟩], .assertZero [⟨1, [0, 3]⟩, ⟨-1, [2]⟩]]
  parameters := [0, 1]
  returnValues := [2]

/-- `fn main(v0: u<n>) -> u<n> { shr v0, c }`, which `remove_bit_shifts` turns
into `div v0, 2^c`: the quotient by `2^c`. -/
def acirGenShr (n c : ℕ) : Circuit where
  opcodes := [.range 0 n] ++ (divPow2Gadget n c).map (Opcode.rename (witnessAt [0, 2, 3])) ++
    [.assertZero [⟨1, [1]⟩, ⟨-1, [2]⟩]]
  parameters := [0]
  returnValues := [1]

/-- `euclidean_division_var(d · x, 2^128, 128 + qb, 1)`: the 128-bit
truncation of `d · x`, with the quotient range-checked to `qb` bits.
`truncateGadget 128` is `qb = 126`, `d = 1`. Witnesses: `0 = x, 1 = q, 2 = r`. -/
def truncGadget128 (qb d : ℕ) : List Opcode :=
  [ .range 1 qb, .range 2 128, .range 1 qb, .range 2 128,
    .assertZero [⟨(2 ^ 128 - 1 : ℕ), []⟩, ⟨-1, [2]⟩, ⟨-1, [3]⟩],
    .range 3 128,
    .assertZero [⟨(d : ℤ), [0]⟩, ⟨-(2 ^ 128 : ℕ), [1]⟩, ⟨-1, [2]⟩],
    .assertZero [⟨(Rq 128 : ℤ), []⟩, ⟨1, [1]⟩, ⟨-1, [4]⟩],
    .range 4 (bits (q0 128)),
    .assertZero [⟨1, []⟩, ⟨1, [1, 5]⟩, ⟨-(q0 128 : ℤ), [5]⟩, ⟨-1, [6]⟩],
    .assertZero [⟨1, [1, 6]⟩, ⟨-(q0 128 : ℤ), [6]⟩],
    .assertZero [⟨1, [2, 6]⟩, ⟨(R' 128 : ℤ), [6]⟩, ⟨-1, [7]⟩],
    .range 7 (N' 128) ]

/-- `fn main(v0: u<n>) -> u<n> { shl v0, c }`, which `remove_bit_shifts` turns
into `v0 · 2^c` truncated to `n` bits. On `u128` with `c ≥ 126` the product
could reach the field size, so it multiplies by `2^63` and then by `2^(c-63)`,
truncating after each. -/
def acirGenShl (n c : ℕ) : Circuit where
  opcodes :=
    if n = 128 ∧ 126 ≤ c then
      [.range 0 128] ++
        (truncGadget128 126 (2 ^ 63)).map (Opcode.rename (witnessAt [0, 2, 3, 4, 5, 6, 7, 8])) ++
        (truncGadget128 126 (2 ^ (c - 63))).map
          (Opcode.rename (witnessAt [3, 9, 10, 11, 12, 13, 14, 15])) ++
        [.assertZero [⟨1, [1]⟩, ⟨-1, [10]⟩]]
    else if n = 128 then
      -- the bound on the quotient is only emitted from `c = 124` (a product of
      -- up to 252 bits); below that the first seven constraints are all there is
      [.range 0 128] ++
        ((truncGadget128 c (2 ^ c)).take (if 124 ≤ c then 13 else 7)).map
          (Opcode.rename (witnessAt [0, 2, 3, 4, 5, 6, 7, 8])) ++
        [.assertZero [⟨1, [1]⟩, ⟨-1, [3]⟩]]
    else
      [ .range 0 n, .range 2 c, .range 3 n, .range 2 c, .range 3 n, .range 3 n,
        .assertZero [⟨(2 ^ c : ℕ), [0]⟩, ⟨-(2 ^ n : ℕ), [2]⟩, ⟨-1, [3]⟩],
        .assertZero [⟨1, [1]⟩, ⟨-1, [3]⟩] ]
  parameters := [0]
  returnValues := [1]

end AcirLean
