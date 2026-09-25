/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Canon
import AcirLean.Spec.Programs
import AcirLean.Templates.Corpus

/-!
# The checker

`checkProg P C` decides whether circuit `C` implements program `P`. It walks the
instructions, tracking where each SSA value lives in `C` (a witness, or `1 - w`
for a comparison result), and for each instruction looks for the proved gadget
template, renamed onto a block of fresh witnesses, among `C`'s constraints. The
search for that block is untrusted: only the membership test that follows it
matters for soundness. `checkProg_sound` proves once that acceptance implies
`SoundFunction C (CorpusSpec P)`.
-/

namespace AcirLean

/-- Where an SSA value lives: a witness, or `1 - w`. -/
inductive Rep where
  | wit (w : ℕ)
  | notw (w : ℕ)
  deriving DecidableEq

def present (cc : List Opcode) (c : Opcode) : Bool := decide (c.canon ∈ cc)

def divMap (wa wb s : ℕ) : List ℕ := [wa, wb, s, s + 1, s + 2, s + 3, s + 4, s + 5, s + 6, s + 7]
def ltMap (wa wb s : ℕ) : List ℕ := [wa, wb, s, s + 1, s + 2]

/-- The instruction's template found at fresh block `s`, and where its result lives. -/
def instrRep (n : ℕ) (cc : List Opcode) (reps : List Rep) (i : CorpusInstruction) (s : ℕ) : Option Rep :=
  match reps[i.a]?, reps[i.b]? with
  | some (.wit wa), some (.wit wb) =>
    match i.op with
    | .div =>
      if (divVarGadget n).all (fun c => present cc (c.rename (witnessAt (divMap wa wb s))))
      then some (.wit (s + 1)) else none
    | .lt =>
      if (moreThanEqGadget n).all (fun c => present cc (c.rename (witnessAt (ltMap wa wb s))))
      then some (.notw s) else none
  | _, _ => none

def findStart (n : ℕ) (cc : List Opcode) (reps : List Rep) (i : CorpusInstruction) (bound : ℕ) : Option ℕ :=
  (List.range bound).find? fun s => (instrRep n cc reps i s).isSome

def stepRep (n : ℕ) (cc : List Opcode) (bound : ℕ) (st : Option (List Rep)) (i : CorpusInstruction) :
    Option (List Rep) :=
  st.bind fun reps =>
    (findStart n cc reps i bound).bind fun s => (instrRep n cc reps i s).map (reps ++ [·])

def Opcode.witnesses : Opcode → List ℕ
  | .assertZero ts => ts.flatMap (·.witnesses)
  | .range w _ => [w]

def maxWitness (C : Circuit) : ℕ := (C.opcodes.flatMap Opcode.witnesses).foldl max 0

def returnOK (cc : List Opcode) (r : ℕ) : Rep → Bool
  | .wit w => present cc (.assertZero [⟨1, [r]⟩, ⟨-1, [w]⟩])
  | .notw w => present cc (.assertZero [⟨1, []⟩, ⟨-1, [r]⟩, ⟨-1, [w]⟩])

def checkProg (P : CorpusProgram) (C : Circuit) : Bool :=
  let n := P.width
  let cc := C.opcodes.map Opcode.canon
  decide (n ∈ pinnedWidths) && decide (C.parameters.length = P.nparams) &&
    C.parameters.all (fun w => present cc (.range w n)) &&
    match C.returnValues, P.body.foldl (stepRep n cc (maxWitness C + 1)) (some (C.parameters.map .wit)) with
    | [r], some reps =>
      match reps[P.ret]? with
      | some rep => returnOK cc r rep
      | none => false
    | _, _ => false

/-! ## Soundness -/

def evalStep (vals : Option (List ℕ)) (i : CorpusInstruction) : Option (List ℕ) :=
  vals.bind fun vs =>
    let x := vs.getD i.a 0
    let y := vs.getD i.b 0
    match i.op with
    | .div => if y = 0 then none else some (vs ++ [x / y])
    | .lt => some (vs ++ [if x < y then 1 else 0])

theorem eval_eq (P : CorpusProgram) (ins : List ℕ) :
    P.eval ins = (P.body.foldl evalStep (some ins)).bind fun vs => vs[P.ret]? := rfl

/-- Where a value lives, and what it is. -/
def RepOK (n : ℕ) (σ : ℕ → F) : Rep → ℕ → Prop
  | .wit w, v => (σ w).val = v ∧ v < 2 ^ n
  | .notw w, v => (1 - σ w).val = v

theorem present_sat {cc : List Opcode} {σ : ℕ → F} (hcc : ∀ d ∈ cc, d.Holds σ) {c : Opcode}
    (h : present cc c = true) : c.Holds σ :=
  (Opcode.canon_sat σ c).1 (hcc _ (of_decide_eq_true h))

