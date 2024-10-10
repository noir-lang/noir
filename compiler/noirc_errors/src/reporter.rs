use std::io::IsTerminal;

use crate::{FileDiagnostic, Location, Span};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::Files;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomDiagnostic {
    pub message: String,
    pub secondaries: Vec<CustomLabel>,
    pub notes: Vec<String>,
    pub kind: DiagnosticKind,
    pub deprecated: bool,
    pub unnecessary: bool,

    /// An optional call stack to display the full runtime call stack
    /// leading up to a runtime error. If this is empty it will not be displayed.
    pub call_stack: Vec<Location>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    Error,
    Bug,
    Warning,
    Info,
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
            deprecated: false,
            unnecessary: false,
            call_stack: Default::default(),
        }
    }

    fn simple_with_kind(
        primary_message: String,
        secondary_message: String,
        secondary_span: Span,
        kind: DiagnosticKind,
    ) -> CustomDiagnostic {
        CustomDiagnostic {
            message: primary_message,
            secondaries: vec![CustomLabel::new(secondary_message, secondary_span, None)],
            notes: Vec::new(),
            kind,
            deprecated: false,
            unnecessary: false,
            call_stack: Default::default(),
        }
    }

    pub fn simple_error(
        primary_message: String,
        secondary_message: String,
        secondary_span: Span,
    ) -> CustomDiagnostic {
        Self::simple_with_kind(
            primary_message,
            secondary_message,
            secondary_span,
            DiagnosticKind::Error,
        )
    }

    pub fn simple_warning(
        primary_message: String,
        secondary_message: String,
        secondary_span: Span,
    ) -> CustomDiagnostic {
        Self::simple_with_kind(
            primary_message,
            secondary_message,
            secondary_span,
            DiagnosticKind::Warning,
        )
    }

    pub fn simple_info(
        primary_message: String,
        secondary_message: String,
        secondary_span: Span,
    ) -> CustomDiagnostic {
        Self::simple_with_kind(
            primary_message,
            secondary_message,
            secondary_span,
            DiagnosticKind::Info,
        )
    }

    pub fn simple_bug(
        primary_message: String,
        secondary_message: String,
        secondary_span: Span,
    ) -> CustomDiagnostic {
        CustomDiagnostic {
            message: primary_message,
            secondaries: vec![CustomLabel::new(secondary_message, secondary_span, None)],
            notes: Vec::new(),
            kind: DiagnosticKind::Bug,
            deprecated: false,
            unnecessary: false,
            call_stack: Default::default(),
        }
    }

    pub fn in_file(self, file_id: fm::FileId) -> FileDiagnostic {
        FileDiagnostic::new(file_id, self)
    }

    pub fn with_call_stack(mut self, call_stack: Vec<Location>) -> Self {
        self.call_stack = call_stack;
        self
    }

    pub fn add_note(&mut self, message: String) {
        self.notes.push(message);
    }

    pub fn add_secondary(&mut self, message: String, span: Span) {
        self.secondaries.push(CustomLabel::new(message, span, None));
    }

    pub fn add_secondary_with_file(&mut self, message: String, span: Span, file: fm::FileId) {
        self.secondaries.push(CustomLabel::new(message, span, Some(file)));
    }

    pub fn is_error(&self) -> bool {
        matches!(self.kind, DiagnosticKind::Error)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self.kind, DiagnosticKind::Warning)
    }

    pub fn is_info(&self) -> bool {
        matches!(self.kind, DiagnosticKind::Info)
    }

    pub fn is_bug(&self) -> bool {
        matches!(self.kind, DiagnosticKind::Bug)
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
    pub message: String,
    pub span: Span,
    pub file: Option<fm::FileId>,
}

impl CustomLabel {
    fn new(message: String, span: Span, file: Option<fm::FileId>) -> CustomLabel {
        CustomLabel { message, span, file }
    }
}

