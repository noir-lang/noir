#!/usr/bin/env python3
"""REVIEWED: part of the trusted base. `AcirLean/Templates/` needs no review
only because it holds plain data: `def`s and `abbrev`s of pinned constraint
lists, circuits and programs, which the golden files compare byte-for-byte with
the compiler's output. This script fails unless every file there is exactly
that.

With comments and string literals removed, a file may contain no keyword that
declares anything other than a definition or changes how Lean elaborates
(`instance`, `theorem`, `@[...]`, `set_option`, `open`, `macro`, ...), and every
definition's name must be a plain identifier or `Term.<name>`/`Opcode.<name>`.
A dotted name such as `Int.tdiv` would declare `AcirLean.Int.tdiv`, which Lean
resolves before the root `Int.tdiv` wherever the claims are written inside
`namespace AcirLean`.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
TEMPLATES = ROOT / "AcirLean" / "Templates"

FORBIDDEN = re.compile(
    r"@\[|#[a-z_]+|\b(instance|theorem|lemma|example|attribute|macro|macro_rules|syntax|"
    r"elab|elab_rules|notation|infix|infixl|infixr|prefix|postfix|set_option|open|export|"
    r"opaque|axiom|partial|unsafe|private|protected|noncomputable|local|scoped|initialize|"
    r"builtin_initialize|variable|universe|section|mutual|class|structure|inductive|"
    r"run_cmd|run_elab|run_meta|unif_hint|deriving|_root_)\b"
)
DECL = re.compile(r"\b(def|abbrev)\s+([^\s(:]+)")
NAME = re.compile(r"^((Term|Opcode)\.)?[A-Za-z_][A-Za-z0-9_']*$")


def strip(src: str) -> str:
    """`src` with comments and string literals replaced by spaces."""
    out = []
    i, depth, n = 0, 0, len(src)
    while i < n:
        if src.startswith("/-", i):
            depth += 1
            i += 2
        elif depth and src.startswith("-/", i):
            depth -= 1
            i += 2
        elif depth:
            out.append("\n" if src[i] == "\n" else " ")
            i += 1
        elif src.startswith("--", i):
            while i < n and src[i] != "\n":
                i += 1
        elif src[i] == '"':
            i += 1
            while i < n and src[i] != '"':
                i += 2 if src[i] == "\\" else 1
            i += 1
            out.append('""')
        else:
            out.append(src[i])
            i += 1
    return "".join(out)


def main() -> int:
    errors = []
    for path in sorted(TEMPLATES.glob("*.lean")):
        code = strip(path.read_text())
        for lineno, line in enumerate(code.splitlines(), 1):
            if line.startswith("import "):
                continue
            m = FORBIDDEN.search(line)
            if m:
                errors.append(f"{path.relative_to(ROOT)}:{lineno}: `{m.group(0)}`")
            for d in DECL.finditer(line):
                if not NAME.match(d.group(2)):
                    errors.append(f"{path.relative_to(ROOT)}:{lineno}: definition name `{d.group(2)}`")
    if errors:
        print("AcirLean/Templates may only contain plain definitions:", file=sys.stderr)
        print("\n".join(errors), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
