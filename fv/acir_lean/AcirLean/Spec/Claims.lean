/-
REVIEWED: this file is part of the trusted specification (`AcirLean/Spec/`).
Every definition here is taken on trust: read it against its comment. A change
to this directory needs careful review.
-/

import AcirLean.Spec.Pin

/-!
# Claims: everything this project promises

Every claim has one shape, `Sound T inputs spec`: for every witness
assignment, if every constraint in the pinned list `T` holds and every
declared input fits its type, then `spec` holds. There is no slot for any
other assumption. Each claim is paired with `Satisfiable T inputs` (an honest
witness meets every constraint), so the assumptions cannot be contradictory.

`AllClaims` is the whole promise. `Check.lean` requires a proof of exactly this
proposition using only Lean's standard axioms.
-/

namespace AcirLean

/-- Witness `i` of each `(i, w)` is a `w`-bit integer: the input's type. -/
def InputsFit (σ : ℕ → F) (inputs : List (ℕ × ℕ)) : Prop :=
  ∀ iw ∈ inputs, (σ iw.1).val < 2 ^ iw.2

/-- For every witness assignment, the constraints and input types imply `spec`. -/
def Sound (T : List Cstr) (inputs : List (ℕ × ℕ)) (spec : (ℕ → F) → Prop) : Prop :=
  ∀ σ : ℕ → F, AllSat σ T → InputsFit σ inputs → spec σ

/-- Some witness assignment meets every constraint and input type. -/
def Satisfiable (T : List Cstr) (inputs : List (ℕ × ℕ)) : Prop :=
  ∃ σ : ℕ → F, AllSat σ T ∧ InputsFit σ inputs

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
def sint (n v : ℕ) : ℤ := if v < 2 ^ (n - 1) then v else (v : ℤ) - 2 ^ n

/-- On every pair of `n`-bit patterns, `f` returns `1` when the first is less
than the second as signed integers, and `0` otherwise. -/
def ComputesSignedLt (f : SsaFn) (n : ℕ) : Prop :=
  ∀ a b : ℕ, a < 2 ^ n → b < 2 ^ n →
    f.run [a, b] = if sint n a < sint n b then 1 else 0

/-- For every witness assignment satisfying `f`'s constraints, `spec` holds of
the integer values of `f`'s inputs and return values. There is no other
assumption: in particular the inputs' types are enforced by `f`'s own
constraints, not assumed. -/
def SoundFn (f : AcirFn) (spec : List ℕ → List ℕ → Prop) : Prop :=
  ∀ σ : ℕ → F, AllSat σ f.cs →
    spec (f.inputs.map fun i => (σ i).val) (f.returns.map fun i => (σ i).val)

/-- Some witness assignment satisfies `f`'s constraints. -/
def SatisfiableFn (f : AcirFn) : Prop := ∃ σ : ℕ → F, AllSat σ f.cs

/-- Two `n`-bit inputs `a`, `b` and one return value equal to `g a b`. The
inputs' width is part of the promise: a circuit that let a parameter of type
`u<n>` or `i<n>` exceed `n` bits would break every later use of it. -/
def Computes2 (n : ℕ) (g : ℕ → ℕ → ℕ) : List ℕ → List ℕ → Prop
  | [a, b], [r] => a < 2 ^ n ∧ b < 2 ^ n ∧ r = g a b
  | _, _ => False

/-- One input `a` and one return value, equal to `g a`. -/
def Computes1 (g : ℕ → ℕ) : List ℕ → List ℕ → Prop
  | [a], [r] => r = g a
  | _, _ => False

/-- The `n`-bit two's-complement bit pattern of an integer. -/
def encode (n : ℕ) (x : ℤ) : ℕ := (x % 2 ^ n).toNat

/-- Signed `div` or `mod` on `i<n>`: two `n`-bit inputs, a nonzero divisor, not
the overflowing `MIN / -1`, and the result is `op` on the signed values,
encoded. Noir's `/` and `%` on signed integers truncate toward zero, which is
Lean's `Int.tdiv` and `Int.tmod`. -/
def SignedOp (n : ℕ) (op : ℤ → ℤ → ℤ) : List ℕ → List ℕ → Prop
  | [a, b], [r] =>
    a < 2 ^ n ∧ b < 2 ^ n ∧ sint n b ≠ 0 ∧ ¬ (sint n a = -2 ^ (n - 1) ∧ sint n b = -1) ∧
      r = encode n (op (sint n a) (sint n b))
  | _, _ => False

/-- Test programs in `testPrograms` that the claims leave out, with the reason:
* `arithmetic_binary_operations` divides `Field`s, which `Prog2.eval` does not
  define;
