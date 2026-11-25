#!/usr/bin/env python3
"""Decides whether a master push needs the benchmark-machine jobs of `reports.yml`.

Usage: reports-needs-bencher.py <base-sha> <after-sha>
       reports-needs-bencher.py --since-last-measured <after-sha>

Prints `true` when some file changed between the two commits can affect what those jobs measure,
and `false` when every changed file is one that cannot. Run from a checkout of <after-sha> that
can also read <base-sha>. Any doubt answers `true`: a commit it can't diff, an embed it can't
resolve, a GitHub API call that fails, or an error of its own. The workflow step also answers
`true` when this script exits without printing an answer.

With `--since-last-measured` the base is the newest earlier master push whose Reports run used
the benchmark machine and hasn't failed (see `choose_base`), so a skipped push never stands on a
run that didn't measure.

A changed file can't affect the measurements when it matches `NO_EFFECT` and none of the rules
that keep a file:

- it is named by `reports.yml`, or by a file that `reports.yml` names, followed transitively
  (`workflow_reads`);
- it is named in a Rust source: the target of an `include_str!`/`include_bytes!`, a file under a
  `rust-embed` `#[folder]`, or any other string literal that resolves to a tracked file or
  directory (`rust_reads`). That covers `include!`, `include_dir!`, proc-macro attributes and
  `build.rs` reads, as long as the path is written out in the source.

All of these are read from <after-sha> each time, so a new embedded asset or a new file the
workflow starts reading is kept without updating this file. A deleted file can't be checked
against them, so deleting anything but documentation (`SAFE_TO_DELETE`) uses the machine.
"""

import json
import os
import re
import subprocess
import sys
import urllib.request
from pathlib import Path, PurePosixPath

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

# Files whose deletion can't affect the measurements. Deleting any other file uses the machine,
# since the keep rules can't tell whether something still names it.
SAFE_TO_DELETE = ["docs/**", "design/**", "**/*.md"]

WORKFLOW = ".github/workflows/reports.yml"
# This script and its tests name files as examples; they are kept, but not read for references.
SELF = {".github/scripts/reports-needs-bencher.py", ".github/scripts/reports-needs-bencher.test.py"}
WAKE_JOB = "Wake the benchmark machine"

# How far back `choose_base` looks for a measured push before giving up and measuring this one.
MAX_RUNS_BACK = 30


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


NO_EFFECT_REGEXES = [glob_to_regex(p) for p in NO_EFFECT]


def no_effect(path):
    return any(r.match(path) for r in NO_EFFECT_REGEXES)


def git(*args):
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def normalize(path):
    """`path` relative to the repository root, or `None` when it is absolute or leaves the root."""
    path = PurePosixPath(path)
    if path.is_absolute():
        return None
    parts = []
    for part in path.parts:
        if part == "..":
            if not parts:
                return None
            parts.pop()
        elif part != ".":
            parts.append(part)
    return "/".join(parts)


class Tree:
    """The tracked files of the checkout, and the directories that hold them."""

    def __init__(self):
        self.files = set(git("ls-files").splitlines())
        self.directories = {str(p) for f in self.files for p in PurePosixPath(f).parents} - {"."}

    def resolve(self, reference, bases, source):
        """Patterns for what `reference`, written in `source`, names when read from each of
        `bases`. A directory that holds `source` itself is skipped: `..` and `.` name where the
        reference is written, not a file it reads."""
        holders = {str(p) for p in PurePosixPath(source).parents}
        found = set()
        for base in bases:
            path = normalize(PurePosixPath(base) / reference.strip("/"))
            if not path:
                continue
            if path in self.files:
                found.add(path)
            elif path in self.directories and path not in holders:
                found.add(path + "/**")
        return found


PATH_TOKEN = re.compile(r"[A-Za-z0-9_.\-]*/[A-Za-z0-9_./\-]*")


def workflow_reads(tree):
    """Every tracked file or directory named by `reports.yml`, following named files in turn."""
    kept = {WORKFLOW}
    pending = [WORKFLOW]
    while pending:
        source = pending.pop()
        try:
            text = Path(source).read_text(errors="replace")
        except OSError:
            continue
        directory = str(PurePosixPath(source).parent)
        for token in PATH_TOKEN.findall(text):
            for pattern in tree.resolve(token, [".", directory], source):
                if pattern in kept:
                    continue
                kept.add(pattern)
                if pattern.endswith("/**"):
                    # A local action is used through its directory; read its definition.
                    for name in ["action.yml", "action.yaml"]:
                        definition = pattern[: -len("**")] + name
                        if definition in tree.files:
                            pending.append(definition)
                elif pattern not in SELF:
                    pending.append(pattern)
    return kept


