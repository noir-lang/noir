use std::{
    cmp::min,
    collections::HashMap,
    fs::{DirBuilder, File},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::AtomicU64,
};

use fm::{FileId, FileManager};
use noirc_abi::{
    input_parser::json::{parse_json, serialize_to_json},
    Abi, InputMap,
};
use proptest::bool;
use rand::prelude::*;
use rand_xorshift::XorShiftRng;
use sha256::digest;
use walkdir::WalkDir;

use std::sync::atomic::Ordering;
const CORPUS_FILE_EXTENSION: &str = "json";

static NEXT_TESTCASE_ID: AtomicU64 = AtomicU64::new(0xdead);

pub type TestCaseId = u64;
/// Generate unique sequential id for a testcase
fn generate_testcase_id() -> TestCaseId {
    NEXT_TESTCASE_ID.fetch_add(1, Ordering::SeqCst)
}

/// An input to the program and an id assigned to it
pub struct TestCase<'a> {
    value: &'a InputMap,
    id: TestCaseId,
}
impl<'a> TestCase<'a> {
    pub fn value(&self) -> &InputMap {
        self.value
    }
    pub fn id(&self) -> TestCaseId {
        self.id
    }
    pub fn with_id(id: TestCaseId, value: &'a InputMap) -> Self {
        Self { id, value }
    }
}
impl<'a> From<&'a InputMap> for TestCase<'a> {
    fn from(value: &'a InputMap) -> Self {
        Self { value, id: generate_testcase_id() }
    }
}
/// A mechanism for interacting with the corpus on the file system
pub struct CorpusFileManager {
    file_manager: FileManager,
    corpus_path: PathBuf,
    abi: Abi,
    parsed_map: HashMap<FileId, InputMap>,
}

impl CorpusFileManager {
    pub fn new(root: &Path, package_name: &str, harness_name: &str, abi: Abi) -> Self {
        let corpus_path = root.join(package_name).join(harness_name);

        Self { file_manager: FileManager::new(root), corpus_path, abi, parsed_map: HashMap::new() }
    }
    /// Loads the whole corpus from the given directory
    pub fn load_corpus_from_disk(&mut self) -> Result<(), String> {
        let mut builder = DirBuilder::new();
        match builder.recursive(true).create(&self.corpus_path) {
            Ok(_) => (),
            Err(_) => {
                return Err(format!(
                    "Couldn't create directory {:?}",
                    self.corpus_path.as_os_str()
                ));
            }
        }
        println!("Path of corpus {:?}", self.corpus_path);
        // Go through all files
        for entry in WalkDir::new(&self.corpus_path) {
            let Ok(entry) = entry else {
                continue;
            };
            if !entry.file_type().is_file() {
                continue;
            }
            if !entry.path().extension().map_or(false, |ext| ext == CORPUS_FILE_EXTENSION) {
                continue;
            };
            let path = entry.into_path();
            // If the file with the correct extension is already in the corpus, no need to load
            if self.file_manager.has_file(&path) {
                continue;
            }
            let source = std::fs::read_to_string(path.as_path())
                .unwrap_or_else(|_| panic!("could not read file {:?} into string", path));

            let parsed_source = parse_json(&source, &self.abi).map_err(|parsing_error| {
                format!("Error while parsing file {:?}: {:?}", path.as_os_str(), parsing_error)
            })?;

            // Add the file with source to the file manager
            let file_id = self.file_manager.add_file_with_source(path.as_path(), source).unwrap();
            self.parsed_map.insert(file_id, parsed_source);
        }
        Ok(())
    }

    /// Get the vector of all elements in the corpus
    pub fn get_full_corpus(&self) -> Vec<&InputMap> {
        let file_ids = self.file_manager.as_file_map().all_file_ids();
        let mut full_corpus = Vec::new();
        for file_id in file_ids {
            full_corpus.push(&self.parsed_map[file_id]);
        }
        full_corpus
    }

