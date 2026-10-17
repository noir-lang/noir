use std::{
    io::{self, Write},
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

/// Where `nargo test` sends its output. A formatter consumes exactly one of two event streams.
pub(crate) enum Output<'a> {
    /// Events as they happen, for machines.
    Live(Box<dyn LiveFormatter + 'a>),
    /// Events one package at a time, for humans.
    Ordered(Box<dyn OrderedFormatter + 'a>),
}

impl Output<'_> {
    /// The live formatter, if this output is live.
    pub(crate) fn live(&self) -> Option<&dyn LiveFormatter> {
        match self {
            Output::Live(formatter) => Some(formatter.as_ref()),
            Output::Ordered(_) => None,
        }
    }
}

/// Shows events the moment they happen, so a reader of the stream gets each result as soon as
/// it exists. Packages interleave, and tests appear in the order they finish.
///
/// For each run:
/// 1. `package_start` for every package, in package order, before any test runs.
/// 2. For each test, on the worker thread that runs it: `test_start`, then `test_end`.
/// 3. `package_end` for each package once its last result has arrived.
pub(crate) trait LiveFormatter: Send + Sync {
    fn package_start(&self, package_name: &str, test_count: usize) -> io::Result<()>;

    fn test_start(&self, name: &str, package_name: &str) -> io::Result<()>;

    fn test_end(&self, test_result: &TestResult) -> io::Result<()>;

    fn package_end(&self, package_name: &str, test_results: &[TestResult]) -> io::Result<()>;
}

/// Shows results one package at a time, in package order, so the output has the same layout
/// however the tests were scheduled. Every event is called from the main thread.
///
/// For each package, in package order:
/// 1. `package_start`
/// 2. `test_end` for each of its tests, numbered `current` of `total`
/// 3. `package_end`
pub(crate) trait OrderedFormatter: Send + Sync {
    fn package_start(&self, package_name: &str, test_count: usize) -> io::Result<()>;

    fn test_end(
        &self,
        test_result: &TestResult,
        current_test_count: usize,
        total_test_count: usize,
    ) -> io::Result<()>;

    fn package_end(&self, package_name: &str, test_results: &[TestResult]) -> io::Result<()>;
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

impl OrderedFormatter for PrettyFormatter<'_> {
    fn package_start(&self, package_name: &str, test_count: usize) -> io::Result<()> {
        package_start(package_name, test_count)
    }

    fn test_end(
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

impl OrderedFormatter for TerseFormatter<'_> {
    fn package_start(&self, package_name: &str, test_count: usize) -> io::Result<()> {
        package_start(package_name, test_count)
    }

    fn test_end(
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

impl LiveFormatter for JsonFormatter<'_> {
    fn package_start(&self, package_name: &str, test_count: usize) -> io::Result<()> {
        let json = json!({"type": "suite", "event": "started", "name": package_name, "test_count": test_count});
        writeln!(io::stdout(), "{json}")
    }

    fn test_start(&self, name: &str, package_name: &str) -> io::Result<()> {
        let json = json!({"type": "test", "event": "started", "name": name, "suite": package_name});
        writeln!(io::stdout(), "{json}")
    }

    fn test_end(&self, test_result: &TestResult) -> io::Result<()> {
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

    fn package_end(&self, package_name: &str, test_results: &[TestResult]) -> io::Result<()> {
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
        let json = json!({"type": "suite", "event": event, "name": package_name, "passed": passed, "failed": failed, "ignored": ignored});
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
