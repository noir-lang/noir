mod errors;
mod expr;
mod stmt;

// Type checking at the moment is very simple due to what is supported in the grammar.
// If polymorphism is never need, then Wands algorithm should be powerful enough to accommodate
// all foreseeable types, if it is needed then we would need to switch to Hindley-Milner type or maybe bidirectional

use errors::TypeCheckError;
use expr::type_check_expression;

use crate::node_interner::{FuncId, NodeInterner};

/// Type checks a function and assigns the
/// appropriate types to expressions in a side table
pub fn type_check_func(interner: &mut NodeInterner, func_id: FuncId) -> Result<(), TypeCheckError> {
    // First fetch the metadata and add the types for parameters
    // Note that we do not look for the defining Identifier for a parameter,
    // since we know that it is the parameter itself
    let meta = interner.function_meta(&func_id);
    let declared_return_type = &meta.return_type;
    let can_ignore_ret = meta.can_ignore_return_type();

    for param in meta.parameters.into_iter() {
        interner.push_ident_type(&param.0, param.1);
    }

    // Fetch the HirFunction and iterate all of it's statements
    let hir_func = interner.function(&func_id);
    let func_as_expr = hir_func.as_expr();

    // Convert the function to a block expression and then type check the block expr
    type_check_expression(interner, func_as_expr)?;

    // Check declared return type and actual return type
    let function_last_type = interner.id_type(func_as_expr);

    if !can_ignore_ret && (&function_last_type != declared_return_type) {
        let func_span = interner.id_span(func_as_expr); // XXX: We could be more specific and return the span of the last stmt, however stmts do not have spans yet
        return Err(TypeCheckError::TypeMismatch {
            expected_typ: declared_return_type.to_string(),
            expr_typ: function_last_type.to_string(),
            expr_span: func_span,
        });
    }

    // Return type cannot be public
    if declared_return_type.is_public() {
        return Err(TypeCheckError::PublicReturnType {
            typ: declared_return_type.clone(),
            span: interner.id_span(func_as_expr),
        });
    }

    Ok(())
}

