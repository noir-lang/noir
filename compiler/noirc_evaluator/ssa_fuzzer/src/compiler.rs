use std::collections::{BTreeSet, BTreeMap};
use acvm::{
    acir::circuit::{
        Circuit, ExpressionWidth,
    },
    FieldElement,
};

use acvm::acir::circuit::PublicInputs;
use noirc_evaluator::acir::generated_acir::GeneratedAcir;
use noirc_evaluator::ssa::ssa_gen::Ssa;
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::function::Function;
use noirc_evaluator::ssa::{
    SsaCircuitArtifact,
    function_builder::FunctionBuilder,
    ir::types::Type,
};
use noirc_evaluator::ssa::SsaBuilder;
use noirc_evaluator::ssa::SsaLogging;
use noirc_evaluator::ssa::{optimize_all, ArtifactsAndWarnings, SsaEvaluatorOptions, SsaProgramArtifact};
use noirc_evaluator::errors::RuntimeError;
use noirc_evaluator::ssa::ir::instruction::ErrorType;
use noirc_abi::{Abi, AbiReturnType, AbiType, AbiVisibility, AbiParameter};
use noirc_driver::{CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING, CompileError};
use noirc_errors::debug_info::{DebugVariables, DebugFunctions, DebugTypes, DebugInfo};
use crate::config::NUMBER_OF_VARIABLES_INITIAL;

fn optimize_into_acir(builder: FunctionBuilder, options: SsaEvaluatorOptions) -> Result<ArtifactsAndWarnings, RuntimeError> {
    let builder = SsaBuilder { ssa: builder.finish(), ssa_logging: SsaLogging::None, print_codegen_timings: false };
    let ssa = optimize_all(builder, &options)?;

    let brillig = ssa.to_brillig(options.enable_brillig_logging); 

    let ssa = SsaBuilder {
        ssa,
        ssa_logging: options.ssa_logging.clone(),
        print_codegen_timings: options.print_codegen_timings,
    }
    .run_pass(|ssa| ssa.fold_constants_with_brillig(&brillig), "Inlining Brillig Calls Inlining")
    .run_pass(Ssa::dead_instruction_elimination, "Dead Instruction Elimination (2nd)")
    .finish();
    let artifacts = ssa.into_acir(&brillig, options.expression_width).unwrap();

    Ok(ArtifactsAndWarnings(artifacts, vec![]))
}

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

    // This converts each im::Vector in the BTreeMap to a Vec
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

fn create_program(builder: FunctionBuilder, options: SsaEvaluatorOptions) -> Result<SsaProgramArtifact, RuntimeError> {
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

// create abi for 10 variables
// Abi { parameters: [AbiParameter { name: "v0", typ: Field, visibility: Private }, AbiParameter { name: "v1", typ: Field, visibility: Public }], 
// return_type: Some(AbiReturnType { abi_type: Field, visibility: Public }), error_types: {} }
fn generate_abi() -> Abi {
    let mut parameters = vec![];
    for i in 0..NUMBER_OF_VARIABLES_INITIAL {
        parameters.push(AbiParameter { name: format!("v{}", i), typ: AbiType::Field, visibility: AbiVisibility::Private });
    }
    let return_type = Some(AbiReturnType { abi_type: AbiType::Field, visibility: AbiVisibility::Public });
    let error_types = BTreeMap::new();
    Abi { parameters, return_type, error_types }
}

pub fn compile(builder: FunctionBuilder, options: &CompileOptions) -> Result<CompiledProgram, CompileError> {
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
        enable_brillig_logging: options.show_brillig,
        print_codegen_timings: options.benchmark_codegen,
        expression_width: ExpressionWidth::default(),
        emit_ssa: { None },
        skip_underconstrained_check: options.skip_underconstrained_check,
        enable_brillig_constraints_check: options.enable_brillig_constraints_check,
        inliner_aggressiveness: options.inliner_aggressiveness,
        max_bytecode_increase_percent: options.max_bytecode_increase_percent,
    };
    let SsaProgramArtifact { program, debug, warnings, names, brillig_names, .. } =
        create_program(builder, ssa_evaluator_options)?;
    let abi = generate_abi();
    let file_map = BTreeMap::new();
    Ok(CompiledProgram {
        hash: 1,
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
