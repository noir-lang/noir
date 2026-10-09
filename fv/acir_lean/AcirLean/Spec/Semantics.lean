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

/-- A polynomial over witnesses: the sum of its terms. Rust's `Expression`
splits the same thing into `mul_terms`, `linear_combinations` and `q_c`; here
every term is a coefficient times any number of witnesses. -/
abbrev Expression := List Term

/-- The ACIR opcodes this development covers (Rust's `Opcode`):
`AssertZero(expr)`, which requires `expr` to be `0`, and the `RANGE` black box,
which requires `witness` to fit in `numBits` bits. -/
inductive Opcode where
  | assertZero (expr : Expression)
  | range (witness numBits : ℕ)
  deriving DecidableEq

/-- The value of a term under the witness assignment `σ` (`σ i` is the value the
prover put in witness `i`). -/
def Term.eval (σ : ℕ → F) (t : Term) : F := (t.coef : F) * (t.witnesses.map σ).prod

/-- An opcode holds: its expression's terms add up to `0`, or the range check
passes. -/
def Opcode.Holds (σ : ℕ → F) : Opcode → Prop
  | .assertZero expr => (expr.map (Term.eval σ)).sum = 0
  | .range witness numBits => Range (σ witness) numBits

/-- Every opcode in the list holds. -/
def AllHold (σ : ℕ → F) (opcodes : List Opcode) : Prop :=
  ∀ c ∈ opcodes, c.Holds σ

/-- An ACIR circuit (Rust's `Circuit`): its opcodes, the witnesses holding its
parameters (private and public, in witness order), and the witnesses holding
its return values. -/
structure Circuit where
  opcodes : List Opcode
  parameters : List ℕ
  returnValues : List ℕ

end AcirLean
