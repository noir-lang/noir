/-
REVIEWED: this file is part of the trusted specification (`AcirLean/Spec/`).
Every definition here is taken on trust: read it against its comment. A change
to this directory needs review from the owners listed in CODEOWNERS.
-/

import AcirLean.Spec.Semantics
import AcirLean.Templates.Gadgets

/-!
# The pin

`renderAll` prints the constraint lists for every width in `pinnedWidths`, in
the canonical text form that `fv_templates.rs` also prints the Rust gadgets'
output in. `scripts/check.sh` fails unless this printout equals
`templates.golden`, and the Rust test fails unless the gadgets' printout does.

Canonical form: `zero c*[i,j] + c*[i] + c*[]` with terms sorted by witness
list, zero terms dropped, and the sign chosen so the first coefficient is at
most `(p-1)/2`; or `range i k`.
-/

namespace AcirLean

/-- The widths whose constraint lists are pinned to the Rust output. The claims
are stated for exactly these widths. -/
def pinnedWidths : List ℕ := [8, 16, 32, 64]

/-- Lexicographic order on witness lists (the order `Vec<u32>` sorts in Rust). -/
def listLe : List ℕ → List ℕ → Bool
  | [], _ => true
  | _ :: _, [] => false
  | a :: as, b :: bs => if a < b then true else if b < a then false else listLe as bs

/-- An integer coefficient as a field element's integer value, in `[0, p)`. -/
def fmod (c : ℤ) : ℕ := (c % (p : ℤ)).toNat

/-- One constraint in canonical form (see the module comment). -/
def Cstr.render : Cstr → String
  | .range w k => s!"range {w} {k}"
  | .zero ts =>
    let ts := (ts.filter (fun t => fmod t.coef ≠ 0)).mergeSort (fun a b => listLe a.ws b.ws)
    let neg : Bool := match ts with
      | t :: _ => decide (fmod t.coef > (p - 1) / 2)
      | [] => false
    let body := ts.map fun t =>
      let c := if neg then fmod (-t.coef) else fmod t.coef
      s!"{c}*[{",".intercalate (t.ws.map toString)}]"
    "zero " ++ " + ".intercalate body

/-- The golden file: one `# <gadget> <width>` section per pinned width. -/
def renderAll : String :=
  let sec (title : String) (cs : List Cstr) :=
    s!"# {title}\n" ++ "\n".intercalate (cs.map Cstr.render)
  let secs := pinnedWidths.map (fun n => sec s!"div_var {n}" (divVarT n)) ++
    pinnedWidths.map (fun k => sec s!"truncate_field {k}" (truncT k))
  "\n".intercalate secs ++ "\n"

end AcirLean
