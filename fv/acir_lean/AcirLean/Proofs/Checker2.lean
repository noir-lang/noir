/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Checker
import AcirLean.Proofs.SignedDivMod
import AcirLean.Spec.SsaSemantics

/-!
# The checker for scalar programs

`checkProg2 P C` decides whether circuit `C` implements the scalar SSA function
`P`. It walks the instructions, keeping for every SSA value some polynomials over
`C`'s witnesses that evaluate to it, the value's type, and bounds on its
integer value. Each instruction is accepted by a local rule whose premises
are constraints of `C` (found by untrusted searches, then checked) and static
bounds; `checkProg2_sound` proves once that acceptance implies
`SoundFunction C (ProgramSpec P)`.
-/

namespace AcirLean

/-! ## Polynomials over witnesses -/

abbrev Poly := List Term

def Poly.eval (σ : ℕ → F) (P : Poly) : F := (P.map (Term.eval σ)).sum

def pconst (c : ℤ) : Poly := [⟨c, []⟩]
def pvar (w : ℕ) : Poly := [⟨1, [w]⟩]
def pscale (c : ℤ) (P : Poly) : Poly := P.map fun t => ⟨c * t.coef, t.witnesses⟩
def psub (P Q : Poly) : Poly := P ++ pscale (-1) Q
def pmul (P Q : Poly) : Poly :=
  P.flatMap fun t => Q.map fun u => ⟨t.coef * u.coef, t.witnesses ++ u.witnesses⟩

/-- Add a term to a polynomial, merging it with a term over the same witnesses. -/
def addTerm (t : Term) : Poly → Poly
  | [] => [t]
  | u :: us => if u.witnesses = t.witnesses then ⟨u.coef + t.coef, u.witnesses⟩ :: us else u :: addTerm t us

/-- Sort each term's witnesses and merge like terms. -/
def collect : Poly → Poly
  | [] => []
  | t :: ts => addTerm ⟨t.coef, isort (fun a b => decide (a ≤ b)) t.witnesses⟩ (collect ts)

/-- The canonical `AssertZero` constraint `P = 0`. -/
def key (P : Poly) : Opcode := (Opcode.assertZero (collect P)).canon

/-- `P = 0` follows from the circuit: it is trivial, or one of its constraints. -/
def holdsZ (cc : List Opcode) (P : Poly) : Bool :=
  match key P with
  | .assertZero [] => true
  | c => decide (c ∈ cc)

/-! ## Untrusted searches, each followed by a check -/

/-- `b^e mod m`, by structural recursion on `fuel` bits of `e`. -/
def powMod (b e m : ℕ) : ℕ → ℕ
  | 0 => 1 % m
  | fuel + 1 =>
    let h := powMod b (e / 2) m fuel
    if e % 2 = 0 then h * h % m else h * h % m * b % m

/-- Candidate constants `c` for witness `w`, with the coefficient `b` of a
constraint `b (w - c) = 0`. -/
def constCands (cc : List Opcode) (w : ℕ) : List (ℕ × ℤ) :=
  cc.filterMap fun c => match c with
    | .assertZero [⟨b, [v]⟩] => if v = w then some (0, b) else none
    | .assertZero [⟨a, []⟩, ⟨b, [v]⟩] =>
      if v = w then some ((p - a.toNat % p) * powMod b.toNat (p - 2) p 256 % p, b) else none
    | _ => none

def rangeBounds (cc : List Opcode) (w : ℕ) : List ℕ :=
  cc.filterMap fun c => match c with
    | .range v k => if v = w then some (2 ^ k - 1) else none
    | _ => none

/-- Bounds `(L, M)` on witness `w`'s integer value: a constant it equals, or its
tightest range check. -/
def wbound (cc : List Opcode) (w : ℕ) : Option (ℕ × ℕ) :=
  match (constCands cc w).find? (fun (c, b) => decide (c < p) && decide ((b : F) ≠ 0) &&
      holdsZ cc (pscale b (psub (pvar w) (pconst (c : ℤ))))) with
  | some (c, _) => some (c, c)
  | none => (rangeBounds cc w).min?.map fun M => (0, M)

