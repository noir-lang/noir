/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Programs
import AcirLean.Proofs.Prime
import AcirLean.Proofs.Satisfiable

/-! Signed `div` and `mod`, as shipped: the overflow check, the sign bits, the
absolute values, the unsigned division, and the sign fix-up. -/

namespace AcirLean

/-- The `eq` gadget: `y = 1 - t*z` and `t*y = 0` make `y` the flag `t == 0`.
The `t ≠ 0` direction needs `F` to be a field. -/
theorem isZero_flag {t y z : F} (h1 : 1 - t * z - y = 0) (h2 : t * y = 0) :
    (t = 0 → y = 1) ∧ (t ≠ 0 → y = 0) := by
  refine ⟨fun ht => by rw [ht] at h1; linear_combination -h1, fun ht => ?_⟩
  rcases mul_eq_zero.1 h2 with h | h
  · exact absurd h ht
  · exact h

/-- A bit and a low part: `x = N*s + r` with `s` one bit and `r < N`. -/
theorem sign_split {x s r : F} {N : ℕ} (hN : 2 * N < p) (hs : s.val < 2) (hr : r.val < N)
    (he : x = (N : F) * s + r) :
    (s = 0 ∧ x.val < N) ∨ (s = 1 ∧ N ≤ x.val ∧ x.val = N + r.val) := by
  rcases val_bit hs with rfl | rfl
  · left; refine ⟨rfl, ?_⟩; rw [he]; simpa using hr
  · right
    have : x.val = N + r.val := by
      rw [he, mul_one, ZMod.val_add_of_lt, val_natCast_of_lt (by omega)]
      rw [val_natCast_of_lt (by omega)]; omega
    exact ⟨rfl, by omega, this⟩

/-- `x - 2*x*s + 2N*s` is `|x|` for the two's-complement pattern `x`. -/
theorem abs_val {x s : F} {N : ℕ} (hN : 2 * N < p) (hx : x.val < 2 * N)
    (h : (s = 0 ∧ x.val < N) ∨ (s = 1 ∧ N ≤ x.val)) :
    (x - 2 * x * s + ((2 * N : ℕ) : F) * s).val = if x.val < N then x.val else 2 * N - x.val := by
  rcases h with ⟨rfl, hl⟩ | ⟨rfl, hl⟩
  · simp [hl]
  · rw [if_neg (by omega), show x - 2 * x * 1 + ((2 * N : ℕ) : F) * 1 = ((2 * N : ℕ) : F) - x by ring,
      ZMod.val_sub (by rw [val_natCast_of_lt hN]; omega), val_natCast_of_lt hN]

/-- `toSigned` as `|x|` and a sign. -/
theorem sint_cases {n x : ℕ} (hn : 1 ≤ n) (hx : x < 2 ^ n) :
    (x < 2 ^ (n - 1) ∧ toSigned n x = x) ∨
      (2 ^ (n - 1) ≤ x ∧ toSigned n x = -((2 ^ n - x : ℕ) : ℤ)) := by
  unfold toSigned
  split_ifs with h
  · exact Or.inl ⟨h, rfl⟩
  · right; refine ⟨by omega, ?_⟩
    push_cast [Nat.cast_sub (le_of_lt hx)]; ring

/-- `|a|` as the circuit computes it, for the pattern `a` of width `n`. -/
def absN (n a : ℕ) : ℕ := if a < 2 ^ (n - 1) then a else 2 ^ n - a

/-- What the shared prefix of the signed `div` and `mod` circuits establishes. -/
structure PrefixFacts (n : ℕ) (σ : ℕ → F) : Prop where
  ha : (σ 0).val < 2 ^ n
  hb : (σ 1).val < 2 ^ n
  no_overflow : ¬ ((σ 0).val = 2 ^ (n - 1) ∧ (σ 1).val = 2 ^ n - 1)
  sa : (σ 7 = 0 ∧ (σ 0).val < 2 ^ (n - 1)) ∨ (σ 7 = 1 ∧ 2 ^ (n - 1) ≤ (σ 0).val)
  sb : (σ 9 = 0 ∧ (σ 1).val < 2 ^ (n - 1)) ∨ (σ 9 = 1 ∧ 2 ^ (n - 1) ≤ (σ 1).val)
  hB : (σ 12).val = absN n (σ 1).val
  hB0 : absN n (σ 1).val ≠ 0
  hq : (σ 13).val = absN n (σ 0).val / absN n (σ 1).val
  hr : (σ 14).val = absN n (σ 0).val % absN n (σ 1).val

