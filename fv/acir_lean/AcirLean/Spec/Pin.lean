/-
REVIEWED: this file is part of the trusted specification (`AcirLean/Spec/`).
Every definition here is taken on trust: read it against its comment. A change
to this directory needs careful review.
-/

import AcirLean.Spec.Semantics
import AcirLean.Templates.Gadgets
import AcirLean.Templates.Signed
import AcirLean.Templates.Programs
import AcirLean.Templates.Shipped
import AcirLean.Templates.SignedDivMod
import AcirLean.Templates.Corpus
import AcirLean.Templates.TestPrograms

/-!
# The pin

`renderAll` prints the constraint lists for every width in `pinnedWidths`, in
the canonical text form that `fv_templates.rs` also prints the Rust gadgets'
output in, followed by the pinned SSA in `Ssa`'s own `Display` syntax. `scripts/check.sh` fails unless this printout equals
`templates.golden`, and the Rust test fails unless the gadgets' printout does.

`renderTestPrograms` prints each test program's final SSA and shipped circuit
in the same syntax; `scripts/check.sh` fails unless it equals
`test_programs.golden`, which `scripts/regen_programs.sh` writes from
`nargo compile` output.

Canonical form: `zero c*[i,j] + c*[i] + c*[]` with each term's witnesses
sorted, terms sorted by witness list, zero terms dropped, and the sign chosen so the first coefficient is at
most `(p-1)/2`; or `range i k`.
-/

namespace AcirLean

/-- The widths whose constraint lists are pinned to the Rust output. The claims
are stated for exactly these widths. -/
def pinnedWidths : List ℕ := [8, 16, 32, 64, 128]

/-- The signed widths pinned for `div` and `mod` (there is no `i128`). -/
def signedWidths : List ℕ := [8, 16, 32, 64]

/-- Lexicographic order on witness lists (the order `Vec<u32>` sorts in Rust). -/
def witnessListLe : List ℕ → List ℕ → Bool
  | [], _ => true
  | _ :: _, [] => false
  | a :: as, b :: bs => if a < b then true else if b < a then false else witnessListLe as bs

/-- An integer coefficient as a field element's integer value, in `[0, p)`. -/
def coefValue (c : ℤ) : ℕ := (c % (p : ℤ)).toNat

/-- One constraint in canonical form (see the module comment). -/
def Constraint.render : Constraint → String
  | .range w k => s!"range {w} {k}"
  | .zero ts =>
    let ts := ts.map (fun t => { t with witnesses := t.witnesses.mergeSort (· ≤ ·) })
    let ts := (ts.filter (fun t => coefValue t.coef ≠ 0)).mergeSort (fun a b => witnessListLe a.witnesses b.witnesses)
    let neg : Bool := match ts with
      | t :: _ => decide (coefValue t.coef > (p - 1) / 2)
      | [] => false
    let body := ts.map fun t =>
      let c := if neg then coefValue (-t.coef) else coefValue t.coef
      s!"{c}*[{",".intercalate (t.witnesses.map toString)}]"
    "zero " ++ " + ".intercalate body

/-- An ACIR function: its constraints, then its input and return witnesses. -/
def AcirFunction.render (f : AcirFunction) : List String :=
  let ws (l : List ℕ) := ",".intercalate (l.map toString)
  f.constraints.map Constraint.render ++ [s!"inputs [{ws f.inputs}]", s!"returns [{ws f.returns}]"]

/-- A straight-line program as `Ssa`'s `Display` prints it. -/
def CorpusProgram.render (P : CorpusProgram) : List String :=
  let params := ", ".intercalate ((List.range P.nparams).map fun i => s!"v{i}: u{P.width}")
  let op : CorpusOp → String
    | .div => "div"
    | .lt => "lt"
  ["acir(inline) fn main f0 {", s!"  b0({params}):"] ++
    (P.body.zipIdx.map fun (i, k) => s!"    v{P.nparams + k} = {op i.op} v{i.a}, v{i.b}") ++
    [s!"    return v{P.ret}", "}"]

/-- A corpus entry: the program, its shipped circuit, and the solved witness. -/
def CorpusEntry.render (e : CorpusEntry) : List String :=
  e.prog.render ++ e.fn.render ++ e.witness.map fun (w, v) => s!"witness {w} {v}"

/-- A test program: its SSA, then its shipped circuit. -/
def TestProgram.render (e : TestProgram) : List String :=
  s!"# program {e.name}" :: e.prog.render ++ e.fn.render

/-- `test_programs.golden`: every test program in `testPrograms`. -/
def renderTestPrograms : String :=
  "".intercalate (testPrograms.map fun e => "\n".intercalate e.render ++ "\n")

/-- The golden file: one `# <gadget> <width>` section per pinned width. -/
def renderAll : String :=
  let sec (title : String) (cs : List Constraint) :=
    s!"# {title}\n" ++ "\n".intercalate (cs.map Constraint.render)
  let secs := pinnedWidths.map (fun n => sec s!"div_var {n}" (divVarGadget n)) ++
    pinnedWidths.map (fun n => sec s!"div_var_predicated {n}" (divPredGadget n)) ++
    pinnedWidths.map (fun k => sec s!"truncate_field {k}" (truncateGadget k)) ++
    pinnedWidths.map (fun m => sec s!"more_than_eq {m}" (moreThanEqGadget m)) ++
    pinnedWidths.map (fun n => s!"# signed_lt {n}\n" ++ "\n".intercalate (signedLtSsa n).render) ++
    pinnedWidths.map (fun n => s!"# acir_div {n}\n" ++ "\n".intercalate (acirGenDiv n).render) ++
    pinnedWidths.map (fun n => s!"# acir_lt {n}\n" ++ "\n".intercalate (acirGenLt n).render) ++
    pinnedWidths.map (fun n =>
      s!"# acir_truncate {n}\n" ++ "\n".intercalate (acirGenTruncate n).render) ++
    pinnedWidths.map (fun n =>
      s!"# acir_signed_lt {n}\n" ++ "\n".intercalate (acirGenSignedLt n).render) ++
    pinnedWidths.map (fun n => s!"# shipped_div {n}\n" ++ "\n".intercalate (shippedDiv n).render) ++
    pinnedWidths.map (fun n => s!"# shipped_lt {n}\n" ++ "\n".intercalate (shippedLt n).render) ++
    pinnedWidths.map (fun n =>
      s!"# shipped_truncate {n}\n" ++ "\n".intercalate (shippedTruncate n).render) ++
    pinnedWidths.map (fun n =>
      s!"# shipped_signed_lt {n}\n" ++ "\n".intercalate (shippedSignedLt n).render) ++
    signedWidths.map (fun n =>
      s!"# shipped_signed_div {n}\n" ++ "\n".intercalate (shippedSignedDiv n).render) ++
    signedWidths.map (fun n =>
      s!"# shipped_signed_mod {n}\n" ++ "\n".intercalate (shippedSignedMod n).render) ++
    corpus.zipIdx.map (fun (e, i) => s!"# corpus {i}\n" ++ "\n".intercalate e.render)
  "\n".intercalate secs ++ "\n"

end AcirLean
