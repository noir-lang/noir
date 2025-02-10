use core::panic;
use std::{
    cmp::max,
    path::{Path, PathBuf},
    time::Instant,
};

use acvm::{
    acir::{
        circuit::Program,
        native_types::{WitnessMap, WitnessStack},
    },
    FieldElement,
};
use coverage::{
    analyze_brillig_program_before_fuzzing, AccumulatedFuzzerCoverage, BrilligCoverageRanges,
    FeatureToIndexMap, PotentialBoolWitnessList, RawBrilligCoverage, SingleTestCaseCoverage,
};
use noir_fuzzer::dictionary::build_dictionary_from_program;

mod corpus;
mod coverage;
mod mutation;
mod types;

use corpus::{Corpus, TestCase, TestCaseId, DEFAULT_CORPUS_FOLDER};
use mutation::InputMutator;
use rayon::iter::ParallelIterator;
use termcolor::{ColorChoice, StandardStream};
pub use types::FuzzTestResult;
use types::{
    CounterExampleOutcome, DiscrepancyOutcome, HarnessExecutionOutcome, ProgramFailureResult,
    SuccessfulCaseOutcome,
};

use noirc_artifacts::program::ProgramArtifact;
use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

use rayon::prelude::IntoParallelIterator;
use std::io::Write;
use termcolor::{Color, ColorSpec, WriteColor};

const FOREIGN_CALL_FAILURE_SUBSTRING: &str = "Failed calling external resolver.";

/// We aim the number of testcases per round so one round takes these many microseconds
const SINGLE_FUZZING_ROUND_TARGET_TIME: u128 = 500_000u128;

/// A seed for the XorShift RNG for use during mutation
type SimpleXorShiftRNGSeed = <XorShiftRng as SeedableRng>::Seed;

/// Information collected from testcase execution on success
pub type WitnessAndCoverage = (WitnessStack<FieldElement>, Option<Vec<u32>>);

/// Information collected from testcase execution on failure
pub type ErrorAndCoverage = (String, Option<Vec<u32>>);

/// A structure with the values for a single mutation-fuzz iteration in the fuzzer
struct FuzzTask {
    /// The id of the main testcase that is going to be mutated
    main_testcase_id: TestCaseId,
    /// An optional id of a second testcase that will be used for splicing
    additional_testcase_id: Option<TestCaseId>,
    /// A seed for the PRNG that will be used for mutating/splicing
    seed: SimpleXorShiftRNGSeed,
}

impl FuzzTask {
    /// Create a new FuzzTask where everything is given
    pub fn new(
        main_testcase_id: TestCaseId,
        additional_testcase_id: Option<TestCaseId>,
        seed: SimpleXorShiftRNGSeed,
    ) -> Self {
        Self { main_testcase_id, additional_testcase_id, seed }
    }

    /// Create a task for executing a testcase without mutation
    pub fn mutationless(main_testcase_id: TestCaseId) -> Self {
        Self {
            main_testcase_id,
            additional_testcase_id: None,
            seed: SimpleXorShiftRNGSeed::default(),
        }
    }

    pub fn prng_seed(&self) -> SimpleXorShiftRNGSeed {
        self.seed
    }
    pub fn main_id(&self) -> TestCaseId {
        self.main_testcase_id
    }
    pub fn additional_id(&self) -> Option<TestCaseId> {
        self.additional_testcase_id
    }
}

/// Contains information from parallel execution of testcases for quick single-threaded processing
/// If no new coverage is detected, the fuzzer can simply quickly update the timing metrics without parsing the outcome
#[derive(Debug)]
struct FastParallelFuzzResult {
    /// Contains the result of executing the testcase and the testcase itself
    outcome: HarnessExecutionOutcome,
    /// If new coverage has been detected when executing the testcase
    new_coverage_detected: bool,
    /// If the fuzzer has detected a condition that will not allow it to continue (a discrepancy, an unexpected execution failure or a failed foreign call)
    failure_detected: bool,
    /// How much time mutating the testcase took before execution (microseconds)
    mutation_time: u128,
    /// How much time executing the ACIR version took (microseconds). Zero, if there was only brillig execution
    acir_time: u128,
    /// How much time executing the brillig version took (microseconds)
    brillig_time: u128,
}

impl FastParallelFuzzResult {
    pub fn new(
        outcome: HarnessExecutionOutcome,
        new_coverage_detected: bool,
        failure_detected: bool,
        mutation_time: u128,
        acir_time: u128,
        brillig_time: u128,
    ) -> Self {
        Self {
            outcome,
            new_coverage_detected,
            failure_detected,
            mutation_time,
            acir_time,
            brillig_time,
        }
    }

    /// True if there is no need to perform the merge check
    pub fn skip_check(&self) -> bool {
        !self.new_coverage_detected
    }

    /// Executing the testcase resulted in failure
    pub fn failed(&self) -> bool {
        self.failure_detected
    }

    /// Get the outcome of the execution
    pub fn outcome(&self) -> &HarnessExecutionOutcome {
        &self.outcome
    }

