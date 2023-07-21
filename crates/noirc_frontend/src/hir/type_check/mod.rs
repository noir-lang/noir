//! This file contains type_check_func, the entry point to the type checking pass (for each function).
//!
//! The pass structure of type checking is relatively straightforward. It is a single pass through
//! the HIR of each function and outputs the inferred type of each HIR node into the NodeInterner,
//! keyed by the ID of the node.
//!
//! The algorithm for checking and inferring types itself is somewhat ad-hoc. It includes both
//! unification and subtyping, with the only difference between the two being how CompTime
//! is handled (See note on CompTime and make_subtype_of for details). Additionally, although
//! this algorithm features inference via TypeVariables, there is no generalization step as
//! all functions are required to give their full signatures. Closures are inferred but are
//! never generalized and thus cannot be used polymorphically.
mod errors;
mod expr;
mod stmt;

pub use errors::TypeCheckError;
use noirc_errors::Span;

use crate::{
    node_interner::{ExprId, FuncId, NodeInterner, StmtId},
    Type,
};

type TypeCheckFn = Box<dyn FnOnce() -> Result<(), TypeCheckError>>;

pub struct TypeChecker<'interner> {
    delayed_type_checks: Vec<TypeCheckFn>,
    current_function: Option<FuncId>,
    interner: &'interner mut NodeInterner,
    errors: Vec<TypeCheckError>,
}

/// Type checks a function and assigns the
/// appropriate types to expressions in a side table
pub fn type_check_func(interner: &mut NodeInterner, func_id: FuncId) -> Vec<TypeCheckError> {
    let meta = interner.function_meta(&func_id);
    let declared_return_type = meta.return_type().clone();
    let can_ignore_ret = meta.can_ignore_return_type();

    let function_body = interner.function(&func_id);
    let function_body_id = function_body.as_expr();

    let mut type_checker = TypeChecker::new(func_id, interner);

    // Bind each parameter to its annotated type.
    // This is locally obvious, but it must be bound here so that the
    // Definition object of the parameter in the NodeInterner is given the correct type.
    for param in meta.parameters.into_iter() {
        type_checker.bind_pattern(&param.0, param.1);
    }

    let (function_last_type, delayed_type_check_functions, mut errors) =
        type_checker.check_function_body(function_body_id);

    println!("{:?}", errors);
    // Go through any delayed type checking errors to see if they are resolved, or error otherwise.
    for type_check_fn in delayed_type_check_functions {
        if let Err(error) = type_check_fn() {
            errors.push(error);
        }
    }

    // Check declared return type and actual return type
    if !can_ignore_ret {
        let func_span = interner.expr_span(function_body_id); // XXX: We could be more specific and return the span of the last stmt, however stmts do not have spans yet
        function_last_type.make_subtype_of(&declared_return_type, func_span, &mut errors, || {
            TypeCheckError::TypeMismatch {
                expected_typ: declared_return_type.to_string(),
                expr_typ: function_last_type.to_string(),
                expr_span: func_span,
            }
        });
    }

    errors
}

impl<'interner> TypeChecker<'interner> {
    fn new(current_function: FuncId, interner: &'interner mut NodeInterner) -> Self {
        Self {
            delayed_type_checks: Vec::new(),
            current_function: Some(current_function),
            interner,
            errors: vec![],
        }
    }

    pub fn push_delayed_type_check(&mut self, f: TypeCheckFn) {
        self.delayed_type_checks.push(f);
    }

    fn check_function_body(
        mut self,
        body: &ExprId,
    ) -> (Type, Vec<TypeCheckFn>, Vec<TypeCheckError>) {
        let body_type = self.check_expression(body);
        (body_type, self.delayed_type_checks, self.errors)
    }

    pub fn check_global(id: &StmtId, interner: &'interner mut NodeInterner) -> Vec<TypeCheckError> {
        let mut this = Self {
            delayed_type_checks: Vec::new(),
            current_function: None,
            interner,
            errors: vec![],
        };
        this.check_statement(id);
        this.errors
    }

    fn is_unconstrained(&self) -> bool {
        self.current_function.map_or(false, |current_function| {
            self.interner.function_meta(&current_function).is_unconstrained
        })
    }