theorem two_pow_pred {n : ℕ} (hn : 1 ≤ n) : 2 ^ n = 2 * 2 ^ (n - 1) := by
  obtain ⟨k, rfl⟩ : ∃ k, n = k + 1 := ⟨n - 1, by omega⟩
  simp [pow_succ]; ring

theorem signedPrefix_facts {n : ℕ} (hn1 : 2 ≤ n) (hn : n ≤ 64) (σ : ℕ → F)
    (h : AllHold σ (signedDivModShared n)) : PrefixFacts n σ := by
  have h2 : 2 ^ n = 2 * 2 ^ (n - 1) := two_pow_pred (by omega)
  have hp : 2 ^ n < p := pow_lt_p (by omega)
  have hp2 : 2 * 2 ^ (n - 1) < p := by omega
  unfold signedDivModShared AllHold at h
  have r0 := h (.range 0 n) (by simp)
  have r1 := h (.range 1 n) (by simp)
  have e3 := h (.assertZero [⟨1, []⟩, ⟨-1, [0, 3]⟩, ⟨(2 ^ (n - 1) : ℕ), [3]⟩, ⟨-1, [4]⟩]) (by simp)
  have e4 := h (.assertZero [⟨1, [0, 4]⟩, ⟨-(2 ^ (n - 1) : ℕ), [4]⟩]) (by simp)
  have e5 := h (.assertZero [⟨1, []⟩, ⟨-1, [1, 5]⟩, ⟨(2 ^ n - 1 : ℕ), [5]⟩, ⟨-1, [6]⟩]) (by simp)
  have e6 := h (.assertZero [⟨1, [1, 6]⟩, ⟨-(2 ^ n - 1 : ℕ), [6]⟩]) (by simp)
  have e7 := h (.assertZero [⟨1, [4, 6]⟩]) (by simp)
  have r7 := h (.range 7 1) (by simp)
  have r8 := h (.range 8 (n - 1)) (by simp)
  have e8 := h (.assertZero [⟨1, [0]⟩, ⟨-(2 ^ (n - 1) : ℕ), [7]⟩, ⟨-1, [8]⟩]) (by simp)
  have r9 := h (.range 9 1) (by simp)
  have r10 := h (.range 10 (n - 1)) (by simp)
  have e9 := h (.assertZero [⟨1, [1]⟩, ⟨-(2 ^ (n - 1) : ℕ), [9]⟩, ⟨-1, [10]⟩]) (by simp)
  have e10 := h (.assertZero [⟨1, [1]⟩, ⟨-2, [1, 9]⟩, ⟨(2 ^ n : ℕ), [9]⟩, ⟨-1, [12]⟩]) (by simp)
  have e11 := h (.assertZero [⟨1, []⟩, ⟨-1, [11, 12]⟩]) (by simp)
  have r13 := h (.range 13 n) (by simp)
  have r14 := h (.range 14 n) (by simp)
  have e12 := h (.assertZero [⟨1, []⟩, ⟨-1, [12]⟩, ⟨1, [14]⟩, ⟨1, [15]⟩]) (by simp)
  have r15 := h (.range 15 n) (by simp)
  have e13 := h (.assertZero [⟨1, [0]⟩, ⟨-2, [0, 7]⟩, ⟨(2 ^ n : ℕ), [7]⟩, ⟨-1, [12, 13]⟩,
    ⟨-1, [14]⟩]) (by simp)
  simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil, Range, Int.cast_natCast, Int.cast_neg, Int.cast_one,
    Int.cast_ofNat, mul_one, one_mul, add_zero] at e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13
  simp only [Opcode.Holds, Range] at r0 r1 r7 r8 r9 r10 r13 r14 r15
  -- the sign bits
  have sa := sign_split (x := σ 0) (s := σ 7) (r := σ 8) (N := 2 ^ (n - 1)) hp2 (by omega) r8
    (by linear_combination e8)
  have sb := sign_split (x := σ 1) (s := σ 9) (r := σ 10) (N := 2 ^ (n - 1)) hp2 (by omega) r10
    (by linear_combination e9)
  -- |a| and |b|
  have hA := abs_val (x := σ 0) (s := σ 7) hp2 (by omega)
    (by rcases sa with ⟨h1, h2⟩ | ⟨h1, h2, _⟩ <;> [exact Or.inl ⟨h1, h2⟩; exact Or.inr ⟨h1, h2⟩])
  have hB := abs_val (x := σ 1) (s := σ 9) hp2 (by omega)
    (by rcases sb with ⟨h1, h2⟩ | ⟨h1, h2, _⟩ <;> [exact Or.inl ⟨h1, h2⟩; exact Or.inr ⟨h1, h2⟩])
  have hB12 : σ 12 = σ 1 - 2 * σ 1 * σ 9 + ((2 * 2 ^ (n - 1) : ℕ) : F) * σ 9 := by
    rw [← h2]; linear_combination -e10
  rw [← hB12, ← h2] at hB
  rw [← h2] at hA
  -- the unsigned division
  have hnz : σ 12 ≠ 0 := by intro h0; rw [h0] at e11; simp at e11
  have hB0 : (σ 12).val ≠ 0 := fun h0 => hnz ((ZMod.val_eq_zero _).1 h0)
  have hBn : (σ 12).val < 2 ^ n := by rw [hB]; split_ifs <;> omega
  have hdiv := div_var_sound (n := n) (by omega) hBn
    (a := σ 0 - 2 * σ 0 * σ 7 + ((2 ^ n : ℕ) : F) * σ 7) (q := σ 13) (r := σ 14) (inv := σ 11)
    ⟨by linear_combination -e11, r13, r14,
      by rw [show σ 12 - (σ 14 + 1) = σ 15 by linear_combination -e12]; exact r15,
      by linear_combination e13⟩
  rw [hA, hB] at hdiv
  -- the overflow check
  have hov : ¬ ((σ 0).val = 2 ^ (n - 1) ∧ (σ 1).val = 2 ^ n - 1) := by
    rintro ⟨ha1, hb1⟩
    have hz4 := (isZero_flag (t := σ 0 - ((2 ^ (n - 1) : ℕ) : F)) (y := σ 4) (z := σ 3)
      (by linear_combination e3) (by linear_combination e4)).1
      (by rw [← ha1, ZMod.natCast_zmod_val, sub_self])
    have hz6 := (isZero_flag (t := σ 1 - ((2 ^ n - 1 : ℕ) : F)) (y := σ 6) (z := σ 5)
      (by linear_combination e5) (by linear_combination e6)).1
      (by rw [← hb1, ZMod.natCast_zmod_val, sub_self])
    rw [hz4, hz6] at e7; simp at e7
  exact ⟨r0, r1, hov,
    by rcases sa with ⟨h1, h2⟩ | ⟨h1, h2, _⟩ <;> [exact Or.inl ⟨h1, h2⟩; exact Or.inr ⟨h1, h2⟩],
    by rcases sb with ⟨h1, h2⟩ | ⟨h1, h2, _⟩ <;> [exact Or.inl ⟨h1, h2⟩; exact Or.inr ⟨h1, h2⟩],
    by rw [hB]; rfl, by rw [← show (σ 12).val = absN n (σ 1).val by rw [hB]; rfl]; exact hB0,
    by rw [hdiv.1]; rfl, by rw [hdiv.2]; rfl⟩

