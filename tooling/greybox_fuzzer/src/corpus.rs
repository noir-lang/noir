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

// File extension used for corpus files
const CORPUS_FILE_EXTENSION: &str = "json";

// Default folder name where corpus files are stored
pub const DEFAULT_CORPUS_FOLDER: &str = "corpus";

// Global atomic counter used to generate unique sequential IDs for testcases
static NEXT_TESTCASE_ID: AtomicU64 = AtomicU64::new(0xdead);

// Type alias for testcase IDs which are 64-bit integers
pub type TestCaseId = u64;

/// Generates a unique sequential ID for a testcase by atomically incrementing a global counter
fn generate_testcase_id() -> TestCaseId {
    NEXT_TESTCASE_ID.fetch_add(1, Ordering::SeqCst)
}

/// Represents a single test case in the corpus, containing both the input values and a unique identifier
pub struct TestCase<'a> {
    // The actual input values for this test case
    value: &'a InputMap,
    // Unique identifier for this test case
    id: TestCaseId,
}

impl<'a> TestCase<'a> {
    /// Returns a reference to the input values for this test case
    pub fn value(&self) -> &InputMap {
        self.value
    }

    /// Returns the unique identifier for this test case
    pub fn id(&self) -> TestCaseId {
        self.id
    }

    /// Creates a new TestCase with a specified ID and input values
    pub fn with_id(id: TestCaseId, value: &'a InputMap) -> Self {
        Self { id, value }
    }
}

impl<'a> From<&'a InputMap> for TestCase<'a> {
    /// Converts an InputMap into a TestCase by generating a new unique ID
    fn from(value: &'a InputMap) -> Self {
        Self { value, id: generate_testcase_id() }
    }
}

/// Manages reading and writing corpus files to the filesystem, including parsing JSON inputs
pub struct CorpusFileManager {
    // Manages file operations and tracks file metadata
    file_manager: FileManager,
    // Path to the directory containing corpus files
    corpus_path: PathBuf,
    // ABI definition used for parsing inputs
    abi: Abi,
    // Maps file IDs to their parsed input values
    parsed_map: HashMap<FileId, InputMap>,
}

impl CorpusFileManager {
    /// Creates a new CorpusFileManager for a specific package and test harness
    pub fn new(root: &Path, package_name: &str, harness_name: &str, abi: Abi) -> Self {
        let corpus_path = root.join(package_name).join(harness_name);

        Self { file_manager: FileManager::new(root), corpus_path, abi, parsed_map: HashMap::new() }
    }

    /// Loads all corpus files from disk into memory, parsing them as JSON inputs.
    /// Creates the corpus directory if it doesn't exist.
    /// Returns an error if directory creation fails or if any files can't be parsed.
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
        // Go through all files in the corpus directory recursively
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
            // Skip files that are already loaded
            if self.file_manager.has_file(&path) {
                continue;
            }
            let source = std::fs::read_to_string(path.as_path())
                .unwrap_or_else(|_| panic!("could not read file {:?} into string", path));

            let parsed_source = parse_json(&source, &self.abi).map_err(|parsing_error| {
                format!("Error while parsing file {:?}: {:?}", path.as_os_str(), parsing_error)
            })?;

