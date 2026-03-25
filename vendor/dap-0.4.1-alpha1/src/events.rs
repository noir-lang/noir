use serde::Serialize;
use serde_json::Value;

#[cfg(feature = "client")]
use serde::Deserialize;

use crate::types::{
  Breakpoint, BreakpointEventReason, Capabilities, InvalidatedAreas, LoadedSourceEventReason,
  Module, ModuleEventReason, OutputEventCategory, OutputEventGroup, ProcessEventStartMethod,
  Source, StoppedEventReason, ThreadEventReason,
};

//// Arguments for a Breakpoint event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BreakpointEventBody {
  /// The reason for the event.
  /// Values: 'changed', 'new', 'removed', etc.
  pub reason: BreakpointEventReason,
  /// The `id` attribute is used to find the target breakpoint, the other
  /// attributes are used as the new values.
  pub breakpoint: Breakpoint,
}

//// Arguments for a Capabilities event
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesEventBody {
  pub capabilities: Capabilities,
}

//// Arguments for a Continued event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContinuedEventBody {
  /// The thread which was continued.
  pub thread_id: i64,
  /// If `allThreadsContinued` is true, a debug adapter can announce that all threads have
  /// continued.
  pub all_threads_continued: Option<bool>,
}

//// Arguments for a Exited event
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExitedEventBody {
  /// The exit code returned from the debuggee.
  pub exit_code: i64,
}

//// Arguments for a Invalidated event
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvalidatedEventBody {
  /// Set of logical areas that got invalidated. This property has a hint
  /// characteristic: a client can only be expected to make a 'best effort' in
  /// honouring the areas but there are no guarantees. If this property is
  /// missing, empty, or if values are not understood, the client should assume
  /// a single value `all`.
  pub areas: Option<Vec<InvalidatedAreas>>,
  /// If specified, the client only needs to refetch data related to this
  /// thread.
  pub thread_id: Option<i64>,
  /// If specified, the client only needs to refetch data related to this stack
  /// frame (and the `threadId` is ignored).
  pub stack_frame_id: Option<i64>,
}

//// Arguments for a LoadedSource event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadedSourceEventBody {
  /// The reason for the event.
  /// Values: 'new', 'changed', 'removed'
  pub reason: LoadedSourceEventReason,
  /// The new, changed, or removed source.
  pub source: Source,
}

//// Arguments for a Memory event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEventBody {
  /// Memory reference of a memory range that has been updated.
  pub memory_reference: String,
  /// Starting offset in bytes where memory has been updated. Can be negative.
  pub offset: i64,
  /// Number of bytes updated.
  pub count: i64,
}

//// Arguments for a Module event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModuleEventBody {
  /// The reason for the event.
  /// Values: 'new', 'changed', 'removed'
  pub reason: ModuleEventReason,
  /// The new, changed, or removed module. In case of `removed` only the module
  /// id is used.
  pub module: Module,
}

