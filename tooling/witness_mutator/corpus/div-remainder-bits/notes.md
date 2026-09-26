# div-remainder-bits

**Origin:** `a0f05179e5d` — "fix: address off-by-one error when calculating bitsize of remainder" (#10721)

**Label: W-candidate** — the circuit is measurably weaker, but the historical exploit is masked on
current `master`. See "status" below.

**Defect.** The remainder's bit size was taken from `rhs.num_bits()` instead of
`(rhs - 1).num_bits()`. For a power-of-two `rhs = 2^k` that is `k + 1` bits rather than `k`, so `r`
is range-constrained one bit too loosely, and the same wrong bit size was passed to the
`r < p - q0*b` bound.

**Patch.** Restores `rhs_bits` in both places.

**Trigger.** `program/`: `x as u128`, with `x = 0` first (the input used by the fix's own test)
followed by the same 20 random inputs.

**Status.** With the patch applied the circuit does change — `truncate_field_to_128_bits` shows the
range checks on `w3`/`w4` widening from 128 to 129 bits — but
`properly_constrains_quotient_when_truncating_fields_to_u128` still passes, i.e. the specific
malicious `(q, r)` from the original fix is now rejected by the `q <= q0` bound that
`47ba6138f81` added later. Whether a *different* second witness survives is an open question, and
one the tool is meant to answer. Keep this entry, and score it as "unknown expected verdict": a
finding here is interesting, and a miss is not evidence of a weak tool.
