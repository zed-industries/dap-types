// This file is autogenerated. Do not edit by hand.
// To regenerate from schema, run `cargo run -p generator`.

use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

/// Event is an event, with associated name and body type.
pub trait Event {
    const EVENT: &'static str;
    type Body: Debug + Clone + Serialize + DeserializeOwned + Send + Sync;
}

/// This event indicates that the debug adapter is ready to accept configuration requests (e.g. `setBreakpoints`, `setExceptionBreakpoints`).
/// A debug adapter is expected to send this event when it is ready to accept configuration requests (but not before the `initialize` request has finished).
/// The sequence of events/requests is as follows:
/// - adapters sends `initialized` event (after the `initialize` request has returned)
/// - client sends zero or more `setBreakpoints` requests
/// - client sends one `setFunctionBreakpoints` request (if corresponding capability `supportsFunctionBreakpoints` is true)
/// - client sends a `setExceptionBreakpoints` request if one or more `exceptionBreakpointFilters` have been defined (or if `supportsConfigurationDoneRequest` is not true)
/// - client sends other future configuration requests
/// - client sends one `configurationDone` request to indicate the end of the configuration.
pub enum Initialized {}

impl Event for Initialized {
    const EVENT: &'static str = "initialized";
    type Body = Option<crate::Capabilities>;
}

/// The event indicates that the execution of the debuggee has stopped due to some condition.
/// This can be caused by a breakpoint previously set, a stepping request has completed, by executing a debugger statement etc.
pub enum Stopped {}

impl Event for Stopped {
    const EVENT: &'static str = "stopped";
    type Body = crate::StoppedEvent;
}

/// The event indicates that the execution of the debuggee has continued.
/// Please note: a debug adapter is not expected to send this event in response to a request that implies that execution continues, e.g. `launch` or `continue`.
/// It is only necessary to send a `continued` event if there was no previous request that implied this.
pub enum Continued {}

impl Event for Continued {
    const EVENT: &'static str = "continued";
    type Body = crate::ContinuedEvent;
}

/// The event indicates that the debuggee has exited and returns its exit code.
pub enum Exited {}

impl Event for Exited {
    const EVENT: &'static str = "exited";
    type Body = crate::ExitedEvent;
}

/// The event indicates that debugging of the debuggee has terminated. This does **not** mean that the debuggee itself has exited.
pub enum Terminated {}

impl Event for Terminated {
    const EVENT: &'static str = "terminated";
    type Body = crate::TerminatedEvent;
}

/// The event indicates that a thread has started or exited.
pub enum Thread {}

impl Event for Thread {
    const EVENT: &'static str = "thread";
    type Body = crate::ThreadEvent;
}

/// The event indicates that the target has produced some output.
pub enum Output {}

impl Event for Output {
    const EVENT: &'static str = "output";
    type Body = crate::OutputEvent;
}

/// The event indicates that some information about a breakpoint has changed.
pub enum Breakpoint {}

impl Event for Breakpoint {
    const EVENT: &'static str = "breakpoint";
    type Body = crate::BreakpointEvent;
}

/// The event indicates that some information about a module has changed.
pub enum Module {}

impl Event for Module {
    const EVENT: &'static str = "module";
    type Body = crate::ModuleEvent;
}

/// The event indicates that some source has been added, changed, or removed from the set of all loaded sources.
pub enum LoadedSource {}

impl Event for LoadedSource {
    const EVENT: &'static str = "loadedSource";
    type Body = crate::LoadedSourceEvent;
}

/// The event indicates that the debugger has begun debugging a new process. Either one that it has launched, or one that it has attached to.
pub enum Process {}

impl Event for Process {
    const EVENT: &'static str = "process";
    type Body = crate::ProcessEvent;
}

/// The event indicates that one or more capabilities have changed.
/// Since the capabilities are dependent on the client and its UI, it might not be possible to change that at random times (or too late).
/// Consequently this event has a hint characteristic: a client can only be expected to make a 'best effort' in honoring individual capabilities but there are no guarantees.
/// Only changed capabilities need to be included, all other capabilities keep their values.
pub enum Capabilities {}

impl Event for Capabilities {
    const EVENT: &'static str = "capabilities";
    type Body = crate::CapabilitiesEvent;
}

/// The event signals that a long running operation is about to start and provides additional information for the client to set up a corresponding progress and cancellation UI.
/// The client is free to delay the showing of the UI in order to reduce flicker.
/// This event should only be sent if the corresponding capability `supportsProgressReporting` is true.
pub enum ProgressStart {}

impl Event for ProgressStart {
    const EVENT: &'static str = "progressStart";
    type Body = crate::ProgressStartEvent;
}

/// The event signals that the progress reporting needs to be updated with a new message and/or percentage.
/// The client does not have to update the UI immediately, but the clients needs to keep track of the message and/or percentage values.
/// This event should only be sent if the corresponding capability `supportsProgressReporting` is true.
pub enum ProgressUpdate {}

impl Event for ProgressUpdate {
    const EVENT: &'static str = "progressUpdate";
    type Body = crate::ProgressUpdateEvent;
}

/// The event signals the end of the progress reporting with a final message.
/// This event should only be sent if the corresponding capability `supportsProgressReporting` is true.
pub enum ProgressEnd {}

impl Event for ProgressEnd {
    const EVENT: &'static str = "progressEnd";
    type Body = crate::ProgressEndEvent;
}

/// This event signals that some state in the debug adapter has changed and requires that the client needs to re-render the data snapshot previously requested.
/// Debug adapters do not have to emit this event for runtime changes like stopped or thread events because in that case the client refetches the new state anyway. But the event can be used for example to refresh the UI after rendering formatting has changed in the debug adapter.
/// This event should only be sent if the corresponding capability `supportsInvalidatedEvent` is true.
pub enum Invalidated {}

impl Event for Invalidated {
    const EVENT: &'static str = "invalidated";
    type Body = crate::InvalidatedEvent;
}

/// This event indicates that some memory range has been updated. It should only be sent if the corresponding capability `supportsMemoryEvent` is true.
/// Clients typically react to the event by re-issuing a `readMemory` request if they show the memory identified by the `memoryReference` and if the updated memory range overlaps the displayed range. Clients should not make assumptions how individual memory references relate to each other, so they should not assume that they are part of a single continuous address range and might overlap.
/// Debug adapters can use this event to indicate that the contents of a memory range has changed due to some other request like `setVariable` or `setExpression`. Debug adapters are not expected to emit this event for each and every memory change of a running program, because that information is typically not available from debuggers and it would flood clients with too many events.
pub enum Memory {}

impl Event for Memory {
    const EVENT: &'static str = "memory";
    type Body = crate::MemoryEvent;
}
