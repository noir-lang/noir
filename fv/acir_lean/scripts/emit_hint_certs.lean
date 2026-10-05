/-
Not trusted. Writes `AcirLean/Proofs/TestProgramCerts.lean`: for each test
program and each instruction, either the positions of the few constraints
`stepP` needs (as `emit_certs.lean` always did), or, where `stepP` fails, a hint
step built from the polynomials the compiler assigned (`hints.txt`, printed by
the `dump_hints` test) and combinations of constraints found by Gaussian
elimination. The kernel checks every combination, so nothing here is trusted.

Usage: lake env lean --run scripts/emit_hint_certs.lean <hints.txt> <out.lean> [name...]
With names, writes only those programs' definitions and logs each step.
-/

import AcirLean.Proofs.HintChecker
import AcirLean.Templates.TestPrograms

open AcirLean

instance : Inhabited Gen := ⟨⟨.flagBit, [], 0⟩⟩

/-! ## Polynomials mod p, for the search -/

abbrev NPoly := Std.HashMap (List ℕ) ℕ

def toN (P : Poly) : NPoly :=
  P.foldl (init := {}) fun m t =>
    let k := isort (fun a b => decide (a ≤ b)) t.witnesses
    let v := ((m.getD k 0) + (modP t.coef).toNat) % p
    if v = 0 then m.erase k else m.insert k v

def lexLt : List ℕ → List ℕ → Bool
  | [], [] => false
  | [], _ :: _ => true
  | _ :: _, [] => false
  | a :: as, b :: bs => if a < b then true else if b < a then false else lexLt as bs

/-- The largest monomial (by length, then lexicographically). -/
def lead (m : NPoly) : Option (List ℕ) :=
  m.fold (init := none) fun acc k _ => match acc with
    | none => some k
    | some k' => if k.length > k'.length ∨ (k.length = k'.length ∧ lexLt k' k) then some k else some k'

def axpy (m : NPoly) (c : ℕ) (r : NPoly) : NPoly :=
  r.fold (init := m) fun acc k v =>
    let nv := ((acc.getD k 0) + c * v) % p
    if nv = 0 then acc.erase k else acc.insert k nv

def invP (a : ℕ) : ℕ := powMod a (p - 2) p 256

structure Row where
  poly : NPoly
  comb : Std.HashMap ℕ ℕ   -- generator index → coefficient

/-- Echelon form: pivot monomial → row. -/
abbrev Basis := Std.HashMap (List ℕ) Row

partial def reduce (B : Basis) (r : Row) : Row :=
  match lead r.poly with
  | none => r
  | some k => match B.get? k with
    | none => r
    | some row =>
      let f := r.poly.getD k 0 * invP (row.poly.getD k 0) % p
      let neg := (p - f) % p
      reduce B ⟨axpy r.poly neg row.poly,
        row.comb.fold (init := r.comb) fun acc g c =>
          let nv := ((acc.getD g 0) + neg * c) % p
          if nv = 0 then acc.erase g else acc.insert g nv⟩

def addRow (B : Basis) (r : Row) : Basis :=
  let r := reduce B r
  match lead r.poly with
  | none => B
  | some k => B.insert k r

/-- Constraint generators: every `AssertZero` and 1-bit range check among
`idxs`, times each of the multipliers. -/
def gensAt (cc : List Opcode) (idxs : List ℕ) (muls : List (List ℕ)) : Array Gen := Id.run do
  let mut out := #[]
  for i in idxs do
    match cc[i]? with
    | some (.assertZero _) | some (.range _ 1) => for m in muls do out := out.push ⟨.con i, m, 1⟩
    | _ => pure ()
  return out

def gens (cc : List Opcode) (muls : List (List ℕ)) : Array Gen :=
  gensAt cc (List.range cc.length) muls

/-- A combination of `gs` equal to `T`, if there is one. -/
def express (cc : List Opcode) (s : Reps × Flag) (gs : Array Gen) (T : Poly) : Option Comb := Id.run do
  let mut B : Basis := {}
  for (g, i) in gs.toList.zipIdx do
    B := addRow B ⟨toN (genPoly cc s g), ({} : Std.HashMap ℕ ℕ).insert i 1⟩
  let r := reduce B ⟨toN T, {}⟩
  if r.poly.isEmpty then
    -- T - Σ c_g g = 0 with r.comb = -c
    some (r.comb.fold (init := []) fun acc i c =>
      let g := gs[i]!
      ⟨g.src, g.mul, ((p - c) % p : ℕ)⟩ :: acc)
  else none

