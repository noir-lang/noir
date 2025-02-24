use std::path::PathBuf;

use acvm::{acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo, AcirField};
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use serde::{Deserialize, Serialize};

use super::{ForeignCallError, ForeignCallExecutor};

#[derive(Debug)]
pub struct RPCForeignCallExecutor {
    /// A randomly generated id for this `DefaultForeignCallExecutor`.
    ///
    /// This is used so that a single `external_resolver` can distinguish between requests from multiple
    /// instantiations of `DefaultForeignCallExecutor`.
    id: u64,
    /// JSON RPC client to resolve foreign calls
    external_resolver: HttpClient,
    /// Root path to the program or workspace in execution.
    root_path: Option<PathBuf>,
    /// Name of the package in execution
    package_name: Option<String>,
    /// Runtime to execute asynchronous tasks on.
    /// See [bridging](https://tokio.rs/tokio/topics/bridging).
    runtime: tokio::runtime::Runtime,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResolveForeignCallRequest<F> {
    /// A session ID which allows the external RPC server to link this foreign call request to other foreign calls
    /// for the same program execution.
    ///
    /// This is intended to allow a single RPC server to maintain state related to multiple program executions being
    /// performed in parallel.
    session_id: u64,

    /// The foreign call which the external RPC server is to provide a response for.
    #[serde(flatten)]
    function_call: ForeignCallWaitInfo<F>,

    /// Root path to the program or workspace in execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    root_path: Option<String>,

    /// Name of the package in execution
    #[serde(skip_serializing_if = "Option::is_none")]
    package_name: Option<String>,
}

type ResolveForeignCallResult<F> = Result<ForeignCallResult<F>, ForeignCallError>;

impl RPCForeignCallExecutor {
    pub fn new(
        resolver_url: &str,
        id: u64,
        root_path: Option<PathBuf>,
        package_name: Option<String>,
    ) -> Self {
        let mut client_builder = HttpClientBuilder::new();

        if let Some(Ok(timeout)) =
            std::env::var("NARGO_FOREIGN_CALL_TIMEOUT").ok().map(|timeout| timeout.parse())
        {
            let timeout_duration = std::time::Duration::from_millis(timeout);
            client_builder = client_builder.request_timeout(timeout_duration);
        };

        let oracle_resolver =
            client_builder.build(resolver_url).expect("Invalid oracle resolver URL");

        // Opcodes are executed in the `ProgramExecutor::execute_circuit` one by one in a loop,
        // we don't need a concurrent thread pool.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .enable_io()
            .build()
            .expect("failed to build tokio runtime");

        RPCForeignCallExecutor {
            external_resolver: oracle_resolver,
            id,
            root_path,
            package_name,
            runtime,
        }
    }
}

impl<F> ForeignCallExecutor<F> for RPCForeignCallExecutor
where
    F: AcirField + Serialize + for<'a> Deserialize<'a>,
{
    /// Execute an async call blocking the current thread.
    /// This method cannot be called from inside a `tokio` runtime, for that to work
    /// we need to offload the execution into a different thread; see the tests.
    fn execute(&mut self, foreign_call: &ForeignCallWaitInfo<F>) -> ResolveForeignCallResult<F> {
        let encoded_params = rpc_params!(ResolveForeignCallRequest {
            session_id: self.id,
            function_call: foreign_call.clone(),
            root_path: self.root_path.clone().map(|path| path.to_str().unwrap().to_string()),
            package_name: self.package_name.clone(),
        });

        let parsed_response = self.runtime.block_on(async {
            self.external_resolver.request("resolve_foreign_call", encoded_params).await
        })?;

        Ok(parsed_response)
    }
}

#[cfg(test)]
mod tests {
    use acvm::{
        acir::brillig::ForeignCallParam, brillig_vm::brillig::ForeignCallResult,
        pwg::ForeignCallWaitInfo, FieldElement,
    };
    use jsonrpsee::proc_macros::rpc;
    use jsonrpsee::server::Server;
    use jsonrpsee::types::ErrorObjectOwned;
    use tokio::sync::{mpsc, oneshot};

    use super::{
        ForeignCallExecutor, RPCForeignCallExecutor, ResolveForeignCallRequest,
        ResolveForeignCallResult,
    };

    #[rpc(server)]
    trait OracleResolver {
        #[method(name = "resolve_foreign_call")]
        fn resolve_foreign_call(
            &self,
            req: ResolveForeignCallRequest<FieldElement>,
        ) -> Result<ForeignCallResult<FieldElement>, ErrorObjectOwned>;
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

