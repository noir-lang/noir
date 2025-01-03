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
use noirc_abi::InputMap;

mod corpus;
mod coverage;
mod strategies;
mod types;

use corpus::{Corpus, TestCase, TestCaseId};
use rayon::{
    iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator},
    ThreadPool,
};
use strategies::InputMutator;
use types::{CaseOutcome, CounterExampleOutcome, DiscrepancyOutcome, FuzzOutcome, FuzzTestResult};

use noirc_artifacts::program::ProgramArtifact;
use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

use rayon::prelude::IntoParallelIterator;
/// An executor for Noir programs which which provides fuzzing support
///
/// After instantiation, calling `fuzz` will proceed to hammer the program with
/// inputs, until it finds a counterexample. The provided [`TestRunner`] contains all the
/// configuration which can be overridden via [environment variables](proptest::test_runner::Config)
pub struct FuzzedExecutor<E, F> {
    /// The program to be fuzzed (acir version)
    acir_program: ProgramArtifact,

    /// The program to be fuzzed (brillig version)
    brillig_program: ProgramArtifact,

    /// A function which executes the programs with a given set of inputs
    acir_executor: E,

    /// A function which executes the programs with a given set of inputs
    brillig_executor: F,

    /// Location to feature map
    location_to_feature_map: BranchToFeatureMap,

    /// Brillig coverage ranges (which are branch coverage and which are comparison coverage)
    brillig_coverage_ranges: BrilligCoverageRanges,

    /// Mutator
    mutator: InputMutator,

    /// Package name
    package_name: String,

    /// Function name
    function_name: String,

    /// Number of threads to use
    num_threads: usize,

    /// Fail on specific asserts
    fail_on_specific_asserts: bool,

