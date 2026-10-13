//! Parsing and aggregation of the JSON span logs that `nargo` emits when run
//! with `NARGO_LOG_DIR=<dir> NOIR_LOG=trace`.
//!
//! Those logs contain one JSON object per line. The interesting events are
//! span `enter`/`close` events emitted by `tracing_subscriber` with
//! `FmtSpan::ENTER | FmtSpan::CLOSE`:
//!
//! - Every event carries `span` (the span's name and fields) and `spans`
//!   (the full ancestor chain, which on `enter` includes the span itself and
//!   on `close` excludes it). The ancestor chain is authoritative, so the
//!   call tree can be reconstructed without a stack machine.
//! - `close` events fire exactly once per span and carry `time.busy` and
//!   `time.idle` (total entered / open-but-not-entered time) as 3-significant
//!   -digit strings such as `"5.11ms"`.
//! - `enter` events may fire multiple times for one span (a span can be
//!   exited and re-entered without closing), so they are only used to get an
//!   exact first-enter timestamp per open span.
//!
//! From these we produce:
//!
//! - Folded stack lines (`frame;frame;... value_in_us`) suitable for
//!   `inferno`, weighted by *self* busy time. A node's un-instrumented time
//!   is made explicit as a `(self)` child frame, and any wall-clock time not
//!   covered by a root span becomes a top-level `(untracked)` frame.
//! - A list of timeline events (one per span instance) for a
//!   chrome-trace/Perfetto timeline.
//!
//! Spans that run on different threads appear as separate roots (spans do
//! not follow threads), e.g. `compile_program` is not a tracing-child of
//! `start_cli` even though it happens within it. Root spans are therefore
//! re-parented under the tightest other root span whose time interval
//! contains them.

use std::collections::HashMap;

use serde_json::Value;

/// One span instance for the chrome-trace timeline.
pub(crate) struct TimelineEvent {
    /// Rendered frame name of the span itself (not the full path).
    pub(crate) name: String,
    /// Absolute microsecond timestamp of the first `enter`.
    pub(crate) start_us: u64,
    /// Open duration (first enter to close) in microseconds.
    pub(crate) dur_us: u64,
}

/// A closed depth-1 span instance, used to re-parent cross-thread roots.
struct RootInstance {
    frame: String,
    start_us: u64,
    end_us: u64,
}

/// Streaming processor for span-log lines.
pub(crate) struct SpanLogProcessor {
    /// Timeline events shorter than this are dropped (the folded stacks are
    /// unaffected).
    timeline_min_us: u64,
    /// Inclusive busy time in microseconds, summed per span path.
    folded: HashMap<Vec<String>, f64>,
    /// First-enter absolute timestamp of each currently-open span path.
    open: HashMap<Vec<String>, u64>,
    timeline: Vec<TimelineEvent>,
    roots: Vec<RootInstance>,
    min_ts_us: Option<u64>,
    max_ts_us: Option<u64>,
    closed_spans: usize,
    /// Lines that could not be parsed as enter/close span events.
    skipped_lines: usize,
}

/// Aggregated results over a whole span log.
pub(crate) struct SpanLogReport {
    /// Sorted folded stack lines (`a;b;c 123`), self-time weighted,
    /// including `(self)` and `(untracked)` frames.
    pub(crate) folded_lines: Vec<String>,
    /// Timeline events with `start_us` normalized so the earliest event
    /// starts at 0, sorted by start time.
    pub(crate) timeline: Vec<TimelineEvent>,
    /// Wall-clock time covered by the log (first to last event).
    pub(crate) wall_us: u64,
    /// Sum of all folded line values; should approximate `wall_us`.
    pub(crate) folded_total_us: u64,
    /// Number of span close events processed.
    pub(crate) closed_spans: usize,
    pub(crate) skipped_lines: usize,
    /// Human-readable anomalies (negative self time, unclosed spans, ...).
    pub(crate) warnings: Vec<String>,
}

impl SpanLogProcessor {
    pub(crate) fn new(timeline_min_us: u64) -> Self {
        Self {
            timeline_min_us,
            folded: HashMap::new(),
            open: HashMap::new(),
            timeline: Vec::new(),
            roots: Vec::new(),
            min_ts_us: None,
            max_ts_us: None,
            closed_spans: 0,
            skipped_lines: 0,
        }
    }

