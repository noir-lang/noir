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

/// Returns the location of every expression in the crate that is not inside a `#[test]` function
/// and not inside a global initializer.
///
/// Global initializer expressions are excluded because the interpreter returns cached values for
/// resolved globals without re-evaluating their initializers, so they are never tracked and would
/// always appear as uncovered lines.
fn baseline_expr_locations(context: &Context, crate_id: CrateId) -> Vec<Location> {
    let def_map = &context.def_maps[&crate_id];
    let allowed_files = def_map.file_ids();

    let test_body_spans: Vec<Location> = def_map
        .get_all_test_functions(&context.def_interner)
        .filter_map(|test_fn| context.def_interner.function(&test_fn.id).try_as_expr())
        .map(|body_id| context.def_interner.expr_location(&body_id))
        .collect();

    let global_init_spans: Vec<Location> = context
        .def_interner
        .get_all_globals()
        .iter()
        .filter(|g| g.crate_id == crate_id)
        .filter_map(|g| context.def_interner.get_global_let_statement(g.id))
        .map(|let_stmt| context.def_interner.expr_location(&let_stmt.expression))
        .collect();

    context
        .def_interner
        .expr_locations_for_files(&allowed_files)
        .filter(|loc| {
            !test_body_spans
                .iter()
                .any(|test_loc| test_loc.file == loc.file && test_loc.span.contains(&loc.span))
                && !global_init_spans
                    .iter()
                    .any(|g_loc| g_loc.file == loc.file && g_loc.span.contains(&loc.span))
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
                let file = context.def_interner.function_meta(&func_id).location.file;
                if offsets_by_file.contains_key(&file) {
                    functions_by_file.entry(file).or_default().push(func_id);
                }
            }
        }
    }

    let mut report = Report::new();

    for (file_id, byte_offsets) in &offsets_by_file {
        let Some(path) = context.file_manager.path(*file_id) else { continue };
        let Some(source) = context.file_manager.fetch_file(*file_id) else { continue };

        let line_starts = build_line_starts(source);

        let mut functions = function::Functions::new();
        for &func_id in functions_by_file.get(file_id).map_or([].as_slice(), Vec::as_slice) {
            let meta = context.def_interner.function_meta(&func_id);
            let name = context.def_interner.function_name(&func_id).to_string();
            let start_line = offset_to_line(meta.location.span.start(), &line_starts);
            functions.insert(
                function::Key { name },
                function::Value { start_line: Some(start_line), count: 0 },
            );
        }

        let mut lines = line::Lines::new();
        for &offset in byte_offsets {
            let line_num = offset_to_line(offset, &line_starts);
            lines
                .entry(line::Key { line: line_num })
                .or_insert(line::Value { count: 0, checksum: None });
        }

        let key = section::Key { test_name: String::new(), source_file: path.to_path_buf() };
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

    for (&file_id, offsets_to_counts) in tracker.hits() {
        let Some(source) = context.file_manager.fetch_file(file_id) else { continue };
        let line_starts = build_line_starts(source);
        let (_, lines) = data.entry(file_id).or_default();

        for (&offset, &count) in offsets_to_counts {
            if let Some(ref body_loc) = test_body_loc
                && body_loc.file == file_id
                && offset >= body_loc.span.start()
                && offset <= body_loc.span.end()
            {
                continue;
            }
            let line_num = offset_to_line(offset, &line_starts);
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
        let Some((functions, _)) = data.get_mut(&file_id) else { continue };
        let Some(source) = context.file_manager.fetch_file(file_id) else { continue };
        let line_starts = build_line_starts(source);
        let start_line = offset_to_line(meta.location.span.start(), &line_starts);
        let name = context.def_interner.function_name(&func_id).to_string();

        functions
            .entry(function::Key { name })
            .and_modify(|v| v.count += count)
            .or_insert(function::Value { start_line: Some(start_line), count });
    }

    let mut report = Report::new();

    for (file_id, (functions, lines)) in data {
        let Some(path) = context.file_manager.path(file_id) else { continue };

        let key =
            section::Key { test_name: test_name.to_string(), source_file: path.to_path_buf() };
        let value = section::Value { functions, branches: Branches::default(), lines };
        if !value.is_empty() {
            report.sections.insert(key, value);
        }
    }

    report
}

/// Returns the path where coverage data for `package_name` should be written.
///
/// When the package sits at the workspace root (single-package layout) the file
/// is `<target>/coverage/lcov.info`, which Coverage Gutters picks up without any extra
/// configuration. In a multi-package workspace each package gets its own
/// subdirectory: `<target>/coverage/<package-name>/lcov.info`.
pub(super) fn package_lcov_path(workspace: &Workspace, package_name: &str) -> PathBuf {
    let mut target_dir = workspace.target_directory_path().join("coverage");
    let is_root_package = workspace
        .members
        .iter()
        .find(|p| p.name.to_string() == package_name)
        .is_none_or(|p| p.root_dir == workspace.root_dir);

    if !is_root_package {
        target_dir = target_dir.join(package_name)
    }

    target_dir.join("lcov.info")
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
        eprintln!("Warning: could not write coverage report to {}: {err}", path.display());
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
