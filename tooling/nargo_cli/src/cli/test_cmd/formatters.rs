use std::{
    io::{self, Write},
    panic::RefUnwindSafe,
    time::Duration,
};

use fm::FileManager;
use nargo::ops::TestStatus;
use noirc_errors::{CustomDiagnostic, reporter::stack_trace};
use noirc_frontend::{
    error_reporting::{function_locations_for_diagnostics, report_one},
    hir::ParsedFiles,
};
use serde_json::json;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, StandardStreamLock, WriteColor};

use super::TestResult;

/// A formatter for showing test results.
///
/// The order of events is:
/// 1. Compilation of all packages happen (in parallel). There's no formatter method for this.
/// 2. If compilation is successful, one `package_start_async` for each package.
/// 3. For each test, one `test_start_async` event and one `test_end_async` event, as the test
///    runs (there's no `test_start_sync` event because it would happen right before `test_end_sync`)
/// 4. For each package, sequentially:
///     1. A `package_start_sync` event
///     2. One `test_end_sync` event for each test
///     3. A `package_end` event
///
/// The reason we have some `sync` and `async` events is that formatters that show output
/// to humans rely on the `sync` events to show a more predictable output (package by package),
/// and formatters that output to a machine-readable format (like JSON) rely on the `async`
/// events to show things as soon as they happen, regardless of a package ordering.
///
/// Every event does nothing by default, so a formatter only implements the events it shows.
pub(crate) trait Formatter: Send + Sync + RefUnwindSafe {
    fn package_start_async(&self, _package_name: &str, _test_count: usize) -> io::Result<()> {
        Ok(())
    }

    fn package_start_sync(&self, _package_name: &str, _test_count: usize) -> io::Result<()> {
        Ok(())
    }

    fn test_start_async(&self, _name: &str, _package_name: &str) -> io::Result<()> {
        Ok(())
    }

    fn test_end_async(&self, _test_result: &TestResult) -> io::Result<()> {
        Ok(())
    }

    fn test_end_sync(
        &self,
        _test_result: &TestResult,
        _current_test_count: usize,
        _total_test_count: usize,
    ) -> io::Result<()> {
        Ok(())
    }

    fn package_end(&self, _package_name: &str, _test_results: &[TestResult]) -> io::Result<()> {
        Ok(())
    }
}

/// What a formatter needs to show test results, besides the results themselves.
#[derive(Clone, Copy)]
pub(crate) struct DisplayOptions<'a> {
    /// The sources that diagnostics point into.
    pub(crate) file_manager: &'a FileManager,
    pub(crate) parsed_files: &'a ParsedFiles,
    /// Show what each test printed (`--show-output`).
    pub(crate) show_output: bool,
    pub(crate) deny_warnings: bool,
    pub(crate) silence_warnings: bool,
}

impl DisplayOptions<'_> {
    /// Prints `diagnostic` to stderr.
    fn report(&self, diagnostic: &CustomDiagnostic) {
        report_one(
            diagnostic,
            self.file_manager,
            self.parsed_files,
            self.deny_warnings,
            self.silence_warnings,
        );
    }

    /// `diagnostic` as plain text, or `None` if `--silence-warnings` hides it.
    fn render(&self, diagnostic: &CustomDiagnostic) -> Option<String> {
        if diagnostic.is_warning() && self.silence_warnings {
            None
        } else {
            Some(diagnostic_to_string(diagnostic, self.file_manager, self.parsed_files))
        }
    }

    /// What `test_result` printed, if it printed anything and `--show-output` asks for it.
    fn output_to_show<'r>(&self, test_result: &'r TestResult) -> Option<&'r str> {
        (self.show_output && !test_result.output.is_empty()).then_some(test_result.output.as_str())
    }
}

/// Prints one line per test, package by package.
pub(crate) struct PrettyFormatter<'a> {
    options: DisplayOptions<'a>,
}

impl<'a> PrettyFormatter<'a> {
    pub(crate) fn new(options: DisplayOptions<'a>) -> Self {
        Self { options }
    }
}