    /// Wrapper of Type::unify using self.errors
    fn unify(
        &mut self,
        actual: &Type,
        expected: &Type,
        span: Span,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        actual.unify(expected, span, &mut self.errors, make_error);
    }

    /// Wrapper of Type::make_subtype_of using self.errors
    fn make_subtype_of(
        &mut self,
        actual: &Type,
        expected: &Type,
        span: Span,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        actual.make_subtype_of(expected, span, &mut self.errors, make_error);
    }
}

// XXX: These tests are all manual currently.
/// We can either build a test apparatus or pass raw code through the resolver
#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use fm::FileId;
    use iter_extended::vecmap;
    use noirc_errors::{Location, Span};

    use crate::graph::CrateId;
    use crate::hir::def_map::{ModuleData, ModuleId, ModuleOrigin};
    use crate::hir::resolution::import::PathResolutionError;
    use crate::hir_def::expr::HirIdent;
    use crate::hir_def::stmt::HirLetStatement;
    use crate::hir_def::stmt::HirPattern::Identifier;
    use crate::hir_def::types::Type;
    use crate::hir_def::{
        expr::{HirBinaryOp, HirBlockExpression, HirExpression, HirInfixExpression},
        function::{FuncMeta, HirFunction, Param},
        stmt::HirStatement,
    };
    use crate::node_interner::{DefinitionKind, FuncId, NodeInterner};
    use crate::BinaryOpKind;
    use crate::{
        hir::{
            def_map::{CrateDefMap, LocalModuleId, ModuleDefId},
            resolution::{path_resolver::PathResolver, resolver::Resolver},
        },
        parse_program, FunctionKind, Path,
    };

    #[test]
    fn basic_let() {
        let mut interner = NodeInterner::default();

        // Add a simple let Statement into the interner
        // let z = x + y;
        //
        // Push x variable
        let x_id = interner.push_definition("x".into(), false, DefinitionKind::Local(None));

        // Safety: The FileId in a location isn't used for tests
        let file = FileId::default();
        let location = Location::new(Span::default(), file);

        let x = HirIdent { id: x_id, location };

        // Push y variable
        let y_id = interner.push_definition("y".into(), false, DefinitionKind::Local(None));
        let y = HirIdent { id: y_id, location };

        // Push z variable
        let z_id = interner.push_definition("z".into(), false, DefinitionKind::Local(None));
        let z = HirIdent { id: z_id, location };

        // Push x and y as expressions
        let x_expr_id = interner.push_expr(HirExpression::Ident(x));
        let y_expr_id = interner.push_expr(HirExpression::Ident(y));

        // Create Infix
        let operator = HirBinaryOp { location, kind: BinaryOpKind::Add };
        let expr = HirInfixExpression { lhs: x_expr_id, operator, rhs: y_expr_id };
        let expr_id = interner.push_expr(HirExpression::Infix(expr));
        interner.push_expr_location(expr_id, Span::single_char(0), file);

        interner.push_expr_location(x_expr_id, Span::single_char(0), file);
        interner.push_expr_location(y_expr_id, Span::single_char(0), file);

        // Create let statement
        let let_stmt = HirLetStatement {
            pattern: Identifier(z),
            r#type: Type::FieldElement(crate::CompTime::No(None)),
            expression: expr_id,
        };
        let stmt_id = interner.push_stmt(HirStatement::Let(let_stmt));
        let expr_id = interner.push_expr(HirExpression::Block(HirBlockExpression(vec![stmt_id])));
        interner.push_expr_location(expr_id, Span::single_char(0), file);

        // Create function to enclose the let statement
        let func = HirFunction::unchecked_from_expr(expr_id);
        let func_id = interner.push_fn(func);

        let name = HirIdent {
            location,
            id: interner.push_definition("test_func".into(), false, DefinitionKind::Local(None)),
        };

        // Add function meta
        let func_meta = FuncMeta {
            name,
            kind: FunctionKind::Normal,
            attributes: None,
            location,
            contract_function_type: None,
            is_internal: None,
            is_unconstrained: false,
            typ: Type::Function(vec![Type::field(None), Type::field(None)], Box::new(Type::Unit)),
            parameters: vec![
                Param(Identifier(x), Type::field(None), noirc_abi::AbiVisibility::Private),
                Param(Identifier(y), Type::field(None), noirc_abi::AbiVisibility::Private),
            ]
            .into(),
            return_visibility: noirc_abi::AbiVisibility::Private,
            return_distinctness: noirc_abi::AbiDistinctness::DuplicationAllowed,
            has_body: true,
        };
        interner.push_fn_meta(func_meta, func_id);

        let errors = super::type_check_func(&mut interner, func_id);
        assert!(errors.is_empty());
    }

    #[test]
    #[should_panic]
    fn basic_let_stmt() {
        let src = r#"
            fn main(x : Field) {
                let k = [x,x];
                let _z = x + k;
            }
        "#;

        type_check_src_code(src, vec![String::from("main")]);
    }

    #[test]
    fn basic_index_expr() {
        let src = r#"
            fn main(x : Field) {
                let k = [x,x];
                let _z = x + k[0];
            }
        "#;

        type_check_src_code(src, vec![String::from("main")]);
    }
    #[test]
    fn basic_call_expr() {
        let src = r#"
            fn main(x : Field) {
                let _z = x + foo(x);
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
        ) -> Result<ModuleDefId, PathResolutionError> {
            // Not here that foo::bar and hello::foo::bar would fetch the same thing
            let name = path.segments.last().unwrap();
            self.0
                .get(&name.0.contents)
                .cloned()
                .ok_or_else(move || PathResolutionError::Unresolved(name.clone()))
        }

        fn local_module_id(&self) -> LocalModuleId {
            LocalModuleId(arena::Index::from_raw_parts(0, 0))
        }

        fn module_id(&self) -> ModuleId {
            ModuleId { krate: CrateId::dummy_id(), local_id: self.local_module_id() }
        }
    }

    impl TestPathResolver {
        fn insert_func(&mut self, name: String, func_id: FuncId) {
            self.0.insert(name, func_id.into());
        }
    }

    // This function assumes that there is only one function and this is the
    // func id that is returned
    fn type_check_src_code(src: &str, func_namespace: Vec<String>) {
        let (program, errors) = parse_program(src);
        let mut interner = NodeInterner::default();

        // Using assert_eq here instead of assert(errors.is_empty()) displays
        // the whole vec if the assert fails rather than just two booleans
        assert_eq!(errors, vec![]);

        let main_id = interner.push_fn(HirFunction::empty());
        interner.push_function_definition("main".into(), main_id);

        let func_ids = vecmap(&func_namespace, |name| {
            let id = interner.push_fn(HirFunction::empty());
            interner.push_function_definition(name.into(), id);
            id
        });

        let mut path_resolver = TestPathResolver(HashMap::new());
        for (name, id) in func_namespace.into_iter().zip(func_ids.clone()) {
            path_resolver.insert_func(name.to_owned(), id);
        }

        let mut def_maps: HashMap<CrateId, CrateDefMap> = HashMap::new();
        let file = FileId::default();

        let mut modules = arena::Arena::new();
        modules.insert(ModuleData::new(None, ModuleOrigin::File(file), false));

        def_maps.insert(
            CrateId::dummy_id(),
            CrateDefMap {
                root: path_resolver.local_module_id(),
                modules,
                krate: CrateId::dummy_id(),
                extern_prelude: HashMap::new(),
            },
        );

        let func_meta = vecmap(program.functions, |nf| {
            let resolver = Resolver::new(&mut interner, &path_resolver, &def_maps, file);
            let (hir_func, func_meta, resolver_errors) = resolver.resolve_function(nf, main_id);
            assert_eq!(resolver_errors, vec![]);
            (hir_func, func_meta)
        });

        for ((hir_func, meta), func_id) in func_meta.into_iter().zip(func_ids.clone()) {
            interner.update_fn(func_id, hir_func);
            interner.push_fn_meta(meta, func_id);
        }

        // Type check section
        let errors = super::type_check_func(&mut interner, func_ids.first().cloned().unwrap());
        assert_eq!(errors, vec![]);
    }
}
