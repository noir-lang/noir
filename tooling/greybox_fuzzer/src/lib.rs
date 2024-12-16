use std::time::Instant;

use acvm::{
    acir::{
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
        //println!("Current threads: {}", current_num_threads());
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
            let (_case, witness, brillig_coverage) = match fuzz_res {
                FuzzOutcome::Case(CaseOutcome { case, witness, brillig_coverage }) => {
                    (case, witness, brillig_coverage)
                }
                _ => panic!("Already checked this"),
            };
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
        let testcases_per_iteration = 128;
        let fuzz_res = loop {
            if current_iteration % 128 == 0 {
                println!("Current iteration {current_iteration}");
            }
            let mut testcase_set: Vec<(usize, InputMap)> = Vec::new();
            testcase_set.reserve(testcases_per_iteration);
            for i in 0..testcases_per_iteration {
                let (main_testcase, additional_testcase) =
                    corpus.get_next_testcase_with_additional(&mut prng);
                let input_map = self.mutator.mutate_input_map_multiple(
                    main_testcase,
                    additional_testcase,
                    &mut prng,
                );
                testcase_set.push((i, input_map));
            }

            let all_fuzzing_results: Vec<FuzzOutcome> = pool
                .install(|| {
                    testcase_set.clone().into_iter().par_bridge().map(|(index, input)| unsafe {
                        let res: FuzzOutcome = self.single_fuzz(&input).unwrap();
                        res
                    })
                })
                .collect::<Vec<FuzzOutcome>>();
            let mut potential_res = None;
            for fuzz_res in all_fuzzing_results.into_iter() {
                let (case, witness, brillig_coverage) = match fuzz_res {
                    FuzzOutcome::Case(CaseOutcome { case, witness, brillig_coverage }) => {
                        (case, witness, brillig_coverage)
                    }
                    _ => {
                        potential_res = Some(fuzz_res);
                        break;
                    }
                };
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
            }
            if potential_res.is_some() {
                break potential_res.unwrap();
            }
            current_iteration += testcases_per_iteration;
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
        println!("Acir: {}, brillig: {}", acir_elapsed.as_micros(), brillig_elapsed.as_micros());

        // TODO: Add handling for `vm.assume` equivalent

        match (result_acir, result_brillig) {
            (Ok(witnesses), Ok((_map, brillig_coverage))) => Ok(FuzzOutcome::Case(CaseOutcome {
                case: input_map.clone(),
                witness: witnesses,
                brillig_coverage: brillig_coverage.unwrap(),
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
}
