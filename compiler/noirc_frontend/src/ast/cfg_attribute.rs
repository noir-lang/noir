use std::fmt::{self, Display};

use super::Location;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CfgAttributed<T> {
    pub cfg_attribute: Option<CfgAttribute>,
    pub inner: T,
}

impl<T> From<T> for CfgAttributed<T> {
    fn from(input: T) -> CfgAttributed<T> {
        CfgAttributed {
            cfg_attribute: None,
            inner: input,
        }
    }
}

impl<T> CfgAttributed<T> {
    pub(crate) fn is_enabled(&self) -> bool {
        self.cfg_attribute.as_ref().map(|cfg_attribute| cfg_attribute.is_enabled()).unwrap_or(false)
    }

    pub fn map<U, F>(self, f: F) -> CfgAttributed<U>
    where
        F: FnOnce(T) -> U,
    {
        let cfg_attribute = self.cfg_attribute.clone();
        let inner = f(self.inner);
        CfgAttributed {
            cfg_attribute,
            inner,
        }
    }

    pub fn each_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        f(&mut self.inner);
    }

    pub fn inner(self) -> T {
        self.inner
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CfgAttribute {
    // feature = "{name}"
    Feature { name: String, location: Location },
}

impl CfgAttribute {
    pub fn name(&self) -> String {
        match self {
            CfgAttribute::Feature { name, .. } => name.clone(),
        }
    }

    pub fn location(&self) -> Location {
        match self {
            CfgAttribute::Feature { location, .. } => *location,
        }
    }
}

impl Display for CfgAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CfgAttribute::Feature { name, location: _ } => {
                write!(f, "feature = {:?}", name)
            }
        }
    }
}

impl CfgAttribute {
    // TODO(https://github.com/noir-lang/noir/issues/7574): enable more features once working
    pub(crate) fn is_enabled(&self) -> bool {
        match self {
            CfgAttribute::Feature { name, .. } => name == "default",
        }
    }

    pub(crate) fn is_disabled(&self) -> bool {
        !self.is_enabled()
    }
}
