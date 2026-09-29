#!/usr/bin/env python3
"""Convert test-program dumps into Lean data for the scalar-program checker.

Usage: gen_programs.py <ssa-dir> <circuits.txt> <outside-out> <lean-out> <golden-out>

<ssa-dir> holds `<name>.ssa`, the `--show-ssa-pass` output of `nargo compile`;
<circuits.txt> is what the `dump_artifacts` test prints for the artifacts. For
every program whose final SSA is one ACIR `main` with one block of scalar
instructions, writes a `TestProgram` (its SSA and its shipped circuit) to
<lean-out> and the text Lean prints for it to <golden-out>. The other programs
are listed in <outside-out> with the reason. Run by `regen_programs.sh`.
Nothing here is trusted: `check.sh` requires Lean's printout of the data to
equal <golden-out>.

The programs covered are the ones `testProgramNames` in
`AcirLean/Spec/Coverage.lean` lists. With FV_PINNED set, rebuilds exactly those
and leaves <outside-out> and that list alone: CI uses this to check that their
SSA and circuits are unchanged without failing every time a test program is
added. It fails, writing nothing, if any of them no longer compiles or no
longer fits the subset. Without FV_PINNED, it takes every program in the subset
and rewrites `testProgramNames` to match, a change to the reviewed spec.
"""
import os
import re
import sys

TY = r"(Field|u\d+|i\d+)"

# Larger programs make the kernel check slow; they are listed as outside.
MAX_OPCODES = 250


def ty(t):
    if t == "Field":
        return ".field"
    return f".{'uint' if t[0] == 'u' else 'sint'} {t[1:]}"


def lean_list(items):
    """A Lean list literal, split into chunks so long lists elaborate."""
    items = list(items)
    if len(items) <= 64:
        return "[" + ", ".join(items) + "]"
    chunks = [items[i:i + 64] for i in range(0, len(items), 64)]
    return "(" + " ++ ".join("[" + ", ".join(c) + "]" for c in chunks) + ")"


def split_top(s):
    """Split on commas outside brackets and parentheses."""
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "[(":
            depth += 1
        elif ch in "])":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def pty(t):
    t = t.strip()
    if re.fullmatch(TY, t):
        return f"(.scalar ({ty(t)}))"
    m = re.fullmatch(r"\[(.*); (\d+)\]", t)
    if not m:
        raise ValueError(f"type {t}")
    elems = m.group(1)
    if elems.startswith("("):
        if not elems.endswith(")"):
            raise ValueError(f"type {t}")
        elems = split_top(elems[1:-1])
    else:
        elems = [elems]
    for e in elems:
        if not re.fullmatch(TY, e):
            raise ValueError(f"type {t}")
    return f"(.array [{', '.join(ty(e) for e in elems)}] {m.group(2)})"


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
    m = re.fullmatch(r"v(\d+) = array_get ([^,]+), index (.+) -> " + TY, l)
    if m:
        return (f".arrayGet {m.group(1)} ({opnd(m.group(2))}) ({opnd(m.group(3))}) "
                f"({ty(m.group(4))})")
    m = re.fullmatch(r"v(\d+) = array_set (mut )?([^,]+), index ([^,]+), value (.+)", l)
    if m:
        u = "true" if m.group(2) else "false"
        return (f".arraySet {m.group(1)} {u} ({opnd(m.group(3))}) ({opnd(m.group(4))}) "
                f"({opnd(m.group(5))})")
    m = re.fullmatch(r"v(\d+) = make_array \[(.*)\] : (.+)", l)
    if m:
        es = [f"({opnd(e)})" for e in split_top(m.group(2))]
        return f".makeArray {m.group(1)} {lean_list(es)} {pty(m.group(3))}"
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
    for p in split_top(m.group(1)):
        pm = re.fullmatch(r"v(\d+): (.+)", p)
        if not pm:
            raise ValueError(f"parameter {p}")
        try:
            params.append(f"({pm.group(1)}, {pty(pm.group(2))})")
        except ValueError:
            raise ValueError(f"parameter {p}")
    if lines[-1] != "}" or not lines[-2].startswith("    return"):
        raise ValueError("more than one block")
    if len(lines) > MAX_OPCODES:
        raise ValueError(f"more than {MAX_OPCODES} instructions")
    body = [instr(l) for l in lines[2:-2]]
    rets = lines[-2][len("    return"):].strip()
    rets = [opnd(r) for r in split_top(rets)] if rets else []
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
    if len(lines) > MAX_OPCODES + 2:
        raise ValueError(f"more than {MAX_OPCODES} opcodes")
    cs = []
    for line in lines:
        if line.startswith("range "):
            _, w, k = line.split()
            cs.append(f".range {w} {k}")
        elif line.startswith("zero "):
            terms = [t.split("*") for t in line[5:].split(" + ")] if line != "zero " else []
            cs.append(".assertZero [" + ", ".join(f"⟨{c}, {ws}⟩" for c, ws in terms) + "]")
        elif line.startswith("inputs "):
            inputs = line[7:]
        elif line.startswith("returns "):
            returns = line[8:]
        else:
            raise ValueError(f"opcode {line[:40]}")
    return f"{{ opcodes := {lean_list(cs)}, parameters := {inputs}, returnValues := {returns} }}"


