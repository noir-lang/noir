//! Enforces the architectural layering of the workspace.
//!
//! The workspace is organised into four layers, listed from lowest to
//! highest: `utils`, `acvm-repo`, `compiler`, and `tooling`. A crate in a
//! lower layer must never depend on a crate in a higher layer (whether as a
//! normal, dev, or build dependency). Crates living outside these
//! directories are unlayered and are not subject to the rule.
//!
//! The classification is purely directory-based, so new crates are picked up
//! automatically with no list to maintain here.
//!
//! `ALLOWED_VIOLATIONS` lists the dependencies that pre-date this check.
//! The list is meant to ratchet down to empty: the test fails both on a
//! new violation and on a stale entry (a listed pair that is no longer a
//! violation), so removing one is the only way it ever changes.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

/// Pre-existing layering violations, expressed as `(crate, depends_on)` pairs.
///
/// Adding an entry here is not a normal workflow — the intent is to remove
/// entries as the underlying dependencies are eliminated or the misplaced
/// crates are relocated.
const ALLOWED_VIOLATIONS: &[(&str, &str)] = &[
    ("acir", "noirc_span"),
    ("noir_wasm", "nargo"),
    ("noir_wasm", "noirc_artifacts"),
    ("noirc_driver", "noirc_abi"),
    ("noirc_driver", "noirc_artifacts"),
    ("noirc_evaluator", "noirc_artifacts"),
    ("noirc_frontend", "noirc_artifacts"),
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Layer {
    Utils,
    Acvm,
    Compiler,
    Tooling,
}

impl Layer {
    fn dir_name(self) -> &'static str {
        match self {
            Layer::Utils => "utils",
            Layer::Acvm => "acvm-repo",
            Layer::Compiler => "compiler",
            Layer::Tooling => "tooling",
        }
    }
}

fn classify(manifest_path: &Path, workspace_root: &Path) -> Option<Layer> {
    let relative = manifest_path.strip_prefix(workspace_root).ok()?;
    let first = relative.iter().next()?.to_str()?;
    match first {
        "utils" => Some(Layer::Utils),
        "acvm-repo" => Some(Layer::Acvm),
        "compiler" => Some(Layer::Compiler),
        "tooling" => Some(Layer::Tooling),
        _ => None,
    }
}

#[test]
fn no_upward_dependencies_between_layers() {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to invoke `cargo metadata`");
    assert!(
        output.status.success(),
        "`cargo metadata` failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let metadata: Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON from `cargo metadata`");
    let workspace_root = PathBuf::from(
        metadata["workspace_root"].as_str().expect("workspace_root missing from cargo metadata"),
    );

    let packages = metadata["packages"].as_array().expect("packages missing from cargo metadata");

    let layered: HashMap<&str, Layer> = packages
        .iter()
        .filter_map(|pkg| {
            let name = pkg["name"].as_str()?;
            let manifest_path = Path::new(pkg["manifest_path"].as_str()?);
            classify(manifest_path, &workspace_root).map(|layer| (name, layer))
        })
        .collect();

    let mut actual: BTreeSet<(String, String)> = BTreeSet::new();
    for pkg in packages {
        let name = pkg["name"].as_str().expect("package name");
        let Some(&src_layer) = layered.get(name) else {
            continue;
        };
        for dep in pkg["dependencies"].as_array().into_iter().flatten() {
            let dep_name = dep["name"].as_str().expect("dependency name");
            let Some(&dep_layer) = layered.get(dep_name) else {
                continue;
            };
            if dep_layer > src_layer {
                actual.insert((name.to_string(), dep_name.to_string()));
            }
        }
    }

    let allowed: BTreeSet<(String, String)> =
        ALLOWED_VIOLATIONS.iter().map(|(src, dep)| (src.to_string(), dep.to_string())).collect();

    let format_pair = |(src, dep): &(String, String)| -> String {
        let src_layer = layered[src.as_str()];
        let dep_layer = layered[dep.as_str()];
        format!(
            "  `{}` ({}) depends on `{}` ({})",
            src,
            src_layer.dir_name(),
            dep,
            dep_layer.dir_name(),
        )
    };

    let new_violations: Vec<String> = actual.difference(&allowed).map(format_pair).collect();
    let stale_entries: Vec<(String, String)> = allowed.difference(&actual).cloned().collect();

    let mut messages: Vec<String> = Vec::new();
    if !new_violations.is_empty() {
        messages.push(format!(
            "New crate layering violation(s) detected. Lower layers must not depend on higher \
             layers (allowed order, low → high: utils → acvm-repo → compiler → tooling):\n\n{}",
            new_violations.join("\n"),
        ));
    }
    if !stale_entries.is_empty() {
        let stale = stale_entries
            .iter()
            .map(|(src, dep)| format!("  (\"{src}\", \"{dep}\")"))
            .collect::<Vec<_>>()
            .join("\n");
        messages.push(format!(
            "Stale entries in `ALLOWED_VIOLATIONS` — these dependencies no longer exist and \
             must be removed from the list:\n\n{stale}",
        ));
    }

    assert!(messages.is_empty(), "{}", messages.join("\n\n"));
}