//// Arguments for an Output event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutputEventBody {
  /// The output category. If not specified or if the category is not
  /// understood by the client, `console` is assumed.
  /// Values:
  /// 'console': Show the output in the client's default message UI, e.g. a
  /// 'debug console'. This category should only be used for informational
  /// output from the debugger (as opposed to the debuggee).
  /// 'important': A hint for the client to show the output in the client's UI
  /// for important and highly visible information, e.g. as a popup
  /// notification. This category should only be used for important messages
  /// from the debugger (as opposed to the debuggee). Since this category value
  /// is a hint, clients might ignore the hint and assume the `console`
  /// category.
  /// 'stdout': Show the output as normal program output from the debuggee.
  /// 'stderr': Show the output as error program output from the debuggee.
  /// 'telemetry': Send the output to telemetry instead of showing it to the
  /// user.
  /// etc.
  pub category: Option<OutputEventCategory>,
  /// The output to report.
  pub output: String,
  /// Support for keeping an output log organized by grouping related messages.
  /// Values:
  /// 'start': Start a new group in expanded mode. Subsequent output events are
  /// members of the group and should be shown indented.
  /// The `output` attribute becomes the name of the group and is not indented.
  /// 'startCollapsed': Start a new group in collapsed mode. Subsequent output
  /// events are members of the group and should be shown indented (as soon as
  /// the group is expanded).
  /// The `output` attribute becomes the name of the group and is not indented.
  /// 'end': End the current group and decrease the indentation of subsequent
  /// output events.
  /// A non-empty `output` attribute is shown as the unindented end of the
  /// group.
  pub group: Option<OutputEventGroup>,
  /// If an attribute `variablesReference` exists and its value is > 0, the
  /// output contains objects which can be retrieved by passing
  /// `variablesReference` to the `variables` request. The value should be less
  /// than or equal to 2147483647 (2^31-1).
  pub variables_reference: Option<i64>,
  /// The source location where the output was produced.
  pub source: Option<Source>,
  /// The source location's line where the output was produced.
  pub line: Option<i64>,
  /// The position in `line` where the output was produced. It is measured in
  /// UTF-16 code units and the client capability `columnsStartAt1` determines
  /// whether it is 0- or 1-based.
  pub column: Option<i64>,
  /// Additional data to report. For the `telemetry` category the data is sent
  /// to telemetry, for the other categories the data is shown in JSON format.
  pub data: Option<Value>,
}

//// Arguments for an Process event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessEventBody {
  /// The logical name of the process. This is usually the full path to
  /// process's executable file. Example: /home/example/myproj/program.js.
  pub name: String,
  /// The system process id of the debugged process. This property is missing
  /// for non-system processes.
  pub system_process_id: Option<i64>,
  /// If true, the process is running on the same computer as the debug
  /// adapter.
  pub is_local_process: Option<bool>,
  /// Describes how the debug engine started debugging this process.
  /// Values:
  /// 'launch': Process was launched under the debugger.
  /// 'attach': Debugger attached to an existing process.
  /// 'attachForSuspendedLaunch': A project launcher component has launched a
  /// new process in a suspended state and then asked the debugger to attach.
  pub start_method: Option<ProcessEventStartMethod>,
  /// The size of a pointer or address for this process, in bits. This value
  /// may be used by clients when formatting addresses for display.
  pub pointer_size: Option<i64>,
}

//// Arguments for a ProgressEnd event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEndEventBody {
  /// The ID that was introduced in the initial `ProgressStartEvent`.
  pub progress_id: String,
  /// More detailed progress message. If omitted, the previous message (if any)
  /// is used.
  pub message: Option<String>,
}

//// Arguments for a ProgressStart event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressStartEventBody {
  /// An ID that can be used in subsequent `progressUpdate` and `progressEnd`
  /// events to make them refer to the same progress reporting.
  /// IDs must be unique within a debug session.
  pub progress_id: String,
  /// Short title of the progress reporting. Shown in the UI to describe the
  /// long running operation.
  pub title: String,
  /// The request ID that this progress report is related to. If specified a
  /// debug adapter is expected to emit progress events for the long running
  /// request until the request has been either completed or cancelled.
  /// If the request ID is omitted, the progress report is assumed to be
  /// related to some general activity of the debug adapter.
  pub request_id: Option<i64>,
  /// If true, the request that reports progress may be cancelled with a
  /// `cancel` request.
  /// So this property basically controls whether the client should use UX that
  /// supports cancellation.
  /// Clients that don't support cancellation are allowed to ignore the
  /// setting.
  pub cancellable: Option<bool>,
  /// More detailed progress message.
  pub message: Option<String>,
  /// Progress percentage to display (value range: 0 to 100). If omitted no
  /// percentage is shown.
  pub percentage: Option<i64>,
}