    /// Get mutation time
    pub fn mutation_time(&self) -> u128 {
        self.mutation_time
    }
    /// Get acir execution time
    pub fn acir_time(&self) -> u128 {
        self.acir_time
    }
    /// Get brillig execution time
    pub fn brillig_time(&self) -> u128 {
        self.brillig_time
    }
}
/// The metrics of the fuzzing process being output to the user
#[derive(Default)]
struct Metrics {
    /// Total time spent executing ACIR programs in microseconds
    total_acir_execution_time: u128,
    /// Total time spent executing Brillig programs in microseconds
    total_brillig_execution_time: u128,
    /// Total time spent mutating testcases
    total_mutation_time: u128,
    /// The number of unique testcases run
    processed_testcase_count: usize,
    /// Number of testcases removed from the corpus
    removed_testcase_count: usize,
    /// Purged a testcase last round
    removed_testcase_last_round: bool,
    /// The size of the corpus being used in mutation schedule
    active_corpus_size: usize,
    /// Last round size
    last_round_size: usize,
    /// Last round execution time
    last_round_execution_time: u128,
    /// Last round accumulated coverage update time
    last_round_update_time: u128,
    /// Number of threads involved in fuzzing
    num_threads: usize,
    /// Number of testcases discovered with ACIR/Brillig tandem
    acir_brillig_discoveries: usize,
    /// Discovered something with ACIR/Brillig tandem last round
    found_new_with_acir_brillig: bool,
    /// Number of testcases discovered with Brillig
    brillig_discoveries: usize,
    /// Discovered something with Brillig last round
    found_new_with_brillig: bool,
}

impl Metrics {
    pub fn increase_total_acir_time(&mut self, update: &u128) {
        self.total_acir_execution_time += update;
    }
    pub fn increase_total_brillig_time(&mut self, update: &u128) {
        self.total_brillig_execution_time += update;
    }
    pub fn increase_total_mutation_time(&mut self, update: &u128) {
        self.total_mutation_time += update;
    }
    /// Tells if more time has been spent in brillig execution than in ACIR execution
    pub fn is_brillig_dominating(&self) -> bool {
        self.total_brillig_execution_time > self.total_acir_execution_time
    }

    pub fn increase_processed_testcase_count(&mut self, update: &usize) {
        self.processed_testcase_count += update;
    }
    pub fn increment_removed_testcase_count(&mut self) {
        self.removed_testcase_count += 1;
        self.removed_testcase_last_round = true;
    }
    pub fn set_active_corpus_size(&mut self, new_size: usize) {
        self.active_corpus_size = new_size;
    }
    pub fn set_last_round_size(&mut self, new_size: usize) {
        self.last_round_size = new_size;
    }
    pub fn set_last_round_execution_time(&mut self, new_time: u128) {
        self.last_round_execution_time = new_time;
    }
    pub fn set_last_round_update_time(&mut self, new_time: u128) {
        self.last_round_update_time = new_time;
    }
    pub fn set_num_threads(&mut self, count: usize) {
        self.num_threads = count;
    }

    pub fn increment_acir_brillig_discoveries(&mut self) {
        self.acir_brillig_discoveries += 1;
        self.found_new_with_acir_brillig = true;
    }
    pub fn increment_brillig_discoveries(&mut self) {
        self.brillig_discoveries += 1;
        self.found_new_with_brillig = true;
    }
    pub fn refresh_round(&mut self) {
        self.found_new_with_acir_brillig = false;
        self.found_new_with_brillig = false;
        self.removed_testcase_last_round = false;
    }
}

pub struct FuzzedExecutorFailureConfiguration {
    /// Fail on specific asserts
    pub fail_on_specific_asserts: bool,

    /// Failure reason
    pub failure_reason: Option<String>,
}

pub struct FuzzedExecutorFolderConfiguration {
    /// Corpus folder. If given, the corpus is stored here
    pub corpus_folder: Option<String>,
    /// Minimized corpus folder. If given, fuzzed executor performs minimization of the corpus instead of fuzzing and tries to save the results into this folder
    pub minimized_corpus_folder: Option<String>,
}
/// An executor for Noir programs which which provides fuzzing support
///
/// After instantiation, calling `fuzz` will proceed to hammer the program with
/// inputs, until it finds a counterexample.

pub struct FuzzedExecutor<E, F> {
    /// The program to be fuzzed (acir version)
    acir_program: ProgramArtifact,

    /// The program to be fuzzed (brillig version)
    brillig_program: ProgramArtifact,

    /// A function which executes the programs with a given set of inputs
    acir_executor: E,

    /// A function which executes the programs with a given set of inputs
    brillig_executor: F,

    /// Location to feature map (used in brillig fuzzing)
    location_to_feature_map: FeatureToIndexMap,

    /// Brillig coverage ranges (which are branch coverage and which are comparison coverage)
    brillig_coverage_ranges: BrilligCoverageRanges,

    /// The object generating mutated version of testcases in the corpus
    mutator: InputMutator,

    /// The name of the package being fuzzed
    package_name: String,

    /// The name of the function being fuzzed
    function_name: String,

    /// Number of threads to use
    num_threads: usize,

    /// Determines what is considered a failure during execution
    failure_configuration: FuzzedExecutorFailureConfiguration,

    /// Corpus folder
    corpus_folder: PathBuf,

    /// If this is set, perform minimization of the corpus
    minimize_corpus: bool,

    /// Corpus with the minimized
    minimized_corpus_folder: PathBuf,

