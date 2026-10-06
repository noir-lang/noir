/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Checker2

/-!
# Hint steps

`checkProg2` finds, for each instruction, the circuit constraints that
implement it by searching for known shapes. The optimizer produces shapes the
searches do not know, so some programs fail although their circuits are sound.

A hint step takes those constraints from a certificate instead. The
certificate names the polynomials involved (the instruction's result, and
the quotient, remainder or inverse a gadget introduces, which the compiler
assigned when it generated the circuit) and, for each fact the instruction
needs, a combination of the circuit's constraints that adds up to it. The
kernel checks each combination as a polynomial identity, so a wrong
certificate can only make a step fail.

Each rule says which facts its instruction needs; the bounds that rule out
wraparound come from the circuit's range checks, also named by the
certificate. Instructions without a hint step run `stepP` as before.
-/

namespace AcirLean

/-! ## The circuit, indexed

Certificates name constraints by position. Looking a position up in a list
walks the list, which the kernel does one cell at a time; a balanced tree
takes logarithmically many steps. -/

inductive OTree where
  | leaf
  | node (l : OTree) (v : Opcode) (r : OTree)

/-- The first `n` constraints of `l` as a balanced tree (the left half, the
middle one, the right half), and the rest of `l`. `fuel` bounds the depth. -/
def OTree.build : ℕ → ℕ → List Opcode → OTree × List Opcode
  | 0, _, l => (.leaf, l)
  | _ + 1, 0, l => (.leaf, l)
  | f + 1, n + 1, l =>
    let (t₁, l) := OTree.build f ((n + 1) / 2) l
    match l with
    | [] => (t₁, [])
    | v :: l =>
      let (t₂, l) := OTree.build f (n - (n + 1) / 2) l
      (.node t₁ v t₂, l)

/-- Position `i` of a tree built from `n` constraints. -/
def OTree.get? : OTree → ℕ → ℕ → Option Opcode
  | .leaf, _, _ => none
  | .node l v r, n, i =>
    if i < n / 2 then l.get? (n / 2) i
    else if i = n / 2 then some v
    else r.get? (n - 1 - n / 2) (i - n / 2 - 1)

structure Circ where
  list : List Opcode
  tree : OTree
  size : ℕ

def Circ.ofList (cc : List Opcode) : Circ :=
  ⟨cc, (OTree.build 64 cc.length cc).1, cc.length⟩

instance : Coe (List Opcode) Circ := ⟨Circ.ofList⟩

def Circ.get? (cc : Circ) (i : ℕ) : Option Opcode := cc.tree.get? cc.size i

/-! ## Combinations of facts -/

/-- A polynomial that is zero under every assignment satisfying the circuit, by
the circuit or by what the steps so far established. -/
inductive Src where
  /-- Constraint `i` of the circuit: an `AssertZero`, or a 1-bit range check on
  `w`, which gives `w² - w`. -/
  | con (i : ℕ)
  /-- Alternatives `i` and `j` of scalar `v`: both evaluate to its value. -/
  | same (v i j : ℕ)
  /-- Alternative `i` of scalar `v` minus `L`, for a scalar whose bounds are
  `L = M`. -/
  | fixed (v i : ℕ)
  /-- `A² - A` for alternative `i` of scalar `v`, for a scalar whose bounds
  are at most `1`. -/
  | bit (v i : ℕ)
  /-- `P² - P` for the side-effects flag `P`, which is `0` or `1`. -/
  | flagBit
  deriving DecidableEq

def srcPoly (cc : Circ) (s : Reps × Flag) : Src → Poly
  | .con i => match cc.get? i with
    | some (.assertZero ts) => ts
    | some (.range w 1) => psub (pmul (pvar w) (pvar w)) (pvar w)
    | _ => []
  | .same v i j => match s.1.lookup v with
    | some (.scalar r) => match r.alts[i]?, r.alts[j]? with
      | some A, some B => psub A B
      | _, _ => []
    | _ => []
  | .fixed v i => match s.1.lookup v with
    | some (.scalar r) => match r.alts[i]? with
      | some A => if r.L = r.M then psub A (pconst r.L) else []
      | none => []
    | _ => []
  | .bit v i => match s.1.lookup v with
    | some (.scalar r) => match r.alts[i]? with
      | some A => if r.M ≤ 1 then psub (pmul A A) A else []
      | none => []
    | _ => []
  | .flagBit => match s.2 with
    | some (P, _) => psub (pmul P P) P
    | none => []

