use noirc_errors::Span;

use crate::{
    hir_def::{function::Parameters, stmt::HirPattern},
    token::{Attribute, AztecAttribute},
    CallExpression, Expression, ExpressionKind, FunctionKind, Ident, LetStatement,
    MethodCallExpression, NoirFunction, ParsedModule, Pattern, Statement, UnresolvedType,
};

pub fn aztec_contracts_macros(ast: &mut ParsedModule) {
    // Usage -> mut ast -> AztecLib.transform(&mut ast)

    // TODO: rm
    // loop over all of the functions and print their attributes, to check what attributes look like and if they
    // can be added at will
    // NOTE: only one attribute can be applied per function at the moment

    // Also do this for each submodule
    for func in ast.functions.iter_mut() {
        transform_function(func);
    }
    // Also for each submodule
    for submodule in ast.submodules.iter_mut() {
        for func in submodule.contents.functions.iter_mut() {
            transform_function(func);
        }
    }
}

fn transform_function(func: &mut NoirFunction) {
    // if func.kind == FunctionKind::Aztec {
    //     println!("found aztec attribute");
    // }
    // TODO: this will need to be added to a later codegen pass
    // Write some code to get the aztec attribute out of the attribute type
    let aztec_attribute = func.def.attribute.as_ref().and_then(|attr| {
        if let Attribute::Aztec(aztec) = attr {
            Some(aztec)
        } else {
            None
        }
    });
    if aztec_attribute.is_some() {
        dbg!(&aztec_attribute);
        let t = aztec_attribute.unwrap();
        match t {
            AztecAttribute::Private => {
                // Edit the ast to inject the private context into the function
                // TODO: clean

                // Create the context using the current params
                let create_context = create_private_context(&func.def.parameters);
                // Insert the context creation as the first action
                func.def.body.0.insert(0, create_context);

                // Add the inputs to the params
                let private_input = create_private_inputs();
                func.def.parameters.insert(0, private_input);

                // Push the finish method call to the end of the function
                let finish_def = create_context_finish();
                // dbg!(&finish_def);
                func.def.body.0.push(finish_def);

                let return_type = create_private_return_type();
                func.def.return_type = return_type;
                func.def.return_visibility = noirc_abi::AbiVisibility::Public;
                func.def.return_distinctness = noirc_abi::AbiDistinctness::Distinct;
            }
            AztecAttribute::Public => {}
        }
        dbg!(&func);
    }
}

/// Helper function that returns what the private context would look like in the ast
/// This should make it available to be consumed within aztec private annotated functions.
pub fn create_private_inputs() -> (Pattern, UnresolvedType, noirc_abi::AbiVisibility) {
    let context_ident = Ident::new("inputs".to_string(), Span::default());
    let context_patt = Pattern::Identifier(context_ident);
    let context_type_ident = Ident::new("PrivateContextInputs".to_string(), Span::default());
    let context_type = UnresolvedType::Named(crate::Path::from_ident(context_type_ident), vec![]);
    let visibility = noirc_abi::AbiVisibility::Private;

    (context_patt, context_type, visibility)
}

