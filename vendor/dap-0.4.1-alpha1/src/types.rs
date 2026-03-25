use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "integration_testing")]
use fake::{Dummy, Fake, Faker};
#[cfg(feature = "integration_testing")]
use rand::Rng;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct ExceptionBreakpointsFilter {
  /// The internal ID of the filter option. This value is passed to the
  /// `setExceptionBreakpoints` request.
  pub filter: String,
  /// The name of the filter option. This is shown in the UI.
  pub label: String,
  /// A help text providing additional information about the exception filter.
  /// This string is typically shown as a hover and can be translated.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  /// Initial value of the filter option. If not specified a value false is
  /// assumed.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub default: Option<bool>,
  /// Controls whether a condition can be specified for this filter option. If
  /// false or missing, a condition can not be set.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_condition: Option<bool>,
  /// A help text providing information about the condition. This string is shown
  /// as the placeholder text for a text box and can be translated.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub condition_description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum ColumnDescriptorType {
  String,
  Number,
  Boolean,
  UnixTimestampUTC,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct ColumnDescriptor {
  /// Name of the attribute rendered in this column.
  pub attribute_name: String,
  /// Header UI label of column.
  pub label: String,
  /// Format to use for the rendered values in this column. TBD how the format
  /// strings looks like.
  pub format: String,
  /// Datatype of values in this column. Defaults to `string` if not specified.
  /// Values: 'string', 'number', 'bool', 'unixTimestampUTC'
  #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
  pub column_descriptor_type: Option<ColumnDescriptorType>,
  /// Width of this column in characters (hint only).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub width: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum ChecksumAlgorithm {
  MD5,
  SHA1,
  SHA256,
  #[serde(rename = "timestamp")]
  Timestamp,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct Capabilities {
  /// The debug adapter supports the `configurationDone` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_configuration_done_request: Option<bool>,
  /// The debug adapter supports function breakpoints.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_function_breakpoints: Option<bool>,
  /// The debug adapter supports conditional breakpoints.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_conditional_breakpoints: Option<bool>,
  /// The debug adapter supports breakpoints that break execution after a
  /// specified i64 of hits.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_hit_conditional_breakpoints: Option<bool>,
  /// The debug adapter supports a (side effect free) `evaluate` request for data
  /// hovers.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_evaluate_for_hovers: Option<bool>,
  /// Available exception filter options for the `setExceptionBreakpoints`
  /// request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub exception_breakpoint_filters: Option<Vec<ExceptionBreakpointsFilter>>,
  /// The debug adapter supports stepping back via the `stepBack` and
  /// `reverseContinue` requests.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_step_back: Option<bool>,
  /// The debug adapter supports setting a variable to a value.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_set_variable: Option<bool>,
  /// The debug adapter supports restarting a frame.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_restart_frame: Option<bool>,
  /// The debug adapter supports the `gotoTargets` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_goto_targets_request: Option<bool>,
  /// The debug adapter supports the `stepInTargets` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_step_in_targets_request: Option<bool>,
  /// The debug adapter supports the `completions` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_completions_request: Option<bool>,
  /// The set of characters that should trigger completion in a REPL. If not
  /// specified, the UI should assume the `.` character.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub completion_trigger_characters: Option<Vec<String>>,
  /// The debug adapter supports the `modules` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_modules_request: Option<bool>,
  /// The set of additional module information exposed by the debug adapter.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub additional_module_columns: Option<Vec<ColumnDescriptor>>,
  /// Checksum algorithms supported by the debug adapter.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supported_checksum_algorithms: Option<Vec<ChecksumAlgorithm>>,
  /// The debug adapter supports the `restart` request. In this case a client
  /// should not implement `restart` by terminating and relaunching the adapter
  /// but by calling the `restart` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_restart_request: Option<bool>,
  /// The debug adapter supports `exceptionOptions` on the
  /// `setExceptionBreakpoints` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_exception_options: Option<bool>,
  /// The debug adapter supports a `format` attribute on the `stackTrace`,
  /// `variables`, and `evaluate` requests.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_value_formatting_options: Option<bool>,
  /// The debug adapter supports the `exceptionInfo` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_exception_info_request: Option<bool>,
  /// The debug adapter supports the `terminateDebuggee` attribute on the
  /// `disconnect` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub support_terminate_debuggee: Option<bool>,
  /// The debug adapter supports the `suspendDebuggee` attribute on the
  /// `disconnect` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub support_suspend_debuggee: Option<bool>,
  /// The debug adapter supports the delayed loading of parts of the stack, which
  /// requires that both the `startFrame` and `levels` arguments and the
  /// `totalFrames` result of the `stackTrace` request are supported.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_delayed_stack_trace_loading: Option<bool>,
  /// The debug adapter supports the `loadedSources` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_loaded_sources_request: Option<bool>,
  /// The debug adapter supports log points by interpreting the `logMessage`
  /// attribute of the `SourceBreakpoint`.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_log_points: Option<bool>,
  /// The debug adapter supports the `terminateThreads` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_terminate_threads_request: Option<bool>,
  /// The debug adapter supports the `setExpression` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_set_expression: Option<bool>,
  /// The debug adapter supports the `terminate` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_terminate_request: Option<bool>,
  /// The debug adapter supports data breakpoints.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_data_breakpoints: Option<bool>,
  /// The debug adapter supports the `readMemory` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_read_memory_request: Option<bool>,
  /// The debug adapter supports the `writeMemory` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_write_memory_request: Option<bool>,
  /// The debug adapter supports the `disassemble` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_disassemble_request: Option<bool>,
  /// The debug adapter supports the `cancel` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_cancel_request: Option<bool>,
  /// The debug adapter supports the `breakpointLocations` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_breakpoint_locations_request: Option<bool>,
  /// The debug adapter supports the `clipboard` context value in the `evaluate`
  /// request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_clipboard_context: Option<bool>,
  /// The debug adapter supports stepping granularities (argument `granularity`)
  /// for the stepping requests.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_stepping_granularity: Option<bool>,
  /// The debug adapter supports adding breakpoints based on instruction
  /// references.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_instruction_breakpoints: Option<bool>,
  /// The debug adapter supports `filterOptions` as an argument on the
  /// `setExceptionBreakpoints` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_exception_filter_options: Option<bool>,
  /// The debug adapter supports the `singleThread` property on the execution
  /// requests (`continue`, `next`, `stepIn`, `stepOut`, `reverseContinue`,
  /// `stepBack`).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub supports_single_thread_execution_requests: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct CustomValue(Value);

#[cfg(feature = "integration_testing")]
struct ValueFaker;

#[cfg(feature = "integration_testing")]
impl Dummy<ValueFaker> for CustomValue {
  fn dummy_with_rng<R: Rng + ?Sized>(_: &ValueFaker, rng: &mut R) -> Self {
    CustomValue(match rng.gen_range(0..=5) {
      1 => Value::Bool(rng.gen()),
      2 => Value::Number(serde_json::Number::from_f64(rng.gen()).unwrap()),
      3 => Value::String(Faker.fake::<String>()),
      _ => Value::Null,
    })
  }
}

/// A Source is a descriptor for source code.
///
/// It is returned from the debug adapter as part of a StackFrame and it is used by clients when
/// specifying breakpoints.
///
/// Specification: [Source](https://microsoft.github.io/debug-adapter-protocol/specification#Types_Source)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct Source {
  /// The short name of the source. Every source returned from the debug adapter
  /// has a name.
  /// When sending a source to the debug adapter this name is optional.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  /// The path of the source to be shown in the UI.
  /// It is only used to locate and load the content of the source if no
  /// `sourceReference` is specified (or its value is 0).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,
  /// If the value > 0 the contents of the source must be retrieved through the
  /// `source` request (even if a path is specified).
  /// Since a `sourceReference` is only valid for a session, it can not be used
  /// to persist a source.
  /// The value should be less than or equal to 2147483647 (2^31-1).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source_reference: Option<i32>,
  /// A hint for how to present the source in the UI.
  /// A value of `deemphasize` can be used to indicate that the source is not
  /// available or that it is skipped on stepping.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub presentation_hint: Option<PresentationHint>,
  /// The origin of this source. For example, 'internal module', 'inlined content
  /// from source map', etc.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub origin: Option<String>,
  /// A list of sources that are related to this source. These may be the source
  /// that generated this source.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sources: Option<Vec<Source>>,
  /// Additional data that a debug adapter might want to loop through the client.
  /// The client should leave the data intact and persist it across sessions. The
  /// client should not interpret the data.
  #[cfg_attr(feature = "integration_testing", dummy(faker = "ValueFaker"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub adapter_data: Option<CustomValue>,
  /// The checksums associated with this file.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub checksums: Option<Vec<Checksum>>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct SourceBreakpoint {
  /// The source line of the breakpoint or logpoint.
  pub line: i64,
  /// Start position within source line of the breakpoint or logpoint. It is
  /// measured in UTF-16 code units and the client capability `columnsStartAt1`
  /// determines whether it is 0- or 1-based.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<i64>,
  /// The expression for conditional breakpoints.
  /// It is only honored by a debug adapter if the corresponding capability
  /// `supportsConditionalBreakpoints` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub condition: Option<String>,
  /// The expression that controls how many hits of the breakpoint are ignored.
  /// The debug adapter is expected to interpret the expression as needed.
  /// The attribute is only honored by a debug adapter if the corresponding
  /// capability `supportsHitConditionalBreakpoints` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hit_condition: Option<String>,
  /// If this attribute exists and is non-empty, the debug adapter must not
  /// 'break' (stop)
  /// but log the message instead. Expressions within `{}` are interpolated.
  /// The attribute is only honored by a debug adapter if the corresponding
  /// capability `supportsLogPoints` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub log_message: Option<String>,
}

/// Information about a breakpoint created in setBreakpoints, setFunctionBreakpoints,
/// setInstructionBreakpoints, or setDataBreakpoints requests.
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct Breakpoint {
  /// The identifier for the breakpoint. It is needed if breakpoint events are
  /// used to update or remove breakpoints.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<i64>,
  /// If true, the breakpoint could be set (but not necessarily at the desired
  /// location).
  pub verified: bool,
  /// A message about the state of the breakpoint.
  /// This is shown to the user and can be used to explain why a breakpoint could
  /// not be verified.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub message: Option<String>,
  /// The source where the breakpoint is located.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source: Option<Source>,
  /// The start line of the actual range covered by the breakpoint.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub line: Option<i64>,
  /// Start position of the source range covered by the breakpoint. It is
  /// measured in UTF-16 code units and the client capability `columnsStartAt1`
  /// determines whether it is 0- or 1-based.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<i64>,
  /// The end line of the actual range covered by the breakpoint.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_line: Option<i64>,
  /// End position of the source range covered by the breakpoint. It is measured
  /// in UTF-16 code units and the client capability `columnsStartAt1` determines
  /// whether it is 0- or 1-based.
  /// If no end line is given, then the end column is assumed to be in the start
  /// line.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_column: Option<i64>,
  /// A memory reference to where the breakpoint is set.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instruction_reference: Option<String>,
  /// The offset from the instruction reference.
  /// This can be negative.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub offset: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum PresentationHint {
  Normal,
  Emphasize,
  DeEmphasize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct Checksum {
  /// The algorithm used to calculate this checksum.
  pub algorithm: ChecksumAlgorithm,
  /// Value of the checksum, encoded as a hexadecimal value.
  pub checksum: String,
}

/// An ExceptionFilterOptions is used to specify an exception filter together with a condition for
/// the setExceptionBreakpoints request.
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct ExceptionFilterOptions {
  /// ID of an exception filter returned by the `exceptionBreakpointFilters`
  /// capability.
  pub filter_id: String,
  /// An expression for conditional exceptions.
  /// The exception breaks into the debugger if the result of the condition is
  /// true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub condition: Option<String>,
}

/// This enumeration defines all possible conditions when a thrown exception should result in a
/// break.
///
/// Specification: [`ExceptionBreakMode`](https://microsoft.github.io/debug-adapter-protocol/specification#Types_ExceptionBreakMode)
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum ExceptionBreakMode {
  /// never breaks
  Never,
  /// always breaks
  Always,
  /// breaks when exception unhandled
  Unhandled,
  /// breaks if the exception is not handled by user code
  UserUnhandled,
}

impl Default for ExceptionBreakMode {
  fn default() -> Self {
    ExceptionBreakMode::Never
  }
}

/// An ExceptionPathSegment represents a segment in a path that is used to match leafs or nodes in
/// a tree of exceptions.
/// If a segment consists of more than one name, it matches the names provided if negate is false
/// or missing, or it matches anything except the names provided if negate is true.
///
/// Specification: [`ExceptionPathSegment`](https://microsoft.github.io/debug-adapter-protocol/specification#Types_ExceptionPathSegment)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct ExceptionPathSegment {
  /// If false or missing this segment matches the names provided, otherwise it
  /// matches anything except the names provided.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub negate: Option<bool>,
  /// Depending on the value of `negate` the names that should match or not
  /// match.
  pub names: Vec<String>,
}

/// An ExceptionOptions assigns configuration options to a set of exceptions.
///
/// Specification: [`ExceptionOptions`](https://microsoft.github.io/debug-adapter-protocol/specification#Types_ExceptionOptions)
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct ExceptionOptions {
  /// A path that selects a single or multiple exceptions in a tree. If `path` is
  /// missing, the whole tree is selected.
  /// By convention the first segment of the path is a category that is used to
  /// group exceptions in the UI.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<Vec<ExceptionPathSegment>>,
  /// Condition when a thrown exception should result in a break.
  pub break_mode: ExceptionBreakMode,
}

/// Properties of a breakpoint passed to the setFunctionBreakpoints request.
///
/// Specification: [FunctionBreakpoint](https://microsoft.github.io/debug-adapter-protocol/specification#Types_FunctionBreakpoint)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct FunctionBreakpoint {
  /// The name of the function.
  pub name: String,
  /// An expression for conditional breakpoints.
  /// It is only honored by a debug adapter if the corresponding capability
  /// `supportsConditionalBreakpoints` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub condition: Option<String>,
  /// An expression that controls how many hits of the breakpoint are ignored.
  /// The debug adapter is expected to interpret the expression as needed.
  /// The attribute is only honored by a debug adapter if the corresponding
  /// capability `supportsHitConditionalBreakpoints` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hit_condition: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum BreakpointEventReason {
  Changed,
  New,
  Removed,
  #[serde(untagged)]
  String(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum InvalidatedAreas {
  All,
  Stacks,
  Threads,
  Variables,
  #[serde(untagged)]
  String(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum LoadedSourceEventReason {
  New,
  Changed,
  Removed,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum ModuleEventReason {
  New,
  Changed,
  Removed,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct Module {
  /// Unique identifier for the module.
  pub id: ModuleId,
  /// A name of the module.
  pub name: String,
  /// Logical full path to the module. The exact definition is implementation
  /// defined, but usually this would be a full path to the on-disk file for the
  /// module.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,
  /// True if the module is optimized.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_optimized: Option<bool>,
  /// True if the module is considered 'user code' by a debugger that supports
  /// 'Just My Code'.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_user_code: Option<bool>,
  /// Version of Module.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
  /// User-understandable description of if symbols were found for the module
  /// (ex: 'Symbols Loaded', 'Symbols not found', etc.)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub symbol_status: Option<String>,
  /// Logical full path to the symbol file. The exact definition is
  /// implementation defined.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub symbol_file_path: Option<String>,
  /// Module created or modified, encoded as a RFC 3339 timestamp.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub date_time_stamp: Option<String>,
  /// Address range covered by this module.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub address_range: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum ModuleId {
  Number,
  #[serde(untagged)]
  String(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum OutputEventCategory {
  Console,
  Important,
  Stdout,
  Stderr,
  Telemetry,
  #[serde(untagged)]
  String(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum OutputEventGroup {
  Start,
  StartCollapsed,
  End,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum ProcessEventStartMethod {
  Launch,
  Attach,
  AttachForSuspendedLaunch,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum StoppedEventReason {
  Step,
  Breakpoint,
  Exception,
  Pause,
  Entry,
  Goto,
  Function,
  Data,
  Instruction,
  #[serde(untagged)]
  String(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum ThreadEventReason {
  Started,
  Exited,
  #[serde(untagged)]
  String(String),
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct ValueFormat {
  /// Display the value in hex.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hex: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StackFrameFormat {
  /// Displays parameters for the stack frame.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parameters: Option<bool>,
  /// Displays the types of parameters for the stack frame.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parameter_types: Option<bool>,
  /// Displays the names of parameters for the stack frame.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parameter_names: Option<bool>,
  /// Displays the values of parameters for the stack frame.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parameter_values: Option<bool>,
  /// Displays the line i64 of the stack frame.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub line: Option<bool>,
  /// Displays the module of the stack frame.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub module: Option<bool>,
  /// Includes all stack frames, including those the debug adapter might
  /// otherwise hide.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub include_all: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum EvaluateArgumentsContext {
  Variables,
  Watch,
  Repl,
  Hover,
  Clipboard,
  #[serde(untagged)]
  String(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum SteppingGranularity {
  Statement,
  Line,
  Instruction,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum DataBreakpointAccessType {
  Read,
  Write,
  ReadWrite,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct DataBreakpoint {
  /// An id representing the data. This id is returned from the
  /// `dataBreakpointInfo` request.
  pub data_id: String,
  /// The access type of the data.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub access_type: Option<DataBreakpointAccessType>,
  /// An expression for conditional breakpoints.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub condition: Option<String>,
  /// An expression that controls how many hits of the breakpoint are ignored.
  /// The debug adapter is expected to interpret the expression as needed.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hit_condition: Option<String>,
}

/// Properties of a breakpoint passed to the setInstructionBreakpoints request
///
/// Specfication: [InstructionBreakpoint](https://microsoft.github.io/debug-adapter-protocol/specification#Types_InstructionBreakpoint)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct InstructionBreakpoint {
  /// The instruction reference of the breakpoint.
  /// This should be a memory or instruction pointer reference from an
  /// `EvaluateResponse`, `Variable`, `StackFrame`, `GotoTarget`, or
  /// `Breakpoint`.
  pub instruction_reference: String,
  /// The offset from the instruction reference.
  /// This can be negative.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub offset: Option<i64>,
  /// An expression for conditional breakpoints.
  /// It is only honored by a debug adapter if the corresponding capability
  /// `supportsConditionalBreakpoints` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub condition: Option<String>,
  /// An expression that controls how many hits of the breakpoint are ignored.
  /// The debug adapter is expected to interpret the expression as needed.
  /// The attribute is only honored by a debug adapter if the corresponding
  /// capability `supportsHitConditionalBreakpoints` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hit_condition: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum VariablesArgumentsFilter {
  Indexed,
  Named,
}

/// Properties of a breakpoint location returned from the breakpointLocations request.

/// Specfication: [BreakpointLocation](https://microsoft.github.io/debug-adapter-protocol/specification#Types_BreakpointLocation)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct BreakpointLocation {
  /// Start line of breakpoint location.
  pub line: i64,
  /// The start position of a breakpoint location. Position is measured in UTF-16
  /// code units and the client capability `columnsStartAt1` determines whether
  /// it is 0- or 1-based.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<i64>,
  /// The end line of breakpoint location if the location covers a range.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_line: Option<i64>,
  /// The end position of a breakpoint location (if the location covers a range).
  /// Position is measured in UTF-16 code units and the client capability
  /// `columnsStartAt1` determines whether it is 0- or 1-based.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_column: Option<i64>,
}

/// Some predefined types for the CompletionItem. Please note that not all clients have specific
/// icons for all of them
///
/// Specification: [CompletionItemType](https://microsoft.github.io/debug-adapter-protocol/specification#Types_CompletionItemType)
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum CompletionItemType {
  Method,
  Function,
  Constructor,
  Field,
  Variable,
  Class,
  Interface,
  Module,
  Property,
  Unit,
  Value,
  Enum,
  Keyword,
  Snippet,
  Text,
  Color,
  File,
  Reference,
  CustomColor,
}

/// `CompletionItems` are the suggestions returned from the `completions` request.
///
/// Specification: [CompletionItem](https://microsoft.github.io/debug-adapter-protocol/specification#Types_CompletionItem)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct CompletionItem {
  /// The label of this completion item. By default this is also the text that is
  /// inserted when selecting this completion.
  pub label: String,
  /// If text is returned and not an empty String, then it is inserted instead of
  /// the label.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub text: Option<String>,
  /// A String that should be used when comparing this item with other items. If
  /// not returned or an empty String, the `label` is used instead.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sort_text: Option<String>,
  /// A human-readable String with additional information about this item, like
  /// type or symbol information.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub detail: Option<String>,
  /// The item's type. Typically the client uses this information to render the
  /// item in the UI with an icon.
  #[serde(rename = "type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub type_field: Option<CompletionItemType>,
  /// Start position (within the `text` attribute of the `completions` request)
  /// where the completion text is added. The position is measured in UTF-16 code
  /// units and the client capability `columnsStartAt1` determines whether it is
  /// 0- or 1-based. If the start position is omitted the text is added at the
  /// location specified by the `column` attribute of the `completions` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub start: Option<i64>,
  /// Length determines how many characters are overwritten by the completion
  /// text and it is measured in UTF-16 code units. If missing the value 0 is
  /// assumed which results in the completion text being inserted.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub length: Option<i64>,
  /// Determines the start of the new selection after the text has been inserted
  /// (or replaced). `selectionStart` is measured in UTF-16 code units and must
  /// be in the range 0 and length of the completion text. If omitted the
  /// selection starts at the end of the completion text.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub selection_start: Option<i64>,
  /// Determines the length of the new selection after the text has been inserted
  /// (or replaced) and it is measured in UTF-16 code units. The selection can
  /// not extend beyond the bounds of the completion text. If omitted the length
  /// is assumed to be 0.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub selection_length: Option<i64>,
}

/// Represents a single disassembled instruction.
///
/// Specification: [DisassembledInstruction](https://microsoft.github.io/debug-adapter-protocol/specification#Types_DisassembledInstruction)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct DisassembledInstruction {
  /// The address of the instruction. Treated as a hex value if prefixed with
  /// `0x`, or as a decimal value otherwise.
  pub address: String,
  /// Raw bytes representing the instruction and its operands, in an
  /// implementation-defined format.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instruction_bytes: Option<String>,
  /// Text representing the instruction and its operands, in an
  /// implementation-defined format.
  pub instruction: String,
  /// Name of the symbol that corresponds with the location of this instruction,
  /// if any.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub symbol: Option<String>,
  /// Source location that corresponds to this instruction, if any.
  /// Should always be set (if available) on the first instruction returned,
  /// but can be omitted afterwards if this instruction maps to the same source
  /// file as the previous instruction.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub location: Option<Source>,
  /// The line within the source location that corresponds to this instruction,
  /// if any.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub line: Option<i64>,
  /// The column within the line that corresponds to this instruction, if any.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<i64>,
  /// The end line of the range that corresponds to this instruction, if any.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_line: Option<i64>,
  /// The end column of the range that corresponds to this instruction, if any.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_column: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum VariablePresentationHintKind {
  /// Indicates that the object is a property.
  Property,
  /// Indicates that the object is a method.
  Method,
  /// Indicates that the object is a class.
  Class,
  /// Indicates that the object is data.
  Data,
  /// Indicates that the object is an event.
  Event,
  /// Indicates that the object is a base class.
  BaseClass,
  /// Indicates that the object is an inner class.
  InnerClass,
  /// Indicates that the object is an interface.
  Interface,
  /// Indicates that the object is the most derived class.
  MostDerivedClass,
  /// Indicates that the object is virtual, that means it is a
  /// synthetic object introduced by the adapter for rendering purposes, e.g. an
  /// index range for large arrays.
  Virtual,
  /// Deprecated: Indicates that a data breakpoint is
  /// registered for the object. The `hasDataBreakpoint` attribute should
  /// generally be used instead.
  DataBreakpoint,
  #[serde(untagged)]
  String(String),
}

/// Set of attributes represented as an array of Strings. Before introducing
/// additional values, try to use the listed values.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum VariablePresentationHintAttributes {
  /// Indicates that the object is static.
  Static,
  /// Indicates that the object is a constant.
  Constant,
  /// Indicates that the object is read only.
  ReadOnly,
  /// Indicates that the object is a raw String.
  RawString,
  /// Indicates that the object can have an Object ID created for it.
  HasObjectId,
  /// Indicates that the object has an Object ID associated with it.
  CanHaveObjectId,
  /// Indicates that the evaluation had side effects.
  HasSideEffects,
  /// Indicates that the object has its value tracked by a data breakpoint.
  HasDataBreakpoint,
  #[serde(untagged)]
  String(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum VariablePresentationHintVisibility {
  Public,
  Private,
  Protected,
  Internal,
  Final,
  #[serde(untagged)]
  String(String),
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
#[serde(rename_all = "camelCase")]
pub struct VariablePresentationHint {
  /// The kind of variable. Before introducing additional values, try to use the
  /// listed values.
  /// Values:
  /// 'property': Indicates that the object is a property.
  /// 'method': Indicates that the object is a method.
  /// 'class': Indicates that the object is a class.
  /// 'data': Indicates that the object is data.
  /// 'event': Indicates that the object is an event.
  /// 'baseClass': Indicates that the object is a base class.
  /// 'innerClass': Indicates that the object is an inner class.
  /// 'interface': Indicates that the object is an interface.
  /// 'mostDerivedClass': Indicates that the object is the most derived class.
  /// 'virtual': Indicates that the object is virtual, that means it is a
  /// synthetic object introduced by the adapter for rendering purposes, e.g. an
  /// index range for large arrays.
  /// 'dataBreakpoint': Deprecated: Indicates that a data breakpoint is
  /// registered for the object. The `hasDataBreakpoint` attribute should
  /// generally be used instead.
  /// etc.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub kind: Option<VariablePresentationHintKind>,
  /// Set of attributes represented as an array of Strings. Before introducing
  /// additional values, try to use the listed values.
  /// Values:
  /// 'static': Indicates that the object is static.
  /// 'constant': Indicates that the object is a constant.
  /// 'readOnly': Indicates that the object is read only.
  /// 'rawString': Indicates that the object is a raw String.
  /// 'hasObjectId': Indicates that the object can have an Object ID created for
  /// it.
  /// 'canHaveObjectId': Indicates that the object has an Object ID associated
  /// with it.
  /// 'hasSideEffects': Indicates that the evaluation had side effects.
  /// 'hasDataBreakpoint': Indicates that the object has its value tracked by a
  /// data breakpoint.
  /// etc.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub attributes: Option<Vec<VariablePresentationHintAttributes>>,
  /// Visibility of variable. Before introducing additional values, try to use
  /// the listed values.
  /// Values: 'public', 'private', 'protected', 'internal', 'final', etc.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub visibility: Option<VariablePresentationHintVisibility>,
  /// If true, clients can present the variable with a UI that supports a
  /// specific gesture to trigger its evaluation.
  /// This mechanism can be used for properties that require executing code when
  /// retrieving their value and where the code execution can be expensive and/or
  /// produce side-effects. A typical example are properties based on a getter
  /// function.
  /// Please note that in addition to the `lazy` flag, the variable's
  /// `variablesReference` is expected to refer to a variable that will provide
  /// the value through another `variable` request.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lazy: Option<bool>,
}

/// Detailed information about an exception that has occurred.
///
/// Specification: [ExceptionDetails](https://microsoft.github.io/debug-adapter-protocol/specification#Types_ExceptionDetails)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct ExceptionDetails {
  /// Message contained in the exception.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub message: Option<String>,
  /// Short type name of the exception object.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub type_name: Option<String>,
  /// Fully-qualified type name of the exception object.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub full_type_name: Option<String>,
  /// An expression that can be evaluated in the current scope to obtain the
  /// exception object.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub evaluate_name: Option<String>,
  /// Stack trace at the time the exception was thrown.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stack_trace: Option<String>,
  /// Details of the exception contained by this exception, if any.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inner_exception: Option<Vec<ExceptionDetails>>,
}

/// A `GotoTarget` describes a code location that can be used as a target in the
/// goto request.
/// The possible goto targets can be determined via the gotoTargets request.
///
/// Specification: [GotoTarget](https://microsoft.github.io/debug-adapter-protocol/specification#Types_GotoTarget)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct GotoTarget {
  /// Unique identifier for a goto target. This is used in the `goto` request.
  pub id: i64,
  /// The name of the goto target (shown in the UI).
  pub label: String,
  /// The line of the goto target.
  pub line: i64,
  /// The column of the goto target.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<i64>,
  /// The end line of the range covered by the goto target.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_line: Option<i64>,
  /// The end column of the range covered by the goto target.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_column: Option<i64>,
  /// A memory reference for the instruction pointer value represented by this
  /// target.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instruction_pointer_reference: Option<String>,
}

/// A hint for how to present this scope in the UI. If this attribute is
/// missing, the scope is shown with a generic UI.
///
/// Specification: [Scope](https://microsoft.github.io/debug-adapter-protocol/specification#Types_Scope)
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum ScopePresentationhint {
  /// Scope contains method arguments.
  Arguments,
  /// Scope contains local variables.
  Locals,
  /// Scope contains registers. Only a single `registers` scope
  /// should be returned from a `scopes` request.
  Registers,
  #[serde(untagged)]
  String(String),
}

/// A Scope is a named container for variables. Optionally a scope can map to a source or a range
/// within a source.
///
/// Specification: [Scope](https://microsoft.github.io/debug-adapter-protocol/specification#Types_Scope)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct Scope {
  /// Name of the scope such as 'Arguments', 'Locals', or 'Registers'. This
  /// String is shown in the UI as is and can be translated.
  pub name: String,
  /// A hint for how to present this scope in the UI. If this attribute is
  /// missing, the scope is shown with a generic UI.
  /// Values:
  /// 'arguments': Scope contains method arguments.
  /// 'locals': Scope contains local variables.
  /// 'registers': Scope contains registers. Only a single `registers` scope
  /// should be returned from a `scopes` request.
  /// etc.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub presentation_hint: Option<ScopePresentationhint>,
  /// The variables of this scope can be retrieved by passing the value of
  /// `variablesReference` to the `variables` request as long as execution
  /// remains suspended. See [Lifetime of Object References](https://microsoft.github.io/debug-adapter-protocol/overview#lifetime-of-objects-references)
  /// in the Overview section of the specification for details.
  pub variables_reference: i64,
  /// The i64 of named variables in this scope.
  /// The client can use this information to present the variables in a paged UI
  /// and fetch them in chunks.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub named_variables: Option<i64>,
  /// The i64 of indexed variables in this scope.
  /// The client can use this information to present the variables in a paged UI
  /// and fetch them in chunks.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub indexed_variables: Option<i64>,
  /// If true, the i64 of variables in this scope is large or expensive to
  /// retrieve.
  pub expensive: bool,
  /// The source for this scope.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source: Option<Source>,
  /// The start line of the range covered by this scope.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub line: Option<i64>,
  /// Start position of the range covered by the scope. It is measured in UTF-16
  /// code units and the client capability `columnsStartAt1` determines whether
  /// it is 0- or 1-based.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column: Option<i64>,
  /// The end line of the range covered by this scope.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_line: Option<i64>,
  /// End position of the range covered by the scope. It is measured in UTF-16
  /// code units and the client capability `columnsStartAt1` determines whether
  /// it is 0- or 1-based.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_column: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum StackFrameModuleid {
  Number(i64),
  String(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum StackFramePresentationhint {
  Normal,
  Label,
  Subtle,
}

/// A Stackframe contains the source location.
///
/// Specification: [StackFrame](https://microsoft.github.io/debug-adapter-protocol/specification#Types_StackFrame)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct StackFrame {
  /// An identifier for the stack frame. It must be unique across all threads.
  /// This id can be used to retrieve the scopes of the frame with the `scopes`
  /// request or to restart the execution of a stackframe.
  pub id: i64,
  /// The name of the stack frame, typically a method name.
  pub name: String,
  /// The source of the frame.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source: Option<Source>,
  /// The line within the source of the frame. If the source attribute is missing
  /// or doesn't exist, `line` is 0 and should be ignored by the client.
  pub line: i64,
  /// Start position of the range covered by the stack frame. It is measured in
  /// UTF-16 code units and the client capability `columnsStartAt1` determines
  /// whether it is 0- or 1-based. If attribute `source` is missing or doesn't
  /// exist, `column` is 0 and should be ignored by the client.
  pub column: i64,
  /// The end line of the range covered by the stack frame.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_line: Option<i64>,
  /// End position of the range covered by the stack frame. It is measured in
  /// UTF-16 code units and the client capability `columnsStartAt1` determines
  /// whether it is 0- or 1-based.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_column: Option<i64>,
  /// Indicates whether this frame can be restarted with the `restart` request.
  /// Clients should only use this if the debug adapter supports the `restart`
  /// request and the corresponding capability `supportsRestartRequest` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub can_restart: Option<bool>,
  /// A memory reference for the current instruction pointer in this frame.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instruction_pointer_reference: Option<String>,
  /// The module associated with this frame, if any.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub module_id: Option<StackFrameModuleid>,
  /// A hint for how to present this frame in the UI.
  /// A value of `label` can be used to indicate that the frame is an artificial
  /// frame that is used as a visual label or separator. A value of `subtle` can
  /// be used to change the appearance of a frame in a 'subtle' way.
  /// Values: 'normal', 'label', 'subtle'
  #[serde(skip_serializing_if = "Option::is_none")]
  pub presentation_hint: Option<StackFramePresentationhint>,
}

/// A thread.
///
/// Specification: [Thread](https://microsoft.github.io/debug-adapter-protocol/specification#Types_Thread)
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct Thread {
  /// Unique identifier for the thread.
  pub id: i64,
  /// The name of the thread.
  pub name: String,
}

/// A Variable is a name/value pair.
///
/// The `type` attribute is shown if space permits or when hovering over the variable’s name.
///
/// The `kind` attribute is used to render additional properties of the variable, e.g. different
/// icons can be used to indicate that a variable is public or private.
///
/// If the value is structured (has children), a handle is provided to retrieve the children with
/// the `variables` request.
///
/// If the number of named or indexed children is large, the numbers should be returned via the
/// `namedVariables` and `indexedVariables` attributes.
///
/// The client can use this information to present the children in a paged UI and fetch them in
/// chunks.
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub struct Variable {
  /// The variable's name.
  pub name: String,
  /// The variable's value.
  /// This can be a multi-line text, e.g. for a function the body of a function.
  /// For structured variables (which do not have a simple value), it is
  /// recommended to provide a one-line representation of the structured object.
  /// This helps to identify the structured object in the collapsed state when
  /// its children are not yet visible.
  /// An empty String can be used if no value should be shown in the UI.
  pub value: String,
  /// The type of the variable's value. Typically shown in the UI when hovering
  /// over the value.
  /// This attribute should only be returned by a debug adapter if the
  /// corresponding capability `supportsVariableType` is true.
  #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
  pub type_field: Option<String>,
  /// Properties of a variable that can be used to determine how to render the
  /// variable in the UI.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub presentation_hint: Option<VariablePresentationHint>,
  /// The evaluatable name of this variable which can be passed to the `evaluate`
  /// request to fetch the variable's value.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub evaluate_name: Option<String>,
  /// If `variablesReference` is > 0, the variable is structured and its children
  /// can be retrieved by passing `variablesReference` to the `variables` request
  /// as long as execution remains suspended. See [Lifetime of Object References](https://microsoft.github.io/debug-adapter-protocol/overview#lifetime-of-objects-references)
  /// in the Overview section of the specification for details.
  pub variables_reference: i64,
  /// The i64 of named child variables.
  /// The client can use this information to present the children in a paged UI
  /// and fetch them in chunks.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub named_variables: Option<i64>,
  /// The i64 of indexed child variables.
  /// The client can use this information to present the children in a paged UI
  /// and fetch them in chunks.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub indexed_variables: Option<i64>,
  /// The memory reference for the variable if the variable represents executable
  /// code, such as a function pointer.
  /// This attribute is only required if the corresponding capability
  /// `supportsMemoryReferences` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_reference: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum RunInTerminalRequestArgumentsKind {
  Integrated,
  External,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
pub enum StartDebuggingRequestKind {
  Launch,
  Attach,
}

/// A structured message object. Used to return errors from requests.
///
/// Specification: [Message](https://microsoft.github.io/debug-adapter-protocol/specification#Types_Message)
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "integration_testing", derive(Dummy))]
#[cfg_attr(feature = "client", derive(Deserialize))]
pub struct Message {
  /// Unique (within a debug adapter implementation) identifier for the message.
  /// The purpose of these error IDs is to help extension authors that have the
  /// requirement that every user visible error message needs a corresponding
  /// error i64, so that users or customer support can find information about
  /// the specific error more easily.
  pub id: i64,
  /// A format String for the message. Embedded variables have the form `{name}`.
  /// If variable name starts with an underscore character, the variable does not
  /// contain user data (PII) and can be safely used for telemetry purposes.
  pub format: String,
  /// An object used as a dictionary for looking up the variables in the format string.
  pub variables: HashMap<String, String>,
  /// An object used as a dictionary for looking up the variables in the format
  /// String.
  /// If true send to telemetry.
  pub send_telemetry: Option<bool>,
  /// If true show user.
  pub show_user: Option<bool>,
  /// A url where additional information about this message can be found.
  pub url: Option<String>,
  /// A label that is presented to the user as the UI for opening the url.
  pub url_label: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[allow(unused)]
  #[test]
  fn test_checksum_algorithm_serde() {
    let sha = ChecksumAlgorithm::SHA256;
    let sha_ser = serde_json::to_value(sha).unwrap();
    assert_eq!("SHA256", sha_ser);
    let sha_deser: ChecksumAlgorithm = serde_json::from_value(sha_ser).unwrap();
    assert!(matches!(ChecksumAlgorithm::SHA256, sha_deser));

    let ts = ChecksumAlgorithm::Timestamp;
    let ts_ser = serde_json::to_value(&ts).unwrap();
    assert_eq!("timestamp", ts_ser);
    #[allow(unused)]
    let ts_deser: ChecksumAlgorithm = serde_json::from_value(ts_ser).unwrap();
    assert!(matches!(ChecksumAlgorithm::Timestamp, ts_deser));
  }

  #[allow(unused)]
  #[test]
  fn test_invalidated_areas_serde() {
    let str = "string".to_string();
    let untagged = InvalidatedAreas::String(str.clone());
    let untagged_ser = serde_json::to_value(untagged).unwrap();
    assert_eq!(str, untagged_ser);
    let untagged_deser: InvalidatedAreas = serde_json::from_value(untagged_ser).unwrap();
    assert!(matches!(InvalidatedAreas::String(str), untagged_deser));
  }
}
