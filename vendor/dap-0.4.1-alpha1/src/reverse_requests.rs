use std::collections::HashMap;

use serde::Serialize;

#[cfg(feature = "client")]
use serde::Deserialize;

use crate::types::{RunInTerminalRequestArgumentsKind, StartDebuggingRequestKind};

#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RunInTerminalRequestArguments {
  /// What kind of terminal to launch.
  /// Values: 'integrated', 'external'
  pub kind: Option<RunInTerminalRequestArgumentsKind>,
  /// Title of the terminal.
  pub title: Option<String>,
  /// Working directory for the command. For non-empty, valid paths this
  /// typically results in execution of a change directory command.
  pub cwd: String,
  /// List of arguments. The first argument is the command to run.
  pub args: Vec<String>,
  /// Environment key-value pairs that are added to or removed from the default
  /// environment.
  pub env: Option<HashMap<String, Option<String>>>,
  /// This property should only be set if the corresponding capability
  /// `supportsArgsCanBeInterpretedByShell` is true. If the client uses an
  /// intermediary shell to launch the application, then the client must not
  /// attempt to escape characters with special meanings for the shell. The user
  /// is fully responsible for escaping as needed and that arguments using
  /// special characters may not be portable across shells.
  pub args_can_be_interpreted_by_shell: Option<bool>,
}

#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StartDebuggingRequestArguments {
  /// Arguments passed to the new debug session. The arguments must only contain
  /// properties understood by the `launch` or `attach` requests of the debug
  /// adapter and they must not contain any client-specific properties (e.g.
  /// `type`) or client-specific features (e.g. substitutable 'variables').
  pub configuration: HashMap<String, serde_json::Value>,
  /// Indicates whether the new debug session should be started with a `launch`
  /// or `attach` request.
  /// Values: 'launch', 'attach'
  pub request: StartDebuggingRequestKind,
}

#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "command", content = "arguments", rename_all = "camelCase")]
pub enum ReverseCommand {
  /// This request is sent from the debug adapter to the client to run a command in a terminal.
  ///
  /// This is typically used to launch the debuggee in a terminal provided by the client.
  ///
  /// This request should only be called if the corresponding client capability
  /// `supportsRunInTerminalRequest` is true.
  ///
  /// Client implementations of `runInTerminal` are free to run the command however they choose
  /// including issuing the command to a command line interpreter (aka 'shell'). Argument strings
  /// passed to the `runInTerminal` request must arrive verbatim in the command to be run.
  /// As a consequence, clients which use a shell are responsible for escaping any special shell
  /// characters in the argument strings to prevent them from being interpreted (and modified) by
  /// the shell.
  ///
  /// Some users may wish to take advantage of shell processing in the argument strings. For
  /// clients which implement `runInTerminal` using an intermediary shell, the
  /// `argsCanBeInterpretedByShell` property can be set to true. In this case the client is
  /// requested not to escape any special shell characters in the argument strings.
  ///
  /// Specification: [RunInTerminal](https://microsoft.github.io/debug-adapter-protocol/specification#Reverse_Requests_RunInTerminal)
  RunInTerminal(RunInTerminalRequestArguments),
  /// This request is sent from the debug adapter to the client to start a new debug session of the
  /// same type as the caller.
  ///
  /// This request should only be sent if the corresponding client capability
  /// `supportsStartDebuggingRequest` is true.
  ///
  /// Specification: [StartDebugging](https://microsoft.github.io/debug-adapter-protocol/specification#Reverse_Requests_StartDebugging)
  StartDebugging(StartDebuggingRequestArguments),
}

/// A debug adapter initiated request.
///
/// The specification treats reverse requests identically to all other requests
/// (even though there is a separate section for them). However, in Rust, it is
/// beneficial to separate them because then we don't need to generate a huge
/// amount of serialization code for all requests and supporting types (that the
/// vast majority of would never be serialized by the adapter, only deserialized).
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReverseRequest {
  /// Sequence number for the Request.
  ///
  /// From the [specification](https://microsoft.github.io/debug-adapter-protocol/specification#Base_Protocol_ProtocolMessage):
  ///
  /// Sequence number of the message (also known as message ID). The `seq` for
  /// the first message sent by a client or debug adapter is 1, and for each
  /// subsequent message is 1 greater than the previous message sent by that
  /// actor. `seq` can be used to order requests, responses, and events, and to
  /// associate requests with their corresponding responses. For protocol
  /// messages of type `request` the sequence number can be used to cancel the
  /// request.
  pub seq: i64,
  /// The command to execute.
  ///
  /// This is stringly typed in the specification, but represented as an enum for better
  /// ergonomics in Rust code, along with the arguments when present.
  #[serde(flatten)]
  pub command: ReverseCommand,
}