    /// Execution metric
    metrics: Metrics,
}
pub struct AcirAndBrilligPrograms {
    pub acir_program: ProgramArtifact,
    pub brillig_program: ProgramArtifact,
}
impl<
        E: Fn(
                &Program<FieldElement>,
                WitnessMap<FieldElement>,
            ) -> Result<WitnessStack<FieldElement>, String>
            + Sync,
        F: Fn(
                &Program<FieldElement>,
                WitnessMap<FieldElement>,
                &FeatureToIndexMap,
            ) -> Result<WitnessAndCoverage, ErrorAndCoverage>
            + Sync,
    > FuzzedExecutor<E, F>
{
    /// Instantiates a fuzzed executor given an executor
    pub fn new(
        acir_and_brillig_programs: AcirAndBrilligPrograms,
        acir_executor: E,
        brillig_executor: F,
        package_name: &str,
        function_name: &str,
        num_threads: usize,
        failure_configuration: FuzzedExecutorFailureConfiguration,
        folder_configuration: FuzzedExecutorFolderConfiguration,
    ) -> Self {
        // Analyze brillig program for branches and comparisons
        let (location_to_feature_map, brillig_coverage_ranges) =
            analyze_brillig_program_before_fuzzing(&acir_and_brillig_programs.brillig_program);

        // Create a dictionary from acir bytecode
        // TODO: replace with building a dictionary from brillig. It makes much more sense
        let dictionary =
            build_dictionary_from_program(&acir_and_brillig_programs.acir_program.bytecode);

        // Create a mutator for the following interface with the dictionary generated from acir bytecode
        let mutator = InputMutator::new(&acir_and_brillig_programs.acir_program.abi, &dictionary);

        Self {
            acir_program: acir_and_brillig_programs.acir_program,
            brillig_program: acir_and_brillig_programs.brillig_program,
            acir_executor,
            brillig_executor,
            location_to_feature_map,
            brillig_coverage_ranges,
            mutator,
            package_name: package_name.to_string(),
            function_name: function_name.to_string(),
            num_threads,
            failure_configuration,
            corpus_folder: PathBuf::from(
                folder_configuration.corpus_folder.unwrap_or(DEFAULT_CORPUS_FOLDER.to_string()),
            ),
            minimize_corpus: folder_configuration.minimized_corpus_folder.is_some(),
            minimized_corpus_folder: PathBuf::from(
                folder_configuration.minimized_corpus_folder.unwrap_or_default(),
            ),

            metrics: Metrics::default(),
        }
    }

    /// Given the witness from ACIR execution and coverage from Brillig execution, check if they have any new coverage
    /// ACIR witness is optional, since we can skip ACIR execution or it could have failed, but we collected coverage from brillig
    /// We never expect brillig to have no coverage (at least for now)
    fn detect_new_coverage_from_witness_and_brillig(
        accumulated_coverage: &AccumulatedFuzzerCoverage,
        witness: &Option<WitnessStack<FieldElement>>,
        brillig_coverage: &Option<RawBrilligCoverage>,
    ) -> bool {
        // Get a list of boolean witnesses (default or taken from accumulated coverage)
        let mut default_list = PotentialBoolWitnessList::default();
        let mut bool_witness_list =
            accumulated_coverage.potential_bool_witness_list.as_ref().unwrap_or(&default_list);

        // If ACVM solved the witness, collect boolean states
        if witness.is_some() {
            default_list = accumulated_coverage
                .potential_bool_witness_list
                .as_ref()
                .unwrap_or(&default_list)
                .merge_new(witness.as_ref().unwrap());
            bool_witness_list = &default_list;
        }

        // Form a coverage object with coverage from this run
        // We don't care about the testcase id, since we are not merging this, just detecting if it has new coverage
        let new_coverage = SingleTestCaseCoverage::new(
            TestCaseId::default(),
            witness,
            brillig_coverage.clone().unwrap(),
            bool_witness_list,
        );
        // Quickly detect if there is any new coverage so that later single-threaded check can quickly discard this testcase
        accumulated_coverage.detect_new_coverage(&new_coverage)
    }

    /// Start the fuzzing campaign
    pub fn fuzz(&mut self) -> FuzzTestResult {
        self.metrics.set_num_threads(self.num_threads);
        // Generate a seed for the campaign
        let seed = thread_rng().gen::<u64>();

        // Init a fast PRNG used throughout the campain
        let mut prng = XorShiftRng::seed_from_u64(seed);

        // Initialize the starting corpus
        let mut corpus = Corpus::new(
            &self.corpus_folder,
            &self.package_name,
            &self.function_name,
            &self.acir_program.abi,
        );

        // Try loading the corpus from previous campaigns
        match corpus.attempt_to_load_corpus_from_disk() {
            Ok(_) => (),
            Err(error_string) => {
                return FuzzTestResult::CorpusFailure(error_string);
            }
        }

        // Init accumulated coverage object for tracking explored states
        let mut accumulated_coverage =
            AccumulatedFuzzerCoverage::new(&self.brillig_coverage_ranges);

        // Get the initial corpus from disk
        let mut starting_corpus_ids: Vec<_> =
            corpus.get_full_stored_corpus().iter().map(|x| x.id()).collect();

        if self.minimize_corpus && starting_corpus_ids.is_empty() {
            return FuzzTestResult::MinimizationFailure(
                "No initial corpus found to minimize".to_string(),
            );
        }

        let minimized_corpus = if self.minimize_corpus {
            Some(Corpus::new(
                &self.minimized_corpus_folder,
                &self.package_name,
                &self.function_name,
                &self.acir_program.abi,
            ))
        } else {
            None
        };
        let mut minimized_corpus_path = PathBuf::new();
        if self.minimize_corpus {
            minimized_corpus_path =
                minimized_corpus.as_ref().unwrap().get_corpus_storage_path().to_path_buf();
        }
        display_starting_info(
            self.minimize_corpus,
            seed,
            starting_corpus_ids.len(),
            self.num_threads,
            &self.package_name,
            &self.function_name,
            corpus.get_corpus_storage_path(),
            &minimized_corpus_path,
        );

        // Generate the default input (it is needed if the corpus is empty)
        let default_map = self.mutator.generate_default_input_map();
        if starting_corpus_ids.is_empty() {
            let default_testcase = TestCase::from(&default_map);
            match corpus.insert(
                default_testcase.id(),
                default_testcase.value().clone(),
                /*save_to_disk=*/ true,
            ) {
                Ok(_) => (),
                Err(error_string) => {
                    return FuzzTestResult::CorpusFailure(error_string);
                }
            }
            starting_corpus_ids.push(default_testcase.id());
        }

        // Initialize the pool we'll be using for parallelizing fuzzing
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .stack_size(4 * 1024 * 1024)
            .build()
            .unwrap();

        let testcases_per_iteration = self.num_threads * 2;
        let mut time_tracker = Instant::now();
        let mut brillig_executions_multiplier = 1usize;
        let mut acir_executions_multiplier = 1usize;
        let mut processed_starting_corpus = false;
        let fuzz_res = loop {
            let mut testcase_set: Vec<FuzzTask> = Vec::new();

            // If the total time spent in brillig is more than the time spent in ACIR, then enable ACIR for execution, otherwise execute just brillig
            // The reason is that brillig can be hundreds of times faster than ACIR and we want to balance execution so we don't waste an opportunity
            // to execute a bunch of testcases while limiting information from ACIR, instead of getting all the information, but from very few testcases
            // We also do an ACIR+Brillig round if we haven't processed the starting corpus yet
            let acir_round = self.metrics.is_brillig_dominating() || !processed_starting_corpus;

            if processed_starting_corpus {
                // If this is a standard and not a starting round
                // We want to send so many testcases to the multithreaded pool that we lose very little execution in relative terms while we wait for all threads to finish
                // So we scale so that the time to execute all of them in parallel is `SINGLE_FUZZING_ROUND_TARGET_TIME`
                let current_testcase_set_size = if acir_round {
                    acir_executions_multiplier * testcases_per_iteration
                } else {
                    brillig_executions_multiplier * testcases_per_iteration
                };
                testcase_set.reserve(current_testcase_set_size);

                // Get indices of testcases from the corpus that will be used in the current round of mutations
                // This is very fast, because we just do some simple arithmetic and get TestCaseIds, no copying of testcase bodies is taking place
                for _ in 0..current_testcase_set_size {
                    let (main_testcase, additional_testcase) = if acir_round {
                        corpus.get_next_testcase_for_acir(&mut prng)
                    } else {
                        corpus.get_next_testcase_for_brillig(&mut prng)
                    };

                    // Generate seeds for use by individual threads (we can't reuse our main PRNG because of parallelism)
                    let mut seed_bytes: SimpleXorShiftRNGSeed = [0; 16];
                    prng.fill_bytes(&mut seed_bytes);

                    testcase_set.push(FuzzTask::new(
                        main_testcase,
                        additional_testcase,
                        seed_bytes,
                    ));
                }
            } else {
                // If this is the initial processing round, then push testcases from the starting corpus into the set
                testcase_set.reserve(starting_corpus_ids.len());
                for id in starting_corpus_ids.iter() {
                    testcase_set.push(FuzzTask::mutationless(*id));
                }
            }
            let mutation_and_fuzzing_time_tracker = Instant::now();
            let current_round_size = testcase_set.len();
            // Mutate and execute the testcases
            let all_fuzzing_results: Vec<FastParallelFuzzResult> = pool.install(|| {
                testcase_set
                    .into_par_iter()
                    .map(|fuzz_task| {
                        // Initialize a prng from per-thread seed
                        let mut thread_prng = XorShiftRng::from_seed(fuzz_task.prng_seed());

                        let mutation_time_tracker = Instant::now();

                        // Generate a mutated input by using the main and additional testcases in the corpus
                        let input = if processed_starting_corpus {
                            self.mutator.generate_mutated_input(
                                corpus.get_testcase_by_id(fuzz_task.main_id()).clone(),
                                fuzz_task.additional_id().map(|additional_testcase_index| {
                                    corpus.get_testcase_by_id(additional_testcase_index).clone()
                                }),
                                &mut thread_prng,
                            )
                        } else {
                            // Or just get the input from the starting corpus if this is the first round
                            corpus.get_testcase_by_id(fuzz_task.main_id()).clone()
                        };

                        // Time mutations
                        let mutation_elapsed = mutation_time_tracker.elapsed().as_micros();

                        // Form a testcase from input (assign a unique id)
                        let testcase = TestCase::from(&input);

                        let fuzz_call_outcome = if acir_round {
                            // If the round uses ACIR, run both ACIR and brillig execution
                            self.single_fuzz_acir_and_brillig(&testcase)
                        } else {
                            // If this is a brillig round, execute just the brillig program
                            self.single_fuzz_brillig(&testcase)
                        };

                        if let HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                            case_id,
                            case,
                            witness,
                            brillig_coverage,
                            acir_time,
                            brillig_time,
                        }) = fuzz_call_outcome
                        {
                            // If the outcome is successful, collect coverage
                            let new_coverage_detected =
                                Self::detect_new_coverage_from_witness_and_brillig(
                                    &accumulated_coverage,
                                    &witness,
                                    &brillig_coverage,
                                );

                            FastParallelFuzzResult::new(
                                HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                                    case_id,
                                    case,
                                    witness,
                                    brillig_coverage,
                                    acir_time,
                                    brillig_time,
                                }),
                                new_coverage_detected,
                                /*failure_detected=*/ false,
                                mutation_elapsed,
                                acir_time,
                                brillig_time,
                            )
                        } else {
                            // We don't care abut acir and brillig time any more if we now need to inform the user that something went wrong or we found a bug
                            FastParallelFuzzResult::new(
                                fuzz_call_outcome,
                                /*new_coverage_detected=*/
                                false, // we don't care about new coverage if we've detected a failure
                                /*failure_detected=*/ true,
                                mutation_elapsed,
                                /*acir_time=*/ 0,
                                /*brillig_time=*/ 0,
                            )
                        }
                    })
                    .collect::<Vec<FastParallelFuzzResult>>()
            });
            let fuzz_time_micros = mutation_and_fuzzing_time_tracker.elapsed().as_micros();

            if processed_starting_corpus {
                // Update the testcase execution multipliers so that we spend at least around 200ms on each round
                let mut time_per_testcase =
                    fuzz_time_micros / brillig_executions_multiplier as u128;
                time_per_testcase = max(time_per_testcase, 30);
                let executions_multiplier =
                    (SINGLE_FUZZING_ROUND_TARGET_TIME / time_per_testcase) as usize;
                if acir_round {
                    acir_executions_multiplier = max(1, executions_multiplier);
                } else {
                    brillig_executions_multiplier = max(1, executions_multiplier);
                }
            }

            let mut failing_result = None;
            let updating_time_tracker = Instant::now();

            self.metrics.increase_processed_testcase_count(&current_round_size);

            // Check if there are any failures and immediately break the loop if some are found
            if let Some(individual_failing_result) =
                all_fuzzing_results.iter().find(|fast_result| fast_result.failed())
            {
                break individual_failing_result.outcome().clone();
            }

            let mut analysis_queue = Vec::new();

            // Update metrics for everything and push interesting results to the analysis queue
            for (index, fast_result) in all_fuzzing_results.iter().enumerate() {
                self.metrics.increase_total_acir_time(&fast_result.acir_time());
                self.metrics.increase_total_brillig_time(&fast_result.brillig_time());
                self.metrics.increase_total_mutation_time(&fast_result.mutation_time());
                if !fast_result.skip_check() {
                    analysis_queue.push(index)
                }
            }

            let mut acir_cases_to_execute = Vec::new();
            // Go through each interesting testcase (new coverage or some issue)
            for index in analysis_queue.into_iter() {
                let fuzzing_outcome = all_fuzzing_results[index].outcome().clone();
                let (case_id, case, witness, brillig_coverage) = match fuzzing_outcome {
                    HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                        case_id,
                        case,
                        witness,
                        brillig_coverage,
                        acir_time: _,
                        brillig_time: _,
                    }) => (case_id, case, witness, brillig_coverage.unwrap()), // There should always be brillig coverage
                    _ => {
                        panic!(
                            "All non-successful fuzzing outcomes should have been detected earlier"
                        )
                    }
                };
                // If we ran ACIR and  managed to produce an ACIR witness
                if acir_round && witness.is_some() {
                    // Update the potential list
                    if accumulated_coverage.potential_bool_witness_list.is_none() {
                        // If it's the first time, we need to assign
                        accumulated_coverage.potential_bool_witness_list =
                            Some(PotentialBoolWitnessList::from(witness.as_ref().unwrap()));
                    } else {
                        accumulated_coverage
                            .potential_bool_witness_list
                            .as_mut()
                            .unwrap()
                            .update(witness.as_ref().unwrap());
                    }
                }

                // Form the coverage object to accumulate
                let new_coverage = SingleTestCaseCoverage::new(
                    case_id,
                    &witness,
                    brillig_coverage.clone(),
                    accumulated_coverage
                        .potential_bool_witness_list
                        .as_ref()
                        .unwrap_or(&PotentialBoolWitnessList::default()),
                );

                // In case this is just a brillig round, we need to detect first, since a merge might skip some witnesses that we haven't added from acir
                if !acir_round && accumulated_coverage.detect_new_coverage(&new_coverage) {
                    acir_cases_to_execute.push((case_id, case.clone(), brillig_coverage));
                    continue;
                }

                // If both acir and brillig have been run, we can try to merge the coverage (there is an automatic detect)
                // There might not be new coverage if there are several testcases with the same new coverage in comparison to the previous round
                let (new_coverage_discovered, testcases_to_remove) =
                    accumulated_coverage.merge(&new_coverage);

                // If there is new coverage
                if new_coverage_discovered {
                    // Remove testcases from the corpus if they have no unique features
                    for &testcase_for_removal in testcases_to_remove.iter() {
                        self.metrics.increment_removed_testcase_count();
                        corpus.remove(testcase_for_removal);
                    }

                    // Add values from the interesting testcase to the dictionary
                    self.mutator.update_dictionary(&case);

                    //Insert the new testcase into the corpus
                    match corpus.insert(case_id, case, true) {
                        Ok(_) => (),
                        Err(error_string) => {
                            return FuzzTestResult::CorpusFailure(error_string);
                        }
                    }
                    self.metrics.increment_acir_brillig_discoveries();
                }
            }

            let updating_time = updating_time_tracker.elapsed().as_micros();

            // Execute interesting testcases in ACIR to collect witness if they have been executed just in brillig
            let all_fuzzing_results: Vec<HarnessExecutionOutcome> = pool.install(|| {
                acir_cases_to_execute
                    .into_par_iter()
                    .map(|(case_id, input, brillig_coverage )| {
                        let testcase = TestCase::with_id(case_id, &input);
                        let fuzz_res = self.single_fuzz_acir(&testcase);
                        match fuzz_res {
                            HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                                case_id,
                                case,
                                witness,
                                brillig_coverage: _,
                                acir_time,
                                brillig_time: _,
                            }) => HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                                case_id,
                                case,
                                witness,
                                brillig_coverage: Some(brillig_coverage),
                                acir_time,
                                brillig_time:0,// we've already used this brillig time in calculations, so it doesn't matter
                            }),
                            HarnessExecutionOutcome::Discrepancy(..) => {
                                panic!("Can't get a discrepancy just from acir")
                            }
                            HarnessExecutionOutcome::CounterExample(CounterExampleOutcome {
                                case_id,
                                counterexample,
                                exit_reason,
                            }) => HarnessExecutionOutcome::Discrepancy(DiscrepancyOutcome {
                                case_id,
                                counterexample,
                                acir_failed: true,
                                exit_reason,
                            }),
                            HarnessExecutionOutcome::ForeignCallFailure(..) => {
                                panic!("Can't get a foreign call problem in ACIR while having none in brillig")
                            }
                        }
                    })
                    .collect::<Vec<HarnessExecutionOutcome>>()
            });

            // Parse results and if there is an unsuccessful case break out of the loop
            for acir_fuzzing_result in all_fuzzing_results.into_iter() {
                let (case_id, case, witness, brillig_coverage, acir_time) =
                    match acir_fuzzing_result {
                        HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                            case_id,
                            case,
                            witness,
                            brillig_coverage,
                            acir_time,
                            brillig_time: _,
                        }) => (
                            case_id,
                            case,
                            witness,
                            brillig_coverage.unwrap(), /*there should always be brillig coverage */
                            acir_time,
                        ),
                        _ => {
                            failing_result = Some(acir_fuzzing_result);
                            break;
                        }
                    };
                self.metrics.increase_total_acir_time(&acir_time);

                // In case ACIR execution was successful
                if witness.is_some() {
                    // Update the potential list
                    if accumulated_coverage.potential_bool_witness_list.is_none() {
                        // If it's the first time, we need to assign
                        accumulated_coverage.potential_bool_witness_list =
                            Some(PotentialBoolWitnessList::from(witness.as_ref().unwrap()));
                    } else {
                        accumulated_coverage
                            .potential_bool_witness_list
                            .as_mut()
                            .unwrap()
                            .update(witness.as_ref().unwrap());
                    }
                }
                // If we ran just brillig at the start, we won't have a potential bool witness list, so we need a dummy
                let mut dummy_witness_list_for_brillig = PotentialBoolWitnessList::default();

                let new_coverage = SingleTestCaseCoverage::new(
                    case_id,
                    &witness,
                    brillig_coverage,
                    accumulated_coverage
                        .potential_bool_witness_list
                        .as_mut()
                        .unwrap_or(&mut dummy_witness_list_for_brillig),
                );
                let (new_coverage_discovered, testcases_to_remove) =
                    accumulated_coverage.merge(&new_coverage);
                if new_coverage_discovered {
                    for &testcase_for_removal in testcases_to_remove.iter() {
                        self.metrics.increment_removed_testcase_count();
                        corpus.remove(testcase_for_removal);
                    }
                    self.mutator.update_dictionary(&case);
                    match corpus.insert(case_id, case, true) {
                        Ok(_) => (),
                        Err(error_string) => {
                            return FuzzTestResult::CorpusFailure(error_string);
                        }
                    }

                    self.metrics.increment_brillig_discoveries();
                }
            }
            // If we've found something, return
            if let Some(result) = failing_result {
                break result;
            }
            if time_tracker.elapsed().as_secs() >= 1 {
                // Update and display metrics
                self.metrics.set_active_corpus_size(corpus.get_testcase_count());
                self.metrics.set_last_round_size(current_round_size);
                self.metrics.set_last_round_update_time(updating_time);
                self.metrics.set_last_round_execution_time(fuzz_time_micros);
                display_metrics(&self.metrics);
                self.metrics.refresh_round();
                time_tracker = Instant::now();
            }
            // We have now definitely processed the starting corpus
            processed_starting_corpus = true;
        };

        // Parse the execution result and convert it to the FuzzTestResult
        match fuzz_res {
            HarnessExecutionOutcome::Case(_) => FuzzTestResult::Success,
            HarnessExecutionOutcome::Discrepancy(DiscrepancyOutcome {
                case_id: _,
                exit_reason: status,
                acir_failed,
                counterexample,
            }) => {
                let reason = match acir_failed {
                    true => {
                        format!("ACIR failed while brillig executed with no issues: {}", status)
                    }
                    false => {
                        format!("brillig failed while ACIR executed with no issues: {}", status)
                    }
                };

                FuzzTestResult::ProgramFailure(ProgramFailureResult {
                    failure_reason: reason,
                    counterexample: counterexample.clone(),
                })
            }
            HarnessExecutionOutcome::CounterExample(CounterExampleOutcome {
                case_id: _,
                exit_reason: status,
                counterexample,
            }) => {
                let reason = status.to_string();
                FuzzTestResult::ProgramFailure(ProgramFailureResult {
                    failure_reason: reason,
                    counterexample: counterexample.clone(),
                })
            }
            HarnessExecutionOutcome::ForeignCallFailure(foreign_call_error_in_fuzzing) => {
                FuzzTestResult::ForeignCallFailure(
                    foreign_call_error_in_fuzzing.exit_reason.to_string(),
                )
            }
        }
    }

    /// Execute acir and brillig programs with the following Testcase
    pub fn single_fuzz_acir_and_brillig(&self, testcase: &TestCase) -> HarnessExecutionOutcome {
        let initial_witness = self.acir_program.abi.encode(testcase.value(), None).unwrap();
        let initial_witness2 = self.acir_program.abi.encode(testcase.value(), None).unwrap();

        let acir_start = Instant::now();
        // Execute and time ACIR
        let result_acir = (self.acir_executor)(&self.acir_program.bytecode, initial_witness);
        let acir_elapsed = acir_start.elapsed();
        let brillig_start = Instant::now();
        // Execute and time Brillig
        let result_brillig = (self.brillig_executor)(
            &self.brillig_program.bytecode,
            initial_witness2,
            &self.location_to_feature_map,
        );
        let brillig_elapsed = brillig_start.elapsed();

        // Parse results
        match (result_acir, result_brillig) {
            (Ok(witnesses), Ok((_map, brillig_coverage))) => {
                // If both were OK, collect coverage and ACIR witnesses along with timings and return
                HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                    case_id: testcase.id(),
                    case: testcase.value().clone(),
                    witness: Some(witnesses),
                    brillig_coverage: Some(brillig_coverage.unwrap()),
                    acir_time: acir_elapsed.as_micros(),
                    brillig_time: brillig_elapsed.as_micros(),
                })
            }
            // If results diverge, it's a discrepancy
            (Err(err), Ok(_)) => HarnessExecutionOutcome::Discrepancy(DiscrepancyOutcome {
                case_id: testcase.id(),
                exit_reason: err,
                acir_failed: true,
                counterexample: testcase.value().clone(),
            }),
            (Ok(_), Err((err, _))) => HarnessExecutionOutcome::Discrepancy(DiscrepancyOutcome {
                case_id: testcase.id(),
                exit_reason: err,
                acir_failed: false,
                counterexample: testcase.value().clone(),
            }),
            // If both failed, then we need to check
            (Err(..), Err((err, coverage))) => {
                // If this is a foreign call failure, we need to inform the user
                if err.contains(FOREIGN_CALL_FAILURE_SUBSTRING) {
                    return HarnessExecutionOutcome::ForeignCallFailure(
                        types::ForeignCallErrorInFuzzing { exit_reason: err },
                    );
                }
                // If failures are expected and this is not the failure that we are looking for, then don't treat as failure
                if self.failure_configuration.fail_on_specific_asserts
                    && !err.contains(
                        self.failure_configuration
                            .failure_reason
                            .as_ref()
                            .expect("Failure reason should be provided"),
                    )
                {
                    return HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                        case_id: testcase.id(),
                        case: testcase.value().clone(),
                        witness: None,
                        brillig_coverage: coverage,
                        acir_time: acir_elapsed.as_micros(),
                        brillig_time: brillig_elapsed.as_micros(),
                    });
                }

                // This is a bug, inform the user
                HarnessExecutionOutcome::CounterExample(CounterExampleOutcome {
                    case_id: testcase.id(),
                    exit_reason: err,
                    counterexample: testcase.value().clone(),
                })
            }
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz_acir(&self, testcase: &TestCase) -> HarnessExecutionOutcome {
        let initial_witness = self.acir_program.abi.encode(testcase.value(), None).unwrap();
        let acir_start = Instant::now();
        let result_acir = (self.acir_executor)(&self.acir_program.bytecode, initial_witness);
        let acir_elapsed = acir_start.elapsed();

        match result_acir {
            Ok(witnesses) => HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                case_id: testcase.id(),
                case: testcase.value().clone(),
                witness: Some(witnesses),
                brillig_coverage: None,
                acir_time: acir_elapsed.as_micros(),
                brillig_time: 0,
            }),
            Err(err) => {
                if err.contains(FOREIGN_CALL_FAILURE_SUBSTRING) {
                    return HarnessExecutionOutcome::ForeignCallFailure(
                        types::ForeignCallErrorInFuzzing { exit_reason: err },
                    );
                }
                if self.failure_configuration.fail_on_specific_asserts
                    && !err.contains(
                        self.failure_configuration
                            .failure_reason
                            .as_ref()
                            .expect("Failure reason should be provided"),
                    )
                {
                    // TODO: in the future we can add partial witness propagation from ACIR
                    return HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                        case_id: testcase.id(),
                        case: testcase.value().clone(),
                        witness: None,
                        brillig_coverage: None,
                        acir_time: acir_elapsed.as_micros(),
                        brillig_time: 0,
                    });
                }
                HarnessExecutionOutcome::CounterExample(CounterExampleOutcome {
                    case_id: testcase.id(),
                    exit_reason: err,
                    counterexample: testcase.value().clone(),
                })
            }
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz_brillig(&self, testcase: &TestCase) -> HarnessExecutionOutcome {
        let initial_witness = self.acir_program.abi.encode(testcase.value(), None).unwrap();
        let brillig_start = Instant::now();
        let result_brillig = (self.brillig_executor)(
            &self.brillig_program.bytecode,
            initial_witness,
            &self.location_to_feature_map,
        );
        let brillig_elapsed = brillig_start.elapsed();

        match result_brillig {
            Ok((_, brillig_coverage)) => HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                case_id: testcase.id(),
                case: testcase.value().clone(),
                witness: None,
                brillig_coverage: Some(brillig_coverage.unwrap()),
                acir_time: 0,
                brillig_time: brillig_elapsed.as_micros(),
            }),
            Err((err, coverage)) => {
                if err.contains(FOREIGN_CALL_FAILURE_SUBSTRING) {
                    return HarnessExecutionOutcome::ForeignCallFailure(
                        types::ForeignCallErrorInFuzzing { exit_reason: err },
                    );
                }
                if self.failure_configuration.fail_on_specific_asserts
                    && !err.contains(
                        self.failure_configuration
                            .failure_reason
                            .as_ref()
                            .expect("Failure reason should be provided"),
                    )
                {
                    return HarnessExecutionOutcome::Case(SuccessfulCaseOutcome {
                        case_id: testcase.id(),
                        case: testcase.value().clone(),
                        witness: None,
                        brillig_coverage: Some(coverage.unwrap()),
                        acir_time: 0,
                        brillig_time: brillig_elapsed.as_micros(),
                    });
                }
                HarnessExecutionOutcome::CounterExample(CounterExampleOutcome {
                    case_id: testcase.id(),
                    exit_reason: err,
                    counterexample: testcase.value().clone(),
                })
            }
        }
    }
}

