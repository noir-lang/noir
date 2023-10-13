use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_errors::Span;

use crate::graph::CrateId;
use crate::hir::def_collector::errors::DefCollectorErrorKind;
use crate::hir_def::expr::{HirExpression, HirLiteral};
use crate::hir_def::stmt::HirStatement;
use crate::node_interner::{NodeInterner, StructId};
use crate::parser::SortedModule;
use crate::token::SecondaryAttribute;
use crate::{
    hir::Context, BlockExpression, CallExpression, CastExpression, Distinctness, Expression,
    ExpressionKind, FunctionReturnType, Ident, IndexExpression, LetStatement, Literal,
    MemberAccessExpression, MethodCallExpression, NoirFunction, Path, PathKind, Pattern, Statement,
    UnresolvedType, UnresolvedTypeData, Visibility,
};
use crate::{
    ForLoopStatement, FunctionDefinition, ImportStatement, NoirStruct, PrefixExpression,
    Signedness, StatementKind, StructType, Type, TypeImpl, UnaryOp,
};
use fm::FileId;

use super::def_map::ModuleDefId;

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

fn pattern(name: &str) -> Pattern {
    Pattern::Identifier(ident(name))
}

fn mutable(name: &str) -> Pattern {
    Pattern::Mutable(Box::new(pattern(name)), Span::default())
}

fn mutable_assignment(name: &str, assigned_to: Expression) -> Statement {
    make_statement(StatementKind::Let(LetStatement {
        pattern: mutable(name),
        r#type: make_type(UnresolvedTypeData::Unspecified),
        expression: assigned_to,
    }))
}

fn mutable_reference(variable_name: &str) -> Expression {
    expression(ExpressionKind::Prefix(Box::new(PrefixExpression {
        operator: UnaryOp::MutableReference,
        rhs: variable(variable_name),
    })))
}

fn assignment(name: &str, assigned_to: Expression) -> Statement {
    make_statement(StatementKind::Let(LetStatement {
        pattern: pattern(name),
        r#type: make_type(UnresolvedTypeData::Unspecified),
        expression: assigned_to,
    }))
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
    mut ast: SortedModule,
    crate_id: &CrateId,
    context: &Context,
) -> Result<SortedModule, (DefCollectorErrorKind, FileId)> {
    // Usage -> mut ast -> aztec_library::transform(&mut ast)

    // Covers all functions in the ast
    for submodule in ast.submodules.iter_mut().filter(|submodule| submodule.is_contract) {
        let storage_defined = check_for_storage_definition(&submodule.contents);

        if transform_module(&mut submodule.contents, storage_defined) {
            match check_for_aztec_dependency(crate_id, context) {
                Ok(()) => include_relevant_imports(&mut submodule.contents),
                Err(file_id) => {
                    return Err((DefCollectorErrorKind::AztecNotFound {}, file_id));
                }
            }
        }
    }
    Ok(ast)
}

//
//                    Transform Hir Nodes for Aztec
//

/// Completes the Hir with data gathered from type resolution
pub(crate) fn transform_hir(crate_id: &CrateId, context: &mut Context) {
    transform_events(crate_id, context);
}

