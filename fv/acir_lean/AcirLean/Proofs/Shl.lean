/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Shift

/-! `shl` by a constant on `u<n>`: `a · 2^c` truncated to `n` bits. -/

namespace AcirLean

/-- `2^c a = 2^n q + r` with `a`, `r` below `2^n` and `q` below `2^c`, and no
wraparound (`n + c ≤ 253`): `r` is `a · 2^c mod 2^n`. -/
theorem shl_core {n c : ℕ} (hnc : n + c ≤ 253) {a q r : F} (h : (2 : F) ^ c * a = 2 ^ n * q + r)
    (ha : a.val < 2 ^ n) (hq : q.val < 2 ^ c) (hr : r.val < 2 ^ n) :
    r.val = a.val * 2 ^ c % 2 ^ n := by
  have hp : (2 : ℕ) ^ 253 < p := by norm_num [p]
  have hAC : 2 ^ n * 2 ^ c ≤ 2 ^ 253 := by
    rw [← pow_add]; exact Nat.pow_le_pow_right (by norm_num) hnc
  have hc : 2 ^ c ≤ 2 ^ 253 := Nat.pow_le_pow_right (by norm_num) (by omega)
  have hn : 2 ^ n ≤ 2 ^ 253 := Nat.pow_le_pow_right (by norm_num) (by omega)
  have v2 : ∀ k, 2 ^ k ≤ 2 ^ 253 → ((2 : F) ^ k).val = 2 ^ k := fun k hk => by
    rw [show (2 : F) ^ k = ((2 ^ k : ℕ) : F) by push_cast; rfl, ZMod.val_natCast,
      Nat.mod_eq_of_lt (by omega)]
  have hpos : 0 < 2 ^ c := Nat.two_pow_pos c
  have hl : ((2 : F) ^ c * a).val = 2 ^ c * a.val := by
    rw [ZMod.val_mul_of_lt, v2 c hc]
    rw [v2 c hc]
    calc 2 ^ c * a.val < 2 ^ c * 2 ^ n := Nat.mul_lt_mul_of_pos_left ha hpos
      _ = 2 ^ n * 2 ^ c := Nat.mul_comm _ _
      _ ≤ 2 ^ 253 := hAC
      _ < p := hp
  have hqr : 2 ^ n * q.val + r.val < 2 ^ n * 2 ^ c := by
    have : 2 ^ n * q.val + 2 ^ n ≤ 2 ^ n * 2 ^ c := by
      rw [← Nat.mul_succ]; exact Nat.mul_le_mul_left _ hq
    omega
  have hmul : ((2 : F) ^ n * q).val = 2 ^ n * q.val := by
    rw [ZMod.val_mul_of_lt, v2 n hn]
    rw [v2 n hn]; omega
  have hrr : ((2 : F) ^ n * q + r).val = 2 ^ n * q.val + r.val := by
    rw [ZMod.val_add_of_lt (by rw [hmul]; omega), hmul]
  have e : 2 ^ c * a.val = 2 ^ n * q.val + r.val := by rw [← hl, ← hrr, h]
  rw [Nat.mul_comm, e, Nat.mul_add_mod, Nat.mod_eq_of_lt hr]

/-- `Term.eval` ignores witness `0`'s value for a term that doesn't mention it. -/
theorem eval_update0 (σ : ℕ → F) (x : F) (t : Term) (h : 0 ∉ t.witnesses) :
    Term.eval (fun i => if i = 0 then x else σ i) t = Term.eval σ t := by
  unfold Term.eval
  congr 1
  congr 1
  apply List.map_congr_left
  intro i hi
  have : i ≠ 0 := by rintro rfl; exact h hi
  rw [if_neg this]

