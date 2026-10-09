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

/-!
# The pin

`renderAll` prints the constraint lists for every width in `pinnedWidths`, in
the canonical text form that `fv_templates.rs` also prints the Rust gadgets'
output in, followed by the pinned SSA in `Ssa`'s own `Display` syntax. `scripts/check.sh` fails unless this printout equals
`templates.golden`, and the Rust test fails unless the gadgets' printout does.

Canonical form (`Opcode.canon`): `zero c*[i,j] + c*[i] + c*[]` with each
term's witnesses sorted, coefficients reduced to `[0, p)`, zero terms dropped,
terms sorted by witness list, and the sign chosen so the first coefficient is
at most `(p-1)/2`; or `range i k`. Terms over the same witnesses are not
merged: Rust merges them, so a constraint with two such terms prints
differently on the two sides and fails the pin.
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

/-- An integer coefficient reduced to a field element's integer value, in `[0, p)`. -/
def modP (c : ℤ) : ℤ := c % (p : ℤ)

/-- Insert `x` before the first element it is `le`. -/
def insertBy {α : Type} (le : α → α → Bool) (x : α) : List α → List α
  | [] => [x]
  | y :: ys => if le x y then x :: y :: ys else y :: insertBy le x ys

/-- Insertion sort, by structural recursion so the kernel can evaluate it. -/
def isort {α : Type} (le : α → α → Bool) : List α → List α
  | [] => []
  | x :: xs => insertBy le x (isort le xs)

/-- A constraint in canonical form (see the module comment). `Opcode.canon_sat`
(`Proofs/Canon.lean`) proves that a constraint holds exactly when its canonical
form does, so nothing about this definition needs checking by eye except that
it matches `fv_templates.rs`'s `canonical`, which the golden files check. -/
def Opcode.canon : Opcode → Opcode
  | .range w k => .range w k
  | .assertZero ts =>
    let ts := (ts.map fun t => (⟨modP t.coef, isort (fun a b => decide (a ≤ b)) t.witnesses⟩ : Term))
    let ts := isort (fun a b => witnessListLe a.witnesses b.witnesses) (ts.filter fun t => t.coef != 0)
    let neg : Bool := match ts with
      | t :: _ => decide (t.coef > ((p - 1) / 2 : ℕ))
      | [] => false
    .assertZero (if neg then ts.map (fun t => (⟨modP (-t.coef), t.witnesses⟩ : Term)) else ts)

/-- One constraint, printed in canonical form. -/
def Opcode.render (c : Opcode) : String :=
  match c.canon with
  | .range w k => s!"range {w} {k}"
  | .assertZero ts =>
    "zero " ++ " + ".intercalate (ts.map fun t =>
      s!"{t.coef.toNat}*[{",".intercalate (t.witnesses.map toString)}]")

/-- An ACIR function: its constraints, then its input and return witnesses. -/
def Circuit.render (f : Circuit) : List String :=
  let ws (l : List ℕ) := ",".intercalate (l.map toString)
  f.opcodes.map Opcode.render ++ [s!"inputs [{ws f.parameters}]", s!"returns [{ws f.returnValues}]"]

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

/-- The golden file: one `# <name> <width>` section per pinned width and
function, then the corpus. -/
def renderAll : String :=
  let sec (title : String) (lines : List String) := s!"# {title}\n" ++ "\n".intercalate lines
  let each (ws : List ℕ) (name : String) (lines : ℕ → List String) :=
    ws.map fun n => sec s!"{name} {n}" (lines n)
  let secs :=
    each pinnedWidths "div_var" (fun n => (divVarGadget n).map Opcode.render) ++
    each pinnedWidths "div_var_predicated" (fun n => (divPredGadget n).map Opcode.render) ++
    each pinnedWidths "truncate_field" (fun k => (truncateGadget k).map Opcode.render) ++
    each pinnedWidths "more_than_eq" (fun m => (moreThanEqGadget m).map Opcode.render) ++
    each pinnedWidths "signed_lt" (fun n => (signedLtSsa n).render) ++
    each pinnedWidths "acir_div" (fun n => (acirGenDiv n).render) ++
    each pinnedWidths "acir_lt" (fun n => (acirGenLt n).render) ++
    each pinnedWidths "acir_truncate" (fun n => (acirGenTruncate n).render) ++
    each pinnedWidths "acir_signed_lt" (fun n => (acirGenSignedLt n).render) ++
    each pinnedWidths "shipped_div" (fun n => (shippedDiv n).render) ++
    each pinnedWidths "shipped_lt" (fun n => (shippedLt n).render) ++
    each pinnedWidths "shipped_truncate" (fun n => (shippedTruncate n).render) ++
    each pinnedWidths "shipped_signed_lt" (fun n => (shippedSignedLt n).render) ++
    each signedWidths "shipped_signed_div" (fun n => (shippedSignedDiv n).render) ++
    each signedWidths "shipped_signed_mod" (fun n => (shippedSignedMod n).render) ++
    corpus.zipIdx.map (fun (e, i) => sec s!"corpus {i}" e.render)
  "\n".intercalate secs ++ "\n"

end AcirLean
