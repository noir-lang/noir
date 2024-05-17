use convert_case::{Case, Casing};
use noirc_errors::Span;
use noirc_frontend::ast::{self, FunctionKind};
use noirc_frontend::ast::{
    BlockExpression, ConstrainKind, ConstrainStatement, Expression, ExpressionKind,
    ForLoopStatement, ForRange, FunctionReturnType, Ident, Literal, NoirFunction, NoirStruct,
    Param, PathKind, Pattern, Signedness, Statement, StatementKind, UnresolvedType,
    UnresolvedTypeData, Visibility,
};

use noirc_frontend::{macros_api::FieldElement, parse_program};

use crate::{
    chained_dep, chained_path,
    utils::{
        ast_utils::{
            assignment, assignment_with_type, call, cast, expression, ident, ident_path,
            index_array, make_eq, make_statement, make_type, method_call, mutable_assignment,
            mutable_reference, path, return_type, variable, variable_ident, variable_path,
        },
        errors::AztecMacroError,
    },
};

// If it does, it will insert the following things:
/// - A new Input that is provided for a kernel app circuit, named: {Public/Private}ContextInputs
/// - Hashes all of the function input variables
///     - This instantiates a helper function
pub fn transform_function(
    ty: &str,
    func: &mut NoirFunction,
    storage_struct_name: Option<String>,
    is_initializer: bool,
    insert_init_check: bool,
    is_internal: bool,
) -> Result<(), AztecMacroError> {
    let context_name = format!("{}Context", ty);
    let inputs_name = format!("{}ContextInputs", ty);
    let return_type_name = format!("{}CircuitPublicInputs", ty);
    let is_avm = ty == "Avm";
    let is_private = ty == "Private";

    // Add check that msg sender equals this address and flag function as internal
    if is_internal {
        let is_internal_check = create_internal_check(func.name());
        func.def.body.statements.insert(0, is_internal_check);
    }

    // Add initialization check
    if insert_init_check {
        let init_check = create_init_check(ty);
        func.def.body.statements.insert(0, init_check);
    }

    // Add assertion for initialization arguments and sender
    if is_initializer {
        func.def.body.statements.insert(0, create_assert_initializer(ty));
    }

    // Add access to the storage struct
    if let Some(storage_struct_name) = storage_struct_name {
        let storage_def = abstract_storage(storage_struct_name, &ty.to_lowercase(), false);
        func.def.body.statements.insert(0, storage_def);
    }

    // Insert the context creation as the first action
    let create_context = if !is_avm {
        create_context(&context_name, &func.def.parameters)?
    } else {
        create_context_avm()?
    };
    func.def.body.statements.splice(0..0, (create_context).iter().cloned());

    // Add the inputs to the params
    let input = create_inputs(&inputs_name);
    func.def.parameters.insert(0, input);

    // Abstract return types such that they get added to the kernel's return_values
    if !is_avm {
        if let Some(return_values_statements) = abstract_return_values(func)? {
            // In case we are pushing return values to the context, we remove the statement that originated it
            // This avoids running duplicate code, since blocks like if/else can be value returning statements
            func.def.body.statements.pop();
            // Add the new return statement
            func.def.body.statements.extend(return_values_statements);
        }
    }

    // Before returning mark the contract as initialized
    if is_initializer {
        let mark_initialized = create_mark_as_initialized(ty);
        func.def.body.statements.push(mark_initialized);
    }

    // Push the finish method call to the end of the function
    if !is_avm {
        let finish_def = create_context_finish();
        func.def.body.statements.push(finish_def);
    }

    // The AVM doesn't need a return type yet.
    if !is_avm {
        let return_type = create_return_type(&return_type_name);
        func.def.return_type = return_type;
        func.def.return_visibility = Visibility::Public;
    } else {
        func.def.return_visibility = Visibility::Public;
    }

    // Public functions should have unconstrained auto-inferred
    func.def.is_unconstrained = matches!(ty, "Public" | "Avm");

    // Private functions need to be recursive
    if is_private {
        func.kind = FunctionKind::Recursive;
    }

    Ok(())
}