/-- A constant `c` and a combination of `gs` equal to `T - c · M`, if any. -/
def expressC (cc : List Opcode) (s : Reps × Flag) (gs : Array Gen) (M T : Poly) : Option (ℕ × Comb) := Id.run do
  let mut B : Basis := {}
  for (g, i) in gs.toList.zipIdx do
    B := addRow B ⟨toN (genPoly cc s g), ({} : Std.HashMap ℕ ℕ).insert i 1⟩
  B := addRow B ⟨toN M, ({} : Std.HashMap ℕ ℕ).insert gs.size 1⟩
  let r := reduce B ⟨toN T, {}⟩
  if r.poly.isEmpty then
    let c := (p - r.comb.getD gs.size 0) % p
    some (c, (r.comb.erase gs.size).fold (init := []) fun acc i c =>
      let g := gs[i]!
      ⟨g.src, g.mul, ((p - c) % p : ℕ)⟩ :: acc)
  else none

def witnessesOf (P : Poly) : List ℕ := (P.flatMap (·.witnesses)).eraseDups

def opWits : Opcode → List ℕ
  | .assertZero ts => witnessesOf ts
  | .range w _ => [w]

/-- The constraints mentioning a witness in `ws`, and every witness they
mention. -/
def hop (cc : List Opcode) (ws : List ℕ) : List ℕ × List ℕ :=
  let idxs := (cc.zipIdx.filter fun (c, _) => (opWits c).any ws.contains).map (·.2)
  (idxs, (ws ++ idxs.flatMap fun i => (cc[i]?.map opWits).getD []).eraseDups)

/-- The facts the steps so far established that mention a witness in `ws`. -/
def facts (s : Reps × Flag) (ws : List ℕ) : List Src :=
  let touches (P : Poly) := (witnessesOf P).any ws.contains
  let fromReps := (s.1.map (·.1)).eraseDups.flatMap fun v => match s.1.lookup v with
    | some (.scalar r) =>
      let idx := List.range r.alts.length
      let near := idx.filter fun i => touches r.alts[i]!
      (if r.L = r.M then near.map (Src.fixed v ·) else []) ++
      (if r.M ≤ 1 then near.map (Src.bit v ·) else []) ++
        near.flatMap fun i => (idx.filter (· ≠ i)).map (Src.same v i ·)
    | _ => []
  let flag := match s.2 with
    | some (P, _) => if touches P then [Src.flagBit] else []
    | none => []
  flag ++ fromReps

/-- A combination for `T`, searched over growing neighbourhoods of `T`'s
witnesses: the constraints touching them, unmultiplied; then the constraints
one step further, times each nearby witness; then also the facts, times
products of up to two nearby witnesses. -/
def find (deep : Bool) (cc : List Opcode) (s : Reps × Flag) (T : Poly) : Option Comb :=
  let w0 := witnessesOf T
  let (i1, w1) := hop cc w0
  match express cc s (gensAt cc i1 [[]]) T with
  | some c => some c
  | none =>
    let (i2, w2) := hop cc w1
    let muls1 := [] :: w1.map ([·])
    let fs := (facts s w1).eraseDups
    match express cc s (gensAt cc i2 muls1 ++ (fs.flatMap fun f => muls1.map (⟨f, ·, 1⟩)).toArray) T with
    | some c => some c
    | none =>
      -- multipliers read off `T`: every sub-multiset of one of its monomials
      let subs (m : List ℕ) : List (List ℕ) :=
        m.foldr (fun w acc => acc ++ acc.map (w :: ·)) [[]] |>.map (isort (fun x y => decide (x ≤ y)))
      let mulsT := (T.flatMap fun t => subs t.witnesses).eraseDups
      let gsT := gensAt cc i2 mulsT ++ (fs.flatMap fun f => mulsT.map (⟨f, ·, 1⟩)).toArray
      match (if gsT.size ≤ 8000 then express cc s gsT T else none) with
      | some c => some c
      | none => if !deep then none else
      let pairs := if w1.length ≤ 24 then
          w1.flatMap fun a => (w1.filter (a ≤ ·)).map fun b => [a, b]
        else w0.flatMap fun a => w1.map fun b => isort (fun x y => decide (x ≤ y)) [a, b]
      let muls := [] :: w2.map ([·]) ++ pairs.eraseDups
      if (i2.length + fs.length) * muls.length > 8000 then none
      else express cc s (gensAt cc i2 muls ++ (fs.flatMap fun f => muls.map (⟨f, ·, 1⟩)).toArray) T

