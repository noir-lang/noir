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

/-- The whole promise, for every pinned width `n`:
* `euclidean_division_var(a, b, n)` with `a`, `b` both `n`-bit computes
  `a / b` and `a % b`, with the predicate constant `1` and with a predicate
  witness that is on;
* `truncate_var(x, n, 254)` on any field element computes `x mod 2^n`;
* `more_than_eq_var(a, b, n)` with `a`, `b` both `n`-bit computes `a >= b`;
* the SSA `expand_signed_math` emits for `lt` on `i<n>` computes signed `<`;
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
  (∀ n ∈ pinnedWidths, ComputesSignedLt (signedLtT n) n)

end AcirLean
