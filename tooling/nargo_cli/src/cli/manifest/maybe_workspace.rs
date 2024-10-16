// The following implementation has been copied from the `Cargo` codebase with slight modifications only.
// The original implementation can be found here:
// https://github.com/rust-lang/cargo/blob/31eda6f7c360d9911f853b3014e057db61238f3e/src/cargo/util/toml/mod.rs#L1071

use anyhow::{bail, Context, Result};
use serde::{de, Deserialize, Serialize};

/// This Trait exists to make [`MaybeWorkspace::Workspace`] generic. It makes deserialization of
/// [`MaybeWorkspace`] much easier, as well as making error messages for
/// [`MaybeWorkspace::resolve`] much nicer.
///
/// Implementors should have a field `workspace` with the type of `bool`. It is used to ensure
/// `workspace` is not `false` in a `Scarb.toml`.
pub trait WorkspaceInherit {
    /// This is the workspace table that is being inherited from.
    /// For example `[workspace.dependencies]` would be the table "dependencies".
    fn inherit_toml_table(&self) -> &str;

    /// This is used to output the value of the implementors `workspace` field.
    fn workspace(&self) -> bool;
}

/// An enum that allows for inheriting keys from a workspace in a Scarb.toml.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum MaybeWorkspace<T, W: WorkspaceInherit> {
    /// The type when inheriting from a workspace.
    Workspace(W),
    /// The "defined" type, or the type that that is used when not inheriting from a workspace.
    Defined(T),
}

impl<'de, T: Deserialize<'de>, W: WorkspaceInherit + de::Deserialize<'de>> de::Deserialize<'de>
    for MaybeWorkspace<T, W>
{
    fn deserialize<D>(deserializer: D) -> Result<MaybeWorkspace<T, W>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let value = serde_value::Value::deserialize(deserializer)?;

        if let Ok(w) = W::deserialize(serde_value::ValueDeserializer::<D::Error>::new(
            value.clone(),
        )) {
            return if w.workspace() {
                Ok(MaybeWorkspace::Workspace(w))
            } else {
                Err(de::Error::custom("`workspace` cannot be false"))
            };
        }
        T::deserialize(serde_value::ValueDeserializer::<D::Error>::new(value))
            .map(MaybeWorkspace::Defined)
    }
}

impl<T, W: WorkspaceInherit> MaybeWorkspace<T, W> {
    pub fn map<Y>(self, f: impl FnOnce(T) -> Result<Y>) -> Result<MaybeWorkspace<Y, W>> {
        Ok(match self {
            MaybeWorkspace::Defined(value) => MaybeWorkspace::Defined(f(value)?),
            MaybeWorkspace::Workspace(w) => MaybeWorkspace::Workspace(w),
        })
    }

    pub fn resolve(self, label: &str, get_ws_inheritable: impl FnOnce() -> Result<T>) -> Result<T> {
        match self {
            MaybeWorkspace::Defined(value) => Ok(value),
            MaybeWorkspace::Workspace(w) => {
                if !w.workspace() {
                    bail!("`workspace` cannot be false");
                }
                get_ws_inheritable().with_context(|| {
                    format!(
                        "error inheriting `{label}` from workspace root manifest's `workspace.{}.{label}`",
                        w.inherit_toml_table(),
                    )
                })
            }
        }
    }

    pub fn as_defined(&self) -> Option<&T> {
        match self {
            MaybeWorkspace::Workspace(_) => None,
            MaybeWorkspace::Defined(defined) => Some(defined),
        }
    }
}
