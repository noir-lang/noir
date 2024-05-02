use noirc_errors::{Location, Span};
use noirc_frontend::ast::{FunctionReturnType, NoirFunction, UnresolvedTypeData};
use noirc_frontend::{
    graph::CrateId,
    macros_api::{FileId, HirContext},
    parse_program, Type,
};

use crate::utils::{
    errors::AztecMacroError,
    hir_utils::{
        collect_crate_functions, collect_traits, fetch_notes, get_contract_module_data,
        get_global_numberic_const, get_serialized_length, inject_fn,
    },
};

// Check if "compute_note_hash_and_nullifier(AztecAddress,Field,Field,Field,[Field; N]) -> [Field; 4]" is defined
fn check_for_compute_note_hash_and_nullifier_definition(
    crate_id: &CrateId,
    context: &HirContext,
) -> bool {
    collect_crate_functions(crate_id, context).iter().any(|funct_id| {
        let func_data = context.def_interner.function_meta(funct_id);
        let func_name = context.def_interner.function_name(funct_id);
        func_name == "compute_note_hash_and_nullifier"
                && func_data.parameters.len() == 5
                && func_data.parameters.0.first().is_some_and(| (_, typ, _) | match typ {
                    Type::Struct(struct_typ, _) => struct_typ.borrow().name.0.contents == "AztecAddress",
                    _ => false
                })
                && func_data.parameters.0.get(1).is_some_and(|(_, typ, _)| typ.is_field())
                && func_data.parameters.0.get(2).is_some_and(|(_, typ, _)| typ.is_field())
                && func_data.parameters.0.get(3).is_some_and(|(_, typ, _)|  typ.is_field())
                // checks if the 5th parameter is an array and contains only fields
                && func_data.parameters.0.get(4).is_some_and(|(_, typ, _)|  match typ {
                    Type::Array(_, inner_type) => inner_type.to_owned().is_field(),
                    _ => false
                })
                // We check the return type the same way as we did the 5th parameter
                && match &func_data.return_type {
                    FunctionReturnType::Default(_) => false,
                    FunctionReturnType::Ty(unresolved_type) => {
                        match &unresolved_type.typ {
                            UnresolvedTypeData::Array(_, inner_type) => matches!(inner_type.typ, UnresolvedTypeData::FieldElement),
                            _ => false,
                        }
                    }
                }
    })
}

pub fn inject_compute_note_hash_and_nullifier(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    if let Some((_, module_id, file_id)) = get_contract_module_data(context, crate_id) {
        // If compute_note_hash_and_nullifier is already defined by the user, we skip auto-generation in order to provide an
        // escape hatch for this mechanism.
        // TODO(#4647): improve this diagnosis and error messaging.
        if context.crate_graph.root_crate_id() != crate_id
            || check_for_compute_note_hash_and_nullifier_definition(crate_id, context)
        {
            return Ok(());
        }

        let traits: Vec<_> = collect_traits(context);

        // Get MAX_NOTE_FIELDS_LENGTH global to check if the notes in our contract are too long.
        let max_note_length_const = get_global_numberic_const(context, "MAX_NOTE_FIELDS_LENGTH")
            .map_err(|err| {
                (
                    AztecMacroError::CouldNotImplementComputeNoteHashAndNullifier {
                        secondary_message: Some(err.primary_message),
                    },
                    file_id,
                )
            })?;

        // In order to implement compute_note_hash_and_nullifier, we need to know all of the different note types the
        // contract might use and their serialized lengths. These are the types that are marked as #[aztec(note)].
        let mut notes_and_lengths = vec![];

        for (path, typ) in fetch_notes(context) {
            let serialized_len: u128 = get_serialized_length(
                &traits,
                "NoteInterface",
                &Type::Struct(typ.clone(), vec![]),
                &context.def_interner,
            )
            .map_err(|_err| {
                (
                    AztecMacroError::CouldNotImplementComputeNoteHashAndNullifier {
                        secondary_message: Some(format!(
                            "Failed to get serialized length for note type {}",
                            path
                        )),
                    },
                    file_id,
                )
            })?
            .into();

            if serialized_len > max_note_length_const {
                return Err((
                   AztecMacroError::CouldNotImplementComputeNoteHashAndNullifier {
                        secondary_message: Some(format!(
                            "Note type {} as {} fields, which is more than the maximum allowed length of {}.",
                            path,
                            serialized_len,
                            max_note_length_const
                        )),
                    },
                    file_id,
                ));
            }

            notes_and_lengths.push((path.to_string(), serialized_len));
        }

        let max_note_length: u128 =
            *notes_and_lengths.iter().map(|(_, serialized_len)| serialized_len).max().unwrap_or(&0);

        let note_types =
            notes_and_lengths.iter().map(|(note_type, _)| note_type.clone()).collect::<Vec<_>>();

        // We can now generate a version of compute_note_hash_and_nullifier tailored for the contract in this crate.
        let func = generate_compute_note_hash_and_nullifier(&note_types, max_note_length);

        // And inject the newly created function into the contract.

        // TODO(#4373): We don't have a reasonable location for the source code of this autogenerated function, so we simply
        // pass an empty span. This function should not produce errors anyway so this should not matter.
        let location = Location::new(Span::empty(0), file_id);

        inject_fn(crate_id, context, func, location, module_id, file_id).map_err(|err| {
            (
                AztecMacroError::CouldNotImplementComputeNoteHashAndNullifier {
                    secondary_message: err.secondary_message,
                },
                file_id,
            )
        })?;
    }
    Ok(())
}

