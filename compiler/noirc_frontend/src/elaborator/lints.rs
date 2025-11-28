//! Lint checks for function attributes, visibility, and usage restrictions.

use crate::{
    NamedGeneric, Type, TypeBinding,
    ast::{Ident, NoirFunction},
    graph::CrateId,
    hir::{
        resolution::errors::{PubPosition, ResolverError},
        type_check::TypeCheckError,
    },
    hir_def::{
        expr::{HirBlockExpression, HirExpression, HirIdent, HirLiteral, HirMatch},
        function::FuncMeta,
        stmt::HirStatement,
    },
    node_interner::{
        DefinitionId, DefinitionKind, ExprId, FuncId, FunctionModifiers, NodeInterner,
    },
    shared::{Signedness, Visibility},
    token::FunctionAttributeKind,
};

use noirc_errors::Location;

pub(super) fn deprecated_function(interner: &NodeInterner, expr: ExprId) -> Option<TypeCheckError> {
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
        location,
    })
}

/// Inline attributes are only relevant for constrained functions
/// as all unconstrained functions are not inlined and so
/// associated attributes are disallowed.
pub(super) fn inlining_attributes(
    func: &FuncMeta,
    modifiers: &FunctionModifiers,
) -> Option<ResolverError> {
    if !modifiers.is_unconstrained {
        return None;
    }

    let attribute = modifiers.attributes.function()?;
    let location = attribute.location;
    match &attribute.kind {
        FunctionAttributeKind::NoPredicates => {
            let ident = func_meta_name_ident(func, modifiers);
            Some(ResolverError::NoPredicatesAttributeOnUnconstrained { ident, location })
        }
        FunctionAttributeKind::Fold => {
            let ident = func_meta_name_ident(func, modifiers);
            Some(ResolverError::FoldAttributeOnUnconstrained { ident, location })
        }
        FunctionAttributeKind::Foreign(_)
        | FunctionAttributeKind::Builtin(_)
        | FunctionAttributeKind::Oracle(_)
        | FunctionAttributeKind::Test(_)
        | FunctionAttributeKind::InlineAlways
        | FunctionAttributeKind::FuzzingHarness(_) => None,
    }
}

/// Attempting to define new low level (`#[builtin]` or `#[foreign]`) functions outside of the stdlib is disallowed.
pub(super) fn low_level_function_outside_stdlib(
    modifiers: &FunctionModifiers,
    crate_id: CrateId,
) -> Option<ResolverError> {
    if crate_id.is_stdlib() {
        return None;
    }

    let attribute = modifiers.attributes.function()?;
    if attribute.kind.is_low_level() {
        Some(ResolverError::LowLevelFunctionOutsideOfStdlib { location: attribute.location })
    } else {
        None
    }
}

/// Oracle definitions (functions with the `#[oracle]` attribute) must be marked as unconstrained.
pub(super) fn oracle_not_marked_unconstrained(
    func: &FuncMeta,
    modifiers: &FunctionModifiers,
) -> Option<ResolverError> {
    if modifiers.is_unconstrained {
        return None;
    }

    let attribute = modifiers.attributes.function()?;
    if attribute.kind.is_oracle() {
        let ident = func_meta_name_ident(func, modifiers);
        let location = attribute.location;
        Some(ResolverError::OracleMarkedAsConstrained { ident, location })
    } else {
        None
    }
}

/// Oracle functions cannot return more than 1 slice in their output.
///
/// This is currently a limitation with the AVM: to return multiple slices
/// of unknown length, it would need to support allocating memory for
/// them in the call handler, and return their final address. Currently
/// only the Brillig codegen knows about the Free Memory Pointer, and
/// the VM writes to whatever address is in the destination, so we
/// can only safely deal with one vector.
pub(super) fn oracle_returns_multiple_slices(
    func: &FuncMeta,
    modifiers: &FunctionModifiers,
) -> Option<ResolverError> {
    let attribute = modifiers.attributes.function()?;
    if !attribute.kind.is_oracle() {
        return None;
    }

    fn slice_count(typ: &Type) -> usize {
        match typ {
            Type::Array(_, item) => slice_count(item),
            Type::Slice(typ) => 1 + slice_count(typ),
            Type::FmtString(_, item) => slice_count(item),
            Type::Tuple(items) => items.iter().map(slice_count).sum(),
            Type::DataType(def, args) => {
                let struct_type = def.borrow();
                if let Some(fields) = struct_type.get_fields(args) {
                    fields.iter().map(|(_, typ, _)| slice_count(typ)).sum()
                } else if let Some(variants) = struct_type.get_variants(args) {
                    variants.iter().flat_map(|(_, types)| types).map(slice_count).sum()
                } else {
                    0
                }
            }
            Type::Alias(def, args) => slice_count(&def.borrow().get_type(args)),
            Type::TypeVariable(type_variable)
            | Type::NamedGeneric(NamedGeneric { type_var: type_variable, .. }) => {
                match &*type_variable.borrow() {
                    TypeBinding::Bound(binding) => slice_count(binding),
                    TypeBinding::Unbound(_, _) => 0,
                }
            }
            Type::Forall(_, _)
            | Type::Constant(_, _)
            | Type::Quoted(_)
            | Type::InfixExpr(_, _, _, _)
            | Type::Reference(_, _)
            | Type::Function(_, _, _, _)
            | Type::CheckedCast { .. }
            | Type::TraitAsType(_, _, _)
            | Type::Error
            | Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::String(_)
            | Type::Unit => 0,
        }
    }

    if slice_count(func.return_type()) > 1 {
        let ident = func_meta_name_ident(func, modifiers);
        Some(ResolverError::OracleReturnsMultipleSlices { location: ident.location() })
    } else {
        None
    }
}

