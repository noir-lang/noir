use std::fmt;
use std::sync::Arc;

pub type CloneableResult<T> = anyhow::Result<T, CloneableError>;

#[derive(Clone)]
pub struct CloneableError(Arc<anyhow::Error>);

impl From<anyhow::Error> for CloneableError {
    fn from(err: anyhow::Error) -> Self {
        CloneableError(Arc::new(err))
    }
}

impl std::error::Error for CloneableError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl fmt::Display for CloneableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self.0, f)
    }
}

impl fmt::Debug for CloneableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self.0, f)
    }
}
