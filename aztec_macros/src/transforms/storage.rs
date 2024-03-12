use std::borrow::{Borrow, BorrowMut};

use noirc_errors::Span;
use noirc_frontend::{
    graph::CrateId,
    macros_api::{
        FieldElement, FileId, HirContext, HirExpression, HirLiteral, HirStatement, NodeInterner,
    },
    node_interner::{TraitId, TraitImplKind},
    parser::SortedModule,
    BlockExpression, Expression, ExpressionKind, FunctionDefinition, Ident, Literal, NoirFunction,
    PathKind, Pattern, StatementKind, Type, TypeImpl, UnresolvedType, UnresolvedTypeData,
};

use crate::{
    chained_dep, chained_path,
    utils::{
        ast_utils::{
            call, expression, ident, ident_path, lambda, make_statement, make_type, pattern,
            return_type, variable, variable_path,
        },
        errors::AztecMacroError,
        hir_utils::{collect_crate_structs, collect_traits},
    },
};

// Check to see if the user has defined a storage struct
pub fn check_for_storage_definition(module: &SortedModule) -> bool {
    module.types.iter().any(|r#struct| r#struct.name.0.contents == "Storage")
}

// Check to see if the user has defined a storage struct
pub fn check_for_storage_implementation(module: &SortedModule) -> bool {
    module.impls.iter().any(|r#impl| match &r#impl.object_type.typ {
        UnresolvedTypeData::Named(path, _, _) => {
            path.segments.last().is_some_and(|segment| segment.0.contents == "Storage")
        }
        _ => false,
    })
}

/// Auxiliary function to generate the storage constructor for a given field, using
/// the Storage definition as a reference. Supports nesting.
pub fn generate_storage_field_constructor(
    (type_ident, unresolved_type): &(Ident, UnresolvedType),
    slot: Expression,
) -> Result<Expression, AztecMacroError> {
    let typ = &unresolved_type.typ;
    match typ {
        UnresolvedTypeData::Named(path, generics, _) => {
            let mut new_path = path.clone().to_owned();
            new_path.segments.push(ident("new"));
            match path.segments.last().unwrap().0.contents.as_str() {
                "Map" => Ok(call(
                    variable_path(new_path),
                    vec![
                        variable("context"),
                        slot,
                        lambda(
                            vec![
                                (
                                    pattern("context"),
                                    make_type(UnresolvedTypeData::Named(
                                        chained_dep!("aztec", "context", "Context"),
                                        vec![],
                                        true,
                                    )),
                                ),
                                (
                                    Pattern::Identifier(ident("slot")),
                                    make_type(UnresolvedTypeData::FieldElement),
                                ),
                            ],
                            generate_storage_field_constructor(
                                &(type_ident.clone(), generics.iter().last().unwrap().clone()),
                                variable("slot"),
                            )?,
                        ),
                    ],
                )),
                _ => Ok(call(variable_path(new_path), vec![variable("context"), slot])),
            }
        }
        _ => Err(AztecMacroError::UnsupportedStorageType {
            typ: typ.clone(),
            span: Some(type_ident.span()),
        }),
    }
}