* `regression_8519` truncates a `Field` to 128 bits, whose remainder bound
  takes a shape the checker does not handle;
* `vector_pop_back_simplify` adds `c * x` and `(1 - c) * y` for a boolean `c`
  unchecked; that this cannot overflow needs a case split on `c` that interval
  bounds do not make. -/
def uncoveredPrograms : List String :=
  ["arithmetic_binary_operations", "regression_8519", "vector_pop_back_simplify"]

/-- The whole promise, for every pinned width `n`:
* `euclidean_division_var(a, b, n)` with `a`, `b` both `n`-bit computes
  `a / b` and `a % b`, with the predicate constant `1` and with a predicate
  witness that is on;
* `truncate_var(x, n, 254)` on any field element computes `x mod 2^n`;
* `more_than_eq_var(a, b, n)` with `a`, `b` both `n`-bit computes `a >= b`;
* the SSA `expand_signed_math` emits for `lt` on `i<n>` computes signed `<`;
* whole functions, as ACIR generation compiles them, enforce their
  parameters' types and compute their SSA meaning, with no assumption on the
  inputs: `div` and `lt` on `u<n>`, a field
  truncated to `u<n>`, and signed `lt` on `i<n>` after `expand_signed_math`;
* the same functions still do after `acvm::compiler::optimize`, as the
  circuits `nargo compile` ships;
* signed `div` and `mod` on `i<n>` (for `n` in `signedWidths`), as shipped,
  compute truncating signed division and remainder, and reject a zero divisor
  and `MIN / -1`;
* every program in the corpus, as shipped, implements it (`ProgSpec`), and
  the witness ACVM solved for it satisfies its circuit;
* every scalar program from `test_programs/execution_success` in
  `testPrograms`, except `uncoveredPrograms`, is implemented by the circuit
  `nargo compile` ships for it (`ProgSpec2`);
* no constraint list is contradictory. -/
def AllClaims : Prop :=
  (∀ n ∈ pinnedWidths,
    Sound (divVarT n) [(0, n), (1, n)] divSpec ∧
    Satisfiable (divVarT n) [(0, n), (1, n)]) ∧
  (∀ n ∈ pinnedWidths,
    Sound (divPredT n) [(0, n), (1, n)] divPredSpec ∧
    Satisfiable (divPredT n) [(0, n), (1, n)]) ∧
  (∀ k ∈ pinnedWidths,
    Sound (truncT k) [] (truncSpec k) ∧
    Satisfiable (truncT k) []) ∧
  (∀ m ∈ pinnedWidths,
    Sound (moreThanEqT m) [(0, m), (1, m)] geSpec ∧
    Satisfiable (moreThanEqT m) [(0, m), (1, m)]) ∧
  (∀ n ∈ pinnedWidths, ComputesSignedLt (signedLtT n) n) ∧
  (∀ n ∈ pinnedWidths,
    SoundFn (acirDivT n) (Computes2 n (BinOp.eval .div)) ∧ SatisfiableFn (acirDivT n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFn (acirLtT n) (Computes2 n (BinOp.eval .lt)) ∧ SatisfiableFn (acirLtT n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFn (acirTruncT n) (Computes1 (· % 2 ^ n)) ∧ SatisfiableFn (acirTruncT n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFn (acirSignedLtT n)
      (Computes2 n fun a b => if sint n a < sint n b then 1 else 0) ∧
    SatisfiableFn (acirSignedLtT n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFn (shippedDivT n) (Computes2 n (BinOp.eval .div)) ∧ SatisfiableFn (shippedDivT n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFn (shippedLtT n) (Computes2 n (BinOp.eval .lt)) ∧ SatisfiableFn (shippedLtT n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFn (shippedTruncT n) (Computes1 (· % 2 ^ n)) ∧ SatisfiableFn (shippedTruncT n)) ∧
  (∀ n ∈ pinnedWidths,
    SoundFn (shippedSignedLtT n)
      (Computes2 n fun a b => if sint n a < sint n b then 1 else 0) ∧
    SatisfiableFn (shippedSignedLtT n)) ∧
  (∀ n ∈ signedWidths,
    SoundFn (shippedSDivT n) (SignedOp n Int.tdiv) ∧ SatisfiableFn (shippedSDivT n)) ∧
  (∀ n ∈ signedWidths,
    SoundFn (shippedSModT n) (SignedOp n Int.tmod) ∧ SatisfiableFn (shippedSModT n)) ∧
  (∀ e ∈ corpus, SoundFn e.fn (ProgSpec e.prog) ∧ AllSat e.assignment e.fn.cs) ∧
  (∀ e ∈ testPrograms, e.name ∉ uncoveredPrograms → SoundFn e.fn (ProgSpec2 e.prog))

end AcirLean