/-- `coef · (Π mul) · g` for the fact `g` names. -/
structure Gen where
  src : Src
  mul : List ℕ
  coef : ℤ
  deriving DecidableEq

def genPoly (cc : Circ) (s : Reps × Flag) (g : Gen) : Poly :=
  pmul [⟨g.coef, g.mul⟩] (srcPoly cc s g.src)

abbrev Comb := List Gen

/-- `T` is the combination `cmb` of facts, as polynomials. -/
def combHolds (cc : Circ) (s : Reps × Flag) (cmb : Comb) (T : Poly) : Bool :=
  match key (psub T (cmb.flatMap (genPoly cc s))) with
  | .assertZero [] => true
  | _ => false

/-- `P < 2^k`: constraint `idx` range-checks a witness `w` to `k` bits, and
`cmb` shows `P = w` (times the flag, under one). -/
structure RangeEv where
  idx : ℕ
  cmb : Comb
  deriving DecidableEq

/-- `T` under the side-effects flag: `P · T` while a flag `P` is tracked. -/
def underFlag : Flag → Poly → Poly
  | none, T => T
  | some (P, _), T => pmul P T

def rangeOf (cc : Circ) (s : Reps × Flag) (P : Poly) (e : RangeEv) : Option ℕ :=
  match cc.get? e.idx with
  | some (.range w k) => if combHolds cc s e.cmb (underFlag s.2 (psub P (pvar w))) then some k else none
  | _ => none

/-- How a hint step shows `r < b`: a range-checked witness equals `b - r - 1`,
or, for a constant `b`, a witness range-checked to `k` bits equals `r + d`
with `2^k - d = b`. -/
inductive LtEv where
  | sub (e : RangeEv)
  | shift (d : ℕ) (e : RangeEv)
  deriving DecidableEq

/-- Which polynomial of an operand a fact is stated over: one of its
representation's polynomials, or another polynomial `H` with a combination
showing `H` equals one of them. -/
inductive Opnd where
  | alt (i : ℕ)
  | via (H : Poly) (i : ℕ) (c : Comb)
  /-- The constant `L`, for an operand whose bounds are `L = M`. -/
  | fixed
  /-- Like `via`, but `H` equals the operand only while the side-effects flag
  is on (the combination shows `P · (H - X) = 0`). Only for instructions the
  flag affects, whose result is `0` while it is off. -/
  | viaOn (H : Poly) (i : ℕ) (c : Comb)
  deriving DecidableEq

/-- A certificate entry for one instruction. `ia`, `ib` pick the polynomials
of the operands the facts are stated over. -/
inductive HStep where
  /-- Checked `add`, `sub` or `mul` on `u<n>`, `n ≥ 2`: `E = a op b`, and `E`
  is range-checked to at most `n` bits. -/
  | arith (E : Poly) (ia ib : Opnd) (c : Comb) (rng : RangeEv)
  /-- `div` or `mod` on `u<n>`: `a = b q + r`, `q` and `r` range-checked, and
  `r < b`. -/
  | divmod (q r : Poly) (ia ib : Opnd) (c : Comb) (qr rr : RangeEv) (lt : LtEv)
  /-- `eq`: `1 - (a - b) z - E = 0` and `(a - b) E = 0`. -/
  | eq (E z : Poly) (ia ib : Opnd) (c₁ c₂ : Comb)
  /-- `lt` on `u<n>`, `n ≥ 2`: `a - b + 2^n E - r = 0` with `E` a bit and `r`
  range-checked to at most `n` bits, so `E = 1` exactly when `a < b`. -/
  | lt (E r : Poly) (ia ib : Opnd) (c cb : Comb) (rr : RangeEv)
  /-- Unchecked `add` of `s · y` and `(1 - s) · z` for a bit `s` (an `if`/`else`
  merge): the sum is `y` or `z`, so it is bounded by the larger of their
  bounds. `y` and `z` are operands named by their polynomials `iy`, `iz`;
  `cs` shows `s² - s = 0`, and `ca`, `cb` the two products. -/
  | mux (sel : Poly) (y : Operand) (iy : ℕ) (z : Operand) (iz : ℕ) (ia ib : Opnd) (cs ca cb : Comb)
  /-- `constrain a == b`: `a - b = 0`. -/
  | constrain (ia ib : Opnd) (c : Comb)
  /-- `constrain a != b`: `(a - b) z = 1`. -/
  | constrainNe (z : Poly) (ia ib : Opnd) (c : Comb)
  /-- A binary instruction on two operands with known constant values (bounds
  `L = M`): the result is the constant `c` the instruction computes, or `P · c`
  for an instruction the side-effects flag `P` affects. -/
  | fold
  /-- `eq` where `a - b` is `0` or `1`, which the combination shows by
  `(a - b)² - (a - b) = 0`: the result is `1 - (a - b)`. -/
  | eqBit (ia ib : Opnd) (c : Comb)
  /-- A binary instruction whose operands equal the constants `ca`, `cb`
  whenever the side-effects flag is on, which the combinations show: the
  result is `P · c` for an instruction the flag `P` affects, or `c` with no
  flag. -/
  | foldOn (ca cb : ℕ) (ia ib : Opnd) (c₁ c₂ : Comb)
  deriving DecidableEq