theorem holds_update0 (σ : ℕ → F) (x : F) (o : Opcode)
    (h : ∀ t, (match o with | .assertZero ts => t ∈ ts | .range _ _ => False) → 0 ∉ t.witnesses)
    (hr : ∀ w k, o = .range w k → w ≠ 0) :
    o.Holds (fun i => if i = 0 then x else σ i) ↔ o.Holds σ := by
  cases o with
  | assertZero ts =>
    simp only [Opcode.Holds]
    rw [List.map_congr_left (fun t ht => eval_update0 σ x t (h t ht))]
  | range w k =>
    simp only [Opcode.Holds, if_neg (hr w k rfl)]

/-- The 128-bit truncation of `d · x` (`truncGadget128 126 d`) is the 128-bit
truncation of a witness holding `d · x`. -/
theorem truncGadget128_scaled (σ : ℕ → F) (d : ℕ) (h : AllHold σ (truncGadget128 126 d)) :
    (σ 2).val = ((d : F) * σ 0).val % 2 ^ 128 := by
  let σ' : ℕ → F := fun i => if i = 0 then (d : F) * σ 0 else σ i
  have h' : AllHold σ' (truncateGadget 128) := by
    intro o ho
    unfold truncateGadget at ho
    rw [if_pos rfl] at ho
    by_cases hid : o = .assertZero [⟨1, [0]⟩, ⟨-(2 ^ 128 : ℕ), [1]⟩, ⟨-1, [2]⟩]
    · subst hid
      have e := h (.assertZero [⟨(d : ℤ), [0]⟩, ⟨-(2 ^ 128 : ℕ), [1]⟩, ⟨-1, [2]⟩]) (by simp [truncGadget128])
      simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
        List.sum_cons, List.sum_nil] at e ⊢
      simp only [σ', if_pos rfl, show (1 : ℕ) ≠ 0 by omega, show (2 : ℕ) ≠ 0 by omega, if_false]
      push_cast at e ⊢
      linear_combination e
    · have hm : o ∈ truncGadget128 126 d := by
        simp only [truncGadget128, List.mem_cons, List.not_mem_nil, or_false] at ho ⊢
        rcases ho with h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 <;>
          subst h1 <;> first | exact absurd rfl hid | simp
      refine (holds_update0 σ _ o ?_ ?_).2 (h o hm)
      · intro t ht
        simp only [List.mem_cons, List.not_mem_nil, or_false] at ho
        rcases ho with h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 <;>
          subst h1 <;> simp only [List.mem_cons, List.not_mem_nil, or_false] at ht <;>
          first
            | exact absurd rfl hid
            | exact ht.elim
            | (rcases ht with rfl | rfl | rfl | rfl <;> decide)
      · intro w k hwk
        simp only [List.mem_cons, List.not_mem_nil, or_false] at ho
        rcases ho with h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 | h1 <;>
          subst h1 <;> (cases hwk <;> omega)
  have := truncateGadget128_sound σ' h'
  simpa [σ'] using this

end AcirLean

namespace AcirLean

private theorem v2pow {k : ℕ} (hk : k ≤ 253) : ((2 : F) ^ k).val = 2 ^ k := by
  have : (2 : ℕ) ^ k < p := lt_of_le_of_lt (Nat.pow_le_pow_right (by norm_num) hk) (by norm_num [p])
  rw [show (2 : F) ^ k = ((2 ^ k : ℕ) : F) by push_cast; rfl, ZMod.val_natCast, Nat.mod_eq_of_lt this]

/-- `(2^k · x).val = 2^k · x.val` when that stays below `2^254`'s neighbourhood. -/
private theorem val_pow_mul {k : ℕ} {x : F} (h : 2 ^ k * x.val ≤ 2 ^ 253) (hk : k ≤ 253) :
    ((2 : F) ^ k * x).val = 2 ^ k * x.val := by
  rw [ZMod.val_mul_of_lt, v2pow hk]
  rw [v2pow hk]; exact lt_of_le_of_lt h (by norm_num [p])

theorem acirGenShl_sound {n c : ℕ} (hn : n ∈ pinnedWidths) (hc : 1 ≤ c) (hcn : c < n) :
    SoundFunction (acirGenShl n c) (ShlOp n c) := by
  intro σ h
  have hbd := pinned_bounds hn
  simp only [acirGenShl, ShlOp, List.map]
  by_cases h1 : n = 128 ∧ 126 ≤ c
  · obtain ⟨rfl, h126⟩ := h1
    simp only [acirGenShl, true_and, if_pos h126, allHold_append, allHold_rename] at h
    obtain ⟨⟨⟨hin, g1⟩, g2⟩, hl⟩ := h
    have ha := hin (.range 0 128) (by simp)
    have e := hl (.assertZero [⟨1, [1]⟩, ⟨-1, [10]⟩]) (by simp)
    simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
      List.sum_cons, List.sum_nil, Range] at e ha
    push_cast at e
    have s1 := truncGadget128_scaled _ _ g1
    have s2 := truncGadget128_scaled _ _ g2
    simp only [Function.comp, witnessAt, List.getD_cons_succ, List.getD_cons_zero,
      Nat.cast_pow, Nat.cast_ofNat] at s1 s2
    have hr1 : (σ 3).val < 2 ^ 128 := by rw [s1]; exact Nat.mod_lt _ (by positivity)
    have hp1 : 2 ^ 63 * (σ 0).val ≤ 2 ^ 253 := by
      calc 2 ^ 63 * (σ 0).val ≤ 2 ^ 63 * 2 ^ 128 := Nat.mul_le_mul_left _ ha.le
        _ ≤ 2 ^ 253 := by norm_num
    have hp2 : 2 ^ (c - 63) * (σ 3).val ≤ 2 ^ 253 := by
      calc 2 ^ (c - 63) * (σ 3).val ≤ 2 ^ (c - 63) * 2 ^ 128 := Nat.mul_le_mul_left _ hr1.le
        _ = 2 ^ (c - 63 + 128) := (pow_add _ _ _).symm
        _ ≤ 2 ^ 253 := Nat.pow_le_pow_right (by norm_num) (by omega)
    rw [val_pow_mul hp1 (by norm_num)] at s1
    rw [val_pow_mul hp2 (by omega)] at s2
    refine ⟨ha, ?_⟩
    rw [show σ 1 = σ 10 by linear_combination e, s2, s1, Nat.mul_mod, Nat.mod_mod,
      ← Nat.mul_mod, ← mul_assoc, ← pow_add, show c - 63 + 63 = c by omega, mul_comm]
  · by_cases h2 : n = 128
    · subst h2
      have hc125 : c ≤ 125 := by omega
      simp only [acirGenShl, true_and, if_neg (show ¬ 126 ≤ c by omega), if_true, allHold_append,
        allHold_rename] at h
      obtain ⟨⟨hin, g⟩, hl⟩ := h
      have ha := hin (.range 0 128) (by simp)
      have e := hl (.assertZero [⟨1, [1]⟩, ⟨-1, [3]⟩]) (by simp)
      have g7 : ∀ o ∈ (truncGadget128 c (2 ^ c)).take 7,
          o.Holds (σ ∘ witnessAt [0, 2, 3, 4, 5, 6, 7, 8]) := by
        intro o ho
        apply g
        split_ifs
        · rw [List.take_of_length_le (by simp [truncGadget128])]; exact List.mem_of_mem_take ho
        · exact ho
      have gq := g7 (.range 1 c) (by simp [truncGadget128])
      have gr := g7 (.range 2 128) (by simp [truncGadget128])
      have gi := g7 (.assertZero [⟨((2 ^ c : ℕ) : ℤ), [0]⟩, ⟨-(2 ^ 128 : ℕ), [1]⟩, ⟨-1, [2]⟩])
        (by simp [truncGadget128])
      simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
        List.sum_cons, List.sum_nil, Range, Function.comp, witnessAt, List.getD_cons_succ,
        List.getD_cons_zero] at e ha gq gr gi
      push_cast at e gi
      refine ⟨ha, ?_⟩
      rw [show σ 1 = σ 3 by linear_combination e]
      exact shl_core (by omega) (by linear_combination gi) ha gq gr
    · have hn64 : n ≤ 64 := by
        simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn; omega
      simp only [acirGenShl, if_neg h1, if_neg h2] at h
      have ha := h (.range 0 n) (by simp)
      have gq := h (.range 2 c) (by simp)
      have gr := h (.range 3 n) (by simp)
      have gi := h (.assertZero [⟨(2 ^ c : ℕ), [0]⟩, ⟨-(2 ^ n : ℕ), [2]⟩, ⟨-1, [3]⟩]) (by simp)
      have e := h (.assertZero [⟨1, [1]⟩, ⟨-1, [3]⟩]) (by simp)
      simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
        List.sum_cons, List.sum_nil, Range] at e ha gq gr gi
      push_cast at e gi
      refine ⟨ha, ?_⟩
      rw [show σ 1 = σ 3 by linear_combination e]
      exact shl_core (by omega) (by linear_combination gi) ha gq gr

