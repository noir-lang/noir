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
use strategies::InputMutator;
use types::{CaseOutcome, CounterExampleOutcome, DiscrepancyOutcome, FuzzOutcome, FuzzTestResult};

use noirc_artifacts::program::ProgramArtifact;
use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

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
}

impl<
        E: Fn(
            &Program<FieldElement>,
            WitnessMap<FieldElement>,
        ) -> Result<WitnessStack<FieldElement>, String>,
        F: Fn(
            &Program<FieldElement>,
            WitnessMap<FieldElement>,
            &BranchToFeatureMap,
        ) -> Result<(WitnessStack<FieldElement>, Option<Vec<u32>>), String>,
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
        for entry in starting_corpus.iter() {
            let (fuzz_res, coverage) = self.single_fuzz(&entry, &mut accumulated_coverage).unwrap();
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
            if accumulated_coverage.merge(&coverage.unwrap()) {
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
        let fuzz_res = loop {
            if current_iteration % 10000 == 0 {
                println!("Current iteration {current_iteration}");
            }
            let (main_testcase, additional_testcase) =
                corpus.get_next_testcase_with_additional(&mut prng);
            let input_map = self.mutator.mutate_input_map_multiple(
                main_testcase,
                additional_testcase,
                &mut prng,
            );
            let (fuzz_res, coverage) =
                self.single_fuzz(&input_map, &mut accumulated_coverage).unwrap();
            match fuzz_res {
                FuzzOutcome::Case(_) => (),
                _ => {
                    break fuzz_res;
                }
            }
            if accumulated_coverage.merge(&coverage.unwrap()) {
                println!("Input: {:?}", input_map);
                self.mutator.update_dictionary(&input_map);
                match corpus.insert(input_map.clone(), true) {
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
            current_iteration += 1;
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

                FuzzTestResult { success: false, reason, counterexample: Some(counterexample) }
            }
            FuzzOutcome::CounterExample(CounterExampleOutcome {
                exit_reason: status,
                counterexample,
            }) => {
                let reason = status.to_string();
                let reason = if reason.is_empty() { None } else { Some(reason) };

                FuzzTestResult { success: false, reason, counterexample: Some(counterexample) }
            }
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz(
        &self,
        input_map: &InputMap,
        accumulated_coverage: &mut AccumulatedFuzzerCoverage,
    ) -> Result<(FuzzOutcome, Option<SingleTestCaseCoverage>), ()> {
        let initial_witness = self.acir_program.abi.encode(&input_map, None).unwrap();
        let initial_witness2 = self.acir_program.abi.encode(&input_map, None).unwrap();
        let result_acir = (self.acir_executor)(&self.acir_program.bytecode, initial_witness);
        let result_brillig = (self.brillig_executor)(
            &self.brillig_program.bytecode,
            initial_witness2,
            &self.location_to_feature_map,
        );

        // TODO: Add handling for `vm.assume` equivalent

        match (result_acir, result_brillig) {
            (Ok(witnesses), Ok((_map, brillig_coverage))) => {
                // Update the potential list
                if accumulated_coverage.potential_bool_witness_list.is_none() {
                    // If it's the first time, we need to assign
                    accumulated_coverage.potential_bool_witness_list =
                        Some(PotentialBoolWitnessList::from(&witnesses));
                } else {
                    accumulated_coverage
                        .potential_bool_witness_list
                        .as_mut()
                        .unwrap()
                        .update(&witnesses);
                }
                let new_coverage = SingleTestCaseCoverage::new(
                    &witnesses,
                    brillig_coverage.unwrap(),
                    &accumulated_coverage.potential_bool_witness_list.as_mut().unwrap(),
                );
                Ok((FuzzOutcome::Case(CaseOutcome { case: input_map.clone() }), Some(new_coverage)))
            }
            (Err(err), Ok(_)) => Ok((
                FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                    exit_reason: err,
                    acir_failed: true,
                    counterexample: input_map.clone(),
                }),
                None,
            )),
            (Ok(_), Err(err)) => Ok((
                FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                    exit_reason: err,
                    acir_failed: false,
                    counterexample: input_map.clone(),
                }),
                None,
            )),
            (Err(err), Err(..)) => Ok((
                FuzzOutcome::CounterExample(CounterExampleOutcome {
                    exit_reason: err,
                    counterexample: input_map.clone(),
                }),
                None,
            )),
        }
    }
}
