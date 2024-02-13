//! This file contains type_check_func, the entry point to the type checking pass (for each function).
//!
//! The pass structure of type checking is relatively straightforward. It is a single pass through
//! the HIR of each function and outputs the inferred type of each HIR node into the NodeInterner,
//! keyed by the ID of the node.
//!
//! Although this algorithm features inference via TypeVariables, there is no generalization step
//! as all functions are required to give their full signatures. Closures are inferred but are
//! never generalized and thus cannot be used polymorphically.
mod errors;
mod expr;
mod stmt;

pub use errors::TypeCheckError;

use crate::{
    hir_def::{expr::HirExpression, stmt::HirStatement, traits::TraitConstraint},
    node_interner::{ExprId, FuncId, GlobalId, NodeInterner},
    Type,
};

use self::errors::Source;

type TypeCheckFn = Box<dyn FnOnce() -> Result<(), TypeCheckError>>;

pub struct TypeChecker<'interner> {
    delayed_type_checks: Vec<TypeCheckFn>,
    interner: &'interner mut NodeInterner,
    errors: Vec<TypeCheckError>,
    current_function: Option<FuncId>,

    /// Trait constraints are collected during type checking until they are
    /// verified at the end of a function. This is because constraints arise
    /// on each variable, but it is only until function calls when the types
    /// needed for the trait constraint may become known.
    trait_constraints: Vec<(TraitConstraint, ExprId)>,
}

/// Type checks a function and assigns the
/// appropriate types to expressions in a side table
pub fn type_check_func(interner: &mut NodeInterner, func_id: FuncId) -> Vec<TypeCheckError> {
    let meta = interner.function_meta(&func_id);
    let declared_return_type = meta.return_type().clone();
    let can_ignore_ret = meta.can_ignore_return_type();

    let function_body = interner.function(&func_id);
    let function_body_id = function_body.as_expr();

    let mut type_checker = TypeChecker::new(interner);
    type_checker.current_function = Some(func_id);

    let meta = type_checker.interner.function_meta(&func_id);
    let parameters = meta.parameters.clone();
    let expected_return_type = meta.return_type.clone();
    let expected_trait_constraints = meta.trait_constraints.clone();
    let name_span = meta.name.location.span;

    let mut errors = Vec::new();

    // Temporarily add any impls in this function's `where` clause to scope
    for constraint in &expected_trait_constraints {
        let object = constraint.typ.clone();
        let trait_id = constraint.trait_id;
        let generics = constraint.trait_generics.clone();

        if !type_checker.interner.add_assumed_trait_implementation(object, trait_id, generics) {
            if let Some(the_trait) = type_checker.interner.try_get_trait(trait_id) {
                let trait_name = the_trait.name.to_string();
                let typ = constraint.typ.clone();
                let span = name_span;
                errors.push(TypeCheckError::UnneededTraitConstraint { trait_name, typ, span });
            }
        }
    }

    // Bind each parameter to its annotated type.
    // This is locally obvious, but it must be bound here so that the
    // Definition object of the parameter in the NodeInterner is given the correct type.
    for param in parameters {
        type_checker.bind_pattern(&param.0, param.1);
    }

    let (function_last_type, delayed_type_check_functions) =
        type_checker.check_function_body(function_body_id);

    // Go through any delayed type checking errors to see if they are resolved, or error otherwise.
    for type_check_fn in delayed_type_check_functions {
        if let Err(error) = type_check_fn() {
            errors.push(error);
        }
    }

    // Verify any remaining trait constraints arising from the function body
    for (constraint, expr_id) in std::mem::take(&mut type_checker.trait_constraints) {
        let span = type_checker.interner.expr_span(&expr_id);
        type_checker.verify_trait_constraint(
            &constraint.typ,
            constraint.trait_id,
            &constraint.trait_generics,
            expr_id,
            span,
        );
    }

    errors.append(&mut type_checker.errors);

    // Now remove all the `where` clause constraints we added
    for constraint in &expected_trait_constraints {
        interner.remove_assumed_trait_implementations_for_trait(constraint.trait_id);
    }

    // Check declared return type and actual return type
    if !can_ignore_ret {
        let (expr_span, empty_function) = function_info(interner, function_body_id);
        let func_span = interner.expr_span(function_body_id); // XXX: We could be more specific and return the span of the last stmt, however stmts do not have spans yet
        if let Type::TraitAsType(trait_id, _, generics) = &declared_return_type {
            if interner
                .lookup_trait_implementation(&function_last_type, *trait_id, generics)
                .is_err()
            {
                let error = TypeCheckError::TypeMismatchWithSource {
                    expected: declared_return_type.clone(),
                    actual: function_last_type,
                    span: func_span,
                    source: Source::Return(expected_return_type, expr_span),
                };
                errors.push(error);
            }
        } else {
            function_last_type.unify_with_coercions(
                &declared_return_type,
                *function_body_id,
                interner,
                &mut errors,
                || {
                    let mut error = TypeCheckError::TypeMismatchWithSource {
                        expected: declared_return_type.clone(),
                        actual: function_last_type.clone(),
                        span: func_span,
                        source: Source::Return(expected_return_type, expr_span),
                    };

                    if empty_function {
                        error = error.add_context(
                        "implicitly returns `()` as its body has no tail or `return` expression",
                    );
                    }
                    error
                },
            );
        }
    }

    errors
}

