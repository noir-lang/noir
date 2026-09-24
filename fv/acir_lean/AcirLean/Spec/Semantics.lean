/-
REVIEWED: this file is part of the trusted specification (`AcirLean/Spec/`).
Every definition here is taken on trust: read it against its comment. A change
to this directory needs careful review.
-/

import Mathlib.Data.ZMod.Basic
import Mathlib.Tactic.NormNum

/-!
# What an ACIR constraint means

A witness assignment `σ` gives every witness index a value in the BN254 scalar
field `F`. An `AssertZero` opcode holds when its polynomial evaluates to `0`;
a `RANGE { num_bits := k }` opcode holds when the witness, read as an integer
in `[0, p)`, is below `2^k`.
-/

namespace AcirLean

/-- The BN254 scalar field modulus (`FieldElement::modulus()`). -/
def p : ℕ := 21888242871839275222246405745257275088548364400416034343698204186575808495617

instance : NeZero p := ⟨by norm_num [p]⟩
instance : Fact (1 < p) := ⟨by norm_num [p]⟩

/-- Field elements. `x.val` is the integer representative of `x`, in `[0, p)`. -/
abbrev F := ZMod p

/-- `RANGE { num_bits := k }`: the witness value is a `k`-bit integer. -/
def Range (x : F) (k : ℕ) : Prop := x.val < 2 ^ k

/-- A monomial `coef * w_1 * … * w_m` over witness indices `ws`. -/
structure Term where
  coef : ℤ
  ws : List ℕ
  deriving DecidableEq

/-- One ACIR constraint: an `AssertZero` polynomial (a sum of terms), or a
`RANGE` check on witness `w`. -/
inductive Cstr where
  | zero (ts : List Term)
  | range (w k : ℕ)
  deriving DecidableEq

/-- The value of a term under the witness assignment `σ`. -/
def Term.eval (σ : ℕ → F) (t : Term) : F := (t.coef : F) * (t.ws.map σ).prod

/-- A constraint holds: the polynomial is `0`, or the range check passes. -/
def Cstr.sat (σ : ℕ → F) : Cstr → Prop
  | .zero ts => (ts.map (Term.eval σ)).sum = 0
  | .range w k => Range (σ w) k

/-- Every constraint in the list holds. -/
def AllSat (σ : ℕ → F) (cs : List Cstr) : Prop := ∀ c ∈ cs, c.sat σ

/-- An ACIR function as ACIR generation emits it: its constraints, and the
witnesses holding its parameters and its return values. -/
structure AcirFn where
  cs : List Cstr
  inputs : List ℕ
  returns : List ℕ

end AcirLean
