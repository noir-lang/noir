use std::collections::BTreeMap;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Args;
use codespan_reporting::files::Files;
use color_eyre::eyre::{self, Context};
use inferno::flamegraph::{from_lines, Options};
use serde::{Deserialize, Serialize};

use acir::circuit::OpcodeLocation;
use nargo::errors::Location;
use noirc_artifacts::debug::DebugArtifact;
use noirc_artifacts::program::ProgramArtifact;
use noirc_errors::reporter::line_and_column_from_span;

#[derive(Debug, Clone, Args)]
pub(crate) struct GatesFlamegraphCommand {
    /// The path to the artifact JSON file
    #[clap(long, short)]
    artifact_path: String,

    /// Path to the noir backend binary
    #[clap(long, short)]
    backend_path: String,

    /// The output folder for the flamegraph svg files
    #[clap(long, short)]
    output: String,
}

trait GatesProvider {
    fn get_gates(&self, artifact_path: &Path) -> eyre::Result<BackendGatesResponse>;
}

struct BackendGatesProvider {
    backend_path: PathBuf,
}

impl GatesProvider for BackendGatesProvider {
    fn get_gates(&self, artifact_path: &Path) -> eyre::Result<BackendGatesResponse> {
        let backend_gates_response =
            Command::new(&self.backend_path).arg("gates").arg("-b").arg(artifact_path).output()?;

        // Parse the backend gates command stdout as json
        let backend_gates_response: BackendGatesResponse =
            serde_json::from_slice(&backend_gates_response.stdout)?;
        Ok(backend_gates_response)
    }
}

trait FlamegraphGenerator {
    fn generate_flamegraph<'lines, I: IntoIterator<Item = &'lines str>>(
        &self,
        folded_lines: I,
        artifact_name: &str,
        function_name: &str,
        output_path: &Path,
    ) -> eyre::Result<()>;
}

struct InfernoFlamegraphGenerator {}