theorem encode_nat {n Q : ℕ} (h : Q < 2 ^ n) : toBitPattern n Q = Q := by
  unfold toBitPattern
  rw [Int.emod_eq_of_lt (by omega) (by exact_mod_cast h)]
  simp

theorem encode_neg {n Q : ℕ} (h0 : 0 < Q) (h : Q ≤ 2 ^ n) :
    toBitPattern n (-(Q : ℤ)) = 2 ^ n - Q := by
  unfold toBitPattern
  have hc : ((2 ^ n - Q : ℕ) : ℤ) = (2 : ℤ) ^ n - Q := by push_cast [Nat.cast_sub h]; ring
  have : (-(Q : ℤ)) % 2 ^ n = ((2 ^ n - Q : ℕ) : ℤ) := by
    rw [show (-(Q : ℤ)) = ((2 ^ n - Q : ℕ) : ℤ) + (-1) * 2 ^ n by rw [hc]; ring,
      Int.add_mul_emod_self_right]
    have hQpos : (0 : ℤ) < Q := by exact_mod_cast h0
    exact Int.emod_eq_of_lt (by positivity) (by rw [hc]; linarith)
  rw [this]; simp

/-- A signed pattern's magnitude and sign. -/
theorem sint_abs {n x : ℕ} (hn : 1 ≤ n) (hx : x < 2 ^ n) :
    (x < 2 ^ (n - 1) ∧ absN n x = x ∧ toSigned n x = (absN n x : ℤ)) ∨
      (2 ^ (n - 1) ≤ x ∧ absN n x = 2 ^ n - x ∧ toSigned n x = -(absN n x : ℤ)) := by
  unfold absN
  rcases sint_cases hn hx with ⟨h1, h2⟩ | ⟨h1, h2⟩
  · exact Or.inl ⟨h1, if_pos h1, by rw [h2, if_pos h1]⟩
  · exact Or.inr ⟨h1, if_neg (by omega), by rw [h2, if_neg (by omega)]⟩