def singles (ts : List Term) : List ℕ :=
  ts.filterMap fun t => match t.witnesses with
    | [w] => some w
    | _ => none

def matCands (cc : List Opcode) : List ℕ :=
  cc.flatMap fun c => match c with
    | .assertZero ts => singles ts
    | _ => []

/-- Witnesses `w` whose constraint `w - P = 0` is in the circuit. Such a
constraint has about one term more than `P`, which rules most constraints out
before any `key` is computed. `matSearch` still picks the first of them in
`matCands` order, since later rules may depend on which witness it picks. -/
def eqCands (cc : List Opcode) (P : Poly) : List ℕ :=
  let n := match key P with
    | .assertZero ts => ts.length
    | _ => 0
  cc.flatMap fun c => match c with
    | .assertZero ts =>
      if n ≤ ts.length + 1 ∧ ts.length ≤ n + 1 then
        (singles ts).filter fun w => decide (key (psub (pvar w) P) = c)
      else []
    | _ => []

def matSearch (cc : List Opcode) (P : Poly) : Option ℕ :=
  match (collect P).filter (fun t => modP t.coef != 0) with
  | [⟨1, [w]⟩] => some w
  | _ =>
    let found := eqCands cc P
    (matCands cc).find? (· ∈ found)

/-- A witness equal to `P`. -/
def matV (cc : List Opcode) (P : Poly) : Option ℕ :=
  (matSearch cc P).filter fun w => holdsZ cc (psub (pvar w) P)

/-- The constant `P` is, if its non-constant terms cancel out. -/
def constPoly (P : Poly) : Option ℕ :=
  match (collect P).filter (fun t => modP t.coef != 0) with
  | [] => some 0
  | [⟨c, []⟩] => some (modP c).toNat
  | _ => none

/-- Bounds on `P`'s integer value: `P = 0`, `P` is a constant, or a bounded
witness equals `P`. -/
def pbound (cc : List Opcode) (P : Poly) : Option (ℕ × ℕ) :=
  if holdsZ cc P then some (0, 0) else
  match constPoly P with
  | some c =>
    if c < p ∧ holdsZ cc (psub P (pconst c)) then some (c, c) else (matV cc P).bind (wbound cc)
  | none => (matV cc P).bind (wbound cc)

/-- Witnesses equal to the polynomials, and the polynomials. -/
def forms (cc : List Opcode) (alts : List Poly) : List Poly :=
  ((alts.filterMap fun P => (matV cc P).map pvar) ++ alts).eraseDups

def cVars : Opcode → List ℕ
  | .assertZero ts => ts.flatMap (·.witnesses)
  | .range w _ => [w]

/-- Witnesses `q`, `r` with `E q r = 0` among the circuit's constraints. -/
def solve2 (cc : List Opcode) (E : ℕ → ℕ → Poly) : List (ℕ × ℕ) :=
  let n := match key (E 1000000007 1000000009) with
    | .assertZero ts => ts.length
    | _ => 0
  (cc.flatMap fun c => match c with
    | .assertZero ts =>
      if ts.length ≤ n ∧ n ≤ ts.length + 2 then
        let vs := (cVars c).eraseDups
        vs.flatMap fun q => vs.filterMap fun r => if key (E q r) = c then some (q, r) else none
      else []
    | _ => []).filter fun (q, r) => holdsZ cc (E q r)

/-- Up to four ways of combining two lists of polynomials. -/
def comb (f : Poly → Poly → Poly) (as bs : List Poly) : List Poly :=
  (as.flatMap fun a => bs.map (f a)).take 4

/-- `P = Q`, directly or through a witness equal to both. -/
def eqVia (cc : List Opcode) (P Q : Poly) : Bool :=
  holdsZ cc (psub P Q) ||
    (eqCands cc P).any fun w => holdsZ cc (psub P (pvar w)) && holdsZ cc (psub (pvar w) Q)

