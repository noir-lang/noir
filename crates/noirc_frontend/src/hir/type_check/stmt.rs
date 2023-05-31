use noirc_errors::Span;

use crate::hir_def::stmt::{
    HirAssignStatement, HirConstrainStatement, HirLValue, HirLetStatement, HirPattern, HirStatement,
};
use crate::hir_def::types::Type;
use crate::node_interner::{DefinitionId, ExprId, StmtId};
use crate::CompTime;

use super::errors::TypeCheckError;
use super::TypeChecker;

impl<'interner> TypeChecker<'interner> {
    /// Type checks a statement and all expressions/statements contained within.
    ///
    /// All statements have a unit type `()` as their type so the type of the statement
    /// is not interesting. Type checking must still be done on statements to ensure any
    /// expressions used within them are typed correctly.
    pub(crate) fn check_statement(&mut self, stmt_id: &StmtId) -> Type {
        match self.interner.statement(stmt_id) {
            // Lets lay out a convincing argument that the handling of
            // SemiExpressions and Expressions below is correct.
            //
            // The only time you will get a Semi expression is if
            // you have an expression by itself
            //
            // Example:
            //
            // 5; or x; or x+a;
            //
            // In these cases, you cannot even get the expr_id because
            // it is not bound to anything. We could therefore.
            //
            // However since TypeChecking checks the return type of the last statement
            // the type checker could in the future incorrectly return the type.
            //
            // As it stands, this is also impossible because the ret_type function
            // does not use the interner to get the type. It returns Unit.
            //
            // The reason why we still modify the database, is to make sure it is future-proof
            HirStatement::Expression(expr_id) => {
                return self.check_expression(&expr_id);
            }
            HirStatement::Semi(expr_id) => {
                self.check_expression(&expr_id);
            }
            HirStatement::Let(let_stmt) => self.check_let_stmt(let_stmt),
            HirStatement::Constrain(constrain_stmt) => self.check_constrain_stmt(constrain_stmt),
            HirStatement::Assign(assign_stmt) => self.check_assign_stmt(assign_stmt, stmt_id),
            HirStatement::Error => (),
        }
        Type::Unit
    }

    /// Associate a given HirPattern with the given Type, and remember
    /// this association in the NodeInterner.
    pub(crate) fn bind_pattern(&mut self, pattern: &HirPattern, typ: Type) {
        match pattern {
            HirPattern::Identifier(ident) => self.interner.push_definition_type(ident.id, typ),
            HirPattern::Mutable(pattern, _) => self.bind_pattern(pattern, typ),
            HirPattern::Tuple(fields, span) => match typ {
                Type::Tuple(field_types) if field_types.len() == fields.len() => {
                    for (field, field_type) in fields.iter().zip(field_types) {
                        self.bind_pattern(field, field_type);
                    }
                }
                Type::Error => (),
                other => {
                    self.errors.push(TypeCheckError::TypeMismatch {
                        expected_typ: other.to_string(),
                        expr_typ: other.to_string(),
                        expr_span: *span,
                    });
                }
            },
            HirPattern::Struct(struct_type, fields, span) => {
                self.unify(struct_type, &typ, *span, || TypeCheckError::TypeMismatch {
                    expected_typ: typ.to_string(),
                    expr_typ: struct_type.to_string(),
                    expr_span: *span,
                });

                if let Type::Struct(struct_type, generics) = struct_type {
                    let struct_type = struct_type.borrow();

                    for (field_name, field_pattern) in fields {
                        if let Some((type_field, _)) =
                            struct_type.get_field(&field_name.0.contents, generics)
                        {
                            self.bind_pattern(field_pattern, type_field);
                        }
                    }
                }
            }
        }
    }

    fn check_assign_stmt(&mut self, assign_stmt: HirAssignStatement, stmt_id: &StmtId) {
        let expr_type = self.check_expression(&assign_stmt.expression);
        let span = self.interner.expr_span(&assign_stmt.expression);
        let (lvalue_type, new_lvalue) = self.check_lvalue(assign_stmt.lvalue, span);

        // Must push new lvalue to the interner, we've resolved any field indices
        self.interner.update_statement(stmt_id, |stmt| match stmt {
            HirStatement::Assign(assign) => assign.lvalue = new_lvalue,
            _ => unreachable!(),
        });

        let span = self.interner.expr_span(&assign_stmt.expression);
        self.make_subtype_of(&expr_type, &lvalue_type, span, || {
            let msg = format!(
                "Cannot assign an expression of type {expr_type} to a value of type {lvalue_type}"
            );

            TypeCheckError::Unstructured { msg, span }
        });
    }