theorem absN_le {n x : ℕ} (hn : 1 ≤ n) (hx : x < 2 ^ n) : absN n x ≤ 2 ^ (n - 1) := by
  have := two_pow_pred hn
  unfold absN; split_ifs <;> omega

/-- The overflow case in terms of the patterns. -/
theorem overflow_iff {n a b : ℕ} (hn : 1 ≤ n) (ha : a < 2 ^ n) (hb : b < 2 ^ n) :
    (toSigned n a = -2 ^ (n - 1) ∧ toSigned n b = -1) → (a = 2 ^ (n - 1) ∧ b = 2 ^ n - 1) := by
  have h2 := two_pow_pred hn
  rintro ⟨h1, h2'⟩
  rcases sint_abs hn ha with ⟨_, ea, sa⟩ | ⟨la, ea, sa⟩ <;>
  rcases sint_abs hn hb with ⟨_, eb, sb⟩ | ⟨lb, eb, sb⟩
  · rw [sb] at h2'; have : (0 : ℤ) ≤ (absN n b : ℤ) := by positivity
    linarith
  · rw [sa] at h1; have : (0 : ℤ) ≤ (absN n a : ℤ) := by positivity
    have : (0 : ℤ) < 2 ^ (n - 1) := by positivity
    linarith
  · rw [sb] at h2'; have : (0 : ℤ) ≤ (absN n b : ℤ) := by positivity
    linarith
  · rw [sa] at h1; rw [sb] at h2'
    have e1 : (absN n a : ℤ) = 2 ^ (n - 1) := by linarith
    have e2 : (absN n b : ℤ) = 1 := by linarith
    have e1' : absN n a = 2 ^ (n - 1) := by exact_mod_cast e1
    have e2' : absN n b = 1 := by exact_mod_cast e2
    omega

theorem sint_of_lt {n x : ℕ} (hn : 1 ≤ n) (hx : x < 2 ^ n) (h : x < 2 ^ (n - 1)) :
    toSigned n x = (absN n x : ℤ) := by
  rcases sint_abs hn hx with ⟨_, _, e⟩ | ⟨h', _, _⟩
  · exact e
  · omega

theorem sint_of_ge {n x : ℕ} (hn : 1 ≤ n) (hx : x < 2 ^ n) (h : 2 ^ (n - 1) ≤ x) :
    toSigned n x = -(absN n x : ℤ) := by
  rcases sint_abs hn hx with ⟨h', _, _⟩ | ⟨_, _, e⟩
  · omega
  · exact e

