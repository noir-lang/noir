//! This crate represents utility methods which can be useful for testing in other crates
//! which also desire to compile the frontend.
//!
//! This module is split out from the `tests` module and has an additional `test_utils` feature
//! as a module configured only for tests will not be accessible in other crates.
//! A crate that needs to use the methods in this module should add the `noirc_frontend`
//! crate as a dev dependency with the `test_utils` feature activated.
#![cfg(any(test, feature = "test_utils"))]

use std::path::Path;

use crate::elaborator::FrontendOptions;

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::hir::Context;
use crate::hir::def_collector::dc_crate::CompilationError;
use crate::hir::def_collector::dc_crate::DefCollector;
use crate::hir::def_map::CrateDefMap;
use crate::hir::def_map::ModuleData;
use crate::parser::{ItemKind, ParserErrorReason};
use crate::token::SecondaryAttribute;
use crate::{ParsedModule, parse_program};
use fm::FileManager;

use crate::monomorphization::{ast::Program, errors::MonomorphizationError, monomorphize};

pub fn get_monomorphized(src: &str) -> Result<Program, MonomorphizationError> {
    get_monomorphized_with_error_filter(src, |_| false)
}

pub fn get_monomorphized_with_error_filter(
    src: &str,
    ignore_error: impl Fn(&CompilationError) -> bool,
) -> Result<Program, MonomorphizationError> {
    let (_parsed_module, mut context, errors) = get_program(src);

    let errors = errors.into_iter().filter(|e| !ignore_error(e)).collect::<Vec<_>>();
    assert!(
        errors.iter().all(|err| !err.is_error()),
        "Expected monomorphized program to have no errors before monomorphization, but found: {errors:?}"
    );

    let main = context
        .get_main_function(context.root_crate_id())
        .unwrap_or_else(|| panic!("get_monomorphized: test program contains no 'main' function"));

    monomorphize(main, &mut context.def_interner, false)
}

pub(crate) fn has_parser_error(errors: &[CompilationError]) -> bool {
    errors.iter().any(|e| matches!(e, CompilationError::ParseError(_)))
}

pub(crate) fn remove_experimental_warnings(errors: &mut Vec<CompilationError>) {
    errors.retain(|error| match error {
        CompilationError::ParseError(error) => {
            !matches!(error.reason(), Some(ParserErrorReason::ExperimentalFeature(..)))
        }
        _ => true,
    });
}

pub(crate) fn get_program(src: &str) -> (ParsedModule, Context, Vec<CompilationError>) {
    let allow_parser_errors = false;
    get_program_with_options(src, allow_parser_errors, FrontendOptions::test_default())
}

pub enum Expect {
    Bug,
    Success,
    Error,
}

/// Compile a program.
///
/// The stdlib is not available for these snippets.
///
/// An optional test path is supplied as an argument.
/// The existence of a test path indicates that we want to emit integration tests
/// for the supplied program as well.
pub(crate) fn get_program_with_options(
    src: &str,
    allow_parser_errors: bool,
    options: FrontendOptions,
) -> (ParsedModule, Context<'static, 'static>, Vec<CompilationError>) {
    let root = Path::new("/");
    let mut fm = FileManager::new(root);
    let root_file_id = fm.add_file_with_source(Path::new("test_file"), src.to_string()).unwrap();
    let mut context = Context::new(fm, Default::default());
    context.enable_pedantic_solving();

    context.def_interner.populate_dummy_operator_traits();
    let root_crate_id = context.crate_graph.add_crate_root(root_file_id);

    let (program, parser_errors) = parse_program(src, root_file_id);
    let mut errors = vecmap(parser_errors, |e| e.into());
    remove_experimental_warnings(&mut errors);

    if allow_parser_errors || !has_parser_error(&errors) {
        let inner_attributes: Vec<SecondaryAttribute> = program
            .items
            .iter()
            .filter_map(|item| {
                if let ItemKind::InnerAttribute(attribute) = &item.kind {
                    Some(attribute.clone())
                } else {
                    None
                }
            })
            .collect();

        let location = Location::new(Default::default(), root_file_id);
        let root_module = ModuleData::new(
            None,
            location,
            Vec::new(),
            inner_attributes.clone(),
            false, // is contract
            false, // is struct
        );

        let def_map = CrateDefMap::new(root_crate_id, root_module);

        // Now we want to populate the CrateDefMap using the DefCollector
        errors.extend(DefCollector::collect_crate_and_dependencies(
            def_map,
            &mut context,
            program.clone().into_sorted(),
            root_file_id,
            options,
        ));
    }

    (program, context, errors)
}
