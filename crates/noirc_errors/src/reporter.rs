use crate::{FileDiagnostic, Location, Span};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomDiagnostic {
    pub message: String,
    pub secondaries: Vec<CustomLabel>,
    notes: Vec<String>,
    pub kind: DiagnosticKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    Error,
    Warning,
}

/// A count of errors that have been already reported to stderr
#[derive(Debug, Copy, Clone)]
pub struct ReportedErrors {
    pub error_count: u32,
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

    pub fn in_file(self, file_id: fm::FileId) -> FileDiagnostic {
        FileDiagnostic::new(file_id, self)
    }

    pub fn add_note(&mut self, message: String) {
        self.notes.push(message);
    }

    pub fn add_secondary(&mut self, message: String, span: Span) {
        self.secondaries.push(CustomLabel::new(message, span));
    }

    pub fn is_error(&self) -> bool {
        matches!(self.kind, DiagnosticKind::Error)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self.kind, DiagnosticKind::Warning)
    }
}

impl std::fmt::Display for CustomDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;

        for secondary in &self.secondaries {
            write!(f, "\nsecondary: {}", secondary.message)?;
        }

        for note in &self.notes {
            write!(f, "\nnote: {note}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomLabel {
    message: String,
    pub span: Span,
}

impl CustomLabel {
    fn new(message: String, span: Span) -> CustomLabel {
        CustomLabel { message, span }
    }
}

/// Writes the given diagnostics to stderr and returns the count
/// of diagnostics that were errors.
pub fn report_all(
    files: &fm::FileManager,
    diagnostics: &[FileDiagnostic],
    deny_warnings: bool,
) -> ReportedErrors {
    let error_count =
        diagnostics.iter().map(|error| error.report(files, deny_warnings) as u32).sum();

    ReportedErrors { error_count }
}

impl FileDiagnostic {
    pub fn report(&self, files: &fm::FileManager, deny_warnings: bool) -> bool {
        report(files, &self.diagnostic, Some(self.file_id), &self.call_stack, deny_warnings)
    }
}

/// Report the given diagnostic, and return true if it was an error
pub fn report(
    files: &fm::FileManager,
    custom_diagnostic: &CustomDiagnostic,
    file: Option<fm::FileId>,
    call_stack: &[Location],
    deny_warnings: bool,
) -> bool {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    let stack_trace = stack_trace(files, call_stack);
    let diagnostic = convert_diagnostic(custom_diagnostic, file, stack_trace, deny_warnings);
    term::emit(&mut writer.lock(), &config, files.as_simple_files(), &diagnostic).unwrap();

    deny_warnings || custom_diagnostic.is_error()
}

fn convert_diagnostic(
    cd: &CustomDiagnostic,
    file: Option<fm::FileId>,
    stack_trace: String,
    deny_warnings: bool,
) -> Diagnostic<usize> {
    let diagnostic = match (cd.kind, deny_warnings) {
        (DiagnosticKind::Warning, false) => Diagnostic::warning(),
        _ => Diagnostic::error(),
    };

    let secondary_labels = if let Some(file_id) = file {
        cd.secondaries
            .iter()
            .map(|sl| {
                let start_span = sl.span.start() as usize;
                let end_span = sl.span.end() as usize + 1;
                Label::secondary(file_id.as_usize(), start_span..end_span).with_message(&sl.message)
            })
            .collect()
    } else {
        vec![]
    };

    let mut notes = cd.notes.clone();
    notes.push(stack_trace);

    diagnostic.with_message(&cd.message).with_labels(secondary_labels).with_notes(notes)
}

fn stack_trace(files: &fm::FileManager, call_stack: &[Location]) -> String {
    if call_stack.is_empty() {
        return String::new();
    }

    let mut result = "Call stack:\n".to_string();

    for (i, call_item) in call_stack.iter().enumerate() {
        let path = files.path(call_item.file);
        let source = files.fetch_file(call_item.file).source();

        let (line, column) = location(source, call_item.span.start());
        result += &format!("{}. {}.nr:{}:{}\n", i + 1, path.display(), line, column);
    }

    result
}

fn location(source: &str, span_start: u32) -> (u32, u32) {
    let mut line = 1;
    let mut column = 0;

    for (i, char) in source.chars().enumerate() {
        column += 1;

        if char == '\n' {
            line += 1;
            column = 0;
        }

        if span_start <= i as u32 {
            break;
        }
    }

    (line, column)
}
