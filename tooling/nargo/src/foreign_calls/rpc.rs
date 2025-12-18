use std::path::PathBuf;

use acvm::{
    AcirField,
    acir::brillig::{ForeignCallParam, ForeignCallResult},
    pwg::ForeignCallWaitInfo,
};
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
    /// External resolver target. We are keeping it to be able to restart httpClient if necessary
    ///
    /// See [`noir-lang/noir#7463`][<https://github.com/noir-lang/noir/issues/7463>]
    resolver_url: String,
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

#[derive(Eq, PartialEq, Debug, Clone)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
struct JSONSerializableFieldElement<F: AcirField>(F);

impl<F: AcirField> JSONSerializableFieldElement<F> {
    fn new(value: F) -> Self {
        JSONSerializableFieldElement(value)
    }

    fn into_inner(self) -> F {
        self.0
    }
}

impl<F: AcirField> Serialize for JSONSerializableFieldElement<F> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_hex().serialize(serializer)
    }
}

impl<'de, F: AcirField> Deserialize<'de> for JSONSerializableFieldElement<F> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: std::borrow::Cow<'de, str> = Deserialize::deserialize(deserializer)?;
        match F::from_hex(&s) {
            Some(value) => Ok(Self(value)),
            None => Err(serde::de::Error::custom(format!("Invalid hex for FieldElement: {s}",))),
        }
    }
}

fn prepare_foreign_call<F: AcirField>(
    foreign_call: ForeignCallWaitInfo<F>,
) -> ForeignCallWaitInfo<JSONSerializableFieldElement<F>> {
    ForeignCallWaitInfo {
        function: foreign_call.function,
        inputs: foreign_call
            .inputs
            .into_iter()
            .map(|param| match param {
                ForeignCallParam::Single(value) => {
                    ForeignCallParam::Single(JSONSerializableFieldElement::new(value))
                }
                ForeignCallParam::Array(values) => ForeignCallParam::Array(
                    values.into_iter().map(JSONSerializableFieldElement::new).collect(),
                ),
            })
            .collect(),
    }
}

fn receive_foreign_call_result<F: AcirField>(
    foreign_call_result: ForeignCallResult<JSONSerializableFieldElement<F>>,
) -> ForeignCallResult<F> {
    ForeignCallResult {
        values: foreign_call_result
            .values
            .into_iter()
            .map(|param| match param {
                ForeignCallParam::Single(value) => ForeignCallParam::Single(value.into_inner()),
                ForeignCallParam::Array(values) => ForeignCallParam::Array(
                    values.into_iter().map(|val| val.into_inner()).collect(),
                ),
            })
            .collect(),
    }
}

type ResolveForeignCallResult<F> = Result<ForeignCallResult<F>, ForeignCallError>;

impl RPCForeignCallExecutor {
    pub fn new(
        resolver_url: &str,
        id: u64,
        root_path: Option<PathBuf>,
        package_name: Option<String>,
    ) -> Self {
        let oracle_resolver = build_http_client(resolver_url);

        // Opcodes are executed in the `ProgramExecutor::execute_circuit` one by one in a loop,
        // we don't need a concurrent thread pool.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .enable_io()
            .build()
            .expect("failed to build tokio runtime");

        RPCForeignCallExecutor {
            external_resolver: oracle_resolver,
            resolver_url: resolver_url.to_string(),
            id,
            root_path,
            package_name,
            runtime,
        }
    }

