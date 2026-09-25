/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Extended
import AcirLean.Proofs.Signed
import AcirLean.Proofs.Satisfiable

/-! Whole compiled functions, from the gadget theorems: renaming witnesses
preserves satisfaction, so each gadget's theorem applies at its new indices. -/

namespace AcirLean

theorem Term.eval_rename (σ : ℕ → F) (f : ℕ → ℕ) (t : Term) :
    (t.rename f).eval σ = t.eval (σ ∘ f) := by
  simp [Term.rename, Term.eval, List.map_map]

theorem Opcode.holds_rename (σ : ℕ → F) (f : ℕ → ℕ) (c : Opcode) :
    (c.rename f).Holds σ ↔ c.Holds (σ ∘ f) := by
  cases c with
  | assertZero ts =>
    simp only [Opcode.rename, Opcode.Holds, List.map_map]
    rw [show Term.eval σ ∘ Term.rename f = Term.eval (σ ∘ f) from
      funext (Term.eval_rename σ f)]
  | range w k => simp [Opcode.rename, Opcode.Holds]

theorem allHold_rename (σ : ℕ → F) (f : ℕ → ℕ) (cs : List Opcode) :
    AllHold σ (cs.map (Opcode.rename f)) ↔ AllHold (σ ∘ f) cs := by
  simp [AllHold, Opcode.holds_rename]

theorem allHold_append (σ : ℕ → F) (a b : List Opcode) :
    AllHold σ (a ++ b) ↔ AllHold σ a ∧ AllHold σ b := by
  simp only [AllHold, List.mem_append]
  exact ⟨fun h => ⟨fun c hc => h c (Or.inl hc), fun c hc => h c (Or.inr hc)⟩,
    fun ⟨h1, h2⟩ c hc => hc.elim (h1 c) (h2 c)⟩

theorem val_bit {u : F} (h : u.val < 2) : u = 0 ∨ u = 1 := by
  rcases Nat.lt_succ_iff_lt_or_eq.1 h with h0 | h1
  · left; exact (ZMod.val_eq_zero u).1 (by omega)
  · right; exact (ZMod.val_eq_one (by norm_num [p]) u).1 h1

/-- `u1` xor as ACIR generation emits it: `a + b - 2ab`. -/
theorem xor_bits {u v : F} (hu : u.val < 2) (hv : v.val < 2) :
    (u + v - 2 * u * v).val = u.val ^^^ v.val := by
  rcases val_bit hu with rfl | rfl <;> rcases val_bit hv with rfl | rfl <;>
    norm_num [ZMod.val_one]

/-- `1 - (a >= b)` is `a < b`. -/
theorem not_ge {g : F} {a b : ℕ} (hg : g.val = if b ≤ a then 1 else 0) :
    (1 - g).val = if a < b then 1 else 0 := by
  split_ifs at hg with h
  · rw [(ZMod.val_eq_one (by norm_num [p]) g).1 hg, sub_self, ZMod.val_zero]
    rw [if_neg (by omega)]
  · rw [(ZMod.val_eq_zero g).1 hg, sub_zero, ZMod.val_one, if_pos (by omega)]

theorem pinned_cases {n : ℕ} (hn : n ∈ pinnedWidths) : (8 ≤ n ∧ n ≤ 64) ∨ n = 128 := by
  simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn
  omega

theorem pinned_bounds {n : ℕ} (hn : n ∈ pinnedWidths) : 8 ≤ n ∧ n ≤ 128 := by
  simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn
  omega

/-! ### `div` -/

theorem acirGenDiv_sound {n : ℕ} (hn : n ∈ pinnedWidths) :
    SoundFunction (acirGenDiv n) (Computes2 n (SsaBinOp.eval .div)) := by
  intro σ h
  simp only [acirGenDiv, allHold_append, allHold_rename] at h
  obtain ⟨⟨hin, hg⟩, hl⟩ := h
  have ha := hin (.range 0 n) (by simp)
  have hb := hin (.range 1 n) (by simp)
  have e := hl (.assertZero [⟨1, [2]⟩, ⟨-1, [4]⟩]) (by simp)
  simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil, Range] at e ha hb
  push_cast at e
  have hq : ((σ ∘ witnessAt [0, 1, 3, 4, 5, 6, 7, 8, 9, 10]) 3).val =
      ((σ ∘ witnessAt [0, 1, 3, 4, 5, 6, 7, 8, 9, 10]) 0).val /
        ((σ ∘ witnessAt [0, 1, 3, 4, 5, 6, 7, 8, 9, 10]) 1).val := by
    rcases pinned_cases hn with h | rfl
    · exact (divVarGadget_sound (by omega) _ hg (by simpa [witnessAt] using hb)).1
    · exact (divVarGadget128_sound _ hg (by simpa [witnessAt] using hb)).1
  simp [witnessAt] at hq
  simp only [acirGenDiv, List.map, Computes2, SsaBinOp.eval]
  refine ⟨ha, hb, ?_⟩
  rw [show σ 2 = σ 4 by linear_combination e, hq]

