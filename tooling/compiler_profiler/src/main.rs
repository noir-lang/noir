#![forbid(unsafe_code)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! Turns the JSON span logs written by a `NARGO_LOG_DIR=<dir> NOIR_LOG=trace
//! nargo` invocation into a compiler-phase flamegraph and a chrome-trace
//! timeline. See README.md for usage and how to read the output.

mod span_log;

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::{self, Context, eyre};
use inferno::flamegraph::{Options, from_lines};
use serde_json::json;

use crate::span_log::{SpanLogProcessor, SpanLogReport};

/// Generates a compiler-phase flamegraph and a chrome-trace timeline from the
/// JSON span logs written by a `NARGO_LOG_DIR=<dir> NOIR_LOG=trace nargo`
/// invocation.
#[derive(Debug, Parser)]
#[command(name = "noir-compiler-profiler", author, version, about)]
struct CompilerProfilerCli {
    /// The directory containing the JSON span logs (the `NARGO_LOG_DIR` of
    /// the profiled run). Every file in it is treated as part of one run, so
    /// use a fresh directory per run.
    #[clap(long, short)]
    log_dir: PathBuf,

    /// The output folder for the flamegraph SVG and chrome-trace JSON
    #[clap(long, short)]
    output: PathBuf,

    /// Spans shorter than this many microseconds are left out of the
    /// chrome-trace timeline (the flamegraph always includes everything)
    #[clap(long, default_value = "100")]
    timeline_min_us: u64,

    /// Title for the flamegraph SVG
    #[clap(long, default_value = "nargo compiler phases")]
    title: String,
}

fn main() {
    color_eyre::install().expect("color_eyre installs only once");
    if let Err(report) = run(CompilerProfilerCli::parse()) {
        eprintln!("{report:#}");
        std::process::exit(1);
    }
}

fn run(args: CompilerProfilerCli) -> eyre::Result<()> {
    let mut log_files: Vec<PathBuf> = std::fs::read_dir(&args.log_dir)
        .with_context(|| format!("Error reading log directory {}", args.log_dir.display()))?
        .map(|entry| Ok(entry?.path()))
        .filter(|path: &eyre::Result<PathBuf>| path.as_ref().is_ok_and(|path| path.is_file()))
        .collect::<eyre::Result<_>>()?;
    if log_files.is_empty() {
        return Err(eyre!(
            "No log files found in {}. Run nargo with NARGO_LOG_DIR={} NOIR_LOG=trace first.",
            args.log_dir.display(),
            args.log_dir.display()
        ));
    }
    // Daily rotation names files by date, so lexicographic order is
    // chronological.
    log_files.sort();

    let mut processor = SpanLogProcessor::new(args.timeline_min_us);
    for path in &log_files {
        let file = std::fs::File::open(path)
            .with_context(|| format!("Error opening log file {}", path.display()))?;
        for line in BufReader::new(file).lines() {
            processor.ingest_line(&line?);
        }
    }
    let report = processor.finish();

    if report.closed_spans == 0 {
        return Err(eyre!(
            "No span close events found in {}. Was nargo run with NOIR_LOG=trace?",
            args.log_dir.display()
        ));
    }

    std::fs::create_dir_all(&args.output)?;
    let svg_path = args.output.join("compiler-flamegraph.svg");
    let trace_path = args.output.join("compiler-trace.json");
    write_flamegraph_svg(&report, &args.title, &svg_path)?;
    write_chrome_trace(&report, &trace_path)?;

    print_report(&report);
    println!("Flamegraph:   {}", svg_path.display());
    println!("Chrome trace: {} (open in https://ui.perfetto.dev)", trace_path.display());

    Ok(())
}

fn write_flamegraph_svg(
    report: &SpanLogReport,
    title: &str,
    svg_path: &std::path::Path,
) -> eyre::Result<()> {
    let mut options = Options::default();
    options.hash = true;
    options.deterministic = true;
    options.title = title.to_string();
    options.subtitle = Some("width = self busy time; '(self)' frames are un-instrumented time within a span; '(untracked)' is wall time outside any span".to_string());
    options.frame_height = 24;
    options.color_diffusion = true;
    // The elaborator's per-expression spans produce hundreds of thousands of
    // distinct stacks; pruning sub-pixel frames keeps the SVG loadable.
    options.min_width = 0.1;
    options.count_name = "µs".to_string();

    let svg_file = std::fs::File::create(svg_path)
        .with_context(|| format!("Error creating SVG file {}", svg_path.display()))?;
    from_lines(
        &mut options,
        report.folded_lines.iter().map(String::as_str),
        BufWriter::new(svg_file),
    )?;
    Ok(())
}

/// Writes the timeline in the chrome trace event format, which Perfetto and
/// `chrome://tracing` can open. Events nest visually by time containment, so
/// everything can live on a single track.
fn write_chrome_trace(report: &SpanLogReport, trace_path: &std::path::Path) -> eyre::Result<()> {
    let events: Vec<serde_json::Value> = report
        .timeline
        .iter()
        .map(|event| {
            json!({
                "name": event.name,
                "ph": "X",
                "ts": event.start_us,
                "dur": event.dur_us,
                "pid": 0,
                "tid": 0,
            })
        })
        .collect();
    let trace = json!({ "traceEvents": events, "displayTimeUnit": "ms" });

    let trace_file = std::fs::File::create(trace_path)
        .with_context(|| format!("Error creating trace file {}", trace_path.display()))?;
    let mut writer = BufWriter::new(trace_file);
    serde_json::to_writer(&mut writer, &trace)?;
    writer.flush()?;
    Ok(())
}

fn print_report(report: &SpanLogReport) {
    let wall_ms = report.wall_us as f64 / 1000.0;
    let folded_ms = report.folded_total_us as f64 / 1000.0;
    println!(
        "Processed {} spans covering {wall_ms:.1}ms wall clock; flamegraph accounts for {folded_ms:.1}ms ({:.1}%)",
        report.closed_spans,
        100.0 * folded_ms / wall_ms.max(f64::EPSILON),
    );
    println!("Timeline events (>= threshold): {}", report.timeline.len());
    if report.skipped_lines > 0 {
        println!("Skipped {} non-span log lines", report.skipped_lines);
    }
    for warning in &report.warnings {
        println!("warning: {warning}");
    }
}
