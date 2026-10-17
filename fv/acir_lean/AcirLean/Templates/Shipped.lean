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
def dropRepeats : List Opcode → List Opcode
  | [] => []
  | c :: cs => c :: (dropRepeats cs).filter (fun d => d != c)

def shippedDiv (n : ℕ) : Circuit := { acirGenDiv n with opcodes := dropRepeats (acirGenDiv n).opcodes }

def shippedLt (n : ℕ) : Circuit := { acirGenLt n with opcodes := dropRepeats (acirGenLt n).opcodes }

def shippedSignedLt (n : ℕ) : Circuit :=
  { acirGenSignedLt n with opcodes := dropRepeats (acirGenSignedLt n).opcodes }

/-- `acirGenTruncate n` for `n < 128` with `N' n = 0`: repeats dropped, and the guard
`r*y + R'*y - u = 0, u = 0` inlined to `r*y + R'*y = 0`. -/
def truncInlinedGadget (n : ℕ) : List Opcode :=
  [ .range 2 (254 - n), .range 3 n,
    .assertZero [⟨1, [0]⟩, ⟨-(2 ^ n : ℕ), [2]⟩, ⟨-1, [3]⟩],
    .assertZero [⟨(q0 n : ℤ), []⟩, ⟨-1, [2]⟩, ⟨-1, [4]⟩],
    .range 4 (254 - n),
    .assertZero [⟨1, []⟩, ⟨1, [2, 5]⟩, ⟨-(q0 n : ℤ), [5]⟩, ⟨-1, [6]⟩],
    .assertZero [⟨1, [2, 6]⟩, ⟨-(q0 n : ℤ), [6]⟩],
    .assertZero [⟨1, [3, 6]⟩, ⟨(R' n : ℤ), [6]⟩],
    .assertZero [⟨1, [1]⟩, ⟨-1, [3]⟩] ]

def shippedTruncate (n : ℕ) : Circuit :=
  { acirGenTruncate n with
    opcodes := if n ≠ 128 ∧ N' n = 0 then truncInlinedGadget n else dropRepeats (acirGenTruncate n).opcodes }

end AcirLean
