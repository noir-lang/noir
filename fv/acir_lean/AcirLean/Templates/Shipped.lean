/-
PINNED: no review needed. These functions are checked byte-for-byte against
the optimized circuits `nargo compile` produces (`acvm::compiler::optimize`
after ACIR generation), for every width in `Spec.Pin.pinnedWidths`
(`templates.golden`, `fv_templates.rs`). `scripts/check.sh` allows only plain
definitions in this directory.
-/

import AcirLean.Templates.Programs

/-!
The shipped circuits are the ACIR-generation output with repeated constraints
removed. For a truncation whose remainder guard is a zero-width range check,
the optimizer also inlines the guard's intermediate witness `u = (r + R') * y`
into its `u = 0` check.
-/

namespace AcirLean

/-- Keep the first occurrence of each constraint. -/
def dropRepeats : List Constraint → List Constraint
  | [] => []
  | c :: cs => c :: (dropRepeats cs).filter (fun d => d != c)

def shippedDiv (n : ℕ) : AcirFunction := { acirGenDiv n with constraints := dropRepeats (acirGenDiv n).constraints }

def shippedLt (n : ℕ) : AcirFunction := { acirGenLt n with constraints := dropRepeats (acirGenLt n).constraints }

def shippedSignedLt (n : ℕ) : AcirFunction :=
  { acirGenSignedLt n with constraints := dropRepeats (acirGenSignedLt n).constraints }

/-- `acirGenTruncate n` for `n < 128` with `N' n = 0`: repeats dropped, and the guard
`r*y + R'*y - u = 0, u = 0` inlined to `r*y + R'*y = 0`. -/
def truncInlinedGadget (n : ℕ) : List Constraint :=
  [ .range 2 (254 - n), .range 3 n,
    .zero [⟨1, [0]⟩, ⟨-(2 ^ n : ℕ), [2]⟩, ⟨-1, [3]⟩],
    .zero [⟨(q0 n : ℤ), []⟩, ⟨-1, [2]⟩, ⟨-1, [4]⟩],
    .range 4 (254 - n),
    .zero [⟨1, []⟩, ⟨1, [2, 5]⟩, ⟨-(q0 n : ℤ), [5]⟩, ⟨-1, [6]⟩],
    .zero [⟨1, [2, 6]⟩, ⟨-(q0 n : ℤ), [6]⟩],
    .zero [⟨1, [3, 6]⟩, ⟨(R' n : ℤ), [6]⟩],
    .zero [⟨1, [1]⟩, ⟨-1, [3]⟩] ]

def shippedTruncate (n : ℕ) : AcirFunction :=
  { acirGenTruncate n with
    constraints := if n ≠ 128 ∧ N' n = 0 then truncInlinedGadget n else dropRepeats (acirGenTruncate n).constraints }

end AcirLean
