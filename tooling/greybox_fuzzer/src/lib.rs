use std::{cmp::max, time::Instant};

use acvm::{
    acir::{
        circuit::Program,
        native_types::{WitnessMap, WitnessStack},
    },
    FieldElement,
};
use coverage::{
    analyze_brillig_program_before_fuzzing, AccumulatedFuzzerCoverage, BranchToFeatureMap,
    BrilligCoverageRanges, PotentialBoolWitnessList, SingleTestCaseCoverage,
};
use noir_fuzzer::dictionary::build_dictionary_from_program;

mod corpus;
mod coverage;
mod strategies;
mod types;

use corpus::{Corpus, TestCase, TestCaseId};
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    ThreadPool,
};
use strategies::InputMutator;
use types::{
    CounterExampleOutcome, DiscrepancyOutcome, FuzzOutcome, FuzzTestResult, SuccessfulCaseOutcome,
};

use noirc_artifacts::program::ProgramArtifact;
use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

use rayon::prelude::IntoParallelIterator;

#[derive(Default)]
struct Metrics {
    total_acir_execution_time: u128,
    total_brillig_execution_time: u128,
}

impl Metrics {
    pub fn update_acir(&mut self, update: &u128) {
        self.total_acir_execution_time += update;
    }
    pub fn update_brillig(&mut self, update: &u128) {
        self.total_brillig_execution_time += update;
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
    location_to_feature_map: BranchToFeatureMap,

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
                &BranchToFeatureMap,
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

    /// Execute given testcases in parallel in the given pool (both acir and brillig versions)
    fn execute_testcases_in_parallel(
        &self,
        pool: &ThreadPool,
        testcases: &Vec<TestCase>,
    ) -> Vec<FuzzOutcome> {
        pool.install(|| {
            testcases
                .par_iter()
                .map(|testcase| self.single_fuzz(testcase))
                .collect::<Vec<FuzzOutcome>>()
        })
    }

    fn parse_fuzzing_results_and_update_accumulated_coverage_and_corpus(
        &mut self,
        fuzz_results: &[FuzzOutcome],
        accumulated_coverage: &mut AccumulatedFuzzerCoverage,
        corpus: &mut Corpus,
        save_to_disk: bool,
    ) -> Result<(), FuzzTestResult> {
        for result in fuzz_results.iter() {
            // Check if the execution was successful
            match result {
                FuzzOutcome::Case(_) => {}
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

                    return Err(FuzzTestResult {
                        success: false,
                        reason,
                        counterexample: Some(counterexample.clone()),
                    });
                }
                FuzzOutcome::CounterExample(CounterExampleOutcome {
                    case_id: _,
                    exit_reason: status,
                    counterexample,
                }) => {
                    let reason = status.to_string();
                    let reason = if reason.is_empty() { None } else { Some(reason) };

                    return Err(FuzzTestResult {
                        success: false,
                        reason,
                        counterexample: Some(counterexample.clone()),
                    });
                }
            }

            // Unpack the values from a successful outcome
            let (case_id, case, witness, brillig_coverage, acir_time, brillig_time) = match result {
                FuzzOutcome::Case(SuccessfulCaseOutcome {
                    case_id,
                    case,
                    witness,
                    brillig_coverage,
                    acir_time,
                    brillig_time,
                }) => (
                    case_id,
                    case,
                    witness,
                    brillig_coverage.as_ref().unwrap(),
                    acir_time,
                    brillig_time,
                ),
                _ => panic!("Already checked this"),
            };

            self.metrics.update_acir(acir_time);
            self.metrics.update_brillig(brillig_time);

            // Update empiric boolean list from the acir execution witness only if we executed ACIR
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

            // If there is a potential boolean witness list, use that, otherwise use default
            let new_coverage = if accumulated_coverage.potential_bool_witness_list.is_some() {
                SingleTestCaseCoverage::new(
                    *case_id,
                    witness,
                    brillig_coverage.clone(),
                    accumulated_coverage.potential_bool_witness_list.as_mut().unwrap(),
                )
            } else {
                SingleTestCaseCoverage::new(
                    *case_id,
                    witness,
                    brillig_coverage.clone(),
                    &PotentialBoolWitnessList::default(),
                )
            };
            let (new_coverage_discovered, testcases_to_remove) =
                accumulated_coverage.merge(&new_coverage);
            if new_coverage_discovered {
                // Remove testcases whose features are covered by new testcases
                for &testcase_for_removal in testcases_to_remove.iter() {
                    corpus.remove(testcase_for_removal);
                }
                // Import values from the new testcase
                self.mutator.update_dictionary(case);

                match corpus.insert(*case_id, case.clone(), save_to_disk) {
                    Ok(_) => (),
                    Err(error_string) => {
                        return Err(FuzzTestResult {
                            success: false,
                            reason: Some(error_string),
                            counterexample: None,
                        })
                    }
                }

                println!("New feature in loaded testcase");
            }
        }
        Ok(())
    }

