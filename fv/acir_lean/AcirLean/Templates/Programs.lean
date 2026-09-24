/-
PINNED: no review needed. These functions are checked byte-for-byte against
what ACIR generation emits, for every width in `Spec.Pin.pinnedWidths`
(`templates.golden`, `fv_templates.rs`). `scripts/check.sh` allows only plain
definitions in this directory.
-/

import AcirLean.Templates.Gadgets

/-!
Whole SSA functions compiled by ACIR generation. Each is the gadget templates
placed at fresh witness indices (`Cstr.rename`), the range checks ACIR
generation puts on the parameters, and a few linking constraints.
-/

namespace AcirLean

/-- Move every witness `i` to `f i`. -/
def Term.rename (f : ℕ → ℕ) (t : Term) : Term := ⟨t.coef, t.ws.map f⟩

def Cstr.rename (f : ℕ → ℕ) : Cstr → Cstr
  | .zero ts => .zero (ts.map (Term.rename f))
  | .range w k => .range (f w) k

/-- Witness `i` goes to position `i` of `m`. -/
def at_ (m : List ℕ) (i : ℕ) : ℕ := m.getD i 0

/-- `euclidean_division_var(a, 2^j, n, 1)`: dividing an `n`-bit value by a
power of two. Witnesses: `0 = a, 1 = q, 2 = r`. -/
def divPow2T (n j : ℕ) : List Cstr :=
  [ .range 1 (n - j), .range 2 j, .range 1 (n - j), .range 2 j, .range 2 j,
    .zero [⟨1, [0]⟩, ⟨-(2 ^ j : ℕ), [1]⟩, ⟨-1, [2]⟩] ]

/-- `fn main(v0: u<n>, v1: u<n>) -> u<n> { div v0, v1 }`. -/
def acirDivT (n : ℕ) : AcirFn where
  cs := [.range 0 n, .range 1 n] ++
    (divVarT n).map (Cstr.rename (at_ [0, 1, 3, 4, 5, 6, 7, 8, 9, 10])) ++
    [.zero [⟨1, [2]⟩, ⟨-1, [4]⟩]]
  inputs := [0, 1]
  returns := [2]

/-- `fn main(v0: u<n>, v1: u<n>) -> u1 { lt v0, v1 }`: `1 - (v0 >= v1)`. -/
def acirLtT (n : ℕ) : AcirFn where
  cs := [.range 0 n, .range 1 n] ++
    (moreThanEqT n).map (Cstr.rename (at_ [0, 1, 3, 4, 5])) ++
    [.zero [⟨1, []⟩, ⟨-1, [2]⟩, ⟨-1, [3]⟩]]
  inputs := [0, 1]
  returns := [2]

/-- `fn main(v0: Field) -> u<n> { cast (truncate v0 to n bits) as u<n> }`. -/
def acirTruncT (n : ℕ) : AcirFn where
  cs := (truncT n).map (Cstr.rename (at_ [0, 2, 3, 4, 5, 6, 7, 8])) ++
    [.zero [⟨1, [1]⟩, ⟨-1, [3]⟩]]
  inputs := [0]
  returns := [1]

/-- The SSA `signedLtT n` compiled: the sign bits `v0 / 2^(n-1)` and
`v1 / 2^(n-1)`, the unsigned comparison, and the two `u1` xors
(`a ^ b = a + b - 2ab`), with `lt = 1 - (v0 >= v1)`. -/
def acirSignedLtT (n : ℕ) : AcirFn :=
  let x := if n = 128 then 10 else 9
  let y := x + 1
  { cs := [.range 0 n, .range 1 n] ++
      (divPow2T n (n - 1)).map (Cstr.rename (at_ [0, 3, 4])) ++
      (divPow2T n (n - 1)).map (Cstr.rename (at_ [1, 5, 6])) ++
      (moreThanEqT n).map (Cstr.rename (at_ [0, 1, 7, 8, 9])) ++
      [ .zero [⟨1, [3]⟩, ⟨-2, [3, 5]⟩, ⟨1, [5]⟩, ⟨-1, [x]⟩],
        .zero [⟨1, []⟩, ⟨-1, [7]⟩, ⟨-1, [y]⟩],
        .zero [⟨1, [2]⟩, ⟨-1, [x]⟩, ⟨2, [x, y]⟩, ⟨-1, [y]⟩] ]
    inputs := [0, 1]
    returns := [2] }

end AcirLean
