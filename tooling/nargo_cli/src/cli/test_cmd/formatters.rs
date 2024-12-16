use std::{io::Write, panic::RefUnwindSafe, time::Duration};

use fm::FileManager;
use nargo::ops::TestStatus;
use noirc_errors::{reporter::stack_trace, FileDiagnostic};
use serde_json::{json, Map};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, StandardStreamLock, WriteColor};

use super::TestResult;

/// A formatter for showing test results.
///
/// The order of events is:
/// 1. Compilation of all packages happen (in parallel). There's no formatter method for this.
/// 2. If compilation is successful, one `package_start_async` for each package.
/// 3. For each test, one `test_start_async` event
///    (there's no `test_start_sync` event because it would happen right before `test_end_sync`)
/// 4. For each package, sequentially:
///     a. A `package_start_sync` event
///     b. One `test_end` event for each test
///     a. A `package_end` event
///
/// The reason we have some `sync` and `async` events is that formatters that show output
/// to humans rely on the `sync` events to show a more predictable output (package by package),
/// and formatters that output to a machine-readable format (like JSON) rely on the `async`
/// events to show things as soon as they happen, regardless of a package ordering.
pub(super) trait Formatter: Send + Sync + RefUnwindSafe {
    fn package_start_async(&self, package_name: &str, test_count: usize) -> std::io::Result<()>;

    fn package_start_sync(&self, package_name: &str, test_count: usize) -> std::io::Result<()>;

    fn test_start_async(&self, name: &str, package_name: &str) -> std::io::Result<()>;

    #[allow(clippy::too_many_arguments)]
    fn test_end_async(
        &self,
        test_result: &TestResult,
        file_manager: &FileManager,
        show_output: bool,
        deny_warnings: bool,
        silence_warnings: bool,
    ) -> std::io::Result<()>;

    #[allow(clippy::too_many_arguments)]
    fn test_end_sync(
        &self,
        test_result: &TestResult,
        current_test_count: usize,
        total_test_count: usize,
        file_manager: &FileManager,
        show_output: bool,
        deny_warnings: bool,
        silence_warnings: bool,
    ) -> std::io::Result<()>;

    fn package_end(
        &self,
        package_name: &str,
        test_results: &[TestResult],
        file_manager: &FileManager,
        show_output: bool,
        deny_warnings: bool,
        silence_warnings: bool,
    ) -> std::io::Result<()>;
}

pub(super) struct PrettyFormatter;

impl Formatter for PrettyFormatter {
    fn package_start_async(&self, _package_name: &str, _test_count: usize) -> std::io::Result<()> {
        Ok(())
    }

    fn package_start_sync(&self, package_name: &str, test_count: usize) -> std::io::Result<()> {
        package_start(package_name, test_count)
    }

    fn test_start_async(&self, _name: &str, _package_name: &str) -> std::io::Result<()> {
        Ok(())
    }

    fn test_end_async(
        &self,
        _test_result: &TestResult,
        _file_manager: &FileManager,
        _show_output: bool,
        _deny_warnings: bool,
        _silence_warnings: bool,
    ) -> std::io::Result<()> {
        Ok(())
    }

    fn test_end_sync(
        &self,
        test_result: &TestResult,
        _current_test_count: usize,
        _total_test_count: usize,
        file_manager: &FileManager,
        show_output: bool,
        deny_warnings: bool,
        silence_warnings: bool,
    ) -> std::io::Result<()> {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let mut writer = writer.lock();

        let is_slow = test_result.time_to_run >= Duration::from_secs(30);
        let show_time = |writer: &mut StandardStreamLock<'_>| {
            if is_slow {
                write!(writer, " <{:.3}s>", test_result.time_to_run.as_secs_f64())
            } else {
                Ok(())
            }
        };

        write!(writer, "[{}] Testing {}... ", &test_result.package_name, &test_result.name)?;
        writer.flush()?;