/// Creates the private context object to be accessed within the function, the parameters need to be extracted to be
/// appended into the args hash object
pub fn create_private_context(
    params: &Vec<(Pattern, UnresolvedType, noirc_abi::AbiVisibility)>,
) -> Statement {
    let hash_args = create_hash_args(params);

    let context_ident = Ident::new("context".to_string(), Span::default());
    let context_patt = Pattern::Identifier(context_ident);
    let context_mut = Pattern::Mutable(Box::new(context_patt.clone()), Span::default());
    let context_type_ident = Ident::new("PrivateContext".to_string(), Span::default());
    let mut context_path = crate::Path::from_ident(context_type_ident);
    let context_type = UnresolvedType::Named(context_path.clone(), vec![]);
    let visibility = noirc_abi::AbiVisibility::Private;

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

fn create_hash_args(
    params: &Vec<(Pattern, UnresolvedType, noirc_abi::AbiVisibility)>,
) -> CallExpression {
    let mut hash_path = crate::Path::from_ident(Ident::new("abi".to_string(), Span::default()));
    hash_path.segments.push(Ident::new("hash_args".to_string(), Span::default()));

    let param_expressions = params
        .iter()
        .map(|param| {
            let param_pattern = &param.0;
            match param_pattern {
                Pattern::Identifier(ident) => Expression::new(
                    ExpressionKind::Variable(crate::Path::from_ident(ident.clone())),
                    Span::default(),
                ),
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

pub fn create_private_return_type() -> UnresolvedType {
    let return_ident_base = Ident::new("abi".to_string(), Span::default());
    let return_ident = Ident::new("PrivateCircuitPublicInputs".to_string(), Span::default());
    let mut return_path = crate::Path::from_ident(return_ident_base);
    return_path.segments.push(return_ident);

    UnresolvedType::Named(return_path, vec![])
}

pub fn create_context_finish() -> Statement {
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

// Expression(
//     Expression {
//         kind: MethodCall(
//             MethodCallExpression {
//                 object: Expression {
//                     kind: Variable(
//                         Path {
//                             segments: [
//                                 Ident(
//                                     Spanned {
//                                         contents: "context",
//                                         span: Span(
//                                             Span {
//                                                 start: ByteIndex(248),
//                                                 end: ByteIndex(254),
//                                             },
//                                         ),
//                                     },
//                                 ),
//                             ],
//                             kind: Plain,
//                         },
//                     ),
//                     span: Span(
//                         Span {
//                             start: ByteIndex(248),
//                             end: ByteIndex(254),
//                         },
//                     ),
//                 },
//                 method_name: Ident(
//                     Spanned {
//                         contents: "finish",
//                         span: Span(
//                             Span {
//                                 start: ByteIndex(256),
//                                 end: ByteIndex(261),
//                             },
//                         ),
//                     },
//                 ),
//                 arguments: [],
//             },
//         ),
//         span: Span(
//             Span {
//                 start: ByteIndex(248),
//                 end: ByteIndex(263),
//             },
//         ),
//     },
// ),

// Let(
//     LetStatement {
//         pattern: Identifier(
//             Ident(
//                 Spanned {
//                     contents: "_x",
//                     span: Span(
//                         Span {
//                             start: ByteIndex(215),
//                             end: ByteIndex(216),
//                         },
//                     ),
//                 },
//             ),
//         ),
//         type: Unspecified,
//         expression: Expression {
//             kind: MethodCall(
//                 MethodCallExpression {
//                     object: Expression {
//                         kind: Variable(
//                             Path {
//                                 segments: [
//                                     Ident(
//                                         Spanned {
//                                             contents: "context",
//                                             span: Span(
//                                                 Span {
//                                                     start: ByteIndex(220),
//                                                     end: ByteIndex(226),
//                                                 },
//                                             ),
//                                         },
//                                     ),
//                                 ],
//                                 kind: Plain,
//                             },
//                         ),
//                         span: Span(
//                             Span {
//                                 start: ByteIndex(220),
//                                 end: ByteIndex(226),
//                             },
//                         ),
//                     },
//                     method_name: Ident(
//                         Spanned {
//                             contents: "this_address",
//                             span: Span(
//                                 Span {
//                                     start: ByteIndex(228),
//                                     end: ByteIndex(239),
//                                 },
//                             ),
//                         },
//                     ),
//                     arguments: [],
//                 },
//             ),
//             span: Span(
//                 Span {
//                     start: ByteIndex(220),
//                     end: ByteIndex(241),
//                 },
//             ),
//         },
//     },
// ),

/// Helper function that returns what the public context would look like in the ast
/// This should make it available to be consumed within aztec public annotated functions.
pub fn create_public_context() {}
