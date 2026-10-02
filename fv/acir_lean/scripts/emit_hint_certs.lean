/-
Not trusted. Writes `AcirLean/Proofs/TestProgramCerts.lean`: for each test
program and each instruction, either the positions of the few constraints
`stepP` needs (as `emit_certs.lean` always did), or, where `stepP` fails, a hint
step built from the polynomials the compiler assigned (`hints.txt`, printed by
the `dump_hints` test) and combinations of constraints found by Gaussian
elimination. The kernel checks every combination, so nothing here is trusted.

Usage: lake env lean --run scripts/emit_hint_certs.lean <hints.txt> <out.lean>
-/

import AcirLean.Proofs.HintChecker
import AcirLean.Templates.TestPrograms

open AcirLean

deriving instance Inhabited for Gen

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

/-- The generators: every `AssertZero` and 1-bit range check, times each of the
multipliers. -/
def gens (cc : List Opcode) (muls : List (List ℕ)) : Array Gen := Id.run do
  let mut out := #[]
  for (c, i) in cc.zipIdx do
    match c with
    | .assertZero _ | .range _ 1 => for m in muls do out := out.push ⟨i, m, 1⟩
    | _ => pure ()
  return out

/-- A combination of `gs` equal to `T`, if there is one. -/
def express (cc : List Opcode) (gs : Array Gen) (T : Poly) : Option Comb := Id.run do
  let mut B : Basis := {}
  for (g, i) in gs.toList.zipIdx do
    B := addRow B ⟨toN (genPoly cc g), ({} : Std.HashMap ℕ ℕ).insert i 1⟩
  let r := reduce B ⟨toN T, {}⟩
  if r.poly.isEmpty then
    -- T - Σ c_g g = 0 with r.comb = -c
    some (r.comb.fold (init := []) fun acc i c =>
      let g := gs[i]!
      ⟨g.idx, g.mul, ((p - c) % p : ℕ)⟩ :: acc)
  else none

def witnessesOf (P : Poly) : List ℕ := (P.flatMap (·.witnesses)).eraseDups

/-- A combination for `T`: first from the constraints alone, then with each
constraint also multiplied by each witness of `T`. -/
def find (cc : List Opcode) (T : Poly) : Option Comb :=
  match express cc (gens cc [[]]) T with
  | some c => some c
  | none => express cc (gens cc ([] :: (witnessesOf T).map ([·]))) T

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

def rangeEvs (cc : List Opcode) (f : Flag) (P : Poly) (maxBits : ℕ) : List RangeEv :=
  (cc.zipIdx.filterMap fun (c, i) => match c with
    | .range w k => if k ≤ maxBits then some (w, i) else none
    | _ => none).filterMap fun (w, i) =>
      (find cc (underFlag f (psub P (pvar w)))).map fun c => ⟨i, c⟩

def firstRange (cc : List Opcode) (f : Flag) (P : Poly) (maxBits : ℕ) : Option RangeEv :=
  -- the witness `P` names first, then any range-checked witness
  let direct := match P with
    | [⟨1, [w]⟩] => (cc.zipIdx.find? fun (c, _) => match c with
        | .range w' k => w' = w ∧ k ≤ maxBits
        | _ => false).map fun (_, i) => (⟨i, []⟩ : RangeEv)
    | _ => none
  match direct with
  | some e => if (rangeOf cc f P e).isSome then some e else (rangeEvs cc f P maxBits).head?
  | none => (rangeEvs cc f P maxBits).head?

/-- The polynomials an operand can be named by: its representation's own, then
the compiler's, where a combination equates it with one of them. -/
def opnds (cc : List Opcode) (r : Rep2) (H : Option Poly) : List (Opnd × Poly) :=
  let own := (List.range r.alts.length).map fun i => (Opnd.alt i, r.alts[i]!)
  let via := match H with
    | some H =>
      if r.alts.contains H then []
      else ((List.range r.alts.length).findSome? fun i =>
        (find cc (psub H r.alts[i]!)).map fun c => (Opnd.via H i c, H)).toList
    | none => []
  via ++ own

