/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Checker
import AcirLean.Proofs.SignedDivMod
import AcirLean.Spec.Programs2

/-!
# The checker for scalar programs

`checkProg2 P C` decides whether circuit `C` implements the scalar SSA function
`P`. It walks the instructions, keeping for every SSA value some polynomials over
`C`'s witnesses that evaluate to it, the value's type, and bounds on its
integer value. Each instruction is accepted by a local rule whose premises
are constraints of `C` (found by untrusted searches, then checked) and static
bounds; `checkProg2_sound` proves once that acceptance implies
`SoundFn C (ProgSpec2 P)`.
-/

namespace AcirLean

/-! ## Polynomials over witnesses -/

abbrev Poly := List Term

def Poly.eval (σ : ℕ → F) (P : Poly) : F := (P.map (Term.eval σ)).sum

def pconst (c : ℤ) : Poly := [⟨c, []⟩]
def pvar (w : ℕ) : Poly := [⟨1, [w]⟩]
def pscale (c : ℤ) (P : Poly) : Poly := P.map fun t => ⟨c * t.coef, t.ws⟩
def psub (P Q : Poly) : Poly := P ++ pscale (-1) Q
def pmul (P Q : Poly) : Poly :=
  P.flatMap fun t => Q.map fun u => ⟨t.coef * u.coef, t.ws ++ u.ws⟩

/-- Add a term to a polynomial, merging it with a term over the same witnesses. -/
def addTerm (t : Term) : Poly → Poly
  | [] => [t]
  | u :: us => if u.ws = t.ws then ⟨u.coef + t.coef, u.ws⟩ :: us else u :: addTerm t us

/-- Sort each term's witnesses and merge like terms. -/
def collect : Poly → Poly
  | [] => []
  | t :: ts => addTerm ⟨t.coef, isort (fun a b => decide (a ≤ b)) t.ws⟩ (collect ts)

/-- The canonical `AssertZero` constraint `P = 0`. -/
def key (P : Poly) : Cstr := (Cstr.zero (collect P)).canon

/-- `P = 0` follows from the circuit: it is trivial, or one of its constraints. -/
def holdsZ (cc : List Cstr) (P : Poly) : Bool :=
  match key P with
  | .zero [] => true
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
def constCands (cc : List Cstr) (w : ℕ) : List (ℕ × ℤ) :=
  cc.filterMap fun c => match c with
    | .zero [⟨b, [v]⟩] => if v = w then some (0, b) else none
    | .zero [⟨a, []⟩, ⟨b, [v]⟩] =>
      if v = w then some ((p - a.toNat % p) * powMod b.toNat (p - 2) p 256 % p, b) else none
    | _ => none

def rangeBounds (cc : List Cstr) (w : ℕ) : List ℕ :=
  cc.filterMap fun c => match c with
    | .range v k => if v = w then some (2 ^ k - 1) else none
    | _ => none

/-- Bounds `(L, M)` on witness `w`'s integer value: a constant it equals, or its
tightest range check. -/
def wbound (cc : List Cstr) (w : ℕ) : Option (ℕ × ℕ) :=
  match (constCands cc w).find? (fun (c, b) => decide (c < p) && decide ((b : F) ≠ 0) &&
      holdsZ cc (pscale b (psub (pvar w) (pconst (c : ℤ))))) with
  | some (c, _) => some (c, c)
  | none => (rangeBounds cc w).min?.map fun M => (0, M)

def singles (ts : List Term) : List ℕ :=
  ts.filterMap fun t => match t.ws with
    | [w] => some w
    | _ => none

def matCands (cc : List Cstr) : List ℕ :=
  cc.flatMap fun c => match c with
    | .zero ts => singles ts
    | _ => []

def matSearch (cc : List Cstr) (P : Poly) : Option ℕ :=
  match collect P with
  | [⟨1, [w]⟩] => some w
  | _ => (matCands cc).find? fun w => holdsZ cc (psub (pvar w) P)

/-- A witness equal to `P`. -/
def matV (cc : List Cstr) (P : Poly) : Option ℕ :=
  (matSearch cc P).filter fun w => holdsZ cc (psub (pvar w) P)

/-- Bounds on `P`'s integer value: `P = 0`, or a bounded witness equals `P`. -/
def pbound (cc : List Cstr) (P : Poly) : Option (ℕ × ℕ) :=
  if holdsZ cc P then some (0, 0) else (matV cc P).bind (wbound cc)

/-- Witnesses equal to the polynomials, and the polynomials. -/
def forms (cc : List Cstr) (alts : List Poly) : List Poly :=
  ((alts.filterMap fun P => (matV cc P).map pvar) ++ alts).eraseDups