//// Arguments for a ProgressUpdate event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressUpdateEventBody {
  /// The ID that was introduced in the initial `progressStart` event.
  pub progress_id: String,
  /// More detailed progress message. If omitted, the previous message (if any)
  /// is used.
  pub message: Option<String>,
  /// Progress percentage to display (value range: 0 to 100). If omitted no
  /// percentage is shown.
  pub percentage: Option<i64>,
}

//// Arguments for a Stopped event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoppedEventBody {
  /// The reason for the event.
  /// For backward compatibility this String is shown in the UI if the
  /// `description` attribute is missing (but it must not be translated).
  /// Values: 'step', 'breakpoint', 'exception', 'pause', 'entry', 'goto',
  /// 'function breakpoint', 'data breakpoint', 'instruction breakpoint', etc.
  pub reason: StoppedEventReason,
  /// The full reason for the event, e.g. 'Paused on exception'. This String is
  /// shown in the UI as is and can be translated.
  pub description: Option<String>,
  /// The thread which was stopped.
  pub thread_id: Option<i64>,
  /// A value of true hints to the client that this event should not change the
  /// focus.
  pub preserve_focus_hint: Option<bool>,
  /// Additional information. E.g. if reason is `exception`, text contains the
  /// exception name. This String is shown in the UI.
  pub text: Option<String>,
  /// If `allThreadsStopped` is true, a debug adapter can announce that all
  /// threads have stopped.
  /// - The client should use this information to enable that all threads can
  /// be expanded to access their stacktraces.
  /// - If the attribute is missing or false, only the thread with the given
  /// `threadId` can be expanded.
  pub all_threads_stopped: Option<bool>,
  /// Ids of the breakpoints that triggered the event. In most cases there is
  /// only a single breakpoint but here are some examples for multiple
  /// breakpoints:
  /// - Different types of breakpoints map to the same location.
  /// - Multiple source breakpoints get collapsed to the same instruction by
  /// the compiler/runtime.
  /// - Multiple function breakpoints with different function names map to the
  /// same location.
  pub hit_breakpoint_ids: Option<Vec<i64>>,
}

//// Arguments for a Terminated event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TerminatedEventBody {
  /// A debug adapter may set `restart` to true (or to an arbitrary object) to
  /// request that the client restarts the session.
  /// The value is not interpreted by the client and passed unmodified as an
  /// attribute `__restart` to the `launch` and `attach` requests.
  pub restart: Option<Value>,
}

//// Arguments for a Thread event.
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ThreadEventBody {
  /// The reason for the event.
  /// Values: 'started', 'exited', etc.
  pub reason: ThreadEventReason,
  /// The identifier of the thread.
  pub thread_id: i64,
}