fn function_info(interner: &NodeInterner, function_body_id: &ExprId) -> (noirc_errors::Span, bool) {
    let (expr_span, empty_function) =
        if let HirExpression::Block(block) = interner.expression(function_body_id) {
            let last_stmt = block.statements().last();
            let mut span = interner.expr_span(function_body_id);

            if let Some(last_stmt) = last_stmt {
                if let HirStatement::Expression(expr) = interner.statement(last_stmt) {
                    span = interner.expr_span(&expr);
                }
            }

            (span, last_stmt.is_none())
        } else {
            (interner.expr_span(function_body_id), false)
        };
    (expr_span, empty_function)
}

impl<'interner> TypeChecker<'interner> {
    fn new(interner: &'interner mut NodeInterner) -> Self {
        Self {
            delayed_type_checks: Vec::new(),
            interner,
            errors: Vec::new(),
            trait_constraints: Vec::new(),
            current_function: None,
        }
    }

    pub fn push_delayed_type_check(&mut self, f: TypeCheckFn) {
        self.delayed_type_checks.push(f);
    }

    fn check_function_body(&mut self, body: &ExprId) -> (Type, Vec<TypeCheckFn>) {
        let body_type = self.check_expression(body);
        (body_type, std::mem::take(&mut self.delayed_type_checks))
    }

    pub fn check_global(
        id: GlobalId,
        interner: &'interner mut NodeInterner,
    ) -> Vec<TypeCheckError> {
        let mut this = Self {
            delayed_type_checks: Vec::new(),
            interner,
            errors: Vec::new(),
            trait_constraints: Vec::new(),
            current_function: None,
        };
        let statement = this.interner.get_global(id).let_statement;
        this.check_statement(&statement);
        this.errors
    }

    /// Wrapper of Type::unify using self.errors
    fn unify(
        &mut self,
        actual: &Type,
        expected: &Type,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        actual.unify(expected, &mut self.errors, make_error);
    }

    /// Wrapper of Type::unify_with_coercions using self.errors
    fn unify_with_coercions(
        &mut self,
        actual: &Type,
        expected: &Type,
        expression: ExprId,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        actual.unify_with_coercions(
            expected,
            expression,
            self.interner,
            &mut self.errors,
            make_error,
        );
    }
}

