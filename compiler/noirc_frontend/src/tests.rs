#![cfg(test)]

#[cfg(test)]
mod name_shadowing;

// XXX: These tests repeat a lot of code
// what we should do is have test cases which are passed to a test harness
// A test harness will allow for more expressive and readable tests
use core::panic;
use std::collections::BTreeMap;

use fm::FileId;

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::hir::comptime::InterpreterError;
use crate::hir::def_collector::dc_crate::CompilationError;
use crate::hir::def_collector::errors::{DefCollectorErrorKind, DuplicateType};
use crate::hir::def_map::ModuleData;
use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::import::PathResolutionError;
use crate::hir::type_check::TypeCheckError;
use crate::hir::Context;
use crate::node_interner::{NodeInterner, StmtId};

use crate::hir::def_collector::dc_crate::DefCollector;
use crate::hir_def::expr::HirExpression;
use crate::hir_def::stmt::HirStatement;
use crate::monomorphization::monomorphize;
use crate::parser::ParserErrorReason;
use crate::ParsedModule;
use crate::{
    hir::def_map::{CrateDefMap, LocalModuleId},
    parse_program,
};
use fm::FileManager;
use noirc_arena::Arena;

pub(crate) fn has_parser_error(errors: &[(CompilationError, FileId)]) -> bool {
    errors.iter().any(|(e, _f)| matches!(e, CompilationError::ParseError(_)))
}

pub(crate) fn remove_experimental_warnings(errors: &mut Vec<(CompilationError, FileId)>) {
    errors.retain(|(error, _)| match error {
        CompilationError::ParseError(error) => {
            !matches!(error.reason(), Some(ParserErrorReason::ExperimentalFeature(..)))
        }
        _ => true,
    });
}

pub(crate) fn get_program(src: &str) -> (ParsedModule, Context, Vec<(CompilationError, FileId)>) {
    let root = std::path::Path::new("/");
    let fm = FileManager::new(root);

    let mut context = Context::new(fm, Default::default());
    context.def_interner.populate_dummy_operator_traits();
    let root_file_id = FileId::dummy();
    let root_crate_id = context.crate_graph.add_crate_root(root_file_id);

    let (program, parser_errors) = parse_program(src);
    let mut errors = vecmap(parser_errors, |e| (e.into(), root_file_id));
    remove_experimental_warnings(&mut errors);

    if !has_parser_error(&errors) {
        // Allocate a default Module for the root, giving it a ModuleId
        let mut modules: Arena<ModuleData> = Arena::default();
        let location = Location::new(Default::default(), root_file_id);
        let root = modules.insert(ModuleData::new(None, location, false));

        let def_map = CrateDefMap {
            root: LocalModuleId(root),
            modules,
            krate: root_crate_id,
            extern_prelude: BTreeMap::new(),
        };

        // Now we want to populate the CrateDefMap using the DefCollector
        errors.extend(DefCollector::collect_crate_and_dependencies(
            def_map,
            &mut context,
            program.clone().into_sorted(),
            root_file_id,
            None,  // No debug_comptime_in_file
            false, // Disallow arithmetic generics
            &[],   // No macro processors
        ));
    }
    (program, context, errors)
}

pub(crate) fn get_program_errors(src: &str) -> Vec<(CompilationError, FileId)> {
    get_program(src).2
}

fn assert_no_errors(src: &str) {
    let errors = get_program_errors(src);
    if !errors.is_empty() {
        panic!("Expected no errors, got: {:?}", errors);
    }
}

#[test]
fn check_trait_implemented_for_all_t() {
    let src = "
    trait Default {
        fn default() -> Self;
    }
    
    trait Eq {
        fn eq(self, other: Self) -> bool;
    }
    
    trait IsDefault {
        fn is_default(self) -> bool;
    }
    
    impl<T> IsDefault for T where T: Default + Eq {
        fn is_default(self) -> bool {
            self.eq(T::default())
        }
    }
    
    struct Foo {
        a: u64,
    }
    
    impl Eq for Foo {
        fn eq(self, other: Foo) -> bool { self.a == other.a } 
    }
    
    impl Default for u64 {
        fn default() -> Self {
            0
        }
    }
    
    impl Default for Foo {
        fn default() -> Self {
            Foo { a: Default::default() }
        }
    }
    
    fn main(a: Foo) -> pub bool {
        a.is_default()
    }";
    assert_no_errors(src);
}

