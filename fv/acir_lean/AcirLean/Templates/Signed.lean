/-
PINNED: no review needed. This SSA is checked byte-for-byte against what
`expand_signed_math` produces, for every width in `Spec.Pin.pinnedWidths`
(`templates.golden`, `fv_templates.rs`). `scripts/check.sh` allows only plain
definitions in this directory.
-/

import AcirLean.Spec.Ssa

namespace AcirLean

/-- `expand_signed_math` applied to `v2 = lt v0, v1` on `i<n>` operands: compare
the bit patterns as unsigned, and flip the result when the signs differ. -/
def signedLtT (n : ℕ) : SsaFn where
  params := [(0, .i n), (1, .i n)]
  body :=
    [ .cast 4 0 (.u n), .cast 5 1 (.u n),
      .bin 7 .div (.var 4) (.const (.u n) (2 ^ (n - 1))), .cast 8 7 (.u 1),
      .bin 9 .div (.var 5) (.const (.u n) (2 ^ (n - 1))), .cast 10 9 (.u 1),
      .bin 11 .xor (.var 8) (.var 10),
      .bin 12 .lt (.var 4) (.var 5),
      .bin 13 .xor (.var 11) (.var 12) ]
  ret := 13

end AcirLean
