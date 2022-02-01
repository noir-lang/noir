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
}

impl CustomDiagnostic {
    pub fn from_message(msg: &str) -> CustomDiagnostic {
        Self {
            message: msg.to_owned(),
            secondaries: Vec::new(),
            notes: Vec::new(),
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
        }
    }
    pub fn add_note(&mut self, message: String) {
        self.notes.push(message);
    }
    pub fn add_secondary(&mut self, message: String, span: Span) {
        self.secondaries.push(CustomLabel::new(message, span));
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
    pub fn with_diagnostics(
        file_id: usize,
        files: &fm::FileManager,
        diagnostics: &[CustomDiagnostic],
    ) {
        // Convert each Custom Diagnostic into a diagnostic
        let diagnostics: Vec<_> = diagnostics
            .iter()
            .map(|cd| {
                let secondary_labels = cd
                    .secondaries
                    .iter()
                    .map(|sl| {
                        let start_span = sl.span.start() as usize;
                        let end_span = sl.span.end() as usize + 1;
                        Label::secondary(file_id, start_span..end_span).with_message(&sl.message)
                    })
                    .collect();

                Diagnostic::error()
                    .with_message(&cd.message)
                    .with_labels(secondary_labels)
                    .with_notes(cd.notes.clone())
            })
            .collect();

        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();

        for diagnostic in diagnostics.iter() {
            term::emit(
                &mut writer.lock(),
                &config,
                files.as_simple_files(),
                diagnostic,
            )
            .unwrap();
        }

        if !diagnostics.is_empty() {
            writer
                .lock()
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                .unwrap();
            writeln!(
                &mut writer.lock(),
                "error: aborting due to {} previous errors",
                diagnostics.len()
            )
            .unwrap();
        }
    }
}