#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "event", content = "body", rename_all = "camelCase")]
pub enum Event {
  /// This event indicates that the debug adapter is ready to accept configuration requests (e.g.
  /// `setBreakpoints`, `setExceptionBreakpoints`).
  /// A debug adapter is expected to send this event when it is ready to accept configuration
  /// requests (but not before the initialize request has finished).
  ///
  /// Specification: [Initialized event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Initialized)
  Initialized,
  /// The event indicates that one or more capabilities have changed.
  /// Since the capabilities are dependent on the client and its UI, it might not be possible to
  /// change that at random times (or too late).
  /// Consequently this event has a hint characteristic: a client can only be expected to make a
  /// ‘best effort’ in honouring individual capabilities but there are no guarantees.
  /// Only changed capabilities need to be included, all other capabilities keep their values.
  ///
  /// Specification: [Capabilities event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Capabilities)
  Capabilities(CapabilitiesEventBody),
  /// The event indicates that some information about a breakpoint has changed.
  ///
  /// Specification: [Breakpoint event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Breakpoint)
  Breakpoint(BreakpointEventBody),
  /// The event indicates that the execution of the debuggee has continued.
  /// Please note: a debug adapter is not expected to send this event in response to a request that
  ///  implies that execution continues, e.g. launch or continue.
  /// It is only necessary to send a continued event if there was no previous request that implied this.
  ///
  /// Specification: [Continued event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Continued)
  Continued(ContinuedEventBody),
  /// The event indicates that the debuggee has exited and returns its exit code.
  ///
  /// Specification: [Exited event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Exited)
  Exited(ExitedEventBody),
  /// This event signals that some state in the debug adapter has changed and requires that the
  /// client needs to re-render the data snapshot previously requested.
  /// Debug adapters do not have to emit this event for runtime changes like stopped or thread
  /// events because in that case the client refetches the new state anyway. But the event can be
  /// used for example to refresh the UI after rendering formatting has changed in the debug adapter.
  ///
  /// Specification: [Invalidated event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Invalidated)
  Invalidated(InvalidatedEventBody),
  /// The event indicates that some source has been added, changed, or removed from the set of all
  /// loaded sources.
  ///
  /// Specification: [LoadedSource event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_LoadedSource)
  LoadedSource(LoadedSourceEventBody),
  /// This event indicates that some memory range has been updated. It should only be sent if the
  /// corresponding capability supportsMemoryEvent is true.
  /// Clients typically react to the event by re-issuing a readMemory request if they show the
  /// memory identified by the memoryReference and if the updated memory range overlaps the
  /// displayed range. Clients should not make assumptions how individual memory references relate
  /// to each other, so they should not assume that they are part of a single continuous address
  /// range and might overlap.
  ///
  /// Specification: [Memory event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Memory)
  Memory(MemoryEventBody),
  /// The event indicates that some information about a module has changed.
  ///
  /// Specification: [Module event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Module)
  Module(ModuleEventBody),
  /// The event indicates that the target has produced some output.
  ///
  /// Specification: [Output event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Output)
  Output(OutputEventBody),
  /// The event indicates that the debugger has begun debugging a new process. Either one that it
  /// has launched, or one that it has attached to.
  ///
  /// Specification: [Process event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Process)
  Process(ProcessEventBody),
  /// The event signals the end of the progress reporting with a final message.
  /// This event should only be sent if the corresponding capability supportsProgressReporting is
  /// true.
  ///
  /// Specification: [ProgressEnd event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_ProgressEnd)
  ProgressEnd(ProgressEndEventBody),
  /// The event signals that a long running operation is about to start and provides additional
  /// information for the client to set up a corresponding progress and cancellation UI.
  /// The client is free to delay the showing of the UI in order to reduce flicker.
  /// This event should only be sent if the corresponding capability `supportsProgressReporting` is
  /// true.
  ///
  /// Specification: [ProgressStart event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_ProgressStart)
  ProgressStart(ProgressStartEventBody),
  /// The event signals that the progress reporting needs to be updated with a new message and/or
  /// percentage.
  /// The client does not have to update the UI immediately, but the clients needs to keep track of
  /// the message and/or percentage values.
  /// This event should only be sent if the corresponding capability supportsProgressReporting is
  /// true.
  ///
  /// Specification: [ProgressUpdate event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_ProgressUpdate)
  ProgressUpdate(ProgressUpdateEventBody),
  /// The event indicates that the execution of the debuggee has stopped due to some condition.
  /// This can be caused by a breakpoint previously set, a stepping request has completed, by
  /// executing a debugger statement etc.
  ///
  /// Specification: [Stopped event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Stopped)
  Stopped(StoppedEventBody),
  /// The event indicates that debugging of the debuggee has terminated. This does not mean that
  /// the debuggee itself has exited.
  ///
  /// Specification: [Terminated event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Terminated)
  Terminated(Option<TerminatedEventBody>),
  /// The event indicates that a thread has started or exited.
  ///
  /// Specification: [Thread event](https://microsoft.github.io/debug-adapter-protocol/specification#Events_Thread)
  Thread(ThreadEventBody),
}
