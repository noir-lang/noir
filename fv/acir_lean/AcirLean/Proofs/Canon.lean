/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Programs

/-! `Opcode.canon` (`Spec/Pin.lean`) preserves what a constraint means. -/

namespace AcirLean

theorem insertBy_perm {α : Type} (le : α → α → Bool) (x : α) :
    ∀ l : List α, (insertBy le x l).Perm (x :: l)
  | [] => .refl _
  | y :: ys => by
    unfold insertBy
    split
    · exact .refl _
    · exact ((insertBy_perm le x ys).cons y).trans (.swap x y ys)

theorem isort_perm {α : Type} (le : α → α → Bool) : ∀ l : List α, (isort le l).Perm l
  | [] => .refl _
  | x :: xs => (insertBy_perm le x _).trans ((isort_perm le xs).cons x)

theorem eval_modP (σ : ℕ → F) (t : Term) :
    Term.eval σ ⟨modP t.coef, isort (fun a b => decide (a ≤ b)) t.witnesses⟩ = Term.eval σ t := by
  unfold Term.eval modP
  rw [ZMod.intCast_mod]
  congr 1
  exact ((isort_perm _ _).map σ).prod_eq

theorem sum_filter_nonzero (σ : ℕ → F) :
    ∀ ts : List Term, (∀ t ∈ ts, t.coef = 0 → Term.eval σ t = 0) →
      ((ts.filter fun t => t.coef != 0).map (Term.eval σ)).sum = (ts.map (Term.eval σ)).sum
  | [], _ => rfl
  | t :: ts, h => by
    have ih := sum_filter_nonzero σ ts (fun t' ht' => h t' (by simp [ht']))
    by_cases ht : t.coef = 0
    · simp [ht, ih, h t (by simp) ht]
    · simp [ht, ih]

theorem sum_map_neg (f : Term → F) : ∀ l : List Term,
    (l.map fun t => -f t).sum = -(l.map f).sum
  | [] => by simp
  | t :: l => by simp [sum_map_neg f l]; ring

theorem Opcode.canon_sat (σ : ℕ → F) (c : Opcode) : c.canon.Holds σ ↔ c.Holds σ := by
  cases c with
  | range w k => rfl
  | and a b k o => rfl
  | xor a b k o => rfl
  | assertZero ts =>
    simp only [Opcode.canon, Opcode.Holds]
    set ts1 := ts.map fun t => (⟨modP t.coef, isort (fun a b => decide (a ≤ b)) t.witnesses⟩ : Term)
    set ts2 := isort (fun a b => witnessListLe a.witnesses b.witnesses) (ts1.filter fun t => t.coef != 0)
    have h1 : (ts1.map (Term.eval σ)).sum = (ts.map (Term.eval σ)).sum := by
      simp only [ts1, List.map_map]
      congr 1
      exact List.map_congr_left (fun t _ => eval_modP σ t)
    have h2 : (ts2.map (Term.eval σ)).sum = (ts.map (Term.eval σ)).sum := by
      rw [((isort_perm _ _).map _).sum_eq, sum_filter_nonzero σ ts1, h1]
      intro t _ ht; simp [Term.eval, ht]
    have hneg : ((ts2.map fun t => (⟨modP (-t.coef), t.witnesses⟩ : Term)).map (Term.eval σ)).sum =
        -(ts2.map (Term.eval σ)).sum := by
      rw [List.map_map, ← sum_map_neg]
      congr 1
      apply List.map_congr_left
      intro t _
      simp [Term.eval, modP, ZMod.intCast_mod]
    split_ifs
    · rw [hneg, h2, neg_eq_zero]
    · rw [h2]

end AcirLean