    fn send_foreign_call<F>(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, jsonrpsee::core::ClientError>
    where
        F: AcirField + Serialize + for<'a> Deserialize<'a>,
    {
        let foreign_call = prepare_foreign_call(foreign_call.clone());

        let params = ResolveForeignCallRequest {
            session_id: self.id,
            function_call: foreign_call,
            root_path: self
                .root_path
                .clone()
                .map(|path| path.to_str().unwrap().to_string())
                .or(Some(String::new())),
            package_name: self.package_name.clone().or(Some(String::new())),
        };
        let encoded_params = rpc_params!(params);
        let response: Result<
            ForeignCallResult<JSONSerializableFieldElement<F>>,
            jsonrpsee::core::ClientError,
        > = self.runtime.block_on(async {
            self.external_resolver.request("resolve_foreign_call", encoded_params).await
        });

        response.map(receive_foreign_call_result)
    }
}

fn build_http_client(target: &str) -> HttpClient {
    let mut client_builder = HttpClientBuilder::new();

    if let Some(Ok(timeout)) =
        std::env::var("NARGO_FOREIGN_CALL_TIMEOUT").ok().map(|timeout| timeout.parse())
    {
        let timeout_duration = std::time::Duration::from_millis(timeout);
        client_builder = client_builder.request_timeout(timeout_duration);
    };

    client_builder.build(target).expect("Invalid oracle resolver URL")
}

impl<F> ForeignCallExecutor<F> for RPCForeignCallExecutor
where
    F: AcirField + Serialize + for<'a> Deserialize<'a>,
{
    /// Execute an async call blocking the current thread.
    /// This method cannot be called from inside a `tokio` runtime, for that to work
    /// we need to offload the execution into a different thread; see the tests.
    fn execute(&mut self, foreign_call: &ForeignCallWaitInfo<F>) -> ResolveForeignCallResult<F> {
        let result = self.send_foreign_call(foreign_call);

        match result {
            Ok(parsed_response) => Ok(parsed_response),
            // TODO: This is a workaround for noir-lang/noir#7463
            // The client is losing connection with the server and it's not being able to manage it
            // so we are re-creating the HttpClient when it happens
            Err(jsonrpsee::core::ClientError::Transport(_)) => {
                self.external_resolver = build_http_client(&self.resolver_url);
                let parsed_response = self.send_foreign_call(foreign_call)?;
                Ok(parsed_response)
            }
            Err(other) => Err(ForeignCallError::from(other)),
        }
    }
}

#[cfg(test)]
mod serialization_tests {
    use acvm::{
        AcirField, FieldElement, acir::brillig::ForeignCallParam,
        brillig_vm::brillig::ForeignCallResult,
    };
    use proptest::prelude::*;

    use super::JSONSerializableFieldElement;

    #[test]
    fn deserializes_json_as_expected() {
        let raw_responses: [(
            &str,
            Vec<ForeignCallParam<JSONSerializableFieldElement<FieldElement>>>,
        ); 3] = [
            ("[]", Vec::new()),
            (
                "[\"0x0000000000000000000000000000000000000000000000000000000000000001\"]",
                vec![ForeignCallParam::Single(JSONSerializableFieldElement::new(
                    FieldElement::one(),
                ))],
            ),
            ("[[]]", vec![ForeignCallParam::Array(Vec::new())]),
        ];

        for (raw_response, expected) in raw_responses {
            let decoded_response: ForeignCallResult<JSONSerializableFieldElement<FieldElement>> =
                serde_json::from_str(&format!("{{ \"values\": {raw_response} }}")).unwrap();
            assert_eq!(decoded_response, ForeignCallResult { values: expected });
        }
    }

    acvm::acir::acir_field::field_wrapper!(TestField, FieldElement);

    impl Arbitrary for TestField {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any::<u128>().prop_map(|v| Self(FieldElement::from(v))).boxed()
        }
    }

    proptest! {
        /// We want to ensure that the serialization and deserialization of arrays of fields works correctly
        /// with `JSONSerializableFieldElement` as JSON serialization on `FieldElement` confuses empty arrays with a field element.
        #[test]
        fn arrays_of_fields_serialization_roundtrip(param: Vec<JSONSerializableFieldElement<TestField>>) {
            let serialized = serde_json::to_string(&param).unwrap();
            let deserialized: Vec<JSONSerializableFieldElement<TestField>> = serde_json::from_str(&serialized).unwrap();

            prop_assert_eq!(param, deserialized);
        }


    }
}

