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

// TODO: might be worth making this a struct to prevent passing the ast around
fn transform_function(func: &mut NoirFunction) {
    if let Some(Attribute::Custom(custom_attribute)) = func.def.attribute.as_ref() {
        // TODO: this can just become the one function!!!
        // match based on the custom attribute
        match custom_attribute.as_str() {
            "aztec(private)" => {
                // temp: if the name = entrypoint then print it out

                // Edit the ast to inject the private context into the function
                // Create the context using the current params
                let create_context = create_context("PrivateContext", &func.def.parameters);
                // Insert the context creation as the first action
                func.def.body.0.splice(0..0, (&create_context).iter().cloned());

                // Add the inputs to the params
                let input = create_inputs("PrivateContextInputs");
                func.def.parameters.insert(0, input);

                // Push the finish method call to the end of the function
                let finish_def = create_context_finish();
                func.def.body.0.push(finish_def);

                let return_type = create_return_type("PrivateCircuitPublicInputs");
                func.def.return_type = return_type;
                func.def.return_visibility = Visibility::Public;
                func.def.return_distinctness = Distinctness::Distinct;

                if func.name() == "entrypoint" {
                    // dbg!(&func);
                }
            }
            "aztec(public)" => {
                let create_context = create_context("PublicContext", &func.def.parameters);
                // Insert the context creation as the first action
                func.def.body.0.splice(0..0, (&create_context).iter().cloned());

                // Add the inputs to the params
                let input = create_inputs("PublicContextInputs");
                func.def.parameters.insert(0, input);

                // Push the finish method call to the end of the function
                let finish_def = create_context_finish();
                func.def.body.0.push(finish_def);

                let return_type = create_return_type("PublicCircuitPublicInputs");
                func.def.return_type = return_type;
                func.def.return_visibility = Visibility::Public;
                // func.def.return_distinctness = Distinctness::Distinct;
            }
            _ => return,
        }
        // dbg!(&func);
    }
}

/// Helper function that returns what the private context would look like in the ast
/// This should make it available to be consumed within aztec private annotated functions.
pub(crate) fn create_inputs(ty: &str) -> (Pattern, UnresolvedType, Visibility) {
    let context_ident = ident!("inputs");
    let context_pattern = Pattern::Identifier(context_ident);
    let context_type = UnresolvedType::Named(ident_path!(ty), vec![]);
    let visibility = Visibility::Private;

    (context_pattern, context_type, visibility)
}