/// Writes the given diagnostics to stderr and returns the count
/// of diagnostics that were errors.
pub fn report_all<'files>(
    files: &'files impl Files<'files, FileId = fm::FileId>,
    diagnostics: &[FileDiagnostic],
    deny_warnings: bool,
    silence_warnings: bool,
) -> ReportedErrors {
    // Report warnings before any errors
    let (warnings_and_bugs, mut errors): (Vec<_>, _) =
        diagnostics.iter().partition(|item| !item.diagnostic.is_error());

    let (warnings, mut bugs): (Vec<_>, _) =
        warnings_and_bugs.iter().partition(|item| item.diagnostic.is_warning());
    let mut diagnostics = if silence_warnings { Vec::new() } else { warnings };
    diagnostics.append(&mut bugs);
    diagnostics.append(&mut errors);

    let error_count =
        diagnostics.iter().map(|error| error.report(files, deny_warnings) as u32).sum();

    ReportedErrors { error_count }
}

impl FileDiagnostic {
    pub fn report<'files>(
        &self,
        files: &'files impl Files<'files, FileId = fm::FileId>,
        deny_warnings: bool,
    ) -> bool {
        report(files, &self.diagnostic, Some(self.file_id), deny_warnings)
    }
}

/// Report the given diagnostic, and return true if it was an error
pub fn report<'files>(
    files: &'files impl Files<'files, FileId = fm::FileId>,
    custom_diagnostic: &CustomDiagnostic,
    file: Option<fm::FileId>,
    deny_warnings: bool,
) -> bool {
    let color_choice =
        if std::io::stderr().is_terminal() { ColorChoice::Auto } else { ColorChoice::Never };
    let writer = StandardStream::stderr(color_choice);
    let config = codespan_reporting::term::Config::default();

    let stack_trace = stack_trace(files, &custom_diagnostic.call_stack);
    let diagnostic = convert_diagnostic(custom_diagnostic, file, stack_trace, deny_warnings);
    term::emit(&mut writer.lock(), &config, files, &diagnostic).unwrap();

    deny_warnings || custom_diagnostic.is_error()
}

fn convert_diagnostic(
    cd: &CustomDiagnostic,
    file: Option<fm::FileId>,
    stack_trace: String,
    deny_warnings: bool,
) -> Diagnostic<fm::FileId> {
    let diagnostic = match (cd.kind, deny_warnings) {
        (DiagnosticKind::Warning, false) => Diagnostic::warning(),
        (DiagnosticKind::Info, _) => Diagnostic::note(),
        (DiagnosticKind::Bug, ..) => Diagnostic::bug(),
        _ => Diagnostic::error(),
    };

    let secondary_labels = if let Some(file_id) = file {
        cd.secondaries
            .iter()
            .map(|sl| {
                let start_span = sl.span.start() as usize;
                let end_span = sl.span.end() as usize;
                let file = sl.file.unwrap_or(file_id);
                Label::secondary(file, start_span..end_span).with_message(&sl.message)
            })
            .collect()
    } else {
        vec![]
    };

    let mut notes = cd.notes.clone();
    notes.push(stack_trace);

    diagnostic.with_message(&cd.message).with_labels(secondary_labels).with_notes(notes)
}

fn stack_trace<'files>(
    files: &'files impl Files<'files, FileId = fm::FileId>,
    call_stack: &[Location],
) -> String {
    if call_stack.is_empty() {
        return String::new();
    }

    let mut result = "Call stack:\n".to_string();

    for (i, call_item) in call_stack.iter().enumerate() {
        let path = files.name(call_item.file).expect("should get file path");
        let source = files.source(call_item.file).expect("should get file source");

        let (line, column) = line_and_column_from_span(source.as_ref(), &call_item.span);
        result += &format!("{}. {}:{}:{}\n", i + 1, path, line, column);
    }

    result
}

pub fn line_and_column_from_span(source: &str, span: &Span) -> (u32, u32) {
    let mut line = 1;
    let mut column = 0;

    for (i, char) in source.chars().enumerate() {
        column += 1;

        if char == '\n' {
            line += 1;
            column = 0;
        }

        if span.start() <= i as u32 {
            break;
        }
    }

    (line, column)
}
