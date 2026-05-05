//! Utilities to support test coverage.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use fm::FileId;
use lcov::Report;
use lcov::report::section;
use lcov::report::section::branch::Branches;
use lcov::report::section::function;
use lcov::report::section::line;
use nargo::workspace::Workspace;
use noirc_errors::Location;
use noirc_frontend::graph::CrateId;
use noirc_frontend::hir::Context;
use noirc_frontend::hir::comptime::EvaluationTracker;
use noirc_frontend::node_interner::FuncId;

/// Returns the location of every expression in the crate that should appear in the coverage
/// baseline, excluding expressions inside `#[test]` function bodies.
fn baseline_expr_locations(context: &Context, crate_id: CrateId) -> Vec<Location> {
    let def_map = &context.def_maps[&crate_id];
    let allowed_files = def_map.file_ids();

    let test_body_spans: Vec<Location> = def_map
        .get_all_test_functions(&context.def_interner)
        .filter_map(|test_fn| context.def_interner.function(&test_fn.id).try_as_expr())
        .map(|body_id| context.def_interner.expr_location(&body_id))
        .collect();

    context
        .def_interner
        .expr_locations_for_files(&allowed_files)
        .filter(|loc| {
            !test_body_spans
                .iter()
                .any(|test_loc| test_loc.file == loc.file && test_loc.span.contains(&loc.span))
        })
        .collect()
}

/// Builds the zero-count baseline `Report` for a compiled package.
///
/// Uses an empty test name so its sections (`("", file)`) are stored under a
/// distinct key from per-test sections (`("test_name", file)`) and are never
/// merged with them by `Report::merge`. `into_records` then emits them as a
/// separate `TN:` block, giving tools a complete picture of every instrumented
/// line even if no test ever reached it.
pub(super) fn baseline_in_package(context: &Context, crate_id: CrateId) -> Report {
    let def_map = &context.def_maps[&crate_id];

    let mut offsets_by_file: HashMap<FileId, Vec<u32>> = HashMap::new();
    for loc in baseline_expr_locations(context, crate_id) {
        offsets_by_file.entry(loc.file).or_default().push(loc.span.start());
    }

    let test_func_ids: HashSet<FuncId> =
        def_map.get_all_test_functions(&context.def_interner).map(|f| f.id).collect();

    let mut functions_by_file: HashMap<FileId, Vec<FuncId>> = HashMap::new();
    for (_, module) in def_map.modules().iter() {
        for def_id in module.value_definitions() {
            if let Some(func_id) = def_id.as_function()
                && !test_func_ids.contains(&func_id)
            {
                let func_meta = context.def_interner.function_meta(&func_id);

                // Don't track trait functions or other functions without bodies.
                if func_meta.is_stub() {
                    continue;
                }

                let file = func_meta.location.file;

                // Remember this function, so we can emit their name and where they are later.
                functions_by_file.entry(file).or_default().push(func_id);

                // Make sure we have an entry in offsets as well, so we can iterate over the files
                // even if all functions had empty bodies.
                offsets_by_file.entry(file).or_default();
            }
        }
    }

    let mut report = Report::new();
    let line_starts = LineStartsCache::new(context);

    for (file_id, byte_offsets) in &offsets_by_file {
        let Some(source_file) = file_path(context, *file_id) else {
            continue;
        };
        let Some(line_starts) = line_starts.build(file_id) else { continue };

        let mut functions = function::Functions::new();
        let mut lines = line::Lines::new();

        for &func_id in functions_by_file.get(file_id).map_or([].as_slice(), Vec::as_slice) {
            let meta = context.def_interner.function_meta(&func_id);
            let name = context.fully_qualified_function_name(&meta.source_crate, &func_id);
            let start_line = offset_to_line(meta.location.span.start(), &line_starts);
            functions.insert(
                function::Key { name },
                function::Value { start_line: Some(start_line), count: 0 },
            );
            // Insert a line for the start to highlight the function as callable.
            lines.insert(line::Key { line: start_line }, line::Value { count: 0, checksum: None });
        }

        for &offset in byte_offsets {
            let line_num = offset_to_line(offset, &line_starts);
            lines
                .entry(line::Key { line: line_num })
                .or_insert(line::Value { count: 0, checksum: None });
        }

        let key = section::Key { test_name: String::new(), source_file };
        let value = section::Value { functions, branches: Branches::default(), lines };
        report.sections.insert(key, value);
    }

    report
}

