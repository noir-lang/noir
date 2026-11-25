#!/usr/bin/env python3
"""Decides whether a master push needs the benchmark-machine jobs of `reports.yml`.

Usage: reports-needs-bencher.py <base-sha> <after-sha>
       reports-needs-bencher.py --since-last-measured <after-sha>

Prints `false` when every file changed between the two commits is on `NO_EFFECT`, a short list of
files that nothing the benchmark machine builds or runs reads, and `true` otherwise. Any doubt
answers `true`: a commit it can't diff, a GitHub API call that fails, or an error of its own. The
workflow step also answers `true` when this script exits without printing an answer.

With `--since-last-measured` the base is the newest earlier master push whose Reports run used
the benchmark machine and hasn't failed (see `choose_base`), so a skipped push never stands on a
run that didn't measure.
"""

import json
import os
import re
import subprocess
import sys
import urllib.request

# Files that nothing the benchmark machine builds or runs reads: the documentation site, the JS
# packages' manifests and lockfiles, and the workflows other than `reports.yml`. No Rust source,
# build script, `reports.yml` or local action it uses names any of them. Everything else, a
# Markdown file or JS source included, uses the machine.
NO_EFFECT = [
    "docs/**",
    "**/package.json",
    "**/yarn.lock",
    ".yarn/**",
    ".yarnrc.yml",
    ".github/dependabot.yml",
    ".github/workflows/*",
]
ALWAYS_MEASURE = [".github/workflows/reports.yml"]

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


def matches_any(path, patterns):
    return any(glob_to_regex(p).match(path) for p in patterns)


def can_affect(path):
    return matches_any(path, ALWAYS_MEASURE) or not matches_any(path, NO_EFFECT)


def git(*args):
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def affecting_files(base, after):
    """Changed files that can affect the measurements, or `None` when the diff fails."""
    try:
        changed = git("diff", "--name-only", "--no-renames", base, after).splitlines()
    except subprocess.CalledProcessError as error:
        print(f"Can't diff {base}..{after}: {error.stderr.strip()}", file=sys.stderr)
        return None
    affecting = [path for path in changed if can_affect(path)]
    print(f"{base[:12]}..{after[:12]}: {len(changed)} changed files, {len(affecting)} can affect the benchmarks", file=sys.stderr)
    for path in affecting[:20]:
        print(f"  {path}", file=sys.stderr)
    return affecting


def choose_base(runs, jobs_of):
    """The commit a skip would stand on, or `None` when this push has to be measured.

    `runs` are the earlier Reports runs of master pushes, newest first, as returned by the GitHub
    API. `jobs_of(run)` returns a run's jobs. Any failed job means this push measures. A run that
    decided to skip the benchmark machine is passed over, since it stands on an earlier run
    itself. The first run that did use the machine decides:

    - finished and green: its commit is the base;
    - finished otherwise: something it measured may be missing, so this push measures;
    - still going: its commit is the base. A failure after this point is not seen by pushes that
      skip; the next push that measures covers it.

    A run still going whose wake job hasn't concluded, or isn't listed yet, may still skip or
    measure. It is the base only once the walk behind it reaches a run that measured without
    failing: whichever it turns out to do, a diff from it covers this push. A finished run without
    a wake job predates the benchmark machine and counts as one that used it.
    """
    pending = None
    for run in runs[:MAX_RUNS_BACK]:
        jobs = jobs_of(run)
        if any(j.get("conclusion") in ("failure", "cancelled", "timed_out") for j in jobs):
            return None
        wake = next((j for j in jobs if j.get("name") == WAKE_JOB), None)
        finished = run.get("status") == "completed"
        if finished and run.get("conclusion") != "success":
            return None
        if wake is not None and wake.get("conclusion") == "skipped":
            continue
        if not finished and (wake is None or wake.get("conclusion") is None):
            pending = pending or run["head_sha"]
            continue
        return pending or run["head_sha"]
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
        "?event=push&branch=master&per_page=100"
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
    affecting = affecting_files(base, after)
    return affecting is None or bool(affecting)


if __name__ == "__main__":
    try:
        result = main(sys.argv[1:])
    except Exception as error:  # noqa: BLE001 - any failure keeps the benchmarks
        print(f"Keeping the benchmarks after an error: {error!r}", file=sys.stderr)
        result = True
    print("true" if result else "false")