/-! ## The rules -/

def opndPoly (cc : Circ) (s : Reps × Flag) (onlyOn : Bool) (r : Rep2) : Opnd → Option Poly
  | .alt i => r.alts[i]?
  | .via H i c => do
    let X ← r.alts[i]?
    if combHolds cc s c (psub H X) then some H else none
  | .fixed => if r.L = r.M then some (pconst r.L) else none
  | .viaOn H i c => do
    let X ← r.alts[i]?
    if onlyOn ∧ combHolds cc s c (underFlag s.2 (psub H X)) then some H else none

/-- The operands' polynomials the certificate names. -/
def operands (cc : Circ) (s : Reps × Flag) (a b : Operand) (ia ib : Opnd)
    (onlyOn : Bool := false) : Option (Rep2 × Rep2 × Poly × Poly) := do
  let ra ← opRep s.1 a
  let rb ← opRep s.1 b
  let Xa ← opndPoly cc s onlyOn ra ia
  let Xb ← opndPoly cc s onlyOn rb ib
  some (ra, rb, Xa, Xb)

/-- `op a b` as a polynomial, for `add`, `sub` and `mul`. -/
def arithPoly : BinaryOp → Poly → Poly → Option Poly
  | .add, A, B => some (A ++ B)
  | .sub, A, B => some (psub A B)
  | .mul, A, B => some (pmul A B)
  | _, _, _ => none

/-- The result's polynomial under the flag: `P · E` for an instruction the
flag affects. -/
def flagged : Flag → Poly → Poly
  | none, E => E
  | some (P, _), E => pmul P E

/-- `r < b` from the evidence, given `r ≤ Mr` and `b`'s representation. -/
def ltOK (cc : Circ) (s : Reps × Flag) (Xb : Poly) (b : Rep2) (r : Poly) (Mr : ℕ) : LtEv → Bool
  | .sub e =>
    match rangeOf cc s (psub (psub Xb r) (pconst 1)) e with
    | some k => decide (2 ^ k + Mr < p)
    | none => false
  | .shift d e =>
    decide (b.L = b.M) &&
    match rangeOf cc s (r ++ pconst d) e with
    | some k => decide (2 ^ k = b.M + d) && decide (Mr + d < p)
    | none => false

