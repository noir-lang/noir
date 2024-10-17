use std::mem;

use anyhow::{anyhow, ensure, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use indoc::formatdoc;
use toml_edit::{value, DocumentMut, Entry, InlineTable, Item};
use url::Url;
use crate::cli::internal::fsx;
use crate::cli::package::name::PackageName;
use crate::cli::source::canonical_url::CanonicalUrl;
use crate::cli::source::GitReference;
use super::tomlx::get_table_mut;
use super::{DepId, DepType, Op, OpCtx};

#[derive(Clone, Debug, Default)]
pub struct AddDependency {
    pub dep: DepId,
    pub path: Option<Utf8PathBuf>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    pub dep_type: DepType,
}

struct Dep {
    name: PackageName,
    source: Box<dyn Source>,
}

trait Source {
    fn insert(self: Box<Self>, tab: &mut InlineTable);
}

struct RegistrySource {
    version: String,
}

struct PathSource {
    version: Option<String>,
    path: String,
}

struct GitSource {
    version: Option<String>,
    git: String,
    reference: GitReference,
}

impl Op for AddDependency {
    #[tracing::instrument(level = "trace", skip(doc, ctx))]
    fn apply_to(self: Box<Self>, doc: &mut DocumentMut, ctx: OpCtx<'_>) -> Result<()> {
        let tab = get_table_mut(doc, &[self.dep_type.toml_section_str()])?;

        let dep = Dep::resolve(*self, ctx)?;

        let was_sorted = {
            let values = tab.as_table_like().unwrap().get_values();
            is_sorted(values.iter().map(|(key, _)| key[0]))
        };

        let dep_key = dep.toml_key().to_string();
        dep.upsert(tab.as_table_like_mut().unwrap().entry(&dep_key));

        if was_sorted {
            if let Some(table) = tab.as_table_like_mut() {
                table.sort_values();
            }
        }

        if let Some(t) = tab.as_inline_table_mut() {
            t.fmt()
        }

        Ok(())
    }
}

impl Dep {
    fn resolve(op: AddDependency, ctx: OpCtx<'_>) -> Result<Self> {
        use GitReference::*;

        let name = op
            .dep
            .name
            .ok_or_else(|| anyhow!("please specify package name"))?;

        ensure!(
            !(op.path.is_some() && op.git.is_some()),
            "dependency ({name}) specification is ambiguous, \
            only one of `git` or `path` is allowed"
        );

        if op.branch.is_some() || op.tag.is_some() || op.rev.is_some() {
            ensure!(
                op.git.is_some(),
                "dependency ({name}) is non-Git, but provides `branch`, `tag` or `rev`"
            );

            ensure!(
                [&op.branch, &op.tag, &op.rev]
                    .iter()
                    .filter(|o| o.is_some())
                    .count()
                    <= 1,
                "dependency ({name}) specification is ambiguous, \
                only one of `branch`, `tag` or `rev` is allowed"
            );
        }

        let version = op.dep.version_req.map(|v| {
            // If this is caret version requirement, then try to endorse putting just
            // the version number.
            v.to_string().trim_start_matches('^').to_string()
        });

        let source: Box<dyn Source> = if let Some(path) = op.path {
            let path = fsx::canonicalize_utf8(path)?;
            let path = path_value(ctx.manifest_path, &path);

            Box::new(PathSource { version, path })
        } else if let Some(git) = op.git {
            let reference = if let Some(branch) = op.branch {
                Branch(branch.into())
            } else if let Some(tag) = op.tag {
                Tag(tag.into())
            } else if let Some(rev) = op.rev {
                Rev(rev.into())
            } else {
                DefaultBranch
            };

            let git = CanonicalUrl::new(&Url::parse(&git).with_context(|| {
                formatdoc!(
                    r#"
                    invalid URL provided: {git}
                    help: use an absolute URL to the Git repository
                    "#,
                )
            })?)
            .map(|git_url| git_url.as_str().to_string())
            .unwrap_or(git);

            Box::new(GitSource {
                version,
                git,
                reference,
            })
        } else {
            Box::new(RegistrySource {
                version: version.ok_or_else(|| {
                    anyhow!("please specify package version requirement, for example: {name}@1.0.0")
                })?,
            })
        };

        Ok(Dep { name, source })
    }

    // TODO(#13): With namespaced packages, this should produce a path.
    fn toml_key(&self) -> &str {
        self.name.as_str()
    }

    fn upsert(self, entry: Entry<'_>) {
        let item = entry.or_insert(value(InlineTable::new()));
        expand_version_shortcut(item);
        purge_source(item.as_inline_table_mut().unwrap());
        self.source.insert(item.as_inline_table_mut().unwrap());
        simplify_to_version_shortcut_if_possible(item);
    }
}

impl Source for RegistrySource {
    fn insert(self: Box<Self>, tab: &mut InlineTable) {
        tab.insert("version", self.version.into());
    }
}

impl Source for PathSource {
    fn insert(self: Box<Self>, tab: &mut InlineTable) {
        if let Some(version) = self.version {
            tab.insert("version", version.into());
        }

        tab.insert("path", self.path.into());
    }
}

impl Source for GitSource {
    fn insert(self: Box<Self>, tab: &mut InlineTable) {
        if let Some(version) = self.version {
            tab.insert("version", version.into());
        }

        tab.insert("git", self.git.into());

        match self.reference {
            GitReference::DefaultBranch => {}
            GitReference::Branch(branch) => {
                tab.insert("branch", branch.to_string().into());
            }
            GitReference::Tag(tag) => {
                tab.insert("tag", tag.to_string().into());
            }
            GitReference::Rev(rev) => {
                tab.insert("rev", rev.to_string().into());
            }
        }
    }
}

fn expand_version_shortcut(item: &mut Item) {
    if item.is_value() && !item.is_inline_table() {
        let version = mem::replace(item, value(InlineTable::new()));
        item["version"] = version;
    }
}

fn simplify_to_version_shortcut_if_possible(item: &mut Item) {
    let can_simplify = item
        .as_table_like()
        .map(|tab| tab.len() == 1 && tab.iter().next().unwrap().0 == "version")
        .unwrap_or(false);

    if can_simplify {
        let version = item.as_table_like_mut().unwrap().remove("version").unwrap();
        *item = version;
    }
}

fn purge_source(tab: &mut InlineTable) {
    tab.remove("path");
    tab.remove("git");
    tab.remove("branch");
    tab.remove("tag");
    tab.remove("rev");
}

fn path_value(manifest_path: &Utf8Path, abs_path: &Utf8Path) -> String {
    let package_root = manifest_path
        .parent()
        .expect("Manifest path should point to manifest file.");
    pathdiff::diff_utf8_paths(abs_path, package_root)
        .expect("Both paths should be absolute at this point.")
        .as_str()
        .replace('\\', "/")
}

// Based on Iterator::is_sorted from nightly std; remove in favor of that when stabilized.
fn is_sorted(mut item: impl Iterator<Item = impl PartialOrd>) -> bool {
    let mut last = match item.next() {
        Some(e) => e,
        None => return true,
    };

    for current in item {
        if current < last {
            return false;
        }
        last = current;
    }

    true
}
