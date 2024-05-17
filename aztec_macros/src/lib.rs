mod transforms;
mod utils;

use transforms::{
    compute_note_hash_and_nullifier::inject_compute_note_hash_and_nullifier,
    contract_interface::{
        generate_contract_interface, stub_function, update_fn_signatures_in_contract_interface,
    },
    events::{generate_selector_impl, transform_events},
    functions::{
        check_for_public_args, export_fn_abi, transform_function, transform_unconstrained,
    },
    note_interface::{generate_note_interface_impl, inject_note_exports},
    storage::{
        assign_storage_slots, check_for_storage_definition, check_for_storage_implementation,
        generate_storage_implementation, generate_storage_layout,
    },
};

use noirc_frontend::macros_api::{
    CrateId, FileId, HirContext, MacroError, MacroProcessor, SortedModule, Span,
};

use utils::{
    ast_utils::is_custom_attribute,
    checks::{check_for_aztec_dependency, has_aztec_dependency},
    constants::MAX_CONTRACT_PRIVATE_FUNCTIONS,
    errors::AztecMacroError,
};
pub struct AztecMacro;

impl MacroProcessor for AztecMacro {
    fn process_untyped_ast(
        &self,
        ast: SortedModule,
        crate_id: &CrateId,
        file_id: FileId,
        context: &HirContext,
    ) -> Result<SortedModule, (MacroError, FileId)> {
        transform(ast, crate_id, file_id, context)
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
    file_id: FileId,
    context: &HirContext,
) -> Result<SortedModule, (MacroError, FileId)> {
    // Usage -> mut ast -> aztec_library::transform(&mut ast)
    // Covers all functions in the ast
    for submodule in ast.submodules.iter_mut().filter(|submodule| submodule.is_contract) {
        if transform_module(
            crate_id,
            context,
            &mut submodule.contents,
            submodule.name.0.contents.as_str(),
        )
        .map_err(|err| (err.into(), file_id))?
        {
            check_for_aztec_dependency(crate_id, context)?;
        }
    }

    generate_note_interface_impl(&mut ast).map_err(|err| (err.into(), file_id))?;

    Ok(ast)
}

/// Determines if ast nodes are annotated with aztec attributes.
/// For annotated functions it calls the `transform` function which will perform the required transformations.
/// Returns true if an annotated node is found, false otherwise
fn transform_module(
    crate_id: &CrateId,
    context: &HirContext,
    module: &mut SortedModule,
    module_name: &str,
) -> Result<bool, AztecMacroError> {
    let mut has_transformed_module = false;

    // Check for a user defined storage struct

    let maybe_storage_struct_name = check_for_storage_definition(module)?;

    let storage_defined = maybe_storage_struct_name.is_some();

    if let Some(ref storage_struct_name) = maybe_storage_struct_name {
        if !check_for_storage_implementation(module, storage_struct_name) {
            generate_storage_implementation(module, storage_struct_name)?;
        }
        // Make sure we're only generating the storage layout for the root crate
        // In case we got a contract importing other contracts for their interface, we
        // don't want to generate the storage layout for them
        if crate_id == context.root_crate_id() {
            generate_storage_layout(module, storage_struct_name.clone())?;
        }
    }

    for structure in module.types.iter_mut() {
        if structure.attributes.iter().any(|attr| is_custom_attribute(attr, "aztec(event)")) {
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

    let mut stubs: Vec<_> = vec![];

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
        if is_private || is_public || is_public_vm {
            let fn_type = if is_private {
                "Private"
            } else if is_public_vm {
                "Avm"
            } else {
                "Public"
            };
            stubs.push(stub_function(fn_type, func));

            export_fn_abi(&mut module.types, func)?;
            transform_function(
                fn_type,
                func,
                maybe_storage_struct_name.clone(),
                is_initializer,
                insert_init_check,
                is_internal,
            )?;
            has_transformed_module = true;
        } else if storage_defined && func.def.is_unconstrained {
            transform_unconstrained(func, maybe_storage_struct_name.clone().unwrap());
            has_transformed_module = true;
        }
    }

    if has_transformed_module {
        // We only want to run these checks if the macro processor has found the module to be an Aztec contract.

        let private_functions: Vec<_> = module
            .functions
            .iter()
            .filter(|func| {
                func.def
                    .attributes
                    .secondary
                    .iter()
                    .any(|attr| is_custom_attribute(attr, "aztec(private)"))
            })
            .collect();

        let public_functions: Vec<_> = module
            .functions
            .iter()
            .filter(|func| {
                func.def
                    .attributes
                    .secondary
                    .iter()
                    .any(|attr| is_custom_attribute(attr, "aztec(public)"))
            })
            .collect();

        let private_function_count = private_functions.len();

        check_for_public_args(&private_functions)?;

        check_for_public_args(&public_functions)?;

        if private_function_count > MAX_CONTRACT_PRIVATE_FUNCTIONS {
            return Err(AztecMacroError::ContractHasTooManyPrivateFunctions {
                span: Span::default(),
            });
        }

        generate_contract_interface(module, module_name, &stubs)?;
    }

    Ok(has_transformed_module)
}

//
//                    Transform Hir Nodes for Aztec
//

/// Completes the Hir with data gathered from type resolution
fn transform_hir(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    if has_aztec_dependency(crate_id, context) {
        transform_events(crate_id, context)?;
        inject_compute_note_hash_and_nullifier(crate_id, context)?;
        assign_storage_slots(crate_id, context)?;
        inject_note_exports(crate_id, context)?;
        update_fn_signatures_in_contract_interface(crate_id, context)
    } else {
        Ok(())
    }
}
