# Threading model: the compiler actor

The LSP server (`tooling/lsp`) runs its `async_lsp` main loop on a single-threaded tokio
runtime. All compiler state — source overrides, parse caches, and the per-package
type-checked caches (`NodeInterner`, `CrateDefMap`s, `CrateGraph`) — is owned by a
"compiler actor": a dedicated thread with an in-order message queue
(`tooling/lsp/src/actor.rs`). The main loop's only state is a handle to the actor; every
request and notification is forwarded to it as a message, and request replies travel back
over oneshot channels.

Reasons for this design:

- **Notifications must be fast.** Handling a notification (e.g. `didSave`) synchronously on
  the main loop while a type-check runs blocks the whole protocol stream: the editor shows
  files as unsaved and requests time out (issue #11349). With the actor, the main loop only
  enqueues.
- **Compiler state cannot cross threads.** `Type` transitively contains `Rc`
  (`Shared<T>(Rc<RefCell<T>>)`, `Rc<String>` names), so `NodeInterner` is `!Send`. That
  rules out "type-check on a worker thread and hand the interner back". The actor sidesteps
  this: the state is created on the actor thread and never leaves it; only messages (jobs
  and their `Send` results) cross the boundary.
- **In-order processing gives freshness without bookkeeping.** A request enqueued after a
  document change is processed after the re-check that change triggered, so it always sees
  up-to-date state. An earlier attempt (PR #11356) instead deferred work with counters, a
  pending-request queue and version-based cancellation; leaked counters and message
  reordering caused unrecoverable hangs. The FIFO queue makes those states unrepresentable.

Details that matter:

- **Coalescing.** The actor drains its queue before processing and merges *adjacent*
  `didChange` messages for the same document (full-document sync means the last change
  carries the whole text). Only adjacent messages are merged so a request ordered between
  two changes still observes the text of the change before it.
- **Panic containment.** Each message is processed under `catch_unwind`. A panicking job
  (e.g. an ICE during type-checking) drops its reply channel — the request resolves with an
  error instead of hanging — and the actor keeps serving. Caches a panic left incomplete are
  rebuilt from sources the next time they are found missing.
- **wasm.** On `wasm32` there are no threads; the actor falls back to processing messages
  inline on the caller's thread (the pre-actor behavior). Unit tests construct `LspState`
  directly and call handlers synchronously, bypassing the actor.
- **Blocking notification handlers no longer kill the server.** Before the actor, a
  notification handler returning `ControlFlow::Break(Err)` terminated the main loop. Errors
  from forwarded notifications are reported to stderr instead.
