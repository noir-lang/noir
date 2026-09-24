/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Programs

/-! The optimized circuits: dropping repeated constraints keeps the same set of
constraints, and the inlined truncation guard is the zero-width range check it
replaces. -/

namespace AcirLean

theorem mem_dropRepeats {c : Cstr} : ∀ {l : List Cstr}, c ∈ dropRepeats l ↔ c ∈ l
  | [] => by simp [dropRepeats]
  | d :: cs => by
    simp only [dropRepeats, List.mem_cons, List.mem_filter, bne_iff_ne, ne_eq]
    rw [mem_dropRepeats]
    by_cases h : c = d <;> simp [h]

theorem allSat_dropRepeats (σ : ℕ → F) (l : List Cstr) :
    AllSat σ (dropRepeats l) ↔ AllSat σ l := by
  simp only [AllSat, mem_dropRepeats]

theorem truncInlinedT_sound {n : ℕ} (hn1 : 2 ≤ n) (hn : n ≤ 125) (hN : N' n = 0) (σ : ℕ → F)
    (h : AllSat σ (truncInlinedT n)) : (σ 1).val = (σ 0).val % 2 ^ n := by
  have hbits : bits (2 ^ n) = n + 1 := Nat.size_pow
  have hmq : 254 - bits (2 ^ n) + 1 = 254 - n := by rw [hbits]; omega
  have hmr : bits (2 ^ n - 1) = n := bits_pow_sub_one (by omega)
  unfold truncInlinedT AllSat at h
  have r2 := h (.range 2 (254 - n)) (by simp)
  have r3 := h (.range 3 n) (by simp)
  have e1 := h (.zero [⟨1, [0]⟩, ⟨-(2 ^ n : ℕ), [2]⟩, ⟨-1, [3]⟩]) (by simp)
  have e2 := h (.zero [⟨(q0 n : ℤ), []⟩, ⟨-1, [2]⟩, ⟨-1, [4]⟩]) (by simp)
  have r4 := h (.range 4 (254 - n)) (by simp)
  have e3 := h (.zero [⟨1, []⟩, ⟨1, [2, 5]⟩, ⟨-(q0 n : ℤ), [5]⟩, ⟨-1, [6]⟩]) (by simp)
  have e4 := h (.zero [⟨1, [2, 6]⟩, ⟨-(q0 n : ℤ), [6]⟩]) (by simp)
  have e5 := h (.zero [⟨1, [3, 6]⟩, ⟨(R' n : ℤ), [6]⟩]) (by simp)
  have e6 := h (.zero [⟨1, [1]⟩, ⟨-1, [3]⟩]) (by simp)
  simp only [Cstr.sat, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at e1 e2 e3 e4 e5 e6 r2 r3 r4
  push_cast at e1 e2 e3 e4 e5 e6
  have ht : σ 4 = ((q0 n : ℕ) : F) - σ 2 := by linear_combination -e2
  unfold q0 at ht e3 e4
  have hr : (σ 3).val = (σ 0).val % 2 ^ n := by
    refine truncate_field_sound hn1 hn (q := σ 2) (y := σ 6) (z := σ 5)
      ⟨⟨by rw [hmq]; exact r2, by rw [hmr]; exact r3, ?_, ?_⟩, ?_, ⟨?_, ?_⟩, ?_⟩
    · unfold BoundConst
      rw [if_pos (Nat.pow_lt_pow_right (by norm_num) (by omega)), hmr]
      simpa using r3
    · push_cast; linear_combination e1
    · rw [hmq, ← ht]; exact r4
    · linear_combination -e3
    · linear_combination -e4
    · have hz : (σ 3 + ((2 ^ bits (p % 2 ^ n - 1) - p % 2 ^ n : ℕ) : F)) * σ 6 = 0 := by
        unfold R' N' M at e5; linear_combination e5
      unfold N' M at hN
      rw [hz, hN]; simp [Range]
  rw [show σ 1 = σ 3 by linear_combination e6, hr]

theorem shipped_sound {n : ℕ} (hn : n ∈ pinnedWidths) :
    SoundFn (shippedDivT n) (Computes2 n (BinOp.eval .div)) ∧
    SoundFn (shippedLtT n) (Computes2 n (BinOp.eval .lt)) ∧
    SoundFn (shippedTruncT n) (Computes1 (· % 2 ^ n)) ∧
    SoundFn (shippedSignedLtT n)
      (Computes2 n fun a b => if sint n a < sint n b then 1 else 0) := by
  refine ⟨fun σ h => acirDivT_sound hn σ ((allSat_dropRepeats σ _).1 h),
    fun σ h => acirLtT_sound hn σ ((allSat_dropRepeats σ _).1 h), ?_,
    fun σ h => acirSignedLtT_sound hn σ ((allSat_dropRepeats σ _).1 h)⟩
  intro σ h
  unfold shippedTruncT at h ⊢
  split_ifs at h with hc
  · have hn64 : n ≤ 64 := by rcases pinned_cases hn with h' | h' <;> [omega; exact absurd h' hc.1]
    have hbd := pinned_bounds hn
    simp only [acirTruncT, List.map, Computes1]
    exact truncInlinedT_sound (by omega) (by omega) hc.2 σ h
  · exact acirTruncT_sound hn σ ((allSat_dropRepeats σ _).1 h)

/-! ### Honest witnesses for the shipped circuits -/

theorem shipped_satisfiable {n : ℕ} (hn : n ∈ pinnedWidths) :
    SatisfiableFn (shippedDivT n) ∧ SatisfiableFn (shippedLtT n) ∧
      SatisfiableFn (shippedTruncT n) ∧ SatisfiableFn (shippedSignedLtT n) := by
  refine ⟨⟨acirDivWitness, ?_⟩, ⟨acirLtWitness, ?_⟩, ⟨acirTruncWitness n, ?_⟩,
    ⟨acirSignedLtWitness n, ?_⟩⟩ <;>
  simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn <;>
  rcases hn with rfl | rfl | rfl | rfl | rfl <;> decide +kernel

end AcirLean