    /// Start the fuzzing campaign
    pub fn fuzz(&mut self) -> FuzzTestResult {
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
                };
            }
        }

        // Init accumulated coverage object for tracking explored states
        let mut accumulated_coverage =
            AccumulatedFuzzerCoverage::new(&self.brillig_coverage_ranges);

        let mut starting_corpus = corpus.get_full_stored_corpus();

        println!("Starting corpus size: {}", starting_corpus.len());

        let mut only_default_input = false;

        // Generate the default input (it is needed if the corpus is empty)
        let default_map = self.mutator.generate_default_input_map();
        if starting_corpus.is_empty() {
            only_default_input = true;
            starting_corpus.push(TestCase::from(&default_map));
        }

        // Initialize the pool we'll be using for parallelising fuzzing
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .stack_size(4 * 1024 * 1024)
            .build()
            .unwrap();

        let fuzzing_results: Vec<FuzzOutcome> =
            self.execute_testcases_in_parallel(&pool, &starting_corpus);

        let mut total_acir_time = 0u128;
        let mut total_brillig_time = 0u128;

        let initial_corpus_result = self
            .parse_fuzzing_results_and_update_accumulated_coverage_and_corpus(
                &fuzzing_results,
                &mut accumulated_coverage,
                &mut corpus,
                only_default_input,
            );
        match initial_corpus_result {
            Ok(_) => {}
            Err(fuzz_outcome) => {
                return fuzz_outcome;
            }
        }
        let mut current_iteration = 0;

        let testcases_per_iteration = self.num_threads * 2;
        let mut time_tracker = Instant::now();
        let mut brillig_executions_multiplier = 1usize;
        let mut acir_executions_multiplier = 1usize;
        let fuzz_res = loop {
            let mut testcase_set: Vec<(
                TestCaseId,
                Option<TestCaseId>,
                <XorShiftRng as SeedableRng>::Seed,
            )> = Vec::new();
            let acir_round = total_acir_time < total_brillig_time;
            let current_testcase_set_size = if acir_round {
                acir_executions_multiplier * testcases_per_iteration
            } else {
                brillig_executions_multiplier * testcases_per_iteration
            };
            testcase_set.reserve(current_testcase_set_size);
            let testcase_generation_tracker = Instant::now();

            // Get indices of testcases from the corpus that will be used in the current round of mutations
            for _ in 0..current_testcase_set_size {
                let (main_testcase, additional_testcase) = if acir_round {
                    corpus.get_next_testcase_for_acir(&mut prng)
                } else {
                    corpus.get_next_testcase_for_brillig(&mut prng)
                };

                // Generate seeds for use by individual threads
                let mut seed_bytes: <XorShiftRng as SeedableRng>::Seed = [0; 16];
                prng.fill_bytes(&mut seed_bytes);

                testcase_set.push((main_testcase, additional_testcase, seed_bytes));
            }
            let testcase_generation_time = testcase_generation_tracker.elapsed().as_micros();

            let mutation_and_fuzzing_time_tracker = Instant::now();

            let all_fuzzing_results: Vec<(FuzzOutcome, bool, u128, u128)> = pool.install(|| {
                testcase_set
                    .into_par_iter()
                    .map(|(main_testcase_index, additional_testcase_index, thread_seed)| {
                        // Initialize a prng from per-thread seed
                        let mut thread_prng = XorShiftRng::from_seed(thread_seed);
                        // Generate a mutated input by using the main and additional testcases in the corpus
                        let input = self.mutator.mutate_input_map_multiple(
                            corpus.get_testcase_by_id(main_testcase_index).clone(),
                            additional_testcase_index.map(|additional_testcase_index| {
                                corpus.get_testcase_by_id(additional_testcase_index).clone()
                            }),
                            &mut thread_prng,
                        );
                        // Form a testcase from input (assign a unique id)
                        let testcase = TestCase::from(&input);

                        if acir_round {
                            // If the round uses ACIR, run both ACIR and brillig execution
                            let paired_fuzz_outcome = self.single_fuzz(&testcase);

                            if let FuzzOutcome::Case(SuccessfulCaseOutcome {
                                case_id,
                                case,
                                witness,
                                brillig_coverage,
                                acir_time,
                                brillig_time,
                            }) = paired_fuzz_outcome
                            {
                                let mut default_list = PotentialBoolWitnessList::default();
                                let mut bool_witness_list = accumulated_coverage
                                    .potential_bool_witness_list
                                    .as_ref()
                                    .unwrap_or(&default_list);

                                if witness.is_some() {
                                    default_list = accumulated_coverage
                                        .potential_bool_witness_list
                                        .as_ref()
                                        .unwrap()
                                        .merge_new(witness.as_ref().unwrap());
                                    bool_witness_list = &default_list;
                                }

                                let new_coverage = SingleTestCaseCoverage::new(
                                    case_id,
                                    &witness,
                                    brillig_coverage.clone().unwrap(),
                                    bool_witness_list,
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
                                    !accumulated_coverage.detect_new_coverage(&new_coverage),
                                    acir_time,
                                    brillig_time,
                                )
                            } else {
                                (paired_fuzz_outcome, false, 0, 0)
                            }
                        } else {
                            let brillig_fuzz_outcome = self.single_fuzz_brillig(&testcase);
                            if let FuzzOutcome::Case(SuccessfulCaseOutcome {
                                case_id,
                                case,
                                witness,
                                brillig_coverage,
                                acir_time,
                                brillig_time,
                            }) = brillig_fuzz_outcome
                            {
                                let new_coverage = SingleTestCaseCoverage::new(
                                    case_id,
                                    &None,
                                    brillig_coverage.clone().unwrap(),
                                    &PotentialBoolWitnessList::default(),
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
                                    !accumulated_coverage.detect_new_coverage(&new_coverage),
                                    acir_time,
                                    brillig_time,
                                )
                            } else {
                                (brillig_fuzz_outcome, false, 0, 0)
                            }
                        }
                    })
                    .collect::<Vec<(FuzzOutcome, bool, u128, u128)>>()
            });
            let fuzz_time_micros = mutation_and_fuzzing_time_tracker.elapsed().as_micros();
            if acir_round {
                let mut time_per_testcase = fuzz_time_micros / acir_executions_multiplier as u128;
                time_per_testcase = max(time_per_testcase, 30);
                acir_executions_multiplier = (200_000u128 / time_per_testcase) as usize;
                if acir_executions_multiplier == 0 {
                    acir_executions_multiplier = 1;
                }
            } else {
                let mut time_per_testcase =
                    fuzz_time_micros / brillig_executions_multiplier as u128;
                time_per_testcase = max(time_per_testcase, 30);
                brillig_executions_multiplier = (200_000u128 / time_per_testcase) as usize;
                if brillig_executions_multiplier == 0 {
                    brillig_executions_multiplier = 1;
                }
            }

            let mut potential_res = None;
            let mut acir_cases_to_execute = Vec::new();
            let updating_time_tracker = Instant::now();
            let mut skipped = 0usize;
            for (_, _, acir_time_f, brillig_time_f) in
                all_fuzzing_results.iter().filter(|(_, should_skip_check, _, _)| *should_skip_check)
            {
                skipped += 1;
                total_acir_time += acir_time_f;
                total_brillig_time += brillig_time_f;
            }
            let mut results_to_analyze = Vec::new();
            for (index, _) in all_fuzzing_results
                .iter()
                .enumerate()
                .filter(|(_, (_, should_skip_check, _, _))| !*should_skip_check)
            {
                results_to_analyze.push(index);
            }
            for index in results_to_analyze.into_iter() {
                let fuzz_res = all_fuzzing_results[index].0.clone();
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
                // In case we fuzzed both ACIR and brillig
                if let Some(brillig_coverage) = brillig_coverage.clone() {
                    total_acir_time += acir_time;
                    total_acir_time += brillig_time;

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
                        accumulated_coverage
                            .potential_bool_witness_list
                            .as_ref()
                            .unwrap_or(&PotentialBoolWitnessList::default()),
                    );
                    let (new_coverage_discovered, testcases_to_remove) =
                        accumulated_coverage.merge(&new_coverage);
                    if new_coverage_discovered {
                        for &testcase_for_removal in testcases_to_remove.iter() {
                            println!("Removing {testcase_for_removal}");
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
                                }
                            }
                        }
                        println!("Found new feature!");
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
                    } else {
                        total_brillig_time += brillig_time;
                    }
                }
            }
            let updating_time = updating_time_tracker.elapsed().as_micros();
            if time_tracker.elapsed().as_secs() >= 1 {
                let format_time = |x: u128| {
                    let microseconds_in_second = 1_000_000;
                    let microseconds_in_millisecond = 1_000;
                    let microseconds_in_minutes = 60_000_000;
                    if x > microseconds_in_minutes {
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
                println!(
                    "iterations: {}, corpus size:{}, acir_time: {}, brillig_time: {}, tc_gen_time:{}, count:{}, fuzzing_time:{}, upd_time: {}, skipped: {}, threads: {}",
                    format_count(current_iteration),corpus.get_testcase_count(), format_time(total_acir_time), format_time(total_brillig_time), format_time(testcase_generation_time),format_count(current_testcase_set_size), format_time(fuzz_time_micros),format_time(updating_time),
                format_count(skipped),self.num_threads

                );
                time_tracker = Instant::now();
            }
            if let Some(result) = potential_res {
                break result;
            }
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
                        }
                    })
                    .collect::<Vec<FuzzOutcome>>()
            });
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
                if let Some(brillig_coverage) = brillig_coverage {
                    total_acir_time += acir_time;
                    total_acir_time += brillig_time;
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
                            println!("Removing {testcase_for_removal}");
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
                                }
                            }
                        }
                        println!("Found new feature just with brillig!");
                    }
                }
            }
            if let Some(result) = potential_res {
                break result;
            }
            current_iteration += current_testcase_set_size;
        };
        println!("Total iterations: {current_iteration}");
        match fuzz_res {
            FuzzOutcome::Case(_) => {
                FuzzTestResult { success: true, reason: None, counterexample: None }
            }
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
                }
            }
        }
    }

    // fn check_failure_message(&self,)
    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz(&self, testcase: &TestCase) -> FuzzOutcome {
        let initial_witness = self.acir_program.abi.encode(testcase.value(), None).unwrap();
        let initial_witness2 = self.acir_program.abi.encode(testcase.value(), None).unwrap();
        let acir_start = Instant::now();
        let result_acir = (self.acir_executor)(&self.acir_program.bytecode, initial_witness);
        let acir_elapsed = acir_start.elapsed();
        let brillig_start = Instant::now();
        let result_brillig = (self.brillig_executor)(
            &self.brillig_program.bytecode,
            initial_witness2,
            &self.location_to_feature_map,
        );
        let brillig_elapsed = brillig_start.elapsed();
        //println!("Acir: {}, brillig: {}", acir_elapsed.as_micros(), brillig_elapsed.as_micros());

        // TODO: Add handling for `vm.assume` equivalent

        match (result_acir, result_brillig) {
            (Ok(witnesses), Ok((_map, brillig_coverage))) => {
                FuzzOutcome::Case(SuccessfulCaseOutcome {
                    case_id: testcase.id(),
                    case: testcase.value().clone(),
                    witness: Some(witnesses),
                    brillig_coverage: Some(brillig_coverage.unwrap()),
                    acir_time: acir_elapsed.as_micros(),
                    brillig_time: brillig_elapsed.as_micros(),
                })
            }
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
            (Err(..), Err((err, coverage))) => {
                if self.fail_on_specific_asserts
                    && !err.contains(
                        self.failure_reason.as_ref().expect("Failure reason should be provided"),
                    )
                {
                    return FuzzOutcome::Case(SuccessfulCaseOutcome {
                        case_id: testcase.id(),
                        case: testcase.value().clone(),
                        witness: None,
                        brillig_coverage: Some(coverage.unwrap()),
                        acir_time: acir_elapsed.as_micros(),
                        brillig_time: brillig_elapsed.as_micros(),
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
            Err(err) => FuzzOutcome::CounterExample(CounterExampleOutcome {
                case_id: testcase.id(),
                exit_reason: err,
                counterexample: testcase.value().clone(),
            }),
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