    /// Type check an lvalue - the left hand side of an assignment statement.
    fn check_lvalue(&mut self, lvalue: HirLValue, assign_span: Span) -> (Type, HirLValue) {
        match lvalue {
            HirLValue::Ident(ident, _) => {
                let typ = if ident.id == DefinitionId::dummy_id() {
                    Type::Error
                } else {
                    let definition = self.interner.definition(ident.id);
                    if !definition.mutable {
                        self.errors.push(TypeCheckError::Unstructured {
                            msg: format!(
                                "Variable {} must be mutable to be assigned to",
                                definition.name
                            ),
                            span: ident.location.span,
                        });
                    }
                    // Do we need to store TypeBindings here?
                    self.interner.id_type(ident.id).instantiate(self.interner).0
                };

                (typ.clone(), HirLValue::Ident(ident, typ))
            }
            HirLValue::MemberAccess { object, field_name, .. } => {
                let (lhs_type, object) = self.check_lvalue(*object, assign_span);
                let object = Box::new(object);

                let span = field_name.span();
                let (typ, field_index) = self
                    .check_field_access(&lhs_type, &field_name.0.contents, span)
                    .unwrap_or((Type::Error, 0));

                let field_index = Some(field_index);
                (typ.clone(), HirLValue::MemberAccess { object, field_name, field_index, typ })
            }
            HirLValue::Index { array, index, .. } => {
                let index_type = self.check_expression(&index);
                let expr_span = self.interner.expr_span(&index);

                index_type.make_subtype_of(
                    &Type::field(Some(expr_span)),
                    expr_span,
                    &mut self.errors,
                    || TypeCheckError::TypeMismatch {
                        expected_typ: "Field".to_owned(),
                        expr_typ: index_type.to_string(),
                        expr_span,
                    },
                );

                let (result, array) = self.check_lvalue(*array, assign_span);
                let array = Box::new(array);

                let typ = match result {
                    Type::Array(_, elem_type) => *elem_type,
                    Type::Error => Type::Error,
                    other => {
                        // TODO: Need a better span here
                        self.errors.push(TypeCheckError::TypeMismatch {
                            expected_typ: "an array".to_string(),
                            expr_typ: other.to_string(),
                            expr_span: assign_span,
                        });
                        Type::Error
                    }
                };

                (typ.clone(), HirLValue::Index { array, index, typ })
            }
        }
    }

    fn check_let_stmt(&mut self, let_stmt: HirLetStatement) {
        let mut resolved_type = self.check_declaration(let_stmt.expression, let_stmt.r#type);

        resolved_type.set_comp_time_span(self.interner.expr_span(&let_stmt.expression));

        // Set the type of the pattern to be equal to the annotated type
        self.bind_pattern(&let_stmt.pattern, resolved_type);
    }

    fn check_constrain_stmt(&mut self, stmt: HirConstrainStatement) {
        let expr_type = self.check_expression(&stmt.0);
        let expr_span = self.interner.expr_span(&stmt.0);

        let bool_type = Type::Bool(CompTime::new(self.interner));
        self.unify(&expr_type, &bool_type, expr_span, || TypeCheckError::TypeMismatch {
            expr_typ: expr_type.to_string(),
            expected_typ: Type::Bool(CompTime::No(None)).to_string(),
            expr_span,
        });
    }

    /// All declaration statements check that the user specified type(UST) is equal to the
    /// expression on the RHS, unless the UST is unspecified in which case
    /// the type of the declaration is inferred to match the RHS.
    fn check_declaration(&mut self, rhs_expr: ExprId, annotated_type: Type) -> Type {
        // Type check the expression on the RHS
        let expr_type = self.check_expression(&rhs_expr);

        // First check if the LHS is unspecified
        // If so, then we give it the same type as the expression
        if annotated_type != Type::Error {
            // Now check if LHS is the same type as the RHS
            // Importantly, we do not coerce any types implicitly
            let expr_span = self.interner.expr_span(&rhs_expr);
            self.make_subtype_of(&expr_type, &annotated_type, expr_span, || {
                TypeCheckError::TypeMismatch {
                    expected_typ: annotated_type.to_string(),
                    expr_typ: expr_type.to_string(),
                    expr_span,
                }
            });
            annotated_type
        } else {
            expr_type
        }
    }
}