    /// Consume one line of the JSON log.
    pub(crate) fn ingest_line(&mut self, line: &str) {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            self.skipped_lines += 1;
            return;
        };
        let message = value.pointer("/fields/message").and_then(Value::as_str);
        let timestamp = value.get("timestamp").and_then(Value::as_str).and_then(parse_timestamp_us);
        let ancestors = value.get("spans").and_then(Value::as_array);

        let (Some(message), Some(ts)) = (message, timestamp) else {
            self.skipped_lines += 1;
            return;
        };

        match message {
            "enter" => {
                let Some(ancestors) = ancestors else {
                    self.skipped_lines += 1;
                    return;
                };
                let path: Vec<String> =
                    ancestors.iter().filter_map(Value::as_object).map(render_frame).collect();
                if path.is_empty() {
                    self.skipped_lines += 1;
                    return;
                }
                self.observe_ts(ts);
                // A span can be exited and re-entered without closing; only
                // the first enter marks the instance's start.
                self.open.entry(path).or_insert(ts);
            }
            "close" => {
                let span = value.get("span").and_then(Value::as_object);
                let busy = value
                    .pointer("/fields/time.busy")
                    .and_then(Value::as_str)
                    .and_then(parse_duration_us);
                let (Some(ancestors), Some(span), Some(busy)) = (ancestors, span, busy) else {
                    self.skipped_lines += 1;
                    return;
                };
                let idle = value
                    .pointer("/fields/time.idle")
                    .and_then(Value::as_str)
                    .and_then(parse_duration_us)
                    .unwrap_or(0.0);

                let mut path: Vec<String> =
                    ancestors.iter().filter_map(Value::as_object).map(render_frame).collect();
                path.push(render_frame(span));

                // Prefer the exact first-enter timestamp; fall back to the
                // (3-significant-digit) open time reported on the close.
                let start = self
                    .open
                    .remove(&path)
                    .unwrap_or_else(|| ts.saturating_sub((busy + idle).round() as u64));
                self.observe_ts(start);
                self.observe_ts(ts);
                self.closed_spans += 1;

                *self.folded.entry(path.clone()).or_insert(0.0) += busy;

                let dur_us = ts.saturating_sub(start);
                if dur_us >= self.timeline_min_us {
                    self.timeline.push(TimelineEvent {
                        name: path.last().expect("path contains the span itself").clone(),
                        start_us: start,
                        dur_us,
                    });
                }
                if path.len() == 1 {
                    self.roots.push(RootInstance {
                        frame: path.into_iter().next().expect("checked length"),
                        start_us: start,
                        end_us: ts,
                    });
                }
            }
            _ => self.skipped_lines += 1,
        }
    }

    fn observe_ts(&mut self, ts: u64) {
        self.min_ts_us = Some(self.min_ts_us.map_or(ts, |min| min.min(ts)));
        self.max_ts_us = Some(self.max_ts_us.map_or(ts, |max| max.max(ts)));
    }

    /// Aggregate everything ingested so far into a report.
    pub(crate) fn finish(mut self) -> SpanLogReport {
        let mut warnings = Vec::new();
        for path in self.open.keys() {
            warnings.push(format!("span '{}' was entered but never closed", path.join(";")));
        }

        // Spans do not follow threads, so work spawned on another thread
        // shows up as an extra root. Re-parent each root frame under the
        // tightest other root instance whose time interval contains it.
        let prefixes = root_prefixes(&self.roots);
        let folded: HashMap<Vec<String>, f64> = std::mem::take(&mut self.folded)
            .into_iter()
            .map(|(path, busy)| {
                let mut new_path = prefixes.get(&path[0]).cloned().unwrap_or_default();
                new_path.extend(path);
                (new_path, busy)
            })
            .collect();

        // Sum each node's direct children so its self time can be computed.
        let mut child_sum: HashMap<&[String], f64> = HashMap::new();
        for (path, busy) in &folded {
            if path.len() > 1 {
                *child_sum.entry(&path[..path.len() - 1]).or_insert(0.0) += busy;
            }
        }

        let mut folded_lines = Vec::new();
        let mut folded_total_us = 0u64;
        let mut top_total = 0.0;
        for (path, busy) in &folded {
            if path.len() == 1 {
                top_total += busy;
            }
            let children = child_sum.get(path.as_slice()).copied();
            let (suffix, value) = match children {
                Some(children_busy) => {
                    let self_time = busy - children_busy;
                    // The 3-significant-digit durations make small negative
                    // self times normal; large ones mean lost parent time
                    // (e.g. children running while the parent is idle).
                    if self_time < -(busy * 0.02 + 50.0) {
                        warnings.push(format!(
                            "children of '{}' sum to {children_busy:.0}µs, more than the span's own {busy:.0}µs",
                            path.join(";")
                        ));
                    }
                    (";(self)", self_time.max(0.0))
                }
                None => ("", *busy),
            };
            let value = value.round() as u64;
            if value > 0 {
                folded_total_us += value;
                folded_lines.push(format!("{}{suffix} {value}", path.join(";")));
            }
        }

        let min_ts = self.min_ts_us.unwrap_or(0);
        let wall_us = self.max_ts_us.unwrap_or(0) - min_ts;

        // Wall-clock time not covered by any root span (e.g. work before the
        // first span or between spans) must stay visible.
        let untracked = wall_us.saturating_sub(top_total.round() as u64);
        if untracked > 0 {
            folded_total_us += untracked;
            folded_lines.push(format!("(untracked) {untracked}"));
        }

        folded_lines.sort();

        let mut timeline = self.timeline;
        for event in &mut timeline {
            event.start_us -= min_ts;
        }
        timeline.sort_by_key(|event| (event.start_us, std::cmp::Reverse(event.dur_us)));

        SpanLogReport {
            folded_lines,
            timeline,
            wall_us,
            folded_total_us,
            closed_spans: self.closed_spans,
            skipped_lines: self.skipped_lines,
            warnings,
        }
    }
}