// Generates a global struct containing the original (before transform_function gets executed) function abi that gets exported
// in the contract artifact after compilation. The abi will be later used to decode the function return values in the simulator.
pub fn export_fn_abi(
    types: &mut Vec<NoirStruct>,
    func: &NoirFunction,
) -> Result<(), AztecMacroError> {
    let mut parameters_struct_source: Option<&str> = None;

    let struct_source = format!(
        "
        struct {}_parameters {{
            {}
        }}
    ",
        func.name(),
        func.parameters()
            .iter()
            .map(|param| {
                let param_name = match param.pattern.clone() {
                    Pattern::Identifier(ident) => Ok(ident.0.contents),
                    _ => Err(AztecMacroError::CouldNotExportFunctionAbi {
                        span: Some(param.span),
                        secondary_message: Some(
                            "Only identifier patterns are supported".to_owned(),
                        ),
                    }),
                };

                format!(
                    "{}: {}",
                    param_name.unwrap(),
                    param.typ.typ.to_string().replace("plain::", "")
                )
            })
            .collect::<Vec<String>>()
            .join(",\n"),
    );

    if !func.parameters().is_empty() {
        parameters_struct_source = Some(&struct_source);
    }

    let mut program = String::new();

    let parameters = if let Some(parameters_struct_source) = parameters_struct_source {
        program.push_str(parameters_struct_source);
        format!("parameters: {}_parameters,\n", func.name())
    } else {
        "".to_string()
    };

    let return_type_str = func.return_type().typ.to_string().replace("plain::", "");
    let return_type = if return_type_str != "()" {
        format!("return_type: {},\n", return_type_str)
    } else {
        "".to_string()
    };

    let export_struct_source = format!(
        "
        #[abi(functions)]
        struct {}_abi {{
            {}{}
        }}",
        func.name(),
        parameters,
        return_type
    );

    program.push_str(&export_struct_source);

    let (ast, errors) = parse_program(&program);
    if !errors.is_empty() {
        return Err(AztecMacroError::CouldNotExportFunctionAbi {
            span: None,
            secondary_message: Some(
                format!("Failed to parse Noir macro code (struct {}_abi). This is either a bug in the compiler or the Noir macro code", func.name())
            )
        });
    }

    let sorted_ast = ast.into_sorted();
    types.extend(sorted_ast.types);
    Ok(())
}

/// Transform Unconstrained
///
/// Inserts the following code at the beginning of an unconstrained function
/// ```noir
/// let storage = Storage::init(Context::none());
/// ```
///
/// This will allow developers to access their contract' storage struct in unconstrained functions
pub fn transform_unconstrained(func: &mut NoirFunction, storage_struct_name: String) {
    func.def
        .body
        .statements
        .insert(0, abstract_storage(storage_struct_name, "Unconstrained", true));
}

