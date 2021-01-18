use crate::{Type, hir::lower::{def_interner::{DefInterner, ExprId, StmtId}, stmt::{HirConstStatement, HirConstrainStatement, HirLetStatement, HirPrivateStatement, HirStatement}}};

use super::expr::type_check_expression;

pub(crate) fn type_check(interner : &mut DefInterner, stmt_id : StmtId) {

    match interner.statement(stmt_id) {
        HirStatement::Expression(expr_id) => type_check_expression(interner, expr_id),
        HirStatement::Private(priv_stmt) => type_check_priv_stmt(interner, priv_stmt),  
        HirStatement::Let(let_stmt) => type_check_let_stmt(interner, let_stmt),
        HirStatement::Const(const_stmt) => type_check_const_stmt(interner, const_stmt),
        HirStatement::Constrain(constrain_stmt) => type_check_constrain_stmt(interner, constrain_stmt),
        HirStatement::Block(block_stmt) => {
            for stmt in block_stmt.statements()  {
                type_check(interner, stmt)
            }
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

fn type_check_priv_stmt(interner : &mut DefInterner, priv_stmt : HirPrivateStatement) {
    let resolved_type = type_check_declaration(interner, priv_stmt.expression, priv_stmt.r#type);

    // Check if this type can be used in a Private statement
    if !resolved_type.can_be_used_in_priv() {
        panic!("the type {} cannot be used in a Private Statement", &resolved_type);
    }

    // Set the type of the identifier to be equal to the annotated type
    interner.push_ident_type(priv_stmt.identifier, resolved_type);
}

fn type_check_let_stmt(interner : &mut DefInterner, let_stmt : HirLetStatement) {
    let resolved_type = type_check_declaration(interner, let_stmt.expression, let_stmt.r#type);

    // Check if this type can be used in a Let statement
    if !resolved_type.can_be_used_in_let() {
        panic!("the type {} cannot be used in a Let Statement", &resolved_type)
    }

    // Set the type of the identifier to be equal to the annotated type
    interner.push_ident_type(let_stmt.identifier, resolved_type);
}
fn type_check_const_stmt(interner : &mut DefInterner, const_stmt : HirConstStatement) {
    
    // XXX: It may not make sense to have annotations for const statements, since they can only have one type
    // Unless we later want to have u32 constants and check those at compile time.
    let resolved_type = type_check_declaration(interner, const_stmt.expression, const_stmt.r#type);
    
    if resolved_type != Type::Constant {
        panic!("constant statements can only contain constant types, found type {}", &resolved_type);
    }
    
    interner.push_ident_type(const_stmt.identifier, resolved_type);
}
fn type_check_constrain_stmt(interner : &mut DefInterner, stmt : HirConstrainStatement) {
    
    type_check_expression(interner, stmt.0.lhs);
    let lhs_type = interner.id_type(stmt.0.lhs.into());
    
    type_check_expression(interner,stmt.0.rhs);
    let rhs_type = interner.id_type(stmt.0.rhs.into());

    // Since constrain statements are not expressions, we do not allow predicate or non-comparison binary operators
    if !stmt.0.operator.is_comparator()  {
        panic!("only comparison operators can be used in a constrain statement");
    };

    if !lhs_type.can_be_used_in_constrain() {
        panic!("found type {} . This type cannot be used in a constrain statement", lhs_type);
    }
    if !rhs_type.can_be_used_in_constrain() {
        panic!("found type {} . This type cannot be used in a constrain statement", rhs_type);
    }
}

/// All declaration statements check that the user specified type(UST) is equal to the 
/// expression on the RHS, unless the UST is unspecified
/// In that case, the UST because the expression
fn type_check_declaration(interner : &mut DefInterner, rhs_expr : ExprId, mut annotated_type : Type) -> Type {
   
    // Type check the expression on the RHS
    type_check_expression(interner,rhs_expr);
    let expr_type = interner.id_type(rhs_expr.into());

    // First check if the LHS is unspecified
    // If so, then we give it the same type as the expression
    if annotated_type == Type::Unspecified {
        annotated_type = expr_type.clone();
    };

    // Now check if LHS is the same type as the RHS
    // Importantly, we do not co-erce any types implicitly
    if annotated_type != expr_type {
        panic!("type mismatch. annotated type is {} but expr type is {}", annotated_type, expr_type);
    }
    
    // At this point annotated type and user specified type are the same
    // so we can return either. Cloning a Type is Cheap and may eventually be Copy
    expr_type
}