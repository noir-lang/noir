use std::path::Path;
use std::{collections::BTreeMap, io::BufWriter};

use acir::circuit::brillig::BrilligFunctionId;
use acir::circuit::OpcodeLocation;
use color_eyre::eyre::{self};
use fm::codespan_files::Files;
use fxhash::FxHashMap as HashMap;
use inferno::flamegraph::{from_lines, Options, TextTruncateDirection};
use noirc_errors::debug_info::DebugInfo;
use noirc_errors::reporter::line_and_column_from_span;
use noirc_errors::Location;
use noirc_evaluator::brillig::ProcedureId;

pub(crate) trait Sample {
    fn count(&self) -> usize;

    fn brillig_function_id(&self) -> Option<BrilligFunctionId>;

    fn call_stack(&self) -> &[OpcodeLocation];

    fn opcode(self) -> Option<String>;
}

#[derive(Debug)]
pub(crate) struct CompilationSample {
    pub(crate) opcode: Option<String>,
    pub(crate) call_stack: Vec<OpcodeLocation>,
    pub(crate) count: usize,
    pub(crate) brillig_function_id: Option<BrilligFunctionId>,
}

impl Sample for CompilationSample {
    fn count(&self) -> usize {
        self.count
    }

    fn brillig_function_id(&self) -> Option<BrilligFunctionId> {
        self.brillig_function_id
    }

    fn call_stack(&self) -> &[OpcodeLocation] {
        &self.call_stack
    }

    fn opcode(self) -> Option<String> {
        self.opcode
    }
}

pub(crate) struct BrilligExecutionSample {
    pub(crate) opcode: Option<String>,
    pub(crate) call_stack: Vec<OpcodeLocation>,
    pub(crate) brillig_function_id: Option<BrilligFunctionId>,
}

impl Sample for BrilligExecutionSample {
    fn count(&self) -> usize {
        1
    }

    fn brillig_function_id(&self) -> Option<BrilligFunctionId> {
        self.brillig_function_id
    }

    fn call_stack(&self) -> &[OpcodeLocation] {
        &self.call_stack
    }

    fn opcode(self) -> Option<String> {
        self.opcode
    }
}

#[derive(Debug, Default)]
pub(crate) struct FoldedStackItem {
    pub(crate) total_samples: usize,
    pub(crate) nested_items: BTreeMap<String, FoldedStackItem>,
}

pub(crate) trait FlamegraphGenerator {
    #[allow(clippy::too_many_arguments)]
    fn generate_flamegraph<'files, S: Sample>(
        &self,
        samples: Vec<S>,
        debug_symbols: &DebugInfo,
        files: &'files impl Files<'files, FileId = fm::FileId>,
        artifact_name: &str,
        function_name: &str,
        output_path: &Path,
    ) -> eyre::Result<()>;
}

pub(crate) struct InfernoFlamegraphGenerator {
    pub(crate) count_name: String,
}

impl FlamegraphGenerator for InfernoFlamegraphGenerator {
    fn generate_flamegraph<'files, S: Sample>(
        &self,
        samples: Vec<S>,
        debug_symbols: &DebugInfo,
        files: &'files impl Files<'files, FileId = fm::FileId>,
        artifact_name: &str,
        function_name: &str,
        output_path: &Path,
    ) -> eyre::Result<()> {
        let folded_lines = generate_folded_sorted_lines(samples, debug_symbols, files);
        let flamegraph_file = std::fs::File::create(output_path)?;
        let flamegraph_writer = BufWriter::new(flamegraph_file);

        let mut options = Options::default();
        options.hash = true;
        options.deterministic = true;
        options.title = format!("{}-{}", artifact_name, function_name);
        options.frame_height = 24;
        options.color_diffusion = true;
        options.min_width = 0.0;
        options.count_name = self.count_name.clone();
        options.text_truncate_direction = TextTruncateDirection::Right;

        from_lines(
            &mut options,
            folded_lines.iter().map(|as_string| as_string.as_str()),
            flamegraph_writer,
        )?;

        Ok(())
    }
}

fn generate_folded_sorted_lines<'files, S: Sample>(
    samples: Vec<S>,
    debug_symbols: &DebugInfo,
    files: &'files impl Files<'files, FileId = fm::FileId>,
) -> Vec<String> {
    // Create a nested hashmap with the stack items, folding the gates for all the callsites that are equal
    let mut folded_stack_items = BTreeMap::new();

    let mut resolution_cache: HashMap<OpcodeLocation, Vec<String>> = HashMap::default();
    for sample in samples {
        let mut location_names = Vec::with_capacity(sample.call_stack().len());
        for opcode_location in sample.call_stack() {
            let callsite_labels = resolution_cache
                .entry(*opcode_location)
                .or_insert_with(|| {
                    find_callsite_labels(
                        debug_symbols,
                        opcode_location,
                        sample.brillig_function_id(),
                        files,
                    )
                })
                .clone();

            location_names.extend(callsite_labels);
        }

        // We move `sample` by calling `sample.opcode()` so we want to fetch the sample count here.
        let count = sample.count();

        if let Some(opcode) = sample.opcode() {
            location_names.push(opcode);
        }

        add_locations_to_folded_stack_items(&mut folded_stack_items, location_names, count);
    }

    to_folded_sorted_lines(&folded_stack_items, Default::default())
}