/-! ## The rules -/

/-- Where an SSA value lives: each polynomial in `alts` evaluates to it, it has
type `ty`, and its integer value lies in `[L, M]`. -/
structure Rep2 where
  alts : List Poly
  ty : ValueType
  L : ℕ
  M : ℕ
  deriving DecidableEq

/-- Where each SSA value lives: a scalar, or an array's scalars in flat order. -/
inductive RVal where
  | scalar (r : Rep2)
  | array (rs : List Rep2)
  deriving DecidableEq

abbrev Reps := List (ℕ × RVal)

def constRep (v : ℤ) (ty : ValueType) : Rep2 :=
  let c := (constVal ty v).val
  ⟨[pconst c], ty, c, c⟩

def opRep (reps : Reps) : Operand → Option Rep2
  | .var id => match reps.lookup id with
    | some (.scalar r) => some r
    | _ => none
  | .const v ty => some (constRep v ty)

def opArr (reps : Reps) : Operand → Option (List Rep2)
  | .var id => match reps.lookup id with
    | some (.array rs) => some rs
    | _ => none
  | .const _ _ => none

/-- An operand's scalars, in flat order. -/
def opFlat (reps : Reps) : Operand → Option (List Rep2)
  | .var id => match reps.lookup id with
    | some (.scalar r) => some [r]
    | some (.array rs) => some rs
    | none => none
  | .const v ty => some [constRep v ty]

/-- A constant `u32` index below `len`. -/
def constIdx (len : ℕ) : Operand → Option ℕ
  | .const c (.uint 32) => if 0 ≤ c ∧ c < len ∧ c < p then some c.toNat else none
  | _ => none

/-- `y` with `1 ∓ t z - y = 0` and `t y = 0`: `y` is the flag `t = 0`. -/
def zeroFlags (cc : List Opcode) (t : Poly) : List ℕ :=
  ((solve2 cc fun z y => psub (psub (pconst 1) (pmul t (pvar z))) (pvar y)) ++
    (solve2 cc fun z y => psub (pconst 1 ++ pmul t (pvar z)) (pvar y))).filterMap
    fun (_, y) => if holdsZ cc (pmul t (pvar y)) then some y else none

/-- `r < b`: a bounded witness equals `b - r - 1`, or `b` is the constant `M`
and a witness equal to `r + d` is below `M + d`. -/
def remLt (cc : List Opcode) (Xb : Poly) (b : Rep2) (r Mr : ℕ) : Bool :=
  (match pbound cc (psub (psub Xb (pvar r)) (pconst 1)) with
    | some (_, M') => decide (M' + Mr + 1 < p)
    | none => false) ||
  (decide (b.L = b.M) &&
    let d := 2 ^ Nat.size (b.M - 1) - b.M
    match pbound cc (pvar r ++ pconst d) with
    | some (_, M') => decide (M' < b.M + d) && decide (Mr + d < p)
    | none => false)

/-- Witnesses `q`, `r` with `a = b q + r`, `r < b` and no wraparound: the
quotient and remainder of `a`'s integer value by `b`'s. -/
def euclid (cc : List Opcode) (a b : Rep2) : List (ℕ × ℕ) :=
  (forms cc a.alts).flatMap fun Xa => (forms cc b.alts).flatMap fun Xb =>
    (solve2 cc fun q r => psub (psub Xa (pmul Xb (pvar q))) (pvar r)).filter fun (q, r) =>
      match wbound cc q, wbound cc r with
      | some (_, Mq), some (_, Mr) => decide (b.M * Mq + Mr < p) && remLt cc Xb b r Mr
      | _, _ => false

