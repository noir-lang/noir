use acvm::FieldElement;
use noirc_errors::{CustomDiagnostic, Span};

use crate::graph::CrateId;
use crate::{
    hir::Context, token::Attribute, BlockExpression, CallExpression, CastExpression, Distinctness,
    Expression, ExpressionKind, ForExpression, FunctionReturnType, Ident, ImportStatement,
    IndexExpression, LetStatement, Literal, MemberAccessExpression, MethodCallExpression,
    NoirFunction, ParsedModule, Path, PathKind, Pattern, Statement, UnresolvedType,
    UnresolvedTypeData, Visibility,
};
use noirc_errors::FileDiagnostic;

//
//             Helper macros for creating noir ast nodes
//
fn ident(name: &str) -> Ident {
    Ident::new(name.to_string(), Span::default())
}

fn ident_path(name: &str) -> Path {
    Path::from_ident(ident(name))
}

fn path(ident: Ident) -> Path {
    Path::from_ident(ident)
}

fn expression(kind: ExpressionKind) -> Expression {
    Expression::new(kind, Span::default())
}

fn variable(name: &str) -> Expression {
    expression(ExpressionKind::Variable(ident_path(name)))
}

fn variable_ident(identifier: Ident) -> Expression {
    expression(ExpressionKind::Variable(path(identifier)))
}

fn variable_path(path: Path) -> Expression {
    expression(ExpressionKind::Variable(path))
}

fn method_call(object: Expression, method_name: &str, arguments: Vec<Expression>) -> Expression {
    expression(ExpressionKind::MethodCall(Box::new(MethodCallExpression {
        object,
        method_name: ident(method_name),
        arguments,
    })))
}

fn call(func: Expression, arguments: Vec<Expression>) -> Expression {
    expression(ExpressionKind::Call(Box::new(CallExpression { func: Box::new(func), arguments })))
}

fn mutable(pattern: &str) -> Pattern {
    Pattern::Mutable(Box::new(Pattern::Identifier(ident(pattern))), Span::default())
}

fn mutable_assignment(name: &str, assigned_to: Expression) -> Statement {
    Statement::Let(LetStatement {
        pattern: mutable(name),
        r#type: make_type(UnresolvedTypeData::Unspecified),
        expression: assigned_to,
    })
}

fn member_access(lhs: &str, rhs: &str) -> Expression {
    expression(ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
        lhs: variable(lhs),
        rhs: ident(rhs),
    })))
}

macro_rules! chained_path {
    ( $base:expr $(, $tail:expr)* ) => {
        {
            let mut base_path = ident_path($base);
            $(
                base_path.segments.push(ident($tail));
            )*
            base_path
        }
    }
}

macro_rules! chained_dep {
    ( $base:expr $(, $tail:expr)* ) => {
        {
            let mut base_path = ident_path($base);
            base_path.kind = PathKind::Dep;
            $(
                base_path.segments.push(ident($tail));
            )*
            base_path
        }
    }
}

