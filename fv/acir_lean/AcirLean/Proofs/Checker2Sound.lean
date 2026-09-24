/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Checker2
import AcirLean.Spec.Claims
import AcirLean.Templates.TestPrograms

/-! Soundness of `checkProg2`, and the test programs it accepts. -/

namespace AcirLean

/-! ## Polynomials -/

@[simp] theorem Poly.eval_nil (σ : ℕ → F) : Poly.eval σ [] = 0 := rfl
@[simp] theorem Poly.eval_cons (σ : ℕ → F) (t : Term) (P : Poly) :
    Poly.eval σ (t :: P) = t.eval σ + P.eval σ := by simp [Poly.eval]
@[simp] theorem Poly.eval_append (σ : ℕ → F) (P Q : Poly) :
    Poly.eval σ (P ++ Q) = P.eval σ + Q.eval σ := by simp [Poly.eval]
@[simp] theorem eval_pconst (σ : ℕ → F) (c : ℤ) : (pconst c).eval σ = c := by
  simp [pconst, Term.eval]
@[simp] theorem eval_pvar (σ : ℕ → F) (w : ℕ) : (pvar w).eval σ = σ w := by
  simp [pvar, Term.eval]
@[simp] theorem eval_pscale (σ : ℕ → F) (c : ℤ) (P : Poly) :
    (pscale c P).eval σ = c * P.eval σ := by
  induction P with
  | nil => simp [pscale]
  | cons t P ih =>
    simp only [pscale, List.map_cons, Poly.eval_cons] at ih ⊢
    rw [ih]; simp [Term.eval]; ring
@[simp] theorem eval_psub (σ : ℕ → F) (P Q : Poly) :
    (psub P Q).eval σ = P.eval σ - Q.eval σ := by
  simp [psub]; ring
theorem eval_mulTerm (σ : ℕ → F) (t : Term) (Q : Poly) :
    Poly.eval σ (Q.map fun u => (⟨t.coef * u.coef, t.ws ++ u.ws⟩ : Term)) = t.eval σ * Q.eval σ := by
  induction Q with
  | nil => simp
  | cons u Q ih =>
    simp only [List.map_cons, Poly.eval_cons, ih]
    simp [Term.eval, List.map_append, List.prod_append]; ring

@[simp] theorem eval_pmul (σ : ℕ → F) (P Q : Poly) :
    (pmul P Q).eval σ = P.eval σ * Q.eval σ := by
  induction P with
  | nil => simp [pmul]
  | cons t P ih =>
    simp only [pmul, List.flatMap_cons, Poly.eval_append, Poly.eval_cons] at ih ⊢
    rw [ih, add_mul, eval_mulTerm]

theorem eval_addTerm (σ : ℕ → F) (t : Term) :
    ∀ P : Poly, (addTerm t P).eval σ = t.eval σ + P.eval σ
  | [] => by simp [addTerm]
  | u :: us => by
    unfold addTerm
    split
    · next h => simp [Term.eval, h]; ring
    · simp [eval_addTerm σ t us]; ring

theorem eval_collect (σ : ℕ → F) : ∀ P : Poly, (collect P).eval σ = P.eval σ
  | [] => rfl
  | t :: ts => by
    simp only [collect, eval_addTerm, eval_collect σ ts, Poly.eval_cons]
    congr 1
    simp only [Term.eval]
    congr 1
    exact ((isort_perm _ _).map σ).prod_eq

theorem key_sat (σ : ℕ → F) (P : Poly) : (key P).sat σ ↔ P.eval σ = 0 := by
  unfold key
  rw [Cstr.canon_sat]
  simp only [Cstr.sat]
  rw [← eval_collect σ P]; rfl

theorem comb_mem {f : Poly → Poly → Poly} {as bs : List Poly} {X : Poly}
    (h : X ∈ comb f as bs) : ∃ a ∈ as, ∃ b ∈ bs, X = f a b := by
  obtain ⟨a, ha, hX⟩ := List.mem_flatMap.1 (List.mem_of_mem_take h)
  obtain ⟨b, hb, rfl⟩ := List.mem_map.1 hX
  exact ⟨a, ha, b, hb, rfl⟩


section
variable {cc : List Cstr} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.sat σ)
include hcc

theorem holdsZ_sound {P : Poly} (h : holdsZ cc P = true) : P.eval σ = 0 := by
  rw [← key_sat]
  unfold holdsZ at h
  split at h
  · next he => rw [he]; simp [Cstr.sat]
  · exact hcc _ (of_decide_eq_true h)

theorem matV_sound {P : Poly} {w : ℕ} (h : matV cc P = some w) : σ w = P.eval σ := by
  have := holdsZ_sound hcc (Option.filter_eq_some_iff.1 h).2
  simp at this; linear_combination this

theorem wbound_sound {w L M : ℕ} (h : wbound cc w = some (L, M)) :
    L ≤ (σ w).val ∧ (σ w).val ≤ M := by
  unfold wbound at h
  split at h
  · next c b hf =>
    simp only [Option.some.injEq, Prod.mk.injEq] at h
    obtain ⟨rfl, rfl⟩ := h
    have hp := List.find?_some hf
    simp only [Bool.and_eq_true, decide_eq_true_eq] at hp
    obtain ⟨⟨hcp, hb⟩, hz⟩ := hp
    have e := holdsZ_sound hcc hz
    simp only [eval_pscale, eval_psub, eval_pvar, eval_pconst] at e
    have : σ w = (c : F) := by
      rcases mul_eq_zero.1 e with h0 | h0
      · exact absurd h0 hb
      · push_cast at h0; linear_combination h0
    rw [this, val_natCast_of_lt hcp]; omega
  · obtain ⟨M', hM, he⟩ := Option.map_eq_some_iff.1 h
    simp only [Prod.mk.injEq] at he
    obtain ⟨rfl, rfl⟩ := he
    obtain ⟨c, hc, hcM⟩ := List.mem_filterMap.1 (List.min?_mem hM)
    split at hcM
    · next v k =>
      split at hcM
      · next hv =>
        simp only [Option.some.injEq] at hcM
        subst hv; subst hcM
        have := hcc _ hc
        simp only [Cstr.sat, Range] at this
        omega
      · simp at hcM
    · simp at hcM

