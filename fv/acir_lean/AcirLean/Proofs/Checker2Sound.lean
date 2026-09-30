/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Checker2
import AcirLean.Spec.Claims
import AcirLean.Templates.TestPrograms
import AcirLean.Proofs.TestProgramCerts

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
    Poly.eval σ (Q.map fun u => (⟨t.coef * u.coef, t.witnesses ++ u.witnesses⟩ : Term)) = t.eval σ * Q.eval σ := by
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

theorem key_sat (σ : ℕ → F) (P : Poly) : (key P).Holds σ ↔ P.eval σ = 0 := by
  unfold key
  rw [Opcode.canon_sat]
  simp only [Opcode.Holds]
  rw [← eval_collect σ P]; rfl

theorem comb_mem {f : Poly → Poly → Poly} {as bs : List Poly} {X : Poly}
    (h : X ∈ comb f as bs) : ∃ a ∈ as, ∃ b ∈ bs, X = f a b := by
  obtain ⟨a, ha, hX⟩ := List.mem_flatMap.1 (List.mem_of_mem_take h)
  obtain ⟨b, hb, rfl⟩ := List.mem_map.1 hX
  exact ⟨a, ha, b, hb, rfl⟩


section
variable {cc : List Opcode} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.Holds σ)
include hcc

theorem holdsZ_sound {P : Poly} (h : holdsZ cc P = true) : P.eval σ = 0 := by
  rw [← key_sat]
  unfold holdsZ at h
  split at h
  · next he => rw [he]; simp [Opcode.Holds]
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
        simp only [Opcode.Holds, Range] at this
        omega
      · simp at hcM
    · simp at hcM

theorem pbound_sound {P : Poly} {L M : ℕ} (h : pbound cc P = some (L, M)) :
    L ≤ (P.eval σ).val ∧ (P.eval σ).val ≤ M := by
  have viaMat : (matV cc P).bind (wbound cc) = some (L, M) →
      L ≤ (P.eval σ).val ∧ (P.eval σ).val ≤ M := fun h => by
    obtain ⟨w, hw, hb⟩ := Option.bind_eq_some_iff.1 h
    rw [← matV_sound hcc hw]
    exact wbound_sound hcc hb
  unfold pbound at h
  split at h
  · next hz =>
    simp only [Option.some.injEq, Prod.mk.injEq] at h
    obtain ⟨rfl, rfl⟩ := h
    rw [holdsZ_sound hcc hz]; simp
  · split at h
    · next c _ =>
      split_ifs at h with hc
      · simp only [Option.some.injEq, Prod.mk.injEq] at h
        obtain ⟨rfl, rfl⟩ := h
        have e := holdsZ_sound hcc hc.2
        simp only [eval_psub, eval_pconst] at e
        have : P.eval σ = ((c : ℕ) : F) := by push_cast at e ⊢; linear_combination e
        rw [this, val_natCast_of_lt hc.1]; omega
      · exact viaMat h
    · exact viaMat h

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
def RepOK2 (σ : ℕ → F) (r : Rep2) (v : F × ValueType) : Prop :=
  r.ty = v.2 ∧ (∀ P ∈ r.alts, P.eval σ = v.1) ∧ r.L ≤ v.1.val ∧ v.1.val ≤ r.M

theorem flag_ok (σ : ℕ → F) (alts : List Poly) (b : Bool) (L : ℕ) (hL : L = 0)
    (h : ∀ P ∈ alts, P.eval σ = (flag b).1) : RepOK2 σ ⟨alts, .uint 1, L, 1⟩ (flag b) := by
  refine ⟨rfl, h, by simp [hL], ?_⟩
  cases b <;> simp [flag, ZMod.val_one]

section
variable {cc : List Opcode} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.Holds σ)
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
  obtain ⟨T, hT, h⟩ := List.mem_flatMap.1 h
  have hTv : T.eval σ = x - y := by
    refine forms_sound hcc (fun P hP => ?_) T hT
    simp only [List.mem_singleton] at hP
    subst hP
    simp [eval_psub, hFa Xa hXa, hFb Xb hXb]
  have hz := zeroFlags_sound hcc h
  simp only [hTv, sub_eq_zero] at hz
  by_cases hxy : x = y
  · simp [flag, hxy, hz.1 hxy]
  · simp [flag, hxy, hz.2 (sub_ne_zero.2 hxy)]

end

section
variable {cc : List Opcode} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.Holds σ)
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
variable {cc : List Opcode} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.Holds σ)
include hcc

theorem truncRep_sound {a : Rep2} {x : F} {tx : ValueType} (ha : RepOK2 σ a (x, tx)) (k : ℕ) :
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

/-- Every binary instruction but unchecked arithmetic, on operands that fit
their type and without wraparound: what `checkedRep` proves, and what
`BinaryOp.apply` agrees with there (`apply_of_fitApply`). -/
def fitApply (op : BinaryOp) (x y : F) : ValueType → Option (F × ValueType)
  | .field =>
    match op with
    | .add => some (x + y, .field)
    | .sub => some (x - y, .field)
    | .mul => some (x * y, .field)
    | .eq => some (flag (x = y))
    | _ => none
  | .uint n =>
    if x.val < 2 ^ n ∧ y.val < 2 ^ n then
      match op with
      | .add =>
        if x.val + y.val < 2 ^ n ∧ x.val + y.val < p then some (((x.val + y.val : ℕ) : F), .uint n)
        else none
      | .sub => if y.val ≤ x.val then some (((x.val - y.val : ℕ) : F), .uint n) else none
      | .mul =>
        if x.val * y.val < 2 ^ n ∧ x.val * y.val < p then some (((x.val * y.val : ℕ) : F), .uint n)
        else none
      | .div => if y.val = 0 then none else some (((x.val / y.val : ℕ) : F), .uint n)
      | .mod => if y.val = 0 then none else some (((x.val % y.val : ℕ) : F), .uint n)
      | .lt => some (flag (x.val < y.val))
      | .eq => some (flag (x = y))
      | .xor => none
    else none
  | .sint n =>
    match op with
    | .eq => if x.val < 2 ^ n ∧ y.val < 2 ^ n then some (flag (x = y)) else none
    | _ => none