theorem template_sat {cc : List Opcode} {σ : ℕ → F} (hcc : ∀ d ∈ cc, d.Holds σ) {T : List Opcode}
    {f : ℕ → ℕ} (h : T.all (fun c => present cc (c.rename f)) = true) : AllHold (σ ∘ f) T := by
  intro c hc
  exact (Opcode.holds_rename σ f c).1 (present_sat hcc (List.all_eq_true.1 h c hc))

theorem forall2_get {n : ℕ} {σ : ℕ → F} :
    ∀ {reps : List Rep} {vals : List ℕ}, List.Forall₂ (RepOK n σ) reps vals →
      ∀ {k : ℕ} {r : Rep}, reps[k]? = some r → ∃ v, vals[k]? = some v ∧ RepOK n σ r v
  | [], [], _, k, r, h => by simp at h
  | _ :: _, _ :: _, .cons hh ht, 0, r, h => by
    simp at h; subst h; exact ⟨_, rfl, hh⟩
  | _ :: _, _ :: _, .cons _ ht, k + 1, r, h => by
    simpa using forall2_get ht (by simpa using h)

theorem instr_ok {n : ℕ} (hn : n ∈ pinnedWidths) {cc : List Opcode} {σ : ℕ → F}
    (hcc : ∀ d ∈ cc, d.Holds σ) {reps : List Rep} {vals : List ℕ}
    (hf : List.Forall₂ (RepOK n σ) reps vals) {i : CorpusInstruction} {s : ℕ} {r : Rep}
    (h : instrRep n cc reps i s = some r) :
    ∃ v, evalStep (some vals) i = some (vals ++ [v]) ∧ RepOK n σ r v := by
  have hbd := pinned_bounds hn
  unfold instrRep at h
  split at h
  next wa wb ha hb =>
    obtain ⟨x, hx, ⟨hxv, hxn⟩⟩ := forall2_get hf ha
    obtain ⟨y, hy, ⟨hyv, hyn⟩⟩ := forall2_get hf hb
    cases hop : i.op <;> rw [hop] at h <;> simp only at h <;> split at h <;>
      simp only [Option.some.injEq, reduceCtorEq] at h <;> subst h
    · -- div
      rename_i hall
      have hT := template_sat hcc hall
      have e1 := hT (.assertZero [⟨1, []⟩, ⟨-1, [1, 2]⟩]) (by simp [divVarGadget])
      simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil, List.sum_cons,
        List.sum_nil, Function.comp, witnessAt, divMap] at e1
      have hy0 : y ≠ 0 := by
        intro h0
        have : σ wb = 0 := (ZMod.val_eq_zero _).1 (by rw [hyv, h0])
        simp [this] at e1
      have hq : (σ (s + 1)).val = x / y ∧ (σ (s + 1)).val < 2 ^ n := by
        have r3 := hT (.range 3 n) (by simp [divVarGadget])
        simp only [Opcode.Holds, Range, Function.comp, witnessAt, divMap] at r3
        refine ⟨?_, by simpa using r3⟩
        rcases pinned_cases hn with h' | rfl
        · have := (divVarGadget_sound (by omega) _ hT (by simpa [witnessAt, divMap, hyv] using hyn)).1
          simpa [witnessAt, divMap, hxv, hyv] using this
        · have := (divVarGadget128_sound _ hT (by simpa [witnessAt, divMap, hyv] using hyn)).1
          simpa [witnessAt, divMap, hxv, hyv] using this
      refine ⟨x / y, ?_, hq.1, by rw [← hq.1]; exact hq.2⟩
      simp [evalStep, hop, hx, hy, hy0]
    · -- lt
      rename_i hall
      have hT := template_sat hcc hall
      have hge := moreThanEqGadget_sound (by omega) hbd.2 _ hT (by simpa [witnessAt, ltMap, hxv] using hxn)
        (by simpa [witnessAt, ltMap, hyv] using hyn)
      simp only [geSpec, Function.comp, witnessAt, ltMap] at hge
      simp only [List.getD_cons_zero, List.getD_cons_succ] at hge
      rw [hxv, hyv] at hge
      exact ⟨_, by simp [evalStep, hop, hx, hy], not_ge hge⟩
  next => simp at h

