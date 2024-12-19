use std::path::PathBuf;

use acvm::AcirField;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::PrintOutput;

use super::{
    layers::{self, Layer, Layering},
    mocker::MockForeignCallExecutor,
    print::PrintForeignCallExecutor,
    rpc::RPCForeignCallExecutor,
    ForeignCallExecutor,
};

pub struct DefaultForeignCallExecutor;

impl DefaultForeignCallExecutor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'a, F>(
        output: PrintOutput<'a>,
        resolver_url: Option<&str>,
        root_path: Option<PathBuf>,
        package_name: Option<String>,
    ) -> impl ForeignCallExecutor<F> + 'a
    where
        F: AcirField + Serialize + for<'de> Deserialize<'de> + 'a,
    {
        Self::with_base(layers::Empty, output, resolver_url, root_path, package_name)
    }

    pub fn with_base<'a, F, B>(
        base: B,
        output: PrintOutput<'a>,
        resolver_url: Option<&str>,
        root_path: Option<PathBuf>,
        package_name: Option<String>,
    ) -> DefaultForeignCallLayers<'a, B, F>
    where
        F: AcirField + Serialize + for<'de> Deserialize<'de> + 'a,
        B: ForeignCallExecutor<F> + 'a,
    {
        // Adding them in the opposite order, so print is the outermost layer.
        base.add_layer(resolver_url.map(|resolver_url| {
            let id = rand::thread_rng().gen();
            RPCForeignCallExecutor::new(resolver_url, id, root_path, package_name)
        }))
        .add_layer(MockForeignCallExecutor::default())
        .add_layer(PrintForeignCallExecutor::new(output))
    }
}

/// Facilitate static typing of layers on a base layer, so inner layers can be accessed.
pub type DefaultForeignCallLayers<'a, B, F> = Layer<
    PrintForeignCallExecutor<'a>,
    Layer<MockForeignCallExecutor<F>, Layer<Option<RPCForeignCallExecutor>, B>>,
>;