theorem checkedRep_sound {op : BinaryOp} {a b r : Rep2} {x y : F} {tx ty : ValueType}
    (h : checkedRep cc op a b = some r) (ha : RepOK2 σ a (x, tx)) (hb : RepOK2 σ b (y, ty))
    (hf : fitsBoth a b = true) : ∃ v, fitApply op x y tx = some v ∧ RepOK2 σ r v := by
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
  cases aty with
  | field =>
    cases op <;> simp only [checkedRep, fitApply, Option.some.injEq, reduceCtorEq] at h ⊢
    case add =>
      subst h
      exact ⟨_, rfl, rfl, hadd, Nat.zero_le _, by dsimp only; have := ZMod.val_lt (x + y); omega⟩
    case sub =>
      subst h
      exact ⟨_, rfl, rfl, hsub, Nat.zero_le _, by dsimp only; have := ZMod.val_lt (x - y); omega⟩
    case mul =>
      subst h
      exact ⟨_, rfl, rfl, hmul, Nat.zero_le _, by dsimp only; have := ZMod.val_lt (x * y); omega⟩
    all_goals
      subst h
      refine ⟨_, rfl, flag_ok σ _ _ 0 rfl ?_⟩
      intro P hP
      obtain ⟨w, hw, rfl⟩ := List.mem_map.1 hP
      rw [eval_pvar]
      exact eqFlags_sound hcc hFa hFb hw

  | uint n =>
    simp only [fitsBoth, decide_eq_true_eq] at hf
    have hxn : x.val < 2 ^ n := by omega
    have hyn : y.val < 2 ^ n := by omega
    cases op <;> simp only [checkedRep, fitApply, hxn, hyn, and_self, if_true,
      Option.some.injEq, reduceCtorEq] at h ⊢
    case add =>
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
    case mul =>
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
    case sub =>
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
    case div =>
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
          exact Nat.div_le_div hMa (max_le hLb (Nat.one_le_iff_ne_zero.2 hs.1)) (by omega)
    case mod =>
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
    case lt =>
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

  | sint n =>
    simp only [fitsBoth, decide_eq_true_eq] at hf
    have hxn : x.val < 2 ^ n := by omega
    have hyn : y.val < 2 ^ n := by omega
    cases op <;> simp only [checkedRep, fitApply, hxn, hyn, and_self, if_true,
      Option.some.injEq, reduceCtorEq] at h ⊢
    all_goals
      subst h
      refine ⟨_, rfl, flag_ok σ _ _ 0 rfl ?_⟩
      intro P hP
      obtain ⟨w, hw, rfl⟩ := List.mem_map.1 hP
      rw [eval_pvar]
      exact eqFlags_sound hcc hFa hFb hw


theorem uncheckedRep_sound {op : BinaryOp} {a b r : Rep2} {x y : F} {ty : ValueType}
    (h : uncheckedRep cc op a b = some r) (ha : RepOK2 σ a (x, a.ty)) (hb : RepOK2 σ b (y, ty)) :
    ∃ z, fieldArith op x y = some z ∧ RepOK2 σ r (z, a.ty) := by
  obtain ⟨-, hPa, hLa, hMa⟩ := ha
  obtain ⟨-, hPb, hLb, hMb⟩ := hb
  simp only at hPa hLa hMa hPb hLb hMb
  have hFa := forms_sound hcc hPa
  have hFb := forms_sound hcc hPb
  have hcomb : ∀ {f : Poly → Poly → Poly} {g : F → F → F},
      (∀ A B, (f A B).eval σ = g (A.eval σ) (B.eval σ)) →
      ∀ P ∈ comb f (forms cc a.alts) (forms cc b.alts), P.eval σ = g x y := by
    intro f g hfg P hP
    obtain ⟨A, hA, B, hB, rfl⟩ := comb_mem hP
    rw [hfg, hFa A hA, hFb B hB]
  have hxp := ZMod.val_lt x
  have hyp := ZMod.val_lt y
  cases op <;> simp only [uncheckedRep, fieldArith, Option.some.injEq, reduceCtorEq] at h ⊢
  case add =>
    have hadd := @hcomb (· ++ ·) (· + ·) (fun A B => Poly.eval_append σ A B)
    have := ZMod.val_lt (x + y)
    split_ifs at h with h1 <;> subst h
    · have hv : (x + y).val = x.val + y.val := ZMod.val_add_of_lt (by omega)
      exact ⟨_, rfl, rfl, hadd, by dsimp only; omega, by dsimp only; omega⟩
    · exact ⟨_, rfl, rfl, hadd, Nat.zero_le _, by dsimp only; omega⟩
  case sub =>
    have hsub := @hcomb psub (· - ·) (fun A B => eval_psub σ A B)
    have := ZMod.val_lt (x - y)
    split_ifs at h with h1 <;> subst h
    · have hv : (x - y).val = x.val - y.val := ZMod.val_sub (by omega)
      exact ⟨_, rfl, rfl, hsub, by dsimp only; omega, by dsimp only; omega⟩
    · exact ⟨_, rfl, rfl, hsub, Nat.zero_le _, by dsimp only; omega⟩
  case mul =>
    have hmul := @hcomb pmul (· * ·) (fun A B => eval_pmul σ A B)
    have := ZMod.val_lt (x * y)
    have hle : x.val * y.val ≤ a.M * b.M := Nat.mul_le_mul hMa hMb
    split_ifs at h with h1 <;> subst h
    · have hv : (x * y).val = x.val * y.val := ZMod.val_mul_of_lt (by omega)
      refine ⟨_, rfl, rfl, hmul, ?_, ?_⟩ <;> dsimp only <;> rw [hv]
      · exact Nat.mul_le_mul hLa hLb
      · exact hle
    · exact ⟨_, rfl, rfl, hmul, Nat.zero_le _, by dsimp only; omega⟩

