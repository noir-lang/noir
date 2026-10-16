#!/usr/bin/env python3
"""List public standard library items and whether they have a doc comment.

Scans `noir_stdlib/src` (except `internal/`) for `pub` functions, structs,
enums, traits, type aliases and globals, plus the method declarations inside
`pub trait` bodies. An item counts as documented when a `///` comment sits
directly above it; attributes and plain `//` comments (such as the
`// docs:start:` markers) between the doc comment and the item are skipped.
An item has an example when its doc comment contains a code block tagged
`noir` (see `scripts/check_stdlib_doc_examples.py`). Methods in trait impls are
not listed: the trait's documentation covers them.

This is a line-based scan, not a parse, so it can miss unusually formatted
items. It does not resolve visibility: a `pub` item inside a private module is
still listed.

Usage:
    scripts/stdlib_doc_inventory.py              summary per file
    scripts/stdlib_doc_inventory.py --missing    every item without a doc comment or without an example
    scripts/stdlib_doc_inventory.py --check      like --missing, exits 1 if any are found
"""

import argparse
import os
import re
import sys
from collections import Counter

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STDLIB_SRC = os.path.join(REPO_ROOT, "noir_stdlib", "src")
EXCLUDED = ("internal" + os.sep, "primitive_docs.nr")

MODIFIERS = r"(?:(?:unconstrained|comptime)\s+)*"
PUB_ITEM = re.compile(
    rf"^pub\s+{MODIFIERS}(?P<kind>fn|struct|enum|trait|type|global)\s+(?P<name>\w+)"
)
TRAIT_METHOD = re.compile(rf"^{MODIFIERS}fn\s+(?P<name>\w+)")
TRAIT_START = re.compile(r"^pub\s+trait\b")


def strip_comment(line):
    index = line.find("//")
    return line if index < 0 else line[:index]


def doc_comment(lines, index):
    """Returns the lines of the `///` comment attached to the item at `index`, or None."""
    j = index - 1
    while j >= 0:
        text = lines[j].strip()
        if text.startswith("///"):
            break
        if text.startswith("#[") or text.startswith("//"):
            j -= 1
            continue
        return None
    if j < 0:
        return None
    doc = []
    while j >= 0 and lines[j].strip().startswith("///"):
        doc.append(lines[j].strip())
        j -= 1
    return doc[::-1]


def has_example(doc):
    return any(re.match(r"^///\s*```noir\b", line) for line in doc)


def scan_file(path, rel_path):
    items = []
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    depth = 0
    trait_depth = None  # brace depth of the `pub trait` body we are in, if any
    for index, line in enumerate(lines):
        code = strip_comment(line).strip()
        kind = name = None
        item = PUB_ITEM.match(code)
        if item:
            kind, name = item.group("kind"), item.group("name")
        elif trait_depth is not None and depth == trait_depth + 1:
            method = TRAIT_METHOD.match(code)
            if method:
                kind, name = "trait method", method.group("name")
        if kind:
            doc = doc_comment(lines, index)
            status = "missing" if doc is None else ("documented" if has_example(doc) else "no example")
            items.append((rel_path, index + 1, kind, name, status))
        if TRAIT_START.match(code):
            trait_depth = depth
        depth += code.count("{") - code.count("}")
        if trait_depth is not None and depth <= trait_depth and "}" in code:
            trait_depth = None
    return items


def collect_items():
    items = []
    for root, _, files in os.walk(STDLIB_SRC):
        for name in sorted(files):
            path = os.path.join(root, name)
            rel_path = os.path.relpath(path, STDLIB_SRC)
            if name.endswith(".nr") and not rel_path.startswith(EXCLUDED):
                items.extend(scan_file(path, rel_path))
    return sorted(items)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--missing", action="store_true", help="list every undocumented item")
    parser.add_argument("--check", action="store_true", help="list undocumented items and fail if there are any")
    args = parser.parse_args()

    items = collect_items()
    incomplete = [item for item in items if item[4] != "documented"]

    if args.missing or args.check:
        for path, line, kind, name, status in incomplete:
            problem = "has no doc comment" if status == "missing" else "has no `noir` example"
            print(f"noir_stdlib/src/{path}:{line}: {kind} `{name}` {problem}")
        if args.check and incomplete:
            print(f"\n{len(incomplete)} public stdlib items need a doc comment or an example.", file=sys.stderr)
            return 1
        return 0

    counts = {kind: Counter() for kind in sorted({item[2] for item in items})}
    for _, _, kind, _, status in items:
        counts[kind][status] += 1
    print(f"  {'kind':13} {'no doc':>7} {'no example':>11} {'complete':>9} {'total':>6}")
    for kind, count in counts.items():
        print(
            f"  {kind:13} {count['missing']:7} {count['no example']:11} "
            f"{count['documented']:9} {sum(count.values()):6}"
        )
    total = Counter(item[4] for item in items)
    print(
        f"  {'total':13} {total['missing']:7} {total['no example']:11} "
        f"{total['documented']:9} {len(items):6}"
    )

    per_file = {}
    for path, _, _, _, status in incomplete:
        per_file.setdefault(path, Counter())[status] += 1
    print("\nIncomplete items per file (no doc / no example):")
    for path, count in sorted(per_file.items(), key=lambda entry: (-sum(entry[1].values()), entry[0])):
        print(f"  {count['missing']:4} / {count['no example']:4}  {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