        match &test_result.status {
            TestStatus::Pass { .. } => {
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
                if let Some(diag) = error_diagnostic {
                    noirc_errors::reporter::report_all(
                        file_manager.as_file_map(),
                        &[diag.clone()],
                        deny_warnings,
                        silence_warnings,
                    );
                }
            }
            TestStatus::Skipped { .. } => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                write!(writer, "skipped")?;
                writer.reset()?;
                show_time(&mut writer)?;
                writeln!(writer)?;
            }
            TestStatus::CompileError(file_diagnostic) => {
                noirc_errors::reporter::report_all(
                    file_manager.as_file_map(),
                    &[file_diagnostic.clone()],
                    deny_warnings,
                    silence_warnings,
                );
            }
        }

        if show_output && !test_result.output.is_empty() {
            writeln!(writer, "--- {} stdout ---", test_result.name)?;
            write!(writer, "{}", test_result.output)?;
            let name_len = test_result.name.len();
            writeln!(writer, "{}", "-".repeat(name_len + "---  stdout ---".len()))
        } else {
            Ok(())
        }
    }

    fn package_end(
        &self,
        package_name: &str,
        test_results: &[TestResult],
        _file_manager: &FileManager,
        _show_output: bool,
        _deny_warnings: bool,
        _silence_warnings: bool,
    ) -> std::io::Result<()> {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let mut writer = writer.lock();

        let failed_tests: Vec<_> = test_results
            .iter()
            .filter_map(|test_result| test_result.status.failed().then_some(&test_result.name))
            .collect();

        if !failed_tests.is_empty() {
            writeln!(writer)?;
            writeln!(writer, "[{}] Failures:", package_name)?;
            for failed_test in failed_tests {
                writeln!(writer, "     {}", failed_test)?;
            }
            writeln!(writer)?;
        }

        write!(writer, "[{}] ", package_name)?;

        let count_all = test_results.len();
        let count_failed =
            test_results.iter().filter(|test_result| test_result.status.failed()).count();
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
}

pub(super) struct TerseFormatter;

impl Formatter for TerseFormatter {
    fn package_start_async(&self, _package_name: &str, _test_count: usize) -> std::io::Result<()> {
        Ok(())
    }

    fn package_start_sync(&self, package_name: &str, test_count: usize) -> std::io::Result<()> {
        package_start(package_name, test_count)
    }

    fn test_start_async(&self, _name: &str, _package_name: &str) -> std::io::Result<()> {
        Ok(())
    }

    fn test_end_async(
        &self,
        _test_result: &TestResult,
        _file_manager: &FileManager,
        _show_output: bool,
        _deny_warnings: bool,
        _silence_warnings: bool,
    ) -> std::io::Result<()> {
        Ok(())
    }

    fn test_end_sync(
        &self,
        test_result: &TestResult,
        current_test_count: usize,
        total_test_count: usize,
        _file_manager: &FileManager,
        _show_output: bool,
        _deny_warnings: bool,
        _silence_warnings: bool,
    ) -> std::io::Result<()> {
        let writer = StandardStream::stderr(ColorChoice::Always);
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

        if current_test_count % MAX_TESTS_PER_LINE == 0 && current_test_count < total_test_count {
            writeln!(writer, " {}/{}", current_test_count, total_test_count)?;
        }

        Ok(())
    }

    fn package_end(
        &self,
        package_name: &str,
        test_results: &[TestResult],
        file_manager: &FileManager,
        show_output: bool,
        deny_warnings: bool,
        silence_warnings: bool,
    ) -> std::io::Result<()> {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let mut writer = writer.lock();

        if !test_results.is_empty() {
            writeln!(writer)?;
        }

        for test_result in test_results {
            if (show_output && !test_result.output.is_empty()) || test_result.status.failed() {
                writeln!(writer, "--- {} stdout ---", test_result.name)?;
                if !test_result.output.is_empty() {
                    write!(writer, "{}", test_result.output)?;
                }

                match &test_result.status {
                    TestStatus::Pass | TestStatus::Skipped => (),
                    TestStatus::Fail { message, error_diagnostic } => {
                        writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                        writeln!(writer, "{message}")?;
                        writer.reset()?;
                        if let Some(diag) = error_diagnostic {
                            noirc_errors::reporter::report_all(
                                file_manager.as_file_map(),
                                &[diag.clone()],
                                deny_warnings,
                                silence_warnings,
                            );
                        }
                    }
                    TestStatus::CompileError(file_diagnostic) => {
                        noirc_errors::reporter::report_all(
                            file_manager.as_file_map(),
                            &[file_diagnostic.clone()],
                            deny_warnings,
                            silence_warnings,
                        );
                    }
                }

                let name_len = test_result.name.len();
                writeln!(writer, "{}", "-".repeat(name_len + "---  stdout ---".len()))?;
            }
        }

        let failed_tests: Vec<_> = test_results
            .iter()
            .filter_map(|test_result| test_result.status.failed().then_some(&test_result.name))
            .collect();

        if !failed_tests.is_empty() {
            writeln!(writer)?;
            writeln!(writer, "[{}] Failures:", package_name)?;
            for failed_test in failed_tests {
                writeln!(writer, "     {}", failed_test)?;
            }
            writeln!(writer)?;
        }

        write!(writer, "[{}] ", package_name)?;

        let count_all = test_results.len();
        let count_failed =
            test_results.iter().filter(|test_result| test_result.status.failed()).count();
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
}

pub(super) struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn package_start_async(&self, package_name: &str, test_count: usize) -> std::io::Result<()> {
        let json = json!({"type": "suite", "event": "started", "name": package_name, "test_count": test_count});
        println!("{json}");
        Ok(())
    }

