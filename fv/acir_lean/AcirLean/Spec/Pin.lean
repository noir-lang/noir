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

/-!
# The pin

`renderAll` prints the constraint lists for every width in `pinnedWidths`, in
the canonical text form that `fv_templates.rs` also prints the Rust gadgets'
output in, followed by the pinned SSA in `Ssa`'s own `Display` syntax. `scripts/check.sh` fails unless this printout equals
`templates.golden`, and the Rust test fails unless the gadgets' printout does.

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
    let ts := ts.map (fun t => { t with ws := t.ws.mergeSort (· ≤ ·) })
    let ts := (ts.filter (fun t => fmod t.coef ≠ 0)).mergeSort (fun a b => listLe a.ws b.ws)
    let neg : Bool := match ts with
      | t :: _ => decide (fmod t.coef > (p - 1) / 2)
      | [] => false
    let body := ts.map fun t =>
      let c := if neg then fmod (-t.coef) else fmod t.coef
      s!"{c}*[{",".intercalate (t.ws.map toString)}]"
    "zero " ++ " + ".intercalate body

/-- An ACIR function: its constraints, then its input and return witnesses. -/
def AcirFn.render (f : AcirFn) : List String :=
  let ws (l : List ℕ) := ",".intercalate (l.map toString)
  f.cs.map Cstr.render ++ [s!"inputs [{ws f.inputs}]", s!"returns [{ws f.returns}]"]

/-- The golden file: one `# <gadget> <width>` section per pinned width. -/
def renderAll : String :=
  let sec (title : String) (cs : List Cstr) :=
    s!"# {title}\n" ++ "\n".intercalate (cs.map Cstr.render)
  let secs := pinnedWidths.map (fun n => sec s!"div_var {n}" (divVarT n)) ++
    pinnedWidths.map (fun n => sec s!"div_var_predicated {n}" (divPredT n)) ++
    pinnedWidths.map (fun k => sec s!"truncate_field {k}" (truncT k)) ++
    pinnedWidths.map (fun m => sec s!"more_than_eq {m}" (moreThanEqT m)) ++
    pinnedWidths.map (fun n => s!"# signed_lt {n}\n" ++ "\n".intercalate (signedLtT n).render) ++
    pinnedWidths.map (fun n => s!"# acir_div {n}\n" ++ "\n".intercalate (acirDivT n).render) ++
    pinnedWidths.map (fun n => s!"# acir_lt {n}\n" ++ "\n".intercalate (acirLtT n).render) ++
    pinnedWidths.map (fun n =>
      s!"# acir_truncate {n}\n" ++ "\n".intercalate (acirTruncT n).render) ++
    pinnedWidths.map (fun n =>
      s!"# acir_signed_lt {n}\n" ++ "\n".intercalate (acirSignedLtT n).render) ++
    pinnedWidths.map (fun n => s!"# shipped_div {n}\n" ++ "\n".intercalate (shippedDivT n).render) ++
    pinnedWidths.map (fun n => s!"# shipped_lt {n}\n" ++ "\n".intercalate (shippedLtT n).render) ++
    pinnedWidths.map (fun n =>
      s!"# shipped_truncate {n}\n" ++ "\n".intercalate (shippedTruncT n).render) ++
    pinnedWidths.map (fun n =>
      s!"# shipped_signed_lt {n}\n" ++ "\n".intercalate (shippedSignedLtT n).render) ++
    signedWidths.map (fun n =>
      s!"# shipped_signed_div {n}\n" ++ "\n".intercalate (shippedSDivT n).render) ++
    signedWidths.map (fun n =>
      s!"# shipped_signed_mod {n}\n" ++ "\n".intercalate (shippedSModT n).render)
  "\n".intercalate secs ++ "\n"

end AcirLean