/-- Witnesses `q ≤ 1`, `r < 2^m` with `2^m + a - b = 2^m q + r`: `q` is `a ≥ b`. -/
def geFlags (cc : List Opcode) (a b : Rep2) (m : ℕ) : List ℕ :=
  (forms cc a.alts).flatMap fun Xa => (forms cc b.alts).flatMap fun Xb =>
    (solve2 cc fun q r =>
      psub (psub (pconst (2 ^ m) ++ psub Xa Xb) (pscale (2 ^ m) (pvar q))) (pvar r)).filterMap
      fun (q, r) => match wbound cc q, wbound cc r with
        | some (_, Mq), some (_, Mr) => if Mq ≤ 1 ∧ Mr < 2 ^ m then some q else none
        | _, _ => none

/-- `eq`: the flag `a - b = 0`. -/
def eqFlags (cc : List Opcode) (a b : Rep2) : List ℕ :=
  (forms cc a.alts).flatMap fun Xa => (forms cc b.alts).flatMap fun Xb =>
    (forms cc [psub Xa Xb]).flatMap (zeroFlags cc)

/-- A bounded witness equal to one of the polynomials, below `2^n`. -/
def checked (cc : List Opcode) (alts : List Poly) (n : ℕ) : Option (ℕ × ℕ) :=
  alts.findSome? fun P => (pbound cc P).filter fun (_, M) => M < 2 ^ n

/-- Every binary instruction other than unchecked integer arithmetic, on
operands that fit their type. -/
def checkedRep (cc : List Opcode) (op : BinaryOp) (a b : Rep2) : Option Rep2 :=
  let comb f as bs := comb f (forms cc as) (forms cc bs)
  match op, a.ty with
  | .eq, _ => some ⟨(eqFlags cc a b).map pvar, .uint 1, 0, 1⟩
  | .add, .field => some ⟨comb (· ++ ·) a.alts b.alts, .field, 0, p - 1⟩
  | .sub, .field => some ⟨comb psub a.alts b.alts, .field, 0, p - 1⟩
  | .mul, .field => some ⟨comb pmul a.alts b.alts, .field, 0, p - 1⟩
  | .add, .uint n =>
    let alts := comb (· ++ ·) a.alts b.alts
    if a.M + b.M < 2 ^ n ∧ a.M + b.M < p then some ⟨alts, .uint n, a.L + b.L, a.M + b.M⟩
    else if a.M + b.M < p then (checked cc alts n).map fun (L, M) => ⟨alts, .uint n, L, M⟩
    else none
  | .mul, .uint n =>
    let alts := comb pmul a.alts b.alts
    if a.M * b.M < 2 ^ n ∧ a.M * b.M < p then some ⟨alts, .uint n, a.L * b.L, a.M * b.M⟩
    else if a.M * b.M < p then (checked cc alts n).map fun (L, M) => ⟨alts, .uint n, L, M⟩
    else none
  | .sub, .uint n =>
    let alts := comb psub a.alts b.alts
    if b.M ≤ a.L then some ⟨alts, .uint n, a.L - b.M, a.M - b.L⟩
    else (checked cc alts n).bind fun (L, M) =>
      if M + b.M < p then some ⟨alts, .uint n, L, M⟩ else none
  | .div, .uint n =>
    let sols := euclid cc a b
    if sols.isEmpty then none else some ⟨sols.map (pvar ·.1), .uint n, 0, a.M / max b.L 1⟩
  | .mod, .uint n =>
    let sols := euclid cc a b
    if sols.isEmpty then none else some ⟨sols.map (pvar ·.2), .uint n, 0, min a.M (b.M - 1)⟩
  | .lt, .uint m =>
    if a.M < 2 ^ m ∧ b.M < 2 ^ m ∧ 2 ^ (m + 1) < p then
      some ⟨(geFlags cc a b m).map fun q => psub (pconst 1) (pvar q), .uint 1, 0, 1⟩
    else none
  | _, _ => none

