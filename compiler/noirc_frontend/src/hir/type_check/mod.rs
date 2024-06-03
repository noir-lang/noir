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
use noirc_errors::Span;

use crate::{
    hir_def::{
        expr::HirExpression,
        function::{Param, Parameters},
        stmt::HirStatement,
        traits::TraitConstraint,
    },
    node_interner::{ExprId, FuncId, GlobalId, NodeInterner},
    Type, TypeBindings,
};

pub use self::errors::Source;

pub struct TypeChecker<'interner> {
    interner: &'interner mut NodeInterner,
    errors: Vec<TypeCheckError>,
    current_function: Option<FuncId>,

    /// Trait constraints are collected during type checking until they are
    /// verified at the end of a function. This is because constraints arise
    /// on each variable, but it is only until function calls when the types
    /// needed for the trait constraint may become known.
    trait_constraints: Vec<(TraitConstraint, ExprId)>,

    /// All type variables created in the current function.
    /// This map is used to default any integer type variables at the end of
    /// a function (before checking trait constraints) if a type wasn't already chosen.
    type_variables: Vec<Type>,
}

/// Type checks a function and assigns the
/// appropriate types to expressions in a side table
pub fn type_check_func(interner: &mut NodeInterner, func_id: FuncId) -> Vec<TypeCheckError> {
    let meta = interner.function_meta(&func_id);
    let declared_return_type = meta.return_type().clone();
    let can_ignore_ret = meta.is_stub();

    let function_body_id = &interner.function(&func_id).as_expr();

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
        check_if_type_is_valid_for_program_input(&type_checker, func_id, &param, &mut errors);
        type_checker.bind_pattern(&param.0, param.1);
    }

    let function_last_type = type_checker.check_function_body(function_body_id);
    // Check declared return type and actual return type
    if !can_ignore_ret {
        let (expr_span, empty_function) = function_info(type_checker.interner, function_body_id);
        let func_span = type_checker.interner.expr_span(function_body_id); // XXX: We could be more specific and return the span of the last stmt, however stmts do not have spans yet
        if let Type::TraitAsType(trait_id, _, generics) = &declared_return_type {
            if type_checker
                .interner
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
                type_checker.interner,
                &mut errors,
                || {
                    let mut error = TypeCheckError::TypeMismatchWithSource {
                        expected: declared_return_type.clone(),
                        actual: function_last_type.clone(),
                        span: func_span,
                        source: Source::Return(expected_return_type, expr_span),
                    };

                    if empty_function {
                        error = error.add_context("implicitly returns `()` as its body has no tail or `return` expression");
                    }
                    error
                },
            );
        }
    }

    // Default any type variables that still need defaulting.
    // This is done before trait impl search since leaving them bindable can lead to errors
    // when multiple impls are available. Instead we default first to choose the Field or u64 impl.
    for typ in &type_checker.type_variables {
        if let Type::TypeVariable(variable, kind) = typ.follow_bindings() {
            let msg = "TypeChecker should only track defaultable type vars";
            variable.bind(kind.default_type().expect(msg));
        }
    }

    // Verify any remaining trait constraints arising from the function body
    for (mut constraint, expr_id) in std::mem::take(&mut type_checker.trait_constraints) {
        let span = type_checker.interner.expr_span(&expr_id);

        if matches!(&constraint.typ, Type::MutableReference(_)) {
            let (_, dereferenced_typ) =
                type_checker.insert_auto_dereferences(expr_id, constraint.typ.clone());
            constraint.typ = dereferenced_typ;
        }

        type_checker.verify_trait_constraint(
            &constraint.typ,
            constraint.trait_id,
            &constraint.trait_generics,
            expr_id,
            span,
        );
    }

    // Now remove all the `where` clause constraints we added
    for constraint in &expected_trait_constraints {
        type_checker.interner.remove_assumed_trait_implementations_for_trait(constraint.trait_id);
    }

    errors.append(&mut type_checker.errors);
    errors
}

