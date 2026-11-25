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
    ".github/critical_libraries_status/noir-lang/lib.failures.jsonl": "",
    "compiler/lib/src/lib.rs": 'const BLNS: &str = include_str!("./blns.json");\n',
    "compiler/lib/src/blns.json": "[]\n",
    "compiler/lib/README.md": "# lib\n",
    "compiler/lib/src/embedded.rs": '#![doc = include_str!("../README.md")]\n',
    "tooling/doc/src/html.rs": 'const JS: &str = include_str!("search.js");\n',
    "tooling/doc/src/search.js": "// search\n",
    "stdlib/src/lib.nr": "fn main() {}\n",
    # rust-embed resolves `folder` from the crate root, the directory holding `Cargo.toml`.
    "compiler/driver/Cargo.toml": "[package]\n",
    "compiler/driver/src/file_manager/stdlib.rs": '#[derive(RustEmbed)]\n#[folder = "../../stdlib/src"]\nstruct S;\n',
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

    def push(self, files, before=None, delete=()):
        self.write(files)
        for path in delete:
            (self.repo / path).unlink()
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
            ".github/critical_libraries_status/noir-lang/lib.failures.jsonl",
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

    def test_an_embedded_folder_outside_the_repository_keeps_every_push(self):
        self.write({"compiler/driver/src/other.rs": '#[folder = "../../../../elsewhere"]\nstruct O;\n'})
        self.base = self.commit()
        self.assertEqual(self.push({"docs/index.md": "# changed\n"}), "true")

    def test_one_rust_file_among_many_docs_files_runs(self):
        files = {f"docs/page{i}.md": f"# {i}\n" for i in range(400)}
        files["compiler/lib/src/lib.rs"] = "// changed\n"
        self.assertEqual(self.push(files), "true")

    def test_an_unknown_previous_commit_runs(self):
        self.assertEqual(self.push({"docs/index.md": "# changed\n"}, before="0" * 40), "true")
        self.assertEqual(self.push({"docs/index.md": "# again\n"}, before="1" * 40), "true")


    def test_a_file_named_by_a_script_the_workflow_runs_is_kept(self):
        self.write({
            ".github/scripts/used.sh": "yq .github/bench/projects.yml\n",
            ".github/bench/projects.yml": "a: 1\n",
            ".github/bench/unused.yml": "a: 1\n",
        })
        self.base = self.commit()
        self.assertEqual(self.push({".github/bench/projects.yml": "a: 2\n"}), "true")
        self.base = run(self.repo, "git", "rev-parse", "HEAD")
        self.assertEqual(self.push({".github/bench/unused.yml": "a: 2\n"}), "false")

    def test_a_js_file_the_workflow_runs_is_kept(self):
        self.write({".github/workflows/reports.yml": WORKFLOW + "      - run: node scripts/report.mjs\n"})
        self.write({"scripts/report.mjs": "// report\n"})
        self.base = self.commit()
        self.assertEqual(self.push({"scripts/report.mjs": "// changed\n"}), "true")

    def test_examples_in_this_script_and_its_tests_are_not_references(self):
        self.write({
            ".github/workflows/reports.yml": WORKFLOW + "      - run: .github/scripts/reports-needs-bencher.py\n",
            ".github/scripts/reports-needs-bencher.py": "# docs/index.md\n",
        })
        self.base = self.commit()
        self.assertEqual(self.push({"docs/index.md": "# changed\n"}), "false")

    def test_an_ignored_file_named_in_rust_is_kept(self):
        for name, source in [
            ("include", 'include!("../../../docs/a.md");\n'),
            ("attribute", '#[template(path = "docs/a.md")]\nstruct T;\n'),
            ("build script", 'fn main() { std::fs::read_to_string("docs/a.md").unwrap(); }\n'),
        ]:
            with self.subTest(name=name):
                self.write({"compiler/lib/src/reads.rs": source, "docs/a.md": "# a\n"})
                self.base = self.commit()
                self.assertEqual(self.push({"docs/a.md": f"# {name}\n"}), "true")

    def test_a_directory_holding_the_rust_source_is_not_a_reference(self):
        self.write({
            "compiler/lib/src/paths.rs": 'const UP: &str = "../..";\nconst HERE: &str = ".";\n',
            "compiler/lib/notes.md": "# notes\n",
        })
        self.base = self.commit()
        self.assertEqual(self.push({"compiler/lib/notes.md": "# changed\n"}), "false")

    def test_a_path_named_only_in_an_integration_test_is_not_kept(self):
        self.write({"compiler/lib/tests/layout.rs": 'const DOCS: &str = "docs/index.md";\n'})
        self.base = self.commit()
        self.assertEqual(self.push({"docs/index.md": "# changed\n"}), "false")

    def test_deleting_a_file_the_workflow_reads_runs(self):
        self.assertEqual(self.push({}, delete=[".github/actions/download-binary/action.yml"]), "true")

    def test_deleting_a_docs_page_skips(self):
        self.assertEqual(self.push({}, delete=["docs/index.md"]), "false")

    def test_a_checkout_without_an_index_uses_the_machine(self):
        self.write({"docs/index.md": "# changed\n"})
        after = self.commit()
        (self.repo / ".git/index").unlink()
        result = subprocess.run([sys.executable, str(SCRIPT), self.base, after], cwd=self.repo,
                                capture_output=True, text=True, check=True)
        self.assertEqual(result.stdout.strip(), "true")

    def test_an_include_dir_from_an_environment_variable_keeps_every_push(self):
        self.write({"compiler/lib/src/dir.rs": 'static D: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");\n'})
        self.base = self.commit()
        self.assertEqual(self.push({"docs/index.md": "# changed\n"}), "true")

    def test_a_quote_char_literal_does_not_hide_a_path(self):
        self.write({
            "compiler/lib/src/quote.rs": "fn f(c: char) { match c { '\"' => read(\"docs/a.md\"), _ => {} } }\n",
            "docs/a.md": "# a\n",
        })
        self.base = self.commit()
        self.assertEqual(self.push({"docs/a.md": "# changed\n"}), "true")

    def test_a_non_literal_include_dir_keeps_every_push(self):
        self.write({"compiler/lib/src/dir.rs": 'static D: Dir = include_dir!(concat!(env!("ROOT"), "/x"));\n'})
        self.base = self.commit()
        self.assertEqual(self.push({"docs/index.md": "# changed\n"}), "true")

    def test_an_include_of_generated_code_is_not_a_reference(self):
        self.write({"compiler/lib/tests/gen.rs": 'include!(concat!(env!("OUT_DIR"), "/gen.rs"));\n'})
        self.base = self.commit()
        self.assertEqual(self.push({"docs/index.md": "# changed\n"}), "false")

    def test_without_the_github_api_the_last_measured_push_is_unknown(self):
        self.write({"docs/index.md": "# changed\n"})
        after = self.commit()
        env = {k: v for k, v in os.environ.items() if not k.startswith("GITHUB_")}
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--since-last-measured", after],
            cwd=self.repo, capture_output=True, text=True, check=True, env=env,
        )
        self.assertEqual(result.stdout.strip(), "true")