    /// Failure reason
    failure_reason: Option<String>,
}
type BrilligCoverage = Vec<u32>;

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
        let (location_to_feature_map, brillig_coverage_ranges) =
            analyze_brillig_program_before_fuzzing(&brillig_program);
        let dictionary = build_dictionary_from_program(&acir_program.bytecode);
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
        }
    }

    fn execute_testcases_in_parallel_without_quick_coverage_check(
        &self,
        pool: &ThreadPool,
        testcases: &Vec<TestCase>,
    ) -> Vec<(TestCaseId, FuzzOutcome)> {
        pool.install(|| {
            testcases
                .par_iter()
                .map(|testcase| (testcase.id(), self.single_fuzz(testcase).unwrap()))
                .collect::<Vec<(TestCaseId, FuzzOutcome)>>()
        })
    }

    fn parse_fuzzing_results_and_update_accumulated_coverage_and_corpus(
        &mut self,
        fuzz_results: &Vec<FuzzOutcome>,
        accumulated_coverage: &mut AccumulatedFuzzerCoverage,
        corpus: &mut Corpus,
        save_to_disk: bool,
    ) -> Result<(u128, u128), FuzzTestResult> {
        let mut total_acir_time = 0;
        let mut total_brillig_time = 0;
        for result in fuzz_results.iter() {
            match result {
                FuzzOutcome::Case(_) => {}
                FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                    exit_reason: status,
                    acir_failed,
                    counterexample,
                }) => {
                    let reason = match acir_failed {
                        true => format!(
                            "ACIR failed while brillig executed with no issues: {}",
                            status.to_string()
                        ),

                        false => format!(
                            "brillig failed while ACIR executed with no issues: {}",
                            status.to_string()
                        ),
                    };
                    let reason = if reason.is_empty() { None } else { Some(reason) };

                    return Err(FuzzTestResult {
                        success: false,
                        reason,
                        counterexample: Some(counterexample.clone()),
                    });
                }
                FuzzOutcome::CounterExample(CounterExampleOutcome {
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
            let (case_id, case, witness, brillig_coverage, acir_time, brillig_time) = match result {
                FuzzOutcome::Case(CaseOutcome {
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
            total_acir_time += acir_time;
            total_brillig_time += brillig_time;

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
            let new_coverage = if accumulated_coverage.potential_bool_witness_list.is_some() {
                SingleTestCaseCoverage::new(
                    *case_id,
                    witness,
                    brillig_coverage.clone(),
                    &accumulated_coverage.potential_bool_witness_list.as_mut().unwrap(),
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
                accumulated_coverage.merge(&&new_coverage);
            if new_coverage_discovered {
                for &testcase_for_removal in testcases_to_remove.iter() {
                    corpus.remove(testcase_for_removal);
                }
                println!("Input: {:?}", case);
                self.mutator.update_dictionary(&case);
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
        return Ok((total_acir_time, total_brillig_time));
    }

    /// Fuzzes the provided program.
    pub fn fuzz(&mut self, only_fail_with: Option<String>) -> FuzzTestResult {
        // Generate a seed for the campaign

        let seed = thread_rng().gen::<u64>();
        println!("Fuzzing seed for this campaign: {}", seed);

        let mut prng = XorShiftRng::seed_from_u64(seed);
        let mut corpus =
            Corpus::new(&self.package_name, &self.function_name, &self.acir_program.abi);
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
        let mut accumulated_coverage = AccumulatedFuzzerCoverage::new(
            self.location_to_feature_map.len(),
            &self.brillig_coverage_ranges,
        );

        let mut starting_corpus = corpus.get_full_stored_corpus();
        println!("Starting corpus size: {}", starting_corpus.len());
        let mut only_default_input = false;
        let default_map = self.mutator.generate_default_input_map();
        if starting_corpus.is_empty() {
            only_default_input = true;
            starting_corpus.push(TestCase::from(&default_map));
        }

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .stack_size(4 * 1024 * 1024)
            .build()
            .unwrap();
        let fuzzing_results: Vec<FuzzOutcome> = self
            .execute_testcases_in_parallel_without_quick_coverage_check(&pool, &starting_corpus)
            .into_iter()
            .map(|(_, outcome)| outcome)
            .collect();
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
            Ok((acir_time, brillig_time)) => {
                total_acir_time = acir_time;
                total_brillig_time = brillig_time
            }
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
                usize,
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
            let testcase_gen_time = Instant::now();
            for i in 0..current_testcase_set_size {
                let (main_testcase, additional_testcase) = if acir_round {
                    corpus.get_next_testcase_for_acir(&mut prng)
                } else {
                    corpus.get_next_testcase_for_brillig(&mut prng)
                };
                let mut seed_bytes: <XorShiftRng as SeedableRng>::Seed = [0; 16];
                prng.fill_bytes(&mut seed_bytes);

                testcase_set.push((i, main_testcase, additional_testcase, seed_bytes));
            }
            let testcase_time = testcase_gen_time.elapsed().as_micros();

            let fuzzing_time = Instant::now();
            let all_fuzzing_results: Vec<(FuzzOutcome, bool, u128, u128)> = pool.install(|| {
                testcase_set
                    .into_par_iter()
                    .map(|(_index, main_testcase_index, additional_testcase_index, thread_seed)| {
                        let mut thread_prng = XorShiftRng::from_seed(thread_seed);
                        let input = self.mutator.mutate_input_map_multiple(
                            corpus.get_testcase_by_id(main_testcase_index).clone(),
                            match additional_testcase_index {
                                Some(additional_testcase_index) => Some(
                                    corpus.get_testcase_by_id(additional_testcase_index).clone(),
                                ),
                                None => None,
                            },
                            &mut thread_prng,
                        );
                        let testcase = TestCase::from(&input);
                        if total_acir_time < total_brillig_time {
                            let paired_fuzz_outcome = self.single_fuzz(&testcase).unwrap();
                            if let FuzzOutcome::Case(CaseOutcome {
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
                                    .unwrap();
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
                                    &bool_witness_list,
                                );
                                (
                                    FuzzOutcome::Case(CaseOutcome {
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
                            let brillig_fuzz_outcome = self.single_fuzz_brillig(&testcase).unwrap();
                            if let FuzzOutcome::Case(CaseOutcome {
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
                                    FuzzOutcome::Case(CaseOutcome {
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
            let fuzz_time_micros = fuzzing_time.elapsed().as_micros();
            if (acir_round) {
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
                        FuzzOutcome::Case(CaseOutcome {
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
                        &accumulated_coverage.potential_bool_witness_list.as_mut().unwrap(),
                    );
                    let (new_coverage_discovered, testcases_to_remove) =
                        accumulated_coverage.merge(&&new_coverage);
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
                    if accumulated_coverage.detect_new_coverage(&&new_coverage) {
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
                    format_count(current_iteration),corpus.get_testcase_count(), format_time(total_acir_time), format_time(total_brillig_time), format_time(testcase_time),format_count(current_testcase_set_size), format_time(fuzz_time_micros),format_time(updating_time),
                format_count(skipped),self.num_threads

                );
                time_tracker = Instant::now();
            }
            if potential_res.is_some() {
                break potential_res.unwrap();
            }
            let all_fuzzing_results: Vec<FuzzOutcome> = pool.install(|| {
                acir_cases_to_execute
                    .into_par_iter()
                    .map(|(case_id, input, brillig_coverage, brillig_time)| {
                        let testcase = TestCase::with_id(case_id, &input);
                        let fuzz_res = self.single_fuzz_acir(&testcase).unwrap();
                        match fuzz_res {
                            FuzzOutcome::Case(CaseOutcome {
                                case_id,
                                case,
                                witness,
                                brillig_coverage: _,
                                acir_time,
                                brillig_time: _,
                            }) => FuzzOutcome::Case(CaseOutcome {
                                case_id,
                                case,
                                witness,
                                brillig_coverage: Some(brillig_coverage),
                                acir_time,
                                brillig_time: brillig_time,
                            }),
                            FuzzOutcome::Discrepancy(..) => {
                                panic!("Can't get a discrepancy just from acir")
                            }
                            FuzzOutcome::CounterExample(CounterExampleOutcome {
                                counterexample,
                                exit_reason,
                            }) => FuzzOutcome::Discrepancy(DiscrepancyOutcome {
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
                        FuzzOutcome::Case(CaseOutcome {
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
                        &accumulated_coverage.potential_bool_witness_list.as_mut().unwrap(),
                    );
                    let (new_coverage_discovered, testcases_to_remove) =
                        accumulated_coverage.merge(&&new_coverage);
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
            if potential_res.is_some() {
                break potential_res.unwrap();
            }
            current_iteration += current_testcase_set_size;
        };
        println!("Total iterations: {current_iteration}");
        match fuzz_res {
            FuzzOutcome::Case(_) => {
                FuzzTestResult { success: true, reason: None, counterexample: None }
            }
            FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                exit_reason: status,
                acir_failed,
                counterexample,
            }) => {
                let reason = match acir_failed {
                    true => format!(
                        "ACIR failed while brillig executed with no issues: {}",
                        status.to_string()
                    ),
                    false => format!(
                        "brillig failed while ACIR executed with no issues: {}",
                        status.to_string()
                    ),
                };
                let reason = if reason.is_empty() { None } else { Some(reason) };

                FuzzTestResult {
                    success: false,
                    reason,
                    counterexample: Some(counterexample.clone()),
                }
            }
            FuzzOutcome::CounterExample(CounterExampleOutcome {
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
    pub fn single_fuzz(&self, testcase: &TestCase) -> Result<FuzzOutcome, ()> {
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
            (Ok(witnesses), Ok((_map, brillig_coverage))) => Ok(FuzzOutcome::Case(CaseOutcome {
                case_id: testcase.id(),
                case: testcase.value().clone(),
                witness: Some(witnesses),
                brillig_coverage: Some(brillig_coverage.unwrap()),
                acir_time: acir_elapsed.as_micros(),
                brillig_time: brillig_elapsed.as_micros(),
            })),
            (Err(err), Ok(_)) => Ok(FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                exit_reason: err,
                acir_failed: true,
                counterexample: testcase.value().clone(),
            })),
            (Ok(_), Err((err, coverage))) => Ok(FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                exit_reason: err,
                acir_failed: false,
                counterexample: testcase.value().clone(),
            })),
            (Err(..), Err((err, coverage))) => {
                if (self.fail_on_specific_asserts) {
                    if (!err.contains(
                        self.failure_reason.as_ref().expect("Failure reason should be provided"),
                    )) {
                        return Ok(FuzzOutcome::Case(CaseOutcome {
                            case_id: testcase.id(),
                            case: testcase.value().clone(),
                            witness: None,
                            brillig_coverage: Some(coverage.unwrap()),
                            acir_time: acir_elapsed.as_micros(),
                            brillig_time: brillig_elapsed.as_micros(),
                        }));
                    }
                }
                Ok(FuzzOutcome::CounterExample(CounterExampleOutcome {
                    exit_reason: err,
                    counterexample: testcase.value().clone(),
                }))
            }
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz_acir(&self, testcase: &TestCase) -> Result<FuzzOutcome, ()> {
        let initial_witness = self.acir_program.abi.encode(testcase.value(), None).unwrap();
        let acir_start = Instant::now();
        let result_acir = (self.acir_executor)(&self.acir_program.bytecode, initial_witness);
        let acir_elapsed = acir_start.elapsed();

        match result_acir {
            Ok(witnesses) => Ok(FuzzOutcome::Case(CaseOutcome {
                case_id: testcase.id(),
                case: testcase.value().clone(),
                witness: Some(witnesses),
                brillig_coverage: None,
                acir_time: acir_elapsed.as_micros(),
                brillig_time: 0,
            })),
            Err(err) => Ok(FuzzOutcome::CounterExample(CounterExampleOutcome {
                exit_reason: err,
                counterexample: testcase.value().clone(),
            })),
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz_brillig(&self, testcase: &TestCase) -> Result<FuzzOutcome, ()> {
        let initial_witness = self.acir_program.abi.encode(testcase.value(), None).unwrap();
        let brillig_start = Instant::now();
        let result_brillig = (self.brillig_executor)(
            &self.brillig_program.bytecode,
            initial_witness,
            &self.location_to_feature_map,
        );
        let brillig_elapsed = brillig_start.elapsed();

        match result_brillig {
            Ok((_, brillig_coverage)) => Ok(FuzzOutcome::Case(CaseOutcome {
                case_id: testcase.id(),
                case: testcase.value().clone(),
                witness: None,
                brillig_coverage: Some(brillig_coverage.unwrap()),
                acir_time: 0,
                brillig_time: brillig_elapsed.as_micros(),
            })),
            Err((err, coverage)) => {
                if (self.fail_on_specific_asserts) {
                    if (!err.contains(
                        self.failure_reason.as_ref().expect("Failure reason should be provided"),
                    )) {
                        return Ok(FuzzOutcome::Case(CaseOutcome {
                            case_id: testcase.id(),
                            case: testcase.value().clone(),
                            witness: None,
                            brillig_coverage: Some(coverage.unwrap()),
                            acir_time: 0,
                            brillig_time: brillig_elapsed.as_micros(),
                        }));
                    }
                }
                Ok(FuzzOutcome::CounterExample(CounterExampleOutcome {
                    exit_reason: err,
                    counterexample: testcase.value().clone(),
                }))
            }
        }
    }
}