    impl OracleResolverServer for OracleResolverImpl {
        fn resolve_foreign_call(
            &self,
            req: ResolveForeignCallRequest<FieldElement>,
        ) -> Result<ForeignCallResult<FieldElement>, ErrorObjectOwned> {
            let response = match req.function_call.function.as_str() {
                "sum" => self.sum(req.function_call.inputs[0].clone()),
                "echo" => self.echo(req.function_call.inputs[0].clone()),
                "id" => FieldElement::from(req.session_id as u128).into(),
                _ => panic!("unexpected foreign call"),
            };
            Ok(response)
        }
    }

    /// The test client send its request and a response channel.
    type RPCForeignCallClientRequest = (
        ForeignCallWaitInfo<FieldElement>,
        oneshot::Sender<ResolveForeignCallResult<FieldElement>>,
    );

    /// Async client used in the tests.
    #[derive(Clone)]
    struct RPCForeignCallClient {
        tx: mpsc::UnboundedSender<RPCForeignCallClientRequest>,
    }

    impl RPCForeignCallExecutor {
        /// Spawn and run the executor in the background until all clients are closed.
        fn run(mut self) -> RPCForeignCallClient {
            let (tx, mut rx) = mpsc::unbounded_channel::<RPCForeignCallClientRequest>();
            let handle = tokio::task::spawn_blocking(move || {
                while let Some((req, tx)) = rx.blocking_recv() {
                    let res = self.execute(&req);
                    let _ = tx.send(res);
                }
            });
            // The task will finish when the client goes out of scope.
            drop(handle);
            RPCForeignCallClient { tx }
        }
    }

    impl RPCForeignCallClient {
        /// Asynchronously execute a foreign call.
        async fn execute(
            &self,
            req: &ForeignCallWaitInfo<FieldElement>,
        ) -> ResolveForeignCallResult<FieldElement> {
            let (tx, rx) = oneshot::channel();
            self.tx.send((req.clone(), tx)).expect("failed to send to executor");
            rx.await.expect("failed to receive from executor")
        }
    }

    /// Start running the Oracle server or a random port, returning the listen URL.
    async fn build_oracle_server() -> std::io::Result<String> {
        // Choosing port 0 results in a random port being assigned.
        let server = Server::builder().build("127.0.0.1:0").await?;
        let addr = server.local_addr()?;
        let handle = server.start(OracleResolverImpl.into_rpc());
        let url = format!("http://{}", addr);
        // In this test we don't care about doing shutdown so let's it run forever.
        tokio::spawn(handle.stopped());
        Ok(url)
    }

    #[tokio::test]
    async fn test_oracle_resolver_echo() {
        let url = build_oracle_server().await.unwrap();

        let executor = RPCForeignCallExecutor::new(&url, 1, None, None).run();

        let foreign_call: ForeignCallWaitInfo<FieldElement> = ForeignCallWaitInfo {
            function: "echo".to_string(),
            inputs: vec![ForeignCallParam::Single(1_u128.into())],
        };

        let result = executor.execute(&foreign_call).await;
        assert_eq!(result.unwrap(), ForeignCallResult { values: foreign_call.inputs });
    }

    #[tokio::test]
    async fn test_oracle_resolver_sum() {
        let url = build_oracle_server().await.unwrap();

        let executor = RPCForeignCallExecutor::new(&url, 2, None, None).run();

        let foreign_call: ForeignCallWaitInfo<FieldElement> = ForeignCallWaitInfo {
            function: "sum".to_string(),
            inputs: vec![ForeignCallParam::Array(vec![1_usize.into(), 2_usize.into()])],
        };

        let result = executor.execute(&foreign_call).await;
        assert_eq!(result.unwrap(), FieldElement::from(3_usize).into());
    }

    #[tokio::test]
    async fn foreign_call_executor_id_is_persistent() {
        let url = build_oracle_server().await.unwrap();

        let executor = RPCForeignCallExecutor::new(&url, 3, None, None).run();

        let foreign_call: ForeignCallWaitInfo<FieldElement> =
            ForeignCallWaitInfo { function: "id".to_string(), inputs: Vec::new() };

        let result_1 = executor.execute(&foreign_call).await.unwrap();
        let result_2 = executor.execute(&foreign_call).await.unwrap();
        assert_eq!(result_1, result_2);
    }

    #[tokio::test]
    async fn oracle_resolver_rpc_can_distinguish_executors() {
        let url = build_oracle_server().await.unwrap();

        let executor_1 = RPCForeignCallExecutor::new(&url, 4, None, None).run();
        let executor_2 = RPCForeignCallExecutor::new(&url, 5, None, None).run();

        let foreign_call: ForeignCallWaitInfo<FieldElement> =
            ForeignCallWaitInfo { function: "id".to_string(), inputs: Vec::new() };

        let result_1 = executor_1.execute(&foreign_call).await.unwrap();
        let result_2 = executor_2.execute(&foreign_call).await.unwrap();
        assert_ne!(result_1, result_2);
    }
}
