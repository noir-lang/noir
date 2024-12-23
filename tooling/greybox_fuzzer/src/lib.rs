use std::{cmp::max, collections::HashSet, time::Instant};

use acvm::{
    acir::{
        brillig,
        circuit::Program,
        native_types::{WitnessMap, WitnessStack},
    },
    FieldElement,
};
use coverage::{
    analyze_brillig_program_before_fuzzing, AccumulatedFuzzerCoverage, BranchToFeatureMap,
    PotentialBoolWitnessList, SingleTestCaseCoverage,
};
use noir_fuzzer::dictionary::build_dictionary_from_program;
use noirc_abi::{
    input_parser::json::{parse_json, serialize_to_json},
    InputMap,
};

mod corpus;
mod coverage;
mod strategies;
mod types;

use corpus::Corpus;
use rayon::{
    current_num_threads,
    iter::{FromParallelIterator, ParallelBridge, ParallelIterator},
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

    /// Mutator
    mutator: InputMutator,

    /// Package name
    package_name: String,

    /// Function name
    function_name: String,

    /// Number of threads to use
    num_threads: usize,
}
type BrilligCoverage = Vec<u32>;
// impl<T: Send> FromParallelIterator<T>
//     for Vec<Result<(FuzzOutcome, Option<SingleTestCaseCoverage>), ()>>
// {
// }