INCLUDE = re.compile(r"\binclude_(?:str|bytes)!\s*\(\s*([^)]*?)\s*\)")
FOLDER = re.compile(r"#\[\s*folder\s*=\s*\"([^\"]+)\"\s*\]")
# Embed macros whose argument has to be a plain literal for `rust_reads` to see what they read.
OTHER_EMBED = re.compile(r"\b(include|include_dir)!\s*\(\s*([^)]*?)\s*\)")
CHAR = re.compile(r"'(?:\\.|[^'\\\n])'")
STRING = re.compile(r"\"((?:[^\"\\\n]|\\.){1,300})\"")


def crate_root(directory):
    """The nearest directory at or above `directory` holding a `Cargo.toml`."""
    for candidate in [directory, *directory.parents]:
        if (Path(candidate) / "Cargo.toml").is_file():
            return candidate
    return None


def rust_reads(tree):
    """Patterns for the files Rust sources name, as (patterns, unresolved)."""
    kept = set()
    unresolved = []
    for source in sorted(f for f in tree.files if f.endswith(".rs")):
        try:
            text = Path(source).read_text(errors="replace")
        except OSError:
            continue
        directory = PurePosixPath(source).parent
        root = crate_root(directory)
        bases = [str(directory), "."] + ([str(root)] if root is not None else [])

        # `include_str!` and `include_bytes!` resolve from the source file.
        for argument in INCLUDE.findall(text):
            literal = re.fullmatch(r"\"([^\"]+)\"", argument)
            path = normalize(directory / literal.group(1)) if literal else None
            if path is None:
                unresolved.append(f"{source}: include of {argument}")
            else:
                kept.add(path)
        # `rust-embed` resolves `#[folder]` from the crate root.
        for folder in FOLDER.findall(text):
            path = None if root is None or "$" in folder else normalize(root / folder)
            if path is None:
                unresolved.append(f"{source}: folder {folder}")
            else:
                kept.add(path + "/**")
        for macro, argument in OTHER_EMBED.findall(text):
            # Generated by a build script into `OUT_DIR`, outside the repository.
            if argument.startswith('concat!(env!("OUT_DIR"'):
                continue
            if not re.fullmatch(r"\"[^\"$]+\"", argument):
                unresolved.append(f"{source}: {macro}! of {argument}")
        # Any other path written in the source: `include!`, `include_dir!`, proc-macro
        # attributes, `build.rs` reads. Only names of ignored files matter; the rest are kept
        # anyway. Integration tests under `tests/` are built into no binary the machine runs.
        if "/tests/" in f"/{source}":
            continue
        # A `'"'` char literal would otherwise open a string.
        for literal in STRING.findall(CHAR.sub("''", text)):
            if "/" not in literal and "." not in literal:
                continue
            for pattern in tree.resolve(literal, bases, source):
                if pattern == "**" or not names_ignored_file(tree, pattern):
                    continue
                kept.add(pattern)
    return kept, unresolved


def names_ignored_file(tree, pattern):
    if not pattern.endswith("/**"):
        return no_effect(pattern)
    prefix = pattern[: -len("**")]
    return any(f.startswith(prefix) and no_effect(f) for f in tree.files)


def matches_any(path, patterns):
    return any(glob_to_regex(p).match(path) for p in patterns)


