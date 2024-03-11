use iter_extended::vecmap;
use noirc_errors::Span;
use noirc_frontend::{
    graph::CrateId,
    macros_api::{
        BlockExpression, FileId, HirContext, HirExpression, HirLiteral, HirStatement, NodeInterner,
        NoirStruct, PathKind, StatementKind, StructId, StructType, Type, TypeImpl,
        UnresolvedTypeData,
    },
    token::SecondaryAttribute,
    ExpressionKind, FunctionDefinition, FunctionReturnType, FunctionVisibility, Literal,
    NoirFunction, Visibility,
};

use crate::{
    chained_dep,
    utils::{
        ast_utils::{
            call, expression, ident, ident_path, make_statement, make_type, path, variable_path,
        },
        constants::SIGNATURE_PLACEHOLDER,
        errors::AztecMacroError,
        hir_utils::{collect_crate_structs, signature_of_type},
    },
};

/// Generates the impl for an event selector
///
/// Inserts the following code:
/// ```noir
/// impl SomeStruct {
///    fn selector() -> FunctionSelector {
///       aztec::protocol_types::abis::function_selector::FunctionSelector::from_signature("SIGNATURE_PLACEHOLDER")
///    }
/// }
/// ```
///
/// This allows developers to emit events without having to write the signature of the event every time they emit it.
/// The signature cannot be known at this point since types are not resolved yet, so we use a signature placeholder.
/// It'll get resolved after by transforming the HIR.
pub fn generate_selector_impl(structure: &NoirStruct) -> TypeImpl {
    let struct_type =
        make_type(UnresolvedTypeData::Named(path(structure.name.clone()), vec![], true));

    let selector_path =
        chained_dep!("aztec", "protocol_types", "abis", "function_selector", "FunctionSelector");
    let mut from_signature_path = selector_path.clone();
    from_signature_path.segments.push(ident("from_signature"));

    let selector_fun_body = BlockExpression(vec![make_statement(StatementKind::Expression(call(
        variable_path(from_signature_path),
        vec![expression(ExpressionKind::Literal(Literal::Str(SIGNATURE_PLACEHOLDER.to_string())))],
    )))]);

    // Define `FunctionSelector` return type
    let return_type =
        FunctionReturnType::Ty(make_type(UnresolvedTypeData::Named(selector_path, vec![], true)));

    let mut selector_fn_def = FunctionDefinition::normal(
        &ident("selector"),
        &vec![],
        &[],
        &selector_fun_body,
        &[],
        &return_type,
    );

    selector_fn_def.visibility = FunctionVisibility::Public;

    // Seems to be necessary on contract modules
    selector_fn_def.return_visibility = Visibility::Public;

    TypeImpl {
        object_type: struct_type,
        type_span: structure.span,
        generics: vec![],
        methods: vec![(NoirFunction::normal(selector_fn_def), Span::default())],
    }
}

/// Computes the signature for a resolved event type.
/// It has the form 'EventName(Field,(Field),[u8;2])'
fn event_signature(event: &StructType) -> String {
    let fields = vecmap(event.get_fields(&[]), |(_, typ)| signature_of_type(&typ));
    format!("{}({})", event.name.0.contents, fields.join(","))
}

/// Substitutes the signature literal that was introduced in the selector method previously with the actual signature.
fn transform_event(
    struct_id: StructId,
    interner: &mut NodeInterner,
) -> Result<(), (AztecMacroError, FileId)> {
    let struct_type = interner.get_struct(struct_id);
    let selector_id = interner
        .lookup_method(&Type::Struct(struct_type.clone(), vec![]), struct_id, "selector", false)
        .ok_or_else(|| {
            let error = AztecMacroError::EventError {
                span: struct_type.borrow().location.span,
                message: "Selector method not found".to_owned(),
            };
            (error, struct_type.borrow().location.file)
        })?;
    let selector_function = interner.function(&selector_id);

    let compute_selector_statement = interner.statement(
        selector_function.block(interner).statements().first().ok_or_else(|| {
            let error = AztecMacroError::EventError {
                span: struct_type.borrow().location.span,
                message: "Compute selector statement not found".to_owned(),
            };
            (error, struct_type.borrow().location.file)
        })?,
    );

    let compute_selector_expression = match compute_selector_statement {
        HirStatement::Expression(expression_id) => match interner.expression(&expression_id) {
            HirExpression::Call(hir_call_expression) => Some(hir_call_expression),
            _ => None,
        },
        _ => None,
    }
    .ok_or_else(|| {
        let error = AztecMacroError::EventError {
            span: struct_type.borrow().location.span,
            message: "Compute selector statement is not a call expression".to_owned(),
        };
        (error, struct_type.borrow().location.file)
    })?;

    let first_arg_id = compute_selector_expression.arguments.first().ok_or_else(|| {
        let error = AztecMacroError::EventError {
            span: struct_type.borrow().location.span,
            message: "Compute selector statement is not a call expression".to_owned(),
        };
        (error, struct_type.borrow().location.file)
    })?;

    match interner.expression(first_arg_id) {
        HirExpression::Literal(HirLiteral::Str(signature))
            if signature == SIGNATURE_PLACEHOLDER =>
        {
            let selector_literal_id = *first_arg_id;

            let structure = interner.get_struct(struct_id);
            let signature = event_signature(&structure.borrow());
            interner.update_expression(selector_literal_id, |expr| {
                *expr = HirExpression::Literal(HirLiteral::Str(signature.clone()));
            });

            // Also update the type! It might have a different length now than the placeholder.
            interner.push_expr_type(
                selector_literal_id,
                Type::String(Box::new(Type::Constant(signature.len() as u64))),
            );
            Ok(())
        }
        _ => Err((
            AztecMacroError::EventError {
                span: struct_type.borrow().location.span,
                message: "Signature placeholder literal does not match".to_owned(),
            },
            struct_type.borrow().location.file,
        )),
    }
}

pub fn transform_events(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    for struct_id in collect_crate_structs(crate_id, context) {
        let attributes = context.def_interner.struct_attributes(&struct_id);
        if attributes.iter().any(|attr| matches!(attr, SecondaryAttribute::Event)) {
            transform_event(struct_id, &mut context.def_interner)?;
        }
    }
    Ok(())
}
