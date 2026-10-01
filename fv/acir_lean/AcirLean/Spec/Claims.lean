/-
REVIEWED: this file is part of the trusted specification (`AcirLean/Spec/`).
Every definition here is taken on trust: read it against its comment. A change
to this directory needs careful review.
-/

import AcirLean.Spec.Pin
import AcirLean.Spec.Coverage

/-!
# Claims: everything this project promises

Every claim has one shape, `Sound T inputs spec`: for every witness
assignment, if every constraint in the pinned list `T` holds and every
declared input fits its type, then `spec` holds. There is no slot for any
other assumption. Each claim is paired with `Satisfiable T inputs` (some
witness meets every constraint), so the assumptions cannot be contradictory.
That is non-vacuity, not completeness: nothing here says that every valid
input has a witness the circuit accepts.

`AllClaims` is the whole promise. `Check.lean` requires a proof of exactly this
proposition using only Lean's standard axioms.
-/

namespace AcirLean

/-- Witness `i` of each `(i, w)` is a `w`-bit integer: the input's type. -/
def InputsFit (σ : ℕ → F) (inputs : List (ℕ × ℕ)) : Prop :=
  ∀ iw ∈ inputs, (σ iw.1).val < 2 ^ iw.2

/-- For every witness assignment, the constraints and input types imply `spec`. -/
def Sound (T : List Opcode) (inputs : List (ℕ × ℕ)) (spec : (ℕ → F) → Prop) : Prop :=
  ∀ σ : ℕ → F, AllHold σ T → InputsFit σ inputs → spec σ

/-- Some witness assignment meets every constraint and input type. -/
def Satisfiable (T : List Opcode) (inputs : List (ℕ × ℕ)) : Prop :=
  ∃ σ : ℕ → F, AllHold σ T ∧ InputsFit σ inputs

/-- `a / b` and `a % b` on integers. Witnesses: `0 = a, 1 = b, 3 = q, 4 = r`. -/
def divSpec (σ : ℕ → F) : Prop :=
  (σ 3).val = (σ 0).val / (σ 1).val ∧ (σ 4).val = (σ 0).val % (σ 1).val

/-- Division under a predicate: when the predicate is on, `a / b` and `a % b`.
Witnesses: `0 = a, 1 = b, 2 = pred, 5 = q, 6 = r`. -/
def divPredSpec (σ : ℕ → F) : Prop :=
  σ 2 = 1 → (σ 5).val = (σ 0).val / (σ 1).val ∧ (σ 6).val = (σ 0).val % (σ 1).val

/-- `x as u_k` for a field element `x`. Witnesses: `0 = x, 2 = r`. -/
def truncSpec (k : ℕ) (σ : ℕ → F) : Prop :=
  (σ 2).val = (σ 0).val % 2 ^ k

/-- `a >= b` as `1` or `0`. Witnesses: `0 = a, 1 = b, 2 = result`. -/
def geSpec (σ : ℕ → F) : Prop :=
  (σ 2).val = if (σ 1).val ≤ (σ 0).val then 1 else 0

/-- The integer an `n`-bit two's-complement bit pattern stands for. -/
def toSigned (n v : ℕ) : ℤ := if v < 2 ^ (n - 1) then v else (v : ℤ) - 2 ^ n

/-- On every pair of `n`-bit patterns, `f` returns `1` when the first is less
than the second as signed integers, and `0` otherwise. -/
def ComputesSignedLt (f : SsaFunction) (n : ℕ) : Prop :=
  ∀ a b : ℕ, a < 2 ^ n → b < 2 ^ n →
    f.run [a, b] = if toSigned n a < toSigned n b then 1 else 0

/-- For every witness assignment satisfying `f`'s constraints, `spec` holds of
the integer values of `f`'s inputs and return values. There is no other
assumption: in particular the inputs' types are enforced by `f`'s own
constraints, not assumed. -/
def SoundFunction (f : Circuit) (spec : List ℕ → List ℕ → Prop) : Prop :=
  ∀ σ : ℕ → F, AllHold σ f.opcodes →
    spec (f.parameters.map fun i => (σ i).val) (f.returnValues.map fun i => (σ i).val)