def affecting_files(base, after):
    """Changed files that can affect the measurements, or `None` when that can't be decided."""
    try:
        rows = git("diff", "--name-status", "--no-renames", base, after).splitlines()
    except subprocess.CalledProcessError as error:
        print(f"Can't diff {base}..{after}: {error.stderr.strip()}", file=sys.stderr)
        return None
    rows = [row.split("\t", 1) for row in rows]
    changed = [path for _, path in rows]
    deleted = {path for status, path in rows if status == "D"}
    tree = Tree()
    if not tree.files or git("rev-parse", "HEAD").strip() != git("rev-parse", after).strip():
        print(f"The checkout is not a complete checkout of {after}", file=sys.stderr)
        return None
    embedded, unresolved = rust_reads(tree)
    if unresolved:
        print("Rust embeds that can't be resolved, so every push is kept:", file=sys.stderr)
        for entry in unresolved:
            print(f"  {entry}", file=sys.stderr)
        return None
    keep = sorted(workflow_reads(tree) | embedded)
    # A deleted file may still be named, but the name no longer resolves to a tracked file in
    # <after-sha>, so the keep rules can't see it.
    affecting = [
        f for f in changed
        if not no_effect(f) or matches_any(f, keep) or (f in deleted and not matches_any(f, SAFE_TO_DELETE))
    ]
    print(f"{base[:12]}..{after[:12]}: {len(changed)} changed files, {len(affecting)} can affect the benchmarks", file=sys.stderr)
    for path in affecting[:20]:
        print(f"  {path}", file=sys.stderr)
    return affecting


def choose_base(runs, jobs_of):
    """The commit a skip would stand on, or `None` when this push has to be measured.

    `runs` are the earlier Reports runs of master pushes, newest first, as returned by the GitHub
    API. `jobs_of(run)` returns a run's jobs. A run that decided to skip the benchmark machine is
    passed over, since it stands on an earlier run itself. The first run that did use the machine
    decides:

    - finished and green: its commit is the base;
    - finished otherwise: something it measured may be missing, so this push measures;
    - still going: its commit is the base, unless one of its jobs has already failed. A failure
      after this point is not seen by pushes that skip; the next push that measures covers it.

    A run without a wake job either predates the benchmark machine or hasn't reached it yet; both
    count as one that used it.
    """
    for run in runs[:MAX_RUNS_BACK]:
        jobs = jobs_of(run)
        if any(j.get("conclusion") in ("failure", "cancelled", "timed_out") for j in jobs):
            return None
        wake = next((j for j in jobs if j.get("name") == WAKE_JOB), None)
        if wake is not None and wake.get("conclusion") == "skipped":
            if run.get("status") == "completed" and run.get("conclusion") != "success":
                return None
            continue
        if run.get("status") == "completed":
            return run["head_sha"] if run.get("conclusion") == "success" else None
        return run["head_sha"]
    return None


def github(path):
    url = os.environ.get("GITHUB_API_URL", "https://api.github.com") + path
    request = urllib.request.Request(url, headers={
        "Authorization": f"Bearer {os.environ['GITHUB_TOKEN']}",
        "Accept": "application/vnd.github+json",
    })
    with urllib.request.urlopen(request, timeout=15) as response:
        return json.load(response)


def earlier_master_runs():
    repository = os.environ["GITHUB_REPOSITORY"]
    current = int(os.environ["GITHUB_RUN_ID"])
    listing = github(
        f"/repos/{repository}/actions/workflows/reports.yml/runs"
        f"?event=push&branch=master&per_page={MAX_RUNS_BACK + 5}"
    )
    runs = listing["workflow_runs"]
    ids = [r["id"] for r in runs]
    if current not in ids:
        raise RuntimeError(f"run {current} is not among the latest master runs")
    return runs[ids.index(current) + 1 :]


def jobs_of(run):
    repository = os.environ["GITHUB_REPOSITORY"]
    jobs = []
    page = 1
    while True:
        listing = github(f"/repos/{repository}/actions/runs/{run['id']}/jobs?per_page=100&page={page}")
        jobs += listing["jobs"]
        if len(jobs) >= listing["total_count"] or not listing["jobs"]:
            return jobs
        page += 1


def main(args):
    if args[0] == "--since-last-measured":
        after = args[1]
        base = choose_base(earlier_master_runs(), jobs_of)
        if base is None:
            print("The last push that used the benchmark machine didn't measure fully", file=sys.stderr)
            return True
        if base != after:
            subprocess.run(["git", "fetch", "--no-tags", "--depth=1", "origin", base], capture_output=True, timeout=60)
    else:
        base, after = args
    if not base or set(base) == {"0"}:
        print("No previous commit to compare with", file=sys.stderr)
        return True
    affecting = affecting_files(base, after)
    return affecting is None or bool(affecting)


if __name__ == "__main__":
    try:
        result = main(sys.argv[1:])
    except Exception as error:  # noqa: BLE001 - any failure keeps the benchmarks
        print(f"Keeping the benchmarks after an error: {error!r}", file=sys.stderr)
        result = True
    print("true" if result else "false")
