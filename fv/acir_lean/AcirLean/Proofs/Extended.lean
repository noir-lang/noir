/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Templates
import AcirLean.Proofs.Compare
import AcirLean.Spec.Claims

/-! Soundness of the pinned lists for `u128` division, predicated division,
truncation to `u128`, and comparison. -/

namespace AcirLean

/-- The `2^64` split `euclidean_division_var(x, 2^64, 128, _)` performs. -/
theorem split64 {x xu xr : F} (hu : Range xu 64) (hr : Range xr 64)
    (he : x = (2 : F) ^ 64 * xu + xr) : DivConstConstraints 128 (2 ^ 64) x xu xr := by
  have hbits : bits (2 ^ 64) = 65 := Nat.size_pow
  have hr64 : bits (2 ^ 64 - 1) = 64 := bits_pow_sub_one (by norm_num)
  refine ⟨by rw [hbits]; exact hu, by rw [hr64]; exact hr, ?_, ?_⟩
  · unfold BoundConst
    rw [if_pos (by norm_num), hr64]
    simpa using hr
  · rw [he]; push_cast; ring

section
attribute [local simp] Cstr.sat Term.eval List.map List.prod_cons List.prod_nil
  List.sum_cons List.sum_nil

theorem divVarT128_sound (σ : ℕ → F) (h : AllSat σ (divVarT 128))
    (hb : (σ 1).val < 2 ^ 128) :
    (σ 3).val = (σ 0).val / (σ 1).val ∧ (σ 4).val = (σ 0).val % (σ 1).val := by
  unfold divVarT AllSat at h
  rw [if_pos rfl] at h
  have e1 := h (.zero [⟨1, []⟩, ⟨-1, [1, 2]⟩]) (by simp)
  have r3 := h (.range 3 128) (by simp)
  have r4 := h (.range 4 128) (by simp)
  have e2 := h (.zero [⟨1, []⟩, ⟨-1, [1]⟩, ⟨1, [4]⟩, ⟨1, [5]⟩]) (by simp)
  have r5 := h (.range 5 128) (by simp)
  have e3 := h (.zero [⟨1, [0]⟩, ⟨-1, [1, 3]⟩, ⟨-1, [4]⟩]) (by simp)
  have r6 := h (.range 6 64) (by simp)
  have r7 := h (.range 7 64) (by simp)
  have e4 := h (.zero [⟨1, [3]⟩, ⟨-(2 ^ 64 : ℕ), [6]⟩, ⟨-1, [7]⟩]) (by simp)
  have r8 := h (.range 8 64) (by simp)
  have r9 := h (.range 9 64) (by simp)
  have e5 := h (.zero [⟨1, [1]⟩, ⟨-(2 ^ 64 : ℕ), [8]⟩, ⟨-1, [9]⟩]) (by simp)
  have e6 := h (.zero [⟨1, [6, 8]⟩]) (by simp)
  simp only [Cstr.sat, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at e1 e2 e3 e4 e5 e6 r3 r4 r5 r6 r7 r8 r9
  push_cast at e1 e2 e3 e4 e5 e6
  refine div_var128_sound hb (inv := σ 2) (qu := σ 6) (qr := σ 7) (bu := σ 8) (br := σ 9)
    ⟨⟨by linear_combination -e1, r3, r4, ?_, by linear_combination e3⟩,
      split64 r6 r7 (by linear_combination e4), split64 r8 r9 (by linear_combination e5),
      by linear_combination e6⟩
  have ht : σ 5 = σ 1 - (σ 4 + 1) := by linear_combination e2
  rw [← ht]; exact r5

/-- The prefix every `divPredT n` shares: with the predicate on, it is the
unpredicated constraint set with `inv = z`. -/
theorem divPred_base {n : ℕ} (σ : ℕ → F) (hp : σ 2 = 1)
    (h : ∀ c ∈ ([ .zero [⟨1, []⟩, ⟨-1, [1, 3]⟩, ⟨-1, [4]⟩],
      .zero [⟨1, [1, 4]⟩],
      .zero [⟨1, [2, 4]⟩],
      .range 5 n, .range 6 n, .range 5 n, .range 6 n,
      .zero [⟨1, [1]⟩, ⟨-1, [2]⟩, ⟨-1, [6]⟩, ⟨-1, [7]⟩],
      .range 7 n,
      .zero [⟨1, [1, 5]⟩, ⟨1, [6]⟩, ⟨-1, [8]⟩],
      .zero [⟨1, [0, 2]⟩, ⟨-1, [2, 8]⟩] ] : List Cstr), c.sat σ) :
    DivVarConstraints n (σ 0) (σ 1) (σ 5) (σ 6) (σ 3) := by
  have e1 := h (.zero [⟨1, []⟩, ⟨-1, [1, 3]⟩, ⟨-1, [4]⟩]) (by simp)
  have e3 := h (.zero [⟨1, [2, 4]⟩]) (by simp)
  have r5 := h (.range 5 n) (by simp)
  have r6 := h (.range 6 n) (by simp)
  have e4 := h (.zero [⟨1, [1]⟩, ⟨-1, [2]⟩, ⟨-1, [6]⟩, ⟨-1, [7]⟩]) (by simp)
  have r7 := h (.range 7 n) (by simp)
  have e5 := h (.zero [⟨1, [1, 5]⟩, ⟨1, [6]⟩, ⟨-1, [8]⟩]) (by simp)
  have e6 := h (.zero [⟨1, [0, 2]⟩, ⟨-1, [2, 8]⟩]) (by simp)
  simp only [Cstr.sat, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at e1 e3 e4 e5 e6 r5 r6 r7
  push_cast at e1 e3 e4 e5 e6
  rw [hp] at e3 e4 e6
  have hy : σ 4 = 0 := by linear_combination e3
  have ht : σ 7 = σ 1 - (σ 6 + 1) := by linear_combination -e4
  refine ⟨by rw [hy] at e1; linear_combination -e1, r5, r6, by rw [← ht]; exact r7, ?_⟩
  linear_combination e6 - e5

theorem divPredT_sound {n : ℕ} (hn : n ≤ 126) (σ : ℕ → F) (h : AllSat σ (divPredT n))
    (hb : (σ 1).val < 2 ^ n) : divPredSpec σ := by
  intro hp
  unfold divPredT AllSat at h
  rw [if_neg (by omega), List.append_nil] at h
  exact div_var_sound hn hb (divPred_base σ hp h)

theorem divPredT128_sound (σ : ℕ → F) (h : AllSat σ (divPredT 128))
    (hb : (σ 1).val < 2 ^ 128) : divPredSpec σ := by
  intro hp
  unfold divPredT AllSat at h
  rw [if_pos rfl] at h
  have base := divPred_base (n := 128) σ hp (fun c hc => h c (List.mem_append_left _ hc))
  have r9 := h (.range 9 64) (by simp)
  have r10 := h (.range 10 64) (by simp)
  have e7 := h (.zero [⟨1, [2, 5]⟩, ⟨-(2 ^ 64 : ℕ), [2, 9]⟩, ⟨-1, [2, 10]⟩]) (by simp)
  have r12 := h (.range 12 64) (by simp)
  have r13 := h (.range 13 64) (by simp)
  have e8 := h (.zero [⟨1, [1, 2]⟩, ⟨-(2 ^ 64 : ℕ), [2, 12]⟩, ⟨-1, [2, 13]⟩]) (by simp)
  have e9 := h (.zero [⟨1, [9, 12]⟩]) (by simp)
  simp only [Cstr.sat, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at e7 e8 e9 r9 r10 r12 r13
  push_cast at e7 e8 e9
  rw [hp] at e7 e8
  exact div_var128_sound hb (qu := σ 9) (qr := σ 10) (bu := σ 12) (br := σ 13)
    ⟨base, split64 r9 r10 (by linear_combination e7), split64 r12 r13 (by linear_combination e8),
      by linear_combination e9⟩

theorem truncT128_sound (σ : ℕ → F) (h : AllSat σ (truncT 128)) :
    (σ 2).val = (σ 0).val % 2 ^ 128 := by
  unfold truncT AllSat at h
  rw [if_pos rfl] at h
  have r1 := h (.range 1 126) (by simp)
  have r2 := h (.range 2 128) (by simp)
  have e1 := h (.zero [⟨(2 ^ 128 - 1 : ℕ), []⟩, ⟨-1, [2]⟩, ⟨-1, [3]⟩]) (by simp)
  have r3 := h (.range 3 128) (by simp)
  have e2 := h (.zero [⟨1, [0]⟩, ⟨-(2 ^ 128 : ℕ), [1]⟩, ⟨-1, [2]⟩]) (by simp)
  have e3 := h (.zero [⟨(Rq 128 : ℤ), []⟩, ⟨1, [1]⟩, ⟨-1, [4]⟩]) (by simp)
  have r4 := h (.range 4 (bits (q0 128))) (by simp)
  have e4 := h (.zero [⟨1, []⟩, ⟨1, [1, 5]⟩, ⟨-(q0 128 : ℤ), [5]⟩, ⟨-1, [6]⟩]) (by simp)
  have e5 := h (.zero [⟨1, [1, 6]⟩, ⟨-(q0 128 : ℤ), [6]⟩]) (by simp)
  have e6 := h (.zero [⟨1, [2, 6]⟩, ⟨(R' 128 : ℤ), [6]⟩, ⟨-1, [7]⟩]) (by simp)
  have r7 := h (.range 7 (N' 128)) (by simp)
  simp only [Cstr.sat, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at e1 e2 e3 e4 e5 e6 r1 r2 r3 r4 r7
  push_cast at e1 e2 e3 e4 e5 e6
  have hbits : bits (2 ^ 128) = 129 := Nat.size_pow
  have hr128 : bits (2 ^ 128 - 1) = 128 := bits_pow_sub_one (by norm_num)
  -- q ≤ q0: the range check on q + Rq cannot wrap
  have hq0 : q0 128 < 2 ^ 126 := by norm_num [q0, p]
  have hN : bits (q0 128) ≤ 126 := Nat.size_le.2 hq0
  have hq0N : q0 128 < 2 ^ bits (q0 128) := Nat.lt_size_self _
  have hNp : 2 ^ bits (q0 128) ≤ 2 ^ 126 := Nat.pow_le_pow_right (by norm_num) hN
  have hqle : (σ 1).val ≤ p / 2 ^ 128 := by
    have h4 : σ 4 = σ 1 + ((Rq 128 : ℕ) : F) := by linear_combination -e3
    unfold Range at r1 r4
    rw [h4, ZMod.val_add_of_lt, val_natCast_of_lt] at r4
    · unfold Rq at r4; unfold q0 at r4 hq0N; omega
    · unfold Rq; have := p_gt; omega
    · rw [val_natCast_of_lt (by unfold Rq; have := p_gt; omega)]
      unfold Rq; have := p_gt; omega
  have base : DivConstConstraints 254 (2 ^ 128) (σ 0) (σ 1) (σ 2) := by
    refine ⟨by rw [hbits]; exact r1, by rw [hr128]; exact r2, ?_, ?_⟩
    · unfold BoundConst
      rw [if_neg (by norm_num), hr128]
      have : ((2 ^ 128 : ℕ) : F) - (σ 2 + 1) = σ 3 := by push_cast; linear_combination e1
      rw [this]; exact r3
    · push_cast; linear_combination e2
  have hmod : 0 < p % 2 ^ 128 := by norm_num [p]
  have hmod' : p % 2 ^ 128 ≤ 2 ^ 128 := le_of_lt (Nat.mod_lt _ (by norm_num))
  refine (div_const_overflow_core (by norm_num) (by norm_num) hmod hmod' base hqle
    (y := σ 6) (z := σ 5) ⟨?_, ?_⟩ ?_).2
  · unfold q0 at e4; push_cast; linear_combination -e4
  · unfold q0 at e5; push_cast; linear_combination -e5
  · have hu : σ 7 = (σ 2 + ((2 ^ bits (p % 2 ^ 128 - 1) - p % 2 ^ 128 : ℕ) : F)) * σ 6 := by
      unfold R' N' M at e6; linear_combination -e6
    rw [← hu]; unfold N' M at r7; exact r7

theorem moreThanEqT_sound {m : ℕ} (hm : 1 ≤ m) (hm128 : m ≤ 128) (σ : ℕ → F)
    (h : AllSat σ (moreThanEqT m)) (ha : (σ 0).val < 2 ^ m) (hb : (σ 1).val < 2 ^ m) :
    geSpec σ := by
  have hbits : bits (2 ^ m) = m + 1 := Nat.size_pow
  have hmr : bits (2 ^ m - 1) = m := bits_pow_sub_one hm
  unfold moreThanEqT AllSat at h
  have hc : DivConstConstraints (m + 1) (2 ^ m) (σ 0 - σ 1 + ((2 ^ m : ℕ) : F)) (σ 2) (σ 3) := by
    split_ifs at h with h128
    · subst h128
      have r2 := h (.range 2 1) (by simp)
      have r3 := h (.range 3 128) (by simp)
      have e1 := h (.zero [⟨(2 ^ 128 - 1 : ℕ), []⟩, ⟨-1, [3]⟩, ⟨-1, [4]⟩]) (by simp)
      have r4 := h (.range 4 128) (by simp)
      have e2 := h (.zero [⟨(2 ^ 128 : ℕ), []⟩, ⟨1, [0]⟩, ⟨-1, [1]⟩, ⟨-(2 ^ 128 : ℕ), [2]⟩,
        ⟨-1, [3]⟩]) (by simp)
      simp only [Cstr.sat, Term.eval, List.map, List.prod_cons, List.prod_nil,
        List.sum_cons, List.sum_nil] at e1 e2 r2 r3 r4
      push_cast at e1 e2
      refine ⟨by rw [hbits]; simpa using r2, by rw [hmr]; exact r3, ?_, ?_⟩
      · unfold BoundConst
        rw [if_neg (by norm_num), hmr]
        have : ((2 ^ 128 : ℕ) : F) - (σ 3 + 1) = σ 4 := by push_cast; linear_combination e1
        rw [this]; exact r4
      · push_cast; linear_combination e2
    · have r2 := h (.range 2 1) (by simp)
      have r3 := h (.range 3 m) (by simp)
      have e2 := h (.zero [⟨(2 ^ m : ℕ), []⟩, ⟨1, [0]⟩, ⟨-1, [1]⟩, ⟨-(2 ^ m : ℕ), [2]⟩,
        ⟨-1, [3]⟩]) (by simp)
      simp only [Cstr.sat, Term.eval, List.map, List.prod_cons, List.prod_nil,
        List.sum_cons, List.sum_nil] at e2 r2 r3
      push_cast at e2
      refine ⟨by rw [hbits]; simpa using r2, by rw [hmr]; exact r3, ?_, ?_⟩
      · unfold BoundConst
        rw [if_pos (Nat.pow_lt_pow_right (by norm_num) (by omega)), hmr]
        simpa using r3
      · push_cast; linear_combination e2
  exact more_than_eq_sound hm hm128 ha hb hc

end

end AcirLean