/// Converts an `EvaluationTracker` collected during one test run into a `Report`
/// section keyed by `test_name`.
///
/// Each `(test_name, source_file)` section is independent and will not be merged
/// with the baseline or other tests when the reports are combined.
///
/// Expressions inside the test function's own body are excluded: they are the
/// test harness, not the code under test.
pub(super) fn tracker_to_report(
    tracker: &EvaluationTracker,
    test_func_id: FuncId,
    test_name: &str,
    context: &Context,
) -> Report {
    // Body span of the test function itself — expressions within it are excluded.
    let test_body_loc = context
        .def_interner
        .function(&test_func_id)
        .try_as_expr()
        .map(|body_id| context.def_interner.expr_location(&body_id));

    // Accumulate (functions, lines) per FileId before building sections.
    let mut data: HashMap<FileId, (function::Functions, line::Lines)> = HashMap::new();
    let mut line_starts = LineStartsCache::new(context);

    for (&file_id, offsets_to_counts) in tracker.hits() {
        let Some(line_starts) = line_starts.get(&file_id) else {
            continue;
        };
        let (_, lines) = data.entry(file_id).or_default();

        for (&offset, &count) in offsets_to_counts {
            if let Some(ref body_loc) = test_body_loc
                && body_loc.file == file_id
                && offset >= body_loc.span.start()
                && offset <= body_loc.span.end()
            {
                continue;
            }
            let line_num = offset_to_line(offset, line_starts);
            lines
                .entry(line::Key { line: line_num })
                .and_modify(|v| v.count += count)
                .or_insert(line::Value { count, checksum: None });
        }
    }

    for (&func_id, &count) in tracker.function_hits() {
        if func_id == test_func_id {
            continue;
        }
        let meta = context.def_interner.function_meta(&func_id);
        let file_id = meta.location.file;
        let Some((functions, lines)) = data.get_mut(&file_id) else { continue };
        let Some(line_starts) = line_starts.get(&file_id) else {
            continue;
        };
        let start_line = offset_to_line(meta.location.span.start(), line_starts);
        let name = context.fully_qualified_function_name(&meta.source_crate, &func_id);

        functions
            .entry(function::Key { name })
            .and_modify(|v| v.count += count)
            .or_insert(function::Value { start_line: Some(start_line), count });

        // Emit a line as well to visually highlight the function as called.
        lines
            .entry(line::Key { line: start_line })
            .and_modify(|v| v.count += count)
            .or_insert(line::Value { count, checksum: None });
    }

    let mut report = Report::new();

    for (file_id, (functions, lines)) in data {
        let Some(source_file) = file_path(context, file_id) else {
            continue;
        };

        let key = section::Key { test_name: test_name.to_string(), source_file };
        let value = section::Value { functions, branches: Branches::default(), lines };
        if !value.is_empty() {
            report.sections.insert(key, value);
        }
    }

    report
}

fn file_path(context: &Context, file_id: FileId) -> Option<PathBuf> {
    context.file_manager.as_file_map().get_name(file_id).ok().map(|p| p.into_path_buf())
}

/// Returns the path where coverage data for `package_name` should be written.
///
/// `coverage_dir_override` takes precedence when provided. Otherwise the base directory is
/// derived from the workspace target directory.
///
/// When the package sits at the workspace root (single-package layout) the file is
/// `<base>/lcov.info`. In a multi-package workspace each package is nested:
/// `<base>/<package-name>/lcov.info`.
pub(super) fn package_lcov_path(
    workspace: &Workspace,
    package_name: &str,
    coverage_dir_override: Option<&std::path::Path>,
) -> PathBuf {
    // Most plugins that display coverage in the editor look for `lcov.info` files.
    // We could use `*.lcov`, but it might need extra configuration; an extra hurdle.
    const LCOV_FILE_NAME: &str = "lcov.info";

    let base_dir = coverage_dir_override
        .map_or_else(|| workspace.target_directory_path().join("coverage"), |p| p.to_path_buf());

    let is_root_package = workspace
        .members
        .iter()
        .find(|p| p.name.to_string() == package_name)
        .is_none_or(|p| p.root_dir == workspace.root_dir);

    if is_root_package {
        base_dir.join(LCOV_FILE_NAME)
    } else {
        base_dir.join(package_name).join(LCOV_FILE_NAME)
    }
}

/// Writes an lcov report to `path`, creating parent directories if necessary.
/// Prints a warning to stderr on failure.
pub(super) fn write_package_coverage(report: Report, path: &std::path::Path) {
    use std::io::Write;

    let write = || -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(path)?;
        for record in report.into_records() {
            writeln!(file, "{record}")?;
        }
        Ok(())
    };

    if let Err(err) = write() {
        noirc_errors::println_to_stderr!(
            "Warning: could not write coverage report to {}: {err}",
            path.display()
        );
    } else {
        noirc_errors::println_to_stdout!("Coverage report written to {}", path.display());
    }
}

/// Returns a vec of byte offsets where each line starts (0-indexed by line, 1-indexed by value).
/// `line_starts[0]` is always 0 (start of line 1).
fn build_line_starts(source: &str) -> Vec<u32> {
    let mut starts = vec![0u32];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i as u32 + 1);
        }
    }
    starts
}

/// Converts a byte offset within a file to a 1-indexed line number.
fn offset_to_line(offset: u32, line_starts: &[u32]) -> u32 {
    line_starts.partition_point(|&start| start <= offset) as u32
}

struct LineStartsCache<'a> {
    context: &'a Context<'a, 'a>,
    line_starts: HashMap<FileId, Vec<u32>>,
}

impl<'a> LineStartsCache<'a> {
    fn new(context: &'a Context<'a, 'a>) -> Self {
        Self { context, line_starts: HashMap::new() }
    }

    /// Get from the cache or build.
    fn get(&mut self, file_id: &FileId) -> Option<&[u32]> {
        if !self.line_starts.contains_key(file_id) {
            let line_starts = self.build(file_id)?;
            self.line_starts.insert(*file_id, line_starts);
        }
        self.line_starts.get(file_id).map(|v| v.as_slice())
    }

    /// Build without caching.
    fn build(&self, file_id: &FileId) -> Option<Vec<u32>> {
        let source = self.context.file_manager.fetch_file(*file_id)?;
        let line_starts = build_line_starts(source);
        Some(line_starts)
    }
}