def load_script():
    spec = importlib.util.spec_from_file_location("reports_needs_bencher", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def workflow_run(sha, status="completed", conclusion="success"):
    return {"id": sha, "head_sha": sha, "status": status, "conclusion": conclusion}


def jobs(wake=None, failed=False):
    listed = [{"name": "Build nargo", "conclusion": "success"}]
    if wake is not None:
        listed.append({"name": "Wake the benchmark machine", "conclusion": wake})
    if failed:
        listed.append({"name": "Rust Benchmarks", "conclusion": "failure"})
    return listed


class ChooseBase(unittest.TestCase):
    """Which earlier push a skip stands on."""

    def choose(self, runs_with_jobs):
        script = load_script()
        runs = [r for r, _ in runs_with_jobs]
        by_id = {r["id"]: j for r, j in runs_with_jobs}
        return script.choose_base(runs, lambda r: by_id[r["id"]])

    def test_the_last_measured_green_push(self):
        self.assertEqual(self.choose([(workflow_run("a"), jobs(wake="success"))]), "a")

    def test_pushes_that_skipped_are_passed_over(self):
        runs = [(workflow_run("b"), jobs(wake="skipped")), (workflow_run("a"), jobs(wake="success"))]
        self.assertEqual(self.choose(runs), "a")

    def test_a_measured_push_that_failed_measures_this_one(self):
        runs = [(workflow_run("b"), jobs(wake="skipped")), (workflow_run("a", conclusion="failure"), jobs(wake="success", failed=True))]
        self.assertIsNone(self.choose(runs))

    def test_a_measured_push_that_was_cancelled_measures_this_one(self):
        self.assertIsNone(self.choose([(workflow_run("a", conclusion="cancelled"), jobs(wake="success"))]))

    def test_a_measured_push_still_running(self):
        self.assertEqual(self.choose([(workflow_run("a", status="in_progress", conclusion=None), jobs(wake="success"))]), "a")

    def test_a_measured_push_still_running_with_a_failed_job(self):
        running = workflow_run("a", status="in_progress", conclusion=None)
        self.assertIsNone(self.choose([(running, jobs(wake="success", failed=True))]))

    def test_a_push_whose_wake_was_skipped_by_a_failure(self):
        self.assertIsNone(self.choose([(workflow_run("a", conclusion="failure"), jobs(wake="skipped"))]))

    def test_a_push_from_before_the_benchmark_machine(self):
        self.assertEqual(self.choose([(workflow_run("a"), jobs())]), "a")

    def test_no_measured_push_in_reach(self):
        self.assertIsNone(self.choose([]))
        self.assertIsNone(self.choose([(workflow_run(str(i)), jobs(wake="skipped")) for i in range(40)]))


if __name__ == "__main__":
    os.environ.setdefault("GIT_CONFIG_NOSYSTEM", "1")
    unittest.main(verbosity=2)