/-! ### `lt` -/

theorem acirGenLt_sound {n : ℕ} (hn : n ∈ pinnedWidths) :
    SoundFunction (acirGenLt n) (Computes2 n (SsaBinOp.eval .lt)) := by
  intro σ h
  simp only [acirGenLt, allHold_append, allHold_rename] at h
  obtain ⟨⟨hin, hg⟩, hl⟩ := h
  have ha := hin (.range 0 n) (by simp)
  have hb := hin (.range 1 n) (by simp)
  have e := hl (.assertZero [⟨1, []⟩, ⟨-1, [2]⟩, ⟨-1, [3]⟩]) (by simp)
  simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil, Range] at e ha hb
  push_cast at e
  have hbd := pinned_bounds hn
  have hge := moreThanEqGadget_sound (by omega) hbd.2 _ hg (by simpa [witnessAt] using ha)
    (by simpa [witnessAt] using hb)
  simp [geSpec, witnessAt] at hge
  simp only [acirGenLt, List.map, Computes2, SsaBinOp.eval]
  refine ⟨ha, hb, ?_⟩
  rw [show σ 2 = 1 - σ 3 by linear_combination -e]
  exact not_ge hge

/-! ### truncation of a field element -/

theorem acirGenTruncate_sound {n : ℕ} (hn : n ∈ pinnedWidths) :
    SoundFunction (acirGenTruncate n) (Computes1 (· % 2 ^ n)) := by
  intro σ h
  simp only [acirGenTruncate, allHold_append, allHold_rename] at h
  obtain ⟨hg, hl⟩ := h
  have e := hl (.assertZero [⟨1, [1]⟩, ⟨-1, [3]⟩]) (by simp)
  simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at e
  push_cast at e
  have hr : ((σ ∘ witnessAt [0, 2, 3, 4, 5, 6, 7, 8]) 2).val =
      ((σ ∘ witnessAt [0, 2, 3, 4, 5, 6, 7, 8]) 0).val % 2 ^ n := by
    rcases pinned_cases hn with h | rfl
    · exact truncateGadget_sound (by omega) (by omega) _ hg
    · exact truncateGadget128_sound _ hg
  simp [witnessAt] at hr
  simp only [acirGenTruncate, List.map, Computes1]
  rw [show σ 1 = σ 3 by linear_combination e, hr]

/-! ### signed `lt`, end to end -/

/-- Dividing an `n`-bit value by `2^j`. -/
theorem divPow2Gadget_sound {n j : ℕ} (hj : 1 ≤ j) (hjn : j < n) (hn : n ≤ 128) (σ : ℕ → F)
    (h : AllHold σ (divPow2Gadget n j)) :
    (σ 1).val = (σ 0).val / 2 ^ j ∧ (σ 1).val < 2 ^ (n - j) := by
  have hbits : bits (2 ^ j) = j + 1 := Nat.size_pow
  have hr : bits (2 ^ j - 1) = j := bits_pow_sub_one hj
  have r1 := h (.range 1 (n - j)) (by simp [divPow2Gadget])
  have r2 := h (.range 2 j) (by simp [divPow2Gadget])
  have e := h (.assertZero [⟨1, [0]⟩, ⟨-(2 ^ j : ℕ), [1]⟩, ⟨-1, [2]⟩]) (by simp [divPow2Gadget])
  simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at e r1 r2
  push_cast at e
  have c : DivConstConstraints n (2 ^ j) (σ 0) (σ 1) (σ 2) := by
    refine ⟨by rw [hbits, show n - (j + 1) + 1 = n - j by omega]; exact r1,
      by rw [hr]; exact r2, ?_, by push_cast; linear_combination e⟩
    unfold BoundConst
    rw [if_pos (Nat.pow_lt_pow_right (by norm_num) (by omega)), hr]
    simpa using r2
  have hc2 : 2 ≤ 2 ^ j := by
    calc 2 = 2 ^ 1 := by norm_num
      _ ≤ 2 ^ j := Nat.pow_le_pow_right (by norm_num) hj
  exact ⟨(div_const_sound hc2 (by rw [hbits]; omega) c).1, r1⟩

