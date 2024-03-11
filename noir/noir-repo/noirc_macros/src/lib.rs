use noirc_frontend::hir::def_collector::dc_crate::UnresolvedFunctions;
use noirc_frontend::hir::def_collector::dc_crate::UnresolvedTraitImpl;
use noirc_frontend::macros_api::parse_program;
use noirc_frontend::macros_api::HirContext;
use noirc_frontend::macros_api::SortedModule;
use noirc_frontend::macros_api::{CrateId, FileId};
use noirc_frontend::macros_api::{MacroError, MacroProcessor};

pub struct AssertMessageMacro;

impl MacroProcessor for AssertMessageMacro {
    fn process_untyped_ast(
        &self,
        ast: SortedModule,
        crate_id: &CrateId,
        _context: &HirContext,
    ) -> Result<SortedModule, (MacroError, FileId)> {
        transform(ast, crate_id)
    }

    fn process_collected_defs(
        &self,
        _crate_id: &CrateId,
        _context: &mut HirContext,
        _collected_trait_impls: &[UnresolvedTraitImpl],
        _collected_functions: &mut [UnresolvedFunctions],
    ) -> Result<(), (MacroError, FileId)> {
        Ok(())
    }

    // This macro does not need to process any information after name resolution
    fn process_typed_ast(
        &self,
        _crate_id: &CrateId,
        _context: &mut HirContext,
    ) -> Result<(), (MacroError, FileId)> {
        Ok(())
    }
}

fn transform(ast: SortedModule, crate_id: &CrateId) -> Result<SortedModule, (MacroError, FileId)> {
    let ast = add_resolve_assert_message_funcs(ast, crate_id)?;

    Ok(ast)
}

fn add_resolve_assert_message_funcs(
    mut ast: SortedModule,
    crate_id: &CrateId,
) -> Result<SortedModule, (MacroError, FileId)> {
    if !crate_id.is_stdlib() {
        return Ok(ast);
    }
    let assert_message_oracles = "
    #[oracle(assert_message)]
    unconstrained fn assert_message_oracle<T>(_input: T) {}
    unconstrained pub fn resolve_assert_message<T>(input: T, condition: bool) {
        if !condition {
            assert_message_oracle(input);
        }
    }";

    let (assert_msg_funcs_ast, errors) = parse_program(assert_message_oracles);
    assert_eq!(errors.len(), 0, "Failed to parse Noir macro code. This is either a bug in the compiler or the Noir macro code");

    let assert_msg_funcs_ast = assert_msg_funcs_ast.into_sorted();

    for func in assert_msg_funcs_ast.functions {
        ast.functions.push(func)
    }

    Ok(ast)
}