impl Formatter for PrettyFormatter<'_> {
    fn package_start_sync(&self, package_name: &str, test_count: usize) -> io::Result<()> {
        package_start(package_name, test_count)
    }

    fn test_end_sync(
        &self,
        test_result: &TestResult,
        _current_test_count: usize,
        _total_test_count: usize,
    ) -> io::Result<()> {
        let writer = stdout();
        let mut writer = writer.lock();

        let is_slow = test_result.time_to_run >= Duration::from_secs(30);
        let show_time = |writer: &mut StandardStreamLock<'_>| {
            if is_slow {
                write!(writer, " <{:.3}s>", test_result.time_to_run.as_secs_f64())
            } else {
                Ok(())
            }
        };

        write!(writer, "[{}] Testing {} ... ", test_result.package_name, test_result.name)?;
        writer.flush()?;

        match &test_result.status {
            TestStatus::Pass => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                write!(writer, "ok")?;
                writer.reset()?;
                show_time(&mut writer)?;
                writeln!(writer)?;
            }
            TestStatus::Fail { message, error_diagnostic } => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                write!(writer, "FAIL\n{message}\n")?;
                writer.reset()?;
                show_time(&mut writer)?;
                writeln!(writer)?;
                if let Some(diagnostic) = error_diagnostic {
                    self.options.report(diagnostic);
                }
            }
            TestStatus::Skipped => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                write!(writer, "skipped")?;
                writer.reset()?;
                show_time(&mut writer)?;
                writeln!(writer)?;
            }
            TestStatus::CompileError(diagnostic) => {
                self.options.report(diagnostic);
            }
        }

        if let Some(output) = self.options.output_to_show(test_result) {
            write_output_header(&mut writer, test_result)?;
            write!(writer, "{output}")?;
            write_output_footer(&mut writer, test_result)?;
        }
        Ok(())
    }

    fn package_end(&self, package_name: &str, test_results: &[TestResult]) -> io::Result<()> {
        let writer = stdout();
        let mut writer = writer.lock();
        write_package_summary(&mut writer, package_name, test_results)
    }
}

/// Prints one character per test, then the output of failed tests once the package is done.
pub(crate) struct TerseFormatter<'a> {
    options: DisplayOptions<'a>,
}

impl<'a> TerseFormatter<'a> {
    pub(crate) fn new(options: DisplayOptions<'a>) -> Self {
        Self { options }
    }
}

impl Formatter for TerseFormatter<'_> {
    fn package_start_sync(&self, package_name: &str, test_count: usize) -> io::Result<()> {
        package_start(package_name, test_count)
    }

    fn test_end_sync(
        &self,
        test_result: &TestResult,
        current_test_count: usize,
        total_test_count: usize,
    ) -> io::Result<()> {
        let writer = stdout();
        let mut writer = writer.lock();

        match &test_result.status {
            TestStatus::Pass => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                write!(writer, ".")?;
                writer.reset()?;
            }
            TestStatus::Fail { .. } | TestStatus::CompileError(_) => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                write!(writer, "F")?;
                writer.reset()?;
            }
            TestStatus::Skipped => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                write!(writer, "s")?;
                writer.reset()?;
            }
        }

        // How many tests ('.', 'F', etc.) to print per line.
        // We use 88 which is a bit more than the traditional 80 columns (screens are larger these days)
        // but we also want the output to be readable in case the terminal isn't maximized.
        const MAX_TESTS_PER_LINE: usize = 88;

        if current_test_count.is_multiple_of(MAX_TESTS_PER_LINE)
            && current_test_count < total_test_count
        {
            writeln!(writer, " {current_test_count}/{total_test_count}")?;
        }

        Ok(())
    }

    fn package_end(&self, package_name: &str, test_results: &[TestResult]) -> io::Result<()> {
        let writer = stdout();
        let mut writer = writer.lock();

        if !test_results.is_empty() {
            writeln!(writer)?;
        }

        for test_result in test_results {
            let failed = test_result.status.failed();
            if !failed && self.options.output_to_show(test_result).is_none() {
                continue;
            }

            write_output_header(&mut writer, test_result)?;
            if !test_result.output.is_empty() {
                write!(writer, "{}", test_result.output)?;
            }

            match &test_result.status {
                TestStatus::Pass | TestStatus::Skipped => (),
                TestStatus::Fail { message, error_diagnostic } => {
                    writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                    writeln!(writer, "{message}")?;
                    writer.reset()?;
                    if let Some(diagnostic) = error_diagnostic {
                        self.options.report(diagnostic);
                    }
                }
                TestStatus::CompileError(diagnostic) => {
                    self.options.report(diagnostic);
                }
            }

            write_output_footer(&mut writer, test_result)?;
        }

        write_package_summary(&mut writer, package_name, test_results)
    }
}

/// Prints one JSON object per line for each event, as soon as it happens.
pub(crate) struct JsonFormatter<'a> {
    options: DisplayOptions<'a>,
}

impl<'a> JsonFormatter<'a> {
    pub(crate) fn new(options: DisplayOptions<'a>) -> Self {
        Self { options }
    }
}

