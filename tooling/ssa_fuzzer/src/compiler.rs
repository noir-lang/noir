use acvm::{
    FieldElement,
    acir::circuit::{Circuit, ExpressionWidth},
};
use std::collections::{BTreeMap, BTreeSet};

use crate::config::NUMBER_OF_VARIABLES_INITIAL;
use acvm::acir::circuit::PublicInputs;
use noirc_abi::{Abi, AbiParameter, AbiReturnType, AbiType, AbiVisibility};
use noirc_driver::{CompileError, CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING};
use noirc_errors::debug_info::{DebugFunctions, DebugInfo, DebugTypes, DebugVariables};
use noirc_evaluator::{
    acir::generated_acir::GeneratedAcir,
    errors::RuntimeError,
    ssa::{
        ArtifactsAndWarnings, BrilligOptions, SsaBuilder, SsaCircuitArtifact, SsaEvaluatorOptions,
        SsaLogging, SsaProgramArtifact, function_builder::FunctionBuilder,
        ir::instruction::ErrorType, optimize_all, ssa_gen::Ssa,
    },
};

/// Optimizes the given FunctionBuilder into ACIR
/// its taken from noirc_evaluator::ssa::optimize_all, but modified to accept FunctionBuilder
fn optimize_into_acir(
    builder: FunctionBuilder,
    options: SsaEvaluatorOptions,
) -> Result<ArtifactsAndWarnings, RuntimeError> {
    let builder = SsaBuilder {
        ssa: builder.finish(),
        ssa_logging: SsaLogging::None,
        print_codegen_timings: false,
    };
    let ssa = optimize_all(builder, &options)?;

    let brillig = ssa.to_brillig(&BrilligOptions::default());

    let ssa = SsaBuilder {
        ssa,
        ssa_logging: options.ssa_logging.clone(),
        print_codegen_timings: options.print_codegen_timings,
    }
    .run_pass(|ssa| ssa.fold_constants_with_brillig(&brillig), "Inlining Brillig Calls Inlining")
    .run_pass(Ssa::dead_instruction_elimination, "Dead Instruction Elimination (2nd)")
    .finish();

    // to triage ssa after optimizations, when crash found
    let formatted_ssa = format!("{}", ssa);
    log::debug!("formatted_ssa: {:?}", formatted_ssa);
    match ssa.into_acir(&brillig, &BrilligOptions::default(), options.expression_width) {
        Ok(artifacts) => Ok(ArtifactsAndWarnings(artifacts, vec![])),
        Err(e) => Err(e),
    }
}

/// Converts the generated ACIR into a circuit artifact
/// its taken from noirc_evaluator::ssa::convert_generated_acir_into_circuit,
/// but modified not to use signature
/// in initial function signature used to split public, private and return witnesses
/// but now we don't need it, because we don't have any public inputs,
/// so we just use all witnesses as private inputs and return witnesses
fn convert_generated_acir_into_circuit_without_signature(
    mut generated_acir: GeneratedAcir<FieldElement>,
    debug_variables: DebugVariables,
    debug_functions: DebugFunctions,
    debug_types: DebugTypes,
) -> SsaCircuitArtifact {
    let opcodes = generated_acir.take_opcodes();
    let current_witness_index = generated_acir.current_witness_index().0;
    let GeneratedAcir {
        return_witnesses,
        locations,
        brillig_locations,
        input_witnesses,
        assertion_payloads: assert_messages,
        warnings,
        name,
        brillig_procedure_locs,
        ..
    } = generated_acir;

    let private_parameters = BTreeSet::from_iter(input_witnesses.iter().copied());
    let public_parameters = PublicInputs(BTreeSet::new());
    let return_values = PublicInputs(return_witnesses.iter().copied().collect());

    let circuit = Circuit {
        current_witness_index,
        expression_width: ExpressionWidth::Unbounded,
        opcodes,
        private_parameters,
        public_parameters,
        return_values,
        assert_messages: assert_messages.into_iter().collect(),
    };

    let locations = locations
        .into_iter()
        .map(|(index, locations)| (index, locations.into_iter().collect()))
        .collect();

    let brillig_locations = brillig_locations
        .into_iter()
        .map(|(function_index, locations)| {
            let locations = locations
                .into_iter()
                .map(|(index, locations)| (index, locations.into_iter().collect()))
                .collect();
            (function_index, locations)
        })
        .collect();

    let mut debug_info = DebugInfo::new(
        locations,
        brillig_locations,
        debug_variables,
        debug_functions,
        debug_types,
        brillig_procedure_locs,
    );

    // Perform any ACIR-level optimizations
    let (optimized_circuit, transformation_map) = acvm::compiler::optimize(circuit);
    debug_info.update_acir(transformation_map);

    SsaCircuitArtifact {
        name,
        circuit: optimized_circuit,
        debug_info,
        warnings,
        input_witnesses,
        return_witnesses,
        error_types: generated_acir.error_types,
    }
}

