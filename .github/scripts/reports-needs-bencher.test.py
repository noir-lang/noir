#!/usr/bin/env python3
"""Tests for `reports-needs-bencher.py`: each builds a small repository, makes one commit on top of
a base commit, and checks whether that push would use the benchmark machine."""

import importlib.util
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

# The tests import the script; keep that from leaving a `__pycache__` in the repository.
sys.dont_write_bytecode = True

SCRIPT = Path(__file__).with_name("reports-needs-bencher.py").resolve()

BASE_FILES = {
    ".github/workflows/reports.yml": "jobs: {}\n",
    ".github/workflows/other.yml": "jobs: {}\n",
    ".github/dependabot.yml": "version: 2\n",
    ".github/actions/download-binary/action.yml": "runs:\n  using: composite\n",
    ".github/scripts/used.sh": "echo used\n",
    ".github/benchmark_projects.yml": "projects: {}\n",
    "compiler/lib/src/lib.rs": "fn main() {}\n",
    "compiler/lib/README.md": "# lib\n",
    "tooling/doc/src/search.js": "// search\n",
    "tooling/noir_js/package.json": "{}\n",
    "docs/index.md": "# docs\n",
    "docs/package.json": "{}\n",
    "package.json": "{}\n",
    "yarn.lock": "# lock\n",
    "Cargo.lock": "# lock\n",
}


def run(cwd, *args):
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True).stdout.strip()


class NeedsBencher(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Path(self.tmp.name)
        run(self.repo, "git", "init", "-q")
        run(self.repo, "git", "config", "user.email", "test@example.com")
        run(self.repo, "git", "config", "user.name", "test")
        self.write(BASE_FILES)
        self.base = self.commit()

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, files):
        for path, content in files.items():
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content)

    def commit(self):
        run(self.repo, "git", "add", "-A")
        run(self.repo, "git", "commit", "-q", "--allow-empty", "-m", "commit")
        return run(self.repo, "git", "rev-parse", "HEAD")

    def decide(self, *args, env=None):
        return subprocess.run([sys.executable, str(SCRIPT), *args], cwd=self.repo,
                              capture_output=True, text=True, check=True, env=env).stdout.strip()

    def push(self, files, delete=()):
        """Whether a push of `files` on top of the previous push uses the machine."""
        self.write(files)
        for path in delete:
            (self.repo / path).unlink()
        before, self.base = self.base, self.commit()
        return self.decide(before, self.base)

    def test_files_on_the_list_skip(self):
        for path in [
            "docs/index.md",
            "docs/package.json",
            "package.json",
            "yarn.lock",
            "tooling/noir_js/package.json",
            ".yarnrc.yml",
            ".github/dependabot.yml",
            ".github/workflows/other.yml",
        ]:
            with self.subTest(path=path):
                self.assertEqual(self.push({path: f"# {path}\n"}), "false")

    def test_several_files_on_the_list_skip(self):
        files = {"docs/index.md": "# x\n", "yarn.lock": "# x\n", "tooling/noir_js/package.json": '{"a": 1}\n'}
        self.assertEqual(self.push(files), "false")

    def test_deleting_a_file_on_the_list_skips(self):
        self.assertEqual(self.push({}, delete=["docs/index.md"]), "false")

    def test_files_off_the_list_run(self):
        for path in [
            "compiler/lib/src/lib.rs",
            "Cargo.lock",
            "compiler/lib/README.md",
            "tooling/doc/src/search.js",
            ".github/workflows/reports.yml",
            ".github/actions/download-binary/action.yml",
            ".github/scripts/used.sh",
            ".github/benchmark_projects.yml",
        ]:
            with self.subTest(path=path):
                self.assertEqual(self.push({path: f"# {path}\n"}), "true")

    def test_deleting_a_file_off_the_list_runs(self):
        self.assertEqual(self.push({}, delete=[".github/actions/download-binary/action.yml"]), "true")

    def test_one_file_off_the_list_among_many_on_it_runs(self):
        files = {f"docs/page{i}.md": f"# {i}\n" for i in range(400)}
        files["compiler/lib/src/lib.rs"] = "// changed\n"
        self.assertEqual(self.push(files), "true")

    def test_an_unknown_previous_commit_runs(self):
        self.write({"docs/index.md": "# changed\n"})
        after = self.commit()
        self.assertEqual(self.decide("0" * 40, after), "true")
        self.assertEqual(self.decide("1" * 40, after), "true")

    def test_without_the_github_api_the_last_measured_push_is_unknown(self):
        self.write({"docs/index.md": "# changed\n"})
        after = self.commit()
        env = {k: v for k, v in os.environ.items() if not k.startswith("GITHUB_")}
        self.assertEqual(self.decide("--since-last-measured", after, env=env), "true")


