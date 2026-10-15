#!/usr/bin/env python3
"""Compile and run the code examples in the standard library's doc comments.

Every fenced block tagged `noir` inside a `///` or `//!` comment under
`noir_stdlib/src` becomes a `#[test]` in a scratch crate, which is then run with
`nargo test`. Blocks tagged with anything else (untagged, `rust`, `text`, ...)
are documentation only and are not checked.

A block's info string can carry comma-separated attributes after `noir`:

    ```noir                  run as a test that must pass
    ```noir,should_fail      run as `#[test(should_fail)]`: the example must fail an assertion
    ```noir,ignore           not compiled (e.g. it needs an oracle or a `main` with inputs)

How a block becomes a test:

- `use` lines are kept at the top of the example's module.
- If the block declares top-level items (`fn`, `struct`, `impl`, `trait`,
  `global`, attributes, ...) it is placed in the module as written. A
  `fn main()` without parameters, if present, is called from the test;
  otherwise the items are only compiled.
- Otherwise the block is the body of the test function.

Each example gets its own module, named after its file and line, so examples
cannot see each other's items and a failing test points at its source.

The examples run against the standard library embedded in the `nargo` binary,
so rebuild nargo after editing the stdlib (`cargo build -p nargo_cli`).

Usage: scripts/check_stdlib_doc_examples.py [--nargo PATH] [--keep DIR] [--list]
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STDLIB_SRC = os.path.join(REPO_ROOT, "noir_stdlib", "src")

DOC_LINE = re.compile(r"^\s*//[/!](?: ?)(.*)$")
FENCE = re.compile(r"^(`{3,})\s*(.*)$")
TOP_LEVEL_ITEM = re.compile(
    r"^(pub(\([a-z]+\))?\s+)?(unconstrained\s+|comptime\s+)*"
    r"(fn|struct|enum|impl|trait|global|mod|type)\b|^#\["
)
MAIN_FN = re.compile(r"^(unconstrained\s+)?fn\s+main\s*\(\s*\)")


@dataclass
class Example:
    path: str  # relative to noir_stdlib/src
    line: int  # 1-based line of the opening fence
    attributes: list
    code: list

    @property
    def module_name(self):
        stem = re.sub(r"[^a-z0-9]+", "_", self.path.lower().removesuffix(".nr"))
        return f"{stem}_l{self.line}"


def extract_examples(path, rel_path):
    examples = []
    fence = None  # (backticks, attributes, start line, code lines) while inside a block
    with open(path, encoding="utf-8") as f:
        for number, raw in enumerate(f, start=1):
            doc = DOC_LINE.match(raw)
            if doc is None:
                if fence is not None:
                    raise SystemExit(
                        f"{rel_path}:{fence[2]}: code block is not closed before the doc comment ends"
                    )
                continue
            text = doc.group(1)
            opening = FENCE.match(text.strip())
            if fence is None:
                if opening:
                    info = [part.strip() for part in opening.group(2).split(",") if part.strip()]
                    fence = (opening.group(1), info, number, [])
            elif opening and opening.group(1) == fence[0] and not opening.group(2):
                backticks, info, start, code = fence
                if info and info[0] == "noir":
                    examples.append(Example(rel_path, start, info[1:], code))
                fence = None
            else:
                fence[3].append(text)
    return examples


def collect_examples():
    examples = []
    for root, _, files in os.walk(STDLIB_SRC):
        for name in sorted(files):
            if name.endswith(".nr"):
                path = os.path.join(root, name)
                rel_path = os.path.relpath(path, STDLIB_SRC)
                examples.extend(extract_examples(path, rel_path))
    examples.sort(key=lambda e: (e.path, e.line))
    return examples


def render_example(example):
    for attribute in example.attributes:
        if attribute not in ("ignore", "should_fail"):
            raise SystemExit(
                f"{example.path}:{example.line}: unknown code block attribute `{attribute}`"
            )

    uses = [line for line in example.code if line.startswith("use ")]
    rest = [line for line in example.code if not line.startswith("use ")]
    has_items = any(TOP_LEVEL_ITEM.match(line) for line in rest)
    has_main = any(MAIN_FN.match(line) for line in rest)
    test_attribute = "#[test(should_fail)]" if "should_fail" in example.attributes else "#[test]"

    out = [f"// {example.path}:{example.line}", f"mod {example.module_name} {{"]
    out += [f"    {line}" for line in uses]
    if has_items:
        out += [f"    {line}" if line else "" for line in rest]
        out.append(f"    {test_attribute}")
        out.append("    fn test() {")
        if has_main:
            out.append("        main();")
        out.append("    }")
    else:
        out.append(f"    {test_attribute}")
        out.append("    fn test() {")
        out += [f"        {line}" if line else "" for line in rest]
        out.append("    }")
    out.append("}")
    return "\n".join(out)


def write_crate(directory, examples):
    os.makedirs(os.path.join(directory, "src"), exist_ok=True)
    with open(os.path.join(directory, "Nargo.toml"), "w", encoding="utf-8") as f:
        f.write('[package]\nname = "stdlib_doc_examples"\ntype = "lib"\nauthors = [""]\n\n[dependencies]\n')
    with open(os.path.join(directory, "src", "lib.nr"), "w", encoding="utf-8") as f:
        f.write("\n\n".join(render_example(e) for e in examples) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--nargo", default=os.environ.get("NARGO", "nargo"), help="nargo binary to test with")
    parser.add_argument("--keep", metavar="DIR", help="write the generated crate to DIR and keep it")
    parser.add_argument("--list", action="store_true", help="list the examples found and exit")
    args = parser.parse_args()

    examples = collect_examples()
    runnable = [e for e in examples if "ignore" not in e.attributes]
    if args.list:
        for e in examples:
            suffix = f" ({', '.join(e.attributes)})" if e.attributes else ""
            print(f"{e.path}:{e.line}{suffix}")
        return 0

    print(f"Found {len(examples)} `noir` examples in doc comments ({len(examples) - len(runnable)} ignored).")
    if not runnable:
        return 0

    directory = args.keep or tempfile.mkdtemp(prefix="stdlib_doc_examples_")
    try:
        if args.keep and os.path.exists(directory):
            shutil.rmtree(directory)
        write_crate(directory, runnable)
        result = subprocess.run([args.nargo, "test", "--program-dir", directory])
        if result.returncode != 0:
            print(
                "\nA doc example failed. Test names are `<file>_l<line>::test`, pointing at the "
                "opening fence in noir_stdlib/src. Rerun with `--keep DIR` to inspect the generated crate.",
                file=sys.stderr,
            )
        return result.returncode
    finally:
        if not args.keep:
            shutil.rmtree(directory, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
