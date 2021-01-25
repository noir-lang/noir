use noirc_errors::Span;

use crate::{Type, hir::lower::{node_interner::{NodeInterner, ExprId, StmtId}, stmt::{HirConstStatement, HirConstrainStatement, HirLetStatement, HirPrivateStatement, HirStatement}}};

use super::{errors::TypeCheckError, expr::type_check_expression};

pub(crate) fn type_check(interner : &mut NodeInterner, stmt_id : StmtId) -> Result<(), TypeCheckError>{

    match interner.statement(stmt_id) {
        HirStatement::Expression(expr_id) => type_check_expression(interner, expr_id),
        HirStatement::Private(priv_stmt) => type_check_priv_stmt(interner, priv_stmt),  
        HirStatement::Let(let_stmt) => type_check_let_stmt(interner, let_stmt),
        HirStatement::Const(const_stmt) => type_check_const_stmt(interner, const_stmt),
        HirStatement::Constrain(constrain_stmt) => type_check_constrain_stmt(interner, constrain_stmt),
        HirStatement::Block(block_stmt) => {
            for stmt in block_stmt.statements()  {
                type_check(interner, stmt)?
            }
            Ok(())
        }
        HirStatement::Public(_) => {
            // XXX: Initially, we were going to give the ability to declare public variables inside of functions.
            // Now it seems more plausible to only have Public variables be declared as function types,
            // So that we can keep track of linear transformations between public variables which may leak a witness
            //
            // although it is syntax sugaring, it allows users to keep track of public variables, we don't necessarily want them 
            // to be limited to this in the main function parameters
            panic!("[Deprecated] : Declaring public variables in block statements is being deprecated. You will still be able to state them as Types in function parameters ")
        }
    }
}

fn type_check_priv_stmt(interner : &mut NodeInterner, priv_stmt : HirPrivateStatement) -> Result<(), TypeCheckError>{
    let resolved_type = type_check_declaration(interner, priv_stmt.expression, priv_stmt.r#type)?;

    // Check if this type can be used in a Private statement
    if !resolved_type.can_be_used_in_priv() {
        let span = interner.expr_span(priv_stmt.expression);
        return Err(TypeCheckError::TypeCannotBeUsed{ typ: resolved_type, place: "private statement", span});
    }

    // Set the type of the identifier to be equal to the annotated type
    interner.push_ident_type(priv_stmt.identifier, resolved_type);

    Ok(())
}

fn type_check_let_stmt(interner : &mut NodeInterner, let_stmt : HirLetStatement) -> Result<(), TypeCheckError>{
    let resolved_type = type_check_declaration(interner, let_stmt.expression, let_stmt.r#type)?;

    // Check if this type can be used in a Let statement
    if !resolved_type.can_be_used_in_let() {
        let span = interner.expr_span(let_stmt.expression.into());
        return Err(TypeCheckError::TypeCannotBeUsed{typ : resolved_type.clone(), place : "let statement", span : span});
    }
    
    // Set the type of the identifier to be equal to the annotated type
    interner.push_ident_type(let_stmt.identifier, resolved_type);
    
    Ok(())
}
fn type_check_const_stmt(interner : &mut NodeInterner, const_stmt : HirConstStatement)-> Result<(), TypeCheckError> {
    
    // XXX: It may not make sense to have annotations for const statements, since they can only have one type
    // Unless we later want to have u32 constants and check those at compile time.
    let resolved_type = type_check_declaration(interner, const_stmt.expression, const_stmt.r#type)?;
    
    if resolved_type != Type::Constant {
        let span = interner.expr_span(const_stmt.expression.into());
        let mut err = TypeCheckError::TypeCannotBeUsed{typ: resolved_type, place: "constant statement", span};
        err = err.add_context("constant statements can only contain constant types").unwrap();
        return Err(err)
    }
    
    interner.push_ident_type(const_stmt.identifier, resolved_type);

    Ok(())
}
fn type_check_constrain_stmt(interner : &mut NodeInterner, stmt : HirConstrainStatement)-> Result<(), TypeCheckError> {
    
    type_check_expression(interner, stmt.0.lhs)?;
    let lhs_type = interner.id_type(stmt.0.lhs.into());
    
    type_check_expression(interner,stmt.0.rhs)?;
    let rhs_type = interner.id_type(stmt.0.rhs.into());

    // Since constrain statements are not expressions, we do not allow predicate or non-comparison binary operators
    if !stmt.0.operator.kind.is_comparator()  {
        let span = stmt.0.operator.span;
        let err = TypeCheckError::OpCannotBeUsed{ op: stmt.0.operator, place: "constrain statement", span};
        let err = err.add_context("only comparison operators can be used in a constrain statement").unwrap();
        return Err(err);
    };

    if !lhs_type.can_be_used_in_constrain() {
        let span = interner.expr_span(stmt.0.lhs.into());
        return Err(TypeCheckError::TypeCannotBeUsed{typ: lhs_type, place: "constrain statement", span});
    }
    if !rhs_type.can_be_used_in_constrain() {
        let span = interner.expr_span(stmt.0.rhs.into());
        return Err(TypeCheckError::TypeCannotBeUsed{typ: rhs_type, place: "constrain statement", span});
    }

    Ok(())
}

/// All declaration statements check that the user specified type(UST) is equal to the 
/// expression on the RHS, unless the UST is unspecified
/// In that case, the UST because the expression
fn type_check_declaration(interner : &mut NodeInterner, rhs_expr : ExprId, mut annotated_type : Type) -> Result<Type, TypeCheckError> {
   
    // Type check the expression on the RHS
    type_check_expression(interner,rhs_expr)?;
    let expr_type = interner.id_type(rhs_expr.into());

    // First check if the LHS is unspecified
    // If so, then we give it the same type as the expression
    if annotated_type == Type::Unspecified {
        annotated_type = expr_type.clone();
    };

    // Now check if LHS is the same type as the RHS
    // Importantly, we do not co-erce any types implicitly
    if annotated_type != expr_type {
        // XXX: Types are not yet spanned!
        let expr_span = interner.expr_span(rhs_expr);
        return Err(TypeCheckError::TypeAnnotationMismatch{ ann_typ: annotated_type, ann_span: Span::default(), expr_typ: expr_type, expr_span});
    }
    
    // At this point annotated type and user specified type are the same
    // so we can return either. Cloning a Type is Cheap and may eventually be Copy
    Ok(expr_type)
}