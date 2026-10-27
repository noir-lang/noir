/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Spec.Semantics
import Mathlib.NumberTheory.LucasPrimality
import Mathlib.Tactic.ReduceModChar
import Mathlib.Tactic.NormNum.Prime

/-!
`p` is prime, by a Pratt certificate: Lucas' test at each node, with the
factorization of `P - 1` checked by `norm_num` and each prime factor proved in
turn (by `norm_num` trial division when it is small). Only the `eq` gadget's
soundness needs this: `t * y = 0` with `t ≠ 0` forces `y = 0` in a field.
-/

namespace AcirLean

theorem prime_dvd_of_dvd_prod {q : ℕ} (hq : q.Prime) :
    ∀ {l : List (ℕ × ℕ)}, (∀ fe ∈ l, fe.1.Prime) →
      q ∣ (l.map fun fe => fe.1 ^ fe.2).prod → q ∈ l.map Prod.fst
  | [], _, h => absurd (Nat.le_of_dvd one_pos (by simpa using h)) (by have := hq.two_le; omega)
  | fe :: l, hl, h => by
    simp only [List.map_cons, List.prod_cons, List.mem_cons] at h ⊢
    rcases (Nat.Prime.dvd_mul hq).1 h with h1 | h1
    · left
      exact (Nat.prime_dvd_prime_iff_eq hq (hl fe (by simp))).1 (hq.dvd_of_dvd_pow h1)
    · right
      exact prime_dvd_of_dvd_prod hq (fun fe' h' => hl fe' (by simp [h'])) h1

theorem natCast_ne_one {P c : ℕ} [NeZero P] (hP : 1 < P) (hc : c < P) (h1 : c ≠ 1) :
    ((c : ℕ) : ZMod P) ≠ 1 := by
  haveI : Fact (1 < P) := ⟨hP⟩
  intro h
  have := congrArg ZMod.val h
  rw [ZMod.val_natCast, Nat.mod_eq_of_lt hc, ZMod.val_one] at this
  exact h1 this

/-- A literal other than `1`, below `P`, is not `1` in `ZMod P`. -/
theorem ofNat_ne_one {P c : ℕ} [NeZero P] [c.AtLeastTwo] (hP : 1 < P) (hc : c < P) :
    (OfNat.ofNat c : ZMod P) ≠ 1 := by
  have h2 : 2 ≤ c := Nat.AtLeastTwo.prop
  rw [← Nat.cast_ofNat]
  exact natCast_ne_one (c := c) hP hc (by omega)

/-- Lucas' test with the conditions stated in `ZMod P`, where `reduce_mod_char`
evaluates them. -/
theorem lucas_zmod {P : ℕ} (g : ℕ) (fs : List (ℕ × ℕ)) (hP : 2 ≤ P)
    (hfac : P - 1 = (fs.map fun fe => fe.1 ^ fe.2).prod)
    (hprimes : ∀ fe ∈ fs, fe.1.Prime)
    (hfull : haveI : NeZero P := ⟨by omega⟩; (g : ZMod P) ^ (P - 1) = 1)
    (hpart : haveI : NeZero P := ⟨by omega⟩;
      ∀ q ∈ fs.map Prod.fst, (g : ZMod P) ^ ((P - 1) / q) ≠ 1) : P.Prime := by
  haveI : NeZero P := ⟨by omega⟩
  refine lucas_primality P (g : ZMod P) hfull ?_
  intro q hq hd
  rw [hfac] at hd
  exact hpart q (prime_dvd_of_dvd_prod hq hprimes hd)

theorem prime_q405928799 : Nat.Prime 405928799 := by
  exact lucas_zmod (P := 405928799) 22 [(2, 1), (11, 1), (3691, 1), (4999, 1)] (by norm_num) (by norm_num)
    (by simp only [List.mem_cons, List.not_mem_nil, or_false, forall_eq_or_imp, forall_eq]
        and_intros <;> first | assumption | norm_num)
    (by simp only [Nat.cast_ofNat]; reduce_mod_char)
    (by simp only [List.map_cons, List.map_nil, List.mem_cons, List.not_mem_nil, or_false,
          forall_eq_or_imp, forall_eq, Nat.cast_ofNat]
        and_intros <;> (reduce_mod_char; exact ofNat_ne_one (by norm_num) (by norm_num)))

theorem prime_q12048837557 : Nat.Prime 12048837557 := by
  exact lucas_zmod (P := 12048837557) 2 [(2, 2), (7, 2), (661, 1), (93001, 1)] (by norm_num) (by norm_num)
    (by simp only [List.mem_cons, List.not_mem_nil, or_false, forall_eq_or_imp, forall_eq]
        and_intros <;> first | assumption | norm_num)
    (by simp only [Nat.cast_ofNat]; reduce_mod_char)
    (by simp only [List.map_cons, List.map_nil, List.mem_cons, List.not_mem_nil, or_false,
          forall_eq_or_imp, forall_eq, Nat.cast_ofNat]
        and_intros <;> (reduce_mod_char; exact ofNat_ne_one (by norm_num) (by norm_num)))

theorem prime_q5156902474397 : Nat.Prime 5156902474397 := by
  have : Nat.Prime 12048837557 := prime_q12048837557
  exact lucas_zmod (P := 5156902474397) 2 [(2, 2), (107, 1), (12048837557, 1)] (by norm_num) (by norm_num)
    (by simp only [List.mem_cons, List.not_mem_nil, or_false, forall_eq_or_imp, forall_eq]
        and_intros <;> first | assumption | norm_num)
    (by simp only [Nat.cast_ofNat]; reduce_mod_char)
    (by simp only [List.map_cons, List.map_nil, List.mem_cons, List.not_mem_nil, or_false,
          forall_eq_or_imp, forall_eq, Nat.cast_ofNat]
        and_intros <;> (reduce_mod_char; exact ofNat_ne_one (by norm_num) (by norm_num)))