fn cast(lhs: Expression, ty: UnresolvedTypeData) -> Expression {
    expression(ExpressionKind::Cast(Box::new(CastExpression { lhs, r#type: make_type(ty) })))
}

fn make_type(typ: UnresolvedTypeData) -> UnresolvedType {
    UnresolvedType { typ, span: None }
}

fn index_array(array: Ident, index: &str) -> Expression {
    expression(ExpressionKind::Index(Box::new(IndexExpression {
        collection: variable_path(path(array)),
        index: variable(index),
    })))
}

fn index_array_variable(array: Expression, index: &str) -> Expression {
    expression(ExpressionKind::Index(Box::new(IndexExpression {
        collection: array,
        index: variable(index),
    })))
}

fn import(path: Path) -> ImportStatement {
    ImportStatement { path, alias: None }
}

//
//                    Create AST Nodes for Aztec
//

/// Traverses every function in the ast, calling `transform_function` which
/// determines if further processing is required
pub(crate) fn transform(
    mut ast: ParsedModule,
    crate_id: &CrateId,
    context: &Context,
    errors: &mut Vec<FileDiagnostic>,
) -> ParsedModule {
    // Usage -> mut ast -> aztec_library::transform(&mut ast)

    // Covers all functions in the ast
    for submodule in ast.submodules.iter_mut().filter(|submodule| submodule.is_contract) {
        if transform_module(&mut submodule.contents.functions) {
            check_for_aztec_dependency(crate_id, context, errors);
            include_relevant_imports(&mut submodule.contents);
        }
    }
    ast
}

/// Includes an import to the aztec library if it has not been included yet
fn include_relevant_imports(ast: &mut ParsedModule) {
    // Create the aztec import path using the assumed chained_dep! macro
    let aztec_import_path = import(chained_dep!("aztec"));

    // Check if the aztec import already exists
    let is_aztec_imported =
        ast.imports.iter().any(|existing_import| existing_import.path == aztec_import_path.path);

    // If aztec is not imported, add the import at the beginning
    if !is_aztec_imported {
        ast.imports.insert(0, aztec_import_path);
    }
}

/// Creates an error alerting the user that they have not downloaded the Aztec-noir library
fn check_for_aztec_dependency(
    crate_id: &CrateId,
    context: &Context,
    errors: &mut Vec<FileDiagnostic>,
) {
    let crate_graph = &context.crate_graph[crate_id];
    let has_aztec_dependency = crate_graph.dependencies.iter().any(|dep| dep.as_name() == "aztec");

    if !has_aztec_dependency {
        errors.push(FileDiagnostic::new(
            crate_graph.root_file_id,
            CustomDiagnostic::from_message(
                "Aztec dependency not found. Please add aztec as a dependency in your Cargo.toml",
            ),
        ));
    }
}

/// Determines if the function is annotated with `aztec(private)` or `aztec(public)`
/// If it is, it calls the `transform` function which will perform the required transformations.
/// Returns true if an annotated function is found, false otherwise
fn transform_module(functions: &mut [NoirFunction]) -> bool {
    let mut has_annotated_functions = false;
    for func in functions.iter_mut() {
        if let Some(Attribute::Custom(custom_attribute)) = func.def.attribute.as_ref() {
            match custom_attribute.as_str() {
                "aztec(private)" => {
                    transform_function("Private", func);
                    has_annotated_functions = true;
                }
                "aztec(public)" => {
                    transform_function("Public", func);
                    has_annotated_functions = true;
                }
                _ => continue,
            }
        }
    }
    has_annotated_functions
}

/// If it does, it will insert the following things:
/// - A new Input that is provided for a kernel app circuit, named: {Public/Private}ContextInputs
/// - Hashes all of the function input variables
///     - This instantiates a helper function  
fn transform_function(ty: &str, func: &mut NoirFunction) {
    let context_name = format!("{}Context", ty);
    let inputs_name = format!("{}ContextInputs", ty);
    let return_type_name = format!("{}CircuitPublicInputs", ty);

    // Insert the context creation as the first action
    let create_context = create_context(&context_name, &func.def.parameters);
    func.def.body.0.splice(0..0, (create_context).iter().cloned());

    // Add the inputs to the params
    let input = create_inputs(&inputs_name);
    func.def.parameters.insert(0, input);

    // Abstract return types such that they get added to the kernel's return_values
    if let Some(return_values) = abstract_return_values(func) {
        func.def.body.0.push(return_values);
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
pub(crate) fn create_inputs(ty: &str) -> (Pattern, UnresolvedType, Visibility) {
    let context_ident = ident("inputs");
    let context_pattern = Pattern::Identifier(context_ident);
    let type_path = chained_path!("aztec", "abi", ty);
    let context_type = make_type(UnresolvedTypeData::Named(type_path, vec![]));
    let visibility = Visibility::Private;

    (context_pattern, context_type, visibility)
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
fn create_context(ty: &str, params: &[(Pattern, UnresolvedType, Visibility)]) -> Vec<Statement> {
    let mut injected_expressions: Vec<Statement> = vec![];

    // `let mut hasher = Hasher::new();`
    let let_hasher = mutable_assignment(
        "hasher", // Assigned to
        call(
            variable_path(chained_path!("aztec", "abi", "Hasher", "new")), // Path
            vec![],                                                        // args
        ),
    );

    // Completes: `let mut hasher = Hasher::new();`
    injected_expressions.push(let_hasher);

    // Iterate over each of the function parameters, adding to them to the hasher
    params.iter().for_each(|(pattern, typ, _vis)| {
        match pattern {
            Pattern::Identifier(identifier) => {
                // Match the type to determine the padding to do
                let unresolved_type = &typ.typ;
                let expression = match unresolved_type {
                    // `hasher.add_multiple({ident}.serialize())`
                    UnresolvedTypeData::Named(..) => add_struct_to_hasher(identifier),
                    // TODO: if this is an array of structs, we should call serialise on each of them (no methods currently do this yet)
                    UnresolvedTypeData::Array(..) => add_array_to_hasher(identifier),
                    // `hasher.add({ident})`
                    UnresolvedTypeData::FieldElement => add_field_to_hasher(identifier),
                    // Add the integer to the hasher, casted to a field
                    // `hasher.add({ident} as Field)`
                    UnresolvedTypeData::Integer(..) => add_int_to_hasher(identifier),
                    _ => unreachable!("[Aztec Noir] Provided parameter type is not supported"),
                };
                injected_expressions.push(expression);
            }
            _ => todo!(), // Maybe unreachable?
        }
    });

    // Create the inputs to the context
    let inputs_expression = variable("inputs");
    // `hasher.hash()`
    let hash_call = method_call(
        variable("hasher"), // variable
        "hash",             // method name
        vec![],             // args
    );

    // let mut context = {ty}::new(inputs, hash);
    let let_context = mutable_assignment(
        "context", // Assigned to
        call(
            variable_path(chained_path!("aztec", "context", ty, "new")), // Path
            vec![inputs_expression, hash_call],                          // args
        ),
    );
    injected_expressions.push(let_context);

    // Return all expressions that will be injected by the hasher
    injected_expressions
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
/// fn foo() -> abi::PrivateCircuitPublicInputs {
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
    let len = func.def.body.len();
    let last_statement = &func.def.body.0[len - 1];

    // TODO: (length, type) => We can limit the size of the array returned to be limited by kernel size
    // Doesnt need done until we have settled on a kernel size
    // TODO: support tuples here and in inputs -> convert into an issue

    // Check if the return type is an expression, if it is, we can handle it
    match last_statement {
        Statement::Expression(expression) => match current_return_type {
            // Call serialize on structs, push the whole array, calling push_array
            UnresolvedTypeData::Named(..) => Some(make_struct_return_type(expression.clone())),
            UnresolvedTypeData::Array(..) => Some(make_array_return_type(expression.clone())),
            // Cast these types to a field before pushing
            UnresolvedTypeData::Bool | UnresolvedTypeData::Integer(..) => {
                Some(make_castable_return_type(expression.clone()))
            }
            UnresolvedTypeData::FieldElement => Some(make_return_push(expression.clone())),
            _ => None,
        },
        _ => None,
    }
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
    Statement::Semi(method_call(context_return_values(), "push", vec![push_value]))
}

/// Make Return push array
///
/// Translates to:
/// `context.return_values.push_array({push_value})`
fn make_return_push_array(push_value: Expression) -> Statement {
    Statement::Semi(method_call(context_return_values(), "push_array", vec![push_value]))
}

/// Make struct return type
///
/// Translates to:
/// ```noir
/// `context.return_values.push_array({push_value}.serialize())`
fn make_struct_return_type(expression: Expression) -> Statement {
    let serialised_call = method_call(
        expression.clone(), // variable
        "serialize",        // method name
        vec![],             // args
    );
    make_return_push_array(serialised_call)
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
    let assignment = Statement::Semi(method_call(
        context_return_values(), // variable
        "push",                  // method name
        vec![inner_cast_expression],
    ));

    create_loop_over(expression.clone(), vec![assignment])
}

/// Castable return type
///
/// Translates to:
/// ```noir
/// context.return_values.push({ident} as Field)
/// ```
fn make_castable_return_type(expression: Expression) -> Statement {
    // Cast these types to a field before pushing
    let cast_expression = cast(expression.clone(), UnresolvedTypeData::FieldElement);
    make_return_push(cast_expression)
}

/// Create Return Type
///
/// Public functions return abi::PublicCircuitPublicInputs while
/// private functions return abi::PrivateCircuitPublicInputs
///
/// This call constructs an ast token referencing the above types
/// The name is set in the function above `transform`, hence the
/// whole token name is passed in
///
/// The replaced code:
/// ```noir
///
/// /// Before
/// fn foo() -> abi::PrivateCircuitPublicInputs {
///    // ...
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() {
///  // ...
/// }
pub(crate) fn create_return_type(ty: &str) -> FunctionReturnType {
    let return_path = chained_path!("aztec", "abi", ty);

    let ty = make_type(UnresolvedTypeData::Named(return_path, vec![]));
    FunctionReturnType::Ty(ty)
}

/// Create Context Finish
///
/// Each aztec function calls `context.finish()` at the end of a function
/// to return values required by the kernel.
///
/// The replaced code:
/// ```noir
/// /// Before
/// fn foo() -> abi::PrivateCircuitPublicInputs {
///   // ...
///  context.finish()
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() {
///  // ...
/// }
pub(crate) fn create_context_finish() -> Statement {
    let method_call = method_call(
        variable("context"), // variable
        "finish",            // method name
        vec![],              // args
    );
    Statement::Expression(method_call)
}

//
//                 Methods to create hasher inputs
//

fn add_struct_to_hasher(identifier: &Ident) -> Statement {
    // If this is a struct, we call serialize and add the array to the hasher
    let serialised_call = method_call(
        variable_path(path(identifier.clone())), // variable
        "serialize",                             // method name
        vec![],                                  // args
    );

    Statement::Semi(method_call(
        variable("hasher"),    // variable
        "add_multiple",        // method name
        vec![serialised_call], // args
    ))
}

fn create_loop_over(var: Expression, loop_body: Vec<Statement>) -> Statement {
    // If this is an array of primitive types (integers / fields) we can add them each to the hasher
    // casted to a field

    // `array.len()`
    let end_range_expression = method_call(
        var.clone(), // variable
        "len",       // method name
        vec![],      // args
    );

    // What will be looped over
    // - `hasher.add({ident}[i] as Field)`
    let for_loop_block = expression(ExpressionKind::Block(BlockExpression(loop_body)));

    // `for i in 0..{ident}.len()`
    Statement::Expression(expression(ExpressionKind::For(Box::new(ForExpression {
        identifier: ident("i"),
        start_range: expression(ExpressionKind::Literal(Literal::Integer(FieldElement::from(
            i128::from(0),
        )))),
        end_range: end_range_expression,
        block: for_loop_block,
    }))))
}

fn add_array_to_hasher(identifier: &Ident) -> Statement {
    // If this is an array of primitive types (integers / fields) we can add them each to the hasher
    // casted to a field

    // Wrap in the semi thing - does that mean ended with semi colon?
    // `hasher.add({ident}[i] as Field)`
    let cast_expression = cast(
        index_array(identifier.clone(), "i"), // lhs - `ident[i]`
        UnresolvedTypeData::FieldElement,     // cast to - `as Field`
    );
    let block_statement = Statement::Semi(method_call(
        variable("hasher"), // variable
        "add",              // method name
        vec![cast_expression],
    ));

    create_loop_over(variable_ident(identifier.clone()), vec![block_statement])
}

fn add_field_to_hasher(identifier: &Ident) -> Statement {
    // `hasher.add({ident})`
    let iden = variable_path(path(identifier.clone()));
    Statement::Semi(method_call(
        variable("hasher"), // variable
        "add",              // method name
        vec![iden],         // args
    ))
}

fn add_int_to_hasher(identifier: &Ident) -> Statement {
    // `hasher.add({ident} as Field)`
    // `{ident} as Field`
    let cast_operation = cast(
        variable_path(path(identifier.clone())), // lhs
        UnresolvedTypeData::FieldElement,        // rhs
    );

    // `hasher.add({ident} as Field)`
    Statement::Semi(method_call(
        variable("hasher"),   // variable
        "add",                // method name
        vec![cast_operation], // args
    ))
}
