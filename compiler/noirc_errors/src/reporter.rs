use std::io::IsTerminal;

use crate::Location;
use crate::function_locations::FunctionLocations;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::Files;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use noirc_span::Span;

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

    pub fn simple_with_kind(
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
        // Avoid adding duplicate labels (can happen during recursive attribute execution)
        let is_duplicate = self
            .secondaries
            .iter()
            .any(|label| label.message == message && label.location == location);
        if !is_duplicate {
            self.secondaries.push(CustomLabel::new(message, location));
        }
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
    function_locations: &FunctionLocations,
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

    let error_count = diagnostics
        .iter()
        .map(|error| u32::from(error.report(files, function_locations, deny_warnings)))
        .sum();

    ReportedErrors { error_count }
}

impl CustomDiagnostic {
    /// Print the report; return true if it was an error.
    pub fn report<'files>(
        &self,
        files: &'files impl Files<'files, FileId = fm::FileId>,
        function_locations: &FunctionLocations,
        deny_warnings: bool,
    ) -> bool {
        report(files, function_locations, self, deny_warnings)
    }
}

/// Report the given diagnostic, and return true if it was an error
pub fn report<'files>(
    files: &'files impl Files<'files, FileId = fm::FileId>,
    function_locations: &FunctionLocations,
    custom_diagnostic: &CustomDiagnostic,
    deny_warnings: bool,
) -> bool {
    let color_choice =
        if std::io::stderr().is_terminal() { ColorChoice::Auto } else { ColorChoice::Never };
    let writer = StandardStream::stderr(color_choice);
    let config = term::Config::default();

    let stack_trace = stack_trace(files, function_locations, &custom_diagnostic.call_stack);
    let diagnostic = convert_diagnostic(custom_diagnostic, stack_trace, deny_warnings);
    term::emit(&mut writer.lock(), &config, files, &diagnostic).unwrap();

    deny_warnings || custom_diagnostic.is_error()
}

fn convert_diagnostic(
    cd: &CustomDiagnostic,
    stack_trace: String,
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
    notes.push(stack_trace);

    diagnostic.with_message(&cd.message).with_labels(secondary_labels).with_notes(notes)
}

pub fn stack_trace<'files>(
    files: &'files impl Files<'files, FileId = fm::FileId>,
    function_locations: &FunctionLocations,
    call_stack: &[Location],
) -> String {
    if call_stack.is_empty() {
        return String::new();
    }

    let repeating_sequences = find_repeating_sequences(call_stack);

    // Compute the length of the longest frame number so we show them like this:
    //   1: ..
    //  23: ..
    // 234: ..
    let maximum_frame_string_length = compute_maximum_frame_string_length(&repeating_sequences);

    let mut result = "Call stack:\n".to_string();
    let mut index = 1;

    // If there are repeated sequences we are going to indent non-repeating sequences so that the entire
    // call stack is aligned (repeating sequences have some ascii chars to show the grouping)
    let has_repetitions = repeating_sequences.iter().any(|(_, times)| *times > 1);

    for (sequence, times) in repeating_sequences {
        for (i, call_item) in sequence.iter().copied().enumerate() {
            let name = function_locations.lookup(call_item).unwrap_or("?");
            let path = files.name(call_item.file).expect("should get file path");
            let source = files.source(call_item.file).expect("should get file source");

            let (line, column) = line_and_column_from_span(source.as_ref(), &call_item.span);
            let prefix = if times == 1 {
                if has_repetitions { "   " } else { "" }
            } else if i == 0 {
                "┌─ "
            } else {
                "│  "
            };
            result += &format!("{prefix}{index:>maximum_frame_string_length$}: {name}\n");

            let prefix =
                if times == 1 { if has_repetitions { "   " } else { "" } } else { "│  " };
            result += &format!("{prefix}        at {path}:{line}:{column}\n");
            index += 1;
        }
        if times > 1 {
            result += &format!("└─ (repeated {times} times)\n");
        }
        index += sequence.len() * (times - 1);
    }

    result
}