    /// Save testcase to the corpus directory and add it to the file manager
    pub fn save_testcase_to_disk(&mut self, contents: &str) -> Result<(), String> {
        let file_name = Path::new(&digest(contents)).with_extension(CORPUS_FILE_EXTENSION);
        let full_file_path = self.corpus_path.join(file_name);
        let mut file = File::create(&full_file_path)
            .map_err(|_x| format!("Couldn't create file {:?}", &full_file_path))?;

        file.write_all(contents.as_bytes())
            .map_err(|_x| format!("Couldn't write to file {:?}", &full_file_path))?;
        self.file_manager.add_file_with_source(&full_file_path, String::from(contents));
        Ok(())
    }
}

/// Sequence stores information on which testcase needs to be used for the following fuzzing iterations and how many more iterations it should be used in
#[derive(Debug)]
struct Sequence {
    testcase_id: TestCaseId,
    executions_left: u64,
}
impl Sequence {
    pub fn new() -> Self {
        Self { testcase_id: 0, executions_left: 0 }
    }
    pub fn is_empty(&self) -> bool {
        self.executions_left == 0
    }
    pub fn clear(&mut self) {
        self.executions_left = 0
    }
    pub fn decrement(&mut self) {
        self.executions_left -= 1
    }
}

/// A manager for selecting the next testcase to mutate
pub struct TestCaseOrchestrator {
    /// How many times each testcase has been used in fuzzing
    executions_per_testcase: HashMap<TestCaseId, u64>,
    /// The index of a sequence for each testcase  (how many times it has been selected for sequential fuzzing)
    sequence_number: HashMap<TestCaseId, u32>,
    /// How many times all the testcases have been executed
    total_executions: u64,
    /// Information about the currently prioritized testcase
    current_sequence: Sequence,
}

/// Index of the next testcase to be mutated for fuzzing and an optional additional one for splicing mutations
type NextSelection = (TestCaseId, Option<TestCaseId>);

impl TestCaseOrchestrator {
    pub fn new() -> Self {
        Self {
            executions_per_testcase: HashMap::new(),
            sequence_number: HashMap::new(),
            total_executions: 0,
            current_sequence: Sequence::new(),
        }
    }

    /// Add a new testcase for scheduling by the orchestrator
    pub fn new_testcase(&mut self, testcase_id: TestCaseId) {
        self.executions_per_testcase.insert(testcase_id, 0);
        self.sequence_number.insert(testcase_id, 0);
    }

    /// Remove a testcase (usually happens when it no longer represents any unique features)
    pub fn remove(&mut self, testcase_id: TestCaseId) {
        let executions = self.executions_per_testcase[&testcase_id];
        self.executions_per_testcase.remove(&testcase_id);
        self.sequence_number.remove(&testcase_id);
        self.total_executions -= executions;
        if self.current_sequence.testcase_id == testcase_id {
            self.current_sequence.clear();
        }
    }

    /// Chooses the next testcase according to the schedule prioritizing least fuzzed testcases in the corpus.
    /// If there is more than one testcase, additionally selects an extra for splicing mutations (randomly)
    pub fn get_next_testcase(&mut self, prng: &mut XorShiftRng) -> NextSelection {
        let testcase_count = self.executions_per_testcase.len();
        // If the sequence is already in place, just update counters
        if !self.current_sequence.is_empty() {
            // Update counts
            self.current_sequence.decrement();
            self.executions_per_testcase
                .entry(self.current_sequence.testcase_id)
                .and_modify(|executions| *executions += 1);
            self.total_executions += 1;
        } else {
            // Compute average
            let average = self.total_executions / testcase_count as u64;

            // Choose the lowest fuzzed testcase
            let chosen_id = self
                .executions_per_testcase
                .iter()
                .min_by_key(|&(_id, value)| value)
                .map(|(&id, _)| id)
                .unwrap();

            self.sequence_number
                .entry(chosen_id)
                .and_modify(|sequence_number| *sequence_number += 1);

            // Generate new sequence
            self.current_sequence = Sequence {
                testcase_id: chosen_id,
                executions_left: min(1u64 << self.sequence_number[&chosen_id], average / 2),
            };

            self.total_executions += 1;
            self.executions_per_testcase.entry(chosen_id).and_modify(|executions| *executions += 1);
        }

        // If there are several testcases, we can provide an additional one for splicing
        if testcase_count > 1 {
            let mut additional_id = self.current_sequence.testcase_id;
            // Sample until we get one different from the current
            while additional_id == self.current_sequence.testcase_id {
                additional_id =
                    self.executions_per_testcase.iter().map(|(&id, _)| id).choose(prng).unwrap();
            }
            return (self.current_sequence.testcase_id, Some(additional_id));
        } else {
            return (self.current_sequence.testcase_id, None);
        }
    }
}
/// Corpus contains all the information needed for selecting the next testcase for mutation
pub struct Corpus {
    /// Vector of all testcases currently being used in the fuzzing
    discovered_testcases: HashMap<TestCaseId, InputMap>,

