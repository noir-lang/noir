/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Overflow

/-! Soundness of the pinned constraint lists, from the gadget lemmas. -/

namespace AcirLean

theorem divVarT_sound {n : ℕ} (hn : n ≤ 126) (σ : ℕ → F) (h : AllSat σ (divVarT n))
    (hb : (σ 1).val < 2 ^ n) :
    (σ 3).val = (σ 0).val / (σ 1).val ∧ (σ 4).val = (σ 0).val % (σ 1).val := by
  have g : ∀ c, c ∈ divVarT n → c.sat σ := h
  have e1 := g (.zero [⟨1, []⟩, ⟨-1, [1, 2]⟩]) (by simp [divVarT])
  have r3 := g (.range 3 n) (by simp [divVarT])
  have r4 := g (.range 4 n) (by simp [divVarT])
  have e2 := g (.zero [⟨1, []⟩, ⟨-1, [1]⟩, ⟨1, [4]⟩, ⟨1, [5]⟩]) (by simp [divVarT])
  have r5 := g (.range 5 n) (by simp [divVarT])
  have e3 := g (.zero [⟨1, [0]⟩, ⟨-1, [1, 3]⟩, ⟨-1, [4]⟩]) (by simp [divVarT])
  simp only [Cstr.sat, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at e1 e2 e3 r3 r4 r5
  have ht : σ 5 = σ 1 - (σ 4 + 1) := by push_cast at e2; linear_combination e2
  refine div_var_sound hn hb (inv := σ 2) ⟨?_, r3, r4, ?_, ?_⟩
  · push_cast at e1; linear_combination -e1
  · rw [← ht]; exact r5
  · push_cast at e3; linear_combination e3

theorem truncT_sound {k : ℕ} (hk1 : 2 ≤ k) (hk : k ≤ 125) (σ : ℕ → F)
    (h : AllSat σ (truncT k)) : (σ 2).val = (σ 0).val % 2 ^ k := by
  have g : ∀ c, c ∈ truncT k → c.sat σ := h
  have hbits : bits (2 ^ k) = k + 1 := Nat.size_pow
  have hmq : 254 - bits (2 ^ k) + 1 = 254 - k := by rw [hbits]; omega
  have hmr : bits (2 ^ k - 1) = k := by
    have h2 : 2 ^ (k - 1) * 2 = 2 ^ k := by rw [← pow_succ]; congr 1; omega
    have h1 := Nat.one_le_two_pow (n := k - 1)
    have hle : bits (2 ^ k - 1) ≤ k := Nat.size_le.2 (by omega)
    have hlt : k - 1 < bits (2 ^ k - 1) := Nat.lt_size.2 (by omega)
    omega
  have r1 := g (.range 1 (254 - k)) (by simp [truncT])
  have r2 := g (.range 2 k) (by simp [truncT])
  have e1 := g (.zero [⟨1, [0]⟩, ⟨-(2 ^ k : ℕ), [1]⟩, ⟨-1, [2]⟩]) (by simp [truncT])
  have e2 := g (.zero [⟨(q0 k : ℤ), []⟩, ⟨-1, [1]⟩, ⟨-1, [3]⟩]) (by simp [truncT])
  have r3 := g (.range 3 (254 - k)) (by simp [truncT])
  have e3 := g (.zero [⟨1, []⟩, ⟨1, [1, 4]⟩, ⟨-(q0 k : ℤ), [4]⟩, ⟨-1, [5]⟩])
    (by simp [truncT])
  have e4 := g (.zero [⟨1, [1, 5]⟩, ⟨-(q0 k : ℤ), [5]⟩]) (by simp [truncT])
  have e5 := g (.zero [⟨1, [2, 5]⟩, ⟨(R' k : ℤ), [5]⟩, ⟨-1, [6]⟩]) (by simp [truncT])
  have r6 := g (if N' k = 0 then .zero [⟨1, [6]⟩] else .range 6 (N' k)) (by simp [truncT])
  simp only [Cstr.sat, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at e1 e2 e3 e4 e5 r1 r2 r3
  push_cast at e1 e2 e3 e4 e5
  have hu : Range (σ 6) (N' k) := by
    split_ifs at r6 with h0
    · simp only [Cstr.sat, Term.eval, List.map, List.prod_cons, List.prod_nil,
        List.sum_cons, List.sum_nil] at r6
      push_cast at r6
      have : σ 6 = 0 := by linear_combination r6
      simp [Range, this, h0]
    · exact r6
  have ht : σ 3 = ((q0 k : ℕ) : F) - σ 1 := by linear_combination -e2
  unfold q0 at ht e3 e4
  refine truncate_field_sound hk1 hk (y := σ 5) (z := σ 4)
    ⟨⟨by rw [hmq]; exact r1, by rw [hmr]; exact r2, ?_, ?_⟩, ?_, ⟨?_, ?_⟩, ?_⟩
  · -- the remainder bound for `c = 2^k < 2^128` is `Range r k` (the offset is 0)
    unfold BoundConst
    rw [if_pos (Nat.pow_lt_pow_right (by norm_num) (by omega)), hmr]
    simpa using r2
  · push_cast; linear_combination e1
  · rw [hmq, ← ht]; exact r3
  · linear_combination -e3
  · linear_combination -e4
  · have hu' : σ 6 = (σ 2 + ((2 ^ bits (p % 2 ^ k - 1) - p % 2 ^ k : ℕ) : F)) * σ 5 := by
      unfold R' N' M at e5; linear_combination -e5
    rw [← hu']; exact hu

end AcirLean
