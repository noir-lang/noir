use acvm::FieldElement;
use noirc_errors::Span;

use crate::{
    token::Attribute, BlockExpression, CallExpression, CastExpression, Distinctness, Expression,
    ExpressionKind, ForExpression, FunctionReturnType, Ident, IndexExpression, LetStatement,
    Literal, MethodCallExpression, NoirFunction, ParsedModule, Path, Pattern, Statement,
    UnresolvedType, Visibility,
};

/////////////////////////////////////////////////////////////////////////
///             Helper macros for creating noir ast nodes             ///
/////////////////////////////////////////////////////////////////////////
macro_rules! ident {
    ($name:expr) => {
        Ident::new($name.to_string(), Span::default())
    };
}

macro_rules! ident_path {
    ($name:expr) => {
        Path::from_ident(ident!($name))
    };
}

macro_rules! expression {
    ($kind:expr) => {
        Expression::new($kind, Span::default())
    };
}

macro_rules! variable {
    ($name:expr) => {
        expression!(ExpressionKind::Variable(ident_path!($name)))
    };
}

macro_rules! variable_path {
    ($path:expr) => {
        expression!(ExpressionKind::Variable($path))
    };
}

macro_rules! method_call {
    ($object:expr, $method_name:expr, $arguments:expr) => {
        expression!(ExpressionKind::MethodCall(Box::new(MethodCallExpression {
            object: $object,
            method_name: ident!($method_name),
            arguments: $arguments,
        })))
    };
}

macro_rules! call {
    ($func:expr, $arguments:expr) => {
        expression!(ExpressionKind::Call(Box::new(CallExpression {
            func: Box::new($func),
            arguments: $arguments,
        })))
    };
}

macro_rules! mutable {
    ( $name:expr ) => {
        Pattern::Mutable(Box::new(Pattern::Identifier(ident!($name))), Span::default())
    };
}

macro_rules! mutable_assignment {
    ( $name:expr, $assigned_to:expr ) => {
        Statement::Let(LetStatement {
            pattern: mutable!($name),
            r#type: UnresolvedType::Unspecified,
            expression: $assigned_to,
        })
    };
}

macro_rules! chained_path {
    ( $base:expr $(, $tail:expr)* ) => {
        {
            let mut base_path = ident_path!($base);
            $(
                base_path.segments.push(ident!($tail));
            )*
            base_path
        }
    }
}