fn find_callsite_labels<'files>(
    debug_symbols: &DebugInfo,
    opcode_location: &OpcodeLocation,
    brillig_function_id: Option<BrilligFunctionId>,
    files: &'files impl Files<'files, FileId = fm::FileId>,
) -> Vec<String> {
    let mut procedure_id = None;
    let source_locations = debug_symbols.opcode_location(opcode_location).unwrap_or_else(|| {
        if let (Some(brillig_function_id), Some(brillig_location)) =
            (brillig_function_id, opcode_location.to_brillig_location())
        {
            let procedure_locs = debug_symbols.brillig_procedure_locs.get(&brillig_function_id);
            if let Some(procedure_locs) = procedure_locs {
                for (procedure, range) in procedure_locs.iter() {
                    if brillig_location.0 >= range.0 && brillig_location.0 <= range.1 {
                        procedure_id = Some(*procedure);
                        break;
                    }
                }
            }
            let brillig_locations = debug_symbols.brillig_locations.get(&brillig_function_id);
            if let Some(brillig_locations) = brillig_locations {
                brillig_locations.get(&brillig_location).cloned().unwrap_or_default()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    });

    let mut callsite_labels: Vec<_> = source_locations
        .into_iter()
        .map(|location| location_to_callsite_label(location, files))
        .collect();

    if let Some(procedure_id) = procedure_id {
        callsite_labels.push(format!("procedure::{}", ProcedureId::from_debug_id(procedure_id)));
    }

    callsite_labels
}

fn location_to_callsite_label<'files>(
    location: Location,
    files: &'files impl Files<'files, FileId = fm::FileId>,
) -> String {
    let filename =
        Path::new(&files.name(location.file).expect("should have a file path").to_string())
            .file_name()
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or("invalid_path".to_string());
    let source = files.source(location.file).expect("should have a file source");

    let code_slice = source
        .as_ref()
        .chars()
        .skip(location.span.start() as usize)
        .take(location.span.end() as usize - location.span.start() as usize)
        .collect::<String>();

    // ";" is used for frame separation, and is not allowed by inferno
    // Check code slice for ";" and replace it with 'GREEK QUESTION MARK' (U+037E)
    let code_slice = code_slice.replace(';', "\u{037E}");

    let (line, column) = line_and_column_from_span(source.as_ref(), &location.span);

    format!("{}:{}:{}::{}", filename, line, column, code_slice)
}

fn add_locations_to_folded_stack_items(
    stack_items: &mut BTreeMap<String, FoldedStackItem>,
    locations: Vec<String>,
    count: usize,
) {
    let mut child_map = stack_items;
    for (index, location) in locations.iter().enumerate() {
        let current_item = child_map.entry(location.clone()).or_default();

        child_map = &mut current_item.nested_items;

        if index == locations.len() - 1 {
            current_item.total_samples += count;
        }
    }
}

/// Creates a vector of lines in the format that inferno expects from a nested hashmap of stack items
/// The lines have to be sorted in the following way, exploring the graph in a depth-first manner:
/// main 100
/// main::foo 0
/// main::foo::bar 200
/// main::baz 27
/// main::baz::qux 800
fn to_folded_sorted_lines(
    folded_stack_items: &BTreeMap<String, FoldedStackItem>,
    parent_stacks: im::Vector<String>,
) -> Vec<String> {
    let mut result_vector = Vec::with_capacity(folded_stack_items.len());

    for (location, folded_stack_item) in folded_stack_items.iter() {
        if folded_stack_item.total_samples > 0 {
            let frame_list: Vec<String> =
                parent_stacks.iter().cloned().chain(std::iter::once(location.clone())).collect();
            let line: String =
                format!("{} {}", frame_list.join(";"), folded_stack_item.total_samples);

            result_vector.push(line);
        };

        let mut new_parent_stacks = parent_stacks.clone();
        new_parent_stacks.push_back(location.clone());
        let child_lines =
            to_folded_sorted_lines(&folded_stack_item.nested_items, new_parent_stacks);

        result_vector.extend(child_lines);
    }

    result_vector
}

#[cfg(test)]
mod tests {
    use acir::{
        circuit::{opcodes::BlockId, Opcode as AcirOpcode, OpcodeLocation},
        native_types::Expression,
        FieldElement,
    };
    use fm::FileManager;
    use noirc_errors::{debug_info::DebugInfo, Location, Span};
    use std::{collections::BTreeMap, path::Path};

    use crate::{flamegraph::CompilationSample, opcode_formatter::format_acir_opcode};

    use super::generate_folded_sorted_lines;

    fn find_spans_for(source: &str, needle: &str) -> Vec<Span> {
        let mut spans = Vec::new();
        let mut start = 0;
        while let Some(start_idx) = source[start..].find(needle) {
            let start_idx = start + start_idx;
            let end_idx = start_idx + needle.len();
            spans.push(Span::inclusive(start_idx as u32, end_idx as u32 - 1));
            start = end_idx;
        }
        spans
    }

    #[test]
    fn simple_test_case() {
        let source_code = r##"
        fn main() {
            foo();
            bar();
            whatever();
        }
        fn foo() {
            baz();
        }
        fn bar () {
            whatever()
        }
        fn baz () {
            whatever()
        }
        "##;

        let source_file_name = Path::new("main.nr");
        let temp_dir = tempfile::tempdir().unwrap();

        let mut fm = FileManager::new(temp_dir.path());
        let file_id = fm.add_file_with_source(source_file_name, source_code.to_string()).unwrap();

        let main_declaration_location =
            Location::new(find_spans_for(source_code, "fn main()")[0], file_id);
        let main_foo_call_location =
            Location::new(find_spans_for(source_code, "foo()")[0], file_id);
        let main_bar_call_location =
            Location::new(find_spans_for(source_code, "bar()")[0], file_id);
        let main_whatever_call_location =
            Location::new(find_spans_for(source_code, "whatever()")[0], file_id);
        let foo_baz_call_location = Location::new(find_spans_for(source_code, "baz()")[0], file_id);
        let bar_whatever_call_location =
            Location::new(find_spans_for(source_code, "whatever()")[1], file_id);
        let baz_whatever_call_location =
            Location::new(find_spans_for(source_code, "whatever()")[2], file_id);

        let mut opcode_locations = BTreeMap::<OpcodeLocation, Vec<Location>>::new();
        // main::foo::baz::whatever
        opcode_locations.insert(
            OpcodeLocation::Acir(0),
            vec![
                main_declaration_location,
                main_foo_call_location,
                foo_baz_call_location,
                baz_whatever_call_location,
            ],
        );

        // main::bar::whatever
        opcode_locations.insert(
            OpcodeLocation::Acir(1),
            vec![main_declaration_location, main_bar_call_location, bar_whatever_call_location],
        );
        // main::whatever
        opcode_locations.insert(
            OpcodeLocation::Acir(2),
            vec![main_declaration_location, main_whatever_call_location],
        );

        let debug_info = DebugInfo::new(
            opcode_locations,
            BTreeMap::default(),
            BTreeMap::default(),
            BTreeMap::default(),
            BTreeMap::default(),
            BTreeMap::default(),
        );

        let samples: Vec<CompilationSample> = vec![
            CompilationSample {
                opcode: Some(format_acir_opcode(&AcirOpcode::AssertZero::<FieldElement>(
                    Expression::default(),
                ))),
                call_stack: vec![OpcodeLocation::Acir(0)],
                count: 10,
                brillig_function_id: None,
            },
            CompilationSample {
                opcode: Some(format_acir_opcode(&AcirOpcode::AssertZero::<FieldElement>(
                    Expression::default(),
                ))),
                call_stack: vec![OpcodeLocation::Acir(1)],
                count: 20,
                brillig_function_id: None,
            },
            CompilationSample {
                opcode: Some(format_acir_opcode(&AcirOpcode::MemoryInit::<FieldElement> {
                    block_id: BlockId(0),
                    init: vec![],
                    block_type: acir::circuit::opcodes::BlockType::Memory,
                })),
                call_stack: vec![OpcodeLocation::Acir(2)],
                count: 30,
                brillig_function_id: None,
            },
        ];

        let expected_folded_sorted_lines = vec![
            "main.nr:2:9::fn main();main.nr:3:13::foo();main.nr:8:13::baz();main.nr:14:13::whatever();acir::arithmetic 10".to_string(),
            "main.nr:2:9::fn main();main.nr:4:13::bar();main.nr:11:13::whatever();acir::arithmetic 20".to_string(),
            "main.nr:2:9::fn main();main.nr:5:13::whatever();acir::memory::init 30".to_string(),
        ];

        let actual_folded_sorted_lines =
            generate_folded_sorted_lines(samples, &debug_info, fm.as_file_map());

        assert_eq!(expected_folded_sorted_lines, actual_folded_sorted_lines);
    }
}
