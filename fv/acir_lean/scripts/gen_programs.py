#!/usr/bin/env python3
"""Convert test-program dumps into Lean data for the scalar-program checker.

Usage: gen_programs.py <ssa-dir> <circuits.txt> <outside-out> <lean-out> <golden-out>

<ssa-dir> holds `<name>.ssa`, the `--show-ssa-pass` output of `nargo compile`;
<circuits.txt> is what the `dump_artifacts` test prints for the artifacts. For
every program whose final SSA is one ACIR `main` with one block of scalar
instructions, writes a `ProgEntry` (its SSA and its shipped circuit) to
<lean-out> and the text Lean prints for it to <golden-out>. The other programs
are listed in <outside-out> with the reason. Run by `regen_programs.sh`.
Nothing here is trusted: `check.sh` requires Lean's printout of the data to
equal <golden-out>.

With FV_PINNED set, keeps only the programs <golden-out> already lists and
leaves <outside-out> alone: CI uses this to check that the pinned programs'
SSA and circuits are unchanged without failing every time a test program is
added.
"""
import os
import re
import sys

TY = r"(Field|u\d+|i\d+)"


def ty(t):
    if t == "Field":
        return ".field"
    return f".{'uint' if t[0] == 'u' else 'sint'} {t[1:]}"


def opnd(s):
    s = s.strip()
    m = re.fullmatch(r"v(\d+)", s)
    if m:
        return f".var {m.group(1)}"
    m = re.fullmatch(TY + r" (-?\d+)", s)
    if m and not m.group(2).startswith("-"):
        return f".const {m.group(2)} ({ty(m.group(1))})"
    raise ValueError(f"operand {s}")


def msg(rest):
    rest = rest.strip()
    if not rest:
        return "none"
    m = re.fullmatch(r', "(.*)"', rest)
    if not m or '"' in m.group(1) or "\\" in m.group(1):
        raise ValueError(f"message {rest}")
    return f'(some "{m.group(1)}")'


def instr(line):
    l = line.strip()
    m = re.fullmatch(r"v(\d+) = (unchecked_)?(add|sub|mul|div|mod|lt|eq) ([^,]+), (.+)", l)
    if m:
        u = "true" if m.group(2) else "false"
        return f".bin {m.group(1)} .{m.group(3)} {u} ({opnd(m.group(4))}) ({opnd(m.group(5))})"
    m = re.fullmatch(r"v(\d+) = not (.+)", l)
    if m:
        return f".not {m.group(1)} ({opnd(m.group(2))})"
    m = re.fullmatch(r"v(\d+) = cast (.+) as " + TY, l)
    if m:
        return f".cast {m.group(1)} ({opnd(m.group(2))}) ({ty(m.group(3))})"
    m = re.fullmatch(r"v(\d+) = truncate (.+) to (\d+) bits, max_bit_size: (\d+)", l)
    if m:
        return f".truncate {m.group(1)} ({opnd(m.group(2))}) {m.group(3)} {m.group(4)}"
    m = re.fullmatch(r"constrain ([^=]+) == ([^,]+)(.*)", l)
    if m:
        return f".constrain ({opnd(m.group(1))}) ({opnd(m.group(2))}) {msg(m.group(3))}"
    m = re.fullmatch(r"range_check (.+) to (\d+) bits(.*)", l)
    if m:
        return f".rangeCheck ({opnd(m.group(1))}) {m.group(2)} {msg(m.group(3))}"
    raise ValueError(f"instruction {l}")


def main_fn(text):
    text = text[text.index("last step"):]
    blocks = re.split(r"\n(?=(?:acir|brillig)\()", text)
    acir = [b for b in blocks if b.startswith("acir(")]
    if len(acir) != 1:
        raise ValueError("not exactly one ACIR function")
    body = acir[0].strip().splitlines()
    return body[: body.index("}") + 1]