/// For each root frame that has a containing root instance, the chain of
/// containing frames (outermost first) to prepend to its paths.
fn root_prefixes(roots: &[RootInstance]) -> HashMap<String, Vec<String>> {
    // First instance of each frame decides its parent; in practice all
    // instances of a frame (e.g. `compile_program` once per package) sit
    // under the same root.
    let mut parent_of: HashMap<&str, &str> = HashMap::new();
    for root in roots {
        if parent_of.contains_key(root.frame.as_str()) {
            continue;
        }
        let container = roots
            .iter()
            .filter(|other| {
                other.frame != root.frame
                    && other.start_us <= root.start_us
                    && root.end_us <= other.end_us
                    && (other.end_us - other.start_us) > (root.end_us - root.start_us)
            })
            .min_by_key(|other| other.end_us - other.start_us);
        if let Some(container) = container {
            parent_of.insert(&root.frame, &container.frame);
        }
    }

    let mut prefixes = HashMap::new();
    for frame in parent_of.keys() {
        let mut chain = Vec::new();
        let mut current = *frame;
        while let Some(parent) = parent_of.get(current) {
            if chain.contains(&parent.to_string()) {
                break; // containment cannot cycle, but guard anyway
            }
            chain.push(parent.to_string());
            current = parent;
        }
        chain.reverse();
        prefixes.insert(frame.to_string(), chain);
    }
    prefixes
}

/// Parse a `tracing_subscriber` duration string (`"740ns"`, `"103µs"`,
/// `"5.11ms"`, `"36.0s"`) into microseconds.
fn parse_duration_us(s: &str) -> Option<f64> {
    let unit_start = s.find(|c: char| !c.is_ascii_digit() && c != '.')?;
    let number: f64 = s[..unit_start].parse().ok()?;
    let scale = match &s[unit_start..] {
        "ns" => 1e-3,
        "µs" | "us" => 1.0,
        "ms" => 1e3,
        "s" => 1e6,
        _ => return None,
    };
    Some(number * scale)
}

/// Parse an RFC3339 timestamp (`"2026-07-07T12:29:08.257380Z"`) into absolute
/// microseconds.
fn parse_timestamp_us(s: &str) -> Option<u64> {
    let timestamp = chrono::DateTime::parse_from_rfc3339(s).ok()?.timestamp_micros();
    u64::try_from(timestamp).ok()
}