def candidates (h : PHints) (k : ℕ) : List Poly :=
  (h.internals.getD k []).reverse ++ (match h.results.get? k with | some E => [E] | none => [])

def tryHint (cc : List Opcode) (s : Reps × Flag) (i : Instruction) (h : PHints) (k : ℕ)
    (hintOf : Operand → Option Poly) : Option HStep := Id.run do
  let cands := candidates h k
  match i with
  | .bin _ op _ a b =>
    let some ra := opRep s.1 a | return none
    let some rb := opRep s.1 b | return none
    let pairs := (opnds cc ra (hintOf a)).flatMap fun x => (opnds cc rb (hintOf b)).map (x, ·)
    if op = .eq then
      let some E := h.results.get? k | return none
      for ((ia, Xa), (ib, Xb)) in pairs do
        let D := psub Xa Xb
        let some c₂ := find cc (pmul D E) | continue
        for z in cands do
          if let some c₁ := find cc (psub (psub (pconst 1) (pmul D z)) E) then
            let st := HStep.eq E z ia ib c₁ c₂
            if (hintStep cc s i st).isSome then return some st
      return none
    if op = .div ∨ op = .mod then
      let n := match ra.ty with | .uint n => n | _ => 0
      for ((ia, Xa), (ib, Xb)) in pairs do
        for q in cands do
          for r in cands do
            let some c := find cc (underFlag s.2 (psub (psub Xa (pmul Xb q)) r)) | continue
            let some qr := firstRange cc s.2 q n | continue
            let some rr := firstRange cc s.2 r n | continue
            let lts : List LtEv :=
              ((firstRange cc s.2 (psub (psub Xb r) (pconst 1)) n).map LtEv.sub).toList ++
              (if rb.L = rb.M then
                (List.range (n + 1)).filterMap fun kk =>
                  if 2 ^ kk ≥ rb.M then
                    (firstRange cc s.2 (r ++ pconst (2 ^ kk - rb.M)) kk).map (LtEv.shift (2 ^ kk - rb.M))
                  else none
              else [])
            for lt in lts do
              let st := HStep.divmod q r ia ib c qr rr lt
              if (hintStep cc s i st).isSome then return some st
      return none
    if op = .add ∨ op = .sub ∨ op = .mul then
      let some E := h.results.get? k | return none
      let n := match ra.ty with | .uint n => n | _ => 0
      for ((ia, Xa), (ib, Xb)) in pairs do
        let some T := arithPoly op Xa Xb | continue
        let some c := find cc (underFlag s.2 (psub E T)) | continue
        let some rng := firstRange cc s.2 E n | continue
        let st := HStep.arith E ia ib c rng
        if (hintStep cc s i st).isSome then return some st
      return none
    return none
  | .constrain a b _ =>
    let some ra := opRep s.1 a | return none
    let some rb := opRep s.1 b | return none
    for (ia, Xa) in opnds cc ra (hintOf a) do
      for (ib, Xb) in opnds cc rb (hintOf b) do
        if let some c := find cc (psub Xa Xb) then
          let st := HStep.constrain ia ib c
          if (hintStep cc s i st).isSome then return some st
    return none
  | .constrainNe a b _ =>
    let some ra := opRep s.1 a | return none
    let some rb := opRep s.1 b | return none
    for (ia, Xa) in opnds cc ra (hintOf a) do
      for (ib, Xb) in opnds cc rb (hintOf b) do
        for z in cands do
          if let some c := find cc (underFlag s.2 (psub (pconst 1) (pmul (psub Xa Xb) z))) then
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