            // Add the file and its parsed contents to our tracking maps
            let file_id = self.file_manager.add_file_with_source(path.as_path(), source).unwrap();
            self.parsed_map.insert(file_id, parsed_source);
        }
        Ok(())
    }

    /// Returns a vector containing all parsed inputs from the corpus
    pub fn get_full_corpus(&self) -> Vec<&InputMap> {
        let file_ids = self.file_manager.as_file_map().all_file_ids();
        let mut full_corpus = Vec::new();
        for file_id in file_ids {
            full_corpus.push(&self.parsed_map[file_id]);
        }
        full_corpus
    }

    /// Returns the path to the corpus directory
    pub fn get_corpus_path(&self) -> &Path {
        &self.corpus_path
    }

    /// Saves a new testcase to disk and adds it to the file manager.
    /// The filename is generated from the SHA-256 hash of the contents.
    /// Returns an error if file creation or writing fails.
    pub fn save_testcase_to_disk(&mut self, contents: &str) -> Result<(), String> {
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

/// Tracks information about a sequence of executions for a particular testcase,
/// including how many executions remain and which testcase is being used
#[derive(Debug)]
struct Sequence {
    // ID of the testcase being executed in sequence
    testcase_id: TestCaseId,
    // Number of remaining executions in this sequence
    executions_left: u64,
}

impl Sequence {
    /// Creates a new empty sequence
    pub fn new() -> Self {
        Self { testcase_id: 0, executions_left: 0 }
    }

    /// Returns true if there are no executions remaining in this sequence
    pub fn is_empty(&self) -> bool {
        self.executions_left == 0
    }

    /// Resets the sequence by setting remaining executions to 0
    pub fn clear(&mut self) {
        self.executions_left = 0
    }

    /// Decrements the number of remaining executions by 1
    pub fn decrement(&mut self) {
        self.executions_left -= 1
    }
}

/// Manages the selection of testcases for fuzzing, tracking execution counts and scheduling
pub struct TestCaseOrchestrator {
    // Maps testcase IDs to their total execution count
    executions_per_testcase: HashMap<TestCaseId, u64>,
    // Maps testcase IDs to their sequence number (how many times they've been selected for sequential fuzzing)
    sequence_number: HashMap<TestCaseId, u32>,
    // Total number of executions across all testcases
    total_executions: u64,
    // Currently active sequence of executions
    current_sequence: Sequence,
}

/// Represents the next testcase(s) selected for fuzzing:
/// - First element is the primary testcase ID
/// - Second optional element is an additional testcase ID for splicing mutations
type NextSelection = (TestCaseId, Option<TestCaseId>);

impl TestCaseOrchestrator {
    /// Creates a new empty TestCaseOrchestrator
    pub fn new() -> Self {
        Self {
            executions_per_testcase: HashMap::new(),
            sequence_number: HashMap::new(),
            total_executions: 0,
            current_sequence: Sequence::new(),
        }
    }

    /// Adds a new testcase to be managed by the orchestrator
    pub fn new_testcase(&mut self, testcase_id: TestCaseId) {
        self.executions_per_testcase.insert(testcase_id, 0);
        self.sequence_number.insert(testcase_id, 0);
    }

    /// Removes a testcase from being managed by the orchestrator.
    /// Updates execution counts and clears current sequence if it was using this testcase.
    pub fn remove(&mut self, testcase_id: TestCaseId) {
        let executions = self.executions_per_testcase[&testcase_id];
        self.executions_per_testcase.remove(&testcase_id);
        self.sequence_number.remove(&testcase_id);
        self.total_executions -= executions;
        if self.current_sequence.testcase_id == testcase_id {
            self.current_sequence.clear();
        }
    }

    /// Selects the next testcase(s) to use for fuzzing based on execution counts.
    /// Prioritizes testcases that have been executed least often.
    /// Returns both a primary testcase and optionally a second one for splicing.
    pub fn get_next_testcase(&mut self, prng: &mut XorShiftRng) -> NextSelection {
        let testcase_count = self.executions_per_testcase.len();
        // If we're in the middle of a sequence, continue using the current testcase
        if !self.current_sequence.is_empty() {
            // Update execution counts
            self.current_sequence.decrement();
            self.executions_per_testcase
                .entry(self.current_sequence.testcase_id)
                .and_modify(|executions| *executions += 1);
            self.total_executions += 1;
        } else {
            // Calculate average executions per testcase
            let average = self.total_executions / testcase_count as u64;

            // Select the testcase with the lowest execution count
            let chosen_id = self
                .executions_per_testcase
                .iter()
                .min_by_key(|&(_id, value)| value)
                .map(|(&id, _)| id)
                .unwrap();

            // Increment sequence number for chosen testcase
            self.sequence_number
                .entry(chosen_id)
                .and_modify(|sequence_number| *sequence_number += 1);

            // Create new sequence with exponentially increasing length based on sequence number
            self.current_sequence = Sequence {
                testcase_id: chosen_id,
                executions_left: min(1u64 << self.sequence_number[&chosen_id], average / 2),
            };

            // Update execution counts
            self.total_executions += 1;
            self.executions_per_testcase.entry(chosen_id).and_modify(|executions| *executions += 1);
        }

        // If we have multiple testcases, randomly select a different one for splicing
        if testcase_count > 1 {
            let mut additional_id = self.current_sequence.testcase_id;
            // Keep sampling until we get a different testcase
            while additional_id == self.current_sequence.testcase_id {
                additional_id =
                    self.executions_per_testcase.iter().map(|(&id, _)| id).choose(prng).unwrap();
            }
            (self.current_sequence.testcase_id, Some(additional_id))
        } else {
            (self.current_sequence.testcase_id, None)
        }
    }
}

/// Main corpus manager that tracks all discovered testcases and orchestrates their selection for fuzzing
pub struct Corpus {
    // Currently active testcases used for fuzzing
    discovered_testcases: HashMap<TestCaseId, InputMap>,

    // Testcases that are stored but not actively used in fuzzing
    cached_testcases: HashMap<TestCaseId, InputMap>,

    // Orchestrator for selecting testcases for Brillig execution
    brillig_orchestrator: TestCaseOrchestrator,

    // Orchestrator for selecting testcases for ACIR execution
    acir_orchestrator: TestCaseOrchestrator,

    // Manages reading/writing corpus files to disk
    corpus_file_manager: CorpusFileManager,
}

impl Corpus {
    /// Creates a new empty corpus for a specific package and function
    pub fn new(base_folder: &Path, package_name: &str, function_name: &str, abi: &Abi) -> Self {
        Self {
            discovered_testcases: HashMap::new(),
            cached_testcases: HashMap::new(),
            brillig_orchestrator: TestCaseOrchestrator::new(),
            acir_orchestrator: TestCaseOrchestrator::new(),
            corpus_file_manager: CorpusFileManager::new(
                base_folder,
                package_name,
                function_name,
                abi.clone(),
            ),
        }
    }

    /// Attempts to load existing corpus files from disk
    pub fn attempt_to_load_corpus_from_disk(&mut self) -> Result<(), String> {
        self.corpus_file_manager.load_corpus_from_disk()
    }

    /// Returns all testcases that have ever been added to the corpus, including cached ones
    pub fn get_full_stored_corpus(&mut self) -> Vec<TestCase> {
        let stored_corpus: Vec<_> =
            self.corpus_file_manager.get_full_corpus().into_iter().map(TestCase::from).collect();
        let id_testcase_pair: Vec<_> =
            stored_corpus.iter().map(|x| (x.id(), x.value().clone())).collect();
        for (id, input_map) in id_testcase_pair.iter() {
            self.insert_into_cache(*id, input_map.clone());
        }
        id_testcase_pair
            .iter()
            .map(|(id, _)| TestCase::with_id(*id, &self.cached_testcases[&id]))
            .collect()
    }

    /// Returns the path where corpus files are stored
    pub fn get_corpus_storage_path(&self) -> &Path {
        self.corpus_file_manager.get_corpus_path()
    }

    /// Adds a testcase to the cache (stored but not active)
    pub fn insert_into_cache(&mut self, testcase_id: TestCaseId, new_testcase_value: InputMap) {
        self.cached_testcases.insert(testcase_id, new_testcase_value);
    }

    /// Adds a new testcase to the active corpus.
    /// Optionally saves it to disk and updates orchestrators.
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

    /// Removes a testcase from the active corpus and both orchestrators
    pub fn remove(&mut self, testcase_id: TestCaseId) {
        self.brillig_orchestrator.remove(testcase_id);
        self.acir_orchestrator.remove(testcase_id);
        self.discovered_testcases.remove(&testcase_id);
    }

    /// Returns the input values for a testcase by its ID, checking both active and cached testcases
    pub fn get_testcase_by_id(&self, id: TestCaseId) -> &InputMap {
        if self.discovered_testcases.contains_key(&id) {
            &self.discovered_testcases[&id]
        } else {
            &self.cached_testcases[&id]
        }
    }

    /// Selects the next testcase(s) for ACIR execution
    pub fn get_next_testcase_for_acir(&mut self, prng: &mut XorShiftRng) -> NextSelection {
        self.acir_orchestrator.get_next_testcase(prng)
    }

    /// Selects the next testcase(s) for Brillig execution
    pub fn get_next_testcase_for_brillig(&mut self, prng: &mut XorShiftRng) -> NextSelection {
        self.brillig_orchestrator.get_next_testcase(prng)
    }

    /// Returns the number of testcases currently active in the corpus
    pub fn get_testcase_count(&self) -> usize {
        self.discovered_testcases.len()
    }

    /// Returns a vector of all currently active testcases
    pub fn get_current_discovered_testcases(&self) -> Vec<TestCase> {
        self.discovered_testcases.iter().map(|(&id, value)| TestCase::with_id(id, value)).collect()
    }
}