/-- Unchecked `add`, `sub` or `mul` on integers: field arithmetic, with bounds
that hold when it cannot wrap around `p`. -/
def uncheckedRep (cc : List Opcode) (op : BinaryOp) (a b : Rep2) : Option Rep2 :=
  let comb f as bs := comb f (forms cc as) (forms cc bs)
  match op with
  | .add =>
    let alts := comb (· ++ ·) a.alts b.alts
    some (if a.M + b.M < p then ⟨alts, a.ty, a.L + b.L, a.M + b.M⟩ else ⟨alts, a.ty, 0, p - 1⟩)
  | .sub =>
    let alts := comb psub a.alts b.alts
    some (if b.M ≤ a.L then ⟨alts, a.ty, a.L - b.M, a.M - b.L⟩ else ⟨alts, a.ty, 0, p - 1⟩)
  | .mul =>
    let alts := comb pmul a.alts b.alts
    some (if a.M * b.M < p then ⟨alts, a.ty, a.L * b.L, a.M * b.M⟩ else ⟨alts, a.ty, 0, p - 1⟩)
  | _ => none

def isArith : BinaryOp → Bool
  | .add | .sub | .mul => true
  | _ => false

/-- Both operands fit their integer type. -/
def fitsBoth (a b : Rep2) : Bool :=
  match a.ty with
  | .field => true
  | .uint n => decide (a.M < 2 ^ n ∧ b.M < 2 ^ n)
  | .sint n => decide (a.M < 2 ^ n ∧ b.M < 2 ^ n)

/-- The constant terms of the constraints, with both signs. -/
def constTerms (cc : List Opcode) : List ℕ :=
  cc.flatMap fun c => match c with
    | .assertZero ts => ts.flatMap fun t =>
      if t.witnesses.isEmpty then [(modP t.coef).toNat, (modP (-t.coef)).toNat] else []
    | .range _ _ => []

/-- A checked `add` on `u<n>` whose operands' bounds are too loose to show they
fit: the field sum, which the circuit must show is below `2^n`, through a
bounded witness or a constraint fixing it to a constant. -/
def addRep (cc : List Opcode) (n : ℕ) (a b : Rep2) : Option Rep2 :=
  let alts := comb (· ++ ·) (forms cc a.alts) (forms cc b.alts)
  match checked cc alts n with
  | some (L, M) => some ⟨alts, .uint n, L, M⟩
  | none =>
    (alts.findSome? fun P => (constTerms cc).find? fun c =>
      decide (c < 2 ^ n) && decide (c < p) && holdsZ cc (psub P (pconst c))).map
      fun c => ⟨alts, .uint n, c, c⟩

/-- `xor` on `u1`: `x + y - 2 x y`. -/
def xorRep (cc : List Opcode) (a b : Rep2) : Rep2 :=
  ⟨comb (fun A B => psub (A ++ B) (pscale 2 (pmul A B))) (forms cc a.alts) (forms cc b.alts),
    .uint 1, 0, 1⟩

/-- `u1` arithmetic is boolean: unchecked `add` is `xor`, which is the sum when
the sum is below `2`; unchecked `sub` and `mul` mean the same as checked. -/
def binRep (cc : List Opcode) (op : BinaryOp) (u : Bool) (a b : Rep2) : Option Rep2 :=
  match a.ty, u && isArith op with
  | .uint 1, true =>
    if ¬fitsBoth a b then none
    else if op = .add then
      let alts := comb (· ++ ·) (forms cc a.alts) (forms cc b.alts)
      (checked cc alts 1).map fun (L, M) => ⟨alts, .uint 1, L, M⟩
    else checkedRep cc op a b
  | .uint _, true | .sint _, true => uncheckedRep cc op a b
  | _, _ =>
    if op = .xor then (if a.ty = .uint 1 ∧ fitsBoth a b then some (xorRep cc a b) else none)
    else if fitsBoth a b then checkedRep cc op a b
    else match a.ty, op with
      | .uint n, .add => if n = 1 then none else addRep cc n a b
      | _, _ => none

