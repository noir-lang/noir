ci := if env("CI", "") == "true" {
  "1"
} else if env("CI", "") == "1" {
  "1"
} else {
  "0"
}
use-cross := env("JUST_USE_CROSS", "")

# target information
target-host := `rustc -vV | grep host: | cut -d ' ' -f 2`
target := env("CARGO_BUILD_TARGET", target-host)

# Install tools
install-tools: install-rust-tools install-js-tools install-foundry

[private]
install-binstall:
  #!/usr/bin/env bash
  has_binstall=$(command -v cargo-binstall >/dev/null 2>&1 && echo "true" || { echo >&2 "cargo-binstall is not installed" && echo "false"; })
  if [[ $has_binstall != "true" ]]; then
    curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
  fi

cargo-binstall-args := if ci == "1" { "--force --locked" } else { "--locked" }

# Installs tools necessary for working with Rust code
install-rust-tools: install-binstall
  cargo binstall cargo-nextest@0.9.103 -y {{cargo-binstall-args}}
  cargo binstall cargo-insta@1.42.2 -y {{cargo-binstall-args}}
  cargo binstall cargo-mutants@25.3.1 -y {{cargo-binstall-args}}

# Installs tools necessary for working with Javascript code
install-js-tools: install-binstall
  cargo binstall wasm-pack@0.13.1 -y {{cargo-binstall-args}}
  cargo binstall wasm-bindgen-cli@0.2.100 -y {{cargo-binstall-args}}
  cargo binstall wasm-opt@0.116.1 -y {{cargo-binstall-args}}

# Installs Playwright (necessary for Javascript browser tests but slow to install)
install-playwright:
  npx -y playwright@1.55.0 install --with-deps

# Installs Foundry (necessary for examples)
install-foundry:
  #!/usr/bin/env bash
  has_foundryup=$(command -v foundryup >/dev/null 2>&1 && echo "true" ||  echo "false"; )
  if [[ $has_foundryup != "true" ]]; then
    curl -L https://foundry.paradigm.xyz | bash
  fi
  foundryup -i v1.3.3 > /dev/null

# Rust

# Formats Rust code
format:
  cargo fmt --all

# Checks formatting of Rust code
format-check:
  cargo fmt --all --check

cargo-clippy-args := if ci == "1" { "-Dwarnings" } else { "" }

# Runs clippy on Rust code
clippy:
  cargo clippy --all-targets --workspace --locked --release -- {{cargo-clippy-args}}

cargo := if use-cross != "" { "cross" } else { "cargo" }
[private]
build-bins: install-binstall
  CARGO_BUILD_TARGET="{{target-host}}" cargo binstall cross@0.2.5 -y --force

  {{cargo}} build --package nargo_cli --release --target={{target}} --no-default-features
  {{cargo}} build --package noir_profiler --release --target={{target}} --no-default-features
  {{cargo}} build --package noir_inspector --release --target={{target}} --no-default-features


# Package release artifacts
[linux]
package: build-bins
  mkdir dist
  cp ./target/{{target}}/release/nargo ./dist/nargo
  cp ./target/{{target}}/release/noir-profiler ./dist/noir-profiler
  cp ./target/{{target}}/release/noir-inspector ./dist/noir-inspector
  # TODO(https://github.com/noir-lang/noir/issues/7445): Remove the separate nargo binary
  tar -czf nargo-{{target}}.tar.gz -C dist nargo
  tar -czf noir-{{target}}.tar.gz -C dist .