def hintStep (cc : Circ) (s : Reps × Flag) : Instruction → HStep → Option (Reps × Flag)
  | .bin d op u a b, .arith E ia ib c rng => do
    let (ra, rb, Xa, Xb) ← operands cc s a b ia ib true
    let T ← arithPoly op Xa Xb
    match ra.ty with
    | .uint n =>
      let k ← rangeOf cc s E rng
      if u = false ∧ 2 ≤ n ∧ k ≤ n ∧ (op = .mul → n = 128 → ra.M * rb.M < 2 ^ 128) ∧
          combHolds cc s c (underFlag s.2 (psub E T)) then
        some ((d, .scalar ⟨[flagged s.2 E], .uint n, 0, 2 ^ k - 1⟩) :: s.1, s.2)
      else none
    | _ => none
  | .bin d op u a b, .divmod q r ia ib c qr rr lt => do
    let (ra, rb, Xa, Xb) ← operands cc s a b ia ib true
    let kq ← rangeOf cc s q qr
    let kr ← rangeOf cc s r rr
    match ra.ty with
    | .uint n =>
      if (op = .div ∨ op = .mod) ∧ fitsBoth ra rb ∧ rb.M * (2 ^ kq - 1) + (2 ^ kr - 1) < p ∧
          combHolds cc s c (underFlag s.2 (psub (psub Xa (pmul Xb q)) r)) ∧
          ltOK cc s Xb rb r (2 ^ kr - 1) lt then
        let res := if op = .div then q else r
        let M := if op = .div then 2 ^ kq - 1 else 2 ^ kr - 1
        some ((d, .scalar ⟨[flagged s.2 res], .uint n, 0, M⟩) :: s.1, s.2)
      else none
    | _ => none
  | .bin d op _ a b, .eq E z ia ib c₁ c₂ => do
    let (ra, rb, Xa, Xb) ← operands cc s a b ia ib
    let D := psub Xa Xb
    if op = .eq ∧ fitsBoth ra rb ∧ combHolds cc s c₁ (psub (psub (pconst 1) (pmul D z)) E) ∧
        combHolds cc s c₂ (pmul D E) then
      some ((d, .scalar ⟨[E], .uint 1, 0, 1⟩) :: s.1, s.2)
    else none
  | .bin d op _ a b, .lt E r ia ib c cb rr => do
    let (ra, rb, Xa, Xb) ← operands cc s a b ia ib
    let kr ← rangeOf cc s r rr
    match ra.ty with
    | .uint n =>
      if op = .lt ∧ 2 ≤ n ∧ 2 ^ (n + 1) < p ∧ kr ≤ n ∧ fitsBoth ra rb ∧
          combHolds cc s c (psub (psub Xa Xb ++ pmul (pconst (2 ^ n)) E) r) ∧
          combHolds cc s cb (psub (pmul E E) E) then
        some ((d, .scalar ⟨[E], .uint 1, 0, 1⟩) :: s.1, s.2)
      else none
    | _ => none
  | .bin d op u a b, .mux sel y iy z iz ia ib cs ca cb => do
    let (ra, _, Xa, Xb) ← operands cc s a b ia ib
    let ry ← opRep s.1 y
    let rz ← opRep s.1 z
    let Y ← ry.alts[iy]?
    let Z ← rz.alts[iz]?
    if op = .add ∧ u = true ∧ ra.ty ≠ .field ∧ ry.M + rz.M < p ∧
        combHolds cc s cs (psub (pmul sel sel) sel) ∧
        combHolds cc s ca (psub Xa (pmul sel Y)) ∧
        combHolds cc s cb (psub Xb (pmul (psub (pconst 1) sel) Z)) then
      some ((d, .scalar ⟨[Xa ++ Xb], ra.ty, min ry.L rz.L, max ry.M rz.M⟩) :: s.1, s.2)
    else none
  | .constrain a b _, .constrain ia ib c => do
    let (_, _, Xa, Xb) ← operands cc s a b ia ib
    if combHolds cc s c (psub Xa Xb) then some s else none
  | .constrainNe a b _, .constrainNe z ia ib c => do
    let (_, _, Xa, Xb) ← operands cc s a b ia ib
    if combHolds cc s c (underFlag s.2 (psub (pconst 1) (pmul (psub Xa Xb) z))) then some s
    else none
  | .bin d op _ a b, .eqBit ia ib c => do
    let (_, _, Xa, Xb) ← operands cc s a b ia ib
    let D := psub Xa Xb
    if op = .eq ∧ combHolds cc s c (psub (pmul D D) D) then
      some ((d, .scalar ⟨[psub (pconst 1) D], .uint 1, 0, 1⟩) :: s.1, s.2)
    else none
  | .bin d op u a b, .fold => do
    let ra ← opRep s.1 a
    let rb ← opRep s.1 b
    if ra.L = ra.M ∧ rb.L = rb.M then
      let (v, t) ← op.apply u (ra.L : F) (rb.L : F) ra.ty
      match s.2 with
      | some (P, _) =>
        if op.predicated u ra.ty then
          some ((d, .scalar ⟨[pmul P (pconst v.val)], t, 0, v.val⟩) :: s.1, s.2)
        else some ((d, .scalar ⟨[pconst v.val], t, v.val, v.val⟩) :: s.1, s.2)
      | none => some ((d, .scalar ⟨[pconst v.val], t, v.val, v.val⟩) :: s.1, s.2)
    else none
  | .bin d op u a b, .foldOn ca cb ia ib c₁ c₂ => do
    let (ra, _, Xa, Xb) ← operands cc s a b ia ib
    if ca < p ∧ cb < p ∧ combHolds cc s c₁ (underFlag s.2 (psub Xa (pconst ca))) ∧
        combHolds cc s c₂ (underFlag s.2 (psub Xb (pconst cb))) then
      let (v, t) ← op.apply u (ca : F) (cb : F) ra.ty
      match s.2 with
      | some (P, _) =>
        if op.predicated u ra.ty then
          some ((d, .scalar ⟨[pmul P (pconst v.val)], t, 0, v.val⟩) :: s.1, s.2)
        else none
      | none => some ((d, .scalar ⟨[pconst v.val], t, v.val, v.val⟩) :: s.1, s.2)
    else none
  | _, _ => none