/-- `r ≤ Mr < 2^k`, `x = 2^k q + r`, and `2^k q + r < p`: either from the bound
on `q`, or, for `q ≤ p / 2^k`, from a flag `y = [q = p / 2^k]` with `(r + d) y`
below `d + p % 2^k`. -/
def truncOK (cc : List Opcode) (k q r : ℕ) : Bool :=
  match wbound cc q, wbound cc r with
  | some (_, Mq), some (_, Mr) =>
    decide (0 < k) && decide (2 ^ k < p) && decide (Mr < 2 ^ k) &&
      (decide (2 ^ k * Mq + Mr < p) ||
        (match pbound cc (psub (pconst (p / 2 ^ k : ℕ)) (pvar q)) with
          | some (_, Mt) => decide (Mt + Mq < p)
          | none => false) &&
        let d := 2 ^ Nat.size (p % 2 ^ k - 1) - p % 2 ^ k
        (zeroFlags cc (psub (pvar q) (pconst (p / 2 ^ k : ℕ)))).any fun y =>
          match pbound cc (pmul (pvar r ++ pconst d) (pvar y)) with
          | some (_, Mu) => decide (Mu < d + p % 2 ^ k) && decide (Mr + d < p)
          | none => false)
  | _, _ => false

def truncRep (cc : List Opcode) (a : Rep2) (k : ℕ) : Rep2 :=
  let same := if a.M < 2 ^ k then a.alts else []
  let rs := (forms cc a.alts).flatMap fun Xa =>
    ((solve2 cc fun q r => psub (psub Xa (pscale (2 ^ k) (pvar q))) (pvar r)).filter
      fun (q, r) => truncOK cc k q r).map (pvar ·.2)
  ⟨same ++ rs, a.ty, 0, min a.M (2 ^ k - 1)⟩

def eqHolds (cc : List Opcode) (a b : Rep2) : Bool :=
  (forms cc a.alts).any fun Xa => (forms cc b.alts).any fun Xb => eqVia cc Xa Xb

/-- `a ≠ b`: some witness `z` has `(a - b) z = ±1`, directly or through a
witness equal to `a - b`. -/
def neHolds (cc : List Opcode) (a b : Rep2) : Bool :=
  (forms cc a.alts).any fun Xa => (forms cc b.alts).any fun Xb =>
    (forms cc [psub Xa Xb]).any fun D => (cc.flatMap cVars).any fun z =>
      holdsZ cc (psub (pconst 1) (pmul D (pvar z))) || holdsZ cc (pconst 1 ++ pmul D (pvar z))

def rangeHolds (cc : List Opcode) (a : Rep2) (k : ℕ) : Bool :=
  decide (a.M < 2 ^ k) || (checked cc a.alts k).isSome

def step2 (cc : List Opcode) (reps : Reps) : Instruction → Option Reps
  | .bin d op u a b => do
    let ra ← opRep reps a
    let rb ← opRep reps b
    let r ← binRep cc op u ra rb
    some ((d, .scalar r) :: reps)
  | .not d a => do
    let ra ← opRep reps a
    match ra.ty with
    | .uint n =>
      if ra.M < 2 ^ n ∧ 2 ^ n ≤ p then
        some ((d, .scalar ⟨ra.alts.map (psub (pconst (2 ^ n - 1 : ℕ))), .uint n,
          2 ^ n - 1 - ra.M, 2 ^ n - 1 - ra.L⟩) :: reps)
      else none
    | _ => none
  | .cast d a ty => do
    let ra ← opRep reps a
    some ((d, .scalar ⟨ra.alts, ty, ra.L, ra.M⟩) :: reps)
  | .truncate d a k _ => do
    let ra ← opRep reps a
    if 0 < k ∧ (ra.ty = .uint 1 → ra.M < 2) then some ((d, .scalar (truncRep cc ra k)) :: reps)
    else none
  | .constrain a b _ => do
    let ra ← opRep reps a
    let rb ← opRep reps b
    if eqHolds cc ra rb then some reps else none
  | .constrainNe a b _ => do
    let ra ← opRep reps a
    let rb ← opRep reps b
    if neHolds cc ra rb then some reps else none
  | .rangeCheck a k _ => do
    let ra ← opRep reps a
    if 0 < k ∧ (ra.ty = .uint 1 → ra.M < 2) ∧ rangeHolds cc ra k then some reps else none
  | .arrayGet d a i _ => do
    let rs ← opArr reps a
    let j ← constIdx rs.length i
    let r ← rs[j]?
    some ((d, .scalar r) :: reps)
  | .arraySet d _ a i v => do
    let rs ← opArr reps a
    let j ← constIdx rs.length i
    let r ← opRep reps v
    some ((d, .array (rs.set j r)) :: reps)
  | .makeArray d es _ => do
    let rs ← es.mapM (opRep reps)
    some ((d, .array rs) :: reps)

