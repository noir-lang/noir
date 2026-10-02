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

/-! ## Combinations of constraints -/

/-- `coef · (Π mul) · g`, where `g` is constraint `idx` of the circuit: an
`AssertZero`, or a 1-bit range check on `w`, which gives `w² - w = 0`. -/
structure Gen where
  idx : ℕ
  mul : List ℕ
  coef : ℤ
  deriving DecidableEq

def genPoly (cc : List Opcode) (g : Gen) : Poly :=
  match cc[g.idx]? with
  | some (.assertZero ts) => pmul [⟨g.coef, g.mul⟩] ts
  | some (.range w 1) => pmul [⟨g.coef, g.mul⟩] (psub (pmul (pvar w) (pvar w)) (pvar w))
  | _ => []

abbrev Comb := List Gen

/-- `T` is the combination `cmb` of constraints, as polynomials. -/
def combHolds (cc : List Opcode) (cmb : Comb) (T : Poly) : Bool :=
  match key (psub T (cmb.flatMap (genPoly cc))) with
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

def rangeOf (cc : List Opcode) (f : Flag) (P : Poly) (e : RangeEv) : Option ℕ :=
  match cc[e.idx]? with
  | some (.range w k) => if combHolds cc e.cmb (underFlag f (psub P (pvar w))) then some k else none
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
  /-- `constrain a == b`: `a - b = 0`. -/
  | constrain (ia ib : Opnd) (c : Comb)
  /-- `constrain a != b`: `(a - b) z = 1`. -/
  | constrainNe (z : Poly) (ia ib : Opnd) (c : Comb)
  deriving DecidableEq

/-! ## The rules -/

def opndPoly (cc : List Opcode) (r : Rep2) : Opnd → Option Poly
  | .alt i => r.alts[i]?
  | .via H i c => do
    let X ← r.alts[i]?
    if combHolds cc c (psub H X) then some H else none

/-- The operands' polynomials the certificate names. -/
def operands (cc : List Opcode) (reps : Reps) (a b : Operand) (ia ib : Opnd) :
    Option (Rep2 × Rep2 × Poly × Poly) := do
  let ra ← opRep reps a
  let rb ← opRep reps b
  let Xa ← opndPoly cc ra ia
  let Xb ← opndPoly cc rb ib
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
def ltOK (cc : List Opcode) (f : Flag) (Xb : Poly) (b : Rep2) (r : Poly) (Mr : ℕ) : LtEv → Bool
  | .sub e =>
    match rangeOf cc f (psub (psub Xb r) (pconst 1)) e with
    | some k => decide (2 ^ k + Mr < p)
    | none => false
  | .shift d e =>
    decide (b.L = b.M) &&
    match rangeOf cc f (r ++ pconst d) e with
    | some k => decide (2 ^ k = b.M + d) && decide (Mr + d < p)
    | none => false