/// Only sized types are valid to be used as main's parameters or the parameters to a contract
/// function. If the given type is not sized (e.g. contains a slice or NamedGeneric type), an
/// error is issued.
fn check_if_type_is_valid_for_program_input(
    type_checker: &TypeChecker<'_>,
    func_id: FuncId,
    param: &Param,
    errors: &mut Vec<TypeCheckError>,
) {
    let meta = type_checker.interner.function_meta(&func_id);
    if (meta.is_entry_point && !param.1.is_valid_for_program_input())
        || (meta.has_inline_attribute && !param.1.is_valid_non_inlined_function_input())
    {
        let span = param.0.span();
        errors.push(TypeCheckError::InvalidTypeForEntryPoint { span });
    }
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

/// Checks that the type of a function in a trait impl matches the type
/// of the corresponding function declaration in the trait itself.
///
/// To do this, given a trait such as:
/// `trait Foo<A> { fn foo<B>(...); }`
///
/// And an impl such as:
/// `impl<C> Foo<D> for Bar<E> { fn foo<F>(...); } `
///
/// We have to substitute:
/// - Self for Bar<E>
/// - A for D
/// - B for F
///
/// Before we can type check. Finally, we must also check that the unification
/// result does not introduce any new bindings. This can happen if the impl
/// function's type is more general than that of the trait function. E.g.
/// `fn baz<A, B>(a: A, b: B)` when the impl required `fn baz<A>(a: A, b: A)`.
///
/// This does not type check the body of the impl function.
pub(crate) fn check_trait_impl_method_matches_declaration(
    interner: &mut NodeInterner,
    function: FuncId,
) -> Vec<TypeCheckError> {
    let meta = interner.function_meta(&function);
    let method_name = interner.function_name(&function);
    let mut errors = Vec::new();

    let definition_type = meta.typ.as_monotype();

    let impl_ =
        meta.trait_impl.expect("Trait impl function should have a corresponding trait impl");

    // If the trait implementation is not defined in the interner then there was a previous
    // error in resolving the trait path and there is likely no trait for this impl.
    let Some(impl_) = interner.try_get_trait_implementation(impl_) else {
        return errors;
    };

    let impl_ = impl_.borrow();
    let trait_info = interner.get_trait(impl_.trait_id);

    let mut bindings = TypeBindings::new();
    bindings.insert(
        trait_info.self_type_typevar_id,
        (trait_info.self_type_typevar.clone(), impl_.typ.clone()),
    );

    if trait_info.generics.len() != impl_.trait_generics.len() {
        let expected = trait_info.generics.len();
        let found = impl_.trait_generics.len();
        let span = impl_.ident.span();
        let item = trait_info.name.to_string();
        errors.push(TypeCheckError::GenericCountMismatch { item, expected, found, span });
    }

    // Substitute each generic on the trait with the corresponding generic on the impl
    for (generic, arg) in trait_info.generics.iter().zip(&impl_.trait_generics) {
        bindings.insert(generic.id(), (generic.clone(), arg.clone()));
    }

    // If this is None, the trait does not have the corresponding function.
    // This error should have been caught in name resolution already so we don't
    // issue an error for it here.
    if let Some(trait_fn_id) = trait_info.method_ids.get(method_name) {
        let trait_fn_meta = interner.function_meta(trait_fn_id);

        if trait_fn_meta.direct_generics.len() != meta.direct_generics.len() {
            let expected = trait_fn_meta.direct_generics.len();
            let found = meta.direct_generics.len();
            let span = meta.name.location.span;
            let item = method_name.to_string();
            errors.push(TypeCheckError::GenericCountMismatch { item, expected, found, span });
        }

        // Substitute each generic on the trait function with the corresponding generic on the impl function
        for ((_, trait_fn_generic), (name, impl_fn_generic)) in
            trait_fn_meta.direct_generics.iter().zip(&meta.direct_generics)
        {
            let arg = Type::NamedGeneric(impl_fn_generic.clone(), name.clone());
            bindings.insert(trait_fn_generic.id(), (trait_fn_generic.clone(), arg));
        }

        let (declaration_type, _) = trait_fn_meta.typ.instantiate_with_bindings(bindings, interner);

        check_function_type_matches_expected_type(
            &declaration_type,
            definition_type,
            method_name,
            &meta.parameters,
            meta.name.location.span,
            &trait_info.name.0.contents,
            &mut errors,
        );
    }

    errors
}

fn check_function_type_matches_expected_type(
    expected: &Type,
    actual: &Type,
    method_name: &str,
    actual_parameters: &Parameters,
    span: Span,
    trait_name: &str,
    errors: &mut Vec<TypeCheckError>,
) {
    let mut bindings = TypeBindings::new();
    // Shouldn't need to unify envs, they should always be equal since they're both free functions
    if let (Type::Function(params_a, ret_a, _env_a), Type::Function(params_b, ret_b, _env_b)) =
        (expected, actual)
    {
        if params_a.len() == params_b.len() {
            for (i, (a, b)) in params_a.iter().zip(params_b.iter()).enumerate() {
                if a.try_unify(b, &mut bindings).is_err() {
                    errors.push(TypeCheckError::TraitMethodParameterTypeMismatch {
                        method_name: method_name.to_string(),
                        expected_typ: a.to_string(),
                        actual_typ: b.to_string(),
                        parameter_span: actual_parameters.0[i].0.span(),
                        parameter_index: i + 1,
                    });
                }
            }

            if ret_b.try_unify(ret_a, &mut bindings).is_err() {
                errors.push(TypeCheckError::TypeMismatch {
                    expected_typ: ret_a.to_string(),
                    expr_typ: ret_b.to_string(),
                    expr_span: span,
                });
            }
        } else {
            errors.push(TypeCheckError::MismatchTraitImplNumParameters {
                actual_num_parameters: params_b.len(),
                expected_num_parameters: params_a.len(),
                trait_name: trait_name.to_string(),
                method_name: method_name.to_string(),
                span,
            });
        }
    }

    // If result bindings is not empty, a type variable was bound which means the two
    // signatures were not a perfect match. Note that this relies on us already binding
    // all the expected generics to each other prior to this check.
    if !bindings.is_empty() {
        let expected_typ = expected.to_string();
        let expr_typ = actual.to_string();
        errors.push(TypeCheckError::TypeMismatch { expected_typ, expr_typ, expr_span: span });
    }
}

impl<'interner> TypeChecker<'interner> {
    fn new(interner: &'interner mut NodeInterner) -> Self {
        Self {
            interner,
            errors: Vec::new(),
            trait_constraints: Vec::new(),
            type_variables: Vec::new(),
            current_function: None,
        }
    }

    fn check_function_body(&mut self, body: &ExprId) -> Type {
        self.check_expression(body)
    }

    pub fn check_global(
        id: GlobalId,
        interner: &'interner mut NodeInterner,
    ) -> Vec<TypeCheckError> {
        let mut this = Self {
            interner,
            errors: Vec::new(),
            trait_constraints: Vec::new(),
            type_variables: Vec::new(),
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

    /// Return a fresh integer or field type variable and log it
    /// in self.type_variables to default it later.
    fn polymorphic_integer_or_field(&mut self) -> Type {
        let typ = Type::polymorphic_integer_or_field(self.interner);
        self.type_variables.push(typ.clone());
        typ
    }

    /// Return a fresh integer type variable and log it
    /// in self.type_variables to default it later.
    fn polymorphic_integer(&mut self) -> Type {
        let typ = Type::polymorphic_integer(self.interner);
        self.type_variables.push(typ.clone());
        typ
    }
}

// XXX: These tests are all manual currently.
/// We can either build a test apparatus or pass raw code through the resolver
#[cfg(test)]
pub mod test {
    use std::collections::{BTreeMap, HashMap};
    use std::vec;

    use fm::FileId;
    use iter_extended::btree_map;
    use noirc_errors::{Location, Span};

    use crate::ast::{BinaryOpKind, FunctionKind, FunctionReturnType, Path, Visibility};
    use crate::graph::CrateId;
    use crate::hir::def_map::{ModuleData, ModuleId};
    use crate::hir::resolution::import::{
        PathResolution, PathResolutionError, PathResolutionResult,
    };
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
        parse_program,
    };

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
        let x_expr_id = interner.push_expr(HirExpression::Ident(x.clone(), None));
        let y_expr_id = interner.push_expr(HirExpression::Ident(y.clone(), None));

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
            attributes: vec![],
            comptime: false,
        };
        let stmt_id = interner.push_stmt(HirStatement::Let(let_stmt));
        let expr_id = interner
            .push_expr(HirExpression::Block(HirBlockExpression { statements: vec![stmt_id] }));
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
            has_body: true,
            trait_impl: None,
            return_type: FunctionReturnType::Default(Span::default()),
            trait_constraints: Vec::new(),
            direct_generics: Vec::new(),
            is_entry_point: true,
            is_trait_function: false,
            has_inline_attribute: false,
            all_generics: Vec::new(),
            parameter_idents: Vec::new(),
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

        type_check_src_code(src, vec![String::from("main")]);
    }
    #[test]
    fn basic_closure() {
        let src = r#"
            fn main(x : Field) -> pub Field {
                let closure = |y| y + x;
                closure(x)
            }
        "#;

        type_check_src_code(src, vec![String::from("main")]);
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

    #[test]
    fn fold_entry_point() {
        let src = r#"
            #[fold]
            fn fold(x: &mut Field) -> Field {
                *x
        }
        "#;

        type_check_src_code_errors_expected(src, vec![String::from("fold")], 1);
    }

    #[test]
    fn fold_numeric_generic() {
        let src = r#"
        #[fold]
            fn fold<T>(x: T) -> T {
                x
            }
        "#;

        type_check_src_code(src, vec![String::from("fold")]);
    }
    // This is the same Stub that is in the resolver, maybe we can pull this out into a test module and re-use?
    struct TestPathResolver(HashMap<String, ModuleDefId>);

    impl PathResolver for TestPathResolver {
        fn resolve(
            &self,
            _def_maps: &BTreeMap<CrateId, CrateDefMap>,
            path: Path,
        ) -> PathResolutionResult {
            // Not here that foo::bar and hello::foo::bar would fetch the same thing
            let name = path.segments.last().unwrap();
            self.0
                .get(&name.0.contents)
                .cloned()
                .map(|module_def_id| PathResolution { module_def_id, error: None })
                .ok_or_else(move || PathResolutionError::Unresolved(name.clone()))
        }

        fn local_module_id(&self) -> LocalModuleId {
            LocalModuleId(noirc_arena::Index::unsafe_zeroed())
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

    pub fn type_check_src_code(src: &str, func_namespace: Vec<String>) -> (NodeInterner, FuncId) {
        type_check_src_code_errors_expected(src, func_namespace, 0)
    }

    // This function assumes that there is only one function and this is the
    // func id that is returned
    fn type_check_src_code_errors_expected(
        src: &str,
        func_namespace: Vec<String>,
        expected_num_type_check_errs: usize,
    ) -> (NodeInterner, FuncId) {
        let (program, errors) = parse_program(src);
        let mut interner = NodeInterner::default();
        interner.populate_dummy_operator_traits();

        assert_eq!(
            errors.len(),
            0,
            "expected 0 parser errors, but got {}, errors: {:?}",
            errors.len(),
            errors
        );

        let func_ids = btree_map(&func_namespace, |name| {
            (name.to_string(), interner.push_test_function_definition(name.into()))
        });

        let main_id =
            *func_ids.get("main").unwrap_or_else(|| func_ids.first_key_value().unwrap().1);

        let mut path_resolver = TestPathResolver(HashMap::new());
        for (name, id) in func_ids.iter() {
            path_resolver.insert_func(name.to_owned(), *id);
        }

        let mut def_maps = BTreeMap::new();
        let file = FileId::default();

        let mut modules = noirc_arena::Arena::default();
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

        for nf in program.into_sorted().functions {
            let resolver = Resolver::new(&mut interner, &path_resolver, &def_maps, file);

            let function_id = *func_ids.get(nf.name()).unwrap();
            let (hir_func, func_meta, resolver_errors) = resolver.resolve_function(nf, function_id);

            interner.push_fn_meta(func_meta, function_id);
            interner.update_fn(function_id, hir_func);
            assert_eq!(resolver_errors, vec![]);
        }

        // Type check section
        let mut errors = Vec::new();

        for function in func_ids.values() {
            errors.extend(super::type_check_func(&mut interner, *function));
        }

        assert_eq!(
            errors.len(),
            expected_num_type_check_errs,
            "expected {} type check errors, but got {}, errors: {:?}",
            expected_num_type_check_errs,
            errors.len(),
            errors
        );

        (interner, main_id)
    }
}
