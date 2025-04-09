use std::io::IsTerminal;
use std::ops::Deref;

use crate::{Location, Span};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::Files;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream, WriteColor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomDiagnostic {
    pub file: fm::FileId,
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
    pub fn from_message(msg: &str, file: fm::FileId) -> CustomDiagnostic {
        Self {
            file,
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
        secondary_location: Location,
        kind: DiagnosticKind,
    ) -> CustomDiagnostic {
        CustomDiagnostic {
            file: secondary_location.file,
            message: primary_message,
            secondaries: vec![CustomLabel::new(secondary_message, secondary_location)],
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
        secondary_location: Location,
    ) -> CustomDiagnostic {
        Self::simple_with_kind(
            primary_message,
            secondary_message,
            secondary_location,
            DiagnosticKind::Error,
        )
    }

    pub fn simple_warning(
        primary_message: String,
        secondary_message: String,
        secondary_location: Location,
    ) -> CustomDiagnostic {
        Self::simple_with_kind(
            primary_message,
            secondary_message,
            secondary_location,
            DiagnosticKind::Warning,
        )
    }

    pub fn simple_info(
        primary_message: String,
        secondary_message: String,
        secondary_location: Location,
    ) -> CustomDiagnostic {
        Self::simple_with_kind(
            primary_message,
            secondary_message,
            secondary_location,
            DiagnosticKind::Info,
        )
    }

    pub fn simple_bug(
        primary_message: String,
        secondary_message: String,
        secondary_location: Location,
    ) -> CustomDiagnostic {
        CustomDiagnostic {
            file: secondary_location.file,
            message: primary_message,
            secondaries: vec![CustomLabel::new(secondary_message, secondary_location)],
            notes: Vec::new(),
            kind: DiagnosticKind::Bug,
            deprecated: false,
            unnecessary: false,
            call_stack: Default::default(),
        }
    }

    pub fn with_call_stack(mut self, call_stack: Vec<Location>) -> Self {
        self.call_stack = call_stack;
        self
    }

    pub fn add_note(&mut self, message: String) {
        self.notes.push(message);
    }

    pub fn add_secondary(&mut self, message: String, location: Location) {
        self.secondaries.push(CustomLabel::new(message, location));
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
    pub location: Location,
}

impl CustomLabel {
    fn new(message: String, location: Location) -> CustomLabel {
        CustomLabel { message, location }
    }
}

/// Writes the given diagnostics to stderr and returns the count
/// of diagnostics that were errors.
pub fn report_all<'files>(
    files: &'files impl Files<'files, FileId = fm::FileId>,
    diagnostics: &[CustomDiagnostic],
    deny_warnings: bool,
    silence_warnings: bool,
) -> ReportedErrors {
    // Report warnings before any errors
    let (warnings_and_bugs, mut errors): (Vec<_>, _) =
        diagnostics.iter().partition(|item| !item.is_error());

    let (warnings, mut bugs): (Vec<_>, _) =
        warnings_and_bugs.iter().partition(|item| item.is_warning());
    let mut diagnostics = if silence_warnings { Vec::new() } else { warnings };
    diagnostics.append(&mut bugs);
    diagnostics.append(&mut errors);

    let error_count =
        diagnostics.iter().map(|error| error.report(files, deny_warnings) as u32).sum();

    ReportedErrors { error_count }
}

impl CustomDiagnostic {
    /// Print the report; return true if it was an error.
    pub fn report<'files>(
        &self,
        files: &'files impl Files<'files, FileId = fm::FileId>,
        deny_warnings: bool,
    ) -> bool {
        report(files, self, deny_warnings)
    }
}

/// Report the given diagnostic, and return true if it was an error
pub fn report<'files>(
    files: &'files impl Files<'files, FileId = fm::FileId>,
    custom_diagnostic: &CustomDiagnostic,
    deny_warnings: bool,
) -> bool {
    let color_choice =
        if std::io::stderr().is_terminal() { ColorChoice::Auto } else { ColorChoice::Never };
    let writer = StandardStream::stderr(color_choice);
    report_with(&mut writer.lock(), files, custom_diagnostic, deny_warnings)
}

pub fn report_with<'files>(
    writer: &mut impl WriteColor,
    files: &'files impl Files<'files, FileId = fm::FileId>,
    custom_diagnostic: &CustomDiagnostic,
    deny_warnings: bool,
) -> bool {
    let config = term::Config::default();

    let diagnostic = convert_diagnostic(custom_diagnostic, files, deny_warnings);
    term::emit(writer, &config, files, &diagnostic).unwrap();

    deny_warnings || custom_diagnostic.is_error()
}

fn convert_diagnostic<'files>(
    cd: &CustomDiagnostic,
    files: &'files impl Files<'files, FileId = fm::FileId>,
    deny_warnings: bool,
) -> Diagnostic<fm::FileId> {
    let diagnostic = match (cd.kind, deny_warnings) {
        (DiagnosticKind::Warning, false) => Diagnostic::warning(),
        (DiagnosticKind::Info, _) => Diagnostic::note(),
        (DiagnosticKind::Bug, ..) => Diagnostic::bug(),
        _ => Diagnostic::error(),
    };

    let secondary_labels = cd
        .secondaries
        .iter()
        .map(|custom_label| {
            let location = custom_label.location;
            let span = location.span;
            let start_span = span.start() as usize;
            let end_span = span.end() as usize;
            let file = location.file;
            Label::secondary(file, start_span..end_span).with_message(&custom_label.message)
        })
        .collect();

    let mut notes = cd.notes.clone();
    let stack_trace = stack_trace(files, &cd.call_stack);
    notes.push(stack_trace);

    diagnostic.with_message(&cd.message).with_labels(secondary_labels).with_notes(notes)
}

pub fn stack_trace<'files>(
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


#[derive(Default)]
pub struct ReportBuffer(Vec<u8>);

impl Deref for ReportBuffer {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::io::Write for ReportBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl WriteColor for ReportBuffer {
    fn supports_color(&self) -> bool {
        false
    }

    fn set_color(&mut self, _spec: &term::termcolor::ColorSpec) -> std::io::Result<()> {
      Ok(())
    }

    fn reset(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}