// XXX: These tests are all manual currently.
/// We can either build a test apparatus or pass raw code through the resolver
#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use noirc_errors::{Span, Spanned};

    use crate::hir_def::{
        expr::{
            HirBinaryOp, HirBinaryOpKind, HirBlockExpression, HirExpression, HirInfixExpression,
        },
        function::{FuncMeta, HirFunction, Param},
        stmt::{HirPrivateStatement, HirStatement},
    };
    use crate::node_interner::{FuncId, NodeInterner};
    use crate::{graph::CrateId, Ident};
    use crate::{
        hir::{
            def_map::{CrateDefMap, ModuleDefId},
            resolution::{path_resolver::PathResolver, resolver::Resolver},
        },
        parse_program, FunctionKind, Path, Type,
    };

    #[test]
    fn basic_priv() {
        let mut interner = NodeInterner::default();

        // Add a simple Priv Statement into the interner
        // let z = x + y;
        //
        // Push x variable
        let x_id = interner.push_ident(Spanned::from(Span::default(), String::from("x")).into());
        interner.linked_ident_to_def(x_id, x_id);
        // Push y variable
        let y_id = interner.push_ident(Spanned::from(Span::default(), String::from("y")).into());
        interner.linked_ident_to_def(y_id, y_id);
        // Push z variable
        let z_id = interner.push_ident(Spanned::from(Span::default(), String::from("z")).into());
        interner.linked_ident_to_def(z_id, z_id);

        // Push x and y as expressions
        let x_expr_id = interner.push_expr(HirExpression::Ident(x_id));
        let y_expr_id = interner.push_expr(HirExpression::Ident(y_id));

        // Create Infix
        let operator = HirBinaryOp {
            span: Span::default(),
            kind: HirBinaryOpKind::Add,
        };
        let expr = HirInfixExpression {
            lhs: x_expr_id,
            operator,
            rhs: y_expr_id,
        };
        let expr_id = interner.push_expr(HirExpression::Infix(expr));

        // Create priv statement
        let priv_stmt = HirPrivateStatement {
            identifier: z_id,
            r#type: crate::Type::Unspecified,
            expression: expr_id,
        };
        let stmt_id = interner.push_stmt(HirStatement::Private(priv_stmt));

        let expr_id = interner.push_expr(HirExpression::Block(HirBlockExpression(vec![stmt_id])));

        // Create function to enclose the private statement
        let func = HirFunction::unsafe_from_expr(expr_id);
        let func_id = interner.push_fn(func);

        // Add function meta
        let func_meta = FuncMeta {
            name: String::from("test_func"),
            kind: FunctionKind::Normal,
            attributes: None,
            parameters: vec![Param(x_id, Type::WITNESS), Param(y_id, Type::WITNESS)].into(),
            return_type: Type::Unit,
            has_body: true,
        };
        interner.push_fn_meta(func_meta, func_id);

        super::type_check_func(&mut interner, func_id).unwrap();
    }
    #[test]
    fn basic_priv_simplified() {
        let src = r#"

            fn main(x : Field) {
                priv k = x;
                priv _z = x + k;
            }

        "#;

        type_check_src_code(src, vec![String::from("main")]);
    }
    #[test]
    #[should_panic]
    fn basic_let_stmt() {
        let src = r#"
            fn main(x : Field) {
                let k = [x,x];
                priv _z = x + k;
            }
        "#;

        type_check_src_code(src, vec![String::from("main")]);
    }
    #[test]
    fn basic_index_expr() {
        let src = r#"
            fn main(x : Field) {
                let k = [x,x];
                priv _z = x + k[0];
            }
        "#;

        type_check_src_code(src, vec![String::from("main")]);
    }
    #[test]
    fn basic_call_expr() {
        let src = r#"
            fn main(x : Field) {
                priv _z = x + foo(x);
            }

            fn foo(x : Field) -> Field {
                x
            }
        "#;

        type_check_src_code(src, vec![String::from("main"), String::from("foo")]);
    }
    #[test]
    fn basic_for_expr() {
        let src = r#"
            fn main(_x : Field) {
                let _j = for _i in 0..10 {
                    for _k in 0..100 {

                    }
                };
            }

        "#;

        type_check_src_code(src, vec![String::from("main"), String::from("foo")]);
    }

    // This is the same Stub that is in the resolver, maybe we can pull this out into a test module and re-use?
    struct TestPathResolver(HashMap<String, ModuleDefId>);

    impl PathResolver for TestPathResolver {
        fn resolve(
            &self,
            _def_maps: &HashMap<CrateId, CrateDefMap>,
            path: Path,
        ) -> Result<Option<ModuleDefId>, Ident> {
            // Not here that foo::bar and hello::foo::bar would fetch the same thing
            let name = path.segments.last().unwrap();
            let mod_def = self.0.get(&name.0.contents).cloned();
            match mod_def {
                None => Err(name.clone()),
                Some(_) => Ok(mod_def),
            }
        }
    }

    impl TestPathResolver {
        pub fn insert_func(&mut self, name: String, func_id: FuncId) {
            self.0.insert(name, func_id.into());
        }
    }

    // This function assumes that there is only one function and this is the
    // func id that is returned
    fn type_check_src_code(src: &str, func_namespace: Vec<String>) {
        let program = parse_program(src).unwrap();
        let mut interner = NodeInterner::default();

        let mut func_ids = Vec::new();
        for _ in 0..func_namespace.len() {
            func_ids.push(interner.push_fn(HirFunction::empty()));
        }

        let mut path_resolver = TestPathResolver(HashMap::new());
        for (name, id) in func_namespace.into_iter().zip(func_ids.clone()) {
            path_resolver.insert_func(name, id);
        }

        let def_maps: HashMap<CrateId, CrateDefMap> = HashMap::new();

        let func_meta: Vec<_> = program
            .functions
            .into_iter()
            .map(|nf| {
                let resolver = Resolver::new(&mut interner, &path_resolver, &def_maps);
                resolver.resolve_function(nf).unwrap()
            })
            .collect();

        for ((hir_func, meta), func_id) in func_meta.into_iter().zip(func_ids.clone()) {
            interner.update_fn(func_id, hir_func);
            interner.push_fn_meta(meta, func_id)
        }

        // Type check section
        super::type_check_func(&mut interner, func_ids.first().cloned().unwrap()).unwrap();
    }
}