def load_script():
    spec = importlib.util.spec_from_file_location("reports_needs_bencher", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


script = load_script()

# `jobs(wake=...)`: no wake job at all, or one with this conclusion (`None` = not decided yet).
NO_WAKE = object()


def workflow_run(sha, status="completed", conclusion="success"):
    return {"id": sha, "head_sha": sha, "status": status, "conclusion": conclusion}


def running(sha):
    return workflow_run(sha, status="in_progress", conclusion=None)


def jobs(wake=NO_WAKE, failed=False):
    listed = [{"name": "Build nargo", "conclusion": "success"}]
    if wake is not NO_WAKE:
        listed.append({"name": script.WAKE_JOB, "conclusion": wake})
    if failed:
        listed.append({"name": "Rust Benchmarks", "conclusion": "failure"})
    return listed


class ChooseBase(unittest.TestCase):
    """Which earlier push a skip stands on."""

    def choose(self, runs_with_jobs):
        runs = [r for r, _ in runs_with_jobs]
        by_id = {r["id"]: j for r, j in runs_with_jobs}
        return script.choose_base(runs, lambda r: by_id[r["id"]])

    def test_the_wake_job_name_matches_the_workflow(self):
        workflow = (SCRIPT.parents[1] / "workflows/reports.yml").read_text()
        self.assertIn(f"  wake-bencher:\n    name: {script.WAKE_JOB}\n", workflow)

    def test_the_last_measured_green_push(self):
        self.assertEqual(self.choose([(workflow_run("a"), jobs(wake="success"))]), "a")

    def test_pushes_that_skipped_are_passed_over(self):
        runs = [(workflow_run("b"), jobs(wake="skipped")), (workflow_run("a"), jobs(wake="success"))]
        self.assertEqual(self.choose(runs), "a")

    def test_a_running_push_that_skipped_is_passed_over(self):
        runs = [(running("b"), jobs(wake="skipped")), (workflow_run("a"), jobs(wake="success"))]
        self.assertEqual(self.choose(runs), "a")

    def test_a_measured_push_that_failed_measures_this_one(self):
        failed = workflow_run("a", conclusion="failure")
        runs = [(workflow_run("b"), jobs(wake="skipped")), (failed, jobs(wake="success", failed=True))]
        self.assertIsNone(self.choose(runs))

    def test_a_measured_push_that_was_cancelled_measures_this_one(self):
        self.assertIsNone(self.choose([(workflow_run("a", conclusion="cancelled"), jobs(wake="success"))]))

    def test_a_measured_push_still_running(self):
        self.assertEqual(self.choose([(running("a"), jobs(wake="success"))]), "a")

    def test_a_measured_push_still_running_with_a_failed_job(self):
        self.assertIsNone(self.choose([(running("a"), jobs(wake="success", failed=True))]))

    def test_a_running_push_that_has_not_decided_stands_on_what_is_behind_it(self):
        for wake in [None, NO_WAKE]:
            with self.subTest(wake=wake):
                runs = [(running("b"), jobs(wake=wake)), (workflow_run("a"), jobs(wake="success"))]
                self.assertEqual(self.choose(runs), "b")

    def test_a_failure_behind_a_running_push_that_has_not_decided_measures_this_one(self):
        failed = workflow_run("a", conclusion="failure")
        for wake in [None, NO_WAKE]:
            with self.subTest(wake=wake):
                runs = [(running("b"), jobs(wake=wake)), (failed, jobs(wake="success", failed=True))]
                self.assertIsNone(self.choose(runs))

    def test_a_push_whose_wake_was_skipped_by_a_failure(self):
        self.assertIsNone(self.choose([(workflow_run("a", conclusion="failure"), jobs(wake="skipped"))]))

    def test_a_push_from_before_the_benchmark_machine(self):
        self.assertEqual(self.choose([(workflow_run("a"), jobs())]), "a")

    def test_no_measured_push_in_reach(self):
        self.assertIsNone(self.choose([]))
        self.assertIsNone(self.choose([(running("b"), jobs(wake=None))]))
        self.assertIsNone(self.choose([(workflow_run(str(i)), jobs(wake="skipped")) for i in range(40)]))


if __name__ == "__main__":
    os.environ.setdefault("GIT_CONFIG_NOSYSTEM", "1")
    unittest.main(verbosity=2)
