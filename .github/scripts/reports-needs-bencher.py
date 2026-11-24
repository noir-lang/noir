#!/usr/bin/env python3
"""Decides whether a master push needs the benchmark-machine jobs of `reports.yml`.

Usage: reports-needs-bencher.py <before-sha> <after-sha>

Prints `true` when some file changed between the two commits can affect what those jobs measure,
and `false` when every changed file is one that cannot. Run from a checkout of <after-sha> that
can also read <before-sha>. Any doubt answers `true`: a commit it can't diff, an embedded file it
can't resolve, or an error of its own.

A changed file can't affect the measurements when it matches `NO_EFFECT` and none of the rules
that keep a file:

- it is read by `reports.yml` or one of the local actions it uses (`.github/...` paths that the
  workflow or those actions name);
- it is compiled into nargo: the target of an `include_str!` or `include_bytes!`, or a file under
  a `rust-embed` `#[folder = "..."]`, in any Rust source of <after-sha>.

Both are read from <after-sha> each time, so a new embedded asset or a new script the workflow
starts using is kept without updating this file.
"""

import re
import subprocess
import sys
from pathlib import PurePosixPath

# Files that change nothing the benchmark machine measures: documentation, the JS/TS packages and
# their tooling (no report runs JS), editor and repository settings, and other workflows.
NO_EFFECT = [
    "docs/**",
    "design/**",
    "examples/**",
    "**/*.md",
    "**/*.png",
    "**/*.js",
    "**/*.cjs",
    "**/*.mjs",
    "**/*.ts",
    "**/*.tsx",
    "**/package.json",
    "**/yarn.lock",
    ".yarn/**",
    ".yarnrc.yml",
    ".nvmrc",
    ".prettierrc",
    ".claude/**",
    ".vscode/**",
    ".githooks/**",
    "cspell.json",
    "context7.json",
    "deny.toml",
    "release-please-config.json",
    ".release-please-manifest.json",
    "LICENSE-*",
    ".github/**",
]

# Files under `.github` that matter without being named by the workflow's steps: the project and
# library lists the matrix jobs read.
ALWAYS_KEEP = [
    ".github/workflows/reports.yml",
    ".github/benchmark_projects.yml",
    ".github/critical_libraries_status/**",
]

WORKFLOW = ".github/workflows/reports.yml"


def glob_to_regex(pattern):
    """`**/` matches any number of directories, `**` anything, `*` anything within a directory."""
    out = ""
    i = 0
    while i < len(pattern):
        if pattern.startswith("**/", i):
            out += "(?:.*/)?"
            i += 3
        elif pattern.startswith("**", i):
            out += ".*"
            i += 2
        elif pattern[i] == "*":
            out += "[^/]*"
            i += 1
        else:
            out += re.escape(pattern[i])
            i += 1
    return re.compile(out + r"\Z")


def matches_any(path, patterns):
    return any(glob_to_regex(p).match(path) for p in patterns)


def git(*args):
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def workflow_reads():
    """`.github` files named by `reports.yml` and by the local actions it uses."""
    kept = set()
    pending = [WORKFLOW]
    seen = set()
    while pending:
        source = pending.pop()
        if source in seen:
            continue
        seen.add(source)
        try:
            with open(source) as f:
                text = f.read()
        except FileNotFoundError:
            continue
        for script in re.findall(r"\.github/scripts/[A-Za-z0-9_.-]+", text):
            kept.add(script)
        for action in re.findall(r"\./\.github/actions/([A-Za-z0-9_-]+)", text):
            directory = f".github/actions/{action}"
            kept.add(f"{directory}/**")
            pending.append(f"{directory}/action.yml")
    return kept


INCLUDE = re.compile(r"\binclude_(?:str|bytes)!\s*\(\s*([^)]*?)\s*\)")
FOLDER = re.compile(r"#\[\s*folder\s*=\s*\"([^\"]+)\"\s*\]")


def embedded_files():
    """Patterns for every non-Rust file compiled into a binary, as (patterns, unresolved)."""
    kept = set()
    unresolved = []
    for source in git("ls-files", "*.rs").splitlines():
        try:
            with open(source) as f:
                text = f.read()
        except OSError:
            continue
        directory = PurePosixPath(source).parent
        for argument in INCLUDE.findall(text):
            literal = re.fullmatch(r"\"([^\"]+)\"", argument)
            if not literal:
                unresolved.append(f"{source}: include of {argument}")
                continue
            kept.add(normalize(directory / literal.group(1)))
        for folder in FOLDER.findall(text):
            kept.add(normalize(directory / folder) + "/**")
    return kept, unresolved


def normalize(path):
    parts = []
    for part in PurePosixPath(path).parts:
        if part == "..":
            if parts:
                parts.pop()
        elif part != ".":
            parts.append(part)
    return "/".join(parts)


def main():
    before, after = sys.argv[1], sys.argv[2]
    if not before or set(before) == {"0"}:
        print("No previous commit to compare with", file=sys.stderr)
        return True
    try:
        changed = git("diff", "--name-only", "--no-renames", before, after).splitlines()
    except subprocess.CalledProcessError as error:
        print(f"Can't diff {before}..{after}: {error.stderr.strip()}", file=sys.stderr)
        return True

    embedded, unresolved = embedded_files()
    if unresolved:
        print("Embedded files that can't be resolved, so every push is kept:", file=sys.stderr)
        for entry in unresolved:
            print(f"  {entry}", file=sys.stderr)
        return True
    keep = ALWAYS_KEEP + sorted(workflow_reads()) + sorted(embedded)

    affecting = [f for f in changed if not matches_any(f, NO_EFFECT) or matches_any(f, keep)]
    print(f"{len(changed)} changed files, {len(affecting)} can affect the benchmarks", file=sys.stderr)
    for path in affecting[:20]:
        print(f"  {path}", file=sys.stderr)
    return bool(affecting)


if __name__ == "__main__":
    try:
        result = main()
    except Exception as error:  # noqa: BLE001 - any failure keeps the benchmarks
        print(f"Keeping the benchmarks after an error: {error!r}", file=sys.stderr)
        result = True
    print("true" if result else "false")
