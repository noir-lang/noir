//! Includes custom serde deserializers for Nargo.toml types that could be implemented
//! using `serde(untagged)` but are not because using that leads to very poor error messages.
//! See https://github.com/noir-lang/noir/issues/11088

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, de::Visitor};

use crate::{Config, DependencyConfig, PackageConfig, PackageMetadata, WorkspaceConfig};

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ConfigMapVisitor)
    }
}

struct ConfigMapVisitor;

impl<'de> Visitor<'de> for ConfigMapVisitor {
    type Value = Config;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map with either a [package] or a [workspace] section")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut package_metadata: Option<PackageMetadata> = None;
        let mut workspace_config: Option<WorkspaceConfig> = None;
        let mut dependencies: BTreeMap<String, DependencyConfig> = BTreeMap::new();

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "package" => {
                    package_metadata = Some(map.next_value()?);
                }
                "workspace" => {
                    workspace_config = Some(map.next_value()?);
                }
                "dependencies" => {
                    dependencies = map.next_value()?;
                }
                _ => {
                    // Skip unknown keys
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
        }

        match (package_metadata, workspace_config) {
            (Some(package), None) => {
                Ok(Config::Package { package_config: PackageConfig { package, dependencies } })
            }
            (None, Some(workspace_config)) => Ok(Config::Workspace { workspace_config }),
            (Some(..), Some(..)) => Err(serde::de::Error::custom(
                "Nargo.toml cannot have both [package] and [workspace] sections",
            )),
            (None, None) => Err(serde::de::Error::custom(
                "Nargo.toml must have either a [package] or [workspace] section",
            )),
        }
    }
}

impl<'de> Deserialize<'de> for DependencyConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(DependencyConfigMapVisitor)
    }
}

struct DependencyConfigMapVisitor;

impl<'de> Visitor<'de> for DependencyConfigMapVisitor {
    type Value = DependencyConfig;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "a dependency to be either `{ git = \"...\", tag = \"...\" }` or `{ path = \"...\" }`",
        )
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut git: Option<String> = None;
        let mut tag: Option<String> = None;
        let mut directory: Option<String> = None;
        let mut path: Option<String> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "git" => {
                    git = Some(map.next_value()?);
                }
                "tag" => {
                    tag = Some(map.next_value()?);
                }
                "directory" => {
                    directory = Some(map.next_value()?);
                }
                "path" => {
                    path = Some(map.next_value()?);
                }
                _ => {
                    // Skip unknown keys
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
        }

        match (git, path) {
            (Some(git), None) => {
                let Some(tag) = tag else {
                    return Err(serde::de::Error::custom("Git dependencies must have a `tag` key"));
                };
                Ok(DependencyConfig::Git { git, tag, directory })
            }
            (None, Some(path)) => Ok(DependencyConfig::Path { path }),
            (Some(..), Some(..)) => Err(serde::de::Error::custom(
                "Dependency must have either a `git` or a `path` key, not both",
            )),
            (None, None) => Err(serde::de::Error::custom(
                "Dependency must have either a `git` or a `path` key, but neither was found",
            )),
        }
    }
}