theorem acirGenSignedLt_sound {n : ℕ} (hn : n ∈ pinnedWidths) :
    SoundFunction (acirGenSignedLt n) (Computes2 n fun a b => if toSigned n a < toSigned n b then 1 else 0) := by
  intro σ h
  have hbd := pinned_bounds hn
  simp only [acirGenSignedLt] at h ⊢
  generalize hx : (if n = 128 then 10 else 9) = x at h
  simp only [allHold_append, allHold_rename] at h
  obtain ⟨⟨⟨⟨hin, hd1⟩, hd2⟩, hge⟩, hl⟩ := h
  have ha := hin (.range 0 n) (by simp)
  have hb := hin (.range 1 n) (by simp)
  have ex := hl (.assertZero [⟨1, [3]⟩, ⟨-2, [3, 5]⟩, ⟨1, [5]⟩, ⟨-1, [x]⟩]) (by simp)
  have ey := hl (.assertZero [⟨1, []⟩, ⟨-1, [7]⟩, ⟨-1, [x + 1]⟩]) (by simp)
  have er := hl (.assertZero [⟨1, [2]⟩, ⟨-1, [x]⟩, ⟨2, [x, x + 1]⟩, ⟨-1, [x + 1]⟩]) (by simp)
  simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil, Range] at ex ey er ha hb
  push_cast at ex ey er
  -- the sign bits
  have s0 := divPow2Gadget_sound (by omega) (by omega) hbd.2 _ hd1
  have s1 := divPow2Gadget_sound (by omega) (by omega) hbd.2 _ hd2
  simp only [show n - (n - 1) = 1 by omega, pow_one] at s0 s1
  simp [witnessAt] at s0 s1
  have b0 : (σ 3).val < 2 := by omega
  have b1 : (σ 5).val < 2 := by omega
  -- the unsigned comparison
  have hg := moreThanEqGadget_sound (by omega) hbd.2 _ hge (by simpa [witnessAt] using ha)
    (by simpa [witnessAt] using hb)
  simp [geSpec, witnessAt] at hg
  -- the two xors
  have hxv : (σ x).val = (σ 0).val / 2 ^ (n - 1) ^^^ (σ 1).val / 2 ^ (n - 1) := by
    rw [show σ x = σ 3 + σ 5 - 2 * σ 3 * σ 5 by linear_combination -ex,
      xor_bits b0 b1, s0.1, s1.1]
  have hyv : (σ (x + 1)).val = if (σ 0).val < (σ 1).val then 1 else 0 := by
    rw [show σ (x + 1) = 1 - σ 7 by linear_combination -ey]; exact not_ge hg
  have hx2 : (σ x).val < 2 := by
    rw [show σ x = σ 3 + σ 5 - 2 * σ 3 * σ 5 by linear_combination -ex, xor_bits b0 b1]
    rcases val_bit b0 with h0 | h0 <;> rcases val_bit b1 with h1 | h1 <;>
      simp [h0, h1, ZMod.val_one]
  have hy2 : (σ (x + 1)).val < 2 := by rw [hyv]; split_ifs <;> norm_num
  simp only [List.map, Computes2]
  refine ⟨ha, hb, ?_⟩
  rw [show σ 2 = σ x + σ (x + 1) - 2 * σ x * σ (x + 1) by linear_combination er,
    xor_bits hx2 hy2, hxv, hyv, ← signedLtSsa_run]
  exact signedLtSsa_correct (by omega) _ _ ha hb

end AcirLean

namespace AcirLean

/-! ### Honest witnesses for the compiled functions -/

instance instDecidableAllSatFn (σ : ℕ → F) (f : Circuit) : Decidable (AllHold σ f.opcodes) :=
  instDecidableAllSat σ f.opcodes

/-- `0 / 1`: `inv = 1`, and at `u128` the divisor's low half `1`. -/
def acirDivWitness : ℕ → F
  | 1 | 3 | 10 => 1
  | _ => 0

/-- `0 < 0`: `0 >= 0` holds, so `lt` returns `0`. -/
def acirLtWitness : ℕ → F
  | 3 => 1
  | 5 => ((2 ^ 128 - 1 : ℕ) : F)
  | _ => 0

/-- `truncWitness` moved to the compiled function's indices; the result is `0`. -/
def acirTruncWitness (n : ℕ) : ℕ → F
  | 0 => truncWitness n 0
  | 1 => 0
  | i + 1 => truncWitness n i

/-- `0 < 0` on signed values: both sign bits `0`, `0 >= 0`, result `0`. -/
def acirSignedLtWitness (n : ℕ) : ℕ → F
  | 7 => 1
  | 9 => if n = 128 then ((2 ^ 128 - 1 : ℕ) : F) else 0
  | _ => 0

theorem acir_satisfiable {n : ℕ} (hn : n ∈ pinnedWidths) :
    SatisfiableFunction (acirGenDiv n) ∧ SatisfiableFunction (acirGenLt n) ∧
      SatisfiableFunction (acirGenTruncate n) ∧ SatisfiableFunction (acirGenSignedLt n) := by
  refine ⟨⟨acirDivWitness, ?_⟩, ⟨acirLtWitness, ?_⟩, ⟨acirTruncWitness n, ?_⟩,
    ⟨acirSignedLtWitness n, ?_⟩⟩ <;>
  simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn <;>
  rcases hn with rfl | rfl | rfl | rfl | rfl <;> decide +kernel

end AcirLean