/// Render a span object (`{"name": "ssa_pass", "pass": "mem2reg (1)"}`) as a
/// single flamegraph frame (`ssa_pass{pass=mem2reg (1)}`).
///
/// `;` separates frames in the folded format, so any `;` in field values is
/// replaced.
fn render_frame(span: &serde_json::Map<String, Value>) -> String {
    let name = span.get("name").and_then(Value::as_str).unwrap_or("<unnamed>");
    let mut fields: Vec<(&String, &Value)> =
        span.iter().filter(|(key, _)| key.as_str() != "name").collect();
    fields.sort_by(|(a, _), (b, _)| a.cmp(b));

    let frame = if fields.is_empty() {
        name.to_string()
    } else {
        let fields = fields
            .iter()
            .map(|(key, value)| match value.as_str() {
                Some(string) => format!("{key}={string}"),
                None => format!("{key}={value}"),
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("{name}{{{fields}}}")
    };
    frame.replace(';', ",").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Build an `enter` event line. `spans` is the full chain including the
    /// entered span itself, as tracing emits it.
    fn enter_line(ts: &str, spans: &[&Value]) -> String {
        let span = *spans.last().unwrap();
        json!({
            "timestamp": ts,
            "level": "TRACE",
            "fields": {"message": "enter"},
            "target": "test",
            "span": span,
            "spans": spans,
        })
        .to_string()
    }

    /// Build a `close` event line. `ancestors` excludes the closed span
    /// itself, as tracing emits it.
    fn close_line(ts: &str, busy: &str, idle: &str, ancestors: &[&Value], span: &Value) -> String {
        json!({
            "timestamp": ts,
            "level": "TRACE",
            "fields": {"message": "close", "time.busy": busy, "time.idle": idle},
            "target": "test",
            "span": span,
            "spans": ancestors,
        })
        .to_string()
    }

    fn ts(us: u64) -> String {
        // Everything within one second keeps the arithmetic obvious.
        format!("2026-07-07T12:00:00.{us:06}Z")
    }

    fn process(lines: &[String]) -> SpanLogReport {
        let mut processor = SpanLogProcessor::new(0);
        for line in lines {
            processor.ingest_line(line);
        }
        processor.finish()
    }

    #[test]
    fn parses_tracing_durations() {
        assert_eq!(parse_duration_us("740ns"), Some(0.74));
        assert_eq!(parse_duration_us("103µs"), Some(103.0));
        assert_eq!(parse_duration_us("5.11ms"), Some(5110.0));
        assert_eq!(parse_duration_us("36.0s"), Some(36_000_000.0));
        assert_eq!(parse_duration_us("bogus"), None);
        assert_eq!(parse_duration_us("12"), None);
    }

    #[test]
    fn parses_timestamps_to_microseconds() {
        let a = parse_timestamp_us("2026-07-07T12:29:08.257380Z").unwrap();
        let b = parse_timestamp_us("2026-07-07T12:29:09.257381Z").unwrap();
        assert_eq!(b - a, 1_000_001);
    }

    #[test]
    fn renders_span_fields_into_frame() {
        let span = json!({"name": "ssa_pass", "pass": "mem2reg (1)"});
        assert_eq!(render_frame(span.as_object().unwrap()), "ssa_pass{pass=mem2reg (1)}");

        let plain = json!({"name": "check_crate"});
        assert_eq!(render_frame(plain.as_object().unwrap()), "check_crate");

        // `;` would split the frame in the folded format.
        let sneaky = json!({"name": "s", "f": "a;b"});
        assert_eq!(render_frame(sneaky.as_object().unwrap()), "s{f=a,b}");
    }

    #[test]
    fn folds_parent_and_child_with_explicit_self_frame() {
        let root = json!({"name": "root"});
        let child = json!({"name": "child"});
        let lines = vec![
            enter_line(&ts(0), &[&root]),
            enter_line(&ts(10), &[&root, &child]),
            close_line(&ts(110), "100µs", "0ns", &[&root], &child),
            close_line(&ts(200), "200µs", "0ns", &[], &root),
        ];
        let report = process(&lines);
        assert_eq!(
            report.folded_lines,
            vec!["root;(self) 100".to_string(), "root;child 100".to_string()]
        );
        assert_eq!(report.wall_us, 200);
        assert_eq!(report.folded_total_us, 200);
        assert_eq!(report.closed_spans, 2);
        assert!(report.warnings.is_empty(), "unexpected warnings: {:?}", report.warnings);
    }

    #[test]
    fn recursion_keeps_each_depth_distinct() {
        let a = json!({"name": "a"});
        let b = json!({"name": "b"});
        let lines = vec![
            enter_line(&ts(0), &[&a]),
            enter_line(&ts(10), &[&a, &b]),
            enter_line(&ts(20), &[&a, &b, &b]),
            close_line(&ts(50), "30.0µs", "0ns", &[&a, &b], &b),
            close_line(&ts(70), "60.0µs", "0ns", &[&a], &b),
            close_line(&ts(100), "100µs", "0ns", &[], &a),
        ];
        let report = process(&lines);
        assert_eq!(
            report.folded_lines,
            vec!["a;(self) 40".to_string(), "a;b;(self) 30".to_string(), "a;b;b 30".to_string(),]
        );
    }

    #[test]
    fn sums_repeated_spans() {
        let root = json!({"name": "root"});
        let child = json!({"name": "child"});
        let lines = vec![
            enter_line(&ts(0), &[&root]),
            enter_line(&ts(10), &[&root, &child]),
            close_line(&ts(40), "30.0µs", "0ns", &[&root], &child),
            enter_line(&ts(50), &[&root, &child]),
            close_line(&ts(70), "20.0µs", "0ns", &[&root], &child),
            close_line(&ts(100), "100µs", "0ns", &[], &root),
        ];
        let report = process(&lines);
        assert_eq!(
            report.folded_lines,
            vec!["root;(self) 50".to_string(), "root;child 50".to_string()]
        );
        // Two child instances plus the root.
        assert_eq!(report.timeline.len(), 3);
    }

    #[test]
    fn reparents_cross_thread_root_by_time_containment() {
        // `inner` runs on another thread so tracing reports it as a root,
        // but its interval is contained in `outer`'s.
        let outer = json!({"name": "outer"});
        let inner = json!({"name": "inner"});
        let leaf = json!({"name": "leaf"});
        let lines = vec![
            enter_line(&ts(0), &[&outer]),
            enter_line(&ts(100), &[&inner]),
            enter_line(&ts(110), &[&inner, &leaf]),
            close_line(&ts(160), "50.0µs", "0ns", &[&inner], &leaf),
            close_line(&ts(300), "200µs", "0ns", &[], &inner),
            close_line(&ts(1000), "1.00ms", "0ns", &[], &outer),
        ];
        let report = process(&lines);
        assert_eq!(
            report.folded_lines,
            vec![
                "outer;(self) 800".to_string(),
                "outer;inner;(self) 150".to_string(),
                "outer;inner;leaf 50".to_string(),
            ]
        );
        assert_eq!(report.folded_total_us, 1000);
    }

    #[test]
    fn uncovered_wall_time_is_untracked() {
        let a = json!({"name": "a"});
        let b = json!({"name": "b"});
        let lines = vec![
            enter_line(&ts(0), &[&a]),
            close_line(&ts(50), "50.0µs", "0ns", &[], &a),
            enter_line(&ts(100), &[&b]),
            close_line(&ts(200), "100µs", "0ns", &[], &b),
        ];
        let report = process(&lines);
        assert_eq!(
            report.folded_lines,
            vec!["(untracked) 50".to_string(), "a 50".to_string(), "b 100".to_string(),]
        );
        assert_eq!(report.wall_us, 200);
        assert_eq!(report.folded_total_us, 200);
    }

    #[test]
    fn timeline_uses_first_enter_and_close_and_honors_min_duration() {
        let root = json!({"name": "root"});
        let child = json!({"name": "tiny"});
        let lines = vec![
            enter_line(&ts(0), &[&root]),
            enter_line(&ts(10), &[&root, &child]),
            close_line(&ts(15), "5.00µs", "0ns", &[&root], &child),
            close_line(&ts(500), "500µs", "0ns", &[], &root),
        ];
        let mut processor = SpanLogProcessor::new(10);
        for line in &lines {
            processor.ingest_line(line);
        }
        let report = processor.finish();
        // The 5µs child is below the 10µs timeline threshold.
        assert_eq!(report.timeline.len(), 1);
        assert_eq!(report.timeline[0].name, "root");
        assert_eq!(report.timeline[0].start_us, 0);
        assert_eq!(report.timeline[0].dur_us, 500);
    }

    #[test]
    fn non_span_lines_are_skipped_not_fatal() {
        let a = json!({"name": "a"});
        let lines = vec![
            "not json at all".to_string(),
            json!({"timestamp": ts(1), "fields": {"message": "some log"}}).to_string(),
            enter_line(&ts(0), &[&a]),
            close_line(&ts(50), "50.0µs", "0ns", &[], &a),
        ];
        let report = process(&lines);
        assert_eq!(report.skipped_lines, 2);
        assert_eq!(report.closed_spans, 1);
    }

    #[test]
    fn unclosed_span_is_reported() {
        let a = json!({"name": "a"});
        let lines = vec![enter_line(&ts(0), &[&a])];
        let report = process(&lines);
        assert!(
            report.warnings.iter().any(|w| w.contains("never closed")),
            "expected an unclosed-span warning, got: {:?}",
            report.warnings
        );
    }
}
