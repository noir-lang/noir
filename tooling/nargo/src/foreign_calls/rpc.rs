use std::path::PathBuf;
use super::{ForeignCallError, ForeignCallExecutor};

/// Stub RPC foreign call executor
#[derive(Debug)]
pub struct RPCForeignCallExecutor {
    /// A randomly generated id for this executor
    id: u64,
    /// External resolver URL
    resolver_url: String,
    /// Root path to the program or workspace in execution
    root_path: Option<PathBuf>,
    /// Name of the package in execution
    package_name: Option<String>,
}

impl RPCForeignCallExecutor {
    pub fn new(
        resolver_url: &str,
        id: u64,
        root_path: Option<PathBuf>,
        package_name: Option<String>,
    ) -> Self {
        Self {
            id,
            resolver_url: resolver_url.to_string(),
            root_path,
            package_name,
        }
    }
}

impl<F> ForeignCallExecutor<F> for RPCForeignCallExecutor {
    fn execute(
        &mut self,
        foreign_call: &str,
        _inputs: &[F],
    ) -> Result<Vec<F>, ForeignCallError> {
        Err(ForeignCallError::Other(
            format!("RPC execution not available without ACVM for call: {}", foreign_call)
        ))
    }
}