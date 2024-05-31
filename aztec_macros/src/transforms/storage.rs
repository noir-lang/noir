use acvm::acir::AcirField;
use noirc_errors::Span;
use noirc_frontend::ast::{
    BlockExpression, Expression, ExpressionKind, FunctionDefinition, Ident, Literal, NoirFunction,
    NoirStruct, Pattern, StatementKind, TypeImpl, UnresolvedType, UnresolvedTypeData,
};
use noirc_frontend::{
    graph::CrateId,
    macros_api::{
        FieldElement, FileId, HirContext, HirExpression, HirLiteral, HirStatement, NodeInterner,
    },
    node_interner::TraitId,
    parse_program,
    parser::SortedModule,
    token::SecondaryAttribute,
    Type,
};

use crate::{
    chained_path,
    utils::{
        ast_utils::{
            call, expression, ident, ident_path, is_custom_attribute, lambda, make_statement,
            make_type, pattern, return_type, variable, variable_path,
        },
        errors::AztecMacroError,
        hir_utils::{
            collect_crate_structs, collect_traits, get_contract_module_data, get_serialized_length,
        },
    },
};

// Check to see if the user has defined a storage struct
pub fn check_for_storage_definition(
    module: &SortedModule,
) -> Result<Option<String>, AztecMacroError> {
    let result: Vec<&NoirStruct> = module
        .types
        .iter()
        .filter(|r#struct| {
            r#struct.attributes.iter().any(|attr| is_custom_attribute(attr, "aztec(storage)"))
        })
        .collect();
    if result.len() > 1 {
        return Err(AztecMacroError::MultipleStorageDefinitions {
            span: result.first().map(|res| res.name.span()),
        });
    }
    Ok(result.iter().map(|&r#struct| r#struct.name.0.contents.clone()).next())
}

// Injects the Context generic in each of the Storage struct fields to avoid boilerplate,
// taking maps into account (including nested maps)
fn inject_context_in_storage_field(field: &mut UnresolvedType) -> Result<(), AztecMacroError> {
    match &mut field.typ {
        UnresolvedTypeData::Named(path, generics, _) => {
            generics.push(make_type(UnresolvedTypeData::Named(
                ident_path("Context"),
                vec![],
                false,
            )));
            match path.segments.last().unwrap().0.contents.as_str() {
                "Map" => inject_context_in_storage_field(&mut generics[1]),
                _ => Ok(()),
            }
        }
        _ => Err(AztecMacroError::CouldNotInjectContextGenericInStorage {
            secondary_message: Some(format!("Unsupported type: {:?}", field.typ)),
        }),
    }
}

// Injects the Context generic in the storage struct to avoid boilerplate
// Transforms this:
// struct Storage {
//     a_var: SomeStoragePrimitive<ASerializableType>,
//     a_map: Map<Field, SomeStoragePrimitive<ASerializableType>>,
// }
//
// Into this:
//
// struct Storage<Context> {
//     a_var: SomeStoragePrimitive<ASerializableType, Context>,
//     a_map: Map<Field, SomeStoragePrimitive<ASerializableType, Context>, Context>,
// }
pub fn inject_context_in_storage(module: &mut SortedModule) -> Result<(), AztecMacroError> {
    let storage_struct = module
        .types
        .iter_mut()
        .find(|r#struct| {
            r#struct.attributes.iter().any(|attr| is_custom_attribute(attr, "aztec(storage)"))
        })
        .unwrap();
    storage_struct.generics.push(ident("Context"));
    storage_struct
        .fields
        .iter_mut()
        .map(|(_, field)| inject_context_in_storage_field(field))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(())
}