macro_rules! cast {
    ( $lhs:expr, $rhs:expr ) => {
        expression!(ExpressionKind::Cast(Box::new(CastExpression { lhs: $lhs, r#type: $rhs })))
    };
}

macro_rules! index_array {
    ( $array:expr, $index:expr ) => {
        expression!(ExpressionKind::Index(Box::new(IndexExpression {
            collection: variable_path!(ident_path!($array)),
            index: variable!($index),
        })))
    };
}

/////////////////////////////////////////////////////////////////////////
///                    Create AST Nodes for Aztec                     ///
/////////////////////////////////////////////////////////////////////////

/// Traverses every function in the ast, calling `transform_function` which
/// determines if further processing is required
pub(crate) fn aztec_contracts_macros(mut ast: ParsedModule) -> ParsedModule {
    // Usage -> mut ast -> AztecLib.transform(&mut ast)

    // Covers all functions in the ast
    for func in ast.functions.iter_mut() {
        transform_function(func);
    }
    for submodule in ast.submodules.iter_mut() {
        for func in submodule.contents.functions.iter_mut() {
            transform_function(func);
        }
    }
    ast
}

/// Determines if the function is annotated with `aztec(private)` or `aztec(public)`
/// If it is, it calls the `transform` function which will perform the required transformations.
fn transform_function(func: &mut NoirFunction) {
    if let Some(Attribute::Custom(custom_attribute)) = func.def.attribute.as_ref() {
        match custom_attribute.as_str() {
            "aztec(private)" => transform("Private", func),
            "aztec(public)" => transform("Public", func),
            _ => return,
        }
    }
}

/// If it does, it will insert the following things:
/// - A new Input that is provided for a kernel app circuit, named: {Public/Private}ContextInputs
/// - Hashes all of the function input variables
///     - This instantiates a helper function  
fn transform(ty: &str, func: &mut NoirFunction) {
    let context_name = format!("{}Context", ty);
    let inputs_name = format!("{}ContextInputs", ty);
    let return_type_name = format!("{}CircuitPublicInputs", ty);

    // Insert the context creation as the first action
    let create_context = create_context(&context_name, &func.def.parameters);
    func.def.body.0.splice(0..0, (&create_context).iter().cloned());

    // Add the inputs to the params
    let input = create_inputs(&inputs_name);
    func.def.parameters.insert(0, input);

    // Push the finish method call to the end of the function
    let finish_def = create_context_finish();
    func.def.body.0.push(finish_def);

    let return_type = create_return_type(&return_type_name);
    func.def.return_type = return_type;
    func.def.return_visibility = Visibility::Public;

    // Distinct return types are only required for private functions
    if ty.eq("Private") {
        func.def.return_distinctness = Distinctness::Distinct;
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
    let context_ident = ident!("inputs");
    let context_pattern = Pattern::Identifier(context_ident);
    let context_type = UnresolvedType::Named(ident_path!(ty), vec![]);
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
fn create_context(ty: &str, params: &Vec<(Pattern, UnresolvedType, Visibility)>) -> Vec<Statement> {
    let mut injected_expressions: Vec<Statement> = vec![];

    // Hasher object for each variable to call
    let hasher_variable = variable!("hasher");

    // `let mut hasher = Hasher::new();`
    let let_hasher = mutable_assignment!(
        "hasher", // Assigned to
        call!(
            variable_path!(chained_path!("Hasher", "new")), // Path
            vec![]                                          // args
        )
    );

    // Completes: `let mut hasher = Hasher::new();`
    injected_expressions.push(let_hasher);

    // Iterate over each of the function parameters, adding to them to the hasher
    params.iter().for_each(|(pattern, ty, _vis)| {
        match pattern {
            Pattern::Identifier(ident) => {
                // Match the type to determine the padding to do
                match ty {
                    // `hasher.add_multiple({ident}.serialize())`
                    UnresolvedType::Named(..) => {
                        // If this is a struct, we call serialize and add the array to the hasher
                        let serialised_call = method_call!(
                            variable_path!(ident_path!(ident.clone())), // variable
                            "serialize",                                // method name
                            vec![]                                      // args
                        );

                        let add_multiple = Statement::Semi(method_call!(
                            hasher_variable.clone(), // variable
                            "add_multiple",          // method name
                            vec![serialised_call]    // args
                        ));

                        // Add this to the return expressions.
                        injected_expressions.push(add_multiple);
                    }
                    UnresolvedType::Array(..) => {
                        // TODO: if this is an array of structs, we should call serialise on each of them (no methods currently do this yet)
                        // If this is an array of primitive types (integers / fields) we can add them each to the hasher
                        // casted to a field

                        // `array.len()`
                        let end_range_expression = method_call!(
                            variable_path!(ident_path!(ident.clone())), // variable
                            "len",                                      // method name
                            vec![]                                      // args
                        );

                        // Wrap in the semi thing - does that mean ended with semi colon?
                        // `hasher.add({ident}[i] as Field)`
                        let cast_expression = cast!(
                            index_array!(ident.clone(), "i"), // lhs - `ident[i]`
                            UnresolvedType::FieldElement      // cast to - `as Field`
                        );
                        // What will be looped over
                        // - `hasher.add({ident}[i] as Field)`
                        let for_loop_block =
                            expression!(ExpressionKind::Block(BlockExpression(vec![
                                Statement::Semi(method_call!(
                                    hasher_variable.clone(), // variable
                                    "add",                   // method name
                                    vec![cast_expression]
                                ),)
                            ])));

                        // `for i in 0..{ident}.len()`
                        let for_loop = Statement::Expression(expression!(ExpressionKind::For(
                            Box::new(ForExpression {
                                identifier: ident!("i"),
                                start_range: expression!(ExpressionKind::Literal(
                                    Literal::Integer(FieldElement::from(i128::from(0)))
                                )),
                                end_range: end_range_expression,
                                block: for_loop_block,
                            })
                        )));

                        // Add the for loop to our list of return expressions
                        injected_expressions.push(for_loop);
                    }
                    // `hasher.add({ident})`
                    UnresolvedType::FieldElement => {
                        let add_field = Statement::Semi(method_call!(
                            hasher_variable.clone(),                          // variable
                            "add",                                            // method name
                            vec![variable_path!(ident_path!(ident.clone()))]  // args
                        ));
                        injected_expressions.push(add_field);
                    }
                    // Add the integer to the hasher, casted to a field
                    // `hasher.add({ident} as Field)`
                    UnresolvedType::Integer(..) => {
                        // `{ident} as Field`
                        let cast_operation = cast!(
                            variable_path!(ident_path!(ident.clone())), // lhs
                            UnresolvedType::FieldElement                // rhs
                        );

                        // `hasher.add({ident} as Field)`
                        let add_casted_integer = Statement::Semi(method_call!(
                            hasher_variable.clone(), // variable
                            "add",                   // method name
                            vec![cast_operation]     // args
                        ));
                        injected_expressions.push(add_casted_integer);
                    }
                    _ => println!("todo"), // Maybe unreachable?
                }
            }
            _ => todo!(), // Maybe unreachable?
        }
    });

    // Create the inputs to the context
    let inputs_expression = variable!("inputs");
    // `hasher.hash()`
    let hash_call = method_call!(
        variable!("hasher"), // variable
        "hash",              // method name
        vec![]               // args
    );

    // let mut context = {ty}::new(inputs, hash);
    let let_context = mutable_assignment!(
        "context", // Assigned to
        call!(
            variable_path!(chained_path!(ty, "new")), // Path
            vec![inputs_expression, hash_call]        // args
        )
    );
    injected_expressions.push(let_context);

    // Return all expressions that will be injected by the hasher
    return injected_expressions;
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
    let return_path = chained_path!("abi", ty);

    let ty = UnresolvedType::Named(return_path, vec![]);
    FunctionReturnType::Ty(ty, Span::default())
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
    let method_call = method_call!(
        variable!("context"), // variable
        ident!("finish"),     // method name
        vec![]                // args
    );
    Statement::Expression(method_call)
}