/-- `q + 2(N - q)` is `2N - q`, the pattern of `-q`. -/
theorem flip_val {q : F} {N Q : ℕ} (hq : q.val = Q) (hQ : Q ≤ 2 * N) (hN : 2 * N < p) :
    (q + 2 * ((N : F) - q)).val = 2 * N - Q := by
  rw [show q + 2 * ((N : F) - q) = ((2 * N : ℕ) : F) - q by push_cast; ring,
    ZMod.val_sub (by rw [val_natCast_of_lt hN]; omega), val_natCast_of_lt hN, hq]

theorem tdiv_signs (A B : ℕ) :
    Int.tdiv (A : ℤ) (B : ℤ) = ((A / B : ℕ) : ℤ) ∧ Int.tdiv (A : ℤ) (-(B : ℤ)) = -((A / B : ℕ) : ℤ) ∧
      Int.tdiv (-(A : ℤ)) (B : ℤ) = -((A / B : ℕ) : ℤ) ∧
      Int.tdiv (-(A : ℤ)) (-(B : ℤ)) = ((A / B : ℕ) : ℤ) := by
  refine ⟨(Int.ofNat_tdiv A B).symm, ?_, ?_, ?_⟩
  · rw [Int.tdiv_neg, ← Int.ofNat_tdiv]
  · rw [Int.neg_tdiv, ← Int.ofNat_tdiv]
  · rw [Int.neg_tdiv, Int.tdiv_neg, neg_neg, ← Int.ofNat_tdiv]

theorem tmod_signs (A B : ℕ) :
    Int.tmod (A : ℤ) (B : ℤ) = ((A % B : ℕ) : ℤ) ∧ Int.tmod (A : ℤ) (-(B : ℤ)) = ((A % B : ℕ) : ℤ) ∧
      Int.tmod (-(A : ℤ)) (B : ℤ) = -((A % B : ℕ) : ℤ) ∧
      Int.tmod (-(A : ℤ)) (-(B : ℤ)) = -((A % B : ℕ) : ℤ) := by
  refine ⟨(Int.ofNat_tmod A B).symm, ?_, ?_, ?_⟩
  · rw [Int.tmod_neg, ← Int.ofNat_tmod]
  · rw [Int.neg_tmod, ← Int.ofNat_tmod]
  · rw [Int.neg_tmod, Int.tmod_neg, ← Int.ofNat_tmod]

