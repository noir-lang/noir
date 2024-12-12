use std::{io::Write, panic::RefUnwindSafe, time::Duration};

use fm::FileManager;
use nargo::ops::TestStatus;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, StandardStreamLock, WriteColor};

use super::TestResult;

pub(super) trait Formatter: Send + Sync + RefUnwindSafe {
    fn package_start(&self, package_name: &str, test_count: usize) -> std::io::Result<()>;

    #[allow(clippy::too_many_arguments)]
    fn test_end(
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
    fn package_start(&self, package_name: &str, test_count: usize) -> std::io::Result<()> {
        package_start(package_name, test_count)
    }

    fn test_end(
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
    fn package_start(&self, package_name: &str, test_count: usize) -> std::io::Result<()> {
        package_start(package_name, test_count)
    }

    fn test_end(
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

fn package_start(package_name: &str, test_count: usize) -> std::io::Result<()> {
    let plural = if test_count == 1 { "" } else { "s" };
    println!("[{package_name}] Running {test_count} test function{plural}");
    Ok(())
}