#[cfg(test)]
mod server_tests {
    use acvm::{
        AcirField, FieldElement, acir::brillig::ForeignCallParam,
        brillig_vm::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo,
    };
    use jsonrpsee::proc_macros::rpc;
    use jsonrpsee::server::Server;
    use jsonrpsee::types::ErrorObjectOwned;
    use tokio::sync::{mpsc, oneshot};

    use super::{
        ForeignCallExecutor, JSONSerializableFieldElement, RPCForeignCallExecutor,
        ResolveForeignCallRequest, ResolveForeignCallResult,
    };

    /// Convert the foreign call request from the RPC client to the internal format.
    ///
    /// This is used to convert the request from the JSON RPC client to the format expected by
    /// the `ForeignCallExecutor`.
    fn receive_foreign_call<F: AcirField>(
        foreign_call: ForeignCallWaitInfo<JSONSerializableFieldElement<F>>,
    ) -> ForeignCallWaitInfo<F> {
        ForeignCallWaitInfo {
            function: foreign_call.function,
            inputs: foreign_call
                .inputs
                .into_iter()
                .map(|param| match param {
                    ForeignCallParam::Single(value) => ForeignCallParam::Single(value.into_inner()),
                    ForeignCallParam::Array(values) => ForeignCallParam::Array(
                        values.into_iter().map(|val| val.into_inner()).collect(),
                    ),
                })
                .collect(),
        }
    }

    /// Convert the foreign call result from the internal format to the JSON RPC client format.
    ///
    /// This is used to convert the response from the `ForeignCallExecutor` to the format expected by
    /// the JSON RPC client.
    fn prepare_foreign_call_result<F: AcirField>(
        foreign_call_result: ForeignCallResult<F>,
    ) -> ForeignCallResult<JSONSerializableFieldElement<F>> {
        ForeignCallResult {
            values: foreign_call_result
                .values
                .into_iter()
                .map(|param| match param {
                    ForeignCallParam::Single(value) => {
                        ForeignCallParam::Single(JSONSerializableFieldElement::new(value))
                    }
                    ForeignCallParam::Array(values) => ForeignCallParam::Array(
                        values.into_iter().map(JSONSerializableFieldElement::new).collect(),
                    ),
                })
                .collect(),
        }
    }

    #[rpc(server)]
    trait OracleResolver {
        #[method(name = "resolve_foreign_call")]
        fn resolve_foreign_call(
            &self,
            req: ResolveForeignCallRequest<JSONSerializableFieldElement<FieldElement>>,
        ) -> Result<ForeignCallResult<JSONSerializableFieldElement<FieldElement>>, ErrorObjectOwned>;
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
            req: ResolveForeignCallRequest<JSONSerializableFieldElement<FieldElement>>,
        ) -> Result<ForeignCallResult<JSONSerializableFieldElement<FieldElement>>, ErrorObjectOwned>
        {
            let foreign_call = receive_foreign_call(req.function_call);
            let response = match foreign_call.function.as_str() {
                "sum" => self.sum(foreign_call.inputs[0].clone()),
                "echo" => self.echo(foreign_call.inputs[0].clone()),
                "id" => FieldElement::from(u128::from(req.session_id)).into(),
                _ => panic!("unexpected foreign call"),
            };
            Ok(prepare_foreign_call_result(response))
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

    /// Start running the Oracle server or a random port, returning the vectoren URL.
    async fn build_oracle_server() -> std::io::Result<String> {
        // Choosing port 0 results in a random port being assigned.
        let server = Server::builder().build("127.0.0.1:0").await?;
        let addr = server.local_addr()?;
        let handle = server.start(OracleResolverImpl.into_rpc());
        let url = format!("http://{addr}");
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
