/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Programs
import AcirLean.Proofs.Prime

/-! `eq` and `not` on `u<n>`, and `div` on `Field`, as ACIR generation compiles them. -/

namespace AcirLean

/-- The value of an `AssertZero` constraint, for the small constraints below. -/
private theorem holds_az {σ : ℕ → F} {ts : List Term} (h : (Opcode.assertZero ts).Holds σ) :
    (ts.map (Term.eval σ)).sum = 0 := h

theorem acirGenEq_sound {n : ℕ} (_hn : n ∈ pinnedWidths) :
    SoundFunction (acirGenEq n) (Computes2 n fun a b => if a = b then 1 else 0) := by
  intro σ h
  have get := fun c (hc : c ∈ (acirGenEq n).opcodes) => h c hc
  have ha := get (.range 0 n) (by simp [acirGenEq])
  have hb := get (.range 1 n) (by simp [acirGenEq])
  have e1 := holds_az (get (.assertZero [⟨1, [0]⟩, ⟨-1, [1]⟩, ⟨-1, [3]⟩]) (by simp [acirGenEq]))
  have e2 := holds_az (get (.assertZero [⟨1, []⟩, ⟨-1, [3, 4]⟩, ⟨-1, [5]⟩]) (by simp [acirGenEq]))
  have e3 := holds_az (get (.assertZero [⟨1, [3, 5]⟩]) (by simp [acirGenEq]))
  have e4 := holds_az (get (.assertZero [⟨1, [2]⟩, ⟨-1, [5]⟩]) (by simp [acirGenEq]))
  simp only [Opcode.Holds, Range] at ha hb
  simp only [Term.eval, List.map, List.prod_cons, List.prod_nil, List.sum_cons, List.sum_nil] at e1 e2 e3 e4
  push_cast at e1 e2 e3 e4
  simp only [acirGenEq, List.map, Computes2]
  refine ⟨ha, hb, ?_⟩
  have hr : σ 2 = σ 5 := by linear_combination e4
  by_cases hab : (σ 0).val = (σ 1).val
  · have h01 : σ 0 = σ 1 := ZMod.val_injective _ hab
    have hd : σ 3 = 0 := by linear_combination -e1 + h01
    have he : σ 5 = 1 := by rw [hd] at e2; linear_combination -e2
    rw [if_pos hab, hr, he, ZMod.val_one]
  · have hd : σ 3 ≠ 0 := by
      intro hd
      exact hab (congrArg ZMod.val (show σ 0 = σ 1 by linear_combination e1 + hd))
    have he : σ 5 = 0 := by
      have : σ 3 * σ 5 = 0 := by linear_combination e3
      exact (mul_eq_zero.1 this).resolve_left hd
    rw [if_neg hab, hr, he, ZMod.val_zero]

theorem acirGenNot_sound {n : ℕ} (hn : n ∈ pinnedWidths) :
    SoundFunction (acirGenNot n) (NotOp n) := by
  intro σ h
  have ha := h (.range 0 n) (by simp [acirGenNot])
  have e := holds_az (h (.assertZero [⟨(2 ^ n - 1 : ℕ), []⟩, ⟨-1, [0]⟩, ⟨-1, [1]⟩]) (by simp [acirGenNot]))
  simp only [Opcode.Holds, Range] at ha
  simp only [Term.eval, List.map, List.prod_cons, List.prod_nil, List.sum_cons, List.sum_nil] at e
  push_cast at e
  simp only [acirGenNot, List.map, NotOp]
  refine ⟨ha, ?_⟩
  have hbd := pinned_bounds hn
  have hlt : 2 ^ n - 1 - (σ 0).val < p := by
    have : 2 ^ n ≤ 2 ^ 128 := Nat.pow_le_pow_right (by norm_num) hbd.2
    have : (2 : ℕ) ^ 128 < p := by norm_num [p]
    omega
  have h1 : σ 1 = ((2 ^ n - 1 - (σ 0).val : ℕ) : F) := by
    rw [Nat.cast_sub (by omega), ZMod.natCast_zmod_val]
    linear_combination -e
  rw [h1, ZMod.val_natCast, Nat.mod_eq_of_lt hlt]

theorem acirGenFieldDiv_sound : SoundFunction acirGenFieldDiv FieldDivOp := by
  intro σ h
  have e1 := holds_az (h (.assertZero [⟨1, []⟩, ⟨-1, [1, 3]⟩]) (by simp [acirGenFieldDiv]))
  have e2 := holds_az (h (.assertZero [⟨1, [0, 3]⟩, ⟨-1, [2]⟩]) (by simp [acirGenFieldDiv]))
  simp only [Term.eval, List.map, List.prod_cons, List.prod_nil, List.sum_cons, List.sum_nil] at e1 e2
  push_cast at e1 e2
  simp only [acirGenFieldDiv, List.map, FieldDivOp]
  refine ⟨?_, ?_⟩
  · intro h0
    rw [(ZMod.val_eq_zero _).1 h0] at e1
    simp at e1
  · rw [← ZMod.val_mul, show σ 2 * σ 1 = σ 0 by linear_combination (-σ 0) * e1 - σ 1 * e2]

/-- `0 == 0`: the difference is `0`, so the result is `1`. -/
def acirEqWitness : ℕ → F
  | 2 | 5 => 1
  | _ => 0

/-- `!0 = 2^n - 1`. -/
def acirNotWitness (n : ℕ) : ℕ → F
  | 1 => ((2 ^ n - 1 : ℕ) : F)
  | _ => 0

/-- `1 / 1 = 1`. -/
def acirFieldDivWitness : ℕ → F
  | 0 | 1 | 2 | 3 => 1
  | _ => 0

theorem eqNotFieldDiv_satisfiable {n : ℕ} (hn : n ∈ pinnedWidths) :
    SatisfiableFunction (acirGenEq n) ∧ SatisfiableFunction (acirGenNot n) ∧
      SatisfiableFunction acirGenFieldDiv := by
  refine ⟨⟨acirEqWitness, ?_⟩, ⟨acirNotWitness n, ?_⟩, ⟨acirFieldDivWitness, by decide +kernel⟩⟩ <;>
  simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn <;>
  rcases hn with rfl | rfl | rfl | rfl | rfl <;> decide +kernel

end AcirLean