/// Oracle functions may not be called by constrained functions directly.
///
/// In order for a constrained function to call an oracle it must first call through an unconstrained function.
pub(super) fn oracle_called_from_constrained_function(
    interner: &NodeInterner,
    called_func: &FuncId,
    calling_from_constrained_runtime: bool,
    location: Location,
) -> Option<ResolverError> {
    if !calling_from_constrained_runtime {
        return None;
    }

    let function_attributes = interner.function_attributes(called_func);
    if function_attributes.function()?.kind.is_oracle() {
        Some(ResolverError::UnconstrainedOracleReturnToConstrained { location })
    } else {
        None
    }
}

/// `pub` is required on return types for entry point functions
pub(super) fn missing_pub(func: &FuncMeta, modifiers: &FunctionModifiers) -> Option<ResolverError> {
    if func.is_entry_point
        && func.return_type() != &Type::Unit
        && func.return_visibility == Visibility::Private
    {
        let ident = func_meta_name_ident(func, modifiers);
        Some(ResolverError::NecessaryPub { ident })
    } else {
        None
    }
}

/// Check that we are not passing a mutable reference from a constrained runtime to an unconstrained runtime.
pub(super) fn unconstrained_function_args(
    function_args: &[(Type, ExprId, Location)],
) -> Vec<TypeCheckError> {
    function_args
        .iter()
        .filter_map(|(typ, _, location)| {
            if !typ.is_valid_for_unconstrained_boundary() {
                Some(TypeCheckError::ConstrainedReferenceToUnconstrained { location: *location })
            } else {
                None
            }
        })
        .collect()
}

/// Check that we are not passing a slice from an unconstrained runtime to a constrained runtime.
pub(super) fn unconstrained_function_return(
    return_type: &Type,
    location: Location,
) -> Option<TypeCheckError> {
    if return_type.contains_slice() {
        Some(TypeCheckError::UnconstrainedSliceReturnToConstrained { location })
    } else if !return_type.is_valid_for_unconstrained_boundary() {
        Some(TypeCheckError::UnconstrainedReferenceToConstrained { location })
    } else {
        None
    }
}

/// Only entrypoint functions require a `pub` visibility modifier applied to their return types.
///
/// Application of `pub` to other functions is not meaningful and is a mistake.
pub(super) fn unnecessary_pub_return(
    func: &FuncMeta,
    modifiers: &FunctionModifiers,
    is_entry_point: bool,
) -> Option<ResolverError> {
    if !is_entry_point && func.return_visibility == Visibility::Public {
        let ident = func_meta_name_ident(func, modifiers);
        Some(ResolverError::UnnecessaryPub { ident, position: PubPosition::ReturnType })
    } else {
        None
    }
}

/// Only arguments to entrypoint functions may have a non-private visibility modifier applied to them.
///
/// Other functions are disallowed from declaring the visibility of their arguments.
pub(super) fn unnecessary_pub_argument(
    func: &NoirFunction,
    arg_visibility: Visibility,
    is_entry_point: bool,
) -> Option<ResolverError> {
    if arg_visibility == Visibility::Public && !is_entry_point {
        Some(ResolverError::UnnecessaryPub {
            ident: func.name_ident().clone(),
            position: PubPosition::Parameter,
        })
    } else {
        None
    }
}

/// call_data and return_data visibility modifiers are only allowed on entry point functions.
pub(super) fn databus_on_non_entry_point(
    func: &NoirFunction,
    visibility: Visibility,
    is_entry_point: bool,
) -> Option<ResolverError> {
    if !is_entry_point {
        match visibility {
            Visibility::CallData(_) | Visibility::ReturnData => {
                Some(ResolverError::DataBusOnNonEntryPoint {
                    ident: func.name_ident().clone(),
                    visibility: visibility.to_string(),
                })
            }
            _ => None,
        }
    } else {
        None
    }
}