def paramRep (cc : List Opcode) (w : ℕ) : ValueType → Option Rep2
  | .field => some ⟨[pvar w], .field, 0, p - 1⟩
  | .uint n => (wbound cc w).bind fun (L, M) =>
    if M < 2 ^ n then some ⟨[pvar w], .uint n, L, M⟩ else none
  | .sint n => (wbound cc w).bind fun (L, M) =>
    if M < 2 ^ n then some ⟨[pvar w], .sint n, L, M⟩ else none

/-- Each parameter's scalars take the next input witnesses, in order. -/
def initReps (cc : List Opcode) : List (ℕ × ParamType) → List ℕ → Option Reps
  | [], _ => some []
  | (id, t) :: ps, ws => do
    let rs ← (t.flat.zip ws).mapM fun (ty, w) => paramRep cc w ty
    let rest ← initReps cc ps (ws.drop t.flat.length)
    match t, rs with
    | .scalar _, [r] => some ((id, .scalar r) :: rest)
    | .array _ _, rs => some ((id, .array rs) :: rest)
    | _, _ => none

/-- Return witness `w` equals scalar `r`. -/
def retOK (cc : List Opcode) (w : ℕ) (r : Rep2) : Bool :=
  (forms cc r.alts).any fun X => eqVia cc (pvar w) X

/-- The return witnesses are the returned values' scalars, in order. -/
def retsOK (cc : List Opcode) (reps : Reps) (ws : List ℕ) (os : List Operand) : Bool :=
  match os.mapM (opFlat reps) with
  | none => false
  | some rss => decide (ws.length = rss.flatten.length) &&
    (ws.zip rss.flatten).all fun (w, r) => retOK cc w r

def checkProg2 (P : Program) (C : Circuit) : Bool :=
  let cc := C.opcodes
  decide (C.parameters.length = P.inputTypes.length) &&
    match initReps cc P.params C.parameters with
    | none => false
    | some reps0 =>
      match P.body.foldlM (step2 cc) reps0 with
      | none => false
      | some reps => retsOK cc reps C.returnValues P.rets

/-! ## Certificates

`checkProg2` searches the whole circuit at every step, which is fast when
compiled but slow in the kernel on large circuits. A certificate lists, for
each instruction, the positions of the few constraints its step needs; the
step then runs over those constraints only. Any sub-list of the circuit's
constraints holds whenever the circuit does, so a wrong certificate can only
make a step fail. -/

/-- The constraints at the given positions. -/
def pick (cc : List Opcode) (idx : List ℕ) : List Opcode := idx.filterMap (cc[·]?)

/-- Run the body, each step over the constraints its certificate entry picks. -/
def stepsWith (cc : List Opcode) : Reps → List Instruction → List (List ℕ) → Option Reps
  | reps, [], [] => some reps
  | reps, i :: is, ix :: ixs => (step2 (pick cc ix) reps i).bind fun r => stepsWith cc r is ixs
  | _, _, _ => none

/-- `checkProg2`, with a certificate for the body. -/
def checkProgWith (P : Program) (C : Circuit) (cert : List (List ℕ)) : Bool :=
  let cc := C.opcodes
  decide (C.parameters.length = P.inputTypes.length) &&
    match initReps cc P.params C.parameters with
    | none => false
    | some reps0 =>
      match stepsWith cc reps0 P.body cert with
      | none => false
      | some reps => retsOK cc reps C.returnValues P.rets

end AcirLean