// Check to see if the user has defined an impl for the storage struct
pub fn check_for_storage_implementation(
    module: &SortedModule,
    storage_struct_name: &String,
) -> bool {
    module.impls.iter().any(|r#impl| match &r#impl.object_type.typ {
        UnresolvedTypeData::Named(path, _, _) => {
            path.segments.last().is_some_and(|segment| segment.0.contents == *storage_struct_name)
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
                            // This lambda will be equivalent to the following
                            // | context, slot | { T::new(context, slot) }
                            // Since the `new` function has type bindings for its arguments, we don't specify the types
                            // of either context nor slot, and avoid that way having to deal with the generic context
                            // type.
                            vec![
                                (pattern("context"), make_type(UnresolvedTypeData::Unspecified)),
                                (
                                    Pattern::Identifier(ident("slot")),
                                    make_type(UnresolvedTypeData::Unspecified),
                                ),
                            ],
                            generate_storage_field_constructor(
                                // Map is expected to have three generic parameters: key, value and context (i.e.
                                // Map<K, V, Context>. Here `get(1)` fetches the value type.
                                &(type_ident.clone(), generics.get(1).unwrap().clone()),
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
/// struct Storage<Context> {
///     a_map: Map<Field, SomeStoragePrimitive<ASerializableType, Context>, Context>,
///     a_nested_map: Map<Field, Map<Field, SomeStoragePrimitive<ASerializableType, Context>, Context>, Context>,
///     a_field: SomeStoragePrimitive<ASerializableType, Context>,
/// }
///
/// To:
///
/// impl<Context> Storage<Contex> {
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
pub fn generate_storage_implementation(
    module: &mut SortedModule,
    storage_struct_name: &String,
) -> Result<(), AztecMacroError> {
    let definition = module
        .types
        .iter()
        .find(|r#struct| r#struct.name.0.contents == *storage_struct_name)
        .unwrap();

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
        ExpressionKind::constructor((chained_path!(storage_struct_name), field_constructors)),
    )));

    // This is the type over which the impl is generic.
    let generic_context_ident = ident("Context");
    let generic_context_type =
        make_type(UnresolvedTypeData::Named(ident_path("Context"), vec![], true));

    let init = NoirFunction::normal(FunctionDefinition::normal(
        &ident("init"),
        &vec![],
        &[(ident("context"), generic_context_type.clone())],
        &BlockExpression { statements: vec![storage_constructor_statement] },
        &[],
        &return_type(chained_path!("Self")),
    ));

    let storage_impl = TypeImpl {
        object_type: UnresolvedType {
            typ: UnresolvedTypeData::Named(
                chained_path!(storage_struct_name),
                vec![generic_context_type.clone()],
                true,
            ),
            span: Some(Span::default()),
        },
        type_span: Span::default(),
        generics: vec![generic_context_ident],

        methods: vec![(init, Span::default())],
    };
    module.impls.push(storage_impl);

    Ok(())
}

/// Obtains the serialized length of a type that implements the Serialize trait.
pub fn get_storage_serialized_length(
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

    let is_note = match stored_in_state {
        Type::Struct(typ, _) => interner
            .struct_attributes(&typ.borrow().id)
            .iter()
            .any(|attr| is_custom_attribute(attr, "aztec(note)")),
        _ => false,
    };

    // Maps and (private) Notes always occupy a single slot. Someone could store a Note in PublicMutable for whatever reason though.
    if struct_name == "Map" || (is_note && struct_name != "PublicMutable") {
        return Ok(1);
    }

    get_serialized_length(traits, "Serialize", stored_in_state, interner).map_err(|err| {
        AztecMacroError::CouldNotAssignStorageSlots { secondary_message: Some(err.primary_message) }
    })
}

/// Assigns storage slots to the storage struct fields based on the serialized length of the types. This automatic assignment
/// will only trigger if the assigned storage slot is invalid (0 as generated by generate_storage_implementation)
pub fn assign_storage_slots(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    let traits: Vec<_> = collect_traits(context);
    if let Some((_, _, file_id)) = get_contract_module_data(context, crate_id) {
        let maybe_storage_struct =
            collect_crate_structs(crate_id, context).iter().find_map(|struct_id| {
                let r#struct = context.def_interner.get_struct(*struct_id);
                let attributes = context.def_interner.struct_attributes(struct_id);
                if attributes.iter().any(|attr| is_custom_attribute(attr, "aztec(storage)"))
                    && r#struct.borrow().id.krate() == *crate_id
                {
                    Some(r#struct)
                } else {
                    None
                }
            });

        let maybe_storage_layout =
            context.def_interner.get_all_globals().iter().find_map(|global_info| {
                let statement = context.def_interner.get_global_let_statement(global_info.id);
                if statement.clone().is_some_and(|stmt| {
                    stmt.attributes
                        .iter()
                        .any(|attr| *attr == SecondaryAttribute::Abi("storage".to_string()))
                }) {
                    let expr = context.def_interner.expression(&statement.unwrap().expression);
                    match expr {
                        HirExpression::Constructor(hir_constructor_expression) => {
                            if hir_constructor_expression.r#type.borrow().id.krate() == *crate_id {
                                Some(hir_constructor_expression)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            });

        if let (Some(storage_struct), Some(storage_layout)) =
            (maybe_storage_struct, maybe_storage_layout)
        {
            let init_id = context
                .def_interner
                .lookup_method(
                    &Type::Struct(
                        context.def_interner.get_struct(storage_struct.borrow().id),
                        vec![],
                    ),
                    storage_struct.borrow().id,
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
            let init_function =
                context.def_interner.function(&init_id).block(&context.def_interner);
            let init_function_statement_id = init_function.statements().first().ok_or((
                AztecMacroError::CouldNotAssignStorageSlots {
                    secondary_message: Some("Init storage statement not found".to_string()),
                },
                file_id,
            ))?;
            let storage_constructor_statement =
                context.def_interner.statement(init_function_statement_id);

            let storage_constructor_expression = match storage_constructor_statement {
                HirStatement::Expression(expression_id) => {
                    match context.def_interner.expression(&expression_id) {
                    HirExpression::Constructor(hir_constructor_expression) => {
                        Ok(hir_constructor_expression)
                    }
                    _ => Err((
                        AztecMacroError::CouldNotAssignStorageSlots {
                            secondary_message: Some(
                                "Storage constructor statement must be a constructor expression"
                                    .to_string(),
                            ),
                        },
                        file_id,
                    )),
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
                let fields = storage_struct
                    .borrow()
                    .get_fields(&storage_constructor_expression.struct_generics);
                let (field_name, field_type) = fields.get(index).unwrap();
                let new_call_expression = match context.def_interner.expression(expr_id) {
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

                let slot_arg_expression =
                    context.def_interner.expression(&new_call_expression.arguments[1]);

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

                let storage_layout_field =
                    storage_layout.fields.iter().find(|field| field.0 .0.contents == *field_name);

                let storage_layout_slot_expr_id =
                    if let Some((_, expr_id)) = storage_layout_field {
                        let expr = context.def_interner.expression(expr_id);
                        if let HirExpression::Constructor(storage_layout_field_storable_expr) = expr
                        {
                            storage_layout_field_storable_expr.fields.iter().find_map(
                                |(field, expr_id)| {
                                    if field.0.contents == "slot" {
                                        Some(*expr_id)
                                    } else {
                                        None
                                    }
                                },
                            )
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                    .ok_or((
                        AztecMacroError::CouldNotAssignStorageSlots {
                            secondary_message: Some(format!(
                                "Storage layout field ({}) not found or has an incorrect type",
                                field_name
                            )),
                        },
                        file_id,
                    ))?;

                let new_storage_slot = if current_storage_slot == 0 {
                    u128::from(storage_slot)
                } else {
                    current_storage_slot
                };

                let type_serialized_len =
                    get_storage_serialized_length(&traits, field_type, &context.def_interner)
                        .map_err(|err| (err, file_id))?;

                context.def_interner.update_expression(new_call_expression.arguments[1], |expr| {
                    *expr = HirExpression::Literal(HirLiteral::Integer(
                        FieldElement::from(new_storage_slot),
                        false,
                    ))
                });

                context.def_interner.update_expression(storage_layout_slot_expr_id, |expr| {
                    *expr = HirExpression::Literal(HirLiteral::Integer(
                        FieldElement::from(new_storage_slot),
                        false,
                    ))
                });

                storage_slot += type_serialized_len;
            }
        }
    }

    Ok(())
}

pub fn generate_storage_layout(
    module: &mut SortedModule,
    storage_struct_name: String,
) -> Result<(), AztecMacroError> {
    let definition = module
        .types
        .iter()
        .find(|r#struct| r#struct.name.0.contents == *storage_struct_name)
        .unwrap();

    let mut generic_args = vec![];
    let mut storable_fields = vec![];
    let mut storable_fields_impl = vec![];

    definition.fields.iter().enumerate().for_each(|(index, (field_ident, field_type))| {
        storable_fields.push(format!("{}: dep::aztec::prelude::Storable<N{}>", field_ident, index));
        generic_args.push(format!("N{}", index));
        storable_fields_impl.push(format!(
            "{}: dep::aztec::prelude::Storable {{ slot: 0, typ: \"{}\" }}",
            field_ident,
            field_type.to_string().replace("plain::", "")
        ));
    });

    let storage_fields_source = format!(
        "
        struct StorageLayout<{}> {{
            {}
        }}

        #[abi(storage)]
        global STORAGE_LAYOUT = StorageLayout {{
            {}
        }};
    ",
        generic_args.join(", "),
        storable_fields.join(",\n"),
        storable_fields_impl.join(",\n")
    );

    let (struct_ast, errors) = parse_program(&storage_fields_source);
    if !errors.is_empty() {
        dbg!(errors);
        return Err(AztecMacroError::CouldNotExportStorageLayout {
            secondary_message: Some("Failed to parse Noir macro code (struct StorageLayout). This is either a bug in the compiler or the Noir macro code".to_string()),
            span: None
        });
    }

    let mut struct_ast = struct_ast.into_sorted();
    module.types.push(struct_ast.types.pop().unwrap());
    module.globals.push(struct_ast.globals.pop().unwrap());

    Ok(())
}
