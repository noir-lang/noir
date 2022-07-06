use crate::Span;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{
    Color, ColorChoice, ColorSpec, StandardStream, WriteColor,
};
use std::io::Write;

#[derive(Debug, PartialEq, Eq)]
pub struct CustomDiagnostic {
    message: String,
    secondaries: Vec<CustomLabel>,
    notes: Vec<String>,
    kind: DiagnosticKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    Error,
    Warning,
}

impl CustomDiagnostic {
    pub fn from_message(msg: &str) -> CustomDiagnostic {
        Self {
            message: msg.to_owned(),
            secondaries: Vec::new(),
            notes: Vec::new(),
            kind: DiagnosticKind::Error,
        }
    }

    pub fn simple_error(
        primary_message: String,
        secondary_message: String,
        secondary_span: Span,
    ) -> CustomDiagnostic {
        CustomDiagnostic {
            message: primary_message,
            secondaries: vec![CustomLabel::new(secondary_message, secondary_span)],
            notes: Vec::new(),
            kind: DiagnosticKind::Error,
        }
    }

    pub fn simple_warning(
        primary_message: String,
        secondary_message: String,
        secondary_span: Span,
    ) -> CustomDiagnostic {
        CustomDiagnostic {
            message: primary_message,
            secondaries: vec![CustomLabel::new(secondary_message, secondary_span)],
            notes: Vec::new(),
            kind: DiagnosticKind::Warning,
        }
    }

    pub fn add_note(&mut self, message: String) {
        self.notes.push(message);
    }

    pub fn add_secondary(&mut self, message: String, span: Span) {
        self.secondaries.push(CustomLabel::new(message, span));
    }
}

impl std::fmt::Display for CustomDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;

        for secondary in &self.secondaries {
            write!(f, "\nsecondary: {}", secondary.message)?;
        }

        for note in &self.notes {
            write!(f, "\nnote: {}", note)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CustomLabel {
    pub message: String,
    pub span: Span,
}

impl CustomLabel {
    pub fn new(message: String, span: Span) -> CustomLabel {
        CustomLabel { message, span }
    }
}

pub struct Reporter;

impl Reporter {
    /// Writes the given diagnostics to stderr and returns the count
    /// of diagnostics that were errors.
    pub fn with_diagnostics(
        file_id: usize,
        files: &fm::FileManager,
        diagnostics: &[CustomDiagnostic],
    ) -> usize {
        let mut error_count = 0;

        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();

        // Convert each Custom Diagnostic into a diagnostic
        for custom_diagnostic in diagnostics {
            let secondary_labels = custom_diagnostic
                .secondaries
                .iter()
                .map(|sl| {
                    let start_span = sl.span.start() as usize;
                    let end_span = sl.span.end() as usize + 1;
                    Label::secondary(file_id, start_span..end_span).with_message(&sl.message)
                })
                .collect();

            let diagnostic = match custom_diagnostic.kind {
                DiagnosticKind::Error => {
                    error_count += 1;
                    Diagnostic::error()
                }
                DiagnosticKind::Warning => Diagnostic::warning(),
            };

            let diagnostic = diagnostic
                .with_message(&custom_diagnostic.message)
                .with_labels(secondary_labels)
                .with_notes(custom_diagnostic.notes.clone());

            term::emit(&mut writer.lock(), &config, files.as_simple_files(), &diagnostic).unwrap();
        }

        error_count
    }

    pub fn finish(error_count: usize) {
        if error_count != 0 {
            let writer = StandardStream::stderr(ColorChoice::Always);
            let mut writer = writer.lock();

            writer.set_color(ColorSpec::new().set_fg(Some(Color::Red))).unwrap();

            writeln!(&mut writer, "error: aborting due to {} previous errors", error_count)
                .unwrap();

            std::process::exit(1);
        }
    }
}