theorem shippedShl_sound {n c : ℕ} (hn : n ∈ pinnedWidths) (hc : 1 ≤ c) (hcn : c < n) :
    SoundFunction (shippedShl n c) (ShlOp n c) :=
  fun σ h => acirGenShl_sound hn hc hcn σ ((allHold_dropRepeats σ _).1 h)


/-- `0 << c = 0` on `u128`: the truncation's guard witnesses for a zero
quotient (`2^128 - 1 - r`, `Rq + q`, and `1 / (q - q0)`, which is `-1/q0`). -/
def shlWitness128 (c : ℕ) : ℕ → F
  | 4 => ((2 ^ 128 - 1 : ℕ) : F)
  | 5 => if 124 ≤ c then ((Rq 128 : ℕ) : F) else 0
  | 6 => if 124 ≤ c then ((3208353812438112359720600922702651292911874477583872349061533756267802487029 : ℕ) : F) else 0
  | 11 => if 126 ≤ c then ((2 ^ 128 - 1 : ℕ) : F) else 0
  | 12 => if 126 ≤ c then ((Rq 128 : ℕ) : F) else 0
  | 13 => if 126 ≤ c then ((3208353812438112359720600922702651292911874477583872349061533756267802487029 : ℕ) : F) else 0
  | _ => 0