omit hcc in
theorem apply_of_fitApply {op : BinaryOp} {u : Bool} {x y : F} {t : ValueType}
    {v : F × ValueType} (h : fitApply op x y t = some v)
    (hu : (t ≠ .field → (u && isArith op) = false) ∨ (t = .uint 1 ∧ op ≠ .add)) :
    op.apply u x y t = some v := by
  have hxp := ZMod.val_lt x
  have hyp := ZMod.val_lt y
  have hinj : ∀ a b : F, (a.val = b.val) ↔ a = b := fun a b => ZMod.val_injective _ |>.eq_iff
  cases t with
  | field =>
    cases op <;> simp only [fitApply, reduceCtorEq] at h <;> simpa [BinaryOp.apply] using h
  | sint n =>
    cases op <;> simp only [fitApply, reduceCtorEq] at h
    split_ifs at h with hf
    rw [← h]
    simp [BinaryOp.apply, lowBits, Nat.mod_eq_of_lt hf.1, Nat.mod_eq_of_lt hf.2, hinj]
  | uint n =>
    simp only [fitApply] at h
    by_cases hf : x.val < 2 ^ n ∧ y.val < 2 ^ n
    swap
    · simp [hf] at h
    rw [if_pos hf] at h
    obtain ⟨hx, hy⟩ := hf
    have cadd : ((x.val + y.val : ℕ) : F) = x + y := by push_cast [cast_val]; rfl
    have cmul : ((x.val * y.val : ℕ) : F) = x * y := by push_cast [cast_val]; rfl
    have csub : y.val ≤ x.val → ((x.val - y.val : ℕ) : F) = x - y := fun hle => by
      push_cast [Nat.cast_sub hle, cast_val]; rfl
    have vadd : x.val + y.val < p → (x + y).val = x.val + y.val := ZMod.val_add_of_lt
    have vmul : x.val * y.val < p → (x * y).val = x.val * y.val := ZMod.val_mul_of_lt
    have vsub : y.val ≤ x.val → (x - y).val = x.val - y.val := ZMod.val_sub
    have mx : x.val % 2 ^ n = x.val := Nat.mod_eq_of_lt hx
    have my : y.val % 2 ^ n = y.val := Nat.mod_eq_of_lt hy
    by_cases hn1 : n = 1
    · subst hn1
      simp only [pow_one] at hx hy
      rcases val_bit hx with rfl | rfl <;> rcases val_bit hy with rfl | rfl <;>
        cases u <;> cases op <;>
        simp_all [BinaryOp.apply, u1Apply, flag, isArith, ZMod.val_one, ZMod.val_zero]
    have hu' : u = false ∨ isArith op = false := by
      rcases hu with hu | ⟨hu, _⟩
      · have hu := hu (by simp)
        cases u <;> simp_all
      · simp at hu; exact absurd hu hn1
    have happ : ∀ {w}, (match op, fieldArith op x y with
        | .div, _ =>
          if lowBits n y = 0 then none else some (((lowBits n x / lowBits n y : ℕ) : F), .uint n)
        | .mod, _ =>
          if lowBits n y = 0 then none else some (((lowBits n x % lowBits n y : ℕ) : F), .uint n)
        | .lt, _ => some (flag (lowBits n x < lowBits n y))
        | .eq, _ => some (flag (lowBits n x = lowBits n y))
        | _, some r =>
          if u then some (r, .uint n)
          else if r.val < 2 ^ n ∧ (op = .mul → n = 128 → x.val * y.val < 2 ^ 128) then
            some (r, .uint n)
          else none
        | _, none => none) = some w → op.apply u x y (.uint n) = some w := by
      intro w hw
      rcases n with _ | _ | k
      · exact hw
      · exact absurd rfl hn1
      · exact hw
    apply happ
    cases op <;> (try dsimp only at h)
    · obtain rfl : u = false := by simpa [isArith] using hu'
      split_ifs at h with h1
      simp only [Option.some.injEq] at h
      subst h
      simp [fieldArith, vadd h1.2, h1.1, cadd]
    · obtain rfl : u = false := by simpa [isArith] using hu'
      split_ifs at h with h1
      simp only [Option.some.injEq] at h
      subst h
      have := vsub h1
      simp [fieldArith, this, csub h1]
      omega
    · obtain rfl : u = false := by simpa [isArith] using hu'
      split_ifs at h with h1
      simp only [Option.some.injEq] at h
      subst h
      simp [fieldArith, vmul h1.2, h1.1, cmul]
      intro hn; subst hn; exact h1.1
    · split_ifs at h with h1
      simp only [Option.some.injEq] at h
      subst h
      simp [lowBits, mx, my, h1]
    · split_ifs at h with h1
      simp only [Option.some.injEq] at h
      subst h
      simp [lowBits, mx, my, h1]
    · simp only [Option.some.injEq] at h
      subst h
      simp [lowBits, mx, my]
    · simp only [Option.some.injEq] at h
      subst h
      simp [lowBits, mx, my, hinj]
    · simp at h