def program(lines):
    header = lines[0]
    if not re.fullmatch(r"acir\(inline\) (\w+ )?fn main f0 \{", header):
        raise ValueError("header")
    m = re.fullmatch(r"  b0\((.*)\):", lines[1])
    if not m:
        raise ValueError("block header")
    params = []
    for p in filter(None, [x.strip() for x in m.group(1).split(",")]):
        pm = re.fullmatch(r"v(\d+): " + TY, p)
        if not pm:
            raise ValueError(f"parameter {p}")
        params.append(f"({pm.group(1)}, {ty(pm.group(2))})")
    if lines[-1] != "}" or not lines[-2].startswith("    return"):
        raise ValueError("more than one block")
    body = [instr(l) for l in lines[2:-2]]
    rets = lines[-2][len("    return"):].strip()
    rets = [opnd(r) for r in rets.split(",")] if rets else []
    return header, params, body, rets


def circuits(path):
    out, cur, name = {}, [], None
    for line in open(path).read().splitlines():
        if line.startswith("# artifact "):
            if name:
                out[name] = cur
            name = line.split("/")[-1][: -len(".json")]
            cur = []
        else:
            cur.append(line)
    if name:
        out[name] = cur
    return out


def lean_fn(lines):
    cs = []
    for line in lines:
        if line.startswith("range "):
            _, w, k = line.split()
            cs.append(f".range {w} {k}")
        elif line.startswith("zero "):
            terms = [t.split("*") for t in line[5:].split(" + ")] if line != "zero " else []
            cs.append(".zero [" + ", ".join(f"⟨{c}, {ws}⟩" for c, ws in terms) + "]")
        elif line.startswith("inputs "):
            inputs = line[7:]
        elif line.startswith("returns "):
            returns = line[8:]
        else:
            raise ValueError(f"opcode {line[:40]}")
    return f"{{ cs := [{', '.join(cs)}], inputs := {inputs}, returns := {returns} }}"


def main():
    ssa_dir, circ_path, names_out, lean_out, expected_out = sys.argv[1:6]
    circs = circuits(circ_path)
    pinned = None
    if os.environ.get("FV_PINNED"):
        pinned = [l[len("# program "):] for l in open(expected_out).read().splitlines()
                  if l.startswith("# program ")]
    entries, skipped, expected = [], [], []
    for name in sorted(circs):
        if pinned is not None and name not in pinned:
            continue
        try:
            lines = main_fn(open(os.path.join(ssa_dir, name + ".ssa")).read())
            header, params, body, rets = program(lines)
            fn = lean_fn(circs[name])
        except ValueError as e:
            skipped.append(f"{name}: {e}")
            continue
        idx = len(entries)
        entries.append(
            f"def prog{idx} : ProgEntry where\n"
            f"  name := \"{name}\"\n"
            f"  prog := {{ header := \"{header}\", params := [{', '.join(params)}], body := [{', '.join(body)}], rets := [{', '.join(f'({r})' for r in rets)}] }}\n"
            f"  fn := {fn}\n"
        )
        expected.append(f"# program {name}")
        expected.extend(lines)
        expected.extend(circs[name])
    with open(lean_out, "w") as f:
        f.write("/-\nPINNED: no review needed. Generated by `scripts/gen_programs.py` from the\n"
                "final SSA and the shipped circuits of `test_programs/execution_success`;\n"
                "`scripts/check.sh` requires Lean's printout of this data to equal\n"
                "`test_programs.golden`, and allows only plain definitions here.\n-/\n\n"
                "import AcirLean.Spec.Programs2\n\nnamespace AcirLean\n\n")
        f.write("\n".join(entries))
        f.write("\ndef testPrograms : List ProgEntry := [" +
                ", ".join(f"prog{i}" for i in range(len(entries))) + "]\n\nend AcirLean\n")
    open(expected_out, "w").write("\n".join(expected) + "\n")
    if pinned is None:
        open(names_out, "w").write("\n".join(skipped) + "\n")
        print(f"{len(entries)} programs in the subset, {len(skipped)} outside it")
    else:
        print(f"{len(entries)} of the {len(pinned)} pinned programs rebuilt")
        for line in skipped:
            print(f"pinned program left the subset: {line}")


main()