/-- Some witness assignment satisfies `f`'s constraints. -/
def SatisfiableFunction (f : Circuit) : Prop := ∃ σ : ℕ → F, AllHold σ f.opcodes

/-- Two `n`-bit inputs `a`, `b` and one return value equal to `g a b`. The
inputs' width is part of the promise: a circuit that let a parameter of type
`u<n>` or `i<n>` exceed `n` bits would break every later use of it. -/
def Computes2 (n : ℕ) (g : ℕ → ℕ → ℕ) : List ℕ → List ℕ → Prop
  | [a, b], [r] => a < 2 ^ n ∧ b < 2 ^ n ∧ r = g a b
  | _, _ => False

/-- `div` on `u<n>`: two `n`-bit inputs, a nonzero divisor, and one return value
equal to the quotient. Noir's SSA interpreter fails on a zero divisor, so a
circuit that accepted one would break this. -/
def DivOp (n : ℕ) : List ℕ → List ℕ → Prop
  | [a, b], [r] => a < 2 ^ n ∧ b < 2 ^ n ∧ b ≠ 0 ∧ r = a / b
  | _, _ => False

/-- One input `a` and one return value, equal to `g a`. -/
def Computes1 (g : ℕ → ℕ) : List ℕ → List ℕ → Prop
  | [a], [r] => r = g a
  | _, _ => False

/-- The `n`-bit two's-complement bit pattern of an integer. -/
def toBitPattern (n : ℕ) (x : ℤ) : ℕ := (x % 2 ^ n).toNat

/-- Signed `div` or `mod` on `i<n>`: two `n`-bit inputs, a nonzero divisor, not
the overflowing `MIN / -1`, and the result is `op` on the signed values,
encoded. Noir's `/` and `%` on signed integers truncate toward zero, which is
Lean's `Int.tdiv` and `Int.tmod`; `AllClaims` names them `_root_.Int.tdiv` and
`_root_.Int.tmod` so that no definition inside `AcirLean` can stand in for
them. -/
def SignedOp (n : ℕ) (op : ℤ → ℤ → ℤ) : List ℕ → List ℕ → Prop
  | [a, b], [r] =>
    a < 2 ^ n ∧ b < 2 ^ n ∧ toSigned n b ≠ 0 ∧ ¬ (toSigned n a = -2 ^ (n - 1) ∧ toSigned n b = -1) ∧
      r = toBitPattern n (op (toSigned n a) (toSigned n b))
  | _, _ => False

/-- Test programs in `testPrograms` that the claims leave out, with the reason:
* `regression_8519` truncates a `Field` to 128 bits, whose remainder bound
  takes a shape the checker does not handle;
* `regression_1144_1169_2399_6609`, `regression_3607`, `regression_6834`,
  `regression_8261`, `regression_9971` and `signed_inactive_division_by_zero`
  do checked arithmetic, `div` or `mod` while side effects may be disabled,
  and the circuit constrains it in a shape the checker does not match yet
  (split across an intermediate witness, or relying on a bit squared being
  itself);
* `regression_10008` constrains a value the checker finds no polynomial for;
* `function_ref` returns a value whose polynomial equals the circuit's output
  only once a bit squared is replaced by itself. -/
def uncoveredPrograms : List String :=
  ["function_ref", "regression_10008", "regression_1144_1169_2399_6609", "regression_3607",
   "regression_6834", "regression_8261", "regression_8519", "regression_9971",
   "signed_inactive_division_by_zero"]

/-- The whole promise, for every pinned width `n`:
* `euclidean_division_var(a, b, n)` with `a`, `b` both `n`-bit computes
  `a / b` and `a % b`, with the predicate constant `1` and with a predicate
  witness that is on;