/// Creates the private context object to be accessed within the function, the parameters need to be extracted to be
/// appended into the args hash object
fn create_context(ty: &str, params: &Vec<(Pattern, UnresolvedType, Visibility)>) -> Vec<Statement> {
    let mut injected_expressions: Vec<Statement> = vec![];

    let mut hash_path = ident_path!("abi");
    hash_path.segments.push(ident!("hash_args"));

    // Create hasher structure
    let mut hasher_path = ident_path!("abi");
    hasher_path.segments.push(ident!("Hasher"));

    // Assign the hasher to a variable
    let hasher_ident = ident!("hasher");
    let hasher_pattern = Pattern::Identifier(hasher_ident.clone());
    let hasher_mut = Pattern::Mutable(Box::new(hasher_pattern.clone()), Span::default());
    let mut hasher_path = ident_path!("Hasher");
    let context_type = UnresolvedType::Named(hasher_path.clone(), vec![]);

    // Create the new hasher
    hasher_path.segments.push(ident!("new"));

    // Hasher object for each variable to call
    let hasher_variable = variable!("hasher");

    // Define the hasher with a let expression
    let let_hasher = Statement::Let(LetStatement {
        pattern: hasher_mut,
        r#type: context_type,
        expression: expression!(ExpressionKind::Call(Box::new(CallExpression {
            func: Box::new(variable_path!(hasher_path)),
            arguments: vec![],
        }))),
    });

    // Completes: `let hasher = Hasher::new();`
    injected_expressions.push(let_hasher);

    params.iter().for_each(|(pattern, ty, _vis)| {
        match pattern {
            Pattern::Identifier(ident) => {
                // Match the type to determine the padding to do
                match ty {
                    // If we get an unresolved type, then we call serialise on it anf add it to our hasher object
                    UnresolvedType::Named(..) => {
                        // dbg!("Named");
                        // Find the type definition in the ast

                        // If the type is unresolved, then call serialise on it
                        // Create a path calling serialize on ident
                        let serialised_call = method_call!(
                            variable_path!(ident_path!(ident.clone())), // variable
                            "serialize",                                // method name
                            vec![]                                      // args
                        );

                        let add_multi = Statement::Semi(method_call!(
                            hasher_variable.clone(), // variable
                            "add_multiple",          // method name
                            vec![serialised_call]    // args
                        ));

                        // Add this to the return expressions.
                        injected_expressions.push(add_multi);
                    }
                    UnresolvedType::Array(..) => {
                        // Note if this is an array of structs, call the above method on each of them
                        // If this is an array of primitives, then cast them to fields

                        // Create an array pushing the value as fields to the hasher
                        let end_range_expression = method_call!(
                            variable_path!(ident_path!(ident.clone())), // variable
                            "len",                                      // method name
                            vec![]                                      // args
                        );

                        // Wrap in the semi thing - does that mean ended with semi colon?
                        let for_loop_block =
                            ExpressionKind::Block(BlockExpression(vec![Statement::Semi(
                                method_call!(
                                    hasher_variable.clone(), // variable
                                    "add",                   // method name
                                    vec![expression!(ExpressionKind::Cast(Box::new(
                                        CastExpression {
                                            lhs: expression!(ExpressionKind::Index(Box::new(
                                                IndexExpression {
                                                    collection: variable_path!(ident_path!(
                                                        ident.clone()
                                                    )),
                                                    index: variable!("i"),
                                                }
                                            ))),
                                            r#type: UnresolvedType::FieldElement,
                                        }
                                    )))]
                                ),
                            )]));

                        let for_loop = Statement::Expression(expression!(ExpressionKind::For(
                            Box::new(ForExpression {
                                identifier: ident!("i"),
                                start_range: expression!(ExpressionKind::Literal(
                                    Literal::Integer(FieldElement::from(i128::from(0)))
                                )),
                                end_range: end_range_expression,
                                block: expression!(for_loop_block),
                            })
                        )));

                        // Add the for loop to our list of return expressions
                        injected_expressions.push(for_loop);
                    }
                    UnresolvedType::FieldElement => {
                        // dbg!("Field");
                        let add_field = Statement::Semi(method_call!(
                            hasher_variable.clone(),                          // variable
                            "add",                                            // method name
                            vec![variable_path!(ident_path!(ident.clone()))]  // args
                        ));
                        injected_expressions.push(add_field);
                    }
                    UnresolvedType::Integer(_, __) => {
                        // dbg!("Integer");
                        // Add the integer to the hasher, casted
                        let add_casted_integer = Statement::Semi(method_call!(
                            hasher_variable.clone(), // variable
                            "add",                   // method name
                            vec![expression!(ExpressionKind::Cast(Box::new(CastExpression {
                                lhs: variable_path!(ident_path!(ident.clone())),
                                r#type: UnresolvedType::FieldElement,
                            })))]
                        ));
                        injected_expressions.push(add_casted_integer);
                    }
                    _ => println!("todo"),
                }
            }
            _ => todo!(),
        }
    });

    // Create the context from the hasher
    let context_ident = ident!("context");
    let context_pattern = Pattern::Identifier(context_ident);
    let context_mut = Pattern::Mutable(Box::new(context_pattern.clone()), Span::default());
    let context_type_ident = ident!(ty);
    let mut context_path = Path::from_ident(context_type_ident);
    let context_type = UnresolvedType::Named(context_path.clone(), vec![]);

    // Create the new context
    context_path.segments.push(ident!("new"));

    let inputs_expression = variable!("inputs");
    let hash_call = method_call!(
        variable!("hasher"), // variable
        "hash",              // method name
        vec![]               // args
    );
    let new_context_args = vec![inputs_expression, hash_call];

    // Call the init of the context
    let expression = call!(variable_path!(context_path), new_context_args);

    let let_expression =
        Statement::Let(LetStatement { pattern: context_mut, r#type: context_type, expression });
    injected_expressions.push(let_expression);

    // Return all expressions that will be injected by the hasher
    return injected_expressions;
}

pub(crate) fn create_return_type(ty: &str) -> FunctionReturnType {
    let return_ident = ident!(ty);
    let mut return_path = ident_path!("abi");
    return_path.segments.push(return_ident);

    let ty = UnresolvedType::Named(return_path, vec![]);
    FunctionReturnType::Ty(ty, Span::default())
}

pub(crate) fn create_context_finish() -> Statement {
    let method_call = method_call!(
        variable!("context"), // variable
        ident!("finish"),     // method name
        vec![]                // args
    );
    Statement::Expression(method_call)
}