theorem binRep_sound {op : BinaryOp} {u : Bool} {a b r : Rep2} {x y : F} {tx ty : ValueType}
    (h : binRep cc op u a b = some r) (ha : RepOK2 σ a (x, tx)) (hb : RepOK2 σ b (y, ty)) :
    ∃ v, op.apply u x y tx = some v ∧ RepOK2 σ r v := by
  have hta : a.ty = tx := ha.1
  subst hta
  unfold binRep at h
  split at h
  · next hty hu =>
    split_ifs at h with hf hadd
    · subst hadd
      obtain ⟨⟨L, M⟩, hk, rfl⟩ := Option.map_eq_some_iff.1 h
      obtain ⟨-, hPa, hLa, hMa⟩ := ha
      obtain ⟨-, hPb, hLb, hMb⟩ := hb
      simp only at hPa hMa hPb hMb
      simp only [fitsBoth, hty, pow_one, decide_eq_true_eq, not_not] at hf
      have hx : x.val < 2 := by omega
      have hy : y.val < 2 := by omega
      have hsum : ∀ P ∈ comb (· ++ ·) (forms cc a.alts) (forms cc b.alts), P.eval σ = x + y := by
        intro P hP
        obtain ⟨A, hA, B, hB, rfl⟩ := comb_mem hP
        rw [Poly.eval_append, forms_sound hcc hPa A hA, forms_sound hcc hPb B hB]
      obtain ⟨hL, hM, hM2⟩ := checked_sound hcc hsum hk
      simp only [Bool.and_eq_true] at hu
      rw [hty, hu.1]
      rcases val_bit hx with rfl | rfl <;> rcases val_bit hy with rfl | rfl
      · refine ⟨flag false, by simp [BinaryOp.apply, u1Apply, flag], rfl, ?_, ?_, ?_⟩ <;>
          simp_all [flag]
      · refine ⟨flag true, by simp [BinaryOp.apply, u1Apply, flag, ZMod.val_one], rfl, ?_, ?_, ?_⟩ <;>
          simp_all [flag, ZMod.val_one]
      · refine ⟨flag true, by simp [BinaryOp.apply, u1Apply, flag, ZMod.val_one], rfl, ?_, ?_, ?_⟩ <;>
          simp_all [flag, ZMod.val_one]
      · exfalso
        have : ((1 : F) + 1).val = 2 := by
          rw [ZMod.val_add_of_lt (by simp [ZMod.val_one]; norm_num [p])]; simp [ZMod.val_one]
        omega
    · obtain ⟨v, hv, hok⟩ := checkedRep_sound hcc h ha hb (by simpa using hf)
      exact ⟨v, apply_of_fitApply hv (.inr ⟨hty, hadd⟩), hok⟩
  · next n hn1 hty hu =>
    obtain ⟨z, hz, hok⟩ := uncheckedRep_sound hcc h ha hb
    simp only [Bool.and_eq_true] at hu
    refine ⟨_, ?_, hok⟩
    rw [hty, hu.1]
    rcases n with _ | _ | k
    · cases op <;> simp_all [BinaryOp.apply, fieldArith]
    · exact absurd rfl hn1
    · cases op <;> simp_all [BinaryOp.apply, fieldArith]
  · next n hty hu =>
    obtain ⟨z, hz, hok⟩ := uncheckedRep_sound hcc h ha hb
    simp only [Bool.and_eq_true] at hu
    refine ⟨_, ?_, hok⟩
    rw [hty, hu.1]
    cases op <;> simp_all [BinaryOp.apply, fieldArith]
  · next h1 h2 h3 =>
    by_cases hxo : op = .xor
    · subst hxo
      simp only [if_true] at h
      split_ifs at h with hc
      simp only [Option.some.injEq] at h
      subst h
      obtain ⟨hty1, hf⟩ := hc
      obtain ⟨-, hPa, -, hMa⟩ := ha
      obtain ⟨-, hPb, -, hMb⟩ := hb
      simp only at hPa hMa hPb hMb
      simp only [fitsBoth, hty1, pow_one, decide_eq_true_eq] at hf
      have hx : x.val < 2 := by omega
      have hy : y.val < 2 := by omega
      have hv : ∀ P ∈ comb (fun A B => psub (A ++ B) (pscale 2 (pmul A B)))
          (forms cc a.alts) (forms cc b.alts), P.eval σ = x + y - 2 * (x * y) := by
        intro P hP
        obtain ⟨A, hA, B, hB, rfl⟩ := comb_mem hP
        simp [forms_sound hcc hPa A hA, forms_sound hcc hPb B hB]
      rw [hty1]
      have h2 : (2 : F) ≠ 0 := by
        intro h0
        have := congrArg ZMod.val h0
        rw [ZMod.val_zero, show (2 : F) = ((2 : ℕ) : F) by norm_num,
          ZMod.val_natCast_of_lt (by norm_num [p])] at this
        omega
      have hflag : ∀ bx : Bool, (flag bx).1.val ≤ 1 := fun bx => by
        cases bx <;> simp [flag, ZMod.val_one]
      rcases val_bit hx with rfl | rfl <;> rcases val_bit hy with rfl | rfl
      · refine ⟨flag false, by simp [BinaryOp.apply, u1Apply, flag], rfl,
          fun P hP => by rw [hv P hP]; simp [flag], Nat.zero_le _, hflag _⟩
      · refine ⟨flag true, by simp [BinaryOp.apply, u1Apply, flag, ZMod.val_one], rfl,
          fun P hP => by rw [hv P hP]; simp [flag], Nat.zero_le _, hflag _⟩
      · refine ⟨flag true, by simp [BinaryOp.apply, u1Apply, flag, ZMod.val_one], rfl,
          fun P hP => by rw [hv P hP]; simp [flag], Nat.zero_le _, hflag _⟩
      · refine ⟨flag false, by simp [BinaryOp.apply, u1Apply, flag, ZMod.val_one], rfl,
          fun P hP => by rw [hv P hP]; simp [flag]; ring, Nat.zero_le _, hflag _⟩
    simp only [hxo, if_false] at h
    split_ifs at h with hf
    · obtain ⟨v, hv, hok⟩ := checkedRep_sound hcc h ha hb hf
      refine ⟨v, apply_of_fitApply hv (.inl fun hne => ?_), hok⟩
      cases hu : u && isArith op
      · rfl
      · exfalso
        cases hty : a.ty with
        | field => exact hne hty
        | uint n => exact h2 n hty hu
        | sint n => exact h3 n hty hu
    · split at h
      · next _ _ n hty =>
        split_ifs at h with hn1
        unfold addRep at h
        obtain ⟨-, hPa, -, -⟩ := ha
        obtain ⟨-, hPb, -, -⟩ := hb
        have hsum : ∀ P ∈ comb (· ++ ·) (forms cc a.alts) (forms cc b.alts), P.eval σ = x + y := by
          intro P hP
          obtain ⟨A, hA, B, hB, rfl⟩ := comb_mem hP
          rw [Poly.eval_append, forms_sound hcc hPa A hA, forms_sound hcc hPb B hB]
        have hbd : ∃ L M, L ≤ (x + y).val ∧ (x + y).val ≤ M ∧ M < 2 ^ n ∧
            r = ⟨comb (· ++ ·) (forms cc a.alts) (forms cc b.alts), .uint n, L, M⟩ := by
          dsimp only at h
          split at h
          · next L M hk =>
            simp only [Option.some.injEq] at h
            obtain ⟨hL, hM, hM2⟩ := checked_sound hcc hsum hk
            exact ⟨L, M, hL, hM, hM2, h.symm⟩
          · obtain ⟨c, hc, rfl⟩ := Option.map_eq_some_iff.1 h
            obtain ⟨P, hP, hf⟩ := List.exists_of_findSome?_eq_some hc
            have hf' := List.find?_some hf
            simp only [Bool.and_eq_true, decide_eq_true_eq] at hf'
            obtain ⟨⟨hcn, hcp⟩, hz⟩ := hf'
            have h0 := holdsZ_sound hcc hz
            simp only [eval_psub, eval_pconst, hsum P hP, Int.cast_natCast] at h0
            have hv : (x + y).val = c := by
              rw [show x + y = (c : F) by linear_combination h0]
              exact val_natCast_of_lt hcp
            exact ⟨c, c, by omega, by omega, hcn, rfl⟩
        obtain ⟨L, M, hL, hM, hM2, rfl⟩ := hbd
        have hu : (u && isArith .add) = false := by
          cases hu : u && isArith .add
          · rfl
          · exact (h2 n hty hu).elim
        have hu' : u = false := by simpa [isArith] using hu
        have hlt : (x + y).val < 2 ^ n := by omega
        refine ⟨(x + y, .uint n), ?_, rfl, hsum, hL, hM⟩
        rw [hty, hu']
        rcases n with _ | _ | k
        · have h0 : (x + y).val = 0 := by simpa using hlt
          simp [BinaryOp.apply, fieldArith, h0]
        · exact absurd rfl hn1
        · simp [BinaryOp.apply, fieldArith, hlt]
      · simp at h

end

/-! ## Programs -/

/-- A value is where its representation says: a scalar, or an array whose
scalars are, position by position. -/
@[reducible] def ValOK (σ : ℕ → F) : RVal → Value → Prop
  | .scalar r, .scalar v => RepOK2 σ r v
  | .array rs, .array xs => List.Forall₂ (RepOK2 σ) rs xs
  | _, _ => False

def EnvOK (σ : ℕ → F) (reps : Reps) (env : Env) : Prop :=
  List.Forall₂ (fun a b => a.1 = b.1 ∧ ValOK σ a.2 b.2) reps env

theorem lookup_ok {σ : ℕ → F} : ∀ {reps : Reps} {env : Env}, EnvOK σ reps env →
    ∀ {id : ℕ} {r : RVal}, reps.lookup id = some r → ∃ v, env.lookup id = some v ∧ ValOK σ r v
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

theorem constRep_ok (σ : ℕ → F) (c : ℤ) (ty : ValueType) :
    RepOK2 σ (constRep c ty) (constVal ty c, ty) := by
  refine ⟨rfl, ?_, ?_, ?_⟩
  · intro P hP
    simp only [constRep, List.mem_singleton] at hP
    subst hP
    simp [cast_val]
  · simp [constRep]
  · simp [constRep]

theorem opRep_ok {σ : ℕ → F} {reps : Reps} {env : Env} (hE : EnvOK σ reps env)
    {o : Operand} {r : Rep2} (h : opRep reps o = some r) :
    ∃ v, o.value env = some v ∧ RepOK2 σ r v := by
  cases o with
  | var id =>
    simp only [opRep] at h
    split at h
    · next r' hl =>
      simp only [Option.some.injEq] at h
      subst h
      obtain ⟨v, hv, hok⟩ := lookup_ok hE hl
      cases v with
      | scalar v => exact ⟨v, by simp [Operand.value, hv], hok⟩
      | array xs => exact hok.elim
    · simp at h
  | const c ty =>
    simp only [opRep, Option.some.injEq] at h
    subst h
    exact ⟨(constVal ty c, ty), rfl, constRep_ok σ c ty⟩

theorem opArr_ok {σ : ℕ → F} {reps : Reps} {env : Env} (hE : EnvOK σ reps env)
    {o : Operand} {rs : List Rep2} (h : opArr reps o = some rs) :
    ∃ xs, o.array env = some xs ∧ List.Forall₂ (RepOK2 σ) rs xs := by
  cases o with
  | var id =>
    simp only [opArr] at h
    split at h
    · next rs' hl =>
      simp only [Option.some.injEq] at h
      subst h
      obtain ⟨v, hv, hok⟩ := lookup_ok hE hl
      cases v with
      | scalar v => exact hok.elim
      | array xs => exact ⟨xs, by simp [Operand.array, hv], hok⟩
    · simp at h
  | const c ty => simp [opArr] at h

/-- The scalars of a returned operand evaluate to its flat values. -/
def FlatOK (σ : ℕ → F) (r : Rep2) (x : F) : Prop := ∀ P ∈ r.alts, P.eval σ = x

theorem opFlat_ok {σ : ℕ → F} {reps : Reps} {env : Env} (hE : EnvOK σ reps env)
    {o : Operand} {rs : List Rep2} (h : opFlat reps o = some rs) :
    ∃ xs, o.flat env = some xs ∧ List.Forall₂ (FlatOK σ) rs xs := by
  cases o with
  | var id =>
    simp only [opFlat] at h
    split at h
    · next r hl =>
      simp only [Option.some.injEq] at h
      subst h
      obtain ⟨v, hv, hok⟩ := lookup_ok hE hl
      cases v with
      | scalar v => exact ⟨[v.1], by simp [Operand.flat, hv], .cons hok.2.1 .nil⟩
      | array xs => exact hok.elim
    · next rs' hl =>
      simp only [Option.some.injEq] at h
      subst h
      obtain ⟨v, hv, hok⟩ := lookup_ok hE hl
      cases v with
      | scalar v => exact hok.elim
      | array xs =>
        refine ⟨xs.map Prod.fst, by simp [Operand.flat, hv], ?_⟩
        rw [List.forall₂_map_right_iff]
        exact hok.imp fun _ _ h => h.2.1
    · simp at h
  | const c ty =>
    simp only [opFlat, Option.some.injEq] at h
    subst h
    exact ⟨[constVal ty c], rfl, .cons (constRep_ok σ c ty).2.1 .nil⟩

theorem forall₂_getElem? {α β : Type} {R : α → β → Prop} :
    ∀ {as : List α} {bs : List β}, List.Forall₂ R as bs →
      ∀ {j : ℕ} {a : α}, as[j]? = some a → ∃ b, bs[j]? = some b ∧ R a b
  | [], [], .nil, _, _, h => by simp at h
  | _ :: _, b :: _, .cons hab ht, 0, a, h => by
    simp only [List.getElem?_cons_zero, Option.some.injEq] at h
    subst h
    exact ⟨b, rfl, hab⟩
  | _ :: _, _ :: _, .cons _ ht, j + 1, a, h => by
    simpa using forall₂_getElem? ht (by simpa using h)

theorem forall₂_set {α β : Type} {R : α → β → Prop} {a : α} {b : β} (hab : R a b) :
    ∀ {as : List α} {bs : List β}, List.Forall₂ R as bs →
      ∀ (j : ℕ), List.Forall₂ R (as.set j a) (bs.set j b)
  | [], [], .nil, _ => by simp
  | _ :: _, _ :: _, .cons _ ht, 0 => by simpa using List.Forall₂.cons hab ht
  | _ :: _, _ :: _, .cons h1 ht, j + 1 => by simpa using List.Forall₂.cons h1 (forall₂_set hab ht j)

theorem opReps_ok {σ : ℕ → F} {reps : Reps} {env : Env} (hE : EnvOK σ reps env) :
    ∀ (es : List Operand) (rs : List Rep2), es.mapM (opRep reps) = some rs →
      ∃ xs, es.mapM (·.value env) = some xs ∧ List.Forall₂ (RepOK2 σ) rs xs
  | [], rs, h => by
    simp only [List.mapM_nil, Option.pure_def, Option.some.injEq] at h
    subst h
    exact ⟨[], rfl, .nil⟩
  | e :: es, rs, h => by
    simp only [List.mapM_cons, Option.pure_def, Option.bind_eq_bind, Option.bind_eq_some_iff,
      Option.some.injEq] at h
    obtain ⟨r, hr, rs', hrs', rfl⟩ := h
    obtain ⟨x, hx, hok⟩ := opRep_ok hE hr
    obtain ⟨xs, hxs, hall⟩ := opReps_ok hE es rs' hrs'
    exact ⟨x :: xs, by simp [List.mapM_cons, hx, hxs], .cons hok hall⟩

theorem constIdx_some {len j : ℕ} {i : Operand} (h : constIdx len i = some j) :
    i = .const (j : ℤ) (.uint 32) ∧ j < len ∧ j < p := by
  unfold constIdx at h
  split at h
  · next c =>
    split_ifs at h with hc
    simp only [Option.some.injEq] at h
    subst h
    obtain ⟨h0, hl, hp⟩ := hc
    refine ⟨by rw [Int.toNat_of_nonneg h0], by omega, by omega⟩
  · simp at h

section
variable {cc : List Opcode} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.Holds σ)
include hcc