theorem shippedSignedDiv_sound {n : ℕ} (hn : n ∈ signedWidths) :
    SoundFunction (shippedSignedDiv n) (SignedOp n Int.tdiv) := by
  intro σ h
  have hbd : 8 ≤ n ∧ n ≤ 64 := by
    simp only [signedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn; omega
  have h2 := two_pow_pred (n := n) (by omega)
  have hp : 2 ^ n < p := pow_lt_p (by omega)
  simp only [shippedSignedDiv, allHold_append] at h
  obtain ⟨hpre, ht⟩ := h
  have PF := signedPrefix_facts (by omega) hbd.2 σ hpre
  unfold AllHold at ht
  have t1 := ht (.assertZero [⟨(2 ^ (n - 1) : ℕ), []⟩, ⟨-1, [13]⟩, ⟨-1, [16]⟩]) (by simp)
  have t2 := ht (.assertZero [⟨1, [7]⟩, ⟨-2, [7, 9]⟩, ⟨1, [9]⟩, ⟨-1, [17]⟩]) (by simp)
  have t3 := ht (.assertZero [⟨1, []⟩, ⟨-1, [13, 18]⟩, ⟨-1, [19]⟩]) (by simp)
  have t4 := ht (.assertZero [⟨1, [13, 19]⟩]) (by simp)
  have t5 := ht (.assertZero [⟨1, [13]⟩, ⟨2, [16, 17]⟩, ⟨-1, [20]⟩]) (by simp)
  have t6 := ht (.assertZero [⟨1, []⟩, ⟨-1, [19]⟩, ⟨-1, [21]⟩]) (by simp)
  have t7 := ht (.assertZero [⟨1, [2]⟩, ⟨-1, [20, 21]⟩]) (by simp)
  simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil, Int.cast_natCast, Int.cast_neg, Int.cast_one,
    Int.cast_ofNat, mul_one, one_mul, add_zero] at t1 t2 t3 t4 t5 t6 t7
  simp only [List.map, SignedOp]
  have hA := absN_le (n := n) (by omega) PF.ha
  have hB := absN_le (n := n) (by omega) PF.hb
  refine ⟨PF.ha, PF.hb, ?_, fun ho => PF.no_overflow (overflow_iff (by omega) PF.ha PF.hb ho), ?_⟩
  · rcases sint_abs (n := n) (by omega) PF.hb with ⟨_, _, e⟩ | ⟨_, _, e⟩ <;> rw [e] <;>
      have := PF.hB0 <;> omega
  dsimp only
  have flag := isZero_flag (t := σ 13) (y := σ 19) (z := σ 18) (by linear_combination t3)
    (by linear_combination t4)
  have hret : σ 2 = (σ 13 + 2 * (((2 ^ (n - 1) : ℕ) : F) - σ 13) * σ 17) * (1 - σ 19) := by
    have e16 : σ 16 = ((2 ^ (n - 1) : ℕ) : F) - σ 13 := by linear_combination -t1
    have e20 : σ 20 = σ 13 + 2 * σ 16 * σ 17 := by linear_combination -t5
    have e21 : σ 21 = 1 - σ 19 := by linear_combination -t6
    rw [show σ 2 = σ 20 * σ 21 by linear_combination t7, e20, e21, e16]
  have hs : σ 17 = σ 7 + σ 9 - 2 * σ 7 * σ 9 := by linear_combination -t2
  obtain ⟨dpp, dpn, dnp, dnn⟩ := tdiv_signs (absN n (σ 0).val) (absN n (σ 1).val)
  have hQle : absN n (σ 0).val / absN n (σ 1).val ≤ 2 ^ (n - 1) :=
    le_trans (Nat.div_le_self _ _) hA
  have hp2 : 2 * 2 ^ (n - 1) < p := by omega
  rcases Nat.eq_zero_or_pos (absN n (σ 0).val / absN n (σ 1).val) with hQ0 | hQ0
  · have h13 : σ 13 = 0 := (ZMod.val_eq_zero _).1 (by rw [PF.hq]; exact hQ0)
    rw [hret, flag.1 h13, sub_self, mul_zero, ZMod.val_zero]
    rcases sint_abs (n := n) (by omega) PF.ha with ⟨_, _, ea⟩ | ⟨_, _, ea⟩ <;>
    rcases sint_abs (n := n) (by omega) PF.hb with ⟨_, _, eb⟩ | ⟨_, _, eb⟩ <;>
    rw [ea, eb] <;> (first | rw [dpp] | rw [dpn] | rw [dnp] | rw [dnn]) <;> rw [hQ0] <;> simp [toBitPattern]
  · have h13 : σ 13 ≠ 0 := fun h0 => by
      have := congrArg ZMod.val h0; rw [ZMod.val_zero, PF.hq] at this; omega
    rw [hret, flag.2 h13, sub_zero, mul_one, hs]
    rcases PF.sa with ⟨s7, la⟩ | ⟨s7, la⟩ <;> rcases PF.sb with ⟨s9, lb⟩ | ⟨s9, lb⟩ <;>
      rw [s7, s9]
    · rw [sint_of_lt (by omega) PF.ha la, sint_of_lt (by omega) PF.hb lb, dpp]
      simp only [mul_zero, zero_mul, add_zero, sub_zero]
      rw [PF.hq, encode_nat (by omega)]
    · rw [sint_of_lt (by omega) PF.ha la, sint_of_ge (by omega) PF.hb lb, dpn]
      simp only [mul_zero, zero_mul, add_zero, zero_add, sub_zero, mul_one]
      rw [flip_val PF.hq (by omega) hp2, ← h2, encode_neg hQ0 (by omega)]
    · rw [sint_of_ge (by omega) PF.ha la, sint_of_lt (by omega) PF.hb lb, dnp]
      simp only [mul_zero, zero_mul, add_zero, zero_add, sub_zero, mul_one]
      rw [flip_val PF.hq (by omega) hp2, ← h2, encode_neg hQ0 (by omega)]
    · rw [sint_of_ge (by omega) PF.ha la, sint_of_ge (by omega) PF.hb lb, dnn]
      simp only [mul_one, one_add_one_eq_two, sub_self, mul_zero, add_zero]
      rw [PF.hq, encode_nat (by omega)]