theorem acirGenShl_satisfiable {n c : ℕ} (hn : n ∈ pinnedWidths) (hc : 1 ≤ c) (hcn : c < n) :
    SatisfiableFunction (acirGenShl n c) := by
  by_cases h128 : n = 128
  · subst h128
    refine ⟨shlWitness128 c, ?_⟩
    interval_cases c <;> decide +kernel
  · refine ⟨fun _ => 0, ?_⟩
    intro o ho
    simp only [acirGenShl, if_neg (show ¬(n = 128 ∧ 126 ≤ c) by tauto), if_neg h128,
      List.mem_cons, List.not_mem_nil, or_false] at ho
    rcases ho with rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl <;>
      simp [Opcode.Holds, Term.eval, Range]

theorem shl_claims {n : ℕ} (hn : n ∈ pinnedWidths) : ∀ c ∈ (List.range n).tail,
    SoundFunction (acirGenShl n c) (ShlOp n c) ∧ SatisfiableFunction (acirGenShl n c) ∧
    SoundFunction (shippedShl n c) (ShlOp n c) ∧ SatisfiableFunction (shippedShl n c) := by
  intro c hc
  have hc' : 1 ≤ c ∧ c < n := by
    rcases n with _ | n
    · simp at hc
    · simp only [List.range_succ_eq_map, List.tail_cons, List.mem_map, List.mem_range] at hc
      omega
  obtain ⟨σ, hσ⟩ := acirGenShl_satisfiable hn hc'.1 hc'.2
  exact ⟨acirGenShl_sound hn hc'.1 hc'.2, ⟨σ, hσ⟩, shippedShl_sound hn hc'.1 hc'.2,
    ⟨σ, (allHold_dropRepeats σ _).2 hσ⟩⟩

end AcirLean