/-- The SSA value an instruction defines. -/
def Instruction.dst? : Instruction → Option ℕ
  | .bin d .. | .not d _ | .cast d .. | .truncate d .. | .arrayGet d .. | .arraySet d .. | .makeArray d .. => some d
  | _ => none

/-- Adds `H` to scalar `d`'s polynomials, where the combination `c` shows `H`
equals its `i`th. -/
def addAlias (cc : Circ) (s : Reps × Flag) (d : ℕ) (H : Poly) (i : ℕ) (c : Comb) :
    Option (Reps × Flag) :=
  match s.1.lookup d with
  | some (.scalar r) => match r.alts[i]? with
    | some X => if combHolds cc s c (psub H X) then some ((d, .scalar { r with alts := r.alts ++ [H] }) :: s.1, s.2) else none
    | none => none
  | _ => none

/-- `c + w` or `c - w` for a witness `w` range-checked to `k` bits by
constraint `idx`: a polynomial with known bounds. -/
structure Form where
  c : ℕ
  neg : Bool
  idx : ℕ
  deriving DecidableEq

def Form.poly (cc : Circ) (f : Form) : Option (Poly × ℕ × ℕ) :=
  match cc.get? f.idx with
  | some (.range w k) =>
    if f.neg then
      if 2 ^ k - 1 ≤ f.c ∧ f.c < p then some (psub (pconst f.c) (pvar w), f.c - (2 ^ k - 1), f.c) else none
    else if f.c + 2 ^ k - 1 < p then some (pconst f.c ++ pvar w, f.c, f.c + 2 ^ k - 1) else none
  | _ => none

/-- Bounds for a result by cases on a bit `sel`: polynomial `i` of the
result equals `f₁` where `sel = 1` and `f₂` where `sel = 0`. -/
structure CaseBound where
  i : ℕ
  sel : Poly
  cs : Comb
  f₁ : Form
  c₁ : Comb
  f₂ : Form
  c₂ : Comb
  deriving DecidableEq

def tighten (cc : Circ) (s : Reps × Flag) (d : ℕ) (b : CaseBound) : Option (Reps × Flag) := do
  let .scalar r ← s.1.lookup d | none
  let X ← r.alts[b.i]?
  let (F₁, L₁, M₁) ← b.f₁.poly cc
  let (F₂, L₂, M₂) ← b.f₂.poly cc
  if combHolds cc s b.cs (psub (pmul b.sel b.sel) b.sel) ∧
      combHolds cc s b.c₁ (pmul b.sel (psub X F₁)) ∧
      combHolds cc s b.c₂ (pmul (psub (pconst 1) b.sel) (psub X F₂)) then
    some ((d, .scalar { r with L := max r.L (min L₁ L₂), M := min r.M (max M₁ M₂) }) :: s.1, s.2)
  else none

/-- A certificate entry for one instruction: the constraints `stepP` runs over
or a hint step, optionally a polynomial to add to the result's (see
`addAlias`), and optionally tighter bounds for it (see `tighten`). -/
structure Entry where
  ix : List ℕ
  step : Option HStep
  extra : Option (Poly × ℕ × Comb)
  bound : Option CaseBound := none
  deriving DecidableEq

def stepE (cc : Circ) (s : Reps × Flag) (i : Instruction) (e : Entry) : Option (Reps × Flag) := do
  let s' ← match e.step with
    | none => stepP (e.ix.filterMap cc.get?) s i
    | some h => hintStep cc s i h
  let s'' ← match e.extra, i.dst? with
    | none, _ => some s'
    | some (H, j, c), some d => addAlias cc s' d H j c
    | some _, none => none
  match e.bound, i.dst? with
  | none, _ => some s''
  | some b, some d => tighten cc s'' d b
  | some _, none => none

/-! ## One theorem per step

`checkProgH` runs the whole body in one kernel computation, which keeps every
intermediate state until it ends. Instead, each step can be checked on its own
(`StepCert.ok`, one theorem per step), and a separate pass (`linkSteps`) checks
that the steps fit together: each step's inputs are what the steps before it
produced. -/