// XXX: These tests are all manual currently.
/// We can either build a test apparatus or pass raw code through the resolver
#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};
    use std::vec;

    use fm::FileId;
    use iter_extended::vecmap;
    use noirc_errors::{Location, Span};

    use crate::graph::CrateId;
    use crate::hir::def_map::{ModuleData, ModuleId};
    use crate::hir::resolution::import::PathResolutionError;
    use crate::hir_def::expr::HirIdent;
    use crate::hir_def::stmt::HirLetStatement;
    use crate::hir_def::stmt::HirPattern::Identifier;
    use crate::hir_def::types::Type;
    use crate::hir_def::{
        expr::{HirBinaryOp, HirBlockExpression, HirExpression, HirInfixExpression},
        function::{FuncMeta, HirFunction},
        stmt::HirStatement,
    };
    use crate::node_interner::{DefinitionKind, FuncId, NodeInterner, TraitId, TraitMethodId};
    use crate::{
        hir::{
            def_map::{CrateDefMap, LocalModuleId, ModuleDefId},
            resolution::{path_resolver::PathResolver, resolver::Resolver},
        },
        parse_program, FunctionKind, Path,
    };
    use crate::{BinaryOpKind, Distinctness, FunctionReturnType, Visibility};

    #[test]
    fn basic_let() {
        let mut interner = NodeInterner::default();
        interner.populate_dummy_operator_traits();

        // Safety: The FileId in a location isn't used for tests
        let file = FileId::default();
        let location = Location::new(Span::default(), file);

        // Add a simple let Statement into the interner
        // let z = x + y;
        //
        // Push x variable
        let x_id =
            interner.push_definition("x".into(), false, DefinitionKind::Local(None), location);

        let x = HirIdent::non_trait_method(x_id, location);

        // Push y variable
        let y_id =
            interner.push_definition("y".into(), false, DefinitionKind::Local(None), location);
        let y = HirIdent::non_trait_method(y_id, location);

        // Push z variable
        let z_id =
            interner.push_definition("z".into(), false, DefinitionKind::Local(None), location);
        let z = HirIdent::non_trait_method(z_id, location);

        // Push x and y as expressions
        let x_expr_id = interner.push_expr(HirExpression::Ident(x.clone()));
        let y_expr_id = interner.push_expr(HirExpression::Ident(y.clone()));

        // Create Infix
        let operator = HirBinaryOp { location, kind: BinaryOpKind::Add };
        let trait_id = TraitId(ModuleId::dummy_id());
        let trait_method_id = TraitMethodId { trait_id, method_index: 0 };
        let expr = HirInfixExpression { lhs: x_expr_id, operator, rhs: y_expr_id, trait_method_id };
        let expr_id = interner.push_expr(HirExpression::Infix(expr));
        interner.push_expr_location(expr_id, Span::single_char(0), file);

        interner.push_expr_location(x_expr_id, Span::single_char(0), file);
        interner.push_expr_location(y_expr_id, Span::single_char(0), file);

        // Create let statement
        let let_stmt = HirLetStatement {
            pattern: Identifier(z),
            r#type: Type::FieldElement,
            expression: expr_id,
        };
        let stmt_id = interner.push_stmt(HirStatement::Let(let_stmt));
        let expr_id = interner.push_expr(HirExpression::Block(HirBlockExpression(vec![stmt_id])));
        interner.push_expr_location(expr_id, Span::single_char(0), file);

        // Create function to enclose the let statement
        let func = HirFunction::unchecked_from_expr(expr_id);
        let func_id = interner.push_fn(func);

        let definition = DefinitionKind::Local(None);
        let id = interner.push_definition("test_func".into(), false, definition, location);
        let name = HirIdent::non_trait_method(id, location);

        // Add function meta
        let func_meta = FuncMeta {
            name,
            kind: FunctionKind::Normal,
            location,
            typ: Type::Function(
                vec![Type::FieldElement, Type::FieldElement],
                Box::new(Type::Unit),
                Box::new(Type::Unit),
            ),
            parameters: vec![
                (Identifier(x), Type::FieldElement, Visibility::Private),
                (Identifier(y), Type::FieldElement, Visibility::Private),
            ]
            .into(),
            return_visibility: Visibility::Private,
            return_distinctness: Distinctness::DuplicationAllowed,
            has_body: true,
            trait_impl: None,
            return_type: FunctionReturnType::Default(Span::default()),
            trait_constraints: Vec::new(),
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
                for _i in 0..10 {
                    for _k in 0..100 {

                    }
                }
            }

        "#;

        let expected_num_errors = 0;
        type_check_src_code_errors_expected(
            src,
            expected_num_errors,
            vec![String::from("main"), String::from("foo")],
        );
    }
    #[test]
    fn basic_closure() {
        let src = r#"
            fn main(x : Field) -> pub Field {
                let closure = |y| y + x;
                closure(x)
            }
        "#;

        type_check_src_code(src, vec![String::from("main"), String::from("foo")]);
    }

    #[test]
    fn closure_with_no_args() {
        let src = r#"
        fn main(x : Field) -> pub Field {
            let closure = || x;
            closure()
        }
       "#;

        type_check_src_code(src, vec![String::from("main")]);
    }
    // This is the same Stub that is in the resolver, maybe we can pull this out into a test module and re-use?
    struct TestPathResolver(HashMap<String, ModuleDefId>);

    impl PathResolver for TestPathResolver {
        fn resolve(
            &self,
            _def_maps: &BTreeMap<CrateId, CrateDefMap>,
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

    fn type_check_src_code(src: &str, func_namespace: Vec<String>) {
        type_check_src_code_errors_expected(src, 0, func_namespace);
    }

    // This function assumes that there is only one function and this is the
    // func id that is returned
    fn type_check_src_code_errors_expected(
        src: &str,
        expected_number_errors: usize,
        func_namespace: Vec<String>,
    ) {
        let (program, errors) = parse_program(src);
        let mut interner = NodeInterner::default();
        interner.populate_dummy_operator_traits();

        assert_eq!(
            errors.len(),
            expected_number_errors,
            "expected {} errors, but got {}, errors: {:?}",
            expected_number_errors,
            errors.len(),
            errors
        );

        let main_id = interner.push_test_function_definition("main".into());

        let func_ids =
            vecmap(&func_namespace, |name| interner.push_test_function_definition(name.into()));

        let mut path_resolver = TestPathResolver(HashMap::new());
        for (name, id) in func_namespace.into_iter().zip(func_ids.clone()) {
            path_resolver.insert_func(name.to_owned(), id);
        }

        let mut def_maps = BTreeMap::new();
        let file = FileId::default();

        let mut modules = arena::Arena::new();
        let location = Location::new(Default::default(), file);
        modules.insert(ModuleData::new(None, location, false));

        def_maps.insert(
            CrateId::dummy_id(),
            CrateDefMap {
                root: path_resolver.local_module_id(),
                modules,
                krate: CrateId::dummy_id(),
                extern_prelude: BTreeMap::new(),
            },
        );

        let func_meta = vecmap(program.into_sorted().functions, |nf| {
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
