use alloy_primitives::U256;
use super::{ForeignCallError, ForeignCallExecutor};

/// A builder for default foreign call executor
pub struct DefaultForeignCallBuilder<W> {
    pub output: W,
    pub enable_mocks: bool,

    #[cfg(feature = "rpc")]
    pub resolver_url: Option<String>,

    #[cfg(feature = "rpc")]
    pub root_path: Option<std::path::PathBuf>,

    #[cfg(feature = "rpc")]
    pub package_name: Option<String>,
}

impl Default for DefaultForeignCallBuilder<std::io::Empty> {
    fn default() -> Self {
        Self {
            output: std::io::empty(),
            enable_mocks: false,
            
            #[cfg(feature = "rpc")]
            resolver_url: None,
            
            #[cfg(feature = "rpc")]
            root_path: None,
            
            #[cfg(feature = "rpc")]
            package_name: None,
        }
    }
}

impl<W> DefaultForeignCallBuilder<W> {
    pub fn new(output: W) -> Self {
        Self {
            output,
            enable_mocks: false,
            
            #[cfg(feature = "rpc")]
            resolver_url: None,
            
            #[cfg(feature = "rpc")]
            root_path: None,
            
            #[cfg(feature = "rpc")]
            package_name: None,
        }
    }

    pub fn enable_mocks(mut self) -> Self {
        self.enable_mocks = true;
        self
    }

    #[cfg(feature = "rpc")]
    pub fn with_resolver(
        mut self,
        resolver_url: String,
        root_path: std::path::PathBuf,
        package_name: String,
    ) -> Self {
        self.resolver_url = Some(resolver_url);
        self.root_path = Some(root_path);
        self.package_name = Some(package_name);
        self
    }

    pub fn build(self) -> DefaultForeignCallExecutor<W> {
        DefaultForeignCallExecutor {
            output: self.output,
            enable_mocks: self.enable_mocks,
        }
    }
}

/// Stub default foreign call executor
pub struct DefaultForeignCallExecutor<W> {
    output: W,
    enable_mocks: bool,
}

impl<W> ForeignCallExecutor<U256> for DefaultForeignCallExecutor<W> {
    fn execute(
        &mut self,
        foreign_call: &str,
        _inputs: &[U256],
    ) -> Result<Vec<U256>, ForeignCallError> {
        Err(ForeignCallError::NoHandler(foreign_call.to_string()))
    }
}