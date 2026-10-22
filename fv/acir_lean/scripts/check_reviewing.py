#!/usr/bin/env python3
"""Keeps `REVIEWING.md` in step with the reviewed Lean.

Fails if a line inside a ```lean block of `REVIEWING.md` does not appear,
verbatim up to indentation, in `AcirLean/Spec/` or `Check.lean` (lines holding
`…` are abbreviations and are skipped), or if a definition in those files is
never mentioned.
"""
import glob
import re
import sys

doc = open("REVIEWING.md").read()
src = "".join(open(f).read() + "\n" for f in sorted(glob.glob("AcirLean/Spec/*.lean")) + ["Check.lean"])
lines = {l.strip() for l in src.splitlines()}

stale = [l.strip() for block in re.findall(r"```lean\n(.*?)```", doc, re.S)
         for l in block.splitlines() if l.strip() and "…" not in l and l.strip() not in lines]
names = sorted(set(re.findall(r"^(?:def|structure|inductive|abbrev) ([A-Za-z0-9_.']+)", src, re.M)))
missing = [n for n in names if not re.search(r"(?<![\w.])" + re.escape(n) + r"(?![\w'])", doc)]

for l in stale:
    print(f"REVIEWING.md quotes a line that is not in the reviewed Lean: {l}", file=sys.stderr)
for n in missing:
    print(f"REVIEWING.md does not mention the reviewed definition `{n}`", file=sys.stderr)
if stale or missing:
    print("Update REVIEWING.md to match AcirLean/Spec/ and Check.lean.", file=sys.stderr)
    sys.exit(1)