theorem step2_ok {reps : Reps} {env : Env} (hE : EnvOK σ reps env) {i : Instruction}
    {reps' : Reps} (h : step2 cc reps i = some reps') :
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
    exact ⟨(d, .scalar v) :: env, by simp [Instruction.run, hx, hy, hv], .cons ⟨rfl, hokv⟩ hE⟩
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
      have hxn : x.val < 2 ^ n := by omega
      refine ⟨(d, .scalar (((2 ^ n - 1 - x.val : ℕ) : F), .uint n)) :: env,
        by
          have hmod : x.val % 2 ^ n = x.val := Nat.mod_eq_of_lt hxn
          have h1' : n = 1 → x.val ≤ 1 := fun h => by subst h; omega
          simp [Instruction.run, hx, hmod]; exact h1',
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
    simp only [Option.some.injEq] at h
    subst h
    exact ⟨(d, .scalar (x, ty)) :: env, by simp [Instruction.run, hx], .cons ⟨rfl, rfl, hP, hL, hM⟩ hE⟩
  | truncate d a k m =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨ra, hra, h⟩ := h
    obtain ⟨⟨x, tx⟩, hx, hok⟩ := opRep_ok hE hra
    split_ifs at h with hk
    simp only [Option.some.injEq] at h
    subst h
    have hg : k = 0 ∨ (tx = .uint 1 → x.val < 2) := by
      obtain ⟨hty, -, -, hM⟩ := hok
      simp only at hty hM
      exact .inr fun h1 => by have := hk.2 (hty.trans h1); omega
    exact ⟨(d, .scalar (((x.val % 2 ^ k : ℕ) : F), tx)) :: env,
      by simp only [Instruction.run, hx, Option.bind_eq_bind, Option.bind_some]; rw [if_pos hg],
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
    exact ⟨env, by simp [Instruction.run, hx, hy, hxy], hE⟩
  | constrainNe a b m =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨ra, hra, rb, hrb, h⟩ := h
    obtain ⟨⟨x, tx⟩, hx, ⟨_, hPa, _, _⟩⟩ := opRep_ok hE hra
    obtain ⟨⟨y, ty⟩, hy, ⟨_, hPb, _, _⟩⟩ := opRep_ok hE hrb
    split_ifs at h with he
    simp only [Option.some.injEq] at h
    subst h
    simp only at hPa hPb
    obtain ⟨Xa, hXa, he⟩ := List.any_eq_true.1 he
    obtain ⟨Xb, hXb, he⟩ := List.any_eq_true.1 he
    obtain ⟨D, hD, he⟩ := List.any_eq_true.1 he
    obtain ⟨z, -, he⟩ := List.any_eq_true.1 he
    have hDv : D.eval σ = x - y := by
      refine forms_sound hcc (fun P hP => ?_) D hD
      simp only [List.mem_singleton] at hP
      subst hP
      simp [forms_sound hcc hPa Xa hXa, forms_sound hcc hPb Xb hXb]
    have hne : x ≠ y := by
      intro hxy
      rcases Bool.or_eq_true_iff.1 he with he | he
      · have := holdsZ_sound hcc he
        simp [hDv, hxy] at this
      · have := holdsZ_sound hcc he
        simp [hDv, hxy] at this
    exact ⟨env, by simp [Instruction.run, hx, hy, hne], hE⟩
  | rangeCheck a k m =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨ra, hra, h⟩ := h
    obtain ⟨⟨x, tx⟩, hx, ⟨hty, hPa, _, hMa⟩⟩ := opRep_ok hE hra
    simp only at hty hPa hMa
    split_ifs at h with hg
    obtain ⟨hk0, hu1, hr⟩ := hg
    simp only [Option.some.injEq] at h
    subst h
    have hu1' : tx = .uint 1 → x.val ≤ 1 := fun h1 => by have := hu1 (hty.trans h1); omega
    have hk : x.val < 2 ^ k := by
      unfold rangeHolds at hr
      rcases Bool.or_eq_true_iff.1 hr with hr | hr
      · have := of_decide_eq_true hr; omega
      · obtain ⟨⟨L, M⟩, hc⟩ := Option.isSome_iff_exists.1 hr
        have := checked_sound hcc hPa hc
        omega
    exact ⟨env, by simp only [Instruction.run, hx, Option.bind_eq_bind, Option.bind_some]; rw [if_pos ⟨hk0, hk, fun h1 => by have := hu1' h1; omega⟩], hE⟩
  | arrayGet d a i ty =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨rs, hrs, j, hj, r, hr, h⟩ := h
    simp only [Option.some.injEq] at h
    subst h
    obtain ⟨xs, hxs, hall⟩ := opArr_ok hE hrs
    obtain ⟨rfl, hjl, hjp⟩ := constIdx_some hj
    obtain ⟨x, hx, hok⟩ := forall₂_getElem? hall hr
    have hlen := hall.length_eq
    have hi : (Operand.const (j : ℤ) (.uint 32)).value env = some ((j : F), .uint 32) := by
      simp [Operand.value, constVal]
    obtain ⟨_, hxj⟩ := List.getElem?_eq_some_iff.1 hx
    refine ⟨(d, .scalar x) :: env, ?_, .cons ⟨rfl, hok⟩ hE⟩
    simp [Instruction.run, hxs, hi, arrayIndex, val_natCast_of_lt hjp, ← hlen, hjl, hxj]
  | arraySet d m a i v =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨rs, hrs, j, hj, r, hr, h⟩ := h
    simp only [Option.some.injEq] at h
    subst h
    obtain ⟨xs, hxs, hall⟩ := opArr_ok hE hrs
    obtain ⟨rfl, hjl, hjp⟩ := constIdx_some hj
    obtain ⟨x, hx, hok⟩ := opRep_ok hE hr
    have hlen := hall.length_eq
    have hi : (Operand.const (j : ℤ) (.uint 32)).value env = some ((j : F), .uint 32) := by
      simp [Operand.value, constVal]
    refine ⟨(d, .array (xs.set j x)) :: env, ?_, .cons ⟨rfl, forall₂_set hok hall j⟩ hE⟩
    simp [Instruction.run, hxs, hi, arrayIndex, val_natCast_of_lt hjp, ← hlen, hjl, hx]
  | makeArray d es ty =>
    simp only [step2, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨rs, hrs, h⟩ := h
    simp only [Option.some.injEq] at h
    subst h
    obtain ⟨xs, hxs, hall⟩ := opReps_ok hE es rs hrs
    exact ⟨(d, .array xs) :: env, by simp [Instruction.run, hxs], .cons ⟨rfl, hall⟩ hE⟩

end

section
variable {cc : List Opcode} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.Holds σ)
include hcc

theorem paramRep_ok {w : ℕ} {ty : ValueType} {r : Rep2} (h : paramRep cc w ty = some r) :
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
    exact ⟨⟨rfl, by simp, this.1, this.2⟩, by simp [ValueType.fits]; omega⟩
  | sint n =>
    simp only [paramRep, Option.bind_eq_some_iff] at h
    obtain ⟨⟨L, M⟩, hb, h⟩ := h
    split_ifs at h with hM
    simp only [Option.some.injEq] at h
    subst h
    have := wbound_sound hcc hb
    exact ⟨⟨rfl, by simp, this.1, this.2⟩, by simp [ValueType.fits]; omega⟩

theorem paramReps_ok : ∀ (l : List (ValueType × ℕ)) (rs : List Rep2),
    l.mapM (fun (ty, w) => paramRep cc w ty) = some rs →
      List.Forall₂ (fun r (e : ValueType × ℕ) => RepOK2 σ r (σ e.2, e.1)) rs l ∧
        ∀ e ∈ l, e.1.fits (σ e.2) = true
  | [], rs, h => by
    simp only [List.mapM_nil, Option.pure_def, Option.some.injEq] at h
    subst h
    exact ⟨.nil, by simp⟩
  | (ty, w) :: l, rs, h => by
    simp only [List.mapM_cons, Option.pure_def, Option.bind_eq_bind, Option.bind_eq_some_iff,
      Option.some.injEq] at h
    obtain ⟨r, hr, rs', hrs', rfl⟩ := h
    obtain ⟨hok, hfit⟩ := paramRep_ok hcc hr
    obtain ⟨hall, hf⟩ := paramReps_ok l rs' hrs'
    refine ⟨.cons hok hall, ?_⟩
    intro e he
    simp only [List.mem_cons] at he
    rcases he with rfl | he
    · exact hfit
    · exact hf e he

omit hcc in
theorem take_zip_map (f : ℕ → F) : ∀ (ts : List ValueType) (ws : List ℕ),
    ((ws.map f).take ts.length).zip ts = (ts.zip ws).map fun (ty, w) => (f w, ty)
  | [], _ => by simp
  | _ :: _, [] => by simp
  | ty :: ts, w :: ws => by simp [take_zip_map f ts ws]

omit hcc in
theorem mem_zip_append {α β : Type} {e : α × β} :
    ∀ (a b : List α) (v : List β), e ∈ (a ++ b).zip v → e ∈ a.zip v ∨ e ∈ b.zip (v.drop a.length)
  | [], _, _, h => .inr (by simpa using h)
  | _ :: _, _, [], h => by simp at h
  | x :: a, b, y :: v, h => by
    simp only [List.cons_append, List.zip_cons_cons, List.mem_cons] at h
    rcases h with rfl | h
    · exact .inl (by simp)
    · rcases mem_zip_append a b v h with h | h
      · exact .inl (by simp [h])
      · exact .inr (by simpa using h)

theorem initReps_ok : ∀ (ps : List (ℕ × ParamType)) (ws : List ℕ) (reps0 : Reps),
    initReps cc ps ws = some reps0 →
    EnvOK σ reps0 (bindParams ps (ws.map σ)) ∧
      ∀ e ∈ (ps.flatMap (·.2.flat)).zip (ws.map σ), e.1.fits e.2 = true
  | [], _, reps0, h => by
    simp only [initReps, Option.some.injEq] at h; subst h; simp [EnvOK, bindParams]
  | (id, t) :: ps, ws, reps0, h => by
    simp only [initReps, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨rs, hrs, rest, hrest, h⟩ := h
    obtain ⟨hall, hfit⟩ := paramReps_ok hcc _ rs hrs
    obtain ⟨hE, hf⟩ := initReps_ok ps (ws.drop t.flat.length) rest hrest
    have hxs := take_zip_map σ t.flat ws
    have hvals : List.Forall₂ (RepOK2 σ) rs (((ws.map σ).take t.flat.length).zip t.flat) := by
      rw [hxs, List.forall₂_map_right_iff]
      exact hall
    have hdrop : (ws.map σ).drop t.flat.length = (ws.drop t.flat.length).map σ := by
      simp [List.map_drop]
    refine ⟨?_, ?_⟩
    · cases t with
      | scalar ty =>
        rcases rs with _ | ⟨r, _ | ⟨r2, rs⟩⟩
        · simp at h
        · simp only [Option.some.injEq] at h
          subst h
          obtain ⟨b, u', hb, hu', hu⟩ := List.forall₂_cons_left_iff.1 hvals
          rw [List.forall₂_nil_left_iff] at hu'
          subst hu'
          have hhead : (((ws.map σ).take (ParamType.scalar ty).flat.length).zip
              (ParamType.scalar ty).flat).headD (0, .field) = b := by rw [hu]; rfl
          simp only [bindParams]
          rw [hhead, hdrop]
          exact .cons ⟨rfl, hb⟩ hE
        · simp at h
      | array ts n =>
        simp only [Option.some.injEq] at h
        subst h
        simp only [bindParams, hdrop]
        exact .cons ⟨rfl, hvals⟩ hE
    · intro e he
      simp only [List.flatMap_cons] at he
      rcases mem_zip_append _ _ _ he with he | he
      · rw [List.zip_map_right] at he
        obtain ⟨⟨ty, w⟩, hmem, rfl⟩ := List.mem_map.1 he
        exact hfit (ty, w) hmem
      · rw [hdrop] at he
        exact hf e he

theorem fold_ok2 : ∀ (body : List Instruction) (reps : Reps) (env : Env), EnvOK σ reps env →
    ∀ reps', body.foldlM (step2 cc) reps = some reps' →
      ∃ env', body.foldlM Instruction.run env = some env' ∧ EnvOK σ reps' env'
  | [], reps, env, hE, reps', h => by
    simp only [List.foldlM_nil, Option.pure_def, Option.some.injEq] at h
    subst h; exact ⟨env, rfl, hE⟩
  | i :: body, reps, env, hE, reps', h => by
    simp only [List.foldlM_cons, Option.bind_eq_bind, Option.bind_eq_some_iff] at h
    obtain ⟨reps1, h1, h2⟩ := h
    obtain ⟨env1, he1, hE1⟩ := step2_ok hcc hE h1
    obtain ⟨env', he', hE'⟩ := fold_ok2 body reps1 env1 hE1 reps' h2
    exact ⟨env', by simp [List.foldlM_cons, he1, he'], hE'⟩

omit hcc in
theorem opFlats_ok {reps : Reps} {env : Env} (hE : EnvOK σ reps env) :
    ∀ (os : List Operand) (rss : List (List Rep2)), os.mapM (opFlat reps) = some rss →
      ∃ xss, os.mapM (·.flat env) = some xss ∧ List.Forall₂ (List.Forall₂ (FlatOK σ)) rss xss
  | [], rss, h => by
    simp only [List.mapM_nil, Option.pure_def, Option.some.injEq] at h
    subst h
    exact ⟨[], rfl, .nil⟩
  | o :: os, rss, h => by
    simp only [List.mapM_cons, Option.pure_def, Option.bind_eq_bind, Option.bind_eq_some_iff,
      Option.some.injEq] at h
    obtain ⟨rs, hr, rss', hrss', rfl⟩ := h
    obtain ⟨xs, hx, hok⟩ := opFlat_ok hE hr
    obtain ⟨xss, hxss, hall⟩ := opFlats_ok hE os rss' hrss'
    exact ⟨xs :: xss, by simp [List.mapM_cons, hx, hxss], .cons hok hall⟩

omit hcc in
theorem forall₂_flatten {α β : Type} {R : α → β → Prop} :
    ∀ {as : List (List α)} {bs : List (List β)}, List.Forall₂ (List.Forall₂ R) as bs →
      List.Forall₂ R as.flatten bs.flatten
  | [], [], .nil => .nil
  | _ :: _, _ :: _, .cons h ht => by
    simpa using List.rel_append h (forall₂_flatten ht)

theorem retWits_ok : ∀ (ws : List ℕ) (rs : List Rep2) (xs : List F),
    List.Forall₂ (FlatOK σ) rs xs → ws.length = rs.length →
      (ws.zip rs).all (fun (w, r) => retOK cc w r) = true →
      ws.map (fun i => (σ i).val) = xs.map ZMod.val
  | [], [], [], _, _, _ => rfl
  | w :: ws, r :: rs, x :: xs, .cons hr ht, hl, h => by
    simp only [List.zip_cons_cons, List.all_cons, Bool.and_eq_true] at h
    obtain ⟨h1, h2⟩ := h
    have hrest := retWits_ok ws rs xs ht (by simpa using hl) h2
    obtain ⟨X, hX, he⟩ := List.any_eq_true.1 h1
    have := eqVia_sound hcc he
    rw [eval_pvar, forms_sound hcc hr X hX] at this
    simp [hrest, this]
  | [], _ :: _, _, _, hl, _ => by simp at hl
  | _ :: _, [], _, _, hl, _ => by simp at hl
  | _ :: _, _ :: _, [], h, _, _ => by cases h
  | [], [], _ :: _, h, _, _ => by cases h

theorem rets_ok {reps : Reps} {env : Env} (hE : EnvOK σ reps env) (ws : List ℕ) (os : List Operand)
    (h : retsOK cc reps ws os = true) :
    ∃ vs, (do let outs ← os.mapM (·.flat env); some outs.flatten) = some vs ∧
      ws.map (fun i => (σ i).val) = vs.map ZMod.val := by
  unfold retsOK at h
  split at h
  · simp at h
  · next rss hrss =>
    simp only [Bool.and_eq_true, decide_eq_true_eq] at h
    obtain ⟨hl, hall⟩ := h
    obtain ⟨xss, hxss, hok⟩ := opFlats_ok hE os rss hrss
    exact ⟨xss.flatten, by simp [hxss],
      retWits_ok hcc ws rss.flatten xss.flatten (forall₂_flatten hok) hl hall⟩

end

/-- Acceptance by the checker implies the circuit implements the program. -/
theorem checkProg2_sound (P : Program) (C : Circuit) (h : checkProg2 P C = true) :
    SoundFunction C (ProgramSpec P) := by
  intro σ hσ
  have hcc : ∀ d ∈ C.opcodes, d.Holds σ := hσ
  unfold checkProg2 at h
  simp only [Bool.and_eq_true, decide_eq_true_eq] at h
  obtain ⟨hin, h⟩ := h
  split at h
  · simp at h
  · next reps0 hinit =>
    split at h
    · simp at h
    · next reps hfold =>
      obtain ⟨hE0, hfit⟩ := initReps_ok hcc P.params C.parameters reps0 hinit
      obtain ⟨env, hrun, hE⟩ := fold_ok2 hcc P.body reps0 _ hE0 reps hfold
      obtain ⟨vs, hvs, hm⟩ := rets_ok hcc hE C.returnValues P.rets h
      have hins : ∀ l : List ℕ, List.flatMap (fun a : ℕ => [(a : F)]) (l.map fun i => (σ i).val) =
          l.map σ := by
        intro l; induction l <;> simp_all [cast_val]
      refine ⟨by simp [hin], ?_, vs, ?_, hm⟩
      · intro e he
        rw [List.zip_map_right] at he
        obtain ⟨⟨ty, w⟩, hmem, rfl⟩ := List.mem_map.1 he
        have := hfit (ty, σ w) (by
          rw [List.zip_map_right]; exact List.mem_map.2 ⟨(ty, w), hmem, rfl⟩)
        simpa [cast_val] using this
      · have heval : ∀ l : List F, l = C.parameters.map σ → P.eval l = some vs := by
          rintro l rfl
          simp only [Program.eval, Option.bind_eq_bind]
          rw [hrun]
          simpa using hvs
        exact heval _ (by simpa using hins C.parameters)

theorem pick_sub {cc : List Opcode} {idx : List ℕ} {c : Opcode} (h : c ∈ pick cc idx) : c ∈ cc := by
  obtain ⟨k, -, hk⟩ := List.mem_filterMap.1 h
  exact List.mem_of_getElem? hk

theorem stepsWith_ok {cc : List Opcode} {σ : ℕ → F} (hcc : ∀ c ∈ cc, c.Holds σ) :
    ∀ (body : List Instruction) (cert : List (List ℕ)) (reps : Reps) (env : Env), EnvOK σ reps env →
      ∀ reps', stepsWith cc reps body cert = some reps' →
        ∃ env', body.foldlM Instruction.run env = some env' ∧ EnvOK σ reps' env'
  | [], [], reps, env, hE, reps', h => by
    simp only [stepsWith, Option.some.injEq] at h
    subst h; exact ⟨env, rfl, hE⟩
  | i :: body, ix :: cert, reps, env, hE, reps', h => by
    simp only [stepsWith, Option.bind_eq_some_iff] at h
    obtain ⟨reps1, h1, h2⟩ := h
    obtain ⟨env1, he1, hE1⟩ := step2_ok (fun c hc => hcc c (pick_sub hc)) hE h1
    obtain ⟨env', he', hE'⟩ := stepsWith_ok hcc body cert reps1 env1 hE1 reps' h2
    exact ⟨env', by simp [List.foldlM_cons, he1, he'], hE'⟩
  | [], _ :: _, _, _, _, _, h => by simp [stepsWith] at h
  | _ :: _, [], _, _, _, _, h => by simp [stepsWith] at h