theorem fold_ok {n : ℕ} (hn : n ∈ pinnedWidths) {cc : List Opcode} {σ : ℕ → F}
    (hcc : ∀ d ∈ cc, d.Holds σ) (bound : ℕ) :
    ∀ (body : List CorpusInstruction) (reps : List Rep) (vals : List ℕ),
      List.Forall₂ (RepOK n σ) reps vals → ∀ reps',
        body.foldl (stepRep n cc bound) (some reps) = some reps' →
        ∃ vals', body.foldl evalStep (some vals) = some vals' ∧
          List.Forall₂ (RepOK n σ) reps' vals'
  | [], reps, vals, hf, reps', h => by
    simp at h; subst h; exact ⟨vals, rfl, hf⟩
  | i :: body, reps, vals, hf, reps', h => by
    simp only [List.foldl_cons] at h
    cases hs : stepRep n cc bound (some reps) i with
    | none =>
      rw [hs] at h
      have : ∀ l : List CorpusInstruction, l.foldl (stepRep n cc bound) none = none := by
        intro l; induction l with
        | nil => rfl
        | cons _ _ ih => simpa [stepRep] using ih
      rw [this] at h; exact absurd h (by simp)
    | some reps1 =>
      rw [hs] at h
      simp only [stepRep, Option.bind_some] at hs
      cases hfs : findStart n cc reps i bound with
      | none => rw [hfs] at hs; simp at hs
      | some s =>
        rw [hfs] at hs
        simp only [Option.bind_some] at hs
        cases hr : instrRep n cc reps i s with
        | none => rw [hr] at hs; simp at hs
        | some r =>
          rw [hr] at hs; simp at hs; subst hs
          obtain ⟨v, hv, hok⟩ := instr_ok hn hcc hf hr
          have hf' : List.Forall₂ (RepOK n σ) (reps ++ [r]) (vals ++ [v]) :=
            List.rel_append hf (.cons hok .nil)
          obtain ⟨vals', h1, h2⟩ := fold_ok hn hcc bound body _ _ hf' reps' h
          exact ⟨vals', by simp only [List.foldl_cons, hv]; exact h1, h2⟩

theorem checkProg_sound (P : CorpusProgram) (C : Circuit) (h : checkProg P C = true) :
    SoundFunction C (CorpusSpec P) := by
  intro σ hσ
  have hcc : ∀ d ∈ C.opcodes.map Opcode.canon, d.Holds σ := by
    intro d hd
    obtain ⟨c, hc, rfl⟩ := List.mem_map.1 hd
    exact (Opcode.canon_sat σ c).2 (hσ c hc)
  unfold checkProg at h
  simp only [Bool.and_eq_true, decide_eq_true_eq] at h
  obtain ⟨⟨⟨hn, hlen⟩, hin⟩, hrest⟩ := h
  -- the parameters
  have hparams : List.Forall₂ (RepOK P.width σ) (C.parameters.map .wit)
      (C.parameters.map fun i => (σ i).val) := by
    rw [List.forall₂_map_left_iff, List.forall₂_map_right_iff]
    apply List.forall₂_same.2
    intro w hw
    refine ⟨rfl, ?_⟩
    have := present_sat hcc (List.all_eq_true.1 hin w hw)
    simpa [Opcode.Holds, Range] using this
  split at hrest
  next r reps hret hfold =>
    split at hrest
    next rep hrep =>
      obtain ⟨vals', hv, hf⟩ := fold_ok hn hcc _ P.body _ _ hparams reps hfold
      obtain ⟨v, hvv, hok⟩ := forall2_get hf hrep
      refine ⟨by simp [hlen], ?_, v, ?_, ?_⟩
      · intro x hx
        obtain ⟨w, hw, rfl⟩ := List.mem_map.1 hx
        have := present_sat hcc (List.all_eq_true.1 hin w hw)
        simpa [Opcode.Holds, Range] using this
      · rw [eval_eq, hv]; simpa using hvv
      · rw [hret]
        simp only [List.map_cons, List.map_nil, List.cons.injEq, and_true]
        cases rep with
        | wit w =>
          have e := present_sat hcc hrest
          simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
            List.sum_cons, List.sum_nil] at e
          rw [show σ r = σ w by push_cast at e; linear_combination e]
          exact hok.1
        | notw w =>
          have e := present_sat hcc hrest
          simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
            List.sum_cons, List.sum_nil] at e
          rw [show σ r = 1 - σ w by push_cast at e; linear_combination -e]
          exact hok
    next => simp at hrest
  next => simp at hrest

/-- The corpus: the checker accepts every program's circuit, and ACVM's witness
satisfies it. Both are decided by evaluation in the kernel. -/
theorem corpus_claims :
    ∀ e ∈ corpus, SoundFunction e.fn (CorpusSpec e.prog) ∧ AllHold e.assignment e.fn.opcodes := by
  have hc : corpus.all (fun e => checkProg e.prog e.fn && decide (AllHold e.assignment e.fn.opcodes)) =
      true := by decide +kernel
  intro e he
  have := List.all_eq_true.1 hc e he
  simp only [Bool.and_eq_true, decide_eq_true_eq] at this
  exact ⟨checkProg_sound _ _ this.1, this.2⟩

end AcirLean