/// Includes an import to the aztec library if it has not been included yet
fn include_relevant_imports(ast: &mut SortedModule) {
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
fn check_for_aztec_dependency(crate_id: &CrateId, context: &Context) -> Result<(), FileId> {
    let crate_graph = &context.crate_graph[crate_id];
    let has_aztec_dependency = crate_graph.dependencies.iter().any(|dep| dep.as_name() == "aztec");
    if has_aztec_dependency {
        Ok(())
    } else {
        Err(crate_graph.root_file_id)
    }
}

// Check to see if the user has defined a storage struct
fn check_for_storage_definition(module: &SortedModule) -> bool {
    module.types.iter().any(|function| function.name.0.contents == "Storage")
}

/// Checks if an attribute is a custom attribute with a specific name
fn is_custom_attribute(attr: &SecondaryAttribute, attribute_name: &str) -> bool {
    if let SecondaryAttribute::Custom(custom_attr) = attr {
        custom_attr.as_str() == attribute_name
    } else {
        false
    }
}

/// Determines if ast nodes are annotated with aztec attributes.
/// For annotated functions it calls the `transform` function which will perform the required transformations.
/// Returns true if an annotated node is found, false otherwise
fn transform_module(module: &mut SortedModule, storage_defined: bool) -> bool {
    let mut has_transformed_module = false;

    for structure in module.types.iter() {
        if structure.attributes.iter().any(|attr| matches!(attr, SecondaryAttribute::Event)) {
            module.impls.push(generate_selector_impl(structure));
            has_transformed_module = true;
        }
    }

    for func in module.functions.iter_mut() {
        for secondary_attribute in func.def.attributes.secondary.clone() {
            if is_custom_attribute(&secondary_attribute, "aztec(private)") {
                transform_function("Private", func, storage_defined);
                has_transformed_module = true;
            } else if is_custom_attribute(&secondary_attribute, "aztec(public)") {
                transform_function("Public", func, storage_defined);
                has_transformed_module = true;
            }
        }
        // Add the storage struct to the beginning of the function if it is unconstrained in an aztec contract
        if storage_defined && func.def.is_unconstrained {
            transform_unconstrained(func);
            has_transformed_module = true;
        }
    }
    has_transformed_module
}

/// If it does, it will insert the following things:
/// - A new Input that is provided for a kernel app circuit, named: {Public/Private}ContextInputs
/// - Hashes all of the function input variables
///     - This instantiates a helper function  
fn transform_function(ty: &str, func: &mut NoirFunction, storage_defined: bool) {
    let context_name = format!("{}Context", ty);
    let inputs_name = format!("{}ContextInputs", ty);
    let return_type_name = format!("{}CircuitPublicInputs", ty);

    // Add access to the storage struct
    if storage_defined {
        let storage_def = abstract_storage(&ty.to_lowercase(), false);
        func.def.body.0.insert(0, storage_def);
    }

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

/// Transform Unconstrained
///
/// Inserts the following code at the beginning of an unconstrained function
/// ```noir
/// let storage = Storage::init(Context::none());
/// ```
///
/// This will allow developers to access their contract' storage struct in unconstrained functions
fn transform_unconstrained(func: &mut NoirFunction) {
    func.def.body.0.insert(0, abstract_storage("Unconstrained", true));
}

fn collect_crate_structs(crate_id: &CrateId, context: &Context) -> Vec<StructId> {
    context
        .def_map(crate_id)
        .expect("ICE: Missing crate in def_map")
        .modules()
        .iter()
        .flat_map(|(_, module)| {
            module.type_definitions().filter_map(|typ| {
                if let ModuleDefId::TypeId(struct_id) = typ {
                    Some(struct_id)
                } else {
                    None
                }
            })
        })
        .collect()
}

/// Substitutes the signature literal that was introduced in the selector method previously with the actual signature.
fn transform_event(struct_id: StructId, interner: &mut NodeInterner) {
    let struct_type = interner.get_struct(struct_id);
    let selector_id = interner
        .lookup_method(&Type::Struct(struct_type, vec![]), struct_id, "selector", false)
        .expect("Selector method not found");
    let selector_function = interner.function(&selector_id);

    let compute_selector_statement = interner.statement(
        selector_function
            .block(interner)
            .statements()
            .first()
            .expect("Compute selector statement not found"),
    );

    let compute_selector_expression = match compute_selector_statement {
        HirStatement::Expression(expression_id) => match interner.expression(&expression_id) {
            HirExpression::Call(hir_call_expression) => Some(hir_call_expression),
            _ => None,
        },
        _ => None,
    }
    .expect("Compute selector statement is not a call expression");

    let first_arg_id = compute_selector_expression
        .arguments
        .first()
        .expect("Missing argument for compute selector");

    match interner.expression(first_arg_id) {
        HirExpression::Literal(HirLiteral::Str(signature))
            if signature == SIGNATURE_PLACEHOLDER =>
        {
            let selector_literal_id = first_arg_id;

            let structure = interner.get_struct(struct_id);
            let signature = event_signature(&structure.borrow());
            interner.update_expression(*selector_literal_id, |expr| {
                *expr = HirExpression::Literal(HirLiteral::Str(signature.clone()));
            });

            // Also update the type! It might have a different length now than the placeholder.
            interner.push_expr_type(
                selector_literal_id,
                Type::String(Box::new(Type::Constant(signature.len() as u64))),
            );
        }
        _ => unreachable!("Signature placeholder literal does not match"),
    }
}

fn transform_events(crate_id: &CrateId, context: &mut Context) {
    for struct_id in collect_crate_structs(crate_id, context) {
        let attributes = context.def_interner.struct_attributes(&struct_id);
        if attributes.iter().any(|attr| matches!(attr, SecondaryAttribute::Event)) {
            transform_event(struct_id, &mut context.def_interner);
        }
    }
}

const SIGNATURE_PLACEHOLDER: &str = "SIGNATURE_PLACEHOLDER";

/// Generates the impl for an event selector
///
/// Inserts the following code:
/// ```noir
/// impl SomeStruct {
///    fn selector() -> Field {
///       aztec::oracle::compute_selector::compute_selector("SIGNATURE_PLACEHOLDER")
///    }
/// }
/// ```
///
/// This allows developers to emit events without having to write the signature of the event every time they emit it.
/// The signature cannot be known at this point since types are not resolved yet, so we use a signature placeholder.
/// It'll get resolved after by transforming the HIR.
fn generate_selector_impl(structure: &NoirStruct) -> TypeImpl {
    let struct_type = make_type(UnresolvedTypeData::Named(path(structure.name.clone()), vec![]));

    let selector_fun_body = BlockExpression(vec![make_statement(StatementKind::Expression(call(
        variable_path(chained_path!("aztec", "selector", "compute_selector")),
        vec![expression(ExpressionKind::Literal(Literal::Str(SIGNATURE_PLACEHOLDER.to_string())))],
    )))]);

    let mut selector_fn_def = FunctionDefinition::normal(
        &ident("selector"),
        &vec![],
        &[],
        &selector_fun_body,
        &[],
        &FunctionReturnType::Ty(make_type(UnresolvedTypeData::FieldElement)),
    );

    selector_fn_def.is_public = true;

    // Seems to be necessary on contract modules
    selector_fn_def.return_visibility = Visibility::Public;

    TypeImpl {
        object_type: struct_type,
        type_span: structure.span,
        generics: vec![],
        methods: vec![NoirFunction::normal(selector_fn_def)],
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
                    // TODO: if this is an array of structs, we should call serialize on each of them (no methods currently do this yet)
                    UnresolvedTypeData::Array(..) => add_array_to_hasher(identifier),
                    // `hasher.add({ident})`
                    UnresolvedTypeData::FieldElement => add_field_to_hasher(identifier),
                    // Add the integer to the hasher, casted to a field
                    // `hasher.add({ident} as Field)`
                    UnresolvedTypeData::Integer(..) | UnresolvedTypeData::Bool => {
                        add_cast_to_hasher(identifier)
                    }
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
            variable_path(chained_path!("aztec", "context", "Context", "none")), // Path
            vec![],                                                              // args
        )
    } else {
        call(
            variable_path(chained_path!("aztec", "context", "Context", typ)), // Path
            vec![mutable_reference("context")],                               // args
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

fn make_statement(kind: StatementKind) -> Statement {
    Statement { span: Span::default(), kind }
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
/// `context.return_values.push_array({push_value})`
fn make_return_push_array(push_value: Expression) -> Statement {
    make_statement(StatementKind::Semi(method_call(
        context_return_values(),
        "push_array",
        vec![push_value],
    )))
}

/// Make struct return type
///
/// Translates to:
/// ```noir
/// `context.return_values.push_array({push_value}.serialize())`
fn make_struct_return_type(expression: Expression) -> Statement {
    let serialized_call = method_call(
        expression,  // variable
        "serialize", // method name
        vec![],      // args
    );
    make_return_push_array(serialized_call)
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

fn create_loop_over(var: Expression, loop_body: Vec<Statement>) -> Statement {
    // If this is an array of primitive types (integers / fields) we can add them each to the hasher
    // casted to a field

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
        identifier: ident("i"),
        start_range: expression(ExpressionKind::Literal(Literal::Integer(FieldElement::from(
            i128::from(0),
        )))),
        end_range: end_range_expression,
        block: for_loop_block,
    }))
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
    let block_statement = make_statement(StatementKind::Semi(method_call(
        variable("hasher"), // variable
        "add",              // method name
        vec![cast_expression],
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

/// Computes the aztec signature for a resolved type.
fn signature_of_type(typ: &Type) -> String {
    match typ {
        Type::Integer(Signedness::Signed, bit_size) => format!("i{}", bit_size),
        Type::Integer(Signedness::Unsigned, bit_size) => format!("u{}", bit_size),
        Type::FieldElement => "Field".to_owned(),
        Type::Bool => "bool".to_owned(),
        Type::Array(len, typ) => {
            if let Type::Constant(len) = **len {
                format!("[{};{len}]", signature_of_type(typ))
            } else {
                unimplemented!("Cannot generate signature for array with length type {:?}", typ)
            }
        }
        Type::Struct(def, args) => {
            let fields = def.borrow().get_fields(args);
            let fields = vecmap(fields, |(_, typ)| signature_of_type(&typ));
            format!("({})", fields.join(","))
        }
        Type::Tuple(types) => {
            let fields = vecmap(types, signature_of_type);
            format!("({})", fields.join(","))
        }
        _ => unimplemented!("Cannot generate signature for type {:?}", typ),
    }
}

/// Computes the signature for a resolved event type.
/// It has the form 'EventName(Field,(Field),[u8;2])'
fn event_signature(event: &StructType) -> String {
    let fields = vecmap(event.get_fields(&[]), |(_, typ)| signature_of_type(&typ));
    format!("{}({})", event.name.0.contents, fields.join(","))
}
