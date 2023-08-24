use noirc_errors::Span;

use crate::{
    token::Attribute, CallExpression, Distinctness, Expression, ExpressionKind, FunctionReturnType,
    Ident, LetStatement, MethodCallExpression, NoirFunction, ParsedModule, Pattern, Statement,
    UnresolvedType, Visibility,
};

pub(crate) fn aztec_contracts_macros(mut ast: ParsedModule) -> ParsedModule {
    // Usage -> mut ast -> AztecLib.transform(&mut ast)

    let ast_copy = ast.clone();
    // Covers all functions in the ast
    for func in ast.functions.iter_mut() {
        transform_function(&ast_copy, func);
    }
    for submodule in ast.submodules.iter_mut() {
        for func in submodule.contents.functions.iter_mut() {
            transform_function(&ast_copy, func);
        }
    }
    ast
}

// TODO: might be worth making this a struct to prevent passing the ast around
fn transform_function(ast: &ParsedModule, func: &mut NoirFunction) {
    if let Some(Attribute::Custom(custom_attribute)) = func.def.attribute.as_ref() {
        // TODO: this can just become the one function!!!
        // match based on the custom attribute
        match custom_attribute.as_str() {
            "aztec(private)" => {
                // Edit the ast to inject the private context into the function
                // Create the context using the current params
                let create_context = create_context(ast, "PrivateContext", &func.def.parameters);
                // Insert the context creation as the first action
                func.def.body.0.insert(0, create_context);

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
            }
            "aztec(public)" => {
                let create_context = create_context(ast, "PublicContext", &func.def.parameters);
                // Insert the context creation as the first action
                func.def.body.0.insert(0, create_context);

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
    let context_ident = Ident::new("inputs".to_string(), Span::default());
    let context_pattern = Pattern::Identifier(context_ident);
    let context_type_ident = Ident::new(ty.to_string(), Span::default());
    let context_type = UnresolvedType::Named(crate::Path::from_ident(context_type_ident), vec![]);
    let visibility = Visibility::Private;

    (context_pattern, context_type, visibility)
}

pub(crate) fn create_context(
    ast: &ParsedModule,
    ty: &str, // type
    params: &Vec<(Pattern, UnresolvedType, Visibility)>,
) -> Statement {
    let hash_args = create_hash_args(ast, params);

    let context_ident = Ident::new("context".to_string(), Span::default());
    let context_pattern = Pattern::Identifier(context_ident);
    let context_mut = Pattern::Mutable(Box::new(context_pattern.clone()), Span::default());
    let context_type_ident = Ident::new(ty.to_string(), Span::default());
    let mut context_path = crate::Path::from_ident(context_type_ident);
    let context_type = UnresolvedType::Named(context_path.clone(), vec![]);

    // Create the new context
    context_path.segments.push(Ident::new("new".to_string(), Span::default()));

    let inputs_expression = Expression::new(
        ExpressionKind::Variable(crate::Path::from_ident(Ident::new(
            "inputs".to_string(),
            Span::default(),
        ))),
        Span::default(),
    );
    let hash_call = Expression::new(ExpressionKind::Call(Box::new(hash_args)), Span::default());
    let new_context_args = vec![inputs_expression, hash_call];

    // Call the init of the context
    let expression = Expression::new(
        ExpressionKind::Call(Box::new(CallExpression {
            func: Box::new(Expression::new(
                ExpressionKind::Variable(context_path),
                Span::default(),
            )),
            arguments: new_context_args,
        })),
        Span::default(),
    );

    let let_expression = LetStatement { pattern: context_mut, r#type: context_type, expression };
    Statement::Let(let_expression)
}

/// Creates the private context object to be accessed within the function, the parameters need to be extracted to be
/// appended into the args hash object
fn create_hash_args(
    ast: &ParsedModule,
    params: &Vec<(Pattern, UnresolvedType, Visibility)>,
) -> CallExpression {
    dbg!(&ast.types);
    let mut hash_path = crate::Path::from_ident(Ident::new("abi".to_string(), Span::default()));
    hash_path.segments.push(Ident::new("hash_args".to_string(), Span::default()));

    let param_expressions = params
        .iter()
        .map(|(pattern, ty, _vis)| {
            match pattern {
                Pattern::Identifier(ident) => {
                    dbg!(ident);
                    dbg!(ty);

                    // Match the type to determine the padding to do
                    match ty {
                        UnresolvedType::Named(path, unresolved_type) => {
                            // Find the type definition in the ast
                            // TODO: look for methods where the type is resolved elsewhere

                            let last_index = path.segments.len() - 1;
                            let last_item = path.segments[last_index].0.contents.clone();
                            let type_def = ast
                                .types
                                .iter()
                                .find(|type_def| type_def.name.0.contents == last_item);
                            let type_def_2 = ast.submodules.iter().map(|submodule| {
                                submodule
                                    .contents
                                    .types
                                    .iter()
                                    .find(|type_def| type_def.name.0.contents == last_item)
                            });
                            dbg!(type_def);
                            dbg!(type_def_2);
                            // dbg!(path);
                            // dbg!(unresolved_type);
                        }
                        _ => println!("todo"),
                    }

                    // Converts each type to a Field Element before hashing
                    let variable = Expression::new(
                        ExpressionKind::Variable(crate::Path::from_ident(ident.clone())),
                        Span::default(),
                    );
                    let cast_expression = ExpressionKind::Cast(Box::new(crate::CastExpression {
                        lhs: variable,
                        r#type: UnresolvedType::FieldElement,
                    }));

                    Expression::new(cast_expression, Span::default())
                }
                _ => todo!(),
            }
        })
        .collect::<Vec<Expression>>();

    let args_array = Expression::new(
        ExpressionKind::Literal(crate::Literal::Array(crate::ArrayLiteral::Standard(
            param_expressions.clone(),
        ))),
        Span::default(),
    );

    let call_args = vec![args_array];

    let call_expression = CallExpression {
        func: Box::new(Expression::new(ExpressionKind::Variable(hash_path), Span::default())),
        arguments: call_args,
    };
    return call_expression;
}

pub(crate) fn create_return_type(ty: &str) -> FunctionReturnType {
    let return_ident_base = Ident::new("abi".to_string(), Span::default());
    let return_ident = Ident::new(ty.to_string(), Span::default());
    let mut return_path = crate::Path::from_ident(return_ident_base);
    return_path.segments.push(return_ident);

    let ty = UnresolvedType::Named(return_path, vec![]);
    FunctionReturnType::Ty(ty, Span::default())
}

pub(crate) fn create_context_finish() -> Statement {
    let context_ident = Ident::new("context".to_string(), Span::default());
    let method_call_expression = MethodCallExpression {
        object: Expression::new(
            ExpressionKind::Variable(crate::Path::from_ident(context_ident)),
            Span::default(),
        ),
        method_name: Ident::new("finish".to_string(), Span::default()),
        arguments: vec![],
    };
    let method_call = ExpressionKind::MethodCall(Box::new(method_call_expression));

    let expression = Expression::new(method_call, Span::default());
    Statement::Expression(expression)
}