theorem pbound_sound {P : Poly} {L M : ℕ} (h : pbound cc P = some (L, M)) :
    L ≤ (P.eval σ).val ∧ (P.eval σ).val ≤ M := by
  unfold pbound at h
  split at h
  · next hz =>
    simp only [Option.some.injEq, Prod.mk.injEq] at h
    obtain ⟨rfl, rfl⟩ := h
    rw [holdsZ_sound hcc hz]; simp
  · obtain ⟨w, hw, hb⟩ := Option.bind_eq_some_iff.1 h
    rw [← matV_sound hcc hw]
    exact wbound_sound hcc hb

theorem forms_sound {alts : List Poly} {x : F} (h : ∀ P ∈ alts, P.eval σ = x) :
    ∀ X ∈ forms cc alts, X.eval σ = x := by
  intro X hX
  unfold forms at hX
  rcases List.mem_append.1 (List.mem_eraseDups.1 hX) with hX | hX
  · obtain ⟨P, hP, hm⟩ := List.mem_filterMap.1 hX
    obtain ⟨w, hw, rfl⟩ := Option.map_eq_some_iff.1 hm
    rw [eval_pvar, matV_sound hcc hw, h P hP]
  · exact h X hX

theorem solve2_sound {E : ℕ → ℕ → Poly} {q r : ℕ} (h : (q, r) ∈ solve2 cc E) :
    (E q r).eval σ = 0 :=
  holdsZ_sound hcc (List.mem_filter.1 h).2

theorem eqVia_sound {P Q : Poly} (h : eqVia cc P Q = true) : P.eval σ = Q.eval σ := by
  unfold eqVia at h
  rcases Bool.or_eq_true_iff.1 h with h | h
  · have := holdsZ_sound hcc h; simp at this; linear_combination this
  · obtain ⟨w, _, hw⟩ := List.any_eq_true.1 h
    simp only [Bool.and_eq_true] at hw
    have h1 := holdsZ_sound hcc hw.1
    have h2 := holdsZ_sound hcc hw.2
    simp at h1 h2; linear_combination h1 + h2