theorem shippedSignedMod_sound {n : ℕ} (hn : n ∈ signedWidths) :
    SoundFunction (shippedSignedMod n) (SignedOp n Int.tmod) := by
  intro σ h
  have hbd : 8 ≤ n ∧ n ≤ 64 := by
    simp only [signedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn; omega
  have h2 := two_pow_pred (n := n) (by omega)
  have hp : 2 ^ n < p := pow_lt_p (by omega)
  simp only [shippedSignedMod, allHold_append] at h
  obtain ⟨hpre, ht⟩ := h
  have PF := signedPrefix_facts (by omega) hbd.2 σ hpre
  unfold AllHold at ht
  have m1 := ht (.assertZero [⟨1, []⟩, ⟨-1, [14, 16]⟩, ⟨-1, [17]⟩]) (by simp)
  have m2 := ht (.assertZero [⟨1, [14, 17]⟩]) (by simp)
  have m3 := ht (.assertZero [⟨(2 ^ n : ℕ), [7]⟩, ⟨-2, [7, 14]⟩, ⟨1, [14]⟩, ⟨-1, [18]⟩]) (by simp)
  have m4 := ht (.assertZero [⟨1, []⟩, ⟨-1, [17]⟩, ⟨-1, [19]⟩]) (by simp)
  have m5 := ht (.assertZero [⟨1, [2]⟩, ⟨-1, [18, 19]⟩]) (by simp)
  simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil, Int.cast_natCast, Int.cast_neg, Int.cast_one,
    Int.cast_ofNat, mul_one, one_mul, add_zero] at m1 m2 m3 m4 m5
  simp only [List.map, SignedOp]
  have hA := absN_le (n := n) (by omega) PF.ha
  have hB := absN_le (n := n) (by omega) PF.hb
  refine ⟨PF.ha, PF.hb, ?_, fun ho => PF.no_overflow (overflow_iff (by omega) PF.ha PF.hb ho), ?_⟩
  · rcases sint_abs (n := n) (by omega) PF.hb with ⟨_, _, e⟩ | ⟨_, _, e⟩ <;> rw [e] <;>
      have := PF.hB0 <;> omega
  dsimp only
  have flag := isZero_flag (t := σ 14) (y := σ 17) (z := σ 16) (by linear_combination m1)
    (by linear_combination m2)
  have hret : σ 2 = (σ 14 - 2 * σ 7 * σ 14 + ((2 ^ n : ℕ) : F) * σ 7) * (1 - σ 17) := by
    rw [show σ 2 = σ 18 * σ 19 by linear_combination m5,
      show σ 18 = σ 14 - 2 * σ 7 * σ 14 + ((2 ^ n : ℕ) : F) * σ 7 by linear_combination -m3,
      show σ 19 = 1 - σ 17 by linear_combination -m4]
  obtain ⟨mpp, mpn, mnp, mnn⟩ := tmod_signs (absN n (σ 0).val) (absN n (σ 1).val)
  have hRlt : absN n (σ 0).val % absN n (σ 1).val < 2 ^ (n - 1) :=
    lt_of_lt_of_le (Nat.mod_lt _ (Nat.pos_of_ne_zero PF.hB0)) hB
  rcases Nat.eq_zero_or_pos (absN n (σ 0).val % absN n (σ 1).val) with hR0 | hR0
  · have h14 : σ 14 = 0 := (ZMod.val_eq_zero _).1 (by rw [PF.hr]; exact hR0)
    rw [hret, flag.1 h14, sub_self, mul_zero, ZMod.val_zero]
    rcases sint_abs (n := n) (by omega) PF.ha with ⟨_, _, ea⟩ | ⟨_, _, ea⟩ <;>
    rcases sint_abs (n := n) (by omega) PF.hb with ⟨_, _, eb⟩ | ⟨_, _, eb⟩ <;>
    rw [ea, eb] <;> (first | rw [mpp] | rw [mpn] | rw [mnp] | rw [mnn]) <;> rw [hR0] <;>
      simp [toBitPattern]
  · have h14 : σ 14 ≠ 0 := fun h0 => by
      have := congrArg ZMod.val h0; rw [ZMod.val_zero, PF.hr] at this; omega
    rw [hret, flag.2 h14, sub_zero, mul_one]
    rcases PF.sa with ⟨s7, la⟩ | ⟨s7, la⟩ <;> rw [s7] <;>
      rcases sint_abs (n := n) (by omega) PF.hb with ⟨_, _, eb⟩ | ⟨_, _, eb⟩
    · rw [sint_of_lt (by omega) PF.ha la, eb, mpp]
      simp only [mul_zero, zero_mul, add_zero, sub_zero]
      rw [PF.hr, encode_nat (by omega)]
    · rw [sint_of_lt (by omega) PF.ha la, eb, mpn]
      simp only [mul_zero, zero_mul, add_zero, sub_zero]
      rw [PF.hr, encode_nat (by omega)]
    · rw [sint_of_ge (by omega) PF.ha la, eb, mnp]
      rw [show σ 14 - 2 * 1 * σ 14 + ((2 ^ n : ℕ) : F) * 1 = ((2 ^ n : ℕ) : F) - σ 14 by ring,
        ZMod.val_sub (by rw [val_natCast_of_lt hp, PF.hr]; omega), val_natCast_of_lt hp, PF.hr,
        encode_neg hR0 (by omega)]
    · rw [sint_of_ge (by omega) PF.ha la, eb, mnn]
      rw [show σ 14 - 2 * 1 * σ 14 + ((2 ^ n : ℕ) : F) * 1 = ((2 ^ n : ℕ) : F) - σ 14 by ring,
        ZMod.val_sub (by rw [val_natCast_of_lt hp, PF.hr]; omega), val_natCast_of_lt hp, PF.hr,
        encode_neg hR0 (by omega)]