/// Computes the maximum number of digits to represent all **shown** frames in the callstack.
fn compute_maximum_frame_string_length(repeating_sequences: &[(Vec<Location>, usize)]) -> usize {
    let mut index = 1;
    let mut maximum_index = 0;
    for (sequence, times) in repeating_sequences {
        index += sequence.len();
        // In a group, the maximum shown frame is the last one in the group
        maximum_index = index;
        index += sequence.len() * (times - 1);
    }
    maximum_index.to_string().len()
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

/// Given an array of items, returns a vector of vectors where each inner vector contains
/// a sequence repeat of a list of items, together with how many times it's repeated.
///
/// For example, given the array `[1, 2, 3, 1, 2, 3, 1, 2, 3, 4, 5, 4, 5, 6, 4, 5, 6, 7, 8]`
/// this function would return `vec![(vec![1, 2, 3], 3), (vec![4, 5], 2), (vec![6, 4, 5, 6, 7, 8], 1)]`.
///
/// For performance reasons, only sequences of length up to 100 are considered.
fn find_repeating_sequences<T: Eq + Copy>(array: &[T]) -> Vec<(Vec<T>, usize)> {
    let mut result: Vec<(Vec<T>, usize)> = Vec::new();
    let mut start = 0;

    while start < array.len() {
        let remaining = &array[start..];
        let mut best_pattern_len = remaining.len();
        let mut best_count = 1;

        for pattern_len in 1..remaining.len().min(100) {
            let pattern = &remaining[..pattern_len];
            let mut count = 1;
            let mut pos = pattern_len;

            while pos + pattern_len <= remaining.len()
                && &remaining[pos..pos + pattern_len] == pattern
            {
                count += 1;
                pos += pattern_len;
            }

            if count > best_count {
                best_count = count;
                best_pattern_len = pattern_len;
            }
        }

        if best_pattern_len == remaining.len() {
            // No repetition, treat the whole remaining as one group
            best_pattern_len = 1;
            best_count = 1;
        }

        // Try to merge a single repetition into a previous pattern if it's only repeated once
        if best_count == 1
            && let Some((pattern, 1)) = result.last_mut()
        {
            pattern.push(remaining[0]);
            start += 1;
            continue;
        }

        result.push((remaining[..best_pattern_len].to_vec(), best_count));
        start += best_pattern_len * best_count;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docstring_example() {
        let array = [1, 2, 3, 1, 2, 3, 1, 2, 3, 4, 5, 4, 5, 6, 4, 5, 6, 7, 8];
        assert_eq!(
            find_repeating_sequences(&array),
            vec![(vec![1, 2, 3], 3), (vec![4, 5], 2), (vec![6, 4, 5, 6, 7, 8], 1),]
        );
    }

    #[test]
    fn test_empty() {
        let array: [i32; 0] = [];
        assert_eq!(find_repeating_sequences(&array), vec![]);
    }

    #[test]
    fn test_single_element() {
        assert_eq!(find_repeating_sequences(&[42]), vec![(vec![42], 1)]);
    }

    #[test]
    fn test_all_same() {
        assert_eq!(find_repeating_sequences(&[3, 3, 3, 3]), vec![(vec![3], 4)]);
    }

    #[test]
    fn test_no_repetitions() {
        assert_eq!(find_repeating_sequences(&[1, 2, 3, 4]), vec![(vec![1, 2, 3, 4], 1)]);
    }

    #[test]
    fn test_two_groups() {
        assert_eq!(
            find_repeating_sequences(&[1, 2, 1, 2, 3, 4, 3, 4]),
            vec![(vec![1, 2], 2), (vec![3, 4], 2)]
        );
    }

    #[test]
    fn test_prefers_more_repetitions() {
        // [1,2] repeats 4 times; [1,2,1,2] repeats 2 times — prefer [1,2]
        assert_eq!(find_repeating_sequences(&[1, 2, 1, 2, 1, 2, 1, 2]), vec![(vec![1, 2], 4)]);
    }

    #[test]
    fn test_no_initial_group() {
        assert_eq!(
            find_repeating_sequences(&[1, 2, 3, 2, 3, 4]),
            vec![(vec![1], 1), (vec![2, 3], 2), (vec![4], 1)]
        );
    }
}