    fn package_start_sync(&self, _package_name: &str, _test_count: usize) -> std::io::Result<()> {
        Ok(())
    }

    fn test_start_async(&self, name: &str, package_name: &str) -> std::io::Result<()> {
        let json = json!({"type": "test", "event": "started", "name": name, "suite": package_name});
        println!("{json}");
        Ok(())
    }

    fn test_end_async(
        &self,
        test_result: &TestResult,
        file_manager: &FileManager,
        show_output: bool,
        _deny_warnings: bool,
        silence_warnings: bool,
    ) -> std::io::Result<()> {
        let mut json = Map::new();
        json.insert("type".to_string(), json!("test"));
        json.insert("name".to_string(), json!(&test_result.name));
        json.insert("exec_time".to_string(), json!(test_result.time_to_run.as_secs_f64()));

        let mut stdout = String::new();
        if show_output && !test_result.output.is_empty() {
            stdout.push_str(test_result.output.trim());
        }

        match &test_result.status {
            TestStatus::Pass => {
                json.insert("event".to_string(), json!("ok"));
            }
            TestStatus::Fail { message, error_diagnostic } => {
                json.insert("event".to_string(), json!("failed"));

                if !stdout.is_empty() {
                    stdout.push('\n');
                }
                stdout.push_str(message.trim());

                if let Some(diagnostic) = error_diagnostic {
                    if !(diagnostic.diagnostic.is_warning() && silence_warnings) {
                        stdout.push('\n');
                        stdout.push_str(&diagnostic_to_string(diagnostic, file_manager));
                    }
                }
            }
            TestStatus::Skipped => {
                json.insert("event".to_string(), json!("ignored"));
            }
            TestStatus::CompileError(diagnostic) => {
                json.insert("event".to_string(), json!("failed"));

                if !(diagnostic.diagnostic.is_warning() && silence_warnings) {
                    if !stdout.is_empty() {
                        stdout.push('\n');
                    }
                    stdout.push_str(&diagnostic_to_string(diagnostic, file_manager));
                }
            }
        }

        if !stdout.is_empty() {
            json.insert("stdout".to_string(), json!(stdout));
        }

        let json = json!(json);
        println!("{json}");

        Ok(())
    }

    fn test_end_sync(
        &self,
        _test_result: &TestResult,
        _current_test_count: usize,
        _total_test_count: usize,
        _file_manager: &FileManager,
        _show_output: bool,
        _deny_warnings: bool,
        _silence_warnings: bool,
    ) -> std::io::Result<()> {
        Ok(())
    }

    fn package_end(
        &self,
        _package_name: &str,
        test_results: &[TestResult],
        _file_manager: &FileManager,
        _show_output: bool,
        _deny_warnings: bool,
        _silence_warnings: bool,
    ) -> std::io::Result<()> {
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
        println!("{json}");
        Ok(())
    }
}

fn package_start(package_name: &str, test_count: usize) -> std::io::Result<()> {
    let plural = if test_count == 1 { "" } else { "s" };
    println!("[{package_name}] Running {test_count} test function{plural}");
    Ok(())
}

fn diagnostic_to_string(file_diagnostic: &FileDiagnostic, file_manager: &FileManager) -> String {
    let file_map = file_manager.as_file_map();

    let custom_diagnostic = &file_diagnostic.diagnostic;
    let mut message = String::new();
    message.push_str(custom_diagnostic.message.trim());

    for note in &custom_diagnostic.notes {
        message.push('\n');
        message.push_str(note.trim());
    }

    if let Ok(name) = file_map.get_name(file_diagnostic.file_id) {
        message.push('\n');
        message.push_str(&format!("at {name}"));
    }

    if !custom_diagnostic.call_stack.is_empty() {
        message.push('\n');
        message.push_str(&stack_trace(file_map, &custom_diagnostic.call_stack));
    }

    message
}