def hintStep (cc : List Opcode) (s : Reps × Flag) : Instruction → HStep → Option (Reps × Flag)
  | .bin d op u a b, .arith E ia ib c rng => do
    let (ra, rb, Xa, Xb) ← operands cc s.1 a b ia ib
    let T ← arithPoly op Xa Xb
    match ra.ty with
    | .uint n =>
      let k ← rangeOf cc s.2 E rng
      if u = false ∧ 2 ≤ n ∧ k ≤ n ∧ (op = .mul → n = 128 → ra.M * rb.M < 2 ^ 128) ∧
          combHolds cc c (underFlag s.2 (psub E T)) then
        some ((d, .scalar ⟨[flagged s.2 E], .uint n, 0, 2 ^ k - 1⟩) :: s.1, s.2)
      else none
    | _ => none
  | .bin d op u a b, .divmod q r ia ib c qr rr lt => do
    let (ra, rb, Xa, Xb) ← operands cc s.1 a b ia ib
    let kq ← rangeOf cc s.2 q qr
    let kr ← rangeOf cc s.2 r rr
    match ra.ty with
    | .uint n =>
      if (op = .div ∨ op = .mod) ∧ fitsBoth ra rb ∧ rb.M * (2 ^ kq - 1) + (2 ^ kr - 1) < p ∧
          combHolds cc c (underFlag s.2 (psub (psub Xa (pmul Xb q)) r)) ∧
          ltOK cc s.2 Xb rb r (2 ^ kr - 1) lt then
        let res := if op = .div then q else r
        let M := if op = .div then 2 ^ kq - 1 else 2 ^ kr - 1
        some ((d, .scalar ⟨[flagged s.2 res], .uint n, 0, M⟩) :: s.1, s.2)
      else none
    | _ => none
  | .bin d op _ a b, .eq E z ia ib c₁ c₂ => do
    let (ra, rb, Xa, Xb) ← operands cc s.1 a b ia ib
    let D := psub Xa Xb
    if op = .eq ∧ fitsBoth ra rb ∧ combHolds cc c₁ (psub (psub (pconst 1) (pmul D z)) E) ∧
        combHolds cc c₂ (pmul D E) then
      some ((d, .scalar ⟨[E], .uint 1, 0, 1⟩) :: s.1, s.2)
    else none
  | .constrain a b _, .constrain ia ib c => do
    let (_, _, Xa, Xb) ← operands cc s.1 a b ia ib
    if combHolds cc c (psub Xa Xb) then some s else none
  | .constrainNe a b _, .constrainNe z ia ib c => do
    let (_, _, Xa, Xb) ← operands cc s.1 a b ia ib
    if combHolds cc c (underFlag s.2 (psub (pconst 1) (pmul (psub Xa Xb) z))) then some s
    else none
  | _, _ => none

/-- One instruction: its hint step if the certificate gives one, else `stepP`. -/
def stepH (cc : List Opcode) (s : Reps × Flag) (i : Instruction) : Option HStep → Option (Reps × Flag)
  | none => stepP cc s i
  | some h => hintStep cc s i h

/-- Run the body; each step over the constraints its certificate entry picks,
or as its hint step says. -/
def stepsWithH (cc : List Opcode) :
    Reps × Flag → List Instruction → List (List ℕ × Option HStep) → Option (Reps × Flag)
  | s, [], [] => some s
  | s, i :: is, (ix, h) :: ixs =>
    (match h with
      | none => stepP (pick cc ix) s i
      | some h => hintStep cc s i h).bind fun r => stepsWithH cc r is ixs
  | _, _, _ => none

/-- Return witness `w` equals scalar `r`: as `retOK` finds, or through a
combination showing `w = ` one of `r`'s polynomials. -/
def retOKH (cc : List Opcode) (w : ℕ) (r : Rep2) : Option (ℕ × Comb) → Bool
  | none => retOK cc w r
  | some (i, c) => match r.alts[i]? with
    | some X => combHolds cc c (psub (pvar w) X)
    | none => false

def retsOKH (cc : List Opcode) (reps : Reps) (ws : List ℕ) (os : List Operand)
    (hs : List (Option (ℕ × Comb))) : Bool :=
  match os.mapM (opFlat reps) with
  | none => false
  | some rss => decide (ws.length = rss.flatten.length) &&
    ((ws.zip rss.flatten).zip (hs ++ List.replicate rss.flatten.length none)).all
      fun ((w, r), h) => retOKH cc w r h

/-- `checkProgWith` with hint steps and return-value hints. -/
def checkProgH (P : Program) (C : Circuit) (cert : List (List ℕ × Option HStep))
    (rets : List (Option (ℕ × Comb))) : Bool :=
  let cc := C.opcodes
  decide (C.parameters.length = P.inputTypes.length) &&
    match initReps cc P.params C.parameters with
    | none => false
    | some reps0 =>
      match stepsWithH cc (reps0, none) P.body cert with
      | none => false
      | some (reps, _) => retsOKH cc reps C.returnValues P.rets rets

end AcirLean