/-- One step of a certificate checked step by step: the instruction, its entry,
the entries of the values it reads, the flag before and after it, and the
entries the step changes (the value it defines, and operands whose bounds it
tightens). -/
structure StepCert where
  i : Instruction
  e : Entry
  reps : Reps
  F : Flag
  F' : Flag
  out : Reps
  deriving DecidableEq

/-- The entries of `after` that differ from `before`, among the value `i`
defines and the values in `before`. -/
def changed (i : Instruction) (before after : Reps) : Reps :=
  ((i.dst?.toList ++ before.map (·.1)).eraseDups).filterMap fun v =>
    match after.lookup v with
    | some r => if decide (before.lookup v = some r) then none else some (v, r)
    | none => none

/-- The step, run from just the entries it reads, gives the flag `F'` and
changes exactly the entries `out`. -/
def StepCert.ok (cc : Circ) (sc : StepCert) : Bool :=
  match stepE cc (sc.reps, sc.F) sc.i sc.e with
  | some s' => decide (s'.2 = sc.F') && decide (changed sc.i sc.reps s'.1 = sc.out)
  | none => false

/-- The steps fit together: step `k` is the `k`th instruction, its flag and the
entries it reads are the current ones, and the entries it changes become
current. -/
def linkSteps : Reps × Flag → List Instruction → List StepCert → Option (Reps × Flag)
  | s, [], [] => some s
  | s, i :: is, sc :: scs =>
    if decide (sc.i = i) && decide (sc.F = s.2) &&
        sc.reps.all (fun (v, r) => decide (s.1.lookup v = some r)) then
      linkSteps (sc.out ++ s.1, sc.F') is scs
    else none
  | _, _, _ => none

def OTree.toList : OTree → List Opcode
  | .leaf => []
  | .node l v r => l.toList ++ v :: r.toList

/-- Run the body, each step as its certificate entry says. -/
def stepsWithH (cc : Circ) :
    Reps × Flag → List Instruction → List Entry → Option (Reps × Flag)
  | s, [], [] => some s
  | s, i :: is, e :: es => (stepE cc s i e).bind fun r => stepsWithH cc r is es
  | _, _, _ => none

/-- Return witness `w` equals scalar `r`: as `retOK` finds, or through a
combination showing `w = ` one of `r`'s polynomials. -/
def retOKH (cc : Circ) (s : Reps × Flag) (w : ℕ) (r : Rep2) : Option (ℕ × Comb) → Bool
  | none => retOK cc.list w r
  | some (i, c) => match r.alts[i]? with
    | some X => combHolds cc s c (psub (pvar w) X)
    | none => false

def retsOKH (cc : Circ) (s : Reps × Flag) (ws : List ℕ) (os : List Operand)
    (hs : List (Option (ℕ × Comb))) : Bool :=
  match os.mapM (opFlat s.1) with
  | none => false
  | some rss => decide (ws.length = rss.flatten.length) &&
    ((ws.zip rss.flatten).zip (hs ++ List.replicate rss.flatten.length none)).all
      fun ((w, r), h) => retOKH cc s w r h

/-- `checkProgWith` with hint steps and return-value hints. -/
def checkProgH (P : Program) (C : Circuit) (cert : List Entry)
    (rets : List (Option (ℕ × Comb))) : Bool :=
  let cc := Circ.ofList C.opcodes
  decide (C.parameters.length = P.inputTypes.length) &&
    match initReps C.opcodes P.params C.parameters with
    | none => false
    | some reps0 =>
      match stepsWithH cc (reps0, none) P.body cert with
      | none => false
      | some s => retsOKH cc s C.returnValues P.rets rets

/-- `checkProgH`, with the steps checked separately: `cc` is the circuit as a
tree, the steps' `StepCert.ok` are separate theorems, and this checks the rest. -/
def checkProgSteps (P : Program) (C : Circuit) (cc : Circ) (scs : List StepCert)
    (rets : List (Option (ℕ × Comb))) : Bool :=
  decide (C.parameters.length = P.inputTypes.length) && decide (cc.tree.toList = C.opcodes) &&
    decide (cc.size = C.opcodes.length) &&
    match initReps C.opcodes P.params C.parameters with
    | none => false
    | some reps0 =>
      match linkSteps (reps0, none) P.body scs with
      | none => false
      | some s => retsOKH ⟨C.opcodes, cc.tree, cc.size⟩ s C.returnValues P.rets rets

end AcirLean
