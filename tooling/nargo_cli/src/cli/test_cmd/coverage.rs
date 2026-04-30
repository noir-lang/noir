//! Utilities to support test coverage.

use noirc_driver::CrateId;
use noirc_errors::Location;
use noirc_frontend::hir::Context;

/// Returns the location of every expression in the crate that is not inside a `#[test]` function.
/// Used to build the zero-count baseline for lcov coverage reports.
fn baseline_expr_locations(context: &Context, crate_id: CrateId) -> Vec<Location> {
    let def_map = &context.def_maps[&crate_id];
    let allowed_files = def_map.file_ids();

    let test_body_spans: Vec<Location> = def_map
        .get_all_test_functions(&context.def_interner)
        .filter_map(|test_fn| context.def_interner.function(&test_fn.id).try_as_expr())
        .map(|body_id| context.def_interner.expr_location(&body_id))
        .collect();

    context
        .def_interner
        .expr_locations_for_files(&allowed_files)
        .filter(|loc| {
            !test_body_spans
                .iter()
                .any(|test_loc| test_loc.file == loc.file && test_loc.span.contains(&loc.span))
        })
        .collect()
}
