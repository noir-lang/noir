use std::path::PathBuf;

use acvm::{acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo, AcirField};
use jsonrpc::{arg as build_json_rpc_arg, minreq_http::Builder, Client};
use noirc_printable_type::ForeignCallError;
use serde::{Deserialize, Serialize};

use super::ForeignCallExecutor;

#[derive(Debug)]
pub(crate) struct RPCForeignCallExecutor {
    /// A randomly generated id for this `DefaultForeignCallExecutor`.
    ///
    /// This is used so that a single `external_resolver` can distinguish between requests from multiple
    /// instantiations of `DefaultForeignCallExecutor`.
    id: u64,
    /// JSON RPC client to resolve foreign calls
    external_resolver: Client,
    /// Root path to the program or workspace in execution.
    root_path: Option<PathBuf>,
    /// Name of the package in execution
    package_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResolveForeignCallRequest<F> {
    /// A session ID which allows the external RPC server to link this foreign call request to other foreign calls
    /// for the same program execution.
    ///
    /// This is intended to allow a single RPC server to maintain state related to multiple program executions being
    /// performed in parallel.
    session_id: u64,

    #[serde(flatten)]
    /// The foreign call which the external RPC server is to provide a response for.
    function_call: ForeignCallWaitInfo<F>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Root path to the program or workspace in execution.
    root_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Name of the package in execution
    package_name: Option<String>,
}

impl RPCForeignCallExecutor {
    pub(crate) fn new(
        resolver_url: &str,
        id: u64,
        root_path: Option<PathBuf>,
        package_name: Option<String>,
    ) -> Self {
        let mut transport_builder =
            Builder::new().url(resolver_url).expect("Invalid oracle resolver URL");

        if let Some(Ok(timeout)) =
            std::env::var("NARGO_FOREIGN_CALL_TIMEOUT").ok().map(|timeout| timeout.parse())
        {
            let timeout_duration = std::time::Duration::from_millis(timeout);
            transport_builder = transport_builder.timeout(timeout_duration);
        };
        let oracle_resolver = Client::with_transport(transport_builder.build());

        RPCForeignCallExecutor { external_resolver: oracle_resolver, id, root_path, package_name }
    }
}

impl<F: AcirField + Serialize + for<'a> Deserialize<'a>> ForeignCallExecutor<F>
    for RPCForeignCallExecutor
{
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        let encoded_params = vec![build_json_rpc_arg(ResolveForeignCallRequest {
            session_id: self.id,
            function_call: foreign_call.clone(),
            root_path: self.root_path.clone().map(|path| path.to_str().unwrap().to_string()),
            package_name: self.package_name.clone(),
        })];

        let req = self.external_resolver.build_request("resolve_foreign_call", &encoded_params);

        let response = self.external_resolver.send_request(req)?;

        let parsed_response: ForeignCallResult<F> = response.result()?;

        Ok(parsed_response)
    }
}

#[cfg(test)]
mod tests {
    use acvm::{
        acir::brillig::ForeignCallParam, brillig_vm::brillig::ForeignCallResult,
        pwg::ForeignCallWaitInfo, FieldElement,
    };
    use jsonrpc_core::Result as RpcResult;
    use jsonrpc_derive::rpc;
    use jsonrpc_http_server::{Server, ServerBuilder};

    use super::{ForeignCallExecutor, RPCForeignCallExecutor, ResolveForeignCallRequest};

    #[allow(unreachable_pub)]
    #[rpc]
    pub trait OracleResolver {
        #[rpc(name = "resolve_foreign_call")]
        fn resolve_foreign_call(
            &self,
            req: ResolveForeignCallRequest<FieldElement>,
        ) -> RpcResult<ForeignCallResult<FieldElement>>;
    }

    struct OracleResolverImpl;

    impl OracleResolverImpl {
        fn echo(&self, param: ForeignCallParam<FieldElement>) -> ForeignCallResult<FieldElement> {
            vec![param].into()
        }

        fn sum(&self, array: ForeignCallParam<FieldElement>) -> ForeignCallResult<FieldElement> {
            let mut res: FieldElement = 0_usize.into();

            for value in array.fields() {
                res += value;
            }

            res.into()
        }
    }

    impl OracleResolver for OracleResolverImpl {
        fn resolve_foreign_call(
            &self,
            req: ResolveForeignCallRequest<FieldElement>,
        ) -> RpcResult<ForeignCallResult<FieldElement>> {
            let response = match req.function_call.function.as_str() {
                "sum" => self.sum(req.function_call.inputs[0].clone()),
                "echo" => self.echo(req.function_call.inputs[0].clone()),
                "id" => FieldElement::from(req.session_id as u128).into(),

                _ => panic!("unexpected foreign call"),
            };
            Ok(response)
        }
    }

    fn build_oracle_server() -> (Server, String) {
        let mut io = jsonrpc_core::IoHandler::new();
        io.extend_with(OracleResolverImpl.to_delegate());

        // Choosing port 0 results in a random port being assigned.
        let server = ServerBuilder::new(io)
            .start_http(&"127.0.0.1:0".parse().expect("Invalid address"))
            .expect("Could not start server");

        let url = format!("http://{}", server.address());
        (server, url)
    }

    #[test]
    fn test_oracle_resolver_echo() {
        let (server, url) = build_oracle_server();

        let mut executor = RPCForeignCallExecutor::new(&url, 1, None, None);

        let foreign_call: ForeignCallWaitInfo<FieldElement> = ForeignCallWaitInfo {
            function: "echo".to_string(),
            inputs: vec![ForeignCallParam::Single(1_u128.into())],
        };

        let result = executor.execute(&foreign_call);
        assert_eq!(result.unwrap(), ForeignCallResult { values: foreign_call.inputs });

        server.close();
    }

    #[test]
    fn test_oracle_resolver_sum() {
        let (server, url) = build_oracle_server();

        let mut executor = RPCForeignCallExecutor::new(&url, 2, None, None);

        let foreign_call: ForeignCallWaitInfo<FieldElement> = ForeignCallWaitInfo {
            function: "sum".to_string(),
            inputs: vec![ForeignCallParam::Array(vec![1_usize.into(), 2_usize.into()])],
        };

        let result = executor.execute(&foreign_call);
        assert_eq!(result.unwrap(), FieldElement::from(3_usize).into());

        server.close();
    }

    #[test]
    fn foreign_call_executor_id_is_persistent() {
        let (server, url) = build_oracle_server();

        let mut executor = RPCForeignCallExecutor::new(&url, 3, None, None);

        let foreign_call: ForeignCallWaitInfo<FieldElement> =
            ForeignCallWaitInfo { function: "id".to_string(), inputs: Vec::new() };

        let result_1 = executor.execute(&foreign_call).unwrap();
        let result_2 = executor.execute(&foreign_call).unwrap();
        assert_eq!(result_1, result_2);

        server.close();
    }

    #[test]
    fn oracle_resolver_rpc_can_distinguish_executors() {
        let (server, url) = build_oracle_server();

        let mut executor_1 = RPCForeignCallExecutor::new(&url, 4, None, None);
        let mut executor_2 = RPCForeignCallExecutor::new(&url, 5, None, None);

        let foreign_call: ForeignCallWaitInfo<FieldElement> =
            ForeignCallWaitInfo { function: "id".to_string(), inputs: Vec::new() };

        let result_1 = executor_1.execute(&foreign_call).unwrap();
        let result_2 = executor_2.execute(&foreign_call).unwrap();
        assert_ne!(result_1, result_2);

        server.close();
    }
}