* `truncate_var(x, n, 254)` on any field element computes `x mod 2^n`;
* `more_than_eq_var(a, b, n)` with `a`, `b` both `n`-bit computes `a >= b`;
* the SSA `expand_signed_math` emits for `lt` on `i<n>` computes signed `<`;
* whole functions, as ACIR generation compiles them, enforce their
  parameters' types and compute their SSA meaning, with no assumption on the
  inputs: `div` (rejecting a zero divisor) and `lt` on `u<n>`, a field
  truncated to `u<n>`, and signed `lt` on `i<n>` after `expand_signed_math`;
* the same functions still do after `acvm::compiler::optimize`, as the
  circuits `nargo compile` ships;
* signed `div` and `mod` on `i<n>` (for `n` in `signedWidths`), as shipped,
  compute truncating signed division and remainder, and reject a zero divisor
  and `MIN / -1`;
* every program in the corpus, as shipped, implements it (`CorpusSpec`), and
  the witness ACVM solved for it satisfies its circuit;
* `testPrograms` holds exactly the programs named in `testProgramNames`, and
  every one of them except `uncoveredPrograms` is implemented by the circuit
  `nargo compile` ships for it (`ProgramSpec`), and the witness `nargo execute`
  solved for it satisfies that circuit;
* no constraint list is contradictory. -/
def AllClaims : Prop :=
  (∀ n ∈ pinnedWidths,
    Sound (divVarGadget n) [(0, n), (1, n)] divSpec ∧
    Satisfiable (divVarGadget n) [(0, n), (1, n)]) ∧
  (∀ n ∈ pinnedWidths,
    Sound (divPredGadget n) [(0, n), (1, n)] divPredSpec ∧
    Satisfiable (divPredGadget n) [(0, n), (1, n)]) ∧
  (∀ k ∈ pinnedWidths,
    Sound (truncateGadget k) [] (truncSpec k) ∧
    Satisfiable (truncateGadget k) []) ∧
  (∀ m ∈ pinnedWidths,
    Sound (moreThanEqGadget m) [(0, m), (1, m)] geSpec ∧
    Satisfiable (moreThanEqGadget m) [(0, m), (1, m)]) ∧
  (∀ n ∈ pinnedWidths, ComputesSignedLt (signedLtSsa n) n) ∧
  (∀ n ∈ pinnedWidths,
    SoundFunction (acirGenDiv n) (DivOp n) ∧ SatisfiableFunction (acirGenDiv n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFunction (acirGenLt n) (Computes2 n (SsaBinOp.eval .lt)) ∧ SatisfiableFunction (acirGenLt n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFunction (acirGenTruncate n) (Computes1 (· % 2 ^ n)) ∧ SatisfiableFunction (acirGenTruncate n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFunction (acirGenSignedLt n)
      (Computes2 n fun a b => if toSigned n a < toSigned n b then 1 else 0) ∧
    SatisfiableFunction (acirGenSignedLt n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFunction (shippedDiv n) (DivOp n) ∧ SatisfiableFunction (shippedDiv n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFunction (shippedLt n) (Computes2 n (SsaBinOp.eval .lt)) ∧ SatisfiableFunction (shippedLt n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFunction (shippedTruncate n) (Computes1 (· % 2 ^ n)) ∧ SatisfiableFunction (shippedTruncate n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFunction (shippedSignedLt n)
      (Computes2 n fun a b => if toSigned n a < toSigned n b then 1 else 0) ∧
    SatisfiableFunction (shippedSignedLt n)) ∧
  (∀ n ∈ signedWidths,
    SoundFunction (shippedSignedDiv n) (SignedOp n _root_.Int.tdiv) ∧ SatisfiableFunction (shippedSignedDiv n)) ∧
  (∀ n ∈ signedWidths,
    SoundFunction (shippedSignedMod n) (SignedOp n _root_.Int.tmod) ∧ SatisfiableFunction (shippedSignedMod n)) ∧
  (∀ e ∈ corpus, SoundFunction e.fn (CorpusSpec e.prog) ∧ AllHold e.assignment e.fn.opcodes) ∧
  testPrograms.map TestProgram.name = testProgramNames ∧
  (∀ e ∈ testPrograms, e.name ∉ uncoveredPrograms →
    SoundFunction e.fn (ProgramSpec e.prog) ∧ AllHold e.assignment e.fn.opcodes)

end AcirLean
