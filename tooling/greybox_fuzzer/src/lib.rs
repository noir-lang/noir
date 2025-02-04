use std::{cmp::max, time::Instant};

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

use corpus::{Corpus, TestCase, TestCaseId};
use mutation::InputMutator;
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    ThreadPool,
};
use termcolor::{ColorChoice, StandardStream};
use types::{
    CounterExampleOutcome, DiscrepancyOutcome, FuzzOutcome, FuzzTestResult, SuccessfulCaseOutcome,
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
    pub fn increment_processed_testcase_count(&mut self) {
        self.processed_testcase_count += 1;
    }
    pub fn increase_processed_testcase_count(&mut self, update: &usize) {
        self.processed_testcase_count += update;
    }
    pub fn increment_removed_testcase_count(&mut self) {
        self.removed_testcase_count += 1;
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

    /// Fail on specific asserts
    fail_on_specific_asserts: bool,

    /// Failure reason
    failure_reason: Option<String>,

    /// Execution metric
    metrics: Metrics,
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
            )
                -> Result<(WitnessStack<FieldElement>, Option<Vec<u32>>), (String, Option<Vec<u32>>)>
            + Sync,
    > FuzzedExecutor<E, F>
{
    /// Instantiates a fuzzed executor given an executor
    pub fn new(
        acir_program: ProgramArtifact,
        brillig_program: ProgramArtifact,
        acir_executor: E,
        brillig_executor: F,
        package_name: &str,
        function_name: &str,
        num_threads: usize,
        fail_on_specific_asserts: bool,
        failure_reason: Option<String>,
    ) -> Self {
        // Analyze brillig program for branches and comparisons
        let (location_to_feature_map, brillig_coverage_ranges) =
            analyze_brillig_program_before_fuzzing(&brillig_program);

        // Create a dictionary from acir bytecode
        // TODO: replace with building a dictionary from brillig. It makes much more sense
        let dictionary = build_dictionary_from_program(&acir_program.bytecode);

        // Create a mutator for the following interface with the dictionary generated from acir bytecode
        let mutator = InputMutator::new(&acir_program.abi, &dictionary);

        Self {
            acir_program,
            brillig_program,
            acir_executor,
            brillig_executor,
            location_to_feature_map,
            brillig_coverage_ranges,
            mutator,
            package_name: package_name.to_string(),
            function_name: function_name.to_string(),
            num_threads,
            fail_on_specific_asserts,
            failure_reason,
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
            &witness,
            brillig_coverage.clone().unwrap(),
            bool_witness_list,
        );
        // Quickly detect if there is any new coverage so that later single-threaded check can quickly discard this testcase
        !accumulated_coverage.detect_new_coverage(&new_coverage)
    }

    /// Start the fuzzing campaign
    pub fn fuzz(&mut self) -> FuzzTestResult {
        self.metrics.set_num_threads(self.num_threads);
        // Generate a seed for the campaign
        let seed = thread_rng().gen::<u64>();
        println!("Fuzzing seed for this campaign: {}", seed);

        // Init a fast PRNG used throughout the campain
        let mut prng = XorShiftRng::seed_from_u64(seed);

        // Initialize the starting corpus
        // TODO: allow setting the base directory
        let mut corpus =
            Corpus::new(&self.package_name, &self.function_name, &self.acir_program.abi);

        // Try loading the corpus from previous campaigns
        match corpus.attempt_to_load_corpus_from_disk() {
            Ok(_) => (),
            Err(error_string) => {
                return FuzzTestResult {
                    success: false,
                    reason: Some(error_string),
                    counterexample: None,
                    foreign_call_failure: false,
                };
            }
        }

        // Init accumulated coverage object for tracking explored states
        let mut accumulated_coverage =
            AccumulatedFuzzerCoverage::new(&self.brillig_coverage_ranges);

        // Get the initial corpus from disk
        let mut starting_corpus_ids: Vec<_> =
            corpus.get_full_stored_corpus().iter().map(|x| x.id()).collect();

        println!("Starting corpus size: {}", starting_corpus_ids.len());

        let mut start_with_only_default_input = false;

        // Generate the default input (it is needed if the corpus is empty)
        let default_map = self.mutator.generate_default_input_map();
        if starting_corpus_ids.is_empty() {
            start_with_only_default_input = true;
            let default_testcase = TestCase::from(&default_map);
            corpus.insert(
                default_testcase.id(),
                default_testcase.value().clone(),
                /*save_to_disk=*/ true,
            );
            starting_corpus_ids.push(default_testcase.id());
        }

        // Initialize the pool we'll be using for parallelizing fuzzing
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .stack_size(4 * 1024 * 1024)
            .build()
            .unwrap();

        // // Execute initial corpus
        // let fuzzing_results: Vec<FuzzOutcome> =
        //     self.execute_testcases_in_parallel(&pool, &starting_corpus);

        // // Parse the result from the initial corpus execution
        // let initial_corpus_result = self
        //     .parse_fuzzing_results_and_update_accumulated_coverage_and_corpus(
        //         &fuzzing_results,
        //         &mut accumulated_coverage,
        //         &mut corpus,
        //         start_with_only_default_input,
        //     );

        // // It is possible that there is already an erroneous input or a foreign call failed to resolve, so check
        // match initial_corpus_result {
        //     Ok(_) => {}
        //     Err(fuzz_outcome) => {
        //         return fuzz_outcome;
        //     }
        // }

        let testcases_per_iteration = self.num_threads * 2;
        let mut time_tracker = Instant::now();
        let mut brillig_executions_multiplier = 1usize;
        let mut acir_executions_multiplier = 1usize;
        let mut processed_starting_corpus = false;
        let fuzz_res = loop {
            let mut testcase_set: Vec<(
                TestCaseId,
                Option<TestCaseId>,
                <XorShiftRng as SeedableRng>::Seed,
            )> = Vec::new();

            // If the total time spent in brillig is more than the time spent in ACIR, then enable ACIR for execution, otherwise execute just brillig
            // The reason is that brillig can be hundreds of times faster than ACIR and we want to balance execution so we don't waste an opportunity
            // to execute a bunch of testcases while limiting information from ACIR, instead of getting all the information, but from very few testcases
            // We also do an ACIR+Brillig round if we haven't processed the starting corpus yet
            let acir_round = self.metrics.is_brillig_dominating() || !processed_starting_corpus;

            if processed_starting_corpus {
                // If this is a standard and not a starting round
                // We want to send so many testcases to the multithreaded pool that we lose very little execution in relative terms while we wait for all threads to finish
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
                    let mut seed_bytes: <XorShiftRng as SeedableRng>::Seed = [0; 16];
                    prng.fill_bytes(&mut seed_bytes);

                    testcase_set.push((main_testcase, additional_testcase, seed_bytes));
                }
            } else {
                // If this is the initial processing round, then push testcases from the starting corpus into the set
                testcase_set.reserve(starting_corpus_ids.len());
                for id in starting_corpus_ids.iter() {
                    testcase_set.push((*id, None, [0; 16]));
                }
            }
            let mutation_and_fuzzing_time_tracker = Instant::now();
            let current_round_size = testcase_set.len();
            // Mutate and execute the testcases
            let all_fuzzing_results: Vec<(FuzzOutcome, bool, u128, u128, u128)> =
                pool.install(|| {
                    testcase_set
                        .into_par_iter()
                        .map(|(main_testcase_index, additional_testcase_index, thread_seed)| {
                            // Initialize a prng from per-thread seed
                            let mut thread_prng = XorShiftRng::from_seed(thread_seed);

                            let mutation_time_tracker = Instant::now();

                            // Generate a mutated input by using the main and additional testcases in the corpus
                            let input = if processed_starting_corpus {
                                self.mutator.generate_mutated_input(
                                    corpus.get_testcase_by_id(main_testcase_index).clone(),
                                    additional_testcase_index.map(|additional_testcase_index| {
                                        corpus.get_testcase_by_id(additional_testcase_index).clone()
                                    }),
                                    &mut thread_prng,
                                )
                            } else {
                                // Or just get the input from the starting corpus if this is the first round
                                corpus.get_testcase_by_id(main_testcase_index).clone()
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

                            if let FuzzOutcome::Case(SuccessfulCaseOutcome {
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
                                (
                                    FuzzOutcome::Case(SuccessfulCaseOutcome {
                                        case_id,
                                        case,
                                        witness,
                                        brillig_coverage,
                                        acir_time,
                                        brillig_time,
                                    }),
                                    new_coverage_detected,
                                    mutation_elapsed,
                                    acir_time,
                                    brillig_time,
                                )
                            } else {
                                // We don't care abut acir and brillig time any more if we now need to inform the user that something went wrong or we found the bug
                                (fuzz_call_outcome, false, mutation_elapsed, 0, 0)
                            }
                        })
                        .collect::<Vec<(FuzzOutcome, bool, u128, u128, u128)>>()
                });
            let fuzz_time_micros = mutation_and_fuzzing_time_tracker.elapsed().as_micros();

            if processed_starting_corpus {
                // Update the testcase execution multipliers so that we spend at least around 200ms on each round
                let timing = 500_000u128;
                if acir_round {
                    let mut time_per_testcase =
                        fuzz_time_micros / acir_executions_multiplier as u128;
                    time_per_testcase = max(time_per_testcase, 30);
                    acir_executions_multiplier = (timing / time_per_testcase) as usize;
                    if acir_executions_multiplier == 0 {
                        acir_executions_multiplier = 1;
                    }
                } else {
                    let mut time_per_testcase =
                        fuzz_time_micros / brillig_executions_multiplier as u128;
                    time_per_testcase = max(time_per_testcase, 30);
                    brillig_executions_multiplier = (timing / time_per_testcase) as usize;
                    if brillig_executions_multiplier == 0 {
                        brillig_executions_multiplier = 1;
                    }
                }
            }
            let mut potential_res = None;
            let mut acir_cases_to_execute = Vec::new();
            let updating_time_tracker = Instant::now();

            self.metrics.increase_processed_testcase_count(&current_round_size);
            // Count how many testcases we skipped this round and update metrics
            for (_, _, mutation_time_micros, acir_time_micros, brillig_time_micros) in
                all_fuzzing_results
                    .iter()
                    .filter(|(_, should_skip_check, _, _, _)| *should_skip_check)
            {
                self.metrics.increase_total_acir_time(acir_time_micros);
                self.metrics.increase_total_brillig_time(brillig_time_micros);
                self.metrics.increase_total_mutation_time(mutation_time_micros);
            }
            let mut results_to_analyze = Vec::new();

            // Find testcases with new coverage and update metrics for them
            for (index, (_, __, mutation_time_micros, acir_time_micros, brillig_time_micros)) in
                all_fuzzing_results
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, should_skip_check, _, _, _))| !*should_skip_check)
            {
                self.metrics.increase_total_acir_time(acir_time_micros);
                self.metrics.increase_total_brillig_time(brillig_time_micros);
                self.metrics.increase_total_mutation_time(mutation_time_micros);
                results_to_analyze.push(index);
            }

            // Go through each interesting testcase (new coverage or some issue)
            for index in results_to_analyze.into_iter() {
                let fuzz_res = all_fuzzing_results[index].0.clone();
                let (case_id, case, witness, brillig_coverage, brillig_time) = match fuzz_res {
                    FuzzOutcome::Case(SuccessfulCaseOutcome {
                        case_id,
                        case,
                        witness,
                        brillig_coverage,
                        acir_time: _,
                        brillig_time,
                    }) => (case_id, case, witness, brillig_coverage, brillig_time),
                    // In case the result is not successful, break out of the loop and return it
                    _ => {
                        potential_res = Some(fuzz_res);
                        break;
                    }
                };
                // In case we fuzzed both ACIR and brillig
                if let Some(brillig_coverage) = brillig_coverage.clone() {
                    // If we managed to produce an ACIR witness
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
                    // Form the coverage object to accumulate
                    let new_coverage = SingleTestCaseCoverage::new(
                        case_id,
                        &witness,
                        brillig_coverage,
                        accumulated_coverage
                            .potential_bool_witness_list
                            .as_ref()
                            .unwrap_or(&PotentialBoolWitnessList::default()),
                    );

                    // Merge the coverage
                    let (new_coverage_discovered, testcases_to_remove) =
                        accumulated_coverage.merge(&new_coverage);

                    // If there is new coverage, which should always be the case here:
                    if new_coverage_discovered {
                        // Remove testcases from the corpus if they have no unique features
                        for &testcase_for_removal in testcases_to_remove.iter() {
                            self.metrics.increment_removed_testcase_count();
                            corpus.remove(testcase_for_removal);
                        }

                        // TODO: create flag to show inputs with new coverage
                        println!("Input: {:?}", case);

                        // Add values from the interesting testcase to the dictionary
                        self.mutator.update_dictionary(&case);
                        //Insert the new testcase into the corpus
                        match corpus.insert(case_id, case, true) {
                            Ok(_) => (),
                            Err(error_string) => {
                                return FuzzTestResult {
                                    success: false,
                                    reason: Some(error_string),
                                    counterexample: None,
                                    foreign_call_failure: false,
                                }
                            }
                        }
                        // TODO: FLAG
                        print!("Found new feature");
                        if processed_starting_corpus {
                            println!("!");
                        } else {
                            println!(" in starting corpus!")
                        }
                    }
                } else if let Some(brillig_coverage) = brillig_coverage {
                    let new_coverage = SingleTestCaseCoverage::new(
                        case_id,
                        &None,
                        brillig_coverage.clone(),
                        &PotentialBoolWitnessList::default(),
                    );
                    if accumulated_coverage.detect_new_coverage(&new_coverage) {
                        acir_cases_to_execute.push((
                            case_id,
                            case.clone(),
                            new_coverage.brillig_coverage,
                            brillig_time,
                        ));
                    }
                }
            }
            let updating_time = updating_time_tracker.elapsed().as_micros();

            if let Some(result) = potential_res {
                break result;
            }

            // Execute interesting testcases in ACIR if they haven't been to collect witness
            let all_fuzzing_results: Vec<FuzzOutcome> = pool.install(|| {
                acir_cases_to_execute
                    .into_par_iter()
                    .map(|(case_id, input, brillig_coverage, brillig_time)| {
                        let testcase = TestCase::with_id(case_id, &input);
                        let fuzz_res = self.single_fuzz_acir(&testcase);
                        match fuzz_res {
                            FuzzOutcome::Case(SuccessfulCaseOutcome {
                                case_id,
                                case,
                                witness,
                                brillig_coverage: _,
                                acir_time,
                                brillig_time: _,
                            }) => FuzzOutcome::Case(SuccessfulCaseOutcome {
                                case_id,
                                case,
                                witness,
                                brillig_coverage: Some(brillig_coverage),
                                acir_time,
                                brillig_time,
                            }),
                            FuzzOutcome::Discrepancy(..) => {
                                panic!("Can't get a discrepancy just from acir")
                            }
                            FuzzOutcome::CounterExample(CounterExampleOutcome {
                                case_id,
                                counterexample,
                                exit_reason,
                            }) => FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                                case_id,
                                counterexample,
                                acir_failed: true,
                                exit_reason,
                            }),
                            FuzzOutcome::ForeignCallFailure(..) => {
                                panic!("Can't get a foreign call problem in ACIR while having none in brillig")
                            }
                        }
                    })
                    .collect::<Vec<FuzzOutcome>>()
            });

            // Parse results and if there is an unsuccessful case break out of the loop
            for fuzz_res in all_fuzzing_results.into_iter() {
                let (case_id, case, witness, brillig_coverage, acir_time, brillig_time) =
                    match fuzz_res {
                        FuzzOutcome::Case(SuccessfulCaseOutcome {
                            case_id,
                            case,
                            witness,
                            brillig_coverage,
                            acir_time,
                            brillig_time,
                        }) => (case_id, case, witness, brillig_coverage, acir_time, brillig_time),
                        _ => {
                            potential_res = Some(fuzz_res);
                            break;
                        }
                    };
                // Parse brillig coverage
                if let Some(brillig_coverage) = brillig_coverage {
                    self.metrics.increase_total_brillig_time(&brillig_time);
                    // In case ACIR execution
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
                    let new_coverage = SingleTestCaseCoverage::new(
                        case_id,
                        &witness,
                        brillig_coverage,
                        accumulated_coverage.potential_bool_witness_list.as_mut().unwrap(),
                    );
                    let (new_coverage_discovered, testcases_to_remove) =
                        accumulated_coverage.merge(&new_coverage);
                    if new_coverage_discovered {
                        for &testcase_for_removal in testcases_to_remove.iter() {
                            self.metrics.increment_removed_testcase_count();
                            corpus.remove(testcase_for_removal);
                        }
                        println!("Input: {:?}", case);
                        self.mutator.update_dictionary(&case);
                        match corpus.insert(case_id, case, true) {
                            Ok(_) => (),
                            Err(error_string) => {
                                return FuzzTestResult {
                                    success: false,
                                    reason: Some(error_string),
                                    counterexample: None,
                                    foreign_call_failure: false,
                                }
                            }
                        }
                        println!("Found new feature just with brillig!");
                    }
                }
            }
            // If we've found something, return
            if let Some(result) = potential_res {
                break result;
            }
            if time_tracker.elapsed().as_secs() >= 1 {
                // Update and display metrics
                self.metrics.set_active_corpus_size(corpus.get_testcase_count());
                self.metrics.set_last_round_size(current_round_size);
                self.metrics.set_last_round_update_time(updating_time);
                self.metrics.set_last_round_execution_time(fuzz_time_micros);
                display_metrics(&self.metrics);
                time_tracker = Instant::now();
            }
            // We have now definitely processed the starting corpus
            processed_starting_corpus = true;
        };

        // Parse the execution result and convert it to the FuzzTestResult
        match fuzz_res {
            FuzzOutcome::Case(_) => FuzzTestResult {
                success: true,
                reason: None,
                counterexample: None,
                foreign_call_failure: false,
            },
            FuzzOutcome::Discrepancy(DiscrepancyOutcome {
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
                let reason = if reason.is_empty() { None } else { Some(reason) };

                FuzzTestResult {
                    success: false,
                    reason,
                    counterexample: Some(counterexample.clone()),
                    foreign_call_failure: false,
                }
            }
            FuzzOutcome::CounterExample(CounterExampleOutcome {
                case_id: _,
                exit_reason: status,
                counterexample,
            }) => {
                let reason = status.to_string();
                let reason = if reason.is_empty() { None } else { Some(reason) };

                FuzzTestResult {
                    success: false,
                    reason,
                    counterexample: Some(counterexample.clone()),
                    foreign_call_failure: false,
                }
            }
            FuzzOutcome::ForeignCallFailure(foreign_call_error_in_fuzzing) => FuzzTestResult {
                success: false,
                foreign_call_failure: true,
                reason: Some(foreign_call_error_in_fuzzing.exit_reason.to_string()),
                counterexample: None,
            },
        }
    }

    /// Execute acir and brillig programs with the following Testcase
    pub fn single_fuzz_acir_and_brillig(&self, testcase: &TestCase) -> FuzzOutcome {
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
                FuzzOutcome::Case(SuccessfulCaseOutcome {
                    case_id: testcase.id(),
                    case: testcase.value().clone(),
                    witness: Some(witnesses),
                    brillig_coverage: Some(brillig_coverage.unwrap()),
                    acir_time: acir_elapsed.as_micros(),
                    brillig_time: brillig_elapsed.as_micros(),
                })
            }
            // If results diverge, it's a discrepancy
            (Err(err), Ok(_)) => FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                case_id: testcase.id(),
                exit_reason: err,
                acir_failed: true,
                counterexample: testcase.value().clone(),
            }),
            (Ok(_), Err((err, _))) => FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                case_id: testcase.id(),
                exit_reason: err,
                acir_failed: false,
                counterexample: testcase.value().clone(),
            }),
            // If both failed, then we need to check
            (Err(..), Err((err, coverage))) => {
                // If this is a foreign call failure, we need to inform the user
                if err.contains(FOREIGN_CALL_FAILURE_SUBSTRING) {
                    return FuzzOutcome::ForeignCallFailure(types::ForeignCallErrorInFuzzing {
                        exit_reason: err,
                    });
                }
                // If failures are expected and this is not the failure that we are looking for, then don't treat as failure
                if self.fail_on_specific_asserts
                    && !err.contains(
                        self.failure_reason.as_ref().expect("Failure reason should be provided"),
                    )
                {
                    return FuzzOutcome::Case(SuccessfulCaseOutcome {
                        case_id: testcase.id(),
                        case: testcase.value().clone(),
                        witness: None,
                        brillig_coverage: coverage,
                        acir_time: acir_elapsed.as_micros(),
                        brillig_time: brillig_elapsed.as_micros(),
                    });
                }

                // This is a bug, inform the user
                FuzzOutcome::CounterExample(CounterExampleOutcome {
                    case_id: testcase.id(),
                    exit_reason: err,
                    counterexample: testcase.value().clone(),
                })
            }
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz_acir(&self, testcase: &TestCase) -> FuzzOutcome {
        let initial_witness = self.acir_program.abi.encode(testcase.value(), None).unwrap();
        let acir_start = Instant::now();
        let result_acir = (self.acir_executor)(&self.acir_program.bytecode, initial_witness);
        let acir_elapsed = acir_start.elapsed();

        match result_acir {
            Ok(witnesses) => FuzzOutcome::Case(SuccessfulCaseOutcome {
                case_id: testcase.id(),
                case: testcase.value().clone(),
                witness: Some(witnesses),
                brillig_coverage: None,
                acir_time: acir_elapsed.as_micros(),
                brillig_time: 0,
            }),
            Err(err) => {
                if err.contains(FOREIGN_CALL_FAILURE_SUBSTRING) {
                    return FuzzOutcome::ForeignCallFailure(types::ForeignCallErrorInFuzzing {
                        exit_reason: err,
                    });
                }
                if self.fail_on_specific_asserts
                    && !err.contains(
                        self.failure_reason.as_ref().expect("Failure reason should be provided"),
                    )
                {
                    // TODO: in the future we can add partial witness propagation from ACIR
                    return FuzzOutcome::Case(SuccessfulCaseOutcome {
                        case_id: testcase.id(),
                        case: testcase.value().clone(),
                        witness: None,
                        brillig_coverage: None,
                        acir_time: acir_elapsed.as_micros(),
                        brillig_time: 0,
                    });
                }
                FuzzOutcome::CounterExample(CounterExampleOutcome {
                    case_id: testcase.id(),
                    exit_reason: err,
                    counterexample: testcase.value().clone(),
                })
            }
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz_brillig(&self, testcase: &TestCase) -> FuzzOutcome {
        let initial_witness = self.acir_program.abi.encode(testcase.value(), None).unwrap();
        let brillig_start = Instant::now();
        let result_brillig = (self.brillig_executor)(
            &self.brillig_program.bytecode,
            initial_witness,
            &self.location_to_feature_map,
        );
        let brillig_elapsed = brillig_start.elapsed();

        match result_brillig {
            Ok((_, brillig_coverage)) => FuzzOutcome::Case(SuccessfulCaseOutcome {
                case_id: testcase.id(),
                case: testcase.value().clone(),
                witness: None,
                brillig_coverage: Some(brillig_coverage.unwrap()),
                acir_time: 0,
                brillig_time: brillig_elapsed.as_micros(),
            }),
            Err((err, coverage)) => {
                if err.contains(FOREIGN_CALL_FAILURE_SUBSTRING) {
                    return FuzzOutcome::ForeignCallFailure(types::ForeignCallErrorInFuzzing {
                        exit_reason: err,
                    });
                }
                if self.fail_on_specific_asserts {
                    if !err.contains(
                        self.failure_reason.as_ref().expect("Failure reason should be provided"),
                    ) {
                        return FuzzOutcome::Case(SuccessfulCaseOutcome {
                            case_id: testcase.id(),
                            case: testcase.value().clone(),
                            witness: None,
                            brillig_coverage: Some(coverage.unwrap()),
                            acir_time: 0,
                            brillig_time: brillig_elapsed.as_micros(),
                        });
                    }
                }
                FuzzOutcome::CounterExample(CounterExampleOutcome {
                    case_id: testcase.id(),
                    exit_reason: err,
                    counterexample: testcase.value().clone(),
                })
            }
        }
    }
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
    write!(writer, "iterations: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_count(metrics.processed_testcase_count))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", corpus_size: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_count(metrics.active_corpus_size))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", acir_time: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_time(metrics.total_acir_execution_time))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", brlg_time: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_time(metrics.total_brillig_execution_time))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", mut_time: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_time(metrics.total_mutation_time))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", rnd_count: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_count(metrics.last_round_size)).expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", r_exec_time: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_time(metrics.last_round_execution_time))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", upd_time: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    write!(writer, "{}", format_time(metrics.last_round_update_time))
        .expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");

    write!(writer, ", threads: ").expect("Failed to write to stderr");
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
    writeln!(writer, "{}", metrics.num_threads).expect("Failed to write to stderr");
    writer.reset().expect("Failed to reset writer");
    writer.flush().expect("Failed to flush writer");
}
