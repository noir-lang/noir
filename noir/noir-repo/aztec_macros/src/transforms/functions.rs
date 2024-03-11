use convert_case::{Case, Casing};
use noirc_errors::Span;
use noirc_frontend::{
    macros_api::FieldElement, BlockExpression, ConstrainKind, ConstrainStatement, Distinctness,
    Expression, ExpressionKind, ForLoopStatement, ForRange, FunctionReturnType, Ident, Literal,
    NoirFunction, Param, PathKind, Pattern, Signedness, Statement, StatementKind, UnresolvedType,
    UnresolvedTypeData, Visibility,
};

use crate::{
    chained_dep, chained_path,
    utils::{
        ast_utils::{
            assignment, call, cast, expression, ident, ident_path, index_array,
            index_array_variable, make_eq, make_statement, make_type, member_access, method_call,
            mutable_assignment, mutable_reference, path, return_type, variable, variable_ident,
            variable_path,
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
    storage_defined: bool,
    is_initializer: bool,
    insert_init_check: bool,
    is_internal: bool,
) -> Result<(), AztecMacroError> {
    let context_name = format!("{}Context", ty);
    let inputs_name = format!("{}ContextInputs", ty);
    let return_type_name = format!("{}CircuitPublicInputs", ty);

    // Add check that msg sender equals this address and flag function as internal
    if is_internal {
        let is_internal_check = create_internal_check(func.name());
        func.def.body.0.insert(0, is_internal_check);
        func.def.is_internal = true;
    }

    // Add initialization check
    if insert_init_check {
        let init_check = create_init_check();
        func.def.body.0.insert(0, init_check);
    }

    // Add access to the storage struct
    if storage_defined {
        let storage_def = abstract_storage(&ty.to_lowercase(), false);
        func.def.body.0.insert(0, storage_def);
    }

    // Insert the context creation as the first action
    let create_context = create_context(&context_name, &func.def.parameters)?;
    func.def.body.0.splice(0..0, (create_context).iter().cloned());

    // Add the inputs to the params
    let input = create_inputs(&inputs_name);
    func.def.parameters.insert(0, input);

    // Abstract return types such that they get added to the kernel's return_values
    if let Some(return_values) = abstract_return_values(func) {
        // In case we are pushing return values to the context, we remove the statement that originated it
        // This avoids running duplicate code, since blocks like if/else can be value returning statements
        func.def.body.0.pop();
        // Add the new return statement
        func.def.body.0.push(return_values);
    }

    // Before returning mark the contract as initialized
    if is_initializer {
        let mark_initialized = create_mark_as_initialized(ty);
        func.def.body.0.push(mark_initialized);
    }

    // Push the finish method call to the end of the function
    let finish_def = create_context_finish();
    func.def.body.0.push(finish_def);

    let return_type = create_return_type(&return_type_name);
    func.def.return_type = return_type;
    func.def.return_visibility = Visibility::Public;

    // Distinct return types are only required for private functions
    // Public functions should have open auto-inferred
    match ty {
        "Private" => func.def.return_distinctness = Distinctness::Distinct,
        "Public" => func.def.is_open = true,
        _ => (),
    }

    Ok(())
}

/// Transform a function to work with AVM bytecode
pub fn transform_vm_function(
    func: &mut NoirFunction,
    storage_defined: bool,
) -> Result<(), AztecMacroError> {
    // Create access to storage
    if storage_defined {
        let storage = abstract_storage("public_vm", true);
        func.def.body.0.insert(0, storage);
    }

    // Push Avm context creation to the beginning of the function
    let create_context = create_avm_context()?;
    func.def.body.0.insert(0, create_context);

    // We want the function to be seen as a public function
    func.def.is_open = true;

    // NOTE: the line below is a temporary hack to trigger external transpilation tools
    // It will be removed once the transpiler is integrated into the Noir compiler
    func.def.name.0.contents = format!("avm_{}", func.def.name.0.contents);
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
pub fn transform_unconstrained(func: &mut NoirFunction) {
    func.def.body.0.insert(0, abstract_storage("Unconstrained", true));
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
fn create_init_check() -> Statement {
    make_statement(StatementKind::Expression(call(
        variable_path(chained_dep!("aztec", "initializer", "assert_is_initialized")),
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
    let name = if ty == "Public" { "mark_as_initialized_public" } else { "mark_as_initialized" };
    make_statement(StatementKind::Expression(call(
        variable_path(chained_dep!("aztec", "initializer", name)),
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
    let mut injected_expressions: Vec<Statement> = vec![];

    // `let mut hasher = Hasher::new();`
    let let_hasher = mutable_assignment(
        "hasher", // Assigned to
        call(
            variable_path(chained_dep!("aztec", "hasher", "Hasher", "new")), // Path
            vec![],                                                          // args
        ),
    );

    // Completes: `let mut hasher = Hasher::new();`
    injected_expressions.push(let_hasher);

    // Iterate over each of the function parameters, adding to them to the hasher
    for Param { pattern, typ, span, .. } in params {
        match pattern {
            Pattern::Identifier(identifier) => {
                // Match the type to determine the padding to do
                let unresolved_type = &typ.typ;
                let expression = match unresolved_type {
                    // `hasher.add_multiple({ident}.serialize())`
                    UnresolvedTypeData::Named(..) => add_struct_to_hasher(identifier),
                    UnresolvedTypeData::Array(_, arr_type) => {
                        add_array_to_hasher(identifier, arr_type)
                    }
                    // `hasher.add({ident})`
                    UnresolvedTypeData::FieldElement => add_field_to_hasher(identifier),
                    // Add the integer to the hasher, casted to a field
                    // `hasher.add({ident} as Field)`
                    UnresolvedTypeData::Integer(..) | UnresolvedTypeData::Bool => {
                        add_cast_to_hasher(identifier)
                    }
                    UnresolvedTypeData::String(..) => {
                        let (var_bytes, id) = str_to_bytes(identifier);
                        injected_expressions.push(var_bytes);
                        add_array_to_hasher(
                            &id,
                            &UnresolvedType {
                                typ: UnresolvedTypeData::Integer(
                                    Signedness::Unsigned,
                                    noirc_frontend::IntegerBitSize::ThirtyTwo,
                                ),
                                span: None,
                            },
                        )
                    }
                    _ => {
                        return Err(AztecMacroError::UnsupportedFunctionArgumentType {
                            typ: unresolved_type.clone(),
                            span: *span,
                        })
                    }
                };
                injected_expressions.push(expression);
            }
            _ => todo!(), // Maybe unreachable?
        }
    }

    // Create the inputs to the context
    let inputs_expression = variable("inputs");
    // `hasher.hash()`
    let hash_call = method_call(
        variable("hasher"), // variable
        "hash",             // method name
        vec![],             // args
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
    injected_expressions.push(let_context);

    // Return all expressions that will be injected by the hasher
    Ok(injected_expressions)
}

/// Creates an mutable avm context
///
/// ```noir
/// /// Before
/// #[aztec(public-vm)]
/// fn foo() -> Field {
///   let mut context = aztec::context::AVMContext::new();
///   let timestamp = context.timestamp();
///   // ...
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() -> Field {
///     let mut timestamp = context.timestamp();
///     // ...
/// }
fn create_avm_context() -> Result<Statement, AztecMacroError> {
    let let_context = mutable_assignment(
        "context", // Assigned to
        call(
            variable_path(chained_dep!("aztec", "context", "AVMContext", "new")), // Path
            vec![],                                                               // args
        ),
    );

    Ok(let_context)
}

/// Abstract Return Type
///
/// This function intercepts the function's current return type and replaces it with pushes
/// To the kernel
///
/// The replaced code:
/// ```noir
/// /// Before
/// #[aztec(private)]
/// fn foo() -> protocol_types::abis::private_circuit_public_inputs::PrivateCircuitPublicInputs {
///   // ...
///   let my_return_value: Field = 10;
///   context.return_values.push(my_return_value);
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() -> Field {
///     // ...
///    let my_return_value: Field = 10;
///    my_return_value
/// }
/// ```
/// Similarly; Structs will be pushed to the context, after serialize() is called on them.
/// Arrays will be iterated over and each element will be pushed to the context.
/// Any primitive type that can be cast will be casted to a field and pushed to the context.
fn abstract_return_values(func: &NoirFunction) -> Option<Statement> {
    let current_return_type = func.return_type().typ;
    let last_statement = func.def.body.0.last()?;

    // TODO: (length, type) => We can limit the size of the array returned to be limited by kernel size
    // Doesn't need done until we have settled on a kernel size
    // TODO: support tuples here and in inputs -> convert into an issue
    // Check if the return type is an expression, if it is, we can handle it
    match last_statement {
        Statement { kind: StatementKind::Expression(expression), .. } => {
            match current_return_type {
                // Call serialize on structs, push the whole array, calling push_array
                UnresolvedTypeData::Named(..) => Some(make_struct_return_type(expression.clone())),
                UnresolvedTypeData::Array(..) => Some(make_array_return_type(expression.clone())),
                // Cast these types to a field before pushing
                UnresolvedTypeData::Bool | UnresolvedTypeData::Integer(..) => {
                    Some(make_castable_return_type(expression.clone()))
                }
                UnresolvedTypeData::FieldElement => Some(make_return_push(expression.clone())),
                _ => None,
            }
        }
        _ => None,
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
fn abstract_storage(typ: &str, unconstrained: bool) -> Statement {
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
            variable_path(chained_path!("Storage", "init")), // Path
            vec![init_context_call],                         // args
        ),
    )
}

/// Context Return Values
///
/// Creates an instance to the context return values
/// ```noir
/// `context.return_values`
/// ```
fn context_return_values() -> Expression {
    member_access("context", "return_values")
}

/// Make return Push
///
/// Translates to:
/// `context.return_values.push({push_value})`
fn make_return_push(push_value: Expression) -> Statement {
    make_statement(StatementKind::Semi(method_call(
        context_return_values(),
        "push",
        vec![push_value],
    )))
}

/// Make Return push array
///
/// Translates to:
/// `context.return_values.extend_from_array({push_value})`
fn make_return_extend_from_array(push_value: Expression) -> Statement {
    make_statement(StatementKind::Semi(method_call(
        context_return_values(),
        "extend_from_array",
        vec![push_value],
    )))
}

/// Make struct return type
///
/// Translates to:
/// ```noir
/// `context.return_values.extend_from_array({push_value}.serialize())`
fn make_struct_return_type(expression: Expression) -> Statement {
    let serialized_call = method_call(
        expression,  // variable
        "serialize", // method name
        vec![],      // args
    );
    make_return_extend_from_array(serialized_call)
}

/// Make array return type
///
/// Translates to:
/// ```noir
/// for i in 0..{ident}.len() {
///    context.return_values.push({ident}[i] as Field)
/// }
/// ```
fn make_array_return_type(expression: Expression) -> Statement {
    let inner_cast_expression =
        cast(index_array_variable(expression.clone(), "i"), UnresolvedTypeData::FieldElement);
    let assignment = make_statement(StatementKind::Semi(method_call(
        context_return_values(), // variable
        "push",                  // method name
        vec![inner_cast_expression],
    )));

    create_loop_over(expression, vec![assignment])
}

/// Castable return type
///
/// Translates to:
/// ```noir
/// context.return_values.push({ident} as Field)
/// ```
fn make_castable_return_type(expression: Expression) -> Statement {
    // Cast these types to a field before pushing
    let cast_expression = cast(expression, UnresolvedTypeData::FieldElement);
    make_return_push(cast_expression)
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

fn add_struct_to_hasher(identifier: &Ident) -> Statement {
    // If this is a struct, we call serialize and add the array to the hasher
    let serialized_call = method_call(
        variable_path(path(identifier.clone())), // variable
        "serialize",                             // method name
        vec![],                                  // args
    );

    make_statement(StatementKind::Semi(method_call(
        variable("hasher"),    // variable
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
    let for_loop_block = expression(ExpressionKind::Block(BlockExpression(loop_body)));

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

fn add_array_to_hasher(identifier: &Ident, arr_type: &UnresolvedType) -> Statement {
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
        variable("hasher"),  // variable
        &hasher_method_name, // method name
        vec![add_expression],
    )));

    create_loop_over(variable_ident(identifier.clone()), vec![block_statement])
}

fn add_field_to_hasher(identifier: &Ident) -> Statement {
    // `hasher.add({ident})`
    let ident = variable_path(path(identifier.clone()));
    make_statement(StatementKind::Semi(method_call(
        variable("hasher"), // variable
        "add",              // method name
        vec![ident],        // args
    )))
}

fn add_cast_to_hasher(identifier: &Ident) -> Statement {
    // `hasher.add({ident} as Field)`
    // `{ident} as Field`
    let cast_operation = cast(
        variable_path(path(identifier.clone())), // lhs
        UnresolvedTypeData::FieldElement,        // rhs
    );

    // `hasher.add({ident} as Field)`
    make_statement(StatementKind::Semi(method_call(
        variable("hasher"),   // variable
        "add",                // method name
        vec![cast_operation], // args
    )))
}