/-- Acceptance with a certificate implies the circuit implements the program. -/
theorem checkProgWith_sound (P : Program) (C : Circuit) (cert : List (List ℕ)) (h : checkProgWith P C cert = true) :
    SoundFunction C (ProgramSpec P) := by
  intro σ hσ
  have hcc : ∀ d ∈ C.opcodes, d.Holds σ := hσ
  unfold checkProgWith at h
  simp only [Bool.and_eq_true, decide_eq_true_eq] at h
  obtain ⟨hin, h⟩ := h
  split at h
  · simp at h
  · next reps0 hinit =>
    split at h
    · simp at h
    · next reps hfold =>
      obtain ⟨hE0, hfit⟩ := initReps_ok hcc P.params C.parameters reps0 hinit
      obtain ⟨env, hrun, hE⟩ := stepsWith_ok hσ P.body cert reps0 _ hE0 reps hfold
      obtain ⟨vs, hvs, hm⟩ := rets_ok hcc hE C.returnValues P.rets h
      have hins : ∀ l : List ℕ, List.flatMap (fun a : ℕ => [(a : F)]) (l.map fun i => (σ i).val) =
          l.map σ := by
        intro l; induction l <;> simp_all [cast_val]
      refine ⟨by simp [hin], ?_, vs, ?_, hm⟩
      · intro e he
        rw [List.zip_map_right] at he
        obtain ⟨⟨ty, w⟩, hmem, rfl⟩ := List.mem_map.1 he
        have := hfit (ty, σ w) (by
          rw [List.zip_map_right]; exact List.mem_map.2 ⟨(ty, w), hmem, rfl⟩)
        simpa [cast_val] using this
      · have heval : ∀ l : List F, l = C.parameters.map σ → P.eval l = some vs := by
          rintro l rfl
          simp only [Program.eval, Option.bind_eq_bind]
          rw [hrun]
          simpa using hvs
        exact heval _ (by simpa using hins C.parameters)

