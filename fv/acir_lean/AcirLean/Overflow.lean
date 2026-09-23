import AcirLean.Division

/-!
The two overflow branches of `euclidean_division_var`, where `q*b + r` could
exceed `p`:

* non-constant divisor at `bit_size = 128`: `q` and `b` are each divided by
  `2^64` (constant-divisor path) and the upper halves must multiply to zero;
* constant divisor with `max_q_bits + bits c ≥ 253` (e.g. truncating a
  `Field`): `q ≤ q0 = p / c`, and when `q = q0` (an `is_zero` gadget) the
  remainder is bounded by `p - q0*c`.
-/

namespace AcirLean

/-- `bound_constraint_with_offset` constant path when the lhs is only known to
be small enough not to wrap. -/
theorem lt_of_bound_const' {x : F} {c N : ℕ} (hc : 1 ≤ c) (hN : N = bits (c - 1))
    (hN128 : N ≤ 128) (hx : x.val < 2 ^ 252)
    (h : (x + ((2 ^ N - c : ℕ) : F)).val < 2 ^ N) : x.val < c := by
  have hcN : c - 1 < 2 ^ N := by rw [hN]; exact Nat.lt_size_self _
  have h128 : 2 ^ N ≤ 2 ^ 128 := Nat.pow_le_pow_right (by norm_num) hN128
  have h252 : (2:ℕ) ^ 128 < 2 ^ 252 := by norm_num
  have hp := p_gt
  have hR : ((2 ^ N - c : ℕ) : F).val = 2 ^ N - c := val_natCast_of_lt (by omega)
  rw [ZMod.val_add_of_lt (by omega), hR] at h
  omega

/-! ### u128 division by a non-constant divisor -/

structure DivVar128Constraints (a b q r inv qu qr bu br : F) : Prop where
  main    : DivVarConstraints 128 a b q r inv
  split_q : DivConstConstraints 128 (2 ^ 64) q qu qr
  split_b : DivConstConstraints 128 (2 ^ 64) b bu br
  uppers  : qu * bu = 0

theorem div_var128_sound {a b q r inv qu qr bu br : F} (hb : b.val < 2 ^ 128)
    (c : DivVar128Constraints a b q r inv qu qr bu br) :
    q.val = a.val / b.val ∧ r.val = a.val % b.val := by
  have hbits : bits (2 ^ 64) = 65 := Nat.size_pow
  have hsq := div_const_sound (n := 128) (by norm_num) (by rw [hbits]; norm_num) c.split_q
  have hsb := div_const_sound (n := 128) (by norm_num) (by rw [hbits]; norm_num) c.split_b
  have hqu := c.split_q.range_q; have hbu := c.split_b.range_q
  unfold Range at hqu hbu; rw [hbits] at hqu hbu; norm_num at hqu hbu
  -- the upper halves are < 2^64, so their field product is their natural product
  have hprod : qu.val * bu.val = 0 := by
    have hlt : qu.val * bu.val < p := by
      have := p_gt; have : qu.val * bu.val < 2 ^ 64 * 2 ^ 64 :=
        Nat.mul_lt_mul_of_lt_of_le hqu (le_of_lt hbu) (by positivity)
      omega
    have := congrArg ZMod.val c.uppers
    rwa [ZMod.val_mul, Nat.mod_eq_of_lt hlt, ZMod.val_zero] at this
  have hb0 : 0 < b.val := by
    rcases Nat.eq_zero_or_pos b.val with h | h
    · have : b = 0 := (ZMod.val_eq_zero b).1 h
      have := c.main.inv_ok; simp_all
    · exact h
  have hrb : r.val < b.val := lt_of_bound (by omega) c.main.range_r c.main.bound_r
  have hq := c.main.range_q; unfold Range at hq
  -- one of q, b is below 2^64
  have hsmall : b.val * q.val < 2 ^ 192 := by
    rcases Nat.mul_eq_zero.1 hprod with h | h
    · rw [hsq.1, Nat.div_eq_zero_iff] at h
      have : q.val < 2 ^ 64 := by omega
      have : b.val * q.val < 2 ^ 128 * 2 ^ 64 :=
        Nat.mul_lt_mul_of_lt_of_le hb (le_of_lt this) (by positivity)
      simpa using this
    · rw [hsb.1, Nat.div_eq_zero_iff] at h
      have : b.val < 2 ^ 64 := by omega
      have : b.val * q.val < 2 ^ 64 * 2 ^ 128 :=
        Nat.mul_lt_mul_of_lt_of_le this (le_of_lt hq) (by positivity)
      simpa using this
  have hlt : b.val * q.val + r.val < p := by have := p_gt; omega
  have heq : a = b * q + r := by simpa using c.main.euclid
  exact divmod_of_eq hb0 hrb (nat_eq_of_field_eq hlt heq)

/-! ### Constant divisor with the `q0 = p / c` overflow guard -/

/-- `is_zero` gadget from `GeneratedAcir::is_zero`: `y + t*z - 1 = 0`, `y*t = 0`. -/
structure IsZero (t y z : F) : Prop where
  y_def  : y + t * z - 1 = 0
  y_zero : y * t = 0

