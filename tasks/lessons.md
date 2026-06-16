# Lessons Learned

## Tool misuse: ScheduleWakeup is /loop-only

**Failure mode:** Called `ScheduleWakeup` as a generic "wait for background tasks" fallback while three `cargo nextest` jobs ran. The tool is specifically for `/loop` dynamic mode — its fire prompt is interpreted as a `/loop` continuation. When the wakeup fired, the harness re-entered the `/loop` skill with no real iteration to do.

**Detection signal:** Received a `<command-name>/loop</command-name>` invocation that I hadn't been asked for, with the prompt I'd passed to `ScheduleWakeup`.

**Prevention rule:** Only call `ScheduleWakeup` when actually inside `/loop` dynamic mode. The harness already notifies on background-task completion via `<task-notification>` — no fallback is needed for `Bash run_in_background` or `Agent run_in_background` work.