// Generates the Storage implementation block from the Storage struct definition if it does not exist
/// From:
///
/// struct Storage {
///     a_map: Map<Field, SomeStoragePrimitive<ASerializableType>>,
///     a_nested_map: Map<Field, Map<Field, SomeStoragePrimitive<ASerializableType>>>,
///     a_field: SomeStoragePrimitive<ASerializableType>,
/// }
///
/// To:
///
/// impl Storage {
///    fn init(context: Context) -> Self {
///        Storage {
///             a_map: Map::new(context, 0, |context, slot| {
///                 SomeStoragePrimitive::new(context, slot)
///             }),
///             a_nested_map: Map::new(context, 0, |context, slot| {
///                 Map::new(context, slot, |context, slot| {
///                     SomeStoragePrimitive::new(context, slot)
///                })
///            }),
///            a_field: SomeStoragePrimitive::new(context, 0),
///         }
///    }
/// }
///
/// Storage slots are generated as 0 and will be populated using the information from the HIR
/// at a later stage.
pub fn generate_storage_implementation(module: &mut SortedModule) -> Result<(), AztecMacroError> {
    let definition =
        module.types.iter().find(|r#struct| r#struct.name.0.contents == "Storage").unwrap();

    let slot_zero = expression(ExpressionKind::Literal(Literal::Integer(
        FieldElement::from(i128::from(0)),
        false,
    )));

    let field_constructors = definition
        .fields
        .iter()
        .flat_map(|field| {
            generate_storage_field_constructor(field, slot_zero.clone())
                .map(|expression| (field.0.clone(), expression))
        })
        .collect();

    let storage_constructor_statement = make_statement(StatementKind::Expression(expression(
        ExpressionKind::constructor((chained_path!("Storage"), field_constructors)),
    )));

    let init = NoirFunction::normal(FunctionDefinition::normal(
        &ident("init"),
        &vec![],
        &[(
            ident("context"),
            make_type(UnresolvedTypeData::Named(
                chained_dep!("aztec", "context", "Context"),
                vec![],
                true,
            )),
        )],
        &BlockExpression(vec![storage_constructor_statement]),
        &[],
        &return_type(chained_path!("Self")),
    ));

    let storage_impl = TypeImpl {
        object_type: UnresolvedType {
            typ: UnresolvedTypeData::Named(chained_path!("Storage"), vec![], true),
            span: Some(Span::default()),
        },
        type_span: Span::default(),
        generics: vec![],
        methods: vec![(init, Span::default())],
    };
    module.impls.push(storage_impl);

    Ok(())
}

/// Obtains the serialized length of a type that implements the Serialize trait.
fn get_serialized_length(
    traits: &[TraitId],
    typ: &Type,
    interner: &NodeInterner,
) -> Result<u64, AztecMacroError> {
    let (struct_name, maybe_stored_in_state) = match typ {
        Type::Struct(struct_type, generics) => {
            Ok((struct_type.borrow().name.0.contents.clone(), generics.first()))
        }
        _ => Err(AztecMacroError::CouldNotAssignStorageSlots {
            secondary_message: Some("State storage variable must be a struct".to_string()),
        }),
    }?;
    let stored_in_state =
        maybe_stored_in_state.ok_or(AztecMacroError::CouldNotAssignStorageSlots {
            secondary_message: Some("State storage variable must be generic".to_string()),
        })?;

    let is_note = traits.iter().any(|&trait_id| {
        let r#trait = interner.get_trait(trait_id);
        r#trait.name.0.contents == "NoteInterface"
            && !interner.lookup_all_trait_implementations(stored_in_state, trait_id).is_empty()
    });

    // Maps and (private) Notes always occupy a single slot. Someone could store a Note in PublicMutable for whatever reason though.
    if struct_name == "Map" || (is_note && struct_name != "PublicMutable") {
        return Ok(1);
    }

    let serialized_trait_impl_kind = traits
        .iter()
        .find_map(|&trait_id| {
            let r#trait = interner.get_trait(trait_id);
            if r#trait.borrow().name.0.contents == "Serialize"
                && r#trait.borrow().generics.len() == 1
            {
                interner
                    .lookup_all_trait_implementations(stored_in_state, trait_id)
                    .into_iter()
                    .next()
            } else {
                None
            }
        })
        .ok_or(AztecMacroError::CouldNotAssignStorageSlots {
            secondary_message: Some("Stored data must implement Serialize trait".to_string()),
        })?;

    let serialized_trait_impl_id = match serialized_trait_impl_kind {
        TraitImplKind::Normal(trait_impl_id) => Ok(trait_impl_id),
        _ => Err(AztecMacroError::CouldNotAssignStorageSlots { secondary_message: None }),
    }?;

    let serialized_trait_impl_shared = interner.get_trait_implementation(*serialized_trait_impl_id);
    let serialized_trait_impl = serialized_trait_impl_shared.borrow();

    match serialized_trait_impl.trait_generics.first().unwrap() {
        Type::Constant(value) => Ok(*value),
        _ => Err(AztecMacroError::CouldNotAssignStorageSlots { secondary_message: None }),
    }
}

/// Assigns storage slots to the storage struct fields based on the serialized length of the types. This automatic assignment
/// will only trigger if the assigned storage slot is invalid (0 as generated by generate_storage_implementation)
pub fn assign_storage_slots(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    let traits: Vec<_> = collect_traits(context);
    for struct_id in collect_crate_structs(crate_id, context) {
        let interner: &mut NodeInterner = context.def_interner.borrow_mut();
        let r#struct = interner.get_struct(struct_id);
        let file_id = r#struct.borrow().location.file;
        if r#struct.borrow().name.0.contents == "Storage" && r#struct.borrow().id.krate().is_root()
        {
            let init_id = interner
                .lookup_method(
                    &Type::Struct(interner.get_struct(struct_id), vec![]),
                    struct_id,
                    "init",
                    false,
                )
                .ok_or((
                    AztecMacroError::CouldNotAssignStorageSlots {
                        secondary_message: Some(
                            "Storage struct must have an init function".to_string(),
                        ),
                    },
                    file_id,
                ))?;
            let init_function = interner.function(&init_id).block(interner);
            let init_function_statement_id = init_function.statements().first().ok_or((
                AztecMacroError::CouldNotAssignStorageSlots {
                    secondary_message: Some("Init storage statement not found".to_string()),
                },
                file_id,
            ))?;
            let storage_constructor_statement = interner.statement(init_function_statement_id);

            let storage_constructor_expression = match storage_constructor_statement {
                HirStatement::Expression(expression_id) => {
                    match interner.expression(&expression_id) {
                        HirExpression::Constructor(hir_constructor_expression) => {
                            Ok(hir_constructor_expression)
                        }
                        _ => Err((AztecMacroError::CouldNotAssignStorageSlots {
                            secondary_message: Some(
                                "Storage constructor statement must be a constructor expression"
                                    .to_string(),
                            ),
                        }, file_id))
                    }
                }
                _ => Err((
                    AztecMacroError::CouldNotAssignStorageSlots {
                        secondary_message: Some(
                            "Storage constructor statement must be an expression".to_string(),
                        ),
                    },
                    file_id,
                )),
            }?;

            let mut storage_slot: u64 = 1;
            for (index, (_, expr_id)) in storage_constructor_expression.fields.iter().enumerate() {
                let fields = r#struct.borrow().get_fields(&[]);
                let (_, field_type) = fields.get(index).unwrap();
                let new_call_expression = match interner.expression(expr_id) {
                    HirExpression::Call(hir_call_expression) => Ok(hir_call_expression),
                    _ => Err((
                        AztecMacroError::CouldNotAssignStorageSlots {
                            secondary_message: Some(
                                "Storage field initialization expression is not a call expression"
                                    .to_string(),
                            ),
                        },
                        file_id,
                    )),
                }?;

                let slot_arg_expression = interner.expression(&new_call_expression.arguments[1]);

                let current_storage_slot = match slot_arg_expression {
                    HirExpression::Literal(HirLiteral::Integer(slot, _)) => Ok(slot.to_u128()),
                    _ => Err((
                        AztecMacroError::CouldNotAssignStorageSlots {
                            secondary_message: Some(
                                "Storage slot argument expression must be a literal integer"
                                    .to_string(),
                            ),
                        },
                        file_id,
                    )),
                }?;

                if current_storage_slot != 0 {
                    continue;
                }

                let type_serialized_len = get_serialized_length(&traits, field_type, interner)
                    .map_err(|err| (err, file_id))?;
                interner.update_expression(new_call_expression.arguments[1], |expr| {
                    *expr = HirExpression::Literal(HirLiteral::Integer(
                        FieldElement::from(u128::from(storage_slot)),
                        false,
                    ));
                });

                storage_slot += type_serialized_len;
            }
        }
    }
    Ok(())
}