def cVars : Cstr → List ℕ
  | .zero ts => ts.flatMap (·.ws)
  | .range w _ => [w]

/-- Witnesses `q`, `r` with `E q r = 0` among the circuit's constraints. -/
def solve2 (cc : List Cstr) (E : ℕ → ℕ → Poly) : List (ℕ × ℕ) :=
  (cc.flatMap fun c => match c with
    | .zero _ =>
      let vs := (cVars c).eraseDups
      vs.flatMap fun q => vs.filterMap fun r => if key (E q r) = c then some (q, r) else none
    | _ => []).filter fun (q, r) => holdsZ cc (E q r)

/-- Up to four ways of combining two lists of polynomials. -/
def comb (f : Poly → Poly → Poly) (as bs : List Poly) : List Poly :=
  (as.flatMap fun a => bs.map (f a)).take 4

/-- `P = Q`, directly or through a witness equal to both. -/
def eqVia (cc : List Cstr) (P Q : Poly) : Bool :=
  holdsZ cc (psub P Q) ||
    (matCands cc).any fun w => holdsZ cc (psub P (pvar w)) && holdsZ cc (psub (pvar w) Q)

/-! ## The rules -/

/-- Where an SSA value lives: each polynomial in `alts` evaluates to it, it has
type `ty`, and its integer value lies in `[L, M]`. -/
structure Rep2 where
  alts : List Poly
  ty : VTy
  L : ℕ
  M : ℕ

def opRep (reps : List (ℕ × Rep2)) : Opnd → Option Rep2
  | .var id => reps.lookup id
  | .const v ty => some ⟨[pconst v], ty, v % p, v % p⟩

/-- `y` with `1 ∓ t z - y = 0` and `t y = 0`: `y` is the flag `t = 0`. -/
def zeroFlags (cc : List Cstr) (t : Poly) : List ℕ :=
  ((solve2 cc fun z y => psub (psub (pconst 1) (pmul t (pvar z))) (pvar y)) ++
    (solve2 cc fun z y => psub (pconst 1 ++ pmul t (pvar z)) (pvar y))).filterMap
    fun (_, y) => if holdsZ cc (pmul t (pvar y)) then some y else none

/-- `r < b`: a bounded witness equals `b - r - 1`, or `b` is the constant `M`
and a witness equal to `r + d` is below `M + d`. -/
def remLt (cc : List Cstr) (Xb : Poly) (b : Rep2) (r Mr : ℕ) : Bool :=
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
def euclid (cc : List Cstr) (a b : Rep2) : List (ℕ × ℕ) :=
  (forms cc a.alts).flatMap fun Xa => (forms cc b.alts).flatMap fun Xb =>
    (solve2 cc fun q r => psub (psub Xa (pmul Xb (pvar q))) (pvar r)).filter fun (q, r) =>
      match wbound cc q, wbound cc r with
      | some (_, Mq), some (_, Mr) => decide (b.M * Mq + Mr < p) && remLt cc Xb b r Mr
      | _, _ => false

/-- Witnesses `q ≤ 1`, `r < 2^m` with `2^m + a - b = 2^m q + r`: `q` is `a ≥ b`. -/
def geFlags (cc : List Cstr) (a b : Rep2) (m : ℕ) : List ℕ :=
  (forms cc a.alts).flatMap fun Xa => (forms cc b.alts).flatMap fun Xb =>
    (solve2 cc fun q r =>
      psub (psub (pconst (2 ^ m) ++ psub Xa Xb) (pscale (2 ^ m) (pvar q))) (pvar r)).filterMap
      fun (q, r) => match wbound cc q, wbound cc r with
        | some (_, Mq), some (_, Mr) => if Mq ≤ 1 ∧ Mr < 2 ^ m then some q else none
        | _, _ => none

/-- `eq`: the flag `a - b = 0`. -/
def eqFlags (cc : List Cstr) (a b : Rep2) : List ℕ :=
  (forms cc a.alts).flatMap fun Xa => (forms cc b.alts).flatMap fun Xb =>
    zeroFlags cc (psub Xa Xb)

/-- A bounded witness equal to one of the polynomials, below `2^n`. -/
def checked (cc : List Cstr) (alts : List Poly) (n : ℕ) : Option (ℕ × ℕ) :=
  alts.findSome? fun P => (pbound cc P).filter fun (_, M) => M < 2 ^ n

def binRep (cc : List Cstr) (op : BOp) (a b : Rep2) : Option Rep2 :=
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
    if sols.isEmpty then none else some ⟨sols.map (pvar ·.1), .uint n, 0, a.M⟩
  | .mod, .uint n =>
    let sols := euclid cc a b
    if sols.isEmpty then none else some ⟨sols.map (pvar ·.2), .uint n, 0, min a.M (b.M - 1)⟩
  | .lt, .uint m =>
    if a.M < 2 ^ m ∧ b.M < 2 ^ m ∧ 2 ^ (m + 1) < p then
      some ⟨(geFlags cc a b m).map fun q => psub (pconst 1) (pvar q), .uint 1, 0, 1⟩
    else none
  | _, _ => none

