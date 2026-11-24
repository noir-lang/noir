#!/usr/bin/env python3
"""Tests for `reports-needs-bencher.py`: each builds a small repository, makes one commit on top of
a base commit, and checks whether that push would use the benchmark machine."""

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("reports-needs-bencher.py").resolve()

WORKFLOW = """\
jobs:
  timed:
    steps:
      - uses: ./.github/actions/download-binary
      - run: .github/scripts/used.sh
"""

BASE_FILES = {
    ".github/workflows/reports.yml": WORKFLOW,
    ".github/workflows/other.yml": "jobs: {}\n",
    ".github/actions/download-binary/action.yml": "runs:\n  using: composite\n",
    ".github/actions/unused/action.yml": "runs:\n  using: composite\n",
    ".github/scripts/used.sh": "echo used\n",
    ".github/scripts/unused.sh": "echo unused\n",
    ".github/benchmark_projects.yml": "projects: {}\n",
    "compiler/lib/src/lib.rs": 'const BLNS: &str = include_str!("./blns.json");\n',
    "compiler/lib/src/blns.json": "[]\n",
    "compiler/lib/README.md": "# lib\n",
    "compiler/lib/src/embedded.rs": '#![doc = include_str!("../README.md")]\n',
    "tooling/doc/src/html.rs": 'const JS: &str = include_str!("search.js");\n',
    "tooling/doc/src/search.js": "// search\n",
    "stdlib/src/lib.nr": "fn main() {}\n",
    "compiler/driver/src/stdlib.rs": '#[derive(RustEmbed)]\n#[folder = "../../../stdlib/src"]\nstruct S;\n',
    "docs/index.md": "# docs\n",
    "docs/package.json": "{}\n",
    "tooling/noir_js/src/index.ts": "export {};\n",
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

    def push(self, files, before=None):
        self.write(files)
        after = self.commit()
        result = subprocess.run(
            [sys.executable, str(SCRIPT), before or self.base, after],
            cwd=self.repo,
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()

    def test_docs_and_js_packages_skip(self):
        files = {"docs/index.md": "# changed\n", "docs/package.json": '{"a": 1}\n'}
        files["tooling/noir_js/src/index.ts"] = "export const a = 1;\n"
        self.assertEqual(self.push(files), "false")

    def test_other_workflows_and_unused_actions_and_scripts_skip(self):
        files = {
            ".github/workflows/other.yml": "jobs: {a: 1}\n",
            ".github/actions/unused/action.yml": "runs: {}\n",
            ".github/scripts/unused.sh": "echo changed\n",
        }
        self.assertEqual(self.push(files), "false")

    def test_rust_source_runs(self):
        self.assertEqual(self.push({"compiler/lib/src/lib.rs": "// changed\n"}), "true")

    def test_cargo_lock_runs(self):
        self.assertEqual(self.push({"Cargo.lock": "# changed\n"}), "true")

    def test_the_workflow_and_what_it_reads_run(self):
        for path in [
            ".github/workflows/reports.yml",
            ".github/benchmark_projects.yml",
            ".github/actions/download-binary/action.yml",
            ".github/scripts/used.sh",
        ]:
            with self.subTest(path=path):
                content = (self.repo / path).read_text() + "# changed\n"
                self.assertEqual(self.push({path: content}), "true")

    def test_an_included_js_file_runs(self):
        self.assertEqual(self.push({"tooling/doc/src/search.js": "// changed\n"}), "true")

    def test_an_included_markdown_file_runs(self):
        self.assertEqual(self.push({"compiler/lib/README.md": "# changed\n"}), "true")

    def test_a_file_under_an_embedded_folder_runs(self):
        self.assertEqual(self.push({"stdlib/src/readme.md": "# new\n"}), "true")

    def test_a_newly_included_file_is_kept_on_later_pushes(self):
        self.write({"compiler/lib/src/notes.rs": 'const N: &str = include_str!("../../../docs/notes.md");\n'})
        self.write({"docs/notes.md": "# notes\n"})
        self.base = self.commit()
        self.assertEqual(self.push({"docs/notes.md": "# changed\n"}), "true")

    def test_a_non_literal_include_keeps_every_push(self):
        self.write({"compiler/lib/src/env.rs": 'const E: &str = include_str!(concat!(env!("OUT_DIR"), "/x"));\n'})
        self.base = self.commit()
        self.assertEqual(self.push({"docs/index.md": "# changed\n"}), "true")

    def test_one_rust_file_among_many_docs_files_runs(self):
        files = {f"docs/page{i}.md": f"# {i}\n" for i in range(400)}
        files["compiler/lib/src/lib.rs"] = "// changed\n"
        self.assertEqual(self.push(files), "true")

    def test_an_unknown_previous_commit_runs(self):
        self.assertEqual(self.push({"docs/index.md": "# changed\n"}, before="0" * 40), "true")
        self.assertEqual(self.push({"docs/index.md": "# again\n"}, before="1" * 40), "true")


if __name__ == "__main__":
    os.environ.setdefault("GIT_CONFIG_NOSYSTEM", "1")
    unittest.main(verbosity=2)