/-- `find` for `T - c · M` with an unknown constant `c`. -/
def findC (cc : List Opcode) (s : Reps × Flag) (M T : Poly) : Option (ℕ × Comb) :=
  let w0 := witnessesOf (T ++ M)
  let (_, w1) := hop cc w0
  let (i2, _) := hop cc w1
  let muls := [] :: w1.map ([·])
  let fs := (facts s w1).eraseDups
  if (i2.length + fs.length) * muls.length > 8000 then none
  else expressC cc s (gensAt cc i2 muls ++ (fs.flatMap fun f => muls.map (⟨f, ·, 1⟩)).toArray) M T

/-- Bounds for scalar `d` by cases on a bit, when its bound does not fit its
type. -/
def caseBoundFor (cc : List Opcode) (s : Reps × Flag) (d : ℕ) : Option CaseBound := Id.run do
  let some (.scalar r) := s.1.lookup d | return none
  let .uint n := r.ty | return none
  if r.M < 2 ^ n then return none
  let bits := (s.1.map (·.1)).eraseDups.flatMap fun v => match s.1.lookup v with
    | some (.scalar rb) => if rb.M ≤ 1 then rb.alts else []
    | _ => []
  for (X, i) in r.alts.zipIdx do
    let wx := witnessesOf X
    let near := (hop cc wx).2
    let ranges := (cc.zipIdx.filterMap fun (c, idx) => match c with
      | .range w k => if k ≤ n ∧ near.contains w then some (w, idx) else none
      | _ => none)
    for sel in bits.eraseDups do
      if !(witnessesOf sel).all wx.contains then continue
      let some cs := find false cc s (psub (pmul sel sel) sel) | continue
      let forms (side : Poly) : List (Form × Comb) := ranges.flatMap fun (w, idx) =>
        [false, true].filterMap fun neg =>
          let T := pmul side (if neg then X ++ pvar w else psub X (pvar w))
          (findC cc s side T).bind fun (c, cmb) =>
            let f : Form := ⟨c, neg, idx⟩
            match f.poly cc with
            | some (F, _, _) => if combHolds cc s cmb (pmul side (psub X F)) then some (f, cmb) else none
            | none => none
      let nsel := psub (pconst 1) sel
      for (f₁, c₁) in forms sel do
        for (f₂, c₂) in forms nsel do
          let b : CaseBound := ⟨i, sel, cs, f₁, c₁, f₂, c₂⟩
          match tighten cc s d b with
          | some s' => match s'.1.lookup d with
            | some (.scalar r') => if r'.M < 2 ^ n then return some b
            | _ => pure ()
          | none => pure ()
  return none

/-! ## Hints -/

structure PHints where
  results : Std.HashMap ℕ Poly := {}
  internals : Std.HashMap ℕ (List Poly) := {}

def parsePoly (s : String) : Poly :=
  (s.splitOn " + ").filterMap fun t => match t.splitOn "*" with
    | [c, ws] =>
      let ws := ((ws.replace "[" "").replace "]" "").splitOn ","
      some ⟨c.toInt!, (ws.filter (· ≠ "")).map String.toNat!⟩
    | _ => none

def parseHints (text : String) : Std.HashMap String PHints := Id.run do
  let mut out : Std.HashMap String PHints := {}
  let mut cur := ""
  for line in text.splitOn "\n" do
    if line.startsWith "# program " then
      cur := (line.drop 10).toString
      out := out.insert cur {}
    else if line.startsWith "hint " then
      match line.splitOn " " with
      | _ :: k :: j :: _ =>
        if j = "0" then
          let rest := (line.drop (5 + k.length + 1 + j.length + 1)).toString
          if rest ≠ "-" then
            out := out.modify cur fun h => { h with results := h.results.insert k.toNat! (parsePoly rest) }
      | _ => pure ()
    else if line.startsWith "internal " then
      match line.splitOn " " with
      | _ :: k :: _ =>
        let rest := (line.drop (9 + k.length + 1)).toString
        out := out.modify cur fun h =>
          { h with internals := h.internals.insert k.toNat! (parsePoly rest :: h.internals.getD k.toNat! []) }
      | _ => pure ()
  return out

/-! ## Building hint steps -/

def rangeEvs (deep : Bool) (cc : List Opcode) (s : Reps × Flag) (P : Poly) (maxBits : ℕ) : List RangeEv :=
  (cc.zipIdx.filterMap fun (c, i) => match c with
    | .range w k => if k ≤ maxBits then some (w, i) else none
    | _ => none).filterMap fun (w, i) =>
      (find deep cc s (underFlag s.2 (psub P (pvar w)))).map fun c => ⟨i, c⟩

def firstRange (deep : Bool) (cc : List Opcode) (s : Reps × Flag) (P : Poly) (maxBits : ℕ) : Option RangeEv :=
  -- the witness `P` names first, then any range-checked witness
  let direct := match P with
    | [⟨1, [w]⟩] => (cc.zipIdx.find? fun (c, _) => match c with
        | .range w' k => w' = w ∧ k ≤ maxBits
        | _ => false).map fun (_, i) => (⟨i, []⟩ : RangeEv)
    | _ => none
  match direct with
  | some e => if (rangeOf cc s P e).isSome then some e else (rangeEvs deep cc s P maxBits).head?
  | none => (rangeEvs deep cc s P maxBits).head?

/-- The polynomials an operand can be named by: its representation's own, then
the compiler's, where a combination equates it with one of them. -/
def opnds (deep : Bool) (cc : List Opcode) (s : Reps × Flag) (r : Rep2) (H : Option Poly)
    (on : Bool := false) : List (Opnd × Poly) :=
  let own := (List.range r.alts.length).map fun i => (Opnd.alt i, r.alts[i]!)
  let via := match H with
    | some H =>
      if r.alts.contains H then []
      else
        let plain := (List.range r.alts.length).findSome? fun i =>
          (find deep cc s (psub H r.alts[i]!)).map fun c => (Opnd.via H i c, H)
        match plain, on, s.2 with
        | some x, _, _ => [x]
        | none, true, some _ => ((List.range r.alts.length).findSome? fun i =>
            (find deep cc s (underFlag s.2 (psub H r.alts[i]!))).map fun c => (Opnd.viaOn H i c, H)).toList
        | none, _, _ => []
    | none => []
  (if r.L = r.M then [(Opnd.fixed, pconst r.L)] else []) ++ via ++ own

/-- The nonconstant polynomials the compiler created for instruction `k`. -/
def candidates (h : PHints) (k : ℕ) : List Poly :=
  ((h.internals.getD k []).reverse ++ (match h.results.get? k with | some E => [E] | none => [])
    |>.map (·.filter (·.coef ≠ 0))
    |>.filter (·.any (·.witnesses ≠ []))).eraseDups

def tryHint (deep : Bool) (body : List Instruction) (cc : List Opcode) (s : Reps × Flag) (i : Instruction) (h : PHints) (k : ℕ)
    (hintOf : Operand → Option Poly) : Option HStep := Id.run do
  let cands := candidates h k
  match i with
  | .bin _ op u a b =>
    let some ra := opRep s.1 a | return none
    let some rb := opRep s.1 b | return none
    if (hintStep cc s i .fold).isSome then return some .fold
    let constOf (o : Operand) (r : Rep2) : List ℕ :=
      let hc := match hintOf o with
        | some H => match H.filter (·.coef ≠ 0) with
          | [] => [0]
          | [⟨c, []⟩] => [(modP c).toNat]
          | _ => []
        | none => []
      (hc ++ (if r.L = r.M then [r.L] else [])).eraseDups
    for ca in constOf a ra do
      for cb in constOf b rb do
        for (ia, Xa) in opnds deep cc s ra (hintOf a) do
          let some c₁ := find deep cc s (underFlag s.2 (psub Xa (pconst ca))) | continue
          for (ib, Xb) in opnds deep cc s rb (hintOf b) do
            let some c₂ := find deep cc s (underFlag s.2 (psub Xb (pconst cb))) | continue
            let st := HStep.foldOn ca cb ia ib c₁ c₂
            if (hintStep cc s i st).isSome then return some st
    let pairs := (opnds deep cc s ra (hintOf a)).flatMap fun x => (opnds deep cc s rb (hintOf b)).map (x, ·)
    let pairsOn := (opnds deep cc s ra (hintOf a) true).flatMap fun x => (opnds deep cc s rb (hintOf b) true).map (x, ·)
    if op = .eq then
      for ((ia, Xa), (ib, Xb)) in pairs do
        let D := psub Xa Xb
        if let some c := find deep cc s (psub (pmul D D) D) then
          let st := HStep.eqBit ia ib c
          if (hintStep cc s i st).isSome then return some st
      let some E := h.results.get? k | return none
      for ((ia, Xa), (ib, Xb)) in pairs do
        let D := psub Xa Xb
        let some c₂ := find deep cc s (pmul D E) | continue
        for z in cands do
          if let some c₁ := find deep cc s (psub (psub (pconst 1) (pmul D z)) E) then
            let st := HStep.eq E z ia ib c₁ c₂
            if (hintStep cc s i st).isSome then return some st
      return none
    if op = .div ∨ op = .mod then
      let n := match ra.ty with | .uint n => n | _ => 0
      let ranged := (cands.filter fun P => match P with
        | [⟨1, [w]⟩] | [⟨1, [w]⟩, ⟨0, []⟩] => cc.any fun c => c == .range w n || (match c with | .range w' k => w' = w ∧ k ≤ n | _ => false)
        | _ => false).map (fun P => P.filter (·.coef ≠ 0)) |>.eraseDups
      for ((ia, Xa), (ib, Xb)) in pairsOn do
        for q in ranged do
          for r in ranged do
            if q == r then continue
            let some c := find deep cc s (underFlag s.2 (psub (psub Xa (pmul Xb q)) r)) | continue
            let some qr := firstRange deep cc s q n | continue
            let some rr := firstRange deep cc s r n | continue
            let lts : List LtEv :=
              ((firstRange deep cc s (psub (psub Xb r) (pconst 1)) n).map LtEv.sub).toList ++
              (if rb.L = rb.M then
                (List.range (n + 1)).filterMap fun kk =>
                  if 2 ^ kk ≥ rb.M then
                    (firstRange deep cc s (r ++ pconst (2 ^ kk - rb.M)) kk).map (LtEv.shift (2 ^ kk - rb.M))
                  else none
              else [])
            for lt in lts do
              let st := HStep.divmod q r ia ib c qr rr lt
              if (hintStep cc s i st).isSome then return some st
      return none
    if op = .lt then
      let some E := h.results.get? k | return none
      let some n := (match ra.ty with | .uint n => some n | _ => none) | return none
      let some cb := find deep cc s (psub (pmul E E) E) | return none
      for ((ia, Xa), (ib, Xb)) in pairs do
        -- `r` is whatever is left: `a - b + 2^n E`, as a range-checked witness
        let T := psub Xa Xb ++ pmul (pconst (2 ^ n)) E
        for r in cands do
          let some c := find deep cc s (psub T r) | continue
          let some rr := firstRange deep cc s r n | continue
          let st := HStep.lt E r ia ib c cb rr
          if (hintStep cc s i st).isSome then return some st
      return none
    if op = .add ∧ ra.ty ≠ .field ∧ !deep then
      -- an `if`/`else` merge: `a = s · y`, `b = (1 - s) · z`
      let factors (o : Operand) : List (Operand × Operand) := match o with
        | .var id => match body.find? (fun j => Instruction.dst? j == some id) with
          | some (.bin _ .mul true x y) => [(x, y), (y, x)]
          | _ => []
        | _ => []
      for (sa, ya) in factors a do
        for (sb, zb) in factors b do
          let some rs := opRep s.1 sa | continue
          let some rs' := opRep s.1 sb | continue
          let some ry := opRep s.1 ya | continue
          let some rz := opRep s.1 zb | continue
          if rs.M > 1 ∨ rs'.M > 1 then continue
          for sel in rs.alts do
            let some cs := find deep cc s (psub (pmul sel sel) sel) | continue
            for ((ia, Xa), (ib, Xb)) in pairs do
              for iy in List.range ry.alts.length do
                let some ca := find deep cc s (psub Xa (pmul sel ry.alts[iy]!)) | continue
                for iz in List.range rz.alts.length do
                  let some cb := find deep cc s (psub Xb (pmul (psub (pconst 1) sel) rz.alts[iz]!)) | continue
                  let st := HStep.mux sel ya iy zb iz ia ib cs ca cb
                  if (hintStep cc s i st).isSome then return some st
    if (op = .add ∨ op = .sub ∨ op = .mul) ∧ !u then
      let some E := h.results.get? k | return none
      let n := match ra.ty with | .uint n => n | _ => 0
      for ((ia, Xa), (ib, Xb)) in pairsOn do
        let some T := arithPoly op Xa Xb | continue
        let some c := find deep cc s (underFlag s.2 (psub E T)) | continue
        let some rng := firstRange deep cc s E n | continue
        let st := HStep.arith E ia ib c rng
        if (hintStep cc s i st).isSome then return some st
      return none
    return none
  | .constrain a b _ =>
    let some ra := opRep s.1 a | return none
    let some rb := opRep s.1 b | return none
    for (ia, Xa) in opnds deep cc s ra (hintOf a) do
      for (ib, Xb) in opnds deep cc s rb (hintOf b) do
        if let some c := find deep cc s (psub Xa Xb) then
          let st := HStep.constrain ia ib c
          if (hintStep cc s i st).isSome then return some st
    return none
  | .constrainNe a b _ =>
    let some ra := opRep s.1 a | return none
    let some rb := opRep s.1 b | return none
    for (ia, Xa) in opnds deep cc s ra (hintOf a) do
      for (ib, Xb) in opnds deep cc s rb (hintOf b) do
        for z in cands do
          if let some c := find deep cc s (underFlag s.2 (psub (pconst 1) (pmul (psub Xa Xb) z))) then
            let st := HStep.constrainNe z ia ib c
            if (hintStep cc s i st).isSome then return some st
    return none
  | _ => return none

/-! ## Certificates -/

/-- Positions in `keep` whose removal leaves `stepP`'s result unchanged are
dropped, in shrinking blocks. -/
partial def shrink (cc : List Opcode) (s : Reps × Flag) (i : Instruction)
    (want : Option (Reps × Flag)) (keep : Array ℕ) (block : ℕ) (at_ : ℕ) : Array ℕ :=
  if block == 0 then keep
  else if at_ ≥ keep.size then shrink cc s i want keep (block / 2) 0
  else
    let trial := keep.extract 0 at_ ++ keep.extract (at_ + block) keep.size
    if stepP (pick cc trial.toList) s i == want then shrink cc s i want trial block at_
    else shrink cc s i want keep block (at_ + block)

def dest : Instruction → Option ℕ
  | .bin d .. | .not d _ | .cast d .. | .truncate d .. | .arrayGet d .. | .arraySet d .. | .makeArray d .. => some d
  | _ => none

/-- A result `stepP` describes poorly, where a hint step may do better: no
polynomial (an `eq` flag the searches did not find), or an integer whose upper
bound does not fit its type. -/
def weak (s' : Reps × Flag) (s : Reps × Flag) : Bool :=
  s'.1.length > s.1.length && match s'.1.head? with
    | some (_, .scalar r) => r.alts.isEmpty || (match r.ty with
      | .uint n => decide (2 ^ n ≤ r.M)
      | _ => false)
    | _ => false

/-- The compiler's polynomial `H` for the value `i` defines, as an alias of one
of its polynomials, when a combination shows they are equal. -/
def aliasFor (cc : List Opcode) (s : Reps × Flag) (i : Instruction) (H : Poly) :
    Option (Poly × ℕ × Comb) := do
  let d ← i.dst?
  let .scalar r ← s.1.lookup d | none
  let H := H.filter (·.coef ≠ 0)
  if r.alts.contains H then none
  let try_ (deep : Bool) := (List.range r.alts.length).findSome? fun j =>
    (find deep cc s (psub H r.alts[j]!)).map (H, j, ·)
  (try_ false).orElse fun _ => try_ true

def tstr (t : Term) : String :=
  let c := (modP t.coef).toNat
  let c : ℤ := if c > p / 2 then (c : ℤ) - p else c
  s!"{c}{t.witnesses}"
def pstr (P : Poly) : String := " + ".intercalate (P.map tstr)

/-- The certificate, the return hints, and a log of each step. -/
def cert (e : TestProgram) (h : PHints) (verbose : Bool) :
    IO (List Entry × List (Option (ℕ × Comb)) × String) := do
  let mut stuckMsg := ""
  let cc := e.fn.opcodes
  let all := (List.range cc.length).toArray
  let some reps0 := initReps cc e.prog.params e.fn.parameters | return ([], [], "initReps")
  let mut s : Reps × Flag := (reps0, none)
  let mut out : Array Entry := #[]
  let mut hmap : Std.HashMap ℕ Poly := {}
  for (i, k) in e.prog.body.zipIdx do
    let hm := hmap
    let hintOf : Operand → Option Poly := fun o => match o with
      | .var id => hm.get? id
      | .const _ _ => none
    if let some E := h.results.get? k then
      if let some d := dest i then hmap := hmap.insert d E
    let t0 ← IO.monoMsNow
    let want := stepP cc s i
    let useHint := match want with
      | none => true
      | some s' => weak s' s
    let hint := if useHint then (tryHint false e.prog.body cc s i h k hintOf).orElse fun _ => tryHint true e.prog.body cc s i h k hintOf else none
    let (ix, st, s') ← match hint, want with
      | some st, _ =>
        match hintStep cc s i st with
        | some s' => pure ([], some st, s')
        | none => return (out.toList, [], "hint rejected")
      | none, some s' => pure ((shrink cc s i want all (max 1 (all.size / 2)) 0).toList, none, s')
      | none, none =>
        stuckMsg := stuckMsg ++ s!"\nSTUCK at {k}: {i.render.trimAsciiStart} flag={s.2.map fun f => pstr f.1} cands={(candidates h k).map pstr}"
        return (out.toList, [], stuckMsg)
    let al := (h.results.get? k).bind (aliasFor cc s' i)
    if verbose then
      IO.eprintln s!"  [{k}] {(← IO.monoMsNow) - t0} ms {i.render.trimAsciiStart}"
      (← IO.getStderr).flush
    let sa := match al, i.dst? with
      | some (H, j, c), some d => (addAlias cc s' d H j c).getD s'
      | _, _ => s'
    let bd := i.dst?.bind (caseBoundFor cc sa)
    let en : Entry := ⟨ix, st, al, bd⟩
    match stepE cc s i en with
    | some s'' =>
      out := out.push en; s := s''
      match i.dst?.bind fun d => s''.1.lookup d with
      | some (.scalar r) =>
        stuckMsg := stuckMsg ++ s!"\n  {k}: {i.render.trimAsciiStart}{if st.isSome then " [hint]" else ""} => " ++
          s!"{r.alts.map pstr} [{r.L},{r.M}] compiler={(h.results.get? k).map pstr}"
      | _ => pure ()
    | none => return (out.toList, [], "entry rejected")
  -- return values
  let some rss := e.prog.rets.mapM (opFlat s.1) | return (out.toList, [], "rets")
  stuckMsg := stuckMsg ++ s!"\nreturns {e.fn.returnValues} = {rss.flatten.map fun r => r.alts.map pstr}"
  let rets := (e.fn.returnValues.zip rss.flatten).map fun (w, r) =>
    if retOK cc w r then none
    else ((List.range r.alts.length).findSome? fun j =>
      (find true cc s (psub (pvar w) r.alts[j]!)).map (j, ·))
  return (out.toList, (if rets.all (·.isNone) then [] else rets), stuckMsg)

def showInt (c : ℤ) : String := if c < 0 then s!"({c})" else toString c
def showPoly (P : Poly) : String :=
  "[" ++ ", ".intercalate (P.map fun t => s!"⟨{showInt t.coef}, {t.witnesses}⟩") ++ "]"
def showSrc : Src → String
  | .con i => s!"(.con {i})"
  | .same v i j => s!"(.same {v} {i} {j})"
  | .fixed v i => s!"(.fixed {v} {i})"
  | .bit v i => s!"(.bit {v} {i})"
  | .flagBit => ".flagBit"
def showComb (c : Comb) : String :=
  "[" ++ ", ".intercalate (c.map fun g => s!"⟨{showSrc g.src}, {g.mul}, {showInt g.coef}⟩") ++ "]"
def showRange (e : RangeEv) : String := s!"⟨{e.idx}, {showComb e.cmb}⟩"
def showTy : ValueType → String
  | .field => ".field"
  | .uint n => s!"(.uint {n})"
  | .sint n => s!"(.sint {n})"
def showOperand : Operand → String
  | .var id => s!".var {id}"
  | .const v ty => s!".const {showInt v} {showTy ty}"
def showForm (f : Form) : String := s!"⟨{f.c}, {f.neg}, {f.idx}⟩"
def showOpnd : Opnd → String
  | .alt i => s!"(.alt {i})"
  | .via H i c => s!"(.via {showPoly H} {i} {showComb c})"
  | .fixed => ".fixed"
  | .viaOn H i c => s!"(.viaOn {showPoly H} {i} {showComb c})"
def showStep : HStep → String
  | .arith E ia ib c r => s!"(.arith {showPoly E} {showOpnd ia} {showOpnd ib} {showComb c} {showRange r})"
  | .divmod q r ia ib c qr rr lt =>
    let l := match lt with
      | .sub e => s!"(.sub {showRange e})"
      | .shift d e => s!"(.shift {d} {showRange e})"
    s!"(.divmod {showPoly q} {showPoly r} {showOpnd ia} {showOpnd ib} {showComb c} {showRange qr} {showRange rr} {l})"
  | .eq E z ia ib c₁ c₂ => s!"(.eq {showPoly E} {showPoly z} {showOpnd ia} {showOpnd ib} {showComb c₁} {showComb c₂})"
  | .constrain ia ib c => s!"(.constrain {showOpnd ia} {showOpnd ib} {showComb c})"
  | .constrainNe z ia ib c => s!"(.constrainNe {showPoly z} {showOpnd ia} {showOpnd ib} {showComb c})"
  | .fold => ".fold"
  | .mux sel y iy z iz ia ib cs ca cb => s!"(.mux {showPoly sel} ({showOperand y}) {iy} ({showOperand z}) {iz} {showOpnd ia} {showOpnd ib} {showComb cs} {showComb ca} {showComb cb})"
  | .lt E r ia ib c cb rr => s!"(.lt {showPoly E} {showPoly r} {showOpnd ia} {showOpnd ib} {showComb c} {showComb cb} {showRange rr})"
  | .eqBit ia ib c => s!"(.eqBit {showOpnd ia} {showOpnd ib} {showComb c})"
  | .foldOn ca cb ia ib c₁ c₂ => s!"(.foldOn {ca} {cb} {showOpnd ia} {showOpnd ib} {showComb c₁} {showComb c₂})"

def main (args : List String) : IO Unit := do
  let hints := parseHints (← IO.FS.readFile args[0]!)
  let mut s := "/-\nMACHINE-CHECKED: no review needed. Generated by `scripts/emit_hint_certs.lean`;\n" ++
    "the kernel runs each step over the constraints listed (`stepsWithH`), or checks\n" ++
    "its hint step.\n-/\n\nimport AcirLean.Proofs.HintChecker\n\nnamespace AcirLean\n\n"
  let mut names := #[]
  let mut part := ""
  let mut nh := 0
  let mut na := 0
  let only := args.drop 2
  for (e, idx) in testPrograms.zipIdx do
    if !only.isEmpty && !only.contains e.name then continue
    let (c, r, msg) ← cert e (hints.getD e.name {}) (!only.isEmpty)
    -- aliases and bounds cost the kernel at every later step; keep them only
    -- where the program needs them
    let bare := c.map fun en => { en with extra := none, bound := none }
    let c := if checkProgH e.prog e.fn bare r then bare else c
    if !checkProgH e.prog e.fn c r then
      IO.eprintln s!"not proved: {e.name}"
      if !only.isEmpty then IO.eprintln msg
    let steps := c.map fun en =>
      let st := match en.step with | none => "none" | some st => s!"some {showStep st}"
      let al := match en.extra with | none => "none" | some (H, j, cmb) => s!"some ({showPoly H}, {j}, {showComb cmb})"
      let bd := match en.bound with
        | none => "none"
        | some b => s!"some ⟨{b.i}, {showPoly b.sel}, {showComb b.cs}, {showForm b.f₁}, {showComb b.c₁}, {showForm b.f₂}, {showComb b.c₂}⟩"
      s!"⟨{en.ix}, {st}, {al}, {bd}⟩"
    nh := nh + (c.filter (·.step.isSome)).length
    na := na + (c.filter (·.extra.isSome)).length
    let rs := r.map fun x => match x with
      | none => "none"
      | some (j, cmb) => s!"some ({j}, {showComb cmb})"
    let d := s!"def cert{idx} : List Entry :=\n  [{", ".intercalate steps}]\n" ++
      s!"def rets{idx} : List (Option (ℕ × Comb)) := [{", ".intercalate rs}]\n\n"
    s := s ++ d
    part := part ++ d
    names := names.push idx
  s := s ++ "def testProgramCerts : List (List Entry × List (Option (ℕ × Comb))) := [" ++
    ", ".intercalate (names.toList.map fun i => s!"(cert{i}, rets{i})") ++ "]\n\nend AcirLean\n"
  -- with names, write only their definitions, for assembling a full file
  if only.isEmpty then IO.FS.writeFile args[1]! s else IO.FS.writeFile args[1]! part
  IO.eprintln s!"{nh} hint steps, {na} aliases"