theorem prime_q1670836401704629 : Nat.Prime 1670836401704629 := by
  have : Nat.Prime 5156902474397 := prime_q5156902474397
  exact lucas_zmod (P := 1670836401704629) 2 [(2, 2), (3, 4), (5156902474397, 1)] (by norm_num) (by norm_num)
    (by simp only [List.mem_cons, List.not_mem_nil, or_false, forall_eq_or_imp, forall_eq]
        and_intros <;> first | assumption | norm_num)
    (by simp only [Nat.cast_ofNat]; reduce_mod_char)
    (by simp only [List.map_cons, List.map_nil, List.mem_cons, List.not_mem_nil, or_false,
          forall_eq_or_imp, forall_eq, Nat.cast_ofNat]
        and_intros <;> (reduce_mod_char; exact ofNat_ne_one (by norm_num) (by norm_num)))

theorem prime_q1593227 : Nat.Prime 1593227 := by
  exact lucas_zmod (P := 1593227) 2 [(2, 1), (19, 1), (41927, 1)] (by norm_num) (by norm_num)
    (by simp only [List.mem_cons, List.not_mem_nil, or_false, forall_eq_or_imp, forall_eq]
        and_intros <;> first | assumption | norm_num)
    (by simp only [Nat.cast_ofNat]; reduce_mod_char)
    (by simp only [List.map_cons, List.map_nil, List.mem_cons, List.not_mem_nil, or_false,
          forall_eq_or_imp, forall_eq, Nat.cast_ofNat]
        and_intros <;> (reduce_mod_char; exact ofNat_ne_one (by norm_num) (by norm_num)))

theorem prime_q639533339 : Nat.Prime 639533339 := by
  exact lucas_zmod (P := 639533339) 2 [(2, 1), (229, 1), (853, 1), (1637, 1)] (by norm_num) (by norm_num)
    (by simp only [List.mem_cons, List.not_mem_nil, or_false, forall_eq_or_imp, forall_eq]
        and_intros <;> first | assumption | norm_num)
    (by simp only [Nat.cast_ofNat]; reduce_mod_char)
    (by simp only [List.map_cons, List.map_nil, List.mem_cons, List.not_mem_nil, or_false,
          forall_eq_or_imp, forall_eq, Nat.cast_ofNat]
        and_intros <;> (reduce_mod_char; exact ofNat_ne_one (by norm_num) (by norm_num)))

theorem prime_q65865678001877903 : Nat.Prime 65865678001877903 := by
  have : Nat.Prime 639533339 := prime_q639533339
  exact lucas_zmod (P := 65865678001877903) 5 [(2, 1), (83, 1), (379, 1), (1637, 1), (639533339, 1)] (by norm_num) (by norm_num)
    (by simp only [List.mem_cons, List.not_mem_nil, or_false, forall_eq_or_imp, forall_eq]
        and_intros <;> first | assumption | norm_num)
    (by simp only [Nat.cast_ofNat]; reduce_mod_char)
    (by simp only [List.map_cons, List.map_nil, List.mem_cons, List.not_mem_nil, or_false,
          forall_eq_or_imp, forall_eq, Nat.cast_ofNat]
        and_intros <;> (reduce_mod_char; exact ofNat_ne_one (by norm_num) (by norm_num)))

theorem prime_q13818364434197438864469338081 : Nat.Prime 13818364434197438864469338081 := by
  have : Nat.Prime 1593227 := prime_q1593227
  have : Nat.Prime 65865678001877903 := prime_q65865678001877903
  exact lucas_zmod (P := 13818364434197438864469338081) 3 [(2, 5), (5, 1), (823, 1), (1593227, 1), (65865678001877903, 1)] (by norm_num) (by norm_num)
    (by simp only [List.mem_cons, List.not_mem_nil, or_false, forall_eq_or_imp, forall_eq]
        and_intros <;> first | assumption | norm_num)
    (by simp only [Nat.cast_ofNat]; reduce_mod_char)
    (by simp only [List.map_cons, List.map_nil, List.mem_cons, List.not_mem_nil, or_false,
          forall_eq_or_imp, forall_eq, Nat.cast_ofNat]
        and_intros <;> (reduce_mod_char; exact ofNat_ne_one (by norm_num) (by norm_num)))

theorem prime_p : Nat.Prime p := by
  have : Nat.Prime 405928799 := prime_q405928799
  have : Nat.Prime 1670836401704629 := prime_q1670836401704629
  have : Nat.Prime 13818364434197438864469338081 := prime_q13818364434197438864469338081
  unfold p; exact lucas_zmod (P := 21888242871839275222246405745257275088548364400416034343698204186575808495617) 5 [(2, 28), (3, 2), (13, 1), (29, 1), (983, 1), (11003, 1), (237073, 1), (405928799, 1), (1670836401704629, 1), (13818364434197438864469338081, 1)] (by norm_num) (by norm_num)
    (by simp only [List.mem_cons, List.not_mem_nil, or_false, forall_eq_or_imp, forall_eq]
        and_intros <;> first | assumption | norm_num)
    (by simp only [Nat.cast_ofNat]; reduce_mod_char)
    (by simp only [List.map_cons, List.map_nil, List.mem_cons, List.not_mem_nil, or_false,
          forall_eq_or_imp, forall_eq, Nat.cast_ofNat]
        and_intros <;> (reduce_mod_char; exact ofNat_ne_one (by norm_num) (by norm_num)))

instance : Fact (Nat.Prime p) := ⟨prime_p⟩

end AcirLean
