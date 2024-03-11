mod transforms;
mod utils;

use transforms::{
    compute_note_hash_and_nullifier::inject_compute_note_hash_and_nullifier,
    events::{generate_selector_impl, transform_events},
    functions::{transform_function, transform_unconstrained, transform_vm_function},
    storage::{
        assign_storage_slots, check_for_storage_definition, check_for_storage_implementation,
        generate_storage_implementation,
    },
};

use noirc_frontend::hir::def_collector::dc_crate::{UnresolvedFunctions, UnresolvedTraitImpl};

use noirc_frontend::macros_api::SortedModule;
use noirc_frontend::macros_api::{CrateId, MacroError};
use noirc_frontend::macros_api::{FileId, MacroProcessor};
use noirc_frontend::macros_api::{HirContext, SecondaryAttribute, Span};

use utils::ast_utils::is_custom_attribute;
use utils::checks::{check_for_aztec_dependency, has_aztec_dependency};
use utils::{constants::MAX_CONTRACT_PRIVATE_FUNCTIONS, errors::AztecMacroError};
pub struct AztecMacro;

impl MacroProcessor for AztecMacro {
    fn process_untyped_ast(
        &self,
        ast: SortedModule,
        crate_id: &CrateId,
        context: &HirContext,
    ) -> Result<SortedModule, (MacroError, FileId)> {
        transform(ast, crate_id, context)
    }

    fn process_collected_defs(
        &self,
        crate_id: &CrateId,
        context: &mut HirContext,
        collected_trait_impls: &[UnresolvedTraitImpl],
        collected_functions: &mut [UnresolvedFunctions],
    ) -> Result<(), (MacroError, FileId)> {
        transform_collected_defs(crate_id, context, collected_trait_impls, collected_functions)
    }

    fn process_typed_ast(
        &self,
        crate_id: &CrateId,
        context: &mut HirContext,
    ) -> Result<(), (MacroError, FileId)> {
        transform_hir(crate_id, context).map_err(|(err, file_id)| (err.into(), file_id))
    }
}

//
//                    Create AST Nodes for Aztec
//

/// Traverses every function in the ast, calling `transform_function` which
/// determines if further processing is required
fn transform(
    mut ast: SortedModule,
    crate_id: &CrateId,
    context: &HirContext,
) -> Result<SortedModule, (MacroError, FileId)> {
    // Usage -> mut ast -> aztec_library::transform(&mut ast)
    // Covers all functions in the ast
    for submodule in ast.submodules.iter_mut().filter(|submodule| submodule.is_contract) {
        if transform_module(&mut submodule.contents, crate_id, context)
            .map_err(|(err, file_id)| (err.into(), file_id))?
        {
            check_for_aztec_dependency(crate_id, context)?;
        }
    }
    Ok(ast)
}

/// Determines if ast nodes are annotated with aztec attributes.
/// For annotated functions it calls the `transform` function which will perform the required transformations.
/// Returns true if an annotated node is found, false otherwise
fn transform_module(
    module: &mut SortedModule,
    crate_id: &CrateId,
    context: &HirContext,
) -> Result<bool, (AztecMacroError, FileId)> {
    let mut has_transformed_module = false;

    // Check for a user defined storage struct
    let storage_defined = check_for_storage_definition(module);
    let storage_implemented = check_for_storage_implementation(module);

    let crate_graph = &context.crate_graph[crate_id];

    if storage_defined && !storage_implemented {
        generate_storage_implementation(module).map_err(|err| (err, crate_graph.root_file_id))?;
    }

    for structure in module.types.iter() {
        if structure.attributes.iter().any(|attr| matches!(attr, SecondaryAttribute::Event)) {
            module.impls.push(generate_selector_impl(structure));
            has_transformed_module = true;
        }
    }

    let has_initializer = module.functions.iter().any(|func| {
        func.def
            .attributes
            .secondary
            .iter()
            .any(|attr| is_custom_attribute(attr, "aztec(initializer)"))
    });

    for func in module.functions.iter_mut() {
        let mut is_private = false;
        let mut is_public = false;
        let mut is_public_vm = false;
        let mut is_initializer = false;
        let mut is_internal = false;
        let mut insert_init_check = has_initializer;

        for secondary_attribute in func.def.attributes.secondary.clone() {
            if is_custom_attribute(&secondary_attribute, "aztec(private)") {
                is_private = true;
            } else if is_custom_attribute(&secondary_attribute, "aztec(initializer)") {
                is_initializer = true;
                insert_init_check = false;
            } else if is_custom_attribute(&secondary_attribute, "aztec(noinitcheck)") {
                insert_init_check = false;
            } else if is_custom_attribute(&secondary_attribute, "aztec(internal)") {
                is_internal = true;
            } else if is_custom_attribute(&secondary_attribute, "aztec(public)") {
                is_public = true;
            } else if is_custom_attribute(&secondary_attribute, "aztec(public-vm)") {
                is_public_vm = true;
            }
        }

        // Apply transformations to the function based on collected attributes
        if is_private || is_public {
            transform_function(
                if is_private { "Private" } else { "Public" },
                func,
                storage_defined,
                is_initializer,
                insert_init_check,
                is_internal,
            )
            .map_err(|err| (err, crate_graph.root_file_id))?;
            has_transformed_module = true;
        } else if is_public_vm {
            transform_vm_function(func, storage_defined)
                .map_err(|err| (err, crate_graph.root_file_id))?;
            has_transformed_module = true;
        }

        // Add the storage struct to the beginning of the function if it is unconstrained in an aztec contract
        if storage_defined && func.def.is_unconstrained {
            transform_unconstrained(func);
            has_transformed_module = true;
        }
    }

    if has_transformed_module {
        // We only want to run these checks if the macro processor has found the module to be an Aztec contract.

        let private_functions_count = module
            .functions
            .iter()
            .filter(|func| {
                func.def
                    .attributes
                    .secondary
                    .iter()
                    .any(|attr| is_custom_attribute(attr, "aztec(private)"))
            })
            .count();

        if private_functions_count > MAX_CONTRACT_PRIVATE_FUNCTIONS {
            let crate_graph = &context.crate_graph[crate_id];
            return Err((
                AztecMacroError::ContractHasTooManyPrivateFunctions { span: Span::default() },
                crate_graph.root_file_id,
            ));
        }

        let constructor_defined = module.functions.iter().any(|func| func.name() == "constructor");
        if !constructor_defined {
            let crate_graph = &context.crate_graph[crate_id];
            return Err((
                AztecMacroError::ContractConstructorMissing { span: Span::default() },
                crate_graph.root_file_id,
            ));
        }
    }

    Ok(has_transformed_module)
}

fn transform_collected_defs(
    crate_id: &CrateId,
    context: &mut HirContext,
    collected_trait_impls: &[UnresolvedTraitImpl],
    collected_functions: &mut [UnresolvedFunctions],
) -> Result<(), (MacroError, FileId)> {
    if has_aztec_dependency(crate_id, context) {
        inject_compute_note_hash_and_nullifier(
            crate_id,
            context,
            collected_trait_impls,
            collected_functions,
        )
    } else {
        Ok(())
    }
}

//
//                    Transform Hir Nodes for Aztec
//

/// Completes the Hir with data gathered from type resolution
fn transform_hir(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    transform_events(crate_id, context)?;
    assign_storage_slots(crate_id, context)
}