/// Creates a program artifact from the given FunctionBuilder
/// its taken from noirc_evaluator::ssa::create_program, but modified to accept FunctionBuilder
fn create_program(
    builder: FunctionBuilder,
    options: SsaEvaluatorOptions,
) -> Result<SsaProgramArtifact, RuntimeError> {
    let ArtifactsAndWarnings(
        (generated_acirs, generated_brillig, brillig_function_names, error_types),
        _ssa_level_warnings,
    ) = optimize_into_acir(builder, options)?;
    let error_types = error_types
        .into_iter()
        .map(|(selector, hir_type)| (selector, ErrorType::Dynamic(hir_type)))
        .collect();
    let mut program_artifact = SsaProgramArtifact::new(generated_brillig, error_types);
    let mut is_main = true;
    // without func_sig
    for acir in generated_acirs.into_iter() {
        let circuit_artifact = convert_generated_acir_into_circuit_without_signature(
            acir,
            // TODO: get rid of these clones
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        );
        program_artifact.add_circuit(circuit_artifact, is_main);
        is_main = false;
    }
    program_artifact.brillig_names = brillig_function_names;

    Ok(program_artifact)
}

/// Creates an ABI for the configured number of variables
/// Seems useless in this case, but its needed for compile function
fn generate_abi() -> Abi {
    let mut parameters = vec![];
    for i in 0..NUMBER_OF_VARIABLES_INITIAL {
        parameters.push(AbiParameter {
            name: format!("v{}", i),
            typ: AbiType::Field,
            visibility: AbiVisibility::Private,
        });
    }
    let return_type =
        Some(AbiReturnType { abi_type: AbiType::Field, visibility: AbiVisibility::Public });
    let error_types = BTreeMap::new();
    Abi { parameters, return_type, error_types }
}

/// Compiles the given FunctionBuilder into a CompiledProgram
/// its taken from noirc_driver::compile_no_check, but modified to accept FunctionBuilder
pub fn compile(
    builder: FunctionBuilder,
    options: &CompileOptions,
) -> Result<CompiledProgram, CompileError> {
    let ssa_evaluator_options = SsaEvaluatorOptions {
        ssa_logging: match &options.show_ssa_pass {
            Some(string) => SsaLogging::Contains(string.clone()),
            None => {
                if options.show_ssa {
                    SsaLogging::All
                } else {
                    SsaLogging::None
                }
            }
        },
        print_codegen_timings: options.benchmark_codegen,
        expression_width: ExpressionWidth::default(),
        emit_ssa: { None },
        skip_underconstrained_check: options.skip_underconstrained_check,
        skip_brillig_constraints_check: true,
        inliner_aggressiveness: options.inliner_aggressiveness,
        max_bytecode_increase_percent: options.max_bytecode_increase_percent,
        brillig_options: BrilligOptions::default(),
        enable_brillig_constraints_check_lookback: false,
    };
    let SsaProgramArtifact { program, debug, warnings, names, brillig_names, .. } =
        create_program(builder, ssa_evaluator_options)?;
    let abi = generate_abi();
    let file_map = BTreeMap::new();
    Ok(CompiledProgram {
        hash: 1, // const hash, doesn't matter in this case
        program,
        debug,
        abi,
        file_map,
        noir_version: NOIR_ARTIFACT_VERSION_STRING.to_string(),
        warnings,
        names,
        brillig_names,
    })
}
