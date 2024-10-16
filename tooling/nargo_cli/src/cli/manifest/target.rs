use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use crate::cli::internal::serdex::toml_merge;
use crate::cli::manifest::{TargetKind, TomlExternalTargetParams};

/// See [`TargetInner`] for public fields reference.
#[derive(Clone, Debug, Hash)]
pub struct Target(Arc<TargetInner>);

#[derive(Debug)]
#[non_exhaustive]
pub struct TargetInner {
    pub kind: TargetKind,
    pub name: SmolStr,
    pub source_path: Utf8PathBuf,
    pub group_id: Option<SmolStr>,
    pub params: toml::Value,
}

impl Deref for Target {
    type Target = TargetInner;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Target {
    pub fn new(
        kind: TargetKind,
        name: impl Into<SmolStr>,
        source_path: impl Into<Utf8PathBuf>,
        group_id: Option<SmolStr>,
        params: toml::Value,
    ) -> Self {
        assert!(params.is_table(), "params must be a TOML table");
        Self(Arc::new(TargetInner {
            kind,
            name: name.into(),
            source_path: source_path.into(),
            group_id,
            params,
        }))
    }

    pub fn without_params(
        kind: TargetKind,
        name: impl Into<SmolStr>,
        source_path: impl Into<Utf8PathBuf>,
    ) -> Self {
        Self::new(
            kind,
            name,
            source_path,
            None,
            toml::Value::Table(toml::Table::new()),
        )
    }

    pub fn try_from_structured_params(
        kind: TargetKind,
        name: impl Into<SmolStr>,
        source_path: impl Into<Utf8PathBuf>,
        group_id: Option<SmolStr>,
        params: impl Serialize,
    ) -> Result<Self> {
        let params = toml::Value::try_from(params)?;
        Ok(Self::new(kind, name, source_path, group_id, params))
    }

    pub fn is_lib(&self) -> bool {
        self.kind == TargetKind::LIB
    }

    pub fn is_cairo_plugin(&self) -> bool {
        self.kind == TargetKind::CAIRO_PLUGIN
    }

    pub fn is_test(&self) -> bool {
        self.kind == TargetKind::TEST
    }

    pub fn source_root(&self) -> &Utf8Path {
        self.source_path
            .parent()
            .expect("Source path is guaranteed to point to a file.")
    }

    pub fn props<'de, P>(&self) -> Result<P>
    where
        P: Default + Serialize + Deserialize<'de>,
    {
        toml_merge(&P::default(), &self.params)
    }
}

impl Hash for TargetInner {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.name.hash(state);
        self.source_path.hash(state);
        self.params.to_string().hash(state);
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TestTargetProps {
    pub test_type: TestTargetType,
    pub build_external_contracts: Option<Vec<String>>,
}

impl TestTargetProps {
    pub fn new(test_type: TestTargetType) -> Self {
        Self {
            test_type,
            build_external_contracts: Default::default(),
        }
    }

    pub fn with_build_external_contracts(self, external: Vec<String>) -> Self {
        Self {
            build_external_contracts: Some(external),
            ..self
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestTargetType {
    #[default]
    Unit,
    Integration,
}

impl TryInto<TomlExternalTargetParams> for TestTargetProps {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<TomlExternalTargetParams, Self::Error> {
        Ok(toml::Value::try_into(toml::Value::try_from(self)?)?)
    }
}
