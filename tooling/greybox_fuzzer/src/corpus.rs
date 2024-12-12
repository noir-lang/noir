use std::{
    cmp::min,
    collections::HashMap,
    fs::{DirBuilder, File},
    io::Write,
    path::{Path, PathBuf},
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

const CORPUS_FILE_EXTENSION: &str = "json";
pub struct CorpusFileManager<'a> {
    file_manager: FileManager,
    root: PathBuf,
    corpus_path: PathBuf,
    abi: &'a Abi,
    parsed_map: HashMap<FileId, InputMap>,
}

impl<'a> CorpusFileManager<'a> {
    pub fn new(root: &Path, package_name: &str, harness_name: &str, abi: &'a Abi) -> Self {
        let cloned_root = root.clone();
        let corpus_path = root.join(package_name).join(harness_name);

        Self {
            file_manager: FileManager::new(root),
            root: cloned_root.to_path_buf(),
            corpus_path,
            abi,
            parsed_map: HashMap::new(),
        }
    }
    pub fn load_corpus(&mut self) -> Result<(), String> {
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
            if self.file_manager.has_file(&path) {
                continue;
            }
            let source = std::fs::read_to_string(path.as_path())
                .unwrap_or_else(|_| panic!("could not read file {:?} into string", path));

            let parsed_source = parse_json(&source, self.abi).map_err(|parsing_error| {
                format!("Error while parsing file {:?}: {:?}", path.as_os_str(), parsing_error)
            })?;

            let file_id = self.file_manager.add_file_with_source(path.as_path(), source).unwrap();
            self.parsed_map.insert(file_id, parsed_source);
        }
        Ok(())
    }
    pub fn get_corpus(&self) -> Vec<InputMap> {
        let file_ids = self.file_manager.as_file_map().all_file_ids();
        let mut full_corpus = Vec::new();
        for file_id in file_ids {
            full_corpus.push(self.parsed_map[file_id].clone());
        }
        full_corpus
    }
    pub fn insert_into_corpus(&mut self, contents: &str) -> Result<(), String> {
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
#[derive(Debug)]
struct Sequence {
    testcase_index: usize,
    executions_left: u64,
}
impl Sequence {
    pub fn new() -> Self {
        Self { testcase_index: 0, executions_left: 0 }
    }
    pub fn is_empty(&self) -> bool {
        self.executions_left == 0
    }
    pub fn decrement(&mut self) {
        self.executions_left -= 1
    }
}
pub struct Corpus<'a> {
    discovered_testcases: Vec<InputMap>,
    executions_per_testcase: Vec<u64>,
    sequence_number: Vec<u32>,
    total_executions: u64,
    current_sequence: Sequence,
    corpus_file_manager: CorpusFileManager<'a>,
}

impl<'a> Corpus<'a> {
    const MAX_EXECUTIONS_PER_SEQUENCE_LOG: u32 = 100;
    pub fn new(package_name: &str, function_name: &str, abi: &'a Abi) -> Self {
        // Self {
        //     discovered_testcases: vec![starting_testcase],
        //     executions_per_testcase: vec![1],
        //     sequence_number: vec![0],
        //     total_executions: 1,
        //     current_sequence: Sequence::new(),
        // }
        Self {
            discovered_testcases: Vec::new(),
            executions_per_testcase: Vec::new(),
            sequence_number: Vec::new(),
            total_executions: 0,
            current_sequence: Sequence::new(),
            corpus_file_manager: CorpusFileManager::new(
                Path::new("corpus"),
                package_name,
                function_name,
                abi,
            ),
        }
    }
    pub fn attempt_load(&mut self) -> Result<(), String> {
        self.corpus_file_manager.load_corpus()
    }
    pub fn get_stored_corpus(&self) -> Vec<InputMap> {
        self.corpus_file_manager.get_corpus()
    }
    pub fn insert(&mut self, new_testcase: InputMap, save_to_disk: bool) -> Result<(), String> {
        if save_to_disk {
            self.corpus_file_manager.insert_into_corpus(
                &serialize_to_json(&new_testcase, &self.corpus_file_manager.abi)
                    .expect("Shouldn't be any issues with serializing input map"),
            )?
        }
        self.executions_per_testcase.push(0);
        self.sequence_number.push(0);
        self.discovered_testcases.push(new_testcase);
        Ok(())
    }
    pub fn get_next_testcase_with_additional(
        &mut self,
        prng: &mut XorShiftRng,
    ) -> (&InputMap, Option<&InputMap>) {
        if !self.current_sequence.is_empty() {
            // Update counts
            self.current_sequence.decrement();
            self.executions_per_testcase[self.current_sequence.testcase_index] += 1;
            self.total_executions += 1;
        } else {
            // Compute average
            let average = self.total_executions / self.discovered_testcases.len() as u64;
            // Omit those that have been fuzzed more than average
            let weakly_fuzzed_group: Vec<_> = (0..(self.discovered_testcases.len()))
                .filter(|&index| self.executions_per_testcase[index] <= average)
                .collect();
            let chosen_index = (0..(self.discovered_testcases.len()))
                .rev()
                .min_by(|&i, &j| {
                    self.executions_per_testcase[i].cmp(&self.executions_per_testcase[j])
                })
                .unwrap();
            self.sequence_number[chosen_index] += 1;
            self.current_sequence = Sequence {
                testcase_index: chosen_index,
                executions_left: min(
                    1u64 << min(
                        Self::MAX_EXECUTIONS_PER_SEQUENCE_LOG,
                        self.sequence_number[chosen_index],
                    ),
                    average / 2,
                ),
            };
            self.total_executions += 1;
            self.executions_per_testcase[chosen_index] += 1;
            // println!(
            //     "Starting sequence {} on input {:?}",
            //     self.current_sequence.executions_left,
            //     self.discovered_testcases[self.current_sequence.testcase_index]
            // );
        }
        if self.discovered_testcases.len() > 1 {
            let mut additional_index = prng.gen_range(0..(self.discovered_testcases.len() - 1));
            if additional_index >= self.current_sequence.testcase_index {
                additional_index += 1;
            }
            return (
                &self.discovered_testcases[self.current_sequence.testcase_index],
                Some(&self.discovered_testcases[additional_index]),
            );
        } else {
            return (&self.discovered_testcases[self.current_sequence.testcase_index], None);
        }
    }
}