/// Helper function that returns what the private context would look like in the ast
/// This should make it available to be consumed within aztec private annotated functions.
///
/// The replaced code:
/// ```noir
/// /// Before
/// fn foo(inputs: PrivateContextInputs) {
///    // ...
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() {
///   // ...
/// }
fn create_inputs(ty: &str) -> Param {
    let context_ident = ident("inputs");
    let context_pattern = Pattern::Identifier(context_ident);

    let path_snippet = ty.to_case(Case::Snake); // e.g. private_context_inputs
    let type_path = chained_dep!("aztec", "context", "inputs", &path_snippet, ty);

    let context_type = make_type(UnresolvedTypeData::Named(type_path, vec![], true));
    let visibility = Visibility::Private;

    Param { pattern: context_pattern, typ: context_type, visibility, span: Span::default() }
}

/// Creates an initialization check to ensure that the contract has been initialized, meant to
/// be injected as the first statement of any function after the context has been created.
///
/// ```noir
/// assert_is_initialized(&mut context);
/// ```
fn create_init_check(ty: &str) -> Statement {
    let fname = format!("assert_is_initialized_{}", ty.to_case(Case::Snake));
    make_statement(StatementKind::Expression(call(
        variable_path(chained_dep!("aztec", "initializer", &fname)),
        vec![mutable_reference("context")],
    )))
}

/// Creates a call to mark_as_initialized which emits the initialization nullifier, meant to
/// be injected as the last statement before returning in a constructor.
///
/// ```noir
/// mark_as_initialized(&mut context);
/// ```
fn create_mark_as_initialized(ty: &str) -> Statement {
    let fname = format!("mark_as_initialized_{}", ty.to_case(Case::Snake));
    make_statement(StatementKind::Expression(call(
        variable_path(chained_dep!("aztec", "initializer", &fname)),
        vec![mutable_reference("context")],
    )))
}

/// Creates a check for internal functions ensuring that the caller is self.
///
/// ```noir
/// assert(context.msg_sender() == context.this_address(), "Function can only be called internally");
/// ```
fn create_internal_check(fname: &str) -> Statement {
    make_statement(StatementKind::Constrain(ConstrainStatement(
        make_eq(
            method_call(variable("context"), "msg_sender", vec![]),
            method_call(variable("context"), "this_address", vec![]),
        ),
        Some(expression(ExpressionKind::Literal(Literal::Str(format!(
            "Function {} can only be called internally",
            fname
        ))))),
        ConstrainKind::Assert,
    )))
}

/// Creates a call to assert_initialization_matches_address_preimage to be inserted
/// in the initializer. Checks that the args and sender to the initializer match the
/// commitments from the address preimage.
///
/// ```noir
/// assert_initialization_matches_address_preimage(context);
/// ```
fn create_assert_initializer(ty: &str) -> Statement {
    let fname =
        format!("assert_initialization_matches_address_preimage_{}", ty.to_case(Case::Snake));
    make_statement(StatementKind::Expression(call(
        variable_path(chained_dep!("aztec", "initializer", &fname)),
        vec![variable("context")],
    )))
}

fn serialize_to_hasher(
    identifier: &Ident,
    typ: &UnresolvedTypeData,
    hasher_name: &str,
) -> Option<Vec<Statement>> {
    let mut statements = Vec::new();

    // Match the type to determine the padding to do
    match typ {
        // `{hasher_name}.extend_from_array({ident}.serialize())`
        UnresolvedTypeData::Named(..) => {
            statements.push(add_struct_to_hasher(identifier, hasher_name));
        }
        UnresolvedTypeData::Array(_, arr_type) => {
            statements.push(add_array_to_hasher(identifier, arr_type, hasher_name));
        }
        // `{hasher_name}.push({ident})`
        UnresolvedTypeData::FieldElement => {
            statements.push(add_field_to_hasher(identifier, hasher_name));
        }
        // Add the integer to the bounded vec, casted to a field
        // `{hasher_name}.push({ident} as Field)`
        UnresolvedTypeData::Integer(..) | UnresolvedTypeData::Bool => {
            statements.push(add_cast_to_hasher(identifier, hasher_name));
        }
        UnresolvedTypeData::String(..) => {
            let (var_bytes, id) = str_to_bytes(identifier);
            statements.push(var_bytes);
            statements.push(add_array_to_hasher(
                &id,
                &UnresolvedType {
                    typ: UnresolvedTypeData::Integer(
                        Signedness::Unsigned,
                        ast::IntegerBitSize::ThirtyTwo,
                    ),
                    span: None,
                },
                hasher_name,
            ))
        }
        _ => return None,
    };
    Some(statements)
}

/// Creates the private context object to be accessed within the function, the parameters need to be extracted to be
/// appended into the args hash object.
///
/// The replaced code:
/// ```noir
/// #[aztec(private)]
/// fn foo(structInput: SomeStruct, arrayInput: [u8; 10], fieldInput: Field) -> Field {
///     // Create the hasher object
///     let mut hasher = Hasher::new();
///
///     // struct inputs call serialize on them to add an array of fields
///     hasher.add_multiple(structInput.serialize());
///
///     // Array inputs are iterated over and each element is added to the hasher (as a field)
///     for i in 0..arrayInput.len() {
///         hasher.add(arrayInput[i] as Field);
///     }
///     // Field inputs are added to the hasher
///     hasher.add({ident});
///
///     // Create the context
///     // The inputs (injected by this `create_inputs`) and completed hash object are passed to the context
///     let mut context = PrivateContext::new(inputs, hasher.hash());
/// }
/// ```
fn create_context(ty: &str, params: &[Param]) -> Result<Vec<Statement>, AztecMacroError> {
    let mut injected_statements: Vec<Statement> = vec![];

    let hasher_name = "args_hasher";

    // `let mut args_hasher = Hasher::new();`
    let let_hasher = mutable_assignment(
        hasher_name, // Assigned to
        call(
            variable_path(chained_dep!("aztec", "hash", "ArgsHasher", "new")), // Path
            vec![],                                                            // args
        ),
    );

    // Completes: `let mut args_hasher = Hasher::new();`
    injected_statements.push(let_hasher);

    // Iterate over each of the function parameters, adding to them to the hasher
    for Param { pattern, typ, span, .. } in params {
        match pattern {
            Pattern::Identifier(identifier) => {
                // Match the type to determine the padding to do
                let unresolved_type = &typ.typ;
                injected_statements.extend(
                    serialize_to_hasher(identifier, unresolved_type, hasher_name).ok_or_else(
                        || AztecMacroError::UnsupportedFunctionArgumentType {
                            typ: unresolved_type.clone(),
                            span: *span,
                        },
                    )?,
                );
            }
            _ => todo!(), // Maybe unreachable?
        }
    }

    // Create the inputs to the context
    let inputs_expression = variable("inputs");
    // `args_hasher.hash()`
    let hash_call = method_call(
        variable(hasher_name), // variable
        "hash",                // method name
        vec![],                // args
    );

    let path_snippet = ty.to_case(Case::Snake); // e.g. private_context

    // let mut context = {ty}::new(inputs, hash);
    let let_context = mutable_assignment(
        "context", // Assigned to
        call(
            variable_path(chained_dep!("aztec", "context", &path_snippet, ty, "new")), // Path
            vec![inputs_expression, hash_call],                                        // args
        ),
    );
    injected_statements.push(let_context);

    // Return all expressions that will be injected by the hasher
    Ok(injected_statements)
}

/// Creates the private context object to be accessed within the function, the parameters need to be extracted to be
/// appended into the args hash object.
///
/// The replaced code:
/// ```noir
/// #[aztec(public-vm)]
/// fn foo(inputs: AvmContextInputs, ...) -> Field {
///     let mut context = AvmContext::new(inputs);
/// }
/// ```
fn create_context_avm() -> Result<Vec<Statement>, AztecMacroError> {
    let mut injected_expressions: Vec<Statement> = vec![];

    // Create the inputs to the context
    let ty = "AvmContext";
    let inputs_expression = variable("inputs");
    let path_snippet = ty.to_case(Case::Snake); // e.g. private_context

    // let mut context = {ty}::new(inputs, hash);
    let let_context = mutable_assignment(
        "context", // Assigned to
        call(
            variable_path(chained_dep!("aztec", "context", &path_snippet, ty, "new")), // Path
            vec![inputs_expression],                                                   // args
        ),
    );
    injected_expressions.push(let_context);

    // Return all expressions that will be injected by the hasher
    Ok(injected_expressions)
}

/// Abstract Return Type
///
/// This function intercepts the function's current return type and replaces it with pushes to a hasher
/// that will be used to generate the returns hash for the kernel.
///
/// The replaced code:
/// ```noir
/// /// Before
/// #[aztec(private)]
/// fn foo() -> Field {
///     // ...
///    let my_return_value: Field = 10;
///    my_return_value
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() -> protocol_types::abis::private_circuit_public_inputs::PrivateCircuitPublicInputs {
///   // ...
///   let my_return_value: Field = 10;
///   let macro__returned__values = my_return_value;
///   let mut returns_hasher = ArgsHasher::new();
///   returns_hasher.add(macro__returned__values);
///   context.set_return_hash(returns_hasher);
/// }
/// ```
/// Similarly; Structs will be pushed to the hasher, after serialize() is called on them.
/// Arrays will be iterated over and each element will be pushed to the hasher.
/// Any primitive type that can be cast will be casted to a field and pushed to the hasher.
fn abstract_return_values(func: &NoirFunction) -> Result<Option<Vec<Statement>>, AztecMacroError> {
    let current_return_type = func.return_type().typ;

    // Short circuit if the function doesn't return anything
    match current_return_type {
        UnresolvedTypeData::Unit | UnresolvedTypeData::Unspecified => return Ok(None),
        _ => (),
    }

    let Some(last_statement) = func.def.body.statements.last() else {
        return Ok(None);
    };

    // TODO: support tuples here and in inputs -> convert into an issue
    // Check if the return type is an expression, if it is, we can handle it
    match last_statement {
        Statement { kind: StatementKind::Expression(expression), .. } => {
            let return_value_name = "macro__returned__values";
            let hasher_name = "returns_hasher";

            let mut replacement_statements = vec![
                assignment_with_type(
                    return_value_name, // Assigned to
                    current_return_type.clone(),
                    expression.clone(),
                ),
                mutable_assignment(
                    hasher_name, // Assigned to
                    call(
                        variable_path(chained_dep!("aztec", "hash", "ArgsHasher", "new")), // Path
                        vec![],                                                            // args
                    ),
                ),
            ];

            let serialization_statements =
                serialize_to_hasher(&ident(return_value_name), &current_return_type, hasher_name)
                    .ok_or_else(|| AztecMacroError::UnsupportedFunctionReturnType {
                    typ: current_return_type.clone(),
                    span: func.return_type().span.unwrap_or_default(),
                })?;

            replacement_statements.extend(serialization_statements);

            replacement_statements.push(make_statement(StatementKind::Semi(method_call(
                variable("context"),
                "set_return_hash",
                vec![variable(hasher_name)],
            ))));

            Ok(Some(replacement_statements))
        }
        _ => Ok(None),
    }
}

/// Abstract storage
///
/// For private functions:
/// ```noir
/// #[aztec(private)]
/// fn lol() {
///     let storage = Storage::init(Context::private(context));
/// }
/// ```
///
/// For public functions:
/// ```noir
/// #[aztec(public)]
/// fn lol() {
///    let storage = Storage::init(Context::public(context));
/// }
/// ```
///
/// For unconstrained functions:
/// ```noir
/// unconstrained fn lol() {
///   let storage = Storage::init(Context::none());
/// }
fn abstract_storage(storage_struct_name: String, typ: &str, unconstrained: bool) -> Statement {
    let init_context_call = if unconstrained {
        call(
            variable_path(chained_dep!("aztec", "context", "Context", "none")), // Path
            vec![],                                                             // args
        )
    } else {
        call(
            variable_path(chained_dep!("aztec", "context", "Context", typ)), // Path
            vec![mutable_reference("context")],                              // args
        )
    };

    assignment(
        "storage", // Assigned to
        call(
            variable_path(chained_path!(storage_struct_name.as_str(), "init")), // Path
            vec![init_context_call],                                            // args
        ),
    )
}

/// Create Return Type
///
/// Public functions return protocol_types::abis::public_circuit_public_inputs::PublicCircuitPublicInputs while
/// private functions return protocol_types::abis::private_circuit_public_inputs::::PrivateCircuitPublicInputs
///
/// This call constructs an ast token referencing the above types
/// The name is set in the function above `transform`, hence the
/// whole token name is passed in
///
/// The replaced code:
/// ```noir
///
/// /// Before
/// fn foo() -> protocol_types::abis::private_circuit_public_inputs::PrivateCircuitPublicInputs {
///    // ...
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() {
///  // ...
/// }
fn create_return_type(ty: &str) -> FunctionReturnType {
    let path_snippet = ty.to_case(Case::Snake); // e.g. private_circuit_public_inputs or public_circuit_public_inputs
    let return_path = chained_dep!("aztec", "protocol_types", "abis", &path_snippet, ty);
    return_type(return_path)
}

/// Create Context Finish
///
/// Each aztec function calls `context.finish()` at the end of a function
/// to return values required by the kernel.
///
/// The replaced code:
/// ```noir
/// /// Before
/// fn foo() -> protocol_types::abis::private_circuit_public_inputs::PrivateCircuitPublicInputs {
///   // ...
///  context.finish()
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() {
///  // ...
/// }
fn create_context_finish() -> Statement {
    let method_call = method_call(
        variable("context"), // variable
        "finish",            // method name
        vec![],              // args
    );
    make_statement(StatementKind::Expression(method_call))
}

//
//                 Methods to create hasher inputs
//

fn add_struct_to_hasher(identifier: &Ident, hasher_name: &str) -> Statement {
    // If this is a struct, we call serialize and add the array to the hasher
    let serialized_call = method_call(
        variable_path(path(identifier.clone())), // variable
        "serialize",                             // method name
        vec![],                                  // args
    );

    make_statement(StatementKind::Semi(method_call(
        variable(hasher_name), // variable
        "add_multiple",        // method name
        vec![serialized_call], // args
    )))
}

fn str_to_bytes(identifier: &Ident) -> (Statement, Ident) {
    // let identifier_as_bytes = identifier.as_bytes();
    let var = variable_ident(identifier.clone());
    let contents = if let ExpressionKind::Variable(p) = &var.kind {
        p.segments.first().cloned().unwrap_or_else(|| panic!("No segments")).0.contents
    } else {
        panic!("Unexpected identifier type")
    };
    let bytes_name = format!("{}_bytes", contents);
    let var_bytes = assignment(&bytes_name, method_call(var, "as_bytes", vec![]));
    let id = Ident::new(bytes_name, Span::default());

    (var_bytes, id)
}

fn create_loop_over(var: Expression, loop_body: Vec<Statement>) -> Statement {
    // If this is an array of primitive types (integers / fields) we can add them each to the hasher
    // casted to a field
    let span = var.span;

    // `array.len()`
    let end_range_expression = method_call(
        var,    // variable
        "len",  // method name
        vec![], // args
    );

    // What will be looped over

    // - `hasher.add({ident}[i] as Field)`
    let for_loop_block =
        expression(ExpressionKind::Block(BlockExpression { statements: loop_body }));

    // `for i in 0..{ident}.len()`
    make_statement(StatementKind::For(ForLoopStatement {
        range: ForRange::Range(
            expression(ExpressionKind::Literal(Literal::Integer(
                FieldElement::from(i128::from(0)),
                false,
            ))),
            end_range_expression,
        ),
        identifier: ident("i"),
        block: for_loop_block,
        span,
    }))
}

fn add_array_to_hasher(
    identifier: &Ident,
    arr_type: &UnresolvedType,
    hasher_name: &str,
) -> Statement {
    // If this is an array of primitive types (integers / fields) we can add them each to the hasher
    // casted to a field

    // Wrap in the semi thing - does that mean ended with semi colon?
    // `hasher.add({ident}[i] as Field)`

    let arr_index = index_array(identifier.clone(), "i");
    let (add_expression, hasher_method_name) = match arr_type.typ {
        UnresolvedTypeData::Named(..) => {
            let hasher_method_name = "add_multiple".to_owned();
            let call = method_call(
                // All serialize on each element
                arr_index,   // variable
                "serialize", // method name
                vec![],      // args
            );
            (call, hasher_method_name)
        }
        _ => {
            let hasher_method_name = "add".to_owned();
            let call = cast(
                arr_index,                        // lhs - `ident[i]`
                UnresolvedTypeData::FieldElement, // cast to - `as Field`
            );
            (call, hasher_method_name)
        }
    };

    let block_statement = make_statement(StatementKind::Semi(method_call(
        variable(hasher_name), // variable
        &hasher_method_name,   // method name
        vec![add_expression],
    )));

    create_loop_over(variable_ident(identifier.clone()), vec![block_statement])
}

fn add_field_to_hasher(identifier: &Ident, hasher_name: &str) -> Statement {
    // `hasher.add({ident})`
    let ident = variable_path(path(identifier.clone()));
    make_statement(StatementKind::Semi(method_call(
        variable(hasher_name), // variable
        "add",                 // method name
        vec![ident],           // args
    )))
}

fn add_cast_to_hasher(identifier: &Ident, hasher_name: &str) -> Statement {
    // `hasher.add({ident} as Field)`
    // `{ident} as Field`
    let cast_operation = cast(
        variable_path(path(identifier.clone())), // lhs
        UnresolvedTypeData::FieldElement,        // rhs
    );

    // `hasher.add({ident} as Field)`
    make_statement(StatementKind::Semi(method_call(
        variable(hasher_name), // variable
        "add",                 // method name
        vec![cast_operation],  // args
    )))
}

/**
 * Takes a vector of functions and checks for the presence of arguments with Public visibility
 * Returns AztecMAcroError::PublicArgsDisallowed if found
 */
pub fn check_for_public_args(functions: &[&NoirFunction]) -> Result<(), AztecMacroError> {
    for func in functions {
        for param in &func.def.parameters {
            if param.visibility == Visibility::Public {
                return Err(AztecMacroError::PublicArgsDisallowed { span: func.span() });
            }
        }
    }
    Ok(())
}