/-- `r ≤ Mr < 2^k`, `x = 2^k q + r`, and `2^k q + r < p`: either from the bound
on `q`, or, for `q ≤ p / 2^k`, from a flag `y = [q = p / 2^k]` with `(r + d) y`
below `d + p % 2^k`. -/
def truncOK (cc : List Cstr) (k q r : ℕ) : Bool :=
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

def truncRep (cc : List Cstr) (a : Rep2) (k : ℕ) : Rep2 :=
  let same := if a.M < 2 ^ k then a.alts else []
  let rs := (forms cc a.alts).flatMap fun Xa =>
    ((solve2 cc fun q r => psub (psub Xa (pscale (2 ^ k) (pvar q))) (pvar r)).filter
      fun (q, r) => truncOK cc k q r).map (pvar ·.2)
  ⟨same ++ rs, a.ty, 0, min a.M (2 ^ k - 1)⟩

def castOK (ty : VTy) (M : ℕ) : Bool :=
  match ty with
  | .field => true
  | .uint m => decide (M < 2 ^ m)
  | .sint m => decide (M < 2 ^ m)

def eqHolds (cc : List Cstr) (a b : Rep2) : Bool :=
  (forms cc a.alts).any fun Xa => (forms cc b.alts).any fun Xb => eqVia cc Xa Xb

def rangeHolds (cc : List Cstr) (a : Rep2) (k : ℕ) : Bool :=
  decide (a.M < 2 ^ k) || (checked cc a.alts k).isSome

def step2 (cc : List Cstr) (reps : List (ℕ × Rep2)) : Ins → Option (List (ℕ × Rep2))
  | .bin d op _ a b => do
    let ra ← opRep reps a
    let rb ← opRep reps b
    let r ← binRep cc op ra rb
    some ((d, r) :: reps)
  | .not d a => do
    let ra ← opRep reps a
    match ra.ty with
    | .uint n =>
      if ra.M < 2 ^ n ∧ 2 ^ n ≤ p then
        some ((d, ⟨ra.alts.map (psub (pconst (2 ^ n - 1 : ℕ))), .uint n,
          2 ^ n - 1 - ra.M, 2 ^ n - 1 - ra.L⟩) :: reps)
      else none
    | _ => none
  | .cast d a ty => do
    let ra ← opRep reps a
    if castOK ty ra.M then some ((d, ⟨ra.alts, ty, ra.L, ra.M⟩) :: reps) else none
  | .truncate d a k _ => do
    let ra ← opRep reps a
    some ((d, truncRep cc ra k) :: reps)
  | .constrain a b _ => do
    let ra ← opRep reps a
    let rb ← opRep reps b
    if eqHolds cc ra rb then some reps else none
  | .rangeCheck a k _ => do
    let ra ← opRep reps a
    if rangeHolds cc ra k then some reps else none

def paramRep (cc : List Cstr) (w : ℕ) : VTy → Option Rep2
  | .field => some ⟨[pvar w], .field, 0, p - 1⟩
  | .uint n => (wbound cc w).bind fun (L, M) =>
    if M < 2 ^ n then some ⟨[pvar w], .uint n, L, M⟩ else none
  | .sint n => (wbound cc w).bind fun (L, M) =>
    if M < 2 ^ n then some ⟨[pvar w], .sint n, L, M⟩ else none

def initReps (cc : List Cstr) : List (ℕ × VTy) → List ℕ → Option (List (ℕ × Rep2))
  | (id, ty) :: ps, w :: ws => do
    let r ← paramRep cc w ty
    let rest ← initReps cc ps ws
    some ((id, r) :: rest)
  | _, _ => some []

def retOK (cc : List Cstr) (reps : List (ℕ × Rep2)) (r : ℕ) (o : Opnd) : Bool :=
  match opRep reps o with
  | some ro => (forms cc ro.alts).any fun X => eqVia cc (pvar r) X
  | none => false

def checkProg2 (P : Prog2) (C : AcirFn) : Bool :=
  let cc := C.cs.map Cstr.canon
  decide (C.inputs.length = P.params.length) && decide (C.returns.length = P.rets.length) &&
    match initReps cc P.params C.inputs with
    | none => false
    | some reps0 =>
      match P.body.foldlM (step2 cc) reps0 with
      | none => false
      | some reps => (C.returns.zip P.rets).all fun (r, o) => retOK cc reps r o

end AcirLean