    /// Information needed for selecting the next testcase for brillig execution
    brillig_orchestrator: TestCaseOrchestrator,

    /// Information needed for selecting the next testcase for acir execution
    acir_orchestrator: TestCaseOrchestrator,

    /// File manager
    corpus_file_manager: CorpusFileManager,
}

impl Corpus {
    /// Create a new object for tracking information about discovered testcases
    pub fn new(package_name: &str, function_name: &str, abi: &Abi) -> Self {
        Self {
            discovered_testcases: HashMap::new(),
            brillig_orchestrator: TestCaseOrchestrator::new(),
            acir_orchestrator: TestCaseOrchestrator::new(),
            corpus_file_manager: CorpusFileManager::new(
                Path::new("corpus"),
                package_name,
                function_name,
                abi.clone(),
            ),
        }
    }

    /// Check if there are already corpus files on disk
    pub fn attempt_to_load_corpus_from_disk(&mut self) -> Result<(), String> {
        self.corpus_file_manager.load_corpus_from_disk()
    }

    /// Get ALL the files that have been added to the corpus all the time (even deprecated ones)
    pub fn get_full_stored_corpus(&self) -> Vec<TestCase> {
        self.corpus_file_manager
            .get_full_corpus()
            .into_iter()
            .map(|value| TestCase::from(value))
            .collect()
    }

    /// Add a new file to the active corpus
    pub fn insert(
        &mut self,
        testcase_id: TestCaseId,
        new_testcase_value: InputMap,
        save_to_disk: bool,
    ) -> Result<TestCaseId, String> {
        if save_to_disk {
            self.corpus_file_manager.save_testcase_to_disk(
                &serialize_to_json(&new_testcase_value, &self.corpus_file_manager.abi)
                    .expect("Shouldn't be any issues with serializing input map"),
            )?
        }
        self.brillig_orchestrator.new_testcase(testcase_id);
        self.acir_orchestrator.new_testcase(testcase_id);
        self.discovered_testcases.insert(testcase_id, new_testcase_value.clone());
        Ok(testcase_id)
    }

    /// Remove a file from the active corpus
    pub fn remove(&mut self, testcase_id: TestCaseId) {
        self.brillig_orchestrator.remove(testcase_id);
        self.acir_orchestrator.remove(testcase_id);
        self.discovered_testcases.remove(&testcase_id);
    }

    /// Get the testcase body
    pub fn get_testcase_by_id(&self, id: TestCaseId) -> &InputMap {
        &self.discovered_testcases[&id]
    }

    /// Get an id of the testcase we should use next for acir testing
    pub fn get_next_testcase_for_acir(&mut self, prng: &mut XorShiftRng) -> NextSelection {
        self.acir_orchestrator.get_next_testcase(prng)
    }

    /// Get an id of the testcase we should use next for brillig testing
    pub fn get_next_testcase_for_brillig(&mut self, prng: &mut XorShiftRng) -> NextSelection {
        self.brillig_orchestrator.get_next_testcase(prng)
    }

    /// Get the number of active testcases in the corpus
    pub fn get_testcase_count(&self) -> usize {
        self.discovered_testcases.len()
    }
}