fn generate_compute_note_hash_and_nullifier(
    note_types: &[String],
    max_note_length: u128,
) -> NoirFunction {
    let function_source =
        generate_compute_note_hash_and_nullifier_source(note_types, max_note_length);

    let (function_ast, errors) = parse_program(&function_source);
    if !errors.is_empty() {
        dbg!(errors.clone());
    }
    assert_eq!(errors.len(), 0, "Failed to parse Noir macro code. This is either a bug in the compiler or the Noir macro code");

    let mut function_ast = function_ast.into_sorted();
    function_ast.functions.remove(0)
}

fn generate_compute_note_hash_and_nullifier_source(
    note_types: &[String],
    max_note_length: u128,
) -> String {
    // TODO(#4649): The serialized_note parameter is a fixed-size array, but we don't know what length it should have.
    // For now we hardcode it to 20, which is the same as MAX_NOTE_FIELDS_LENGTH.

    if note_types.is_empty() {
        // Even if the contract does not include any notes, other parts of the stack expect for this function to exist,
        // so we include a dummy version.
        format!(
            "
        unconstrained fn compute_note_hash_and_nullifier(
            contract_address: dep::aztec::protocol_types::address::AztecAddress,
            nonce: Field,
            storage_slot: Field,
            note_type_id: Field,
            serialized_note: [Field; {}]
        ) -> pub [Field; 4] {{
            assert(false, \"This contract does not use private notes\");
            [0, 0, 0, 0]
        }}",
            max_note_length
        )
    } else {
        // For contracts that include notes we do a simple if-else chain comparing note_type_id with the different
        // get_note_type_id of each of the note types.

        let if_statements: Vec<String> = note_types.iter().map(|note_type| format!(
            "if (note_type_id == {0}::get_note_type_id()) {{
                dep::aztec::note::utils::compute_note_hash_and_nullifier({0}::deserialize_content, note_header, serialized_note)
            }}"
        , note_type)).collect();

        let full_if_statement = if_statements.join(" else ")
            + "
            else {
                assert(false, \"Unknown note type ID\");
                [0, 0, 0, 0]
            }";

        format!(
            "
            unconstrained fn compute_note_hash_and_nullifier(
                contract_address: dep::aztec::protocol_types::address::AztecAddress,
                nonce: Field,
                storage_slot: Field,
                note_type_id: Field,
                serialized_note: [Field; {}]
            ) -> pub [Field; 4] {{
                let note_header = dep::aztec::prelude::NoteHeader::new(contract_address, nonce, storage_slot);

                {}
            }}",
            max_note_length,
            full_if_statement
        )
    }
}
