//! Tests of the context an elaborator has installed before any item installs its own.

use crate::elaborator::{Elaborator, ElaboratorOptions};
use crate::hir::def_map::ModuleId;
use crate::parser::parse_expression_in_quote_body;
use crate::tests::assert_no_errors;

/// A freshly constructed elaborator has no item installed. Paths it resolves in that state are
/// resolved against the crate root: an item declared at the root is found by its bare name and
/// an item in a submodule only through the submodule.
#[test]
fn elaborator_without_an_item_resolves_paths_against_the_crate_root() {
    let src = r#"
    pub fn at_root() -> Field {
        1
    }

    pub mod inner {
        pub fn in_submodule() -> Field {
            2
        }
    }

    fn main() {}
    "#;
    let mut context = assert_no_errors(src);
    let crate_id = *context.root_crate_id();
    let root = context.def_maps[&crate_id].root();

    let options = ElaboratorOptions {
        debug_comptime_in_file: None,
        enabled_unstable_features: &[],
        disable_required_unstable_features: false,
    };
    let mut elaborator = Elaborator::from_context(&mut context, crate_id, options);
    assert_eq!(elaborator.module_id(), ModuleId { krate: crate_id, local_id: root });

    let errors_elaborating = |elaborator: &mut Elaborator, source: &str| {
        let expr = parse_expression_in_quote_body(source).expect("expression should parse");
        elaborator.elaborate_expression(expr);
        std::mem::take(&mut elaborator.errors).into_errors()
    };

    assert_eq!(errors_elaborating(&mut elaborator, "at_root()"), vec![]);
    assert_eq!(errors_elaborating(&mut elaborator, "inner::in_submodule()"), vec![]);

    let errors = errors_elaborating(&mut elaborator, "in_submodule()");
    assert_eq!(errors.len(), 1, "a submodule's item is not in scope at the root: {errors:?}");
}