impl Formatter for JsonFormatter<'_> {
    fn package_start_async(&self, package_name: &str, test_count: usize) -> io::Result<()> {
        let json = json!({"type": "suite", "event": "started", "name": package_name, "test_count": test_count});
        writeln!(io::stdout(), "{json}")
    }

    fn test_start_async(&self, name: &str, package_name: &str) -> io::Result<()> {
        let json = json!({"type": "test", "event": "started", "name": name, "suite": package_name});
        writeln!(io::stdout(), "{json}")
    }

    fn test_end_async(&self, test_result: &TestResult) -> io::Result<()> {
        let mut stdout = String::new();
        if let Some(output) = self.options.output_to_show(test_result) {
            stdout.push_str(output.trim());
        }

        let event = match &test_result.status {
            TestStatus::Pass => "ok",
            TestStatus::Fail { message, error_diagnostic } => {
                if !stdout.is_empty() {
                    stdout.push('\n');
                }
                stdout.push_str(message.trim());

                if let Some(diagnostic) =
                    error_diagnostic.as_ref().and_then(|diagnostic| self.options.render(diagnostic))
                {
                    stdout.push('\n');
                    stdout.push_str(&diagnostic);
                }
                "failed"
            }
            TestStatus::Skipped => "ignored",
            TestStatus::CompileError(diagnostic) => {
                if let Some(diagnostic) = self.options.render(diagnostic) {
                    if !stdout.is_empty() {
                        stdout.push('\n');
                    }
                    stdout.push_str(&diagnostic);
                }
                "failed"
            }
        };

        let mut json = json!({
            "type": "test",
            "event": event,
            "name": &test_result.name,
            "suite": &test_result.package_name,
            "exec_time": test_result.time_to_run.as_secs_f64(),
        });
        if !stdout.is_empty() {
            json["stdout"] = json!(stdout);
        }
        writeln!(io::stdout(), "{json}")
    }

    fn package_end(&self, _package_name: &str, test_results: &[TestResult]) -> io::Result<()> {
        let mut passed = 0;
        let mut failed = 0;
        let mut ignored = 0;
        for test_result in test_results {
            match &test_result.status {
                TestStatus::Pass => passed += 1,
                TestStatus::Fail { .. } | TestStatus::CompileError(..) => failed += 1,
                TestStatus::Skipped => ignored += 1,
            }
        }
        let event = if failed == 0 { "ok" } else { "failed" };
        let json = json!({"type": "suite", "event": event, "passed": passed, "failed": failed, "ignored": ignored});
        writeln!(io::stdout(), "{json}")
    }
}

fn package_start(package_name: &str, test_count: usize) -> io::Result<()> {
    let plural = if test_count == 1 { "" } else { "s" };
    writeln!(io::stdout(), "[{package_name}] Running {test_count} test function{plural}")
}

fn write_output_header(
    writer: &mut StandardStreamLock<'_>,
    test_result: &TestResult,
) -> io::Result<()> {
    writeln!(writer, "--- {} stdout ---", test_result.name)
}

/// A line of dashes as long as the header [`write_output_header`] writes.
fn write_output_footer(
    writer: &mut StandardStreamLock<'_>,
    test_result: &TestResult,
) -> io::Result<()> {
    let name_len = test_result.name.len();
    writeln!(writer, "{}", "-".repeat(name_len + "---  stdout ---".len()))
}

/// Lists the package's failed tests, then how many tests passed and failed.
fn write_package_summary(
    writer: &mut StandardStreamLock<'_>,
    package_name: &str,
    test_results: &[TestResult],
) -> io::Result<()> {
    let failed_tests: Vec<_> = test_results
        .iter()
        .filter_map(|test_result| test_result.status.failed().then_some(&test_result.name))
        .collect();

    if !failed_tests.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "[{package_name}] Failures:")?;
        for failed_test in &failed_tests {
            writeln!(writer, "     {failed_test}")?;
        }
        writeln!(writer)?;
    }

    write!(writer, "[{package_name}] ")?;

    let count_all = test_results.len();
    let count_failed = failed_tests.len();
    let plural = if count_all == 1 { "" } else { "s" };
    if count_failed == 0 {
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(writer, "{count_all} test{plural} passed")?;
        writer.reset()?;
        writeln!(writer)?;
    } else {
        let count_passed = count_all - count_failed;
        let plural_failed = if count_failed == 1 { "" } else { "s" };
        let plural_passed = if count_passed == 1 { "" } else { "s" };

        if count_passed != 0 {
            writer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(writer, "{count_passed} test{plural_passed} passed, ")?;
        }

        writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        writeln!(writer, "{count_failed} test{plural_failed} failed")?;
        writer.reset()?;
    }

    Ok(())
}

pub(crate) fn diagnostic_to_string(
    custom_diagnostic: &CustomDiagnostic,
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
) -> String {
    let file_map = file_manager.as_file_map();

    let mut message = String::new();
    message.push_str(custom_diagnostic.message.trim());

    for note in &custom_diagnostic.notes {
        message.push('\n');
        message.push_str(note.trim());
    }

    if let Ok(name) = file_map.get_name(custom_diagnostic.file) {
        message.push('\n');
        message.push_str(&format!("at {name}"));
    }

    if !custom_diagnostic.call_stack.is_empty() {
        let diagnostics = std::slice::from_ref(custom_diagnostic);
        let function_locations = function_locations_for_diagnostics(diagnostics, parsed_files);
        message.push('\n');
        message.push_str(&stack_trace(
            file_map,
            &function_locations,
            &custom_diagnostic.call_stack,
        ));
    }

    message
}

fn stdout() -> StandardStream {
    StandardStream::stdout(ColorChoice::Always)
}