impl FlamegraphGenerator for InfernoFlamegraphGenerator {
    fn generate_flamegraph<'lines, I: IntoIterator<Item = &'lines str>>(
        &self,
        folded_lines: I,
        artifact_name: &str,
        function_name: &str,
        output_path: &Path,
    ) -> eyre::Result<()> {
        let flamegraph_file = std::fs::File::create(output_path)?;
        let flamegraph_writer = BufWriter::new(flamegraph_file);

        let mut options = Options::default();
        options.hash = true;
        options.deterministic = true;
        options.title = format!("{}-{}", artifact_name, function_name);
        options.subtitle = Some("Sample = Gate".to_string());
        options.frame_height = 24;
        options.color_diffusion = true;

        from_lines(&mut options, folded_lines, flamegraph_writer)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackendGatesReport {
    acir_opcodes: usize,
    circuit_size: usize,
    gates_per_opcode: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackendGatesResponse {
    functions: Vec<BackendGatesReport>,
}

struct FoldedStackItem {
    total_gates: usize,
    nested_items: BTreeMap<String, FoldedStackItem>,
}

pub(crate) fn run(args: GatesFlamegraphCommand) -> eyre::Result<()> {
    run_with_provider(
        &PathBuf::from(args.artifact_path),
        &BackendGatesProvider { backend_path: PathBuf::from(args.backend_path) },
        &InfernoFlamegraphGenerator {},
        &PathBuf::from(args.output),
    )
}

fn run_with_provider<Provider: GatesProvider, Generator: FlamegraphGenerator>(
    artifact_path: &Path,
    gates_provider: &Provider,
    flamegraph_generator: &Generator,
    output_path: &Path,
) -> eyre::Result<()> {
    let program =
        read_program_from_file(artifact_path).context("Error reading program from file")?;

    let backend_gates_response =
        gates_provider.get_gates(artifact_path).context("Error querying backend for gates")?;

    let function_names = program.names.clone();

    let debug_artifact: DebugArtifact = program.into();

    for (func_idx, (func_gates, func_name)) in
        backend_gates_response.functions.into_iter().zip(function_names).enumerate()
    {
        println!(
            "Opcode count: {}, Total gates by opcodes: {}, Circuit size: {}",
            func_gates.acir_opcodes,
            func_gates.gates_per_opcode.iter().sum::<usize>(),
            func_gates.circuit_size
        );

        // Create a nested hashmap with the stack items, folding the gates for all the callsites that are equal
        let mut folded_stack_items = BTreeMap::new();

        func_gates.gates_per_opcode.into_iter().enumerate().for_each(|(opcode_index, gates)| {
            let call_stack = &debug_artifact.debug_symbols[func_idx]
                .locations
                .get(&OpcodeLocation::Acir(opcode_index));
            let location_names = if let Some(call_stack) = call_stack {
                call_stack
                    .iter()
                    .map(|location| location_to_callsite_label(*location, &debug_artifact))
                    .collect::<Vec<String>>()
            } else {
                vec!["unknown".to_string()]
            };

            add_locations_to_folded_stack_items(&mut folded_stack_items, location_names, gates);
        });
        let folded_lines = to_folded_sorted_lines(&folded_stack_items, Default::default());

        flamegraph_generator.generate_flamegraph(
            folded_lines.iter().map(|as_string| as_string.as_str()),
            artifact_path.to_str().unwrap(),
            &func_name,
            &Path::new(&output_path).join(Path::new(&format!("{}.svg", &func_name))),
        )?;
    }

    Ok(())
}

pub(crate) fn read_program_from_file<P: AsRef<Path>>(
    circuit_path: P,
) -> eyre::Result<ProgramArtifact> {
    let file_path = circuit_path.as_ref().with_extension("json");

    let input_string = std::fs::read(file_path)?;
    let program = serde_json::from_slice(&input_string)?;

    Ok(program)
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
    gates: usize,
) {
    let mut child_map = stack_items;
    for (index, location) in locations.iter().enumerate() {
        let current_item = child_map
            .entry(location.clone())
            .or_insert(FoldedStackItem { total_gates: 0, nested_items: BTreeMap::new() });

        child_map = &mut current_item.nested_items;

        if index == locations.len() - 1 {
            current_item.total_gates += gates;
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
    folded_stack_items
        .iter()
        .flat_map(move |(location, folded_stack_item)| {
            let frame_list: Vec<String> =
                parent_stacks.iter().cloned().chain(std::iter::once(location.clone())).collect();
            let line: String =
                format!("{} {}", frame_list.join(";"), folded_stack_item.total_gates);

            let mut new_parent_stacks = parent_stacks.clone();
            new_parent_stacks.push_back(location.clone());

            let child_lines: Vec<String> =
                to_folded_sorted_lines(&folded_stack_item.nested_items, new_parent_stacks);

            std::iter::once(line).chain(child_lines)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use acir::circuit::{OpcodeLocation, Program};
    use color_eyre::eyre::{self};
    use fm::{FileId, FileManager};
    use noirc_artifacts::program::ProgramArtifact;
    use noirc_driver::DebugFile;
    use noirc_errors::{
        debug_info::{DebugInfo, ProgramDebugInfo},
        Location, Span,
    };
    use std::{
        cell::RefCell,
        collections::{BTreeMap, HashMap},
        path::{Path, PathBuf},
    };
    use tempfile::TempDir;

    use super::{BackendGatesReport, BackendGatesResponse, GatesProvider};

    struct TestGateProvider {
        mock_responses: HashMap<PathBuf, BackendGatesResponse>,
    }

    impl GatesProvider for TestGateProvider {
        fn get_gates(&self, artifact_path: &std::path::Path) -> eyre::Result<BackendGatesResponse> {
            let response = self
                .mock_responses
                .get(artifact_path)
                .expect("should have a mock response for the artifact path");

            Ok(response.clone())
        }
    }

    #[derive(Default)]
    struct TestFlamegraphGenerator {
        lines_received: RefCell<Vec<Vec<String>>>,
    }

    impl super::FlamegraphGenerator for TestFlamegraphGenerator {
        fn generate_flamegraph<'lines, I: IntoIterator<Item = &'lines str>>(
            &self,
            folded_lines: I,
            _artifact_name: &str,
            _function_name: &str,
            _output_path: &std::path::Path,
        ) -> eyre::Result<()> {
            let lines = folded_lines.into_iter().map(|line| line.to_string()).collect();
            self.lines_received.borrow_mut().push(lines);
            Ok(())
        }
    }

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

    struct TestCase {
        expected_folded_sorted_lines: Vec<Vec<String>>,
        debug_symbols: ProgramDebugInfo,
        file_map: BTreeMap<FileId, DebugFile>,
        gates_report: BackendGatesResponse,
    }

    fn simple_test_case(temp_dir: &TempDir) -> TestCase {
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

        let file_map = BTreeMap::from_iter(vec![(
            file_id,
            DebugFile { source: source_code.to_string(), path: source_file_name.to_path_buf() },
        )]);

        let debug_symbols = ProgramDebugInfo {
            debug_infos: vec![DebugInfo::new(
                opcode_locations,
                BTreeMap::default(),
                BTreeMap::default(),
                BTreeMap::default(),
            )],
        };

        let backend_gates_response = BackendGatesResponse {
            functions: vec![BackendGatesReport {
                acir_opcodes: 3,
                circuit_size: 100,
                gates_per_opcode: vec![10, 20, 30],
            }],
        };

        let expected_folded_sorted_lines = vec![
            "main.nr:2:9::fn main() 0".to_string(),
            "main.nr:2:9::fn main();main.nr:3:13::foo() 0".to_string(),
            "main.nr:2:9::fn main();main.nr:3:13::foo();main.nr:8:13::baz() 0".to_string(),
            "main.nr:2:9::fn main();main.nr:3:13::foo();main.nr:8:13::baz();main.nr:14:13::whatever() 10".to_string(),
            "main.nr:2:9::fn main();main.nr:4:13::bar() 0".to_string(),
            "main.nr:2:9::fn main();main.nr:4:13::bar();main.nr:11:13::whatever() 20".to_string(),
            "main.nr:2:9::fn main();main.nr:5:13::whatever() 30".to_string(),
        ];

        TestCase {
            expected_folded_sorted_lines: vec![expected_folded_sorted_lines],
            debug_symbols,
            file_map,
            gates_report: backend_gates_response,
        }
    }

    #[test]
    fn test_flamegraph() {
        let temp_dir = tempfile::tempdir().unwrap();

        let test_cases = vec![simple_test_case(&temp_dir)];
        let artifact_names: Vec<_> =
            test_cases.iter().enumerate().map(|(idx, _)| format!("test{}.json", idx)).collect();

        let test_cases_with_names: Vec<_> = test_cases.into_iter().zip(artifact_names).collect();

        let mut mock_responses: HashMap<PathBuf, BackendGatesResponse> = HashMap::new();
        // Collect mock responses
        for (test_case, artifact_name) in test_cases_with_names.iter() {
            mock_responses.insert(
                temp_dir.path().join(artifact_name.clone()),
                test_case.gates_report.clone(),
            );
        }

        let provider = TestGateProvider { mock_responses };

        for (test_case, artifact_name) in test_cases_with_names.iter() {
            let artifact_path = temp_dir.path().join(artifact_name.clone());

            let artifact = ProgramArtifact {
                noir_version: "0.0.0".to_string(),
                hash: 27,
                abi: noirc_abi::Abi::default(),
                bytecode: Program::default(),
                debug_symbols: test_case.debug_symbols.clone(),
                file_map: test_case.file_map.clone(),
                names: vec!["main".to_string()],
            };

            // Write the artifact to a file
            let artifact_file = std::fs::File::create(&artifact_path).unwrap();
            serde_json::to_writer(artifact_file, &artifact).unwrap();

            let flamegraph_generator = TestFlamegraphGenerator::default();

            super::run_with_provider(
                &artifact_path,
                &provider,
                &flamegraph_generator,
                temp_dir.path(),
            )
            .expect("should run without errors");

            // Check that the flamegraph generator was called with the correct folded sorted lines
            let calls_received = flamegraph_generator.lines_received.borrow().clone();

            assert_eq!(calls_received, test_case.expected_folded_sorted_lines);
        }
    }
}