theorem zeroFlags_sound {t : Poly} {y : ℕ} (h : y ∈ zeroFlags cc t) :
    (t.eval σ = 0 → σ y = 1) ∧ (t.eval σ ≠ 0 → σ y = 0) := by
  obtain ⟨⟨z, y'⟩, hm, hy⟩ := List.mem_filterMap.1 h
  by_cases hty : holdsZ cc (pmul t (pvar y')) = true
  · simp only [hty, if_true, Option.some.injEq] at hy
    subst hy
    have h2 := holdsZ_sound hcc hty
    simp only [eval_pmul, eval_pvar] at h2
    rcases List.mem_append.1 hm with hm | hm
    · have h1 := solve2_sound hcc hm
      simp only [eval_psub, eval_pconst, eval_pmul, eval_pvar] at h1
      exact isZero_flag (z := σ z) (by push_cast at h1; linear_combination h1) h2
    · have h1 := solve2_sound hcc hm
      simp only [eval_psub, Poly.eval_append, eval_pconst, eval_pmul, eval_pvar] at h1
      exact isZero_flag (z := -σ z) (by push_cast at h1; linear_combination h1) h2
  · simp [hty] at hy

end

/-! ## Integer values -/

theorem sub_wrap {a b : F} (h : a.val < b.val) : p - b.val ≤ (a - b).val := by
  have hne : b - a ≠ 0 := by
    intro h0
    have : b = a := sub_eq_zero.1 h0
    rw [this] at h; omega
  have e1 : (b - a).val = b.val - a.val := ZMod.val_sub h.le
  have e2 : a - b = -(b - a) := by ring
  haveI : NeZero (b - a) := ⟨hne⟩
  have hb := ZMod.val_lt b
  rw [e2, ZMod.val_neg_of_ne_zero, e1]; omega

theorem sub_noWrap {x y : F} {M Mb : ℕ} (h1 : (x - y).val ≤ M) (h2 : y.val ≤ Mb)
    (h3 : M + Mb < p) : y.val ≤ x.val := by
  by_contra hc
  have := sub_wrap (not_le.1 hc)
  omega

theorem val_lin {X B q r : F} (h : X = B * q + r) (hno : B.val * q.val + r.val < p) :
    X.val = B.val * q.val + r.val := by
  rw [h, ZMod.val_add_of_lt (by rw [ZMod.val_mul_of_lt (by omega)]; omega),
    ZMod.val_mul_of_lt (by omega)]

theorem cast_val (x : F) : ((x.val : ℕ) : F) = x := ZMod.natCast_zmod_val x

/-! ## The rules -/

/-- `r` describes the value `v`. -/
def RepOK2 (σ : ℕ → F) (r : Rep2) (v : F × VTy) : Prop :=
  r.ty = v.2 ∧ (∀ P ∈ r.alts, P.eval σ = v.1) ∧ r.L ≤ v.1.val ∧ v.1.val ≤ r.M

theorem flag_ok (σ : ℕ → F) (alts : List Poly) (b : Bool) (L : ℕ) (hL : L = 0)
    (h : ∀ P ∈ alts, P.eval σ = (flag b).1) : RepOK2 σ ⟨alts, .uint 1, L, 1⟩ (flag b) := by
  refine ⟨rfl, h, by simp [hL], ?_⟩
  cases b <;> simp [flag, ZMod.val_one]

section
variable {cc : List Cstr} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.sat σ)
include hcc

theorem checked_sound {alts : List Poly} {x : F} (h : ∀ P ∈ alts, P.eval σ = x) {n L M : ℕ}
    (hc : checked cc alts n = some (L, M)) : L ≤ x.val ∧ x.val ≤ M ∧ M < 2 ^ n := by
  obtain ⟨P, hP, hb⟩ := List.exists_of_findSome?_eq_some hc
  obtain ⟨hb, hM⟩ := Option.filter_eq_some_iff.1 hb
  have := pbound_sound hcc hb
  rw [h P hP] at this
  exact ⟨this.1, this.2, of_decide_eq_true hM⟩

theorem euclid_sound {a b : Rep2} {x y : F}
    (hFa : ∀ X ∈ forms cc a.alts, X.eval σ = x) (hFb : ∀ X ∈ forms cc b.alts, X.eval σ = y)
    (hLb : b.L ≤ y.val) (hMb : y.val ≤ b.M) {q r : ℕ} (h : (q, r) ∈ euclid cc a b) :
    y.val ≠ 0 ∧ (σ q).val = x.val / y.val ∧ (σ r).val = x.val % y.val := by
  obtain ⟨Xa, hXa, h⟩ := List.mem_flatMap.1 h
  obtain ⟨Xb, hXb, h⟩ := List.mem_flatMap.1 h
  obtain ⟨hs, hc⟩ := List.mem_filter.1 h
  have e := solve2_sound hcc hs
  simp only [eval_psub, eval_pmul, eval_pvar, hFa Xa hXa, hFb Xb hXb] at e
  rcases hq : wbound cc q with _ | ⟨Lq, Mq⟩ <;> rcases hr : wbound cc r with _ | ⟨Lr, Mr⟩ <;>
    simp only [hq, hr, Bool.false_eq_true] at hc
  · simp only [Bool.and_eq_true, decide_eq_true_eq] at hc
    obtain ⟨hno, hrem⟩ := hc
    have bq := wbound_sound hcc hq
    have br := wbound_sound hcc hr
    -- `r < b`
    have hrb : (σ r).val < y.val := by
      unfold remLt at hrem
      rcases Bool.or_eq_true_iff.1 hrem with h1 | h1
      · split at h1
        · next L' M' hb' =>
          have hb2 := pbound_sound hcc hb'
          simp only [eval_psub, eval_pvar, eval_pconst, hFb Xb hXb] at hb2
          have hM := of_decide_eq_true h1
          by_contra hc
          have hw := sub_wrap (a := y) (b := σ r + 1) (by
            rw [ZMod.val_add_of_lt (by
              have := ZMod.val_lt (σ r); rw [ZMod.val_one]; omega), ZMod.val_one]; omega)
          rw [ZMod.val_add_of_lt (by rw [ZMod.val_one]; omega), ZMod.val_one] at hw
          have : y - (σ r + 1) = y - σ r - ((1 : ℤ) : F) := by push_cast; ring
          rw [this] at hw
          omega
        · simp at h1
      · simp only [Bool.and_eq_true, decide_eq_true_eq] at h1
        obtain ⟨hLM, h1⟩ := h1
        split at h1
        · next L' M' hb' =>
          simp only [Bool.and_eq_true, decide_eq_true_eq] at h1
          have hb2 := pbound_sound hcc hb'
          simp only [Poly.eval_append, eval_pvar, eval_pconst] at hb2
          set d := 2 ^ Nat.size (b.M - 1) - b.M
          have hd : ((d : ℤ) : F) = ((d : ℕ) : F) := by push_cast; rfl
          rw [hd, ZMod.val_add_of_lt (by rw [val_natCast_of_lt (by omega)]; omega),
            val_natCast_of_lt (by omega)] at hb2
          omega
        · simp at h1
    have hlin := val_lin (X := x) (B := y) (q := σ q) (r := σ r) (by linear_combination e)
      (by
        have h1 : y.val * (σ q).val ≤ b.M * Mq := Nat.mul_le_mul hMb bq.2
        omega)
    obtain ⟨h1, h2⟩ := (Nat.div_mod_unique (b := y.val) (a := x.val) (d := (σ q).val)
      (c := (σ r).val) (by omega)).2 ⟨by rw [hlin]; ring, hrb⟩
    exact ⟨by omega, h1.symm, h2.symm⟩

theorem geFlags_sound {a b : Rep2} {x y : F} {m : ℕ}
    (hFa : ∀ X ∈ forms cc a.alts, X.eval σ = x) (hFb : ∀ X ∈ forms cc b.alts, X.eval σ = y)
    (hx : x.val < 2 ^ m) (hy : y.val < 2 ^ m) (hm : 2 ^ (m + 1) < p) {q : ℕ}
    (h : q ∈ geFlags cc a b m) : σ q = if y.val ≤ x.val then 1 else 0 := by
  obtain ⟨Xa, hXa, h⟩ := List.mem_flatMap.1 h
  obtain ⟨Xb, hXb, h⟩ := List.mem_flatMap.1 h
  obtain ⟨⟨q', r⟩, hs, hc⟩ := List.mem_filterMap.1 h
  have e := solve2_sound hcc hs
  simp only [eval_psub, Poly.eval_append, eval_pscale, eval_pconst, eval_pvar,
    hFa Xa hXa, hFb Xb hXb] at e
  rcases hq : wbound cc q' with _ | ⟨Lq, Mq⟩ <;> rcases hr : wbound cc r with _ | ⟨Lr, Mr⟩ <;>
    simp only [hq, hr, reduceCtorEq] at hc
  · split_ifs at hc with hb
    simp only [Option.some.injEq] at hc
    subst hc
    have bq := wbound_sound hcc hq
    have br := wbound_sound hcc hr
    have hpow : 2 ^ (m + 1) = 2 * 2 ^ m := by ring
    -- both sides of `2^m + x - y = 2^m q + r` as integers
    have hl : ((2 ^ m + x.val - y.val : ℕ) : F) = ((2 ^ m : ℕ) : F) * σ q' + σ r := by
      rw [Nat.cast_sub (by omega)]; push_cast [cast_val]; push_cast at e
      linear_combination e
    have hv := val_lin hl (by rw [val_natCast_of_lt (by omega)]; nlinarith)
    rw [val_natCast_of_lt (by omega), val_natCast_of_lt (by omega)] at hv
    rcases val_bit (show (σ q').val < 2 by omega) with h0 | h1
    · rw [h0] at hv ⊢; rw [ZMod.val_zero] at hv
      rw [if_neg (by omega)]
    · rw [h1] at hv ⊢; rw [ZMod.val_one] at hv
      rw [if_pos (by omega)]

theorem eqFlags_sound {a b : Rep2} {x y : F}
    (hFa : ∀ X ∈ forms cc a.alts, X.eval σ = x) (hFb : ∀ X ∈ forms cc b.alts, X.eval σ = y)
    {w : ℕ} (h : w ∈ eqFlags cc a b) : σ w = (flag (decide (x = y))).1 := by
  obtain ⟨Xa, hXa, h⟩ := List.mem_flatMap.1 h
  obtain ⟨Xb, hXb, h⟩ := List.mem_flatMap.1 h
  have hz := zeroFlags_sound hcc h
  simp only [eval_psub, hFa Xa hXa, hFb Xb hXb, sub_eq_zero] at hz
  by_cases hxy : x = y
  · simp [flag, hxy, hz.1 hxy]
  · simp [flag, hxy, hz.2 (sub_ne_zero.2 hxy)]

end

section
variable {cc : List Cstr} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.sat σ)
include hcc

theorem truncOK_sound {k q r : ℕ} {x : F} (h : truncOK cc k q r = true)
    (e : x = ((2 ^ k : ℕ) : F) * σ q + σ r) : (σ r).val = x.val % 2 ^ k := by
  unfold truncOK at h
  rcases hq : wbound cc q with _ | ⟨Lq, Mq⟩ <;> rcases hr : wbound cc r with _ | ⟨Lr, Mr⟩ <;>
    simp only [hq, hr, Bool.false_eq_true] at h
  simp only [Bool.and_eq_true, Bool.or_eq_true, decide_eq_true_eq] at h
  obtain ⟨⟨⟨hk0, hkp⟩, hMr⟩, h⟩ := h
  have bq := wbound_sound hcc hq
  have br := wbound_sound hcc hr
  have hK : ((2 ^ k : ℕ) : F).val = 2 ^ k := val_natCast_of_lt hkp
  have hk2 : 1 < 2 ^ k := Nat.one_lt_two_pow (by omega)
  set K := 2 ^ k with hKdef
  -- `K q + r < p`
  have hno : K * (σ q).val + (σ r).val < p := by
    rcases h with h | ⟨ht, hy⟩
    · have := Nat.mul_le_mul_left K bq.2
      omega
    · set q0 := p / K with hq0def
      have hq0 : q0 * K ≤ p := Nat.div_mul_le_self p K
      have hq0p : q0 < p := Nat.div_lt_self (by norm_num [p]) hk2
      have hqle : (σ q).val ≤ q0 := by
        split at ht
        · next Lt Mt hb =>
          have hb2 := pbound_sound hcc hb
          simp only [eval_psub, eval_pconst, eval_pvar] at hb2
          have hc : ((q0 : ℤ) : F) = ((q0 : ℕ) : F) := by push_cast; rfl
          rw [hc] at hb2
          have := sub_noWrap hb2.2 bq.2 (of_decide_eq_true ht)
          rwa [val_natCast_of_lt hq0p] at this
        · simp at ht
      rcases Nat.lt_or_ge (σ q).val q0 with hlt | hge
      · have h1 := Nat.mul_le_mul_left K (show (σ q).val + 1 ≤ q0 by omega)
        rw [Nat.mul_add, Nat.mul_one] at h1
        rw [Nat.mul_comm] at hq0
        omega
      · have hqeq : (σ q).val = q0 := le_antisymm hqle hge
        obtain ⟨yw, hyw, hyb⟩ := List.any_eq_true.1 hy
        have hz := zeroFlags_sound hcc hyw
        simp only [eval_psub, eval_pvar, eval_pconst] at hz
        have h0 : σ q - ((q0 : ℕ) : ℤ) = 0 := by
          rw [← cast_val (σ q), hqeq]; push_cast; ring
        have hy1 := hz.1 h0
        split at hyb
        · next Lu Mu hb =>
          simp only [Bool.and_eq_true, decide_eq_true_eq] at hyb
          have hb2 := pbound_sound hcc hb
          simp only [eval_pmul, Poly.eval_append, eval_pvar, eval_pconst, hy1, mul_one] at hb2
          set d := 2 ^ Nat.size (p % K - 1) - p % K
          rw [show ((d : ℤ) : F) = ((d : ℕ) : F) by push_cast; rfl,
            ZMod.val_add_of_lt (by rw [val_natCast_of_lt (by omega)]; omega),
            val_natCast_of_lt (by omega)] at hb2
          have hmod := Nat.div_add_mod p K
          rw [← hq0def] at hmod
          rw [hqeq]
          omega
        · simp at hyb
  rw [val_lin (B := ((K : ℕ) : F)) e (by rw [hK]; exact hno), hK]
  rw [Nat.mul_add_mod]
  exact (Nat.mod_eq_of_lt (by omega)).symm

end

section
variable {cc : List Cstr} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.sat σ)
include hcc

theorem truncRep_sound {a : Rep2} {x : F} {tx : VTy} (ha : RepOK2 σ a (x, tx)) (k : ℕ) :
    RepOK2 σ (truncRep cc a k) (((x.val % 2 ^ k : ℕ) : F), tx) := by
  obtain ⟨hta, hPa, hLa, hMa⟩ := ha
  simp only at hta hPa hLa hMa
  have hFa := forms_sound hcc hPa
  have hmp : x.val % 2 ^ k < p := lt_of_le_of_lt (Nat.mod_le _ _) (ZMod.val_lt x)
  refine ⟨hta, ?_, by simp [truncRep], ?_⟩
  · intro P hP
    simp only [truncRep, List.mem_append] at hP
    rcases hP with hP | hP
    · split_ifs at hP with hM
      · rw [hPa P hP, Nat.mod_eq_of_lt (by omega), cast_val]
      · simp at hP
    · obtain ⟨Xa, hXa, hP⟩ := List.mem_flatMap.1 hP
      obtain ⟨⟨q, r⟩, hqr, rfl⟩ := List.mem_map.1 hP
      obtain ⟨hs, ht⟩ := List.mem_filter.1 hqr
      have e := solve2_sound hcc hs
      simp only [eval_psub, eval_pscale, eval_pvar, hFa Xa hXa] at e
      have := truncOK_sound hcc ht (x := x) (by push_cast at e ⊢; linear_combination e)
      rw [eval_pvar, ← this, cast_val]
  · simp only [truncRep, val_natCast_of_lt hmp]
    have := Nat.mod_lt x.val (show 0 < 2 ^ k by positivity)
    have := Nat.mod_le x.val (2 ^ k)
    omega

theorem binRep_sound {op : BOp} {a b r : Rep2} {x y : F} {tx ty : VTy}
    (h : binRep cc op a b = some r) (ha : RepOK2 σ a (x, tx)) (hb : RepOK2 σ b (y, ty)) :
    ∃ v, op.apply x y tx = some v ∧ RepOK2 σ r v := by
  obtain ⟨hta, hPa, hLa, hMa⟩ := ha
  obtain ⟨-, hPb, hLb, hMb⟩ := hb
  simp only at hta hPa hLa hMa hPb hLb hMb
  subst hta
  have hFa := forms_sound hcc hPa
  have hFb := forms_sound hcc hPb
  have hxp := ZMod.val_lt x
  have hyp := ZMod.val_lt y
  have hcomb : ∀ {f : Poly → Poly → Poly} {g : F → F → F},
      (∀ A B, (f A B).eval σ = g (A.eval σ) (B.eval σ)) →
      ∀ P ∈ comb f (forms cc a.alts) (forms cc b.alts), P.eval σ = g x y := by
    intro f g hfg P hP
    obtain ⟨A, hA, B, hB, rfl⟩ := comb_mem hP
    rw [hfg, hFa A hA, hFb B hB]
  have hadd := @hcomb (· ++ ·) (· + ·) (fun A B => Poly.eval_append σ A B)
  have hsub := @hcomb psub (· - ·) (fun A B => eval_psub σ A B)
  have hmul := @hcomb pmul (· * ·) (fun A B => eval_pmul σ A B)
  obtain ⟨aalts, aty, aL, aM⟩ := a
  simp only at hadd hsub hmul hFa hLa hMa ⊢
  cases op <;> cases aty <;> simp only [binRep, BOp.apply, Option.some.injEq, reduceCtorEq] at h ⊢
  case add.field =>
    subst h
    exact ⟨_, rfl, rfl, hadd, Nat.zero_le _, by dsimp only; have := ZMod.val_lt (x + y); omega⟩
  case sub.field =>
    subst h
    exact ⟨_, rfl, rfl, hsub, Nat.zero_le _, by dsimp only; have := ZMod.val_lt (x - y); omega⟩
  case mul.field =>
    subst h
    exact ⟨_, rfl, rfl, hmul, Nat.zero_le _, by dsimp only; have := ZMod.val_lt (x * y); omega⟩
  case add.uint n =>
    have hc : ((x.val + y.val : ℕ) : F) = x + y := by push_cast [cast_val]; rfl
    split_ifs at h with h1 h2
    · simp only [Option.some.injEq] at h; subst h
      rw [if_pos (by omega)]
      refine ⟨_, rfl, rfl, fun P hP => by rw [hadd P hP, hc], ?_, ?_⟩ <;>
        simp only [val_natCast_of_lt (show x.val + y.val < p by omega)] <;> omega
    · obtain ⟨⟨L, M⟩, hk, rfl⟩ := Option.map_eq_some_iff.1 h
      have hv : (x + y).val = x.val + y.val := ZMod.val_add_of_lt (by omega)
      have := checked_sound hcc hadd hk
      rw [hv] at this
      rw [if_pos (by omega)]
      refine ⟨_, rfl, rfl, fun P hP => by rw [hadd P hP, hc], ?_, ?_⟩ <;>
        simp only [val_natCast_of_lt (show x.val + y.val < p by omega)] <;> omega
  case mul.uint n =>
    have hc : ((x.val * y.val : ℕ) : F) = x * y := by push_cast [cast_val]; rfl
    have hle : x.val * y.val ≤ aM * b.M := Nat.mul_le_mul hMa hMb
    split_ifs at h with h1 h2
    · simp only [Option.some.injEq] at h; subst h
      rw [if_pos (by omega)]
      refine ⟨_, rfl, rfl, fun P hP => by rw [hmul P hP, hc], ?_, ?_⟩ <;>
        simp only [val_natCast_of_lt (show x.val * y.val < p by omega)]
      · exact Nat.mul_le_mul hLa hLb
      · exact hle
    · obtain ⟨⟨L, M⟩, hk, rfl⟩ := Option.map_eq_some_iff.1 h
      have hv : (x * y).val = x.val * y.val := ZMod.val_mul_of_lt (by omega)
      have := checked_sound hcc hmul hk
      rw [hv] at this
      rw [if_pos (by omega)]
      refine ⟨_, rfl, rfl, fun P hP => by rw [hmul P hP, hc], ?_, ?_⟩ <;>
        simp only [val_natCast_of_lt (show x.val * y.val < p by omega)] <;> omega
  case sub.uint n =>
    split_ifs at h with h1
    · simp only [Option.some.injEq] at h; subst h
      have hyx : y.val ≤ x.val := by omega
      have hc : ((x.val - y.val : ℕ) : F) = x - y := by push_cast [Nat.cast_sub hyx, cast_val]; rfl
      rw [if_pos hyx]
      refine ⟨_, rfl, rfl, fun P hP => by rw [hsub P hP, hc], ?_, ?_⟩ <;>
        simp only [val_natCast_of_lt (show x.val - y.val < p by omega)] <;> omega
    · obtain ⟨⟨L, M⟩, hk, h⟩ := Option.bind_eq_some_iff.1 h
      split_ifs at h with h2
      simp only [Option.some.injEq] at h; subst h
      have := checked_sound hcc hsub hk
      have hyx := sub_noWrap this.2.1 hMb h2
      have hc : ((x.val - y.val : ℕ) : F) = x - y := by push_cast [Nat.cast_sub hyx, cast_val]; rfl
      have hv : (x - y).val = x.val - y.val := ZMod.val_sub hyx
      rw [hv] at this
      rw [if_pos hyx]
      refine ⟨_, rfl, rfl, fun P hP => by rw [hsub P hP, hc], ?_, ?_⟩ <;>
        simp only [val_natCast_of_lt (show x.val - y.val < p by omega)] <;> omega
  case div.uint n =>
    split_ifs at h with he
    simp only [Option.some.injEq] at h; subst h
    cases hl : euclid cc ⟨aalts, .uint n, aL, aM⟩ b with
    | nil => simp [hl] at he
    | cons s rest =>
      have hs := euclid_sound hcc hFa hFb hLb hMb (show (s.1, s.2) ∈ _ by rw [hl]; simp)
      rw [if_neg hs.1]
      have hdp : x.val / y.val < p := lt_of_le_of_lt (Nat.div_le_self _ _) hxp
      refine ⟨_, rfl, rfl, ?_, Nat.zero_le _, ?_⟩
      · intro P hP
        obtain ⟨⟨q, r⟩, hqr, rfl⟩ := List.mem_map.1 hP
        have := euclid_sound hcc hFa hFb hLb hMb (show (q, r) ∈ _ by rw [hl]; exact hqr)
        rw [eval_pvar, ← this.2.1, cast_val]
      · simp only [val_natCast_of_lt hdp]
        exact le_trans (Nat.div_le_self _ _) hMa
  case mod.uint n =>
    split_ifs at h with he
    simp only [Option.some.injEq] at h; subst h
    cases hl : euclid cc ⟨aalts, .uint n, aL, aM⟩ b with
    | nil => simp [hl] at he
    | cons s rest =>
      have hs := euclid_sound hcc hFa hFb hLb hMb (show (s.1, s.2) ∈ _ by rw [hl]; simp)
      rw [if_neg hs.1]
      have hmp : x.val % y.val < p := lt_of_le_of_lt (Nat.mod_le _ _) hxp
      refine ⟨_, rfl, rfl, ?_, Nat.zero_le _, ?_⟩
      · intro P hP
        obtain ⟨⟨q, r⟩, hqr, rfl⟩ := List.mem_map.1 hP
        have := euclid_sound hcc hFa hFb hLb hMb (show (q, r) ∈ _ by rw [hl]; exact hqr)
        rw [eval_pvar, ← this.2.2, cast_val]
      · simp only [val_natCast_of_lt hmp]
        have := Nat.mod_lt x.val (Nat.pos_of_ne_zero hs.1)
        have := Nat.mod_le x.val y.val
        omega
  case lt.uint n =>
    split_ifs at h with h1
    simp only [Option.some.injEq] at h; subst h
    refine ⟨_, rfl, flag_ok σ _ _ 0 rfl ?_⟩
    intro P hP
    obtain ⟨q, hq, rfl⟩ := List.mem_map.1 hP
    have := geFlags_sound hcc hFa hFb (by omega) (by omega) h1.2.2 hq
    simp only [eval_psub, eval_pconst, eval_pvar, this, flag]
    by_cases hxy : x.val < y.val
    · simp [hxy, show ¬ y.val ≤ x.val by omega]
    · simp [hxy, show y.val ≤ x.val by omega]
  all_goals
    subst h
    refine ⟨_, rfl, flag_ok σ _ _ 0 rfl ?_⟩
    intro P hP
    obtain ⟨w, hw, rfl⟩ := List.mem_map.1 hP
    rw [eval_pvar]
    exact eqFlags_sound hcc hFa hFb hw

end

/-! ## Programs -/

def EnvOK (σ : ℕ → F) (reps : List (ℕ × Rep2)) (env : Env) : Prop :=
  List.Forall₂ (fun a b => a.1 = b.1 ∧ RepOK2 σ a.2 b.2) reps env

theorem lookup_ok {σ : ℕ → F} : ∀ {reps : List (ℕ × Rep2)} {env : Env}, EnvOK σ reps env →
    ∀ {id : ℕ} {r : Rep2}, reps.lookup id = some r → ∃ v, env.lookup id = some v ∧ RepOK2 σ r v
  | [], [], _, _, _, h => by simp at h
  | (i, a) :: _, (j, v) :: _, .cons ⟨hij, hok⟩ ht, id, r, h => by
    simp only at hij
    subst hij
    by_cases hid : id = i
    · subst hid
      simp only [List.lookup, BEq.rfl, Option.some.injEq] at h ⊢
      subst h
      exact ⟨v, rfl, hok⟩
    · have hne : (id == i) = false := by simpa using hid
      simp only [List.lookup, hne] at h ⊢
      exact lookup_ok ht h

theorem opRep_ok {σ : ℕ → F} {reps : List (ℕ × Rep2)} {env : Env} (hE : EnvOK σ reps env)
    {o : Opnd} {r : Rep2} (h : opRep reps o = some r) :
    ∃ v, o.value env = some v ∧ RepOK2 σ r v := by
  cases o with
  | var id => exact lookup_ok hE h
  | const c ty =>
    simp only [opRep, Option.some.injEq] at h
    subst h
    refine ⟨((c : F), ty), rfl, rfl, ?_, ?_, ?_⟩
    · intro P hP
      simp only [List.mem_singleton] at hP
      subst hP
      simp
    · simp [ZMod.val_natCast]
    · simp [ZMod.val_natCast]

section
variable {cc : List Cstr} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.sat σ)
include hcc

