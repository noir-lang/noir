use crate::{
    hir::type_check::TypeCheckError,
    hir_def::expr::HirIdent,
    macros_api::{HirExpression, NodeInterner},
    node_interner::{DefinitionKind, ExprId},
};

pub(crate) fn lint_deprecated_function(
    interner: &NodeInterner,
    expr: ExprId,
) -> Option<TypeCheckError> {
    let HirExpression::Ident(HirIdent { location, id, impl_kind: _ }, _) =
        interner.expression(&expr)
    else {
        return None;
    };

    let Some(DefinitionKind::Function(func_id)) = interner.try_definition(id).map(|def| &def.kind)
    else {
        return None;
    };

    let attributes = interner.function_attributes(func_id);
    attributes.get_deprecated_note().map(|note| TypeCheckError::CallDeprecated {
        name: interner.definition_name(id).to_string(),
        note,
        span: location.span,
    })
}
