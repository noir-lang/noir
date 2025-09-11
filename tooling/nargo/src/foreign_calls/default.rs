use acvm::AcirField;
use serde::{Deserialize, Serialize};

use super::{
    ForeignCallExecutor,
    layers::{self, Either, Layer, Layering},
    mocker::{DisabledMockForeignCallExecutor, MockForeignCallExecutor},
    print::PrintForeignCallExecutor,
};

#[cfg(feature = "rpc")]
use super::rpc::RPCForeignCallExecutor;

/// A builder for [DefaultForeignCallLayers] where we can enable fields based on feature flags,
/// which is easier than providing different overrides for a `new` method.
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
            enable_mocks: true,

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
    /// Override the output.
    pub fn with_output<T>(self, output: T) -> DefaultForeignCallBuilder<T> {
        DefaultForeignCallBuilder {
            output,
            enable_mocks: self.enable_mocks,
            #[cfg(feature = "rpc")]
            resolver_url: self.resolver_url,
            #[cfg(feature = "rpc")]
            root_path: self.root_path,
            #[cfg(feature = "rpc")]
            package_name: self.package_name,
        }
    }

    /// Enable or disable mocks.
    pub fn with_mocks(mut self, enabled: bool) -> Self {
        self.enable_mocks = enabled;
        self
    }

    /// Set or unset resolver url.
    #[cfg(feature = "rpc")]
    pub fn with_resolver_url(mut self, resolver_url: Option<String>) -> Self {
        self.resolver_url = resolver_url;
        self
    }

    /// Compose the executor layers with [layers::Empty] as the default handler.
    pub fn build<F>(self) -> DefaultForeignCallLayers<W, layers::Empty, F>
    where
        F: AcirField + Serialize + for<'de> Deserialize<'de>,
    {
        self.build_with_base(layers::Empty)
    }

    /// Compose the executor layers with `base` as the default handler.
    pub fn build_with_base<B, F>(self, base: B) -> DefaultForeignCallLayers<W, B, F>
    where
        F: AcirField + Serialize + for<'de> Deserialize<'de>,
        B: ForeignCallExecutor<F>,
    {
        let executor = {
            #[cfg(feature = "rpc")]
            {
                use rand::Rng;

                base.add_layer(self.resolver_url.map(|resolver_url| {
                    let id = rand::rng().random::<u64>();
                    RPCForeignCallExecutor::new(
                        &resolver_url,
                        id,
                        self.root_path,
                        self.package_name,
                    )
                }))
            }
            #[cfg(not(feature = "rpc"))]
            {
                base
            }
        };

        executor
            .add_layer(if self.enable_mocks {
                Either::Left(MockForeignCallExecutor::default())
            } else {
                Either::Right(DisabledMockForeignCallExecutor)
            })
            .add_layer(PrintForeignCallExecutor::new(self.output))
    }
}

/// Facilitate static typing of layers on a base layer, so inner layers can be accessed.
#[cfg(feature = "rpc")]
pub type DefaultForeignCallLayers<W, B, F> = Layer<
    PrintForeignCallExecutor<W>,
    Layer<
        Either<MockForeignCallExecutor<F>, DisabledMockForeignCallExecutor>,
        Layer<Option<RPCForeignCallExecutor>, B>,
    >,
>;
#[cfg(not(feature = "rpc"))]
pub type DefaultForeignCallLayers<W, B, F> = Layer<
    PrintForeignCallExecutor<W>,
    Layer<Either<MockForeignCallExecutor<F>, DisabledMockForeignCallExecutor>, B>,
>;

/// Convenience constructor for code that used to create the executor this way.
#[cfg(feature = "rpc")]
pub struct DefaultForeignCallExecutor;

/// Convenience constructors for the RPC case. Non-RPC versions are not provided
/// because once a crate opts into this within the workspace, everyone gets it
/// even if they don't want to. For the non-RPC case we can nudge people to
/// use `DefaultForeignCallBuilder` which is easier to keep flexible.
#[cfg(feature = "rpc")]
impl DefaultForeignCallExecutor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'a, W, F>(
        output: W,
        resolver_url: Option<&str>,
        root_path: Option<std::path::PathBuf>,
        package_name: Option<String>,
    ) -> impl ForeignCallExecutor<F> + 'a + use<'a, W, F>
    where
        W: std::io::Write + 'a,
        F: AcirField + Serialize + for<'de> Deserialize<'de> + 'a,
    {
        DefaultForeignCallBuilder {
            output,
            enable_mocks: true,
            resolver_url: resolver_url.map(|s| s.to_string()),
            root_path,
            package_name,
        }
        .build()
    }
}