theorem step2_ok {reps : List (ℕ × Rep2)} {env : Env} (hE : EnvOK σ reps env) {i : Ins}
    {reps' : List (ℕ × Rep2)} (h : step2 cc reps i = some reps') :
    ∃ env', i.run env = some env' ∧ EnvOK σ reps' env' := by
  cases i with
  | bin d op u a b =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨ra, hra, rb, hrb, r, hr, h⟩ := h
    obtain ⟨⟨x, tx⟩, hx, hokx⟩ := opRep_ok hE hra
    obtain ⟨⟨y, ty⟩, hy, hoky⟩ := opRep_ok hE hrb
    obtain ⟨v, hv, hokv⟩ := binRep_sound hcc hr hokx hoky
    simp only [Option.some.injEq] at h
    subst h
    exact ⟨(d, v) :: env, by simp [Ins.run, hx, hy, hv], .cons ⟨rfl, hokv⟩ hE⟩
  | not d a =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨ra, hra, h⟩ := h
    obtain ⟨⟨x, tx⟩, hx, ⟨hty, hP, hL, hM⟩⟩ := opRep_ok hE hra
    simp only at hty hP hL hM
    split at h
    · next n hn =>
      split_ifs at h with h1
      simp only [Option.some.injEq] at h
      subst h
      rw [hn] at hty
      subst hty
      have hxp : x.val ≤ 2 ^ n - 1 := by omega
      have hv : ((2 ^ n - 1 - x.val : ℕ) : F).val = 2 ^ n - 1 - x.val := val_natCast_of_lt (by omega)
      refine ⟨(d, (((2 ^ n - 1 - x.val : ℕ) : F), .uint n)) :: env, by simp [Ins.run, hx],
        .cons ⟨rfl, rfl, ?_, ?_, ?_⟩ hE⟩
      · intro P hP'
        obtain ⟨Q, hQ, rfl⟩ := List.mem_map.1 hP'
        simp only [eval_psub, eval_pconst, hP Q hQ, Int.cast_natCast]
        rw [Nat.cast_sub hxp, cast_val]
      · simp only [hv]; omega
      · simp only [hv]; omega
    · simp at h
  | cast d a ty =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨ra, hra, h⟩ := h
    obtain ⟨⟨x, tx⟩, hx, ⟨hty, hP, hL, hM⟩⟩ := opRep_ok hE hra
    simp only at hty hP hL hM
    split_ifs at h with hc
    simp only [Option.some.injEq] at h
    subst h
    have hfit : ty.fits x = true := by
      cases ty <;> simp only [castOK, decide_eq_true_eq] at hc <;>
        simp only [VTy.fits, decide_eq_true_eq] <;> omega
    exact ⟨(d, (x, ty)) :: env, by simp [Ins.run, hx, hfit], .cons ⟨rfl, rfl, hP, hL, hM⟩ hE⟩
  | truncate d a k m =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨ra, hra, h⟩ := h
    obtain ⟨⟨x, tx⟩, hx, hok⟩ := opRep_ok hE hra
    simp only [Option.some.injEq] at h
    subst h
    exact ⟨(d, (((x.val % 2 ^ k : ℕ) : F), tx)) :: env, by simp [Ins.run, hx],
      .cons ⟨rfl, truncRep_sound hcc hok k⟩ hE⟩
  | constrain a b m =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨ra, hra, rb, hrb, h⟩ := h
    obtain ⟨⟨x, tx⟩, hx, ⟨_, hPa, _, _⟩⟩ := opRep_ok hE hra
    obtain ⟨⟨y, ty⟩, hy, ⟨_, hPb, _, _⟩⟩ := opRep_ok hE hrb
    split_ifs at h with he
    simp only [Option.some.injEq] at h
    subst h
    obtain ⟨Xa, hXa, he⟩ := List.any_eq_true.1 he
    obtain ⟨Xb, hXb, he⟩ := List.any_eq_true.1 he
    have hxy := eqVia_sound hcc he
    simp only at hPa hPb
    rw [forms_sound hcc hPa Xa hXa, forms_sound hcc hPb Xb hXb] at hxy
    exact ⟨env, by simp [Ins.run, hx, hy, hxy], hE⟩
  | rangeCheck a k m =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨ra, hra, h⟩ := h
    obtain ⟨⟨x, tx⟩, hx, ⟨_, hPa, _, hMa⟩⟩ := opRep_ok hE hra
    simp only at hPa hMa
    split_ifs at h with hr
    simp only [Option.some.injEq] at h
    subst h
    have hk : x.val < 2 ^ k := by
      unfold rangeHolds at hr
      rcases Bool.or_eq_true_iff.1 hr with hr | hr
      · have := of_decide_eq_true hr; omega
      · obtain ⟨⟨L, M⟩, hc⟩ := Option.isSome_iff_exists.1 hr
        have := checked_sound hcc hPa hc
        omega
    exact ⟨env, by simp [Ins.run, hx, hk], hE⟩

end

section
variable {cc : List Cstr} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.sat σ)
include hcc

theorem paramRep_ok {w : ℕ} {ty : VTy} {r : Rep2} (h : paramRep cc w ty = some r) :
    RepOK2 σ r (σ w, ty) ∧ ty.fits (σ w) = true := by
  have hw := ZMod.val_lt (σ w)
  cases ty with
  | field =>
    simp only [paramRep, Option.some.injEq] at h
    subst h
    exact ⟨⟨rfl, by simp, Nat.zero_le _, by dsimp only; omega⟩, rfl⟩
  | uint n =>
    simp only [paramRep, Option.bind_eq_some_iff] at h
    obtain ⟨⟨L, M⟩, hb, h⟩ := h
    split_ifs at h with hM
    simp only [Option.some.injEq] at h
    subst h
    have := wbound_sound hcc hb
    exact ⟨⟨rfl, by simp, this.1, this.2⟩, by simp [VTy.fits]; omega⟩
  | sint n =>
    simp only [paramRep, Option.bind_eq_some_iff] at h
    obtain ⟨⟨L, M⟩, hb, h⟩ := h
    split_ifs at h with hM
    simp only [Option.some.injEq] at h
    subst h
    have := wbound_sound hcc hb
    exact ⟨⟨rfl, by simp, this.1, this.2⟩, by simp [VTy.fits]; omega⟩

