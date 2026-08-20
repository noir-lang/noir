//! Source-location annotations for displayed ACIR opcodes.
//!
//! Uses the call stacks recorded in [DebugInfo] and the sources in an artifact's
//! `file_map` to describe, for each ACIR opcode, the Noir source it was compiled
//! from. The annotations are meant to be attached as `//` comments when displaying
//! a circuit (see `display_program` in the `acir` crate).

use std::collections::BTreeMap;
use std::path::Path;

use acir::AcirField;
use acir::circuit::{AcirOpcodeLocation, Program};
use fm::FileId;
use noirc_errors::Location;
use noirc_errors::call_stack::CallStack;
use noirc_errors::reporter::line_and_column_from_span;

use crate::debug::{DebugFile, DebugInfo};

/// Builds the per-circuit opcode annotations (see [acir_opcode_annotations]) for every
/// ACIR function of a program. `debug` must be parallel to `program.functions`.
pub fn program_opcode_annotations<F: AcirField>(
    program: &Program<F>,
    debug: &[DebugInfo],
    file_map: &BTreeMap<FileId, DebugFile>,
) -> Vec<BTreeMap<usize, String>> {
    program
        .functions
        .iter()
        .zip(debug)
        .map(|(circuit, debug_info)| {
            acir_opcode_annotations(debug_info, file_map, circuit.opcodes.len())
        })
        .collect()
}

/// Builds `opcode index -> comment` annotations describing the Noir source each opcode
/// of a single ACIR circuit was compiled from.
///
/// Only the first opcode of a consecutive run sharing the same call stack is annotated,
/// so a source expression that expands to several opcodes gets a single comment. An
/// opcode with no recorded location that follows an annotated one is marked explicitly,
/// so that it doesn't appear to belong to the preceding run.
pub fn acir_opcode_annotations(
    debug_info: &DebugInfo,
    file_map: &BTreeMap<FileId, DebugFile>,
    num_opcodes: usize,
) -> BTreeMap<usize, String> {
    let mut annotations = BTreeMap::new();
    let mut previous_call_stack = None;
    let current_dir = std::env::current_dir().ok();

    for index in 0..num_opcodes {
        let call_stack_id = debug_info.acir_locations.get(&AcirOpcodeLocation::new(index)).copied();
        if call_stack_id == previous_call_stack {
            continue;
        }
        // Opcodes before the first attributed one are left bare rather than marked
        // as having no source location.
        if call_stack_id.is_none() && previous_call_stack.is_none() {
            continue;
        }
        previous_call_stack = call_stack_id;

        let annotation = call_stack_id.and_then(|call_stack_id| {
            let call_stack = debug_info.location_tree.get_call_stack(call_stack_id);
            format_call_stack_annotation(&call_stack, file_map, current_dir.as_deref())
        });
        annotations.insert(index, annotation.unwrap_or_else(|| "no source location".to_string()));
    }

    annotations
}

/// Formats a call stack as `file:line:col: snippet`, where the location and snippet are
/// those of the innermost frame. If the innermost frame was reached through inlined
/// calls, the callers are appended as ` (via caller1 <- caller2)`, innermost caller
/// first. Returns `None` if no frame has a resolvable location.
fn format_call_stack_annotation(
    call_stack: &CallStack,
    file_map: &BTreeMap<FileId, DebugFile>,
    current_dir: Option<&Path>,
) -> Option<String> {
    let locations: Vec<&Location> =
        call_stack.into_iter().filter(|location| !location.is_dummy()).collect();
    let (innermost, callers) = locations.split_last()?;

    let mut annotation = format_location(innermost, file_map, current_dir, true)?;

    let callers = callers
        .iter()
        .rev()
        .filter_map(|location| format_location(location, file_map, current_dir, false))
        .collect::<Vec<_>>();
    if !callers.is_empty() {
        annotation.push_str(&format!(" (via {})", callers.join(" <- ")));
    }

    Some(annotation)
}

/// Formats a single [Location] as `file:line:col`, appending `: snippet` when requested.
/// The file path is shown relative to the current directory when possible. The snippet
/// is the source text covered by the location's span, with whitespace runs (including
/// newlines) collapsed to single spaces and truncated to a maximum length.
/// Returns `None` if the location's file is not in the file map.
fn format_location(
    location: &Location,
    file_map: &BTreeMap<FileId, DebugFile>,
    current_dir: Option<&Path>,
    with_snippet: bool,
) -> Option<String> {
    let file = file_map.get(&location.file)?;
    let path = current_dir
        .and_then(|current_dir| file.path.strip_prefix(current_dir).ok())
        .unwrap_or(&file.path);
    let (line, column) = line_and_column_from_span(&file.source, &location.span);
    let mut result = format!("{}:{line}:{column}", path.display());

    if with_snippet
        && let Some(snippet) =
            file.source.get(location.span.start() as usize..location.span.end() as usize)
    {
        const MAX_SNIPPET_LENGTH: usize = 80;

        let snippet = snippet.split_whitespace().collect::<Vec<_>>().join(" ");
        result.push_str(": ");
        if snippet.chars().count() > MAX_SNIPPET_LENGTH {
            result.extend(snippet.chars().take(MAX_SNIPPET_LENGTH));
            result.push('…');
        } else {
            result.push_str(&snippet);
        }
    }

    Some(result)
}