theorem exists_mem_zip {α β : Type} {a : α} :
    ∀ {l : List α} {m : List β}, l.length = m.length → a ∈ l → ∃ b, (a, b) ∈ l.zip m
  | [], _, _, h => by simp at h
  | _ :: _, [], hl, _ => by simp at hl
  | x :: l, y :: m, hl, h => by
    rcases List.mem_cons.1 h with rfl | h
    · exact ⟨y, by simp⟩
    · obtain ⟨b, hb⟩ := exists_mem_zip (by simpa using hl) h
      exact ⟨b, by simp [hb]⟩

-- Checking every program takes more steps than the default budget.
set_option maxHeartbeats 2000000 in
/-- The test programs: the checker accepts every one outside
`uncoveredPrograms` with its certificate from `testProgramCerts`, and its
solved witness satisfies its circuit, decided by evaluation in the kernel. -/
theorem testPrograms_claims :
    ∀ e ∈ testPrograms, e.name ∉ uncoveredPrograms →
      SoundFunction e.fn (ProgramSpec e.prog) ∧ AllHold e.assignment e.fn.opcodes := by
  have hc : testPrograms.length = testProgramCerts.length ∧
      (testPrograms.zip testProgramCerts).all (fun (e, c) =>
        decide (e.name ∈ uncoveredPrograms) ||
          (checkProgWith e.prog e.fn c && decide (AllHold e.assignment e.fn.opcodes))) = true := by
    decide +kernel
  intro e he hn
  obtain ⟨c, hmem⟩ := exists_mem_zip hc.1 he
  have := List.all_eq_true.1 hc.2 _ hmem
  simp only [Bool.or_eq_true, Bool.and_eq_true, decide_eq_true_eq] at this
  obtain ⟨h1, h2⟩ := this.resolve_left hn
  exact ⟨checkProgWith_sound _ _ c h1, h2⟩

end AcirLean
