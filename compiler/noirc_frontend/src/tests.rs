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
        errors.extend(DefCollector::collect(
            def_map,
            &mut context,
            program.clone().into_sorted(),
            root_file_id,
            false,
            &[], // No macro processors
        ));
    }
    (program, context, errors)
}

pub(crate) fn get_program_errors(src: &str) -> Vec<(CompilationError, FileId)> {
    get_program(src).2
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

    let errors = get_program_errors(src);
    errors.iter().for_each(|err| println!("{:?}", err));
    assert!(errors.is_empty());
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
                assert_eq!(not_a_trait_name.to_string(), "plain::Default");
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
    let errors = get_program_errors(src);
    errors.iter().for_each(|err| println!("{:?}", err));
    assert!(errors.is_empty());
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

    let errors = get_program_errors(src);
    errors.iter().for_each(|err| println!("{:?}", err));
    assert!(errors.is_empty());
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

    let errors = get_program_errors(src);
    errors.iter().for_each(|err| println!("{:?}", err));
    assert!(errors.is_empty());
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
    assert!(get_program_errors(src).is_empty());
}
#[test]
fn resolve_basic_function() {
    let src = r#"
        fn main(x : Field) {
            let y = x + x;
            assert(y == x);
        }
    "#;
    assert!(get_program_errors(src).is_empty());
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
    assert!(get_program_errors(src).is_empty());
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
    assert!(get_program_errors(src).is_empty());
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
    assert!(get_program_errors(src).is_empty());
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
    assert!(get_program_errors(src).is_empty());
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
    assert!(get_program_errors(src).is_empty());
}

#[test]
fn resolve_basic_closure() {
    let src = r#"
        fn main(x : Field) -> pub Field {
            let closure = |y| y + x;
            closure(x)
        }
    "#;
    assert!(get_program_errors(src).is_empty());
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
    assert!(get_program_errors(src).is_empty(), "there should be no errors");

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
    assert_eq!(get_program_errors(src).len(), 0);
}

// Regression for #4545
#[test]
fn type_aliases_in_main() {
    let src = r#"
        type Outer<N> = [u8; N];
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