#[test]
fn check_trait_implementation_duplicate_method() {
    let src = "
    trait Default {
        fn default(x: Field, y: Field) -> Field;
    }
    
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }
    
    impl Default for Foo {
        // Duplicate trait methods should not compile
        fn default(x: Field, y: Field) -> Field {
            y + 2 * x
        }
        // Duplicate trait methods should not compile
        fn default(x: Field, y: Field) -> Field {
            x + 2 * y
        }
    }
    
    fn main() {}";

    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);

    for (err, _file_id) in errors {
        match &err {
            CompilationError::DefinitionError(DefCollectorErrorKind::Duplicate {
                typ,
                first_def,
                second_def,
            }) => {
                assert_eq!(typ, &DuplicateType::TraitAssociatedFunction);
                assert_eq!(first_def, "default");
                assert_eq!(second_def, "default");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_wrong_method_return_type() {
    let src = "
    trait Default {
        fn default() -> Self;
    }
    
    struct Foo {
    }
    
    impl Default for Foo {
        fn default() -> Field {
            0
        }
    }
    
    fn main() {
    }
    ";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);

    for (err, _file_id) in errors {
        match &err {
            CompilationError::TypeError(TypeCheckError::TypeMismatch {
                expected_typ,
                expr_typ,
                expr_span: _,
            }) => {
                assert_eq!(expected_typ, "Foo");
                assert_eq!(expr_typ, "Field");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_wrong_method_return_type2() {
    let src = "
    trait Default {
        fn default(x: Field, y: Field) -> Self;
    }
    
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }
    
    impl Default for Foo {
        fn default(x: Field, _y: Field) -> Field {
            x
        }
    }
    
    fn main() {
    }";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);

    for (err, _file_id) in errors {
        match &err {
            CompilationError::TypeError(TypeCheckError::TypeMismatch {
                expected_typ,
                expr_typ,
                expr_span: _,
            }) => {
                assert_eq!(expected_typ, "Foo");
                assert_eq!(expr_typ, "Field");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_missing_implementation() {
    let src = "
    trait Default {
        fn default(x: Field, y: Field) -> Self;
    
        fn method2(x: Field) -> Field;
    
    }
    
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }
    
    impl Default for Foo {
        fn default(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }
    
    fn main() {
    }
    ";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);

    for (err, _file_id) in errors {
        match &err {
            CompilationError::DefinitionError(DefCollectorErrorKind::TraitMissingMethod {
                trait_name,
                method_name,
                trait_impl_span: _,
            }) => {
                assert_eq!(trait_name, "Default");
                assert_eq!(method_name, "method2");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_not_in_scope() {
    let src = "
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }
    
    // Default trait does not exist
    impl Default for Foo {
        fn default(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }
    
    fn main() {
    }
    
    ";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);
    for (err, _file_id) in errors {
        match &err {
            CompilationError::DefinitionError(DefCollectorErrorKind::TraitNotFound {
                trait_path,
            }) => {
                assert_eq!(trait_path.as_string(), "Default");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_wrong_method_name() {
    let src = "
    trait Default {
    }
    
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }
    
    // wrong trait name method should not compile
    impl Default for Foo {
        fn does_not_exist(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }
    
    fn main() {
    }";
    let compilation_errors = get_program_errors(src);
    assert!(!has_parser_error(&compilation_errors));
    assert!(
        compilation_errors.len() == 1,
        "Expected 1 compilation error, got: {:?}",
        compilation_errors
    );

    for (err, _file_id) in compilation_errors {
        match &err {
            CompilationError::DefinitionError(DefCollectorErrorKind::MethodNotInTrait {
                trait_name,
                impl_method,
            }) => {
                assert_eq!(trait_name, "Default");
                assert_eq!(impl_method, "does_not_exist");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_wrong_parameter() {
    let src = "
    trait Default {
        fn default(x: Field) -> Self;
    }
    
    struct Foo {
        bar: u32,
    }
    
    impl Default for Foo {
        fn default(x: u32) -> Self {
            Foo {bar: x}
        }
    }
    
    fn main() {
    }
    ";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);

    for (err, _file_id) in errors {
        match &err {
            CompilationError::TypeError(TypeCheckError::TraitMethodParameterTypeMismatch {
                method_name,
                expected_typ,
                actual_typ,
                ..
            }) => {
                assert_eq!(method_name, "default");
                assert_eq!(expected_typ, "Field");
                assert_eq!(actual_typ, "u32");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_wrong_parameter2() {
    let src = "
    trait Default {
        fn default(x: Field, y: Field) -> Self;
    }
    
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }
    
    impl Default for Foo {
        fn default(x: Field, y: Foo) -> Self {
            Self { bar: x, array: [x, y.bar] }
        }
    }
    
    fn main() {
    }";

    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);

    for (err, _file_id) in errors {
        match &err {
            CompilationError::TypeError(TypeCheckError::TraitMethodParameterTypeMismatch {
                method_name,
                expected_typ,
                actual_typ,
                ..
            }) => {
                assert_eq!(method_name, "default");
                assert_eq!(expected_typ, "Field");
                assert_eq!(actual_typ, "Foo");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_wrong_parameter_type() {
    let src = "
    trait Default {
        fn default(x: Field, y: NotAType) -> Field;
    }
    
    fn main(x: Field, y: Field) {
        assert(y == x);
    }";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));

    // This is a duplicate error in the name resolver & type checker.
    // In the elaborator there is no duplicate and only 1 error is issued
    assert!(errors.len() <= 2, "Expected 1 or 2 errors, got: {:?}", errors);

    for (err, _file_id) in errors {
        match &err {
            CompilationError::ResolverError(ResolverError::PathResolutionError(
                PathResolutionError::Unresolved(ident),
            )) => {
                assert_eq!(ident, "NotAType");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_wrong_parameters_count() {
    let src = "
    trait Default {
        fn default(x: Field, y: Field) -> Self;
    }
    
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }
    
    impl Default for Foo {
        fn default(x: Field) -> Self {
            Self { bar: x, array: [x, x] }
        }
    }
    
    fn main() {
    }
    ";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);
    for (err, _file_id) in errors {
        match &err {
            CompilationError::TypeError(TypeCheckError::MismatchTraitImplNumParameters {
                actual_num_parameters,
                expected_num_parameters,
                trait_name,
                method_name,
                ..
            }) => {
                assert_eq!(actual_num_parameters, &1_usize);
                assert_eq!(expected_num_parameters, &2_usize);
                assert_eq!(method_name, "default");
                assert_eq!(trait_name, "Default");
            }
            _ => {
                panic!("No other errors are expected in this test case! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_impl_for_non_type() {
    let src = "
    trait Default {
        fn default(x: Field, y: Field) -> Field;
    }

    impl Default for main {
        fn default(x: Field, y: Field) -> Field {
            x + y
        }
    }

    fn main() {}
    ";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);
    for (err, _file_id) in errors {
        match &err {
            CompilationError::ResolverError(ResolverError::Expected { expected, got, .. }) => {
                assert_eq!(expected, "type");
                assert_eq!(got, "function");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_impl_struct_not_trait() {
    let src = "
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    struct Default {
        x: Field,
        z: Field, 
    }
    
    // Default is a struct not a trait
    impl Default for Foo {
        fn default(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }
    
    fn main() {}
    ";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);
    for (err, _file_id) in errors {
        match &err {
            CompilationError::DefinitionError(DefCollectorErrorKind::NotATrait {
                not_a_trait_name,
            }) => {
                assert_eq!(not_a_trait_name.to_string(), "Default");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_duplicate_declaration() {
    let src = "
    trait Default {
        fn default(x: Field, y: Field) -> Self;
    }
    
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }
    
    impl Default for Foo {
        fn default(x: Field,y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }
    
    
    trait Default {
        fn default(x: Field) -> Self;
    }
    
    fn main() {
    }";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);
    for (err, _file_id) in errors {
        match &err {
            CompilationError::DefinitionError(DefCollectorErrorKind::Duplicate {
                typ,
                first_def,
                second_def,
            }) => {
                assert_eq!(typ, &DuplicateType::Trait);
                assert_eq!(first_def, "Default");
                assert_eq!(second_def, "Default");
            }
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_duplicate_implementation() {
    let src = "
    trait Default {
    }
    struct Foo {
        bar: Field,
    }
    
    impl Default for Foo {
    }
    impl Default for Foo {
    }
    fn main() {
    }
    ";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 2, "Expected 2 errors, got: {:?}", errors);
    for (err, _file_id) in errors {
        match &err {
            CompilationError::DefinitionError(DefCollectorErrorKind::OverlappingImpl {
                ..
            }) => (),
            CompilationError::DefinitionError(DefCollectorErrorKind::OverlappingImplNote {
                ..
            }) => (),
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn check_trait_duplicate_implementation_with_alias() {
    let src = "
    trait Default {
    }
    
    struct MyStruct {
    }
    
    type MyType = MyStruct;
    
    impl Default for MyStruct {
    }
    
    impl Default for MyType {
    }
    
    fn main() {
    }
    ";
    let errors = get_program_errors(src);
    assert!(!has_parser_error(&errors));
    assert!(errors.len() == 2, "Expected 2 errors, got: {:?}", errors);
    for (err, _file_id) in errors {
        match &err {
            CompilationError::DefinitionError(DefCollectorErrorKind::OverlappingImpl {
                ..
            }) => (),
            CompilationError::DefinitionError(DefCollectorErrorKind::OverlappingImplNote {
                ..
            }) => (),
            _ => {
                panic!("No other errors are expected! Found = {:?}", err);
            }
        };
    }
}

#[test]
fn test_impl_self_within_default_def() {
    let src = "
    trait Bar {
        fn ok(self) -> Self;

        fn ref_ok(self) -> Self {
            self.ok()
        }
    }

    impl<T> Bar for (T, T) where T: Bar {
        fn ok(self) -> Self {
            self
        }
    }";
    assert_no_errors(src);
}

#[test]
fn check_trait_as_type_as_fn_parameter() {
    let src = "
    trait Eq {
        fn eq(self, other: Self) -> bool;
    }

    struct Foo {
        a: u64,
    }

    impl Eq for Foo {
        fn eq(self, other: Foo) -> bool { self.a == other.a } 
    }

    fn test_eq(x: impl Eq) -> bool {
        x.eq(x)
    }

    fn main(a: Foo) -> pub bool {
        test_eq(a)
    }";
    assert_no_errors(src);
}

#[test]
fn check_trait_as_type_as_two_fn_parameters() {
    let src = "
    trait Eq {
        fn eq(self, other: Self) -> bool;
    }

    trait Test {
        fn test(self) -> bool;
    }

    struct Foo {
        a: u64,
    }

    impl Eq for Foo {
        fn eq(self, other: Foo) -> bool { self.a == other.a } 
    }

    impl Test for u64 {
        fn test(self) -> bool { self == self } 
    }

    fn test_eq(x: impl Eq, y: impl Test) -> bool {
        x.eq(x) == y.test()
    }

    fn main(a: Foo, b: u64) -> pub bool {
        test_eq(a, b)
    }";
    assert_no_errors(src);
}

fn get_program_captures(src: &str) -> Vec<Vec<String>> {
    let (program, context, _errors) = get_program(src);
    let interner = context.def_interner;
    let mut all_captures: Vec<Vec<String>> = Vec::new();
    for func in program.into_sorted().functions {
        let func_id = interner.find_function(func.name()).unwrap();
        let hir_func = interner.function(&func_id);
        // Iterate over function statements and apply filtering function
        find_lambda_captures(hir_func.block(&interner).statements(), &interner, &mut all_captures);
    }
    all_captures
}

fn find_lambda_captures(stmts: &[StmtId], interner: &NodeInterner, result: &mut Vec<Vec<String>>) {
    for stmt_id in stmts.iter() {
        let hir_stmt = interner.statement(stmt_id);
        let expr_id = match hir_stmt {
            HirStatement::Expression(expr_id) => expr_id,
            HirStatement::Let(let_stmt) => let_stmt.expression,
            HirStatement::Assign(assign_stmt) => assign_stmt.expression,
            HirStatement::Constrain(constr_stmt) => constr_stmt.0,
            HirStatement::Semi(semi_expr) => semi_expr,
            HirStatement::For(for_loop) => for_loop.block,
            HirStatement::Error => panic!("Invalid HirStatement!"),
            HirStatement::Break => panic!("Unexpected break"),
            HirStatement::Continue => panic!("Unexpected continue"),
            HirStatement::Comptime(_) => panic!("Unexpected comptime"),
        };
        let expr = interner.expression(&expr_id);

        get_lambda_captures(expr, interner, result); // TODO: dyn filter function as parameter
    }
}

fn get_lambda_captures(
    expr: HirExpression,
    interner: &NodeInterner,
    result: &mut Vec<Vec<String>>,
) {
    if let HirExpression::Lambda(lambda_expr) = expr {
        let mut cur_capture = Vec::new();

        for capture in lambda_expr.captures.iter() {
            cur_capture.push(interner.definition(capture.ident.id).name.clone());
        }
        result.push(cur_capture);

        // Check for other captures recursively within the lambda body
        let hir_body_expr = interner.expression(&lambda_expr.body);
        if let HirExpression::Block(block_expr) = hir_body_expr {
            find_lambda_captures(block_expr.statements(), interner, result);
        }
    }
}

#[test]
fn resolve_empty_function() {
    let src = "
        fn main() {

        }
    ";
    assert_no_errors(src);
}
#[test]
fn resolve_basic_function() {
    let src = r#"
        fn main(x : Field) {
            let y = x + x;
            assert(y == x);
        }
    "#;
    assert_no_errors(src);
}
#[test]
fn resolve_unused_var() {
    let src = r#"
        fn main(x : Field) {
            let y = x + x;
            assert(x == x);
        }
    "#;

    let errors = get_program_errors(src);
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);
    // It should be regarding the unused variable
    match &errors[0].0 {
        CompilationError::ResolverError(ResolverError::UnusedVariable { ident }) => {
            assert_eq!(&ident.0.contents, "y");
        }
        _ => unreachable!("we should only have an unused var error"),
    }
}

#[test]
fn resolve_unresolved_var() {
    let src = r#"
        fn main(x : Field) {
            let y = x + x;
            assert(y == z);
        }
    "#;
    let errors = get_program_errors(src);
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);
    // It should be regarding the unresolved var `z` (Maybe change to undeclared and special case)
    match &errors[0].0 {
        CompilationError::ResolverError(ResolverError::VariableNotDeclared { name, span: _ }) => {
            assert_eq!(name, "z");
        }
        _ => unimplemented!("we should only have an unresolved variable"),
    }
}

#[test]
fn unresolved_path() {
    let src = "
        fn main(x : Field) {
            let _z = some::path::to::a::func(x);
        }
    ";
    let errors = get_program_errors(src);
    assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);
    for (compilation_error, _file_id) in errors {
        match compilation_error {
            CompilationError::ResolverError(err) => {
                match err {
                    ResolverError::PathResolutionError(PathResolutionError::Unresolved(name)) => {
                        assert_eq!(name.to_string(), "some");
                    }
                    _ => unimplemented!("we should only have an unresolved function"),
                };
            }
            _ => unimplemented!(),
        }
    }
}

#[test]
fn resolve_literal_expr() {
    let src = r#"
        fn main(x : Field) {
            let y = 5;
            assert(y == x);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn multiple_resolution_errors() {
    let src = r#"
        fn main(x : Field) {
           let y = foo::bar(x);
           let z = y + a;
        }
    "#;

    let errors = get_program_errors(src);
    assert!(errors.len() == 3, "Expected 3 errors, got: {:?}", errors);

    // Errors are:
    // `a` is undeclared
    // `z` is unused
    // `foo::bar` does not exist
    for (compilation_error, _file_id) in errors {
        match compilation_error {
            CompilationError::ResolverError(err) => {
                match err {
                    ResolverError::UnusedVariable { ident } => {
                        assert_eq!(&ident.0.contents, "z");
                    }
                    ResolverError::VariableNotDeclared { name, .. } => {
                        assert_eq!(name, "a");
                    }
                    ResolverError::PathResolutionError(PathResolutionError::Unresolved(name)) => {
                        assert_eq!(name.to_string(), "foo");
                    }
                    _ => unimplemented!(),
                };
            }
            _ => unimplemented!(),
        }
    }
}

#[test]
fn resolve_prefix_expr() {
    let src = r#"
        fn main(x : Field) {
            let _y = -x;
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolve_for_expr() {
    let src = r#"
        fn main(x : u64) {
            for i in 1..20 {
                let _z = x + i;
            };
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolve_call_expr() {
    let src = r#"
        fn main(x : Field) {
            let _z = foo(x);
        }

        fn foo(x : Field) -> Field {
            x
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolve_shadowing() {
    let src = r#"
        fn main(x : Field) {
            let x = foo(x);
            let x = x;
            let (x, x) = (x, x);
            let _ = x;
        }

        fn foo(x : Field) -> Field {
            x
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolve_basic_closure() {
    let src = r#"
        fn main(x : Field) -> pub Field {
            let closure = |y| y + x;
            closure(x)
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolve_simplified_closure() {
    // based on bug https://github.com/noir-lang/noir/issues/1088

    let src = r#"fn do_closure(x: Field) -> Field {
        let y = x;
        let ret_capture = || {
          y
        };
        ret_capture()
      }

      fn main(x: Field) {
          assert(do_closure(x) == 100);
      }

      "#;
    let parsed_captures = get_program_captures(src);
    let expected_captures = vec![vec!["y".to_string()]];
    assert_eq!(expected_captures, parsed_captures);
}

#[test]
fn resolve_complex_closures() {
    let src = r#"
        fn main(x: Field) -> pub Field {
            let closure_without_captures = |x: Field| -> Field { x + x };
            let a = closure_without_captures(1);

            let closure_capturing_a_param = |y: Field| -> Field { y + x };
            let b = closure_capturing_a_param(2);

            let closure_capturing_a_local_var = |y: Field| -> Field { y + b };
            let c = closure_capturing_a_local_var(3);

            let closure_with_transitive_captures = |y: Field| -> Field {
                let d = 5;
                let nested_closure = |z: Field| -> Field {
                    let doubly_nested_closure = |w: Field| -> Field { w + x + b };
                    a + z + y + d + x + doubly_nested_closure(4) + x + y
                };
                let res = nested_closure(5);
                res
            };

            a + b + c + closure_with_transitive_captures(6)
        }
    "#;
    assert_no_errors(src);

    let expected_captures = vec![
        vec![],
        vec!["x".to_string()],
        vec!["b".to_string()],
        vec!["x".to_string(), "b".to_string(), "a".to_string()],
        vec!["x".to_string(), "b".to_string(), "a".to_string(), "y".to_string(), "d".to_string()],
        vec!["x".to_string(), "b".to_string()],
    ];

    let parsed_captures = get_program_captures(src);

    assert_eq!(expected_captures, parsed_captures);
}

#[test]
fn resolve_fmt_strings() {
    let src = r#"
        fn main() {
            let string = f"this is i: {i}";
            println(string);

            println(f"I want to print {0}");

            let new_val = 10;
            println(f"random_string{new_val}{new_val}");
        }
        fn println<T>(x : T) -> T {
            x
        }
    "#;

    let errors = get_program_errors(src);
    assert!(errors.len() == 5, "Expected 5 errors, got: {:?}", errors);

    for (err, _file_id) in errors {
        match &err {
            CompilationError::ResolverError(ResolverError::VariableNotDeclared {
                name, ..
            }) => {
                assert_eq!(name, "i");
            }
            CompilationError::ResolverError(ResolverError::NumericConstantInFormatString {
                name,
                ..
            }) => {
                assert_eq!(name, "0");
            }
            CompilationError::TypeError(TypeCheckError::UnusedResultError {
                expr_type: _,
                expr_span,
            }) => {
                let a = src.get(expr_span.start() as usize..expr_span.end() as usize).unwrap();
                assert!(
                    a == "println(string)"
                        || a == "println(f\"I want to print {0}\")"
                        || a == "println(f\"random_string{new_val}{new_val}\")"
                );
            }
            _ => unimplemented!(),
        };
    }
}

fn check_rewrite(src: &str, expected: &str) {
    let (_program, mut context, _errors) = get_program(src);
    let main_func_id = context.def_interner.find_function("main").unwrap();
    let program = monomorphize(main_func_id, &mut context.def_interner).unwrap();
    assert!(format!("{}", program) == expected);
}

#[test]
fn simple_closure_with_no_captured_variables() {
    let src = r#"
    fn main() -> pub Field {
        let x = 1;
        let closure = || x;
        closure()
    }
    "#;

    let expected_rewrite = r#"fn main$f0() -> Field {
    let x$0 = 1;
    let closure$3 = {
        let closure_variable$2 = {
            let env$1 = (x$l0);
            (env$l1, lambda$f1)
        };
        closure_variable$l2
    };
    {
        let tmp$4 = closure$l3;
        tmp$l4.1(tmp$l4.0)
    }
}
fn lambda$f1(mut env$l1: (Field)) -> Field {
    env$l1.0
}
"#;
    check_rewrite(src, expected_rewrite);
}

#[test]
fn deny_cyclic_globals() {
    let src = r#"
        global A = B;
        global B = A;
        fn main() {}
    "#;
    assert_eq!(get_program_errors(src).len(), 1);
}

#[test]
fn deny_cyclic_type_aliases() {
    let src = r#"
        type A = B;
        type B = A;
        fn main() {}
    "#;
    assert_eq!(get_program_errors(src).len(), 1);
}

#[test]
fn ensure_nested_type_aliases_type_check() {
    let src = r#"
        type A = B;
        type B = u8;
        fn main() {
            let _a: A = 0 as u16;
        }
    "#;
    assert_eq!(get_program_errors(src).len(), 1);
}

#[test]
fn type_aliases_in_entry_point() {
    let src = r#"
        type Foo = u8;
        fn main(_x: Foo) {}
    "#;
    assert_eq!(get_program_errors(src).len(), 0);
}

#[test]
fn operators_in_global_used_in_type() {
    let src = r#"
        global ONE = 1;
        global COUNT = ONE + 2;
        fn main() {
            let _array: [Field; COUNT] = [1, 2, 3];
        }
    "#;
    assert_eq!(get_program_errors(src).len(), 0);
}

#[test]
fn break_and_continue_in_constrained_fn() {
    let src = r#"
        fn main() {
            for i in 0 .. 10 {
                if i == 2 {
                    continue;
                }
                if i == 5 {
                    break;
                }
            }
        }
    "#;
    assert_eq!(get_program_errors(src).len(), 2);
}

#[test]
fn break_and_continue_outside_loop() {
    let src = r#"
        unconstrained fn main() {
            continue;
            break;
        }
    "#;
    assert_eq!(get_program_errors(src).len(), 2);
}

// Regression for #2540
#[test]
fn for_loop_over_array() {
    let src = r#"
        fn hello<N>(_array: [u1; N]) {
            for _ in 0..N {}
        }

        fn main() {
            let array: [u1; 2] = [0, 1];
            hello(array);
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(get_program_errors(src).len(), 1);

    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::UseExplicitNumericGeneric { .. })
    ));
}

// Regression for #4545
#[test]
fn type_aliases_in_main() {
    let src = r#"
        type Outer<let N: u32> = [u8; N];
        fn main(_arg: Outer<1>) {}
    "#;
    assert_eq!(get_program_errors(src).len(), 0);
}

#[test]
fn ban_mutable_globals() {
    // Mutable globals are only allowed in a comptime context
    let src = r#"
        mut global FOO: Field = 0;
        fn main() {}
    "#;
    assert_eq!(get_program_errors(src).len(), 1);
}

#[test]
fn deny_inline_attribute_on_unconstrained() {
    let src = r#"
        #[no_predicates]
        unconstrained fn foo(x: Field, y: Field) {
            assert(x != y);
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::NoPredicatesAttributeOnUnconstrained { .. })
    ));
}

#[test]
fn deny_fold_attribute_on_unconstrained() {
    let src = r#"
        #[fold]
        unconstrained fn foo(x: Field, y: Field) {
            assert(x != y);
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::FoldAttributeOnUnconstrained { .. })
    ));
}

#[test]
fn specify_function_types_with_turbofish() {
    let src = r#"
        trait Default {
            fn default() -> Self;
        }

        impl Default for Field {
            fn default() -> Self { 0 }
        }

        impl Default for u64 {
            fn default() -> Self { 0 }
        }

        // Need the above as we don't have access to the stdlib here.
        // We also need to construct a concrete value of `U` without giving away its type
        // as otherwise the unspecified type is ignored.

        fn generic_func<T, U>() -> (T, U) where T: Default, U: Default {
            (T::default(), U::default())
        }
    
        fn main() {
            let _ = generic_func::<u64, Field>();
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 0);
}

#[test]
fn specify_method_types_with_turbofish() {
    let src = r#"
        trait Default {
            fn default() -> Self;
        }

        impl Default for Field {
            fn default() -> Self { 0 }
        }

        // Need the above as we don't have access to the stdlib here.
        // We also need to construct a concrete value of `U` without giving away its type
        // as otherwise the unspecified type is ignored.

        struct Foo<T> {
            inner: T
        }
        
        impl<T> Foo<T> {
            fn generic_method<U>(_self: Self) -> U where U: Default {
                U::default()
            }
        }
        
        fn main() {
            let foo: Foo<Field> = Foo { inner: 1 };
            let _ = foo.generic_method::<Field>();
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 0);
}

#[test]
fn incorrect_turbofish_count_function_call() {
    let src = r#"
        trait Default {
            fn default() -> Self;
        }

        impl Default for Field {
            fn default() -> Self { 0 }
        }

        impl Default for u64 {
            fn default() -> Self { 0 }
        }

        // Need the above as we don't have access to the stdlib here.
        // We also need to construct a concrete value of `U` without giving away its type
        // as otherwise the unspecified type is ignored.

        fn generic_func<T, U>() -> (T, U) where T: Default, U: Default {
            (T::default(), U::default())
        }

        fn main() {
            let _ = generic_func::<u64, Field, Field>();
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::IncorrectTurbofishGenericCount { .. }),
    ));
}

#[test]
fn incorrect_turbofish_count_method_call() {
    let src = r#"
        trait Default {
            fn default() -> Self;
        }

        impl Default for Field {
            fn default() -> Self { 0 }
        }

        // Need the above as we don't have access to the stdlib here.
        // We also need to construct a concrete value of `U` without giving away its type
        // as otherwise the unspecified type is ignored.

        struct Foo<T> {
            inner: T
        }
        
        impl<T> Foo<T> {
            fn generic_method<U>(_self: Self) -> U where U: Default {
                U::default()
            }
        }
        
        fn main() {
            let foo: Foo<Field> = Foo { inner: 1 };
            let _ = foo.generic_method::<Field, u32>();
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::IncorrectTurbofishGenericCount { .. }),
    ));
}

#[test]
fn struct_numeric_generic_in_function() {
    let src = r#"
    struct Foo {
        inner: u64
    }

    fn bar<let N: Foo>() { }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::UnsupportedNumericGenericType { .. }),
    ));
}

#[test]
fn struct_numeric_generic_in_struct() {
    let src = r#"
    struct Foo {
        inner: u64
    }

    struct Bar<let N: Foo> { }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::DefinitionError(
            DefCollectorErrorKind::UnsupportedNumericGenericType { .. }
        ),
    ));
}

#[test]
fn bool_numeric_generic() {
    let src = r#"
    fn read<let N: bool>() -> Field {
        if N {
            0
        } else {
            1
        }
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::UnsupportedNumericGenericType { .. }),
    ));
}

#[test]
fn numeric_generic_binary_operation_type_mismatch() {
    let src = r#"
    fn foo<let N: Field>() -> bool {
        let mut check: bool = true;
        check = N;
        check
    }   
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::TypeMismatchWithSource { .. }),
    ));
}

#[test]
fn bool_generic_as_loop_bound() {
    let src = r#"
    fn read<let N: bool>() {
        let mut fields = [0; N];
        for i in 0..N {
            fields[i] = i + 1;
        }
        assert(fields[0] == 1);
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 2);

    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::UnsupportedNumericGenericType { .. }),
    ));

    let CompilationError::TypeError(TypeCheckError::TypeMismatch {
        expected_typ, expr_typ, ..
    }) = &errors[1].0
    else {
        panic!("Got an error other than a type mismatch");
    };

    assert_eq!(expected_typ, "Field");
    assert_eq!(expr_typ, "bool");
}

#[test]
fn numeric_generic_in_function_signature() {
    let src = r#"
    fn foo<let N: u8>(arr: [Field; N]) -> [Field; N] { arr }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_as_struct_field_type() {
    let src = r#"
    struct Foo<let N: u32> {
        a: Field,
        b: N,
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::NumericGenericUsedForType { .. }),
    ));
}

#[test]
fn normal_generic_as_array_length() {
    let src = r#"
    struct Foo<N> {
        a: Field,
        b: [Field; N],
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    // TODO(https://github.com/noir-lang/noir/issues/5156): This should be switched to a hard type error rather than
    // the `UseExplicitNumericGeneric` once implicit numeric generics are removed.
    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::UseExplicitNumericGeneric { .. }),
    ));
}

#[test]
fn numeric_generic_as_param_type() {
    let src = r#"
    fn foo<let I: Field>(x: I) -> I {
        let _q: I = 5;
        x
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 3);
    // Error from the parameter type
    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::NumericGenericUsedForType { .. }),
    ));
    // Error from the let statement annotated type
    assert!(matches!(
        errors[1].0,
        CompilationError::ResolverError(ResolverError::NumericGenericUsedForType { .. }),
    ));
    // Error from the return type
    assert!(matches!(
        errors[2].0,
        CompilationError::ResolverError(ResolverError::NumericGenericUsedForType { .. }),
    ));
}

#[test]
fn numeric_generic_used_in_nested_type_fail() {
    let src = r#"
    struct Foo<let N: u32> {
        a: Field,
        b: Bar<N>,
    }
    struct Bar<N> {
        inner: N
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::NumericGenericUsedForType { .. }),
    ));
}

#[test]
fn normal_generic_used_in_nested_array_length_fail() {
    let src = r#"
    struct Foo<N> {
        a: Field,
        b: Bar<N>,
    }
    struct Bar<let N: u32> {
        inner: [Field; N]
    }
    "#;
    let errors = get_program_errors(src);
    // TODO(https://github.com/noir-lang/noir/issues/5156): This should be switched to a hard type error once implicit numeric generics are removed.
    assert_eq!(errors.len(), 0);
}

#[test]
fn numeric_generic_used_in_nested_type_pass() {
    // The order of these structs should not be changed to make sure
    // that we are accurately resolving all struct generics before struct fields
    let src = r#"
    struct NestedNumeric<let N: u32> {
        a: Field,
        b: InnerNumeric<N>
    }
    struct InnerNumeric<let N: u32> {
        inner: [u64; N],
    }    
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_used_in_trait() {
    // We want to make sure that `N` in `impl<let N: u32, T> Deserialize<N, T>` does
    // not trigger `expected type, found numeric generic parameter N` as the trait
    // does in fact expect a numeric generic.
    let src = r#"
    struct MyType<T> {
        a: Field,
        b: Field,
        c: Field,
        d: T,
    }
    
    impl<let N: u32, T> Deserialize<N, T> for MyType<T> {
        fn deserialize(fields: [Field; N], other: T) -> Self {
            MyType { a: fields[0], b: fields[1], c: fields[2], d: other }
        }
    }
    
    trait Deserialize<let N: u32, T> {
        fn deserialize(fields: [Field; N], other: T) -> Self;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_in_trait_impl_with_extra_impl_generics() {
    let src = r#"
    trait Default {
        fn default() -> Self;
    }

    struct MyType<T> {
        a: Field,
        b: Field,
        c: Field,
        d: T,
    }
    
    // Make sure that `T` is placed before `N` as we want to test that the order of the generics is correctly maintained.
    // `N` is used first in the trait impl generics (`Deserialize<N> for MyType<T>`).
    // We want to make sure that the compiler correctly accounts for that `N` has a numeric kind
    // while `T` has a normal kind. 
    impl<T, let N: u32> Deserialize<N> for MyType<T> where T: Default {
        fn deserialize(fields: [Field; N]) -> Self {
            MyType { a: fields[0], b: fields[1], c: fields[2], d: T::default() }
        }
    }
    
    trait Deserialize<let N: u32> {
        fn deserialize(fields: [Field; N]) -> Self;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_used_in_where_clause() {
    let src = r#"
    trait Deserialize<let N: u32> {
        fn deserialize(fields: [Field; N]) -> Self;
    }

    fn read<T, let N: u32>() -> T where T: Deserialize<N> {
        let mut fields: [Field; N] = [0; N];
        for i in 0..N {
            fields[i] = i as Field + 1;
        }
        T::deserialize(fields)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_used_in_turbofish() {
    let src = r#"
    fn double<let N: u32>() -> u32 {
        // Used as an expression
        N * 2
    }

    fn double_numeric_generics_test() {
        // Example usage of a numeric generic arguments.
        assert(double::<9>() == 18);
        assert(double::<7 + 8>() == 30);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn constant_used_with_numeric_generic() {
    let src = r#"
    struct ValueNote {
        value: Field,
    }

    trait Serialize<let N: u32> {
        fn serialize(self) -> [Field; N];
    }

    impl Serialize<1> for ValueNote {
        fn serialize(self) -> [Field; 1] {
            [self.value]
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn normal_generic_used_when_numeric_expected_in_where_clause() {
    let src = r#"
    trait Deserialize<let N: u32> {
        fn deserialize(fields: [Field; N]) -> Self;
    }

    fn read<T, N>() -> T where T: Deserialize<N> {
        T::deserialize([0, 1])
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::TypeMismatch { .. }),
    ));

    let src = r#"
    trait Deserialize<let N: u32> {
        fn deserialize(fields: [Field; N]) -> Self;
    }

    fn read<T, N>() -> T where T: Deserialize<N> {
        let mut fields: [Field; N] = [0; N];
        for i in 0..N {
            fields[i] = i as Field + 1;
        }
        T::deserialize(fields)
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::VariableNotDeclared { .. }),
    ));
}

// TODO(https://github.com/noir-lang/noir/issues/5156): Remove this test once we ban implicit numeric generics
#[test]
fn implicit_numeric_generics_elaborator() {
    let src = r#"
    struct BoundedVec<T, MaxLen> {
        storage: [T; MaxLen],
        len: u64,
    }
    
    impl<T, MaxLen> BoundedVec<T, MaxLen> {

        // Test that we have an implicit numeric generic for "Len" as well as "MaxLen"
        pub fn extend_from_bounded_vec<Len>(&mut self, _vec: BoundedVec<T, Len>) { 
            // We do this to avoid an unused variable warning on `self`
            let _ = self.len;
            for _ in 0..Len { }
        }

        pub fn push(&mut self, elem: T) {
            assert(self.len < MaxLen, "push out of bounds");
            self.storage[self.len] = elem;
            self.len += 1;
        }
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 4);

    for error in errors.iter() {
        if let CompilationError::ResolverError(ResolverError::UseExplicitNumericGeneric { ident }) =
            &errors[0].0
        {
            assert!(matches!(ident.0.contents.as_str(), "MaxLen" | "Len"));
        } else {
            panic!("Expected ResolverError::UseExplicitNumericGeneric but got {:?}", error);
        }
    }
}

#[test]
fn quote_code_fragments() {
    // This test ensures we can quote (and unquote/splice) code fragments
    // which by themselves are not valid code. They only need to be valid
    // by the time they are unquoted into the macro's call site.
    let src = r#"
        fn main() {
            comptime {
                concat!(quote { assert( }, quote { false); });
            }
        }

        comptime fn concat(a: Quoted, b: Quoted) -> Quoted {
            quote { $a $b }
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    use InterpreterError::FailingConstraint;
    assert!(matches!(&errors[0].0, CompilationError::InterpreterError(FailingConstraint { .. })));
}

#[test]
fn impl_stricter_than_trait_no_trait_method_constraints() {
    // This test ensures that the error we get from the where clause on the trait impl method
    // is a `DefCollectorErrorKind::ImplIsStricterThanTrait` error.
    let src = r#"
    trait Serialize<let N: u32> {
        // We want to make sure we trigger the error when override a trait method 
        // which itself has no trait constraints.
        fn serialize(self) -> [Field; N];
    }

    trait ToField {
        fn to_field(self) -> Field;
    }

    fn process_array<let N: u32>(array: [Field; N]) -> Field {
        array[0]
    }

    fn serialize_thing<A, let N: u32>(thing: A) -> [Field; N] where A: Serialize<N> {
        thing.serialize()
    }

    struct MyType<T> {
        a: T,
        b: T,
    }

    impl<T> Serialize<2> for MyType<T> {
        fn serialize(self) -> [Field; 2] where T: ToField {
            [ self.a.to_field(), self.b.to_field() ]
        }
    }

    impl<T> MyType<T> {
        fn do_thing_with_serialization_with_extra_steps(self) -> Field {
            process_array(serialize_thing(self))
        }
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0].0,
        CompilationError::DefinitionError(DefCollectorErrorKind::ImplIsStricterThanTrait { .. })
    ));
}

#[test]
fn impl_stricter_than_trait_different_generics() {
    let src = r#"
    trait Default { }

    // Object type of the trait constraint differs
    trait Foo<T> {
        fn foo_good<U>() where T: Default;

        fn foo_bad<U>() where T: Default;
    }

    impl<A> Foo<A> for () {
        fn foo_good<B>() where A: Default {}

        fn foo_bad<B>() where B: Default {}
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    if let CompilationError::DefinitionError(DefCollectorErrorKind::ImplIsStricterThanTrait {
        constraint_typ,
        ..
    }) = &errors[0].0
    {
        assert!(matches!(constraint_typ.to_string().as_str(), "B"));
    } else {
        panic!("Expected DefCollectorErrorKind::ImplIsStricterThanTrait but got {:?}", errors[0].0);
    }
}

#[test]
fn impl_stricter_than_trait_different_object_generics() {
    let src = r#"
    trait MyTrait { }

    trait OtherTrait {}

    struct Option<T> {
        inner: T
    }

    struct OtherOption<T> {
        inner: Option<T>,
    }

    trait Bar<T> {
        fn bar_good<U>() where Option<T>: MyTrait, OtherOption<Option<T>>: OtherTrait;

        fn bar_bad<U>() where Option<T>: MyTrait, OtherOption<Option<T>>: OtherTrait;

        fn array_good<U>() where [T; 8]: MyTrait;

        fn array_bad<U>() where [T; 8]: MyTrait;

        fn tuple_good<U>() where (Option<T>, Option<U>): MyTrait;

        fn tuple_bad<U>() where (Option<T>, Option<U>): MyTrait;
    }

    impl<A> Bar<A> for () {
        fn bar_good<B>() 
        where 
            OtherOption<Option<A>>: OtherTrait, 
            Option<A>: MyTrait { }

        fn bar_bad<B>() 
        where 
            OtherOption<Option<A>>: OtherTrait, 
            Option<B>: MyTrait { }

        fn array_good<B>() where [A; 8]: MyTrait { }

        fn array_bad<B>() where [B; 8]: MyTrait { }

        fn tuple_good<B>() where (Option<A>, Option<B>): MyTrait { }

        fn tuple_bad<B>() where (Option<B>, Option<A>): MyTrait { }
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 3);
    if let CompilationError::DefinitionError(DefCollectorErrorKind::ImplIsStricterThanTrait {
        constraint_typ,
        constraint_name,
        ..
    }) = &errors[0].0
    {
        assert!(matches!(constraint_typ.to_string().as_str(), "Option<B>"));
        assert!(matches!(constraint_name.as_str(), "MyTrait"));
    } else {
        panic!("Expected DefCollectorErrorKind::ImplIsStricterThanTrait but got {:?}", errors[0].0);
    }

    if let CompilationError::DefinitionError(DefCollectorErrorKind::ImplIsStricterThanTrait {
        constraint_typ,
        constraint_name,
        ..
    }) = &errors[1].0
    {
        assert!(matches!(constraint_typ.to_string().as_str(), "[B; 8]"));
        assert!(matches!(constraint_name.as_str(), "MyTrait"));
    } else {
        panic!("Expected DefCollectorErrorKind::ImplIsStricterThanTrait but got {:?}", errors[0].0);
    }

    if let CompilationError::DefinitionError(DefCollectorErrorKind::ImplIsStricterThanTrait {
        constraint_typ,
        constraint_name,
        ..
    }) = &errors[2].0
    {
        assert!(matches!(constraint_typ.to_string().as_str(), "(Option<B>, Option<A>)"));
        assert!(matches!(constraint_name.as_str(), "MyTrait"));
    } else {
        panic!("Expected DefCollectorErrorKind::ImplIsStricterThanTrait but got {:?}", errors[0].0);
    }
}

#[test]
fn impl_stricter_than_trait_different_trait() {
    let src = r#"
    trait Default { }

    trait OtherDefault { }

    struct Option<T> {
        inner: T
    }

    trait Bar<T> {
        fn bar<U>() where Option<T>: Default;
    }

    impl<A> Bar<A> for () {
        // Trait constraint differs due to the trait even though the constraint
        // types are the same.
        fn bar<B>() where Option<A>: OtherDefault {}
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    if let CompilationError::DefinitionError(DefCollectorErrorKind::ImplIsStricterThanTrait {
        constraint_typ,
        constraint_name,
        ..
    }) = &errors[0].0
    {
        assert!(matches!(constraint_typ.to_string().as_str(), "Option<A>"));
        assert!(matches!(constraint_name.as_str(), "OtherDefault"));
    } else {
        panic!("Expected DefCollectorErrorKind::ImplIsStricterThanTrait but got {:?}", errors[0].0);
    }
}

#[test]
fn trait_impl_where_clause_stricter_pass() {
    let src = r#"
    trait MyTrait {
        fn good_foo<T, H>() where H: OtherTrait;

        fn bad_foo<T, H>() where H: OtherTrait;
    }

    trait OtherTrait {}

    struct Option<T> {
        inner: T
    }

    impl<T> MyTrait for [T] where Option<T>: MyTrait {
        fn good_foo<A, B>() where B: OtherTrait { }

        fn bad_foo<A, B>() where A: OtherTrait { }
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    if let CompilationError::DefinitionError(DefCollectorErrorKind::ImplIsStricterThanTrait {
        constraint_typ,
        constraint_name,
        ..
    }) = &errors[0].0
    {
        assert!(matches!(constraint_typ.to_string().as_str(), "A"));
        assert!(matches!(constraint_name.as_str(), "OtherTrait"));
    } else {
        panic!("Expected DefCollectorErrorKind::ImplIsStricterThanTrait but got {:?}", errors[0].0);
    }
}

#[test]
fn impl_stricter_than_trait_different_trait_generics() {
    let src = r#"
    trait Foo<T> {
        fn foo<U>() where T: T2<T>;
    }

    impl<A> Foo<A> for () {
        // Should be A: T2<A>
        fn foo<B>() where A: T2<B> {}
    }

    trait T2<C> {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    if let CompilationError::DefinitionError(DefCollectorErrorKind::ImplIsStricterThanTrait {
        constraint_typ,
        constraint_name,
        constraint_generics,
        ..
    }) = &errors[0].0
    {
        assert!(matches!(constraint_typ.to_string().as_str(), "A"));
        assert!(matches!(constraint_name.as_str(), "T2"));
        assert!(matches!(constraint_generics[0].to_string().as_str(), "B"));
    } else {
        panic!("Expected DefCollectorErrorKind::ImplIsStricterThanTrait but got {:?}", errors[0].0);
    }
}

#[test]
fn impl_not_found_for_inner_impl() {
    // We want to guarantee that we get a no impl found error
    let src = r#"
    trait Serialize<let N: u32> {
        fn serialize(self) -> [Field; N];
    }

    trait ToField {
        fn to_field(self) -> Field;
    }

    fn process_array<let N: u32>(array: [Field; N]) -> Field {
        array[0]
    }

    fn serialize_thing<A, let N: u32>(thing: A) -> [Field; N] where A: Serialize<N> {
        thing.serialize()
    }

    struct MyType<T> {
        a: T,
        b: T,
    }

    impl<T> Serialize<2> for MyType<T> where T: ToField {
        fn serialize(self) -> [Field; 2] {
            [ self.a.to_field(), self.b.to_field() ]
        }
    }

    impl<T> MyType<T> {
        fn do_thing_with_serialization_with_extra_steps(self) -> Field {
            process_array(serialize_thing(self))
        }
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0].0,
        CompilationError::TypeError(TypeCheckError::NoMatchingImplFound { .. })
    ));
}

// Regression for #5388
#[test]
fn comptime_let() {
    let src = r#"fn main() {
        comptime let my_var = 2;
        assert_eq(my_var, 2);
    }"#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 0);
}

#[test]
fn overflowing_u8() {
    let src = r#"
        fn main() {
            let _: u8 = 256;
        }"#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    if let CompilationError::TypeError(error) = &errors[0].0 {
        assert_eq!(
            error.to_string(),
            "The value `2⁸` cannot fit into `u8` which has range `0..=255`"
        );
    } else {
        panic!("Expected OverflowingAssignment error, got {:?}", errors[0].0);
    }
}

#[test]
fn underflowing_u8() {
    let src = r#"
        fn main() {
            let _: u8 = -1;
        }"#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    if let CompilationError::TypeError(error) = &errors[0].0 {
        assert_eq!(
            error.to_string(),
            "The value `-1` cannot fit into `u8` which has range `0..=255`"
        );
    } else {
        panic!("Expected OverflowingAssignment error, got {:?}", errors[0].0);
    }
}

#[test]
fn overflowing_i8() {
    let src = r#"
        fn main() {
            let _: i8 = 128;
        }"#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    if let CompilationError::TypeError(error) = &errors[0].0 {
        assert_eq!(
            error.to_string(),
            "The value `2⁷` cannot fit into `i8` which has range `-128..=127`"
        );
    } else {
        panic!("Expected OverflowingAssignment error, got {:?}", errors[0].0);
    }
}

#[test]
fn underflowing_i8() {
    let src = r#"
        fn main() {
            let _: i8 = -129;
        }"#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    if let CompilationError::TypeError(error) = &errors[0].0 {
        assert_eq!(
            error.to_string(),
            "The value `-129` cannot fit into `i8` which has range `-128..=127`"
        );
    } else {
        panic!("Expected OverflowingAssignment error, got {:?}", errors[0].0);
    }
}

#[test]
fn turbofish_numeric_generic_nested_call() {
    // Check for turbofish numeric generics used with function calls
    let src = r#"
    fn foo<let N: u32>() -> [u8; N] {
        [0; N]
    }

    fn bar<let N: u32>() -> [u8; N] {
        foo::<N>()
    }

    global M: u32 = 3;

    fn main() {
        let _ = bar::<M>();
    }
    "#;
    assert_no_errors(src);

    // Check for turbofish numeric generics used with method calls
    let src = r#"
    struct Foo<T> {
        a: T
    }

    impl<T> Foo<T> {
        fn static_method<let N: u32>() -> [u8; N] {
            [0; N]
        }

        fn impl_method<let N: u32>(self) -> [T; N] {
            [self.a; N]
        }
    }

    fn bar<let N: u32>() -> [u8; N] {
        let _ = Foo::static_method::<N>();
        let x: Foo<u8> = Foo { a: 0 };
        x.impl_method::<N>()
    }

    global M: u32 = 3;

    fn main() {
        let _ = bar::<M>();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn use_super() {
    let src = r#"
    fn some_func() {}

    mod foo {
        use super::some_func;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn use_super_in_path() {
    let src = r#"
    fn some_func() {}

    mod foo {
        fn func() {
            super::some_func();
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_super() {
    let src = "use super::some_func;";
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::DefinitionError(DefCollectorErrorKind::PathResolutionError(
        PathResolutionError::NoSuper(span),
    )) = &errors[0].0
    else {
        panic!("Expected a 'no super' error, got {:?}", errors[0].0);
    };

    assert_eq!(span.start(), 4);
    assert_eq!(span.end(), 9);
}

#[test]
fn cannot_call_unconstrained_function_outside_of_unsafe() {
    let src = r#"
    fn main() {
        foo();
    }

    unconstrained fn foo() {}
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::Unsafe { .. }) = &errors[0].0 else {
        panic!("Expected an 'unsafe' error, got {:?}", errors[0].0);
    };
}

#[test]
fn cannot_call_unconstrained_first_class_function_outside_of_unsafe() {
    let src = r#"
    fn main() {
        let func = foo;
        // Warning should trigger here
        func();
        inner(func);
    }

    fn inner(x: unconstrained fn() -> ()) {
        // Warning should trigger here
        x();
    }

    unconstrained fn foo() {}
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 2);

    for error in &errors {
        let CompilationError::TypeError(TypeCheckError::Unsafe { .. }) = &error.0 else {
            panic!("Expected an 'unsafe' error, got {:?}", errors[0].0);
        };
    }
}

#[test]
fn missing_unsafe_block_when_needing_type_annotations() {
    // This test is a regression check that even when an unsafe block is missing
    // that we still appropriately continue type checking and infer type annotations.
    let src = r#"
    fn main() {
        let z = BigNum { limbs: [2, 0, 0] };
        assert(z.__is_zero() == false);
    }

    struct BigNum<let N: u32> {
        limbs: [u64; N],
    }

    impl<let N: u32> BigNum<N> {
        unconstrained fn __is_zero_impl(self) -> bool {
            let mut result: bool = true;
            for i in 0..N {
                result = result & (self.limbs[i] == 0);
            }
            result
        }
    }

    trait BigNumTrait {
        fn __is_zero(self) -> bool;
    }

    impl<let N: u32> BigNumTrait for BigNum<N> {
        fn __is_zero(self) -> bool {
            self.__is_zero_impl()
        }
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::Unsafe { .. }) = &errors[0].0 else {
        panic!("Expected an 'unsafe' error, got {:?}", errors[0].0);
    };
}

#[test]
fn cannot_pass_unconstrained_function_to_regular_function() {
    let src = r#"
    fn main() {
        let func = foo;
        expect_regular(func);
    }

    unconstrained fn foo() {}

    fn expect_regular(_func: fn() -> ()) {
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::UnsafeFn { .. }) = &errors[0].0 else {
        panic!("Expected an UnsafeFn error, got {:?}", errors[0].0);
    };
}

#[test]
fn cannot_assign_unconstrained_and_regular_fn_to_variable() {
    let src = r#"
    fn main() {
        let _func = if true { foo } else { bar };
    }

    fn foo() {}
    unconstrained fn bar() {}
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::Context { err, .. }) = &errors[0].0 else {
        panic!("Expected a context error, got {:?}", errors[0].0);
    };

    if let TypeCheckError::TypeMismatch { expected_typ, expr_typ, .. } = err.as_ref() {
        assert_eq!(expected_typ, "fn() -> ()");
        assert_eq!(expr_typ, "unconstrained fn() -> ()");
    } else {
        panic!("Expected a type mismatch error, got {:?}", errors[0].0);
    };
}

#[test]
fn can_pass_regular_function_to_unconstrained_function() {
    let src = r#"
    fn main() {
        let func = foo;
        expect_unconstrained(func);
    }

    fn foo() {}

    fn expect_unconstrained(_func: unconstrained fn() -> ()) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn cannot_pass_unconstrained_function_to_constrained_function() {
    let src = r#"
    fn main() {
        let func = foo;
        expect_regular(func);
    }

    unconstrained fn foo() {}

    fn expect_regular(_func: fn() -> ()) {}
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::UnsafeFn { .. }) = &errors[0].0 else {
        panic!("Expected an UnsafeFn error, got {:?}", errors[0].0);
    };
}

#[test]
fn can_assign_regular_function_to_unconstrained_function_in_explicitly_typed_var() {
    let src = r#"
    fn main() {
        let _func: unconstrained fn() -> () = foo;
    }

    fn foo() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn can_assign_regular_function_to_unconstrained_function_in_struct_member() {
    let src = r#"
    fn main() {
        let _ = Foo { func: foo };
    }

    fn foo() {}

    struct Foo {
        func: unconstrained fn() -> (),
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_impl_generics_count_mismatch() {
    let src = r#"
    trait Foo {}

    impl Foo<()> for Field {}

    fn main() {}"#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::GenericCountMismatch {
        item,
        expected,
        found,
        ..
    }) = &errors[0].0
    else {
        panic!("Expected a generic count mismatch error, got {:?}", errors[0].0);
    };

    assert_eq!(item, "Foo");
    assert_eq!(*expected, 0);
    assert_eq!(*found, 1);
}

#[test]
fn bit_not_on_untyped_integer() {
    let src = r#"
    fn main() {
        let _: u32 = 3 & !1;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn duplicate_struct_field() {
    let src = r#"
    struct Foo {
        x: i32,
        x: i32,
    }

    fn main() {}
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::DefinitionError(DefCollectorErrorKind::DuplicateField {
        first_def,
        second_def,
    }) = &errors[0].0
    else {
        panic!("Expected a duplicate field error, got {:?}", errors[0].0);
    };

    assert_eq!(first_def.to_string(), "x");
    assert_eq!(second_def.to_string(), "x");

    assert_eq!(first_def.span().start(), 26);
    assert_eq!(second_def.span().start(), 42);
}

#[test]
fn trait_constraint_on_tuple_type() {
    let src = r#"
        trait Foo<A> {
            fn foo(self, x: A) -> bool;
        }

        fn bar<T, U, V>(x: (T, U), y: V) -> bool where (T, U): Foo<V> {
            x.foo(y)
        }

        fn main() {}"#;
    assert_no_errors(src);
}

#[test]
fn turbofish_in_constructor_generics_mismatch() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let _ = Foo::<i32, i64> { x: 1 };
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::GenericCountMismatch { .. }),
    ));
}

#[test]
fn turbofish_in_constructor() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let x: Field = 0;
        let _ = Foo::<i32> { x: x };
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::TypeMismatch {
        expected_typ, expr_typ, ..
    }) = &errors[0].0
    else {
        panic!("Expected a type mismatch error, got {:?}", errors[0].0);
    };

    assert_eq!(expected_typ, "i32");
    assert_eq!(expr_typ, "Field");
}

#[test]
fn turbofish_in_middle_of_variable_unsupported_yet() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    impl <T> Foo<T> {
        fn new(x: T) -> Self {
            Foo { x }
        }
    }

    fn main() {
        let _ = Foo::<i32>::new(1);
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::UnsupportedTurbofishUsage { .. }),
    ));
}

#[test]
fn turbofish_in_struct_pattern() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let value: Field = 0;
        let Foo::<Field> { x } = Foo { x: value };
        let _ = x;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn turbofish_in_struct_pattern_errors_if_type_mismatch() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let value: Field = 0;
        let Foo::<i32> { x } = Foo { x: value };
        let _ = x;
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::TypeMismatchWithSource { .. }) = &errors[0].0
    else {
        panic!("Expected a type mismatch error, got {:?}", errors[0].0);
    };
}

#[test]
fn turbofish_in_struct_pattern_generic_count_mismatch() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let value = 0;
        let Foo::<i32, i64> { x } = Foo { x: value };
        let _ = x;
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::GenericCountMismatch {
        item,
        expected,
        found,
        ..
    }) = &errors[0].0
    else {
        panic!("Expected a generic count mismatch error, got {:?}", errors[0].0);
    };

    assert_eq!(item, "struct Foo");
    assert_eq!(*expected, 1);
    assert_eq!(*found, 2);
}

#[test]
fn incorrect_generic_count_on_struct_impl() {
    let src = r#"
    struct Foo {}
    impl <T> Foo<T> {}
    fn main() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::IncorrectGenericCount {
        actual,
        expected,
        ..
    }) = errors[0].0
    else {
        panic!("Expected an incorrect generic count mismatch error, got {:?}", errors[0].0);
    };

    assert_eq!(actual, 1);
    assert_eq!(expected, 0);
}

#[test]
fn incorrect_generic_count_on_type_alias() {
    let src = r#"
    struct Foo {}
    type Bar = Foo<i32>;
    fn main() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::IncorrectGenericCount {
        actual,
        expected,
        ..
    }) = errors[0].0
    else {
        panic!("Expected an incorrect generic count mismatch error, got {:?}", errors[0].0);
    };

    assert_eq!(actual, 1);
    assert_eq!(expected, 0);
}

#[test]
fn uses_self_type_for_struct_function_call() {
    let src = r#"
    struct S { }

    impl S {
        fn one() -> Field {
            1
        }

        fn two() -> Field {
            Self::one() + Self::one()
        }
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn uses_self_type_inside_trait() {
    let src = r#"
    trait Foo {
        fn foo() -> Self {
            Self::bar()
        }

        fn bar() -> Self;
    }

    impl Foo for Field {
        fn bar() -> Self {
            1
        }
    }

    fn main() {
        let _: Field = Foo::foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn uses_self_type_in_trait_where_clause() {
    let src = r#"
    trait Trait {
        fn trait_func() -> bool;
    }

    trait Foo where Self: Trait {
        fn foo(self) -> bool {
            self.trait_func()
        }
    }

    struct Bar {

    }

    impl Foo for Bar {

    }

    fn main() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::UnresolvedMethodCall { method_name, .. }) =
        &errors[0].0
    else {
        panic!("Expected an unresolved method call error, got {:?}", errors[0].0);
    };

    assert_eq!(method_name, "trait_func");
}

#[test]
fn do_not_eagerly_error_on_cast_on_type_variable() {
    let src = r#"
    pub fn foo<T, U>(x: T, f: fn(T) -> U) -> U {
        f(x)
    }

    fn main() {
        let x: u8 = 1;
        let _: Field = foo(x, |x| x as Field);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn error_on_cast_over_type_variable() {
    let src = r#"
    pub fn foo<T, U>(x: T, f: fn(T) -> U) -> U {
        f(x)
    }

    fn main() {
        let x = "a";
        let _: Field = foo(x, |x| x as Field);
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::TypeMismatch { .. })
    ));
}

#[test]
fn trait_impl_for_a_type_that_implements_another_trait() {
    let src = r#"
    trait One {
        fn one(self) -> i32;
    }

    impl One for i32 {
        fn one(self) -> i32 {
            self
        }
    }

    trait Two {
        fn two(self) -> i32;
    }

    impl<T> Two for T where T: One {
        fn two(self) -> i32 {
            self.one() + 1
        }
    }

    fn use_it<T>(t: T) -> i32 where T: Two {
        Two::two(t)
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_impl_for_a_type_that_implements_another_trait_with_another_impl_used() {
    let src = r#"
    trait One {
        fn one(self) -> i32;
    }

    impl One for i32 {
        fn one(self) -> i32 {
            let _ = self;
            1
        }
    }

    trait Two {
        fn two(self) -> i32;
    }

    impl<T> Two for T where T: One {
        fn two(self) -> i32 {
            self.one() + 1
        }
    }

    impl Two for u32 {
        fn two(self) -> i32 {
            let _ = self;
            0
        }
    }

    fn use_it(t: u32) -> i32 {
        Two::two(t)
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}