theorem IsZero.one_of_zero {t y z : F} (g : IsZero t y z) (ht : t = 0) : y = 1 := by
  have := g.y_def; rw [ht] at this; linear_combination this

/-- Constraints for a constant divisor `c` at bit size `n` when the overflow
branch fires and `q0 = p / c ≥ 2^128` (the `q ≤ q0` bound takes the general
path) and `p % c < 2^128` (the conditional remainder bound takes the
constant-rhs path). -/
structure DivConstOverflowConstraints (n c : ℕ) (a q r y z : F) : Prop where
  base    : DivConstConstraints n c a q r
  bound_q : Range (((p / c : ℕ) : F) - q) (n - bits c + 1)
  eq_q0   : IsZero (((p / c : ℕ) : F) - q) y z
  bound_r : Range ((r + ((2 ^ bits (p % c - 1) - p % c : ℕ) : F)) * y) (bits (p % c - 1))

theorem div_const_overflow_sound {n c : ℕ} (hc : 2 ≤ c) (hc252 : c < 2 ^ 252)
    (hmaxq : n - bits c + 1 ≤ 252) (hmodpos : 0 < p % c) (hmod128 : p % c ≤ 2 ^ 128)
    {a q r y z : F} (k : DivConstOverflowConstraints n c a q r y z) :
    q.val = a.val / c ∧ r.val = a.val % c := by
  have hrc := rem_lt_const hc hc252 k.base.range_r k.base.bound_r
  have hq := k.base.range_q; unfold Range at hq
  have hcv : ((c : ℕ) : F).val = c := val_natCast_of_lt (by have := p_gt; omega)
  have hq0v : ((p / c : ℕ) : F).val = p / c :=
    val_natCast_of_lt (Nat.div_lt_self (by norm_num [p]) (by omega))
  have hqle : q.val ≤ p / c := by
    have := le_of_bound0 hmaxq hq k.bound_q; rwa [hq0v] at this
  have hcq0 : c * (p / c) + p % c = p := Nat.div_add_mod p c
  have hlt : c * q.val + r.val < p := by
    rcases Nat.lt_or_ge q.val (p / c) with h | h
    · have : c * (q.val + 1) ≤ c * (p / c) := Nat.mul_le_mul_left _ h
      nlinarith
    · have hqq : q.val = p / c := le_antisymm hqle h
      have ht : ((p / c : ℕ) : F) - q = 0 := by
        rw [← hqq, ZMod.natCast_zmod_val, sub_self]
      have hy := k.eq_q0.one_of_zero ht
      have hb := k.bound_r; rw [hy, mul_one] at hb
      have hN : bits (p % c - 1) ≤ 128 := Nat.size_le.2 (by omega)
      have hr252 : r.val < 2 ^ 252 := by
        have := k.base.range_r; unfold Range at this
        exact lt_of_lt_of_le this (Nat.pow_le_pow_right (by norm_num)
          (Nat.size_le.2 (by omega)))
      have := lt_of_bound_const' (by omega) rfl hN hr252 hb
      rw [hqq]; omega
  have heq : a = (c : F) * q + r := by simpa using k.base.euclid
  have := nat_eq_of_field_eq (by rwa [hcv]) heq
  rw [hcv] at this
  exact divmod_of_eq (by omega) hrc this

/-- `truncate_var(x, k, 254)`: truncating a full field element to `k` bits. -/
theorem truncate_field_sound {k : ℕ} (hk1 : 2 ≤ k) (hk : k ≤ 125) {a q r y z : F}
    (c : DivConstOverflowConstraints 254 (2 ^ k) a q r y z) :
    r.val = a.val % 2 ^ k := by
  have hbits : bits (2 ^ k) = k + 1 := Nat.size_pow
  have hc2 : 2 ≤ 2 ^ k := by
    calc 2 = 2 ^ 1 := by norm_num
      _ ≤ 2 ^ k := Nat.pow_le_pow_right (by norm_num) (by omega)
  have hc252 : 2 ^ k < 2 ^ 252 := Nat.pow_lt_pow_right (by norm_num) (by omega)
  -- p is odd, so p % 2^k ≠ 0
  have hodd : p % 2 = 1 := by norm_num [p]
  have hmodpos : 0 < p % 2 ^ k := by
    rcases Nat.eq_zero_or_pos (p % 2 ^ k) with h | h
    · have hdvd : 2 ∣ p := by
        have : 2 ^ k ∣ p := Nat.dvd_of_mod_eq_zero h
        exact dvd_trans (dvd_pow_self 2 (by omega)) this
      omega
    · exact h
  have hmod : p % 2 ^ k ≤ 2 ^ 128 :=
    le_trans (le_of_lt (Nat.mod_lt _ (by positivity)))
      (Nat.pow_le_pow_right (by norm_num) (by omega))
  exact (div_const_overflow_sound hc2 hc252 (by rw [hbits]; omega) hmodpos hmod c).2

end AcirLean