// impl FromParallelIterator<Result<(FuzzOutcome, Option<SingleTestCaseCoverage>), ()>>
//     for Vec<Result<(FuzzOutcome, Option<SingleTestCaseCoverage>), ()>>
// {
//     fn from_par_iter<I>(par_iter: I) -> Self
//     where
//         I: IntoParallelIterator<Item = Result<(FuzzOutcome, Option<SingleTestCaseCoverage>), ()>>,
//     {
//         par_iter.into_par_iter().collect()
//     }
// }
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
            ) -> Result<(WitnessStack<FieldElement>, Option<Vec<u32>>), String>
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
    ) -> Self {
        let location_to_feature_map = analyze_brillig_program_before_fuzzing(&brillig_program);
        let dictionary = build_dictionary_from_program(&acir_program.bytecode);
        let mutator = InputMutator::new(&acir_program.abi, &dictionary);
        Self {
            acir_program,
            brillig_program,
            acir_executor,
            brillig_executor,
            location_to_feature_map,
            mutator,
            package_name: package_name.to_string(),
            function_name: function_name.to_string(),
            num_threads,
        }
    }

    /// Fuzzes the provided program.
    pub fn fuzz(&mut self) -> FuzzTestResult {
        // Generate a seed for the campaign

        let seed = thread_rng().gen::<u64>();
        println!("Fuzzing seed for this campaign: {}", seed);

        let mut prng = XorShiftRng::seed_from_u64(seed);
        let mut corpus =
            Corpus::new(&self.package_name, &self.function_name, &self.acir_program.abi);
        match corpus.attempt_load() {
            Ok(_) => (),
            Err(error_string) => {
                return FuzzTestResult {
                    success: false,
                    reason: Some(error_string),
                    counterexample: None,
                };
            }
        }
        let mut accumulated_coverage =
            AccumulatedFuzzerCoverage::new(self.location_to_feature_map.len());

        let mut starting_corpus = corpus.get_stored_corpus();
        println!("Starting corpus size: {}", starting_corpus.len());
        let mut only_default_input = false;
        if starting_corpus.is_empty() {
            only_default_input = true;
            starting_corpus.push(self.mutator.generate_default_input_map());
        }
        let mut total_acir_time = 0u128;
        let mut total_brillig_time = 0u128;
        for entry in starting_corpus.iter() {
            let fuzz_res = self.single_fuzz(&entry).unwrap();
            match fuzz_res {
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

                    return FuzzTestResult {
                        success: false,
                        reason,
                        counterexample: Some(counterexample),
                    };
                }
                FuzzOutcome::CounterExample(CounterExampleOutcome {
                    exit_reason: status,
                    counterexample,
                }) => {
                    let reason = status.to_string();
                    let reason = if reason.is_empty() { None } else { Some(reason) };

                    return FuzzTestResult {
                        success: false,
                        reason,
                        counterexample: Some(counterexample),
                    };
                }
            }
            let (_, witness, brillig_coverage, acir_time, brillig_time) = match fuzz_res {
                FuzzOutcome::Case(CaseOutcome {
                    case,
                    witness,
                    brillig_coverage,
                    acir_time,
                    brillig_time,
                }) => (case, witness.unwrap(), brillig_coverage.unwrap(), acir_time, brillig_time),
                _ => panic!("Already checked this"),
            };
            total_acir_time += acir_time;
            total_brillig_time += brillig_time;
            // Update the potential list
            if accumulated_coverage.potential_bool_witness_list.is_none() {
                // If it's the first time, we need to assign
                accumulated_coverage.potential_bool_witness_list =
                    Some(PotentialBoolWitnessList::from(&witness));
            } else {
                accumulated_coverage.potential_bool_witness_list.as_mut().unwrap().update(&witness);
            }
            let new_coverage = SingleTestCaseCoverage::new(
                &witness,
                brillig_coverage,
                &accumulated_coverage.potential_bool_witness_list.as_mut().unwrap(),
            );
            if accumulated_coverage.merge(&&new_coverage) {
                println!("Input: {:?}", entry);
                self.mutator.update_dictionary(&entry);
                match corpus.insert(entry.clone(), only_default_input) {
                    Ok(_) => (),
                    Err(error_string) => {
                        return FuzzTestResult {
                            success: false,
                            reason: Some(error_string),
                            counterexample: None,
                        }
                    }
                }

                println!("New feature in loaded testcase");
            }
        }
        let mut current_iteration = 0;
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .stack_size(4 * 1024 * 1024)
            .build()
            .unwrap();
        let testcases_per_iteration = self.num_threads * 2;
        let mut time_tracker = Instant::now();
        let mut brillig_executions_multiplier = 1usize;
        let mut acir_executions_multiplier = 1usize;
        let fuzz_res = loop {
            let mut testcase_set: Vec<(
                usize,
                usize,
                Option<usize>,
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
                let (main_testcase, additional_testcase) =
                    corpus.get_next_testcase_with_additional(&mut prng);
                let mut seed_bytes: <XorShiftRng as SeedableRng>::Seed = [0; 16];
                prng.fill_bytes(&mut seed_bytes);

                testcase_set.push((i, main_testcase, additional_testcase, seed_bytes));
            }
            let testcase_time = testcase_gen_time.elapsed().as_micros();

            let fuzzing_time = Instant::now();
            let all_fuzzing_results: Vec<(FuzzOutcome, bool)> = pool
                .install(|| {
                    testcase_set.clone().into_iter().par_bridge().map(
                        |(index, main_testcase_index, additional_testcase_index, thread_seed)| {
                            let mut thread_prng = XorShiftRng::from_seed(thread_seed);
                            let input = self.mutator.mutate_input_map_multiple(
                                corpus.get_testcase_by_index(main_testcase_index).clone(),
                                match additional_testcase_index {
                                    Some(additional_testcase_index) => Some(
                                        corpus
                                            .get_testcase_by_index(additional_testcase_index)
                                            .clone(),
                                    ),
                                    None => None,
                                },
                                &mut thread_prng,
                            );
                            if total_acir_time < total_brillig_time {
                                let paired_fuzz_outcome = self.single_fuzz(&input).unwrap();
                                if let FuzzOutcome::Case(CaseOutcome {
                                    case,
                                    witness,
                                    brillig_coverage,
                                    acir_time,
                                    brillig_time,
                                }) = paired_fuzz_outcome
                                {
                                    let new_coverage = SingleTestCaseCoverage::new(
                                        &witness.as_ref().unwrap(),
                                        brillig_coverage.clone().unwrap(),
                                        &accumulated_coverage
                                            .potential_bool_witness_list
                                            .as_ref()
                                            .unwrap()
                                            .merge_new(&witness.as_ref().unwrap()),
                                    );
                                    (
                                        FuzzOutcome::Case(CaseOutcome {
                                            case,
                                            witness,
                                            brillig_coverage,
                                            acir_time,
                                            brillig_time,
                                        }),
                                        !accumulated_coverage.detect_new_coverage(&new_coverage),
                                    )
                                } else {
                                    (paired_fuzz_outcome, false)
                                }
                            } else {
                                let brillig_fuzz_outcome =
                                    self.single_fuzz_brillig(&input).unwrap();
                                if let FuzzOutcome::Case(CaseOutcome {
                                    case,
                                    witness,
                                    brillig_coverage,
                                    acir_time,
                                    brillig_time,
                                }) = brillig_fuzz_outcome
                                {
                                    let new_coverage = SingleTestCaseCoverage::new(
                                        &WitnessStack::default(),
                                        brillig_coverage.clone().unwrap(),
                                        &PotentialBoolWitnessList::default(),
                                    );
                                    (
                                        FuzzOutcome::Case(CaseOutcome {
                                            case,
                                            witness,
                                            brillig_coverage,
                                            acir_time,
                                            brillig_time,
                                        }),
                                        !accumulated_coverage.detect_new_coverage(&new_coverage),
                                    )
                                } else {
                                    (brillig_fuzz_outcome, false)
                                }
                            }
                        },
                    )
                })
                .collect::<Vec<(FuzzOutcome, bool)>>();
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
            let updating_time = Instant::now();
            let mut skipped = 0usize;
            for (fuzz_res, should_skip_check) in all_fuzzing_results.into_iter() {
                if should_skip_check {
                    if let FuzzOutcome::Case(CaseOutcome {
                        case,
                        witness,
                        brillig_coverage,
                        acir_time,
                        brillig_time,
                    }) = fuzz_res
                    {
                        skipped += 1;
                        total_acir_time += acir_time;
                        total_brillig_time += brillig_time;
                        continue;
                    }
                }
                let (case, witness, brillig_coverage, acir_time, brillig_time) = match fuzz_res {
                    FuzzOutcome::Case(CaseOutcome {
                        case,
                        witness,
                        brillig_coverage,
                        acir_time,
                        brillig_time,
                    }) => (case, witness, brillig_coverage, acir_time, brillig_time),
                    _ => {
                        potential_res = Some(fuzz_res);
                        break;
                    }
                };
                // In case we fuzzed both ACIR and brillig
                if let (Some(witness), Some(brillig_coverage)) = (witness, brillig_coverage.clone())
                {
                    total_acir_time += acir_time;
                    total_acir_time += brillig_time;
                    // Update the potential list
                    if accumulated_coverage.potential_bool_witness_list.is_none() {
                        // If it's the first time, we need to assign
                        accumulated_coverage.potential_bool_witness_list =
                            Some(PotentialBoolWitnessList::from(&witness));
                    } else {
                        accumulated_coverage
                            .potential_bool_witness_list
                            .as_mut()
                            .unwrap()
                            .update(&witness);
                    }
                    let new_coverage = SingleTestCaseCoverage::new(
                        &witness,
                        brillig_coverage,
                        &accumulated_coverage.potential_bool_witness_list.as_mut().unwrap(),
                    );
                    if accumulated_coverage.merge(&&new_coverage) {
                        println!("Input: {:?}", case);
                        self.mutator.update_dictionary(&case);
                        match corpus.insert(case, true) {
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
                        &WitnessStack::default(),
                        brillig_coverage,
                        &PotentialBoolWitnessList::default(),
                    );
                    if accumulated_coverage.detect_new_coverage(&&new_coverage) {
                        println!("Detected here");
                        acir_cases_to_execute.push((
                            case,
                            new_coverage.brillig_coverage,
                            brillig_time,
                        ));
                    } else {
                        total_brillig_time += brillig_time;
                    }
                }
            }
            if time_tracker.elapsed().as_secs() >= 1 {
                println!(
                    "iterations: {}, acir_time: {}ms, brillig_time: {}ms, testcase_generation_time:{}mcrs, count:{}, fuzzing_time:{}mcrs, updating time: {}mcrs, skipped: {}",
                    current_iteration, total_acir_time/1000, total_brillig_time/1000, testcase_time,current_testcase_set_size, fuzz_time_micros,updating_time.elapsed().as_micros(),
                skipped

                );
                time_tracker = Instant::now();
            }
            if potential_res.is_some() {
                break potential_res.unwrap();
            }
            let all_fuzzing_results: Vec<FuzzOutcome> = pool
                .install(|| {
                    acir_cases_to_execute.clone().into_iter().par_bridge().map(
                        |(input, brillig_coverage, brillig_time)| {
                            let fuzz_res = self.single_fuzz_acir(&input).unwrap();
                            match fuzz_res {
                                FuzzOutcome::Case(CaseOutcome {
                                    case,
                                    witness,
                                    brillig_coverage: _,
                                    acir_time,
                                    brillig_time: _,
                                }) => FuzzOutcome::Case(CaseOutcome {
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
                                    counterexample,
                                    exit_reason,
                                }) => FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                                    counterexample,
                                    acir_failed: true,
                                    exit_reason,
                                }),
                            }
                        },
                    )
                })
                .collect::<Vec<FuzzOutcome>>();
            for fuzz_res in all_fuzzing_results.into_iter() {
                let (case, witness, brillig_coverage, acir_time, brillig_time) = match fuzz_res {
                    FuzzOutcome::Case(CaseOutcome {
                        case,
                        witness,
                        brillig_coverage,
                        acir_time,
                        brillig_time,
                    }) => (case, witness, brillig_coverage, acir_time, brillig_time),
                    _ => {
                        potential_res = Some(fuzz_res);
                        break;
                    }
                };
                // In case we fuzzed both ACIR and brillig
                if let (Some(witness), Some(brillig_coverage)) = (witness, brillig_coverage.clone())
                {
                    total_acir_time += acir_time;
                    total_acir_time += brillig_time;
                    // Update the potential list
                    if accumulated_coverage.potential_bool_witness_list.is_none() {
                        // If it's the first time, we need to assign
                        accumulated_coverage.potential_bool_witness_list =
                            Some(PotentialBoolWitnessList::from(&witness));
                    } else {
                        accumulated_coverage
                            .potential_bool_witness_list
                            .as_mut()
                            .unwrap()
                            .update(&witness);
                    }
                    let new_coverage = SingleTestCaseCoverage::new(
                        &witness,
                        brillig_coverage,
                        &accumulated_coverage.potential_bool_witness_list.as_mut().unwrap(),
                    );
                    if accumulated_coverage.merge(&&new_coverage) {
                        println!("Input: {:?}", case);
                        self.mutator.update_dictionary(&case);
                        match corpus.insert(case, true) {
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

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz(&self, input_map: &InputMap) -> Result<FuzzOutcome, ()> {
        let initial_witness = self.acir_program.abi.encode(&input_map, None).unwrap();
        let initial_witness2 = self.acir_program.abi.encode(&input_map, None).unwrap();
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
                case: input_map.clone(),
                witness: Some(witnesses),
                brillig_coverage: Some(brillig_coverage.unwrap()),
                acir_time: acir_elapsed.as_nanos(),
                brillig_time: brillig_elapsed.as_nanos(),
            })),
            (Err(err), Ok(_)) => Ok(FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                exit_reason: err,
                acir_failed: true,
                counterexample: input_map.clone(),
            })),
            (Ok(_), Err(err)) => Ok(FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                exit_reason: err,
                acir_failed: false,
                counterexample: input_map.clone(),
            })),
            (Err(err), Err(..)) => Ok(FuzzOutcome::CounterExample(CounterExampleOutcome {
                exit_reason: err,
                counterexample: input_map.clone(),
            })),
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz_acir(&self, input_map: &InputMap) -> Result<FuzzOutcome, ()> {
        let initial_witness = self.acir_program.abi.encode(&input_map, None).unwrap();
        let acir_start = Instant::now();
        let result_acir = (self.acir_executor)(&self.acir_program.bytecode, initial_witness);
        let acir_elapsed = acir_start.elapsed();

        match result_acir {
            Ok(witnesses) => Ok(FuzzOutcome::Case(CaseOutcome {
                case: input_map.clone(),
                witness: Some(witnesses),
                brillig_coverage: None,
                acir_time: acir_elapsed.as_nanos(),
                brillig_time: 0,
            })),
            Err(err) => Ok(FuzzOutcome::CounterExample(CounterExampleOutcome {
                exit_reason: err,
                counterexample: input_map.clone(),
            })),
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz_brillig(&self, input_map: &InputMap) -> Result<FuzzOutcome, ()> {
        let initial_witness = self.acir_program.abi.encode(&input_map, None).unwrap();
        let brillig_start = Instant::now();
        let result_brillig = (self.brillig_executor)(
            &self.brillig_program.bytecode,
            initial_witness,
            &self.location_to_feature_map,
        );
        let brillig_elapsed = brillig_start.elapsed();

        match result_brillig {
            Ok((_, brillig_coverage)) => Ok(FuzzOutcome::Case(CaseOutcome {
                case: input_map.clone(),
                witness: None,
                brillig_coverage: Some(brillig_coverage.unwrap()),
                acir_time: 0,
                brillig_time: brillig_elapsed.as_nanos(),
            })),
            Err(err) => Ok(FuzzOutcome::CounterExample(CounterExampleOutcome {
                exit_reason: err,
                counterexample: input_map.clone(),
            })),
        }
    }
}