# Macos uses a 7z instead of tar
[macos]
package: build-bins
  mkdir dist
  cp ./target/{{target}}/release/nargo ./dist/nargo
  cp ./target/{{target}}/release/noir-profiler ./dist/noir-profiler
  cp ./target/{{target}}/release/noir-inspector ./dist/noir-inspector

  # TODO(https://github.com/noir-lang/noir/issues/7445): Remove the separate nargo binary
  7z a -ttar -so -an ./dist/nargo | 7z a -si ./nargo-{{target}}.tar.gz
  7z a -ttar -so -an ./dist/* | 7z a -si ./noir-{{target}}.tar.gz

# Run tests
test:
  cargo nextest run --no-fail-fast -j32 --workspace

export NOIR_AST_FUZZER_BUDGET_SECS := env_var_or_default("NOIR_AST_FUZZER_BUDGET_SECS", "300")

# Performs a nightly fuzzing run
fuzz-nightly: install-rust-tools
  @echo "env NOIR_AST_FUZZER_BUDGET_SECS='$NOIR_AST_FUZZER_BUDGET_SECS'"
  # On regular PRs we run deterministic fuzzing to avoid flaky tests on CI.
  # In the nightly tests we want to explore uncharted territory.
  NOIR_AST_FUZZER_FORCE_NON_DETERMINISTIC=1 cargo nextest run -p noir_ast_fuzzer_fuzz --no-fail-fast


cargo-mutants-args := if ci == "1" { "--in-place -vV" } else { "-j2" }

mutation-test base="master": install-rust-tools
  #!/usr/bin/env bash
  tmpdir=$(mktemp -d)
  trap "rm -rf $tmpdir" EXIT

  git diff origin/{{base}}.. | tee $tmpdir/git.diff
  cargo mutants --no-shuffle --test-tool=nextest -p acir_field -p acir -p acvm -p brillig -p brillig_vm -p blackbox_solver --in-diff $tmpdir/git.diff {{cargo-mutants-args}}
  cargo mutants --no-shuffle --test-tool=nextest -p noirc_evaluator  --in-diff $tmpdir/git.diff {{cargo-mutants-args}}

# Checks if there are any pending insta.rs snapshots and errors if any exist.
check-pending-snapshots:
  #!/usr/bin/env bash
  snapshots=$(find . -name *.snap.new)
  if [[ -n "$snapshots" ]]; then \
    echo "Found pending snapshots:"
    echo ""
    echo $snapshots
    exit 1
  fi

export RUSTDOCFLAGS := "-Dwarnings -Drustdoc::unescaped_backticks"
# Generate doc.rs site for Rust code.
doc:
  cargo doc --no-deps --document-private-items --workspace

# Noir

# Format noir code
format-noir:
  cargo run -- --program-dir={{justfile_dir()}}/noir_stdlib fmt --check
  cd ./test_programs && NARGO="{{justfile_dir()}}/target/debug/nargo" ./format.sh check

# Visualize the CFG after a certain SSA pass and open the Mermaid Live editor.
# This is mostly here for reference: it only works if the pass matches a single unique pass in the pipeline, and there are no errors.
[no-cd]
visualize-ssa-cfg PASS:
  open https://mermaid.live/view#$( \
    cargo run -q -p nargo_cli -- compile --show-ssa-pass {{PASS}} \
      | grep -v After \
      | cargo run -q -p noir_ssa_cli -- visualize --url-encode)

# Javascript

# Lints Javascript code
lint:
  yarn lint

# Builds named Javascript package
build-package PACKAGE: install-js-tools
  yarn workspace {{PACKAGE}} build

# Examples

# Runs test for all examples
run-examples:
  for file in `ls {{justfile_dir()}}/examples`; do \
    just --justfile {{justfile()}} run-example $file; \
  done

# Runs test for example named `EXAMPLE`
run-example EXAMPLE:
  echo "Running {{EXAMPLE}}"; \
  cd {{justfile_dir()}}/examples/{{EXAMPLE}} && ./test.sh; \


# Spellcheck

# Runs spellcheck on Rust source and markdown files
spellcheck:
  yarn spellcheck

stdlib_docs:
  cd noir_stdlib && nargo doc
  rm -rf noir_stdlib/docs
  mv noir_stdlib/target/docs noir_stdlib/docs