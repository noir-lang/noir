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
def dropRepeats : List Cstr → List Cstr
  | [] => []
  | c :: cs => c :: (dropRepeats cs).filter (fun d => d != c)

def shippedDivT (n : ℕ) : AcirFn := { acirDivT n with cs := dropRepeats (acirDivT n).cs }

def shippedLtT (n : ℕ) : AcirFn := { acirLtT n with cs := dropRepeats (acirLtT n).cs }

def shippedSignedLtT (n : ℕ) : AcirFn :=
  { acirSignedLtT n with cs := dropRepeats (acirSignedLtT n).cs }

/-- `acirTruncT n` for `n < 128` with `N' n = 0`: repeats dropped, and the guard
`r*y + R'*y - u = 0, u = 0` inlined to `r*y + R'*y = 0`. -/
def truncInlinedT (n : ℕ) : List Cstr :=
  [ .range 2 (254 - n), .range 3 n,
    .zero [⟨1, [0]⟩, ⟨-(2 ^ n : ℕ), [2]⟩, ⟨-1, [3]⟩],
    .zero [⟨(q0 n : ℤ), []⟩, ⟨-1, [2]⟩, ⟨-1, [4]⟩],
    .range 4 (254 - n),
    .zero [⟨1, []⟩, ⟨1, [2, 5]⟩, ⟨-(q0 n : ℤ), [5]⟩, ⟨-1, [6]⟩],
    .zero [⟨1, [2, 6]⟩, ⟨-(q0 n : ℤ), [6]⟩],
    .zero [⟨1, [3, 6]⟩, ⟨(R' n : ℤ), [6]⟩],
    .zero [⟨1, [1]⟩, ⟨-1, [3]⟩] ]

def shippedTruncT (n : ℕ) : AcirFn :=
  { acirTruncT n with
    cs := if n ≠ 128 ∧ N' n = 0 then truncInlinedT n else dropRepeats (acirTruncT n).cs }

end AcirLean
