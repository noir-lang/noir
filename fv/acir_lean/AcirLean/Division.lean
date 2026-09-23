import AcirLean.Basic

/-!
Soundness of `AcirContext::euclidean_division_var`
(`compiler/noirc_evaluator/src/acir/acir_context/mod.rs`) with an active
predicate. Each theorem's hypotheses are exactly the constraints that path
emits, plus the operand-typing premise the caller guarantees; `q` and `r` are
the unconstrained Brillig outputs, so they are universally quantified.
-/

namespace AcirLean

/-- Natural-number Euclidean division from `a = b*q + r` without wraparound. -/
theorem divmod_of_eq {a b q r : ℕ} (hb : 0 < b) (hr : r < b) (h : a = b * q + r) :
    q = a / b ∧ r = a % b := by
  subst h
  constructor
  · rw [Nat.add_comm, Nat.add_mul_div_left _ _ hb, Nat.div_eq_of_lt hr, zero_add]
  · rw [Nat.add_comm, Nat.add_mul_mod_self_left, Nat.mod_eq_of_lt hr]

/-- `x = y*z + w` over `F` becomes the natural equation when it cannot wrap. -/
theorem nat_eq_of_field_eq {a b q r : F} (hlt : b.val * q.val + r.val < p)
    (h : a = b * q + r) : a.val = b.val * q.val + r.val := by
  apply eq_nat_of_eq_cast hlt
  rw [h]; push_cast; simp

/-- The constraint set emitted for a non-constant divisor, bit size `n ≤ 126`
(`max_q_bits + max_rhs_bits = 2n < 253`, so no overflow branch). -/
structure DivVarConstraints (n : ℕ) (a b q r inv : F) : Prop where
  inv_ok   : inv * b * 1 = 1               -- `inv_var(rhs, one)`
  range_q  : Range q n                     -- `range_constrain_var(q, n)`
  range_r  : Range r n                     -- `range_constrain_var(r, n)`
  bound_r  : Range (b - (r + 1)) n         -- `bound_constraint_with_offset(r, b, 1, n)`
  euclid   : a * 1 = (b * q + r) * 1       -- `a * pred == (b*q + r) * pred`

theorem div_var_sound {n : ℕ} (hn : n ≤ 126) {a b q r inv : F}
    (hb : b.val < 2 ^ n) (c : DivVarConstraints n a b q r inv) :
    q.val = a.val / b.val ∧ r.val = a.val % b.val := by
  have hb0 : 0 < b.val := by
    rcases Nat.eq_zero_or_pos b.val with h | h
    · have : b = 0 := (ZMod.val_eq_zero b).1 h
      have := c.inv_ok; simp_all
    · exact h
  have hrb : r.val < b.val := lt_of_bound (by omega) c.range_r c.bound_r
  have hq := c.range_q
  unfold Range at hq
  have hbq : b.val * q.val < 2 ^ n * 2 ^ n :=
    Nat.mul_lt_mul_of_lt_of_le hb (le_of_lt hq) (by positivity)
  have h2n : 2 ^ n * 2 ^ n ≤ 2 ^ 252 := by
    rw [← pow_add]; exact Nat.pow_le_pow_right (by norm_num) (by omega)
  have hlt : b.val * q.val + r.val < p := by
    have := p_gt; have : b.val * q.val + r.val < 2 ^ 252 + 2 ^ n := by omega
    have : 2 ^ n ≤ 2 ^ 252 := Nat.pow_le_pow_right (by norm_num) (by omega)
    omega
  have heq : a = b * q + r := by simpa using c.euclid
  exact divmod_of_eq hb0 hrb (nat_eq_of_field_eq hlt heq)

/-- The constraint set emitted for a constant divisor `c ∉ {0, 1}`, bit size
`n` (with `bits c ≤ n`), when no overflow branch fires. `max_q_bits =
n - bits c + 1`, `max_r_bits = bits (c - 1)`. The remainder bound takes the
constant-rhs optimization when `c` fits a `u128` and the general path
otherwise. -/
def BoundConst (r : F) (c : ℕ) : Prop :=
  if c < 2 ^ 128 then
    Range (r + ((2 ^ bits (c - 1) - c : ℕ) : F)) (bits (c - 1))
  else
    Range ((c : F) - (r + 1)) (bits (c - 1))

structure DivConstConstraints (n c : ℕ) (a q r : F) : Prop where
  range_q : Range q (n - bits c + 1)
  range_r : Range r (bits (c - 1))
  bound_r : BoundConst r c
  euclid  : a * 1 = ((c : F) * q + r) * 1

theorem rem_lt_const {c : ℕ} (hc : 2 ≤ c) (hc' : c < 2 ^ 252) {r : F}
    (hr : Range r (bits (c - 1))) (hb : BoundConst r c) : r.val < c := by
  have hN : bits (c - 1) ≤ 252 := Nat.size_le.2 (by omega)
  unfold BoundConst at hb
  split_ifs at hb with h128
  · exact lt_of_bound_const (by omega) rfl hN hr hb
  · have hcv : ((c : ℕ) : F).val = c := val_natCast_of_lt (by have := p_gt; omega)
    have := lt_of_bound hN hr hb
    rwa [hcv] at this

/-- No overflow branch: `max_q_bits + bits c < 253`. -/
theorem div_const_sound {n c : ℕ} (hc : 2 ≤ c)
    (hno : (n - bits c + 1) + bits c ≤ 252) {a q r : F}
    (k : DivConstConstraints n c a q r) :
    q.val = a.val / c ∧ r.val = a.val % c := by
  have hcpow : c < 2 ^ bits c := Nat.lt_size_self c
  have hc252 : c < 2 ^ 252 :=
    lt_of_lt_of_le hcpow (Nat.pow_le_pow_right (by norm_num) (by omega))
  have hrc := rem_lt_const hc hc252 k.range_r k.bound_r
  have hq := k.range_q; unfold Range at hq
  have hcv : ((c : ℕ) : F).val = c := val_natCast_of_lt (by have := p_gt; omega)
  -- c*q + r < c*(q+1) ≤ 2^bits c * 2^max_q ≤ 2^252
  have hcq : c * (q.val + 1) ≤ 2 ^ bits c * 2 ^ (n - bits c + 1) :=
    Nat.mul_le_mul (le_of_lt hcpow) (by omega)
  have hpow : 2 ^ bits c * 2 ^ (n - bits c + 1) ≤ 2 ^ 252 := by
    rw [← pow_add]; exact Nat.pow_le_pow_right (by norm_num) (by omega)
  have hlt : ((c : ℕ) : F).val * q.val + r.val < p := by
    rw [hcv]; have := p_gt; nlinarith
  have heq : a = (c : F) * q + r := by simpa using k.euclid
  have := nat_eq_of_field_eq hlt heq
  rw [hcv] at this
  exact divmod_of_eq (by omega) hrc this

end AcirLean