/// Checks if an ExprId, which has to be an integer literal, fits in its type.
pub(crate) fn check_integer_literal_fits_its_type(
    interner: &NodeInterner,
    expr_id: &ExprId,
) -> Option<TypeCheckError> {
    let expr = interner.expression(expr_id);
    let typ = interner.id_type(expr_id).follow_bindings();
    let location = interner.expr_location(expr_id);

    match expr {
        HirExpression::Literal(HirLiteral::Integer(value)) => match typ {
            Type::Integer(Signedness::Unsigned, bit_size) => {
                let bit_size: u32 = bit_size.into();
                let max = if bit_size == 128 { u128::MAX } else { 2u128.pow(bit_size) - 1 };
                if value.absolute_value() > max.into() || value.is_negative() {
                    return Some(TypeCheckError::IntegerLiteralDoesNotFitItsType {
                        expr: value,
                        ty: typ.clone(),
                        range: format!("0..={max}"),
                        location,
                    });
                }
            }
            Type::Integer(Signedness::Signed, bit_count) => {
                let bit_count: u32 = bit_count.into();
                let min = 2u128.pow(bit_count - 1);
                let max = 2u128.pow(bit_count - 1) - 1;

                let is_negative = value.is_negative();
                let abs = value.absolute_value();

                if (is_negative && abs > min.into()) || (!is_negative && abs > max.into()) {
                    return Some(TypeCheckError::IntegerLiteralDoesNotFitItsType {
                        expr: value,
                        ty: typ.clone(),
                        range: format!("-{min}..={max}"),
                        location,
                    });
                }
            }
            _ => (),
        },
        _ => panic!("Expected an integer literal"),
    }

    None
}

fn func_meta_name_ident(func: &FuncMeta, modifiers: &FunctionModifiers) -> Ident {
    Ident::new(modifiers.name.clone(), func.name.location)
}

/// Check that a recursive function *can* return without endlessly calling itself.
pub(crate) fn unbounded_recursion<'a>(
    interner: &'a NodeInterner,
    func_id: FuncId,
    func_name: impl FnOnce() -> &'a str,
    func_location: Location,
    body_id: ExprId,
) -> Option<ResolverError> {
    if !can_return_without_recursing(interner, func_id, body_id) {
        Some(ResolverError::UnconditionalRecursion {
            name: func_name().to_string(),
            location: func_location,
        })
    } else {
        None
    }
}

/// Check if an expression will end up calling a specific function.
fn can_return_without_recursing(interner: &NodeInterner, func_id: FuncId, expr_id: ExprId) -> bool {
    let check = |e| can_return_without_recursing(interner, func_id, e);

    let check_block = |block: HirBlockExpression| {
        block.statements.iter().all(|stmt_id| match interner.statement(stmt_id) {
            HirStatement::Let(s) => check(s.expression),
            HirStatement::Assign(s) => check(s.expression),
            HirStatement::Expression(e) => check(e),
            HirStatement::Semi(e) => check(e),
            // Rust doesn't seem to check the for loop body (it's bounds might mean it's never called).
            HirStatement::For(e) => check(e.start_range) && check(e.end_range),
            HirStatement::Loop(e) => check(e),
            HirStatement::While(condition, block) => check(condition) && check(block),
            HirStatement::Comptime(_)
            | HirStatement::Break
            | HirStatement::Continue
            | HirStatement::Error => true,
        })
    };

    match interner.expression(&expr_id) {
        HirExpression::Ident(ident, _) => {
            if ident.id == DefinitionId::dummy_id() {
                return true;
            }
            let definition = interner.definition(ident.id);
            if let DefinitionKind::Function(id) = definition.kind { func_id != id } else { true }
        }
        HirExpression::Block(b) => check_block(b),
        HirExpression::Prefix(e) => check(e.rhs),
        HirExpression::Infix(e) => check(e.lhs) && check(e.rhs),
        HirExpression::Index(e) => check(e.collection) && check(e.index),
        HirExpression::MemberAccess(e) => check(e.lhs),
        HirExpression::Call(e) => check(e.func) && e.arguments.iter().cloned().all(check),
        HirExpression::Constrain(e) => check(e.0) && e.2.map(check).unwrap_or(true),
        HirExpression::Cast(e) => check(e.lhs),
        HirExpression::If(e) => {
            check(e.condition) && (check(e.consequence) || e.alternative.map(check).unwrap_or(true))
        }
        HirExpression::Match(e) => can_return_without_recursing_match(interner, func_id, &e),
        HirExpression::Tuple(e) => e.iter().cloned().all(check),
        HirExpression::Unsafe(b) => check_block(b),
        // Rust doesn't check the lambda body (it might not be called).
        HirExpression::Lambda(_)
        | HirExpression::Literal(_)
        | HirExpression::Constructor(_)
        | HirExpression::EnumConstructor(_)
        | HirExpression::Quote(_)
        | HirExpression::Unquote(_)
        | HirExpression::Error => true,
    }
}

fn can_return_without_recursing_match(
    interner: &NodeInterner,
    func_id: FuncId,
    match_expr: &HirMatch,
) -> bool {
    let check_match = |e| can_return_without_recursing_match(interner, func_id, e);
    let check = |e| can_return_without_recursing(interner, func_id, e);

    match match_expr {
        HirMatch::Success(expr) => check(*expr),
        HirMatch::Failure { .. } => true,
        HirMatch::Guard { cond: _, body, otherwise } => check(*body) && check_match(otherwise),
        HirMatch::Switch(_, cases, otherwise) => {
            cases.iter().all(|case| check_match(&case.body))
                && otherwise.as_ref().is_none_or(|case| check_match(case))
        }
    }
}