COVERAGE = "AcirLean/Spec/Coverage.lean"
NAMES = re.compile(r"(def testProgramNames : List String :=\n  \[)(.*?)(\])", re.S)


def covered_names():
    return re.findall(r'"([^"]+)"', NAMES.search(open(COVERAGE).read()).group(2))


def write_covered_names(names):
    text = open(COVERAGE).read()
    body = ",\n   ".join(f'"{n}"' for n in names)
    open(COVERAGE, "w").write(NAMES.sub(lambda m: m.group(1) + body + m.group(3), text))


def main():
    ssa_dir, circ_path, names_out, lean_out, expected_out = sys.argv[1:6]
    circs = circuits(circ_path)
    pinned = covered_names() if os.environ.get("FV_PINNED") else None
    entries, skipped, expected, names = [], [], [], []
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
            f"def prog{idx} : TestProgram where\n"
            f"  name := \"{name}\"\n"
            f"  prog := {{ header := \"{header}\", params := [{', '.join(params)}], body := {lean_list(body)}, rets := [{', '.join(f'({r})' for r in rets)}] }}\n"
            f"  fn := {fn}\n"
        )
        names.append(name)
        expected.append(f"# program {name}")
        expected.extend(lines)
        expected.extend(circs[name])
    if pinned is not None:
        lost = [f"{n}: did not compile" for n in pinned if n not in circs] + skipped
        if lost:
            for line in lost:
                print(f"covered program could not be rebuilt: {line}", file=sys.stderr)
            print(f"Nothing written. Removing a program from the claims is a reviewed change: "
                  f"take it out of `testProgramNames` in {COVERAGE}.", file=sys.stderr)
            sys.exit(1)
    with open(lean_out, "w") as f:
        f.write("/-\nPINNED: no review needed. Generated by `scripts/gen_programs.py` from the\n"
                "final SSA and the shipped circuits of `test_programs/execution_success`;\n"
                "`scripts/check.sh` requires Lean's printout of this data to equal\n"
                "`test_programs.golden`, and allows only plain definitions here.\n-/\n\n"
                "import AcirLean.Spec.Programs2\n\nnamespace AcirLean\n\n")
        f.write("\n".join(entries))
        f.write("\ndef testPrograms : List TestProgram := [" +
                ", ".join(f"prog{i}" for i in range(len(entries))) + "]\n\nend AcirLean\n")
    open(expected_out, "w").write("\n".join(expected) + "\n")
    if pinned is None:
        write_covered_names(names)
        open(names_out, "w").write("\n".join(skipped) + "\n")
        print(f"{len(entries)} programs in the subset, {len(skipped)} outside it")
    else:
        print(f"all {len(entries)} covered programs rebuilt")

main()
