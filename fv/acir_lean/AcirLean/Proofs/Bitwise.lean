/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.EqNotFieldDiv

/-! `and`, `xor` and `or` on `u<n>`: the `AND` / `XOR` black boxes. -/

namespace AcirLean

/-- `!(!a & !b) = a | b` on `n`-bit values, with `!x = 2^n - 1 - x`. -/
theorem or_via_and {n a b : ℕ} (ha : a < 2 ^ n) (hb : b < 2 ^ n) :
    2 ^ n - 1 - ((2 ^ n - 1 - a) &&& (2 ^ n - 1 - b)) = a ||| b := by
  have hA : (BitVec.ofNat n a).toNat = a := by simp [Nat.mod_eq_of_lt ha]
  have hB : (BitVec.ofNat n b).toNat = b := by simp [Nat.mod_eq_of_lt hb]
  have key : ~~~(~~~(BitVec.ofNat n a) &&& ~~~(BitVec.ofNat n b)) =
      BitVec.ofNat n a ||| BitVec.ofNat n b := by
    ext i; simp
  have := congrArg BitVec.toNat key
  simpa [BitVec.toNat_not, BitVec.toNat_and, BitVec.toNat_or, hA, hB] using this

/-- `y = 2^n - 1 - x` in the field, for an `n`-bit `x`: the same as integers. -/
theorem not_val {n : ℕ} (hn : n ≤ 128) {x y : F} (hx : x.val < 2 ^ n)
    (e : ((2 ^ n - 1 : ℕ) : F) - x - y = 0) : y.val = 2 ^ n - 1 - x.val := by
  have hlt : 2 ^ n - 1 - x.val < p := by
    have : 2 ^ n ≤ 2 ^ 128 := Nat.pow_le_pow_right (by norm_num) hn
    have : (2 : ℕ) ^ 128 < p := by norm_num [p]
    omega
  have h1 : y = ((2 ^ n - 1 - x.val : ℕ) : F) := by
    rw [Nat.cast_sub (by omega), ZMod.natCast_zmod_val]
    linear_combination -e
  rw [h1, ZMod.val_natCast, Nat.mod_eq_of_lt hlt]

theorem acirGenBitwise_sound {n : ℕ} (xor : Bool) :
    SoundFunction (acirGenBitwise xor n)
      (Computes2 n (if xor then (· ^^^ ·) else (· &&& ·))) := by
  intro σ h
  have e := h (.assertZero [⟨1, [2]⟩, ⟨-1, [3]⟩]) (by simp [acirGenBitwise])
  simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at e
  push_cast at e
  have h23 : σ 2 = σ 3 := by linear_combination e
  simp only [acirGenBitwise, List.map, Computes2]
  cases xor with
  | false =>
    have hb := h (.and 0 1 n 3) (by simp [acirGenBitwise])
    simp only [Opcode.Holds, Range] at hb
    exact ⟨hb.1, hb.2.1, by rw [h23, hb.2.2]; rfl⟩
  | true =>
    have hb := h (.xor 0 1 n 3) (by simp [acirGenBitwise])
    simp only [Opcode.Holds, Range] at hb
    exact ⟨hb.1, hb.2.1, by rw [h23, hb.2.2]; rfl⟩

theorem shippedBitwise_sound {n : ℕ} (xor : Bool) :
    SoundFunction (shippedBitwise xor n)
      (Computes2 n (if xor then (· ^^^ ·) else (· &&& ·))) := by
  intro σ h
  have e := h (.assertZero [⟨1, [2]⟩, ⟨-1, [3]⟩]) (by simp [shippedBitwise, acirGenBitwise])
  simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at e
  push_cast at e
  have h23 : σ 2 = σ 3 := by linear_combination e
  simp only [shippedBitwise, acirGenBitwise, List.map, Computes2]
  cases xor with
  | false =>
    have hb := h (.and 0 1 n 3) (by simp [shippedBitwise, acirGenBitwise])
    simp only [Opcode.Holds, Range] at hb
    exact ⟨hb.1, hb.2.1, by rw [h23, hb.2.2]; rfl⟩
  | true =>
    have hb := h (.xor 0 1 n 3) (by simp [shippedBitwise, acirGenBitwise])
    simp only [Opcode.Holds, Range] at hb
    exact ⟨hb.1, hb.2.1, by rw [h23, hb.2.2]; rfl⟩

theorem acirGenOr_sound {n : ℕ} (hn : n ∈ pinnedWidths) :
    SoundFunction (acirGenOr n) (Computes2 n (· ||| ·)) := by
  intro σ h
  have hbd := pinned_bounds hn
  have ha := h (.range 0 n) (by simp [acirGenOr])
  have hb := h (.range 1 n) (by simp [acirGenOr])
  have e3 := h (.assertZero [⟨(2 ^ n - 1 : ℕ), []⟩, ⟨-1, [0]⟩, ⟨-1, [3]⟩]) (by simp [acirGenOr])
  have e4 := h (.assertZero [⟨(2 ^ n - 1 : ℕ), []⟩, ⟨-1, [1]⟩, ⟨-1, [4]⟩]) (by simp [acirGenOr])
  have hand := h (.and 3 4 n 5) (by simp [acirGenOr])
  have e6 := h (.assertZero [⟨(2 ^ n - 1 : ℕ), []⟩, ⟨-1, [2]⟩, ⟨-1, [5]⟩]) (by simp [acirGenOr])
  simp only [Opcode.Holds, Range, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil] at ha hb e3 e4 hand e6
  simp only [Int.cast_natCast, Int.cast_neg, Int.cast_one] at e3 e4 e6
  have v3 := not_val (y := σ 3) hbd.2 ha (by linear_combination e3)
  have v4 := not_val (y := σ 4) hbd.2 hb (by linear_combination e4)
  have h5 : (σ 5).val < 2 ^ n := by rw [hand.2.2]; exact lt_of_le_of_lt Nat.and_le_left hand.1
  have v2 := not_val (y := σ 2) hbd.2 h5 (by linear_combination e6)
  simp only [acirGenOr, List.map, Computes2]
  refine ⟨ha, hb, ?_⟩
  rw [v2, hand.2.2, v3, v4, or_via_and ha hb]

/-- `0 op 0 = 0`; for `or`, the negations `2^n - 1`. -/
def acirOrWitness (n : ℕ) : ℕ → F
  | 3 | 4 | 5 => ((2 ^ n - 1 : ℕ) : F)
  | _ => 0

theorem bitwise_satisfiable {n : ℕ} (hn : n ∈ pinnedWidths) :
    SatisfiableFunction (acirGenBitwise false n) ∧ SatisfiableFunction (acirGenBitwise true n) ∧
      SatisfiableFunction (shippedBitwise false n) ∧ SatisfiableFunction (shippedBitwise true n) ∧
      SatisfiableFunction (acirGenOr n) := by
  refine ⟨⟨fun _ => 0, ?_⟩, ⟨fun _ => 0, ?_⟩, ⟨fun _ => 0, ?_⟩, ⟨fun _ => 0, ?_⟩,
    ⟨acirOrWitness n, ?_⟩⟩ <;>
  simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn <;>
  rcases hn with rfl | rfl | rfl | rfl | rfl <;> decide +kernel

end AcirLean
