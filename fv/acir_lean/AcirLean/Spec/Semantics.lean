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

/-- A monomial `coef * w_1 * … * w_m`: a coefficient times the product of the
listed witnesses (a constant term lists none). -/
structure Term where
  coef : ℤ
  witnesses : List ℕ
  deriving DecidableEq

/-- One ACIR constraint: an `AssertZero` polynomial (a sum of terms that must
be `0`), or a `RANGE` check that `witness` fits in `bits` bits. -/
inductive Constraint where
  | zero (terms : List Term)
  | range (witness bits : ℕ)
  deriving DecidableEq

/-- The value of a term under the witness assignment `σ` (`σ i` is the value the
prover put in witness `i`). -/
def Term.eval (σ : ℕ → F) (t : Term) : F := (t.coef : F) * (t.witnesses.map σ).prod

/-- A constraint holds: the polynomial is `0`, or the range check passes. -/
def Constraint.Holds (σ : ℕ → F) : Constraint → Prop
  | .zero terms => (terms.map (Term.eval σ)).sum = 0
  | .range witness bits => Range (σ witness) bits

/-- Every constraint in the list holds. -/
def AllHold (σ : ℕ → F) (constraints : List Constraint) : Prop :=
  ∀ c ∈ constraints, c.Holds σ

/-- An ACIR function as ACIR generation emits it: its constraints, and the
witnesses holding its parameters and its return values. -/
structure AcirFunction where
  constraints : List Constraint
  inputs : List ℕ
  returns : List ℕ

end AcirLean