/-! ### Honest witnesses: `0 / 1` and `0 % 1` -/

/-- `1 / (0 - 2^(n-1))`, the `z` of the `a == MIN` check at `a = 0`. -/
def invNegHalf : ℕ → F
  | 8 => (171001897436244337673800044884822461629284096878250268310142220207623503872 : ℕ)
  | 16 => (667976161860329444038281425331337740739391003430665110586493047686029312 : ℕ)
  | 32 => (2736030369172416755385925291495193830211272306939684399021008289839509138216 : ℕ)
  | 64 => (10451182222713362943806261179797484296939952078258681390973810567089310802792 : ℕ)
  | _ => 0

/-- `1 / (1 - (2^n - 1))`, the `z` of the `b == -1` check at `b = 1`. -/
def invOneMinusMax : ℕ → F
  | 8 => (430870922674001480752882002859395178908432370086929809915318980050704891646 : ℕ)
  | 16 => (9141198754224816485452770757804290321031528447446919827153479878878037396108 : ℕ)
  | 32 => (12387033891700413737683124372579164722447538227871327576669075373896533794773 : ℕ)
  | 64 => (20763418735206616112454938172785494801206593451268382521205564247474549593416 : ℕ)
  | _ => 0

def sdivWitness (n : ℕ) : ℕ → F
  | 1 | 10 | 11 | 12 | 19 => 1
  | 3 => invNegHalf n
  | 5 => invOneMinusMax n
  | 16 => ((2 ^ (n - 1) : ℕ) : F)
  | _ => 0

def smodWitness (n : ℕ) : ℕ → F
  | 1 | 10 | 11 | 12 | 17 => 1
  | 3 => invNegHalf n
  | 5 => invOneMinusMax n
  | _ => 0

theorem signed_satisfiable {n : ℕ} (hn : n ∈ signedWidths) :
    SatisfiableFunction (shippedSignedDiv n) ∧ SatisfiableFunction (shippedSignedMod n) := by
  refine ⟨⟨sdivWitness n, ?_⟩, ⟨smodWitness n, ?_⟩⟩ <;>
  simp only [signedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn <;>
  rcases hn with rfl | rfl | rfl | rfl <;> decide +kernel

end AcirLean