/-- A result whose `eq` flag the searches did not find has no polynomial; a
hint step does better. -/
def weak (s' : Reps × Flag) (s : Reps × Flag) : Bool :=
  s'.1.length > s.1.length && match s'.1.head? with
    | some (_, .scalar r) => r.alts.isEmpty
    | _ => false

def cert (e : TestProgram) (h : PHints) :
    List (List ℕ × Option HStep) × List (Option (ℕ × Comb)) × String := Id.run do
  let mut stuckMsg := ""
  let cc := e.fn.opcodes
  let all := (List.range cc.length).toArray
  let some reps0 := initReps cc e.prog.params e.fn.parameters | return ([], [], "initReps")
  let mut s : Reps × Flag := (reps0, none)
  let mut out := #[]
  let mut hmap : Std.HashMap ℕ Poly := {}
  for (i, k) in e.prog.body.zipIdx do
    let hm := hmap
    let hintOf : Operand → Option Poly := fun o => match o with
      | .var id => hm.get? id
      | .const _ _ => none
    if let some E := h.results.get? k then
      if let some d := dest i then hmap := hmap.insert d E
    let want := stepP cc s i
    let useHint := match want with
      | none => true
      | some s' => weak s' s
    let hint := if useHint then tryHint cc s i h k hintOf else none
    match hint, want with
    | some st, _ =>
      match hintStep cc s i st with
      | some s' => out := out.push ([], some st); s := s'
      | none => return (out.toList, [], "hint rejected")
    | none, some s' =>
      out := out.push ((shrink cc s i want all (max 1 (all.size / 2)) 0).toList, none)
      s := s'
    | none, none =>
      stuckMsg := s!"{e.name}: stuck at {k}: {i.render.trimAsciiStart} flag={s.2.isSome} cands={(candidates h k).length}"
      return (out.toList, [], stuckMsg)
  -- return values
  let some rss := e.prog.rets.mapM (opFlat s.1) | return (out.toList, [], "rets")
  let rets := (e.fn.returnValues.zip rss.flatten).map fun (w, r) =>
    if retOK cc w r then none
    else ((List.range r.alts.length).findSome? fun j =>
      (find cc (psub (pvar w) r.alts[j]!)).map (j, ·))
  return (out.toList, (if rets.all (·.isNone) then [] else rets), stuckMsg)

def showInt (c : ℤ) : String := if c < 0 then s!"({c})" else toString c
def showPoly (P : Poly) : String :=
  "[" ++ ", ".intercalate (P.map fun t => s!"⟨{showInt t.coef}, {t.witnesses}⟩") ++ "]"
def showComb (c : Comb) : String :=
  "[" ++ ", ".intercalate (c.map fun g => s!"⟨{g.idx}, {g.mul}, {showInt g.coef}⟩") ++ "]"
def showRange (e : RangeEv) : String := s!"⟨{e.idx}, {showComb e.cmb}⟩"
def showOpnd : Opnd → String
  | .alt i => s!"(.alt {i})"
  | .via H i c => s!"(.via {showPoly H} {i} {showComb c})"
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

def main (args : List String) : IO Unit := do
  let hints := parseHints (← IO.FS.readFile args[0]!)
  let mut s := "/-\nMACHINE-CHECKED: no review needed. Generated by `scripts/emit_hint_certs.lean`;\n" ++
    "the kernel runs each step over the constraints listed (`stepsWithH`), or checks\n" ++
    "its hint step.\n-/\n\nimport AcirLean.Proofs.HintChecker\n\nnamespace AcirLean\n\n"
  let mut names := #[]
  let mut nh := 0
  for (e, idx) in testPrograms.zipIdx do
    let (c, r, msg) := cert e (hints.getD e.name {})
    if !checkProgH e.prog e.fn c r then IO.eprintln s!"not proved: {e.name} {msg}"
    let steps := c.map fun (ix, st) => s!"({ix}, {match st with | none => "none" | some st => s!"some {showStep st}"})"
    nh := nh + (c.filter (·.2.isSome)).length
    let rs := r.map fun x => match x with
      | none => "none"
      | some (j, cmb) => s!"some ({j}, {showComb cmb})"
    s := s ++ s!"def cert{idx} : List (List ℕ × Option HStep) :=\n  [{", ".intercalate steps}]\n" ++
      s!"def rets{idx} : List (Option (ℕ × Comb)) := [{", ".intercalate rs}]\n\n"
    names := names.push idx
  s := s ++ "def testProgramCerts : List (List (List ℕ × Option HStep) × List (Option (ℕ × Comb))) := [" ++
    ", ".intercalate (names.toList.map fun i => s!"(cert{i}, rets{i})") ++ "]\n\nend AcirLean\n"
  IO.FS.writeFile args[1]! s
  IO.eprintln s!"{nh} hint steps"