// A method for pretty display starting information
fn display_starting_info(
    minimize_corpus: bool,
    seed: u64,
    starting_corpus_size: usize,
    num_threads: usize,
    package_name: &str,
    fuzzing_harness_name: &str,
    corpus_path: &Path,
    minimized_corpus_path: &Path,
) {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();
    if minimize_corpus {
        write!(writer, "Attempting to minimize corpus for fuzzing harness ")
            .expect("Failed to write to stderr");
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
        write!(writer, "{}", fuzzing_harness_name).expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
        write!(writer, " of package ").expect("Failed to write to stderr");
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
        writeln!(writer, "{}", package_name).expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
        write!(writer, "Corpus path: \"").expect("Failed to write to stderr");
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
        write!(writer, "{}", corpus_path.to_str().expect("There can't be no path"))
            .expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
        write!(writer, "\"\nMinimized corpus path: \"").expect("Failed to write to stderr");
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
        write!(writer, "{}", minimized_corpus_path.to_str().expect("There can't be no path"))
            .expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
        writeln!(writer, "\"").expect("Failed to write to stderr");
    } else {
        write!(writer, "Starting fuzzing with harness ").expect("Failed to write to stderr");
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
        write!(writer, "{}", fuzzing_harness_name).expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
        write!(writer, " of package ").expect("Failed to write to stderr");
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
        writeln!(writer, "{}", package_name).expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
        write!(writer, "Corpus path: \"").expect("Failed to write to stderr");
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
        write!(writer, "{}", corpus_path.to_str().expect("There can't be no path"))
            .expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
        writeln!(writer, "\"").expect("Failed to write to stderr");
    }
    write!(writer, "seed: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{:#016x}", seed).expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", starting_corpus_size: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", starting_corpus_size).expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", num_threads: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    writeln!(writer, "{}", num_threads).expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");
    writer.flush().expect("Failed to flush writer");
}
// A method for pretty display of fuzzing metrics
fn display_metrics(metrics: &Metrics) {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();
    let format_time = |x: u128| {
        let microseconds_in_second = 1_000_000;
        let microseconds_in_millisecond = 1_000;
        let microseconds_in_minutes = 60_000_000;
        let microseconds_in_an_hour = microseconds_in_minutes * 60;
        let microseconds_in_4_hours = microseconds_in_an_hour * 4;
        if x > microseconds_in_4_hours {
            format!("{}h", x / microseconds_in_an_hour)
        } else if x > microseconds_in_minutes {
            format!("{}m", x / microseconds_in_minutes)
        } else if x > microseconds_in_second {
            format!("{}s", x / microseconds_in_second)
        } else if x > microseconds_in_millisecond {
            format!("{}ms", x / microseconds_in_millisecond)
        } else {
            format!("{}us", x)
        }
    };
    let format_count = |x: usize| {
        let million = 1_000_000;
        let thousand = 1_000;
        let billion = 1_000_000_000;
        if x > billion {
            format!("{}G", x / billion)
        } else if x > million {
            format!("{}M", x / million)
        } else if x > thousand {
            format!("{}k", x / thousand)
        } else {
            format!("{}", x)
        }
    };
    if metrics.found_new_with_acir_brillig || metrics.found_new_with_brillig {
        writer
            .set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))
            .expect("Failed to set color");
        write!(writer, "NEW:  ").expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
    } else {
        write!(writer, "LOOP: ").expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
    }
    write!(writer, "CNT: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_count(metrics.processed_testcase_count))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", CRPS: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_count(metrics.active_corpus_size))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", AB_NEW: ").expect("Failed to write to stderr");
    writer
        .set_color(ColorSpec::new().set_fg(if metrics.found_new_with_acir_brillig {
            Some(Color::Magenta)
        } else {
            Some(Color::Blue)
        }))
        .expect("Failed to set color");
    write!(writer, "{}", format_count(metrics.acir_brillig_discoveries))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", B_NEW: ").expect("Failed to write to stderr");
    writer
        .set_color(ColorSpec::new().set_fg(if metrics.found_new_with_brillig {
            Some(Color::Magenta)
        } else {
            Some(Color::Blue)
        }))
        .expect("Failed to set color");
    write!(writer, "{}", format_count(metrics.brillig_discoveries))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", RMVD: ").expect("Failed to write to stderr");
    writer
        .set_color(ColorSpec::new().set_fg(if metrics.removed_testcase_last_round {
            Some(Color::Magenta)
        } else {
            Some(Color::Blue)
        }))
        .expect("Failed to set color");
    write!(writer, "{}", format_count(metrics.removed_testcase_count))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", A_TIME: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_time(metrics.total_acir_execution_time))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", B_TIME: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_time(metrics.total_brillig_execution_time))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", M_TIME: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_time(metrics.total_mutation_time))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", RND_SIZE: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_count(metrics.last_round_size)).expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", RND_EX_TIME: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_time(metrics.last_round_execution_time))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", UPD_TIME: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    writeln!(writer, "{}", format_time(metrics.last_round_update_time))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    writer.flush().expect("Failed to flush writer");
}
