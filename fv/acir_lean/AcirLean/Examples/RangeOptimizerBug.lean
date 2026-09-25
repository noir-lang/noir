/-
EXAMPLE: not part of the claims, no review needed. Lean checks it.
-/

import AcirLean.Proofs.Shipped

/-!
A range-optimizer bug the shipped-circuit claims rule out. `collect_ranges`
reads an `AssertZero` of the form `k·w + c = 0` as the range `w ≤ -c/k`. If
the check that the expression has a single witness were lost, the division
circuit's `1 - b + r + t = 0` would read as "`b` is 1", and the optimizer would
drop `b`'s 8-bit range check as redundant. `buggyShippedDiv8` is exactly
the circuit that mutation produces for `div` on `u8`.

The quotient is still arithmetically right (`44 / 300 = 0`), but `b = 300`
is accepted as a `u8`, which breaks every later use of `b`.
-/

namespace AcirLean

def buggyShippedDiv8 : AcirFunction :=
  { shippedDiv 8 with constraints := (shippedDiv 8).constraints.filter (· != .range 1 8) }

/-- `a = 44`, `b = 300`, quotient `0`, remainder `44`, `t = b - r - 1 = 255`. -/
def forgedDiv : ℕ → F
  | 0 => 44
  | 1 => 300
  | 3 => (3429158049921486451485270233423639763872577089398512047179385322563543330980 : ℕ)
  | 5 => 44
  | 6 => 255
  | _ => 0

theorem forgedDiv_satisfies : AllHold forgedDiv buggyShippedDiv8.constraints := by decide +kernel

theorem buggy_range_optimizer_not_sound :
    ¬ SoundFunction buggyShippedDiv8 (Computes2 8 (SsaBinOp.eval .div)) := by
  intro h
  have hs := h forgedDiv forgedDiv_satisfies
  simp only [buggyShippedDiv8, shippedDiv, acirGenDiv, List.map, Computes2] at hs
  have hb : (forgedDiv 1).val = 300 := by decide +kernel
  have := hs.2.1
  omega

end AcirLean