theorem initReps_ok : ∀ (ps : List (ℕ × VTy)) (ws : List ℕ) (reps0 : List (ℕ × Rep2)),
    initReps cc ps ws = some reps0 →
    EnvOK σ reps0 ((ps.zip ((ws.map fun i => (σ i).val).map fun x => (x : F))).map
        fun ((id, ty), x) => (id, (x, ty))) ∧
      ∀ e ∈ ps.zip (ws.map fun i => (σ i).val), e.1.2.fits (e.2 : F) = true
  | [], _, reps0, h => by
    simp only [initReps, Option.some.injEq] at h; subst h; simp [EnvOK]
  | _ :: _, [], reps0, h => by
    simp only [initReps, Option.some.injEq] at h; subst h; simp [EnvOK]
  | (id, ty) :: ps, w :: ws, reps0, h => by
    simp only [initReps, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨r, hr, rest, hrest, h⟩ := h
    simp only [Option.some.injEq] at h
    subst h
    obtain ⟨hok, hfit⟩ := paramRep_ok hcc hr
    obtain ⟨hE, hf⟩ := initReps_ok ps ws rest hrest
    refine ⟨.cons ⟨rfl, by simpa [cast_val] using hok⟩ hE, ?_⟩
    intro e he
    simp only [List.map_cons, List.zip_cons_cons, List.mem_cons] at he
    rcases he with rfl | he
    · simpa [cast_val] using hfit
    · exact hf e he

theorem fold_ok2 : ∀ (body : List Ins) (reps : List (ℕ × Rep2)) (env : Env), EnvOK σ reps env →
    ∀ reps', body.foldlM (step2 cc) reps = some reps' →
      ∃ env', body.foldlM Ins.run env = some env' ∧ EnvOK σ reps' env'
  | [], reps, env, hE, reps', h => by
    simp only [List.foldlM_nil, Option.pure_def, Option.some.injEq] at h
    subst h; exact ⟨env, rfl, hE⟩
  | i :: body, reps, env, hE, reps', h => by
    simp only [List.foldlM_cons, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨reps1, h1, h2⟩ := h
    obtain ⟨env1, he1, hE1⟩ := step2_ok hcc hE h1
    obtain ⟨env', he', hE'⟩ := fold_ok2 body reps1 env1 hE1 reps' h2
    exact ⟨env', by simp [List.foldlM_cons, he1, he'], hE'⟩

theorem rets_ok {reps : List (ℕ × Rep2)} {env : Env} (hE : EnvOK σ reps env) :
    ∀ (rs : List ℕ) (os : List Opnd), rs.length = os.length →
      (rs.zip os).all (fun (r, o) => retOK cc reps r o) = true →
      ∃ vs, os.mapM (fun o => (o.value env).map Prod.fst) = some vs ∧
        rs.map (fun i => (σ i).val) = vs.map ZMod.val
  | [], [], _, _ => ⟨[], rfl, rfl⟩
  | r :: rs, o :: os, hl, h => by
    simp only [List.zip_cons_cons, List.all_cons, Bool.and_eq_true] at h
    obtain ⟨h1, h2⟩ := h
    obtain ⟨vs, hvs, hm⟩ := rets_ok hE rs os (by simpa using hl) h2
    unfold retOK at h1
    split at h1
    · next ro hro =>
      obtain ⟨⟨x, tx⟩, hx, ⟨_, hP, _, _⟩⟩ := opRep_ok hE hro
      simp only at hP
      obtain ⟨X, hX, he⟩ := List.any_eq_true.1 h1
      have := eqVia_sound hcc he
      rw [eval_pvar, forms_sound hcc hP X hX] at this
      refine ⟨x :: vs, by simp [List.mapM_cons, hx, hvs], ?_⟩
      simp [hm, this]
    · simp at h1

end

/-- Acceptance by the checker implies the circuit implements the program. -/
theorem checkProg2_sound (P : Prog2) (C : AcirFn) (h : checkProg2 P C = true) :
    SoundFn C (ProgSpec2 P) := by
  intro σ hσ
  have hcc : ∀ d ∈ C.cs.map Cstr.canon, d.sat σ := by
    intro d hd
    obtain ⟨c, hc, rfl⟩ := List.mem_map.1 hd
    exact (Cstr.canon_sat σ c).2 (hσ c hc)
  unfold checkProg2 at h
  simp only [Bool.and_eq_true, decide_eq_true_eq] at h
  obtain ⟨⟨hin, hret⟩, h⟩ := h
  split at h
  · simp at h
  · next reps0 hinit =>
    split at h
    · simp at h
    · next reps hfold =>
      obtain ⟨hE0, hfit⟩ := initReps_ok hcc P.params C.inputs reps0 hinit
      obtain ⟨env, hrun, hE⟩ := fold_ok2 hcc P.body reps0 _ hE0 reps hfold
      obtain ⟨vs, hvs, hm⟩ := rets_ok hcc hE C.returns P.rets hret h
      refine ⟨by simp [hin], hfit, vs, ?_, hm⟩
      simp only [Prog2.eval, Option.bind_eq_bind]
      rw [hrun]
      simpa using hvs

/-- The test programs: the checker accepts every one outside
`uncoveredPrograms`, decided by evaluation in the kernel. -/
theorem testPrograms_claims :
    ∀ e ∈ testPrograms, e.name ∉ uncoveredPrograms → SoundFn e.fn (ProgSpec2 e.prog) := by
  have hc : testPrograms.all (fun e =>
      decide (e.name ∈ uncoveredPrograms) || checkProg2 e.prog e.fn) = true := by
    decide +kernel
  intro e he hn
  have := List.all_eq_true.1 hc e he
  simp only [Bool.or_eq_true, decide_eq_true_eq] at this
  exact checkProg2_sound _ _ (this.resolve_left hn)

end AcirLean
