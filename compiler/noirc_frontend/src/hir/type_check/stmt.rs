use acvm::acir::AcirField;
use iter_extended::vecmap;
use noirc_errors::Span;

use crate::ast::UnaryOp;
use crate::hir_def::expr::{HirExpression, HirIdent, HirLiteral};
use crate::hir_def::stmt::{
    HirAssignStatement, HirConstrainStatement, HirForStatement, HirLValue, HirLetStatement,
    HirPattern, HirStatement,
};
use crate::hir_def::types::Type;
use crate::node_interner::{DefinitionId, ExprId, StmtId};

use super::errors::{Source, TypeCheckError};
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
            HirStatement::For(for_loop) => self.check_for_loop(for_loop),
            HirStatement::Comptime(statement) => return self.check_statement(&statement),
            HirStatement::Break | HirStatement::Continue | HirStatement::Error => (),
        }
        Type::Unit
    }

    fn check_for_loop(&mut self, for_loop: HirForStatement) {
        let start_range_type = self.check_expression(&for_loop.start_range);
        let end_range_type = self.check_expression(&for_loop.end_range);

        let start_span = self.interner.expr_span(&for_loop.start_range);
        let end_span = self.interner.expr_span(&for_loop.end_range);

        // Check that start range and end range have the same types
        let range_span = start_span.merge(end_span);
        self.unify(&start_range_type, &end_range_type, || TypeCheckError::TypeMismatch {
            expected_typ: start_range_type.to_string(),
            expr_typ: end_range_type.to_string(),
            expr_span: range_span,
        });

        let expected_type = self.polymorphic_integer();

        self.unify(&start_range_type, &expected_type, || TypeCheckError::TypeCannotBeUsed {
            typ: start_range_type.clone(),
            place: "for loop",
            span: range_span,
        });

        self.interner.push_definition_type(for_loop.identifier.id, start_range_type);

        self.check_expression(&for_loop.block);
    }

    /// Associate a given HirPattern with the given Type, and remember
    /// this association in the NodeInterner.
    pub(crate) fn bind_pattern(&mut self, pattern: &HirPattern, typ: Type) {
        match pattern {
            HirPattern::Identifier(ident) => self.interner.push_definition_type(ident.id, typ),
            HirPattern::Mutable(pattern, _) => self.bind_pattern(pattern, typ),
            HirPattern::Tuple(fields, location) => match typ.follow_bindings() {
                Type::Tuple(field_types) if field_types.len() == fields.len() => {
                    for (field, field_type) in fields.iter().zip(field_types) {
                        self.bind_pattern(field, field_type);
                    }
                }
                Type::Error => (),
                other => {
                    let expected =
                        Type::Tuple(vecmap(fields, |_| self.interner.next_type_variable()));

                    self.errors.push(TypeCheckError::TypeMismatchWithSource {
                        expected,
                        actual: other,
                        span: location.span,
                        source: Source::Assignment,
                    });
                }
            },
            HirPattern::Struct(struct_type, fields, location) => {
                self.unify(struct_type, &typ, || TypeCheckError::TypeMismatchWithSource {
                    expected: struct_type.clone(),
                    actual: typ.clone(),
                    span: location.span,
                    source: Source::Assignment,
                });

                if let Type::Struct(struct_type, generics) = struct_type.follow_bindings() {
                    let struct_type = struct_type.borrow();

                    for (field_name, field_pattern) in fields {
                        if let Some((type_field, _)) =
                            struct_type.get_field(&field_name.0.contents, &generics)
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
        let (lvalue_type, new_lvalue, mutable) = self.check_lvalue(&assign_stmt.lvalue, span);

        if !mutable {
            let (name, span) = self.get_lvalue_name_and_span(&assign_stmt.lvalue);
            self.errors.push(TypeCheckError::VariableMustBeMutable { name, span });
        }

        // Must push new lvalue to the interner, we've resolved any field indices
        self.interner.update_statement(stmt_id, |stmt| match stmt {
            HirStatement::Assign(assign) => assign.lvalue = new_lvalue,
            _ => unreachable!("statement is known to be assignment"),
        });

        let span = self.interner.expr_span(&assign_stmt.expression);
        self.unify_with_coercions(&expr_type, &lvalue_type, assign_stmt.expression, || {
            TypeCheckError::TypeMismatchWithSource {
                actual: expr_type.clone(),
                expected: lvalue_type.clone(),
                span,
                source: Source::Assignment,
            }
        });
    }

    fn get_lvalue_name_and_span(&self, lvalue: &HirLValue) -> (String, Span) {
        match lvalue {
            HirLValue::Ident(name, _) => {
                let span = name.location.span;

                if let Some(definition) = self.interner.try_definition(name.id) {
                    (definition.name.clone(), span)
                } else {
                    ("(undeclared variable)".into(), span)
                }
            }
            HirLValue::MemberAccess { object, .. } => self.get_lvalue_name_and_span(object),
            HirLValue::Index { array, .. } => self.get_lvalue_name_and_span(array),
            HirLValue::Dereference { lvalue, .. } => self.get_lvalue_name_and_span(lvalue),
        }
    }

    /// Type check an lvalue - the left hand side of an assignment statement.
    fn check_lvalue(&mut self, lvalue: &HirLValue, assign_span: Span) -> (Type, HirLValue, bool) {
        match lvalue {
            HirLValue::Ident(ident, _) => {
                let mut mutable = true;

                let typ = if ident.id == DefinitionId::dummy_id() {
                    Type::Error
                } else {
                    if let Some(definition) = self.interner.try_definition(ident.id) {
                        mutable = definition.mutable;
                    }

                    let typ = self.interner.definition_type(ident.id).instantiate(self.interner).0;
                    typ.follow_bindings()
                };

                (typ.clone(), HirLValue::Ident(ident.clone(), typ), mutable)
            }
            HirLValue::MemberAccess { object, field_name, location, .. } => {
                let (lhs_type, object, mut mutable) = self.check_lvalue(object, assign_span);
                let mut object = Box::new(object);
                let field_name = field_name.clone();

                let object_ref = &mut object;
                let mutable_ref = &mut mutable;
                let location = *location;

                let dereference_lhs = move |_: &mut Self, _, element_type| {
                    // We must create a temporary value first to move out of object_ref before
                    // we eventually reassign to it.
                    let id = DefinitionId::dummy_id();
                    let ident = HirIdent::non_trait_method(id, location);
                    let tmp_value = HirLValue::Ident(ident, Type::Error);

                    let lvalue = std::mem::replace(object_ref, Box::new(tmp_value));
                    *object_ref =
                        Box::new(HirLValue::Dereference { lvalue, element_type, location });
                    *mutable_ref = true;
                };

                let name = &field_name.0.contents;
                let (object_type, field_index) = self
                    .check_field_access(&lhs_type, name, field_name.span(), Some(dereference_lhs))
                    .unwrap_or((Type::Error, 0));

                let field_index = Some(field_index);
                let typ = object_type.clone();
                let lvalue =
                    HirLValue::MemberAccess { object, field_name, field_index, typ, location };
                (object_type, lvalue, mutable)
            }
            HirLValue::Index { array, index, location, .. } => {
                let index_type = self.check_expression(index);
                let expr_span = self.interner.expr_span(index);
                let location = *location;

                index_type.unify(&self.polymorphic_integer_or_field(), &mut self.errors, || {
                    TypeCheckError::TypeMismatch {
                        expected_typ: "an integer".to_owned(),
                        expr_typ: index_type.to_string(),
                        expr_span,
                    }
                });

                let (mut lvalue_type, mut lvalue, mut mutable) =
                    self.check_lvalue(array, assign_span);

                // Before we check that the lvalue is an array, try to dereference it as many times
                // as needed to unwrap any &mut wrappers.
                while let Type::MutableReference(element) = lvalue_type.follow_bindings() {
                    let element_type = element.as_ref().clone();
                    lvalue =
                        HirLValue::Dereference { lvalue: Box::new(lvalue), element_type, location };
                    lvalue_type = *element;
                    // We know this value to be mutable now since we found an `&mut`
                    mutable = true;
                }

                let typ = match lvalue_type.follow_bindings() {
                    Type::Array(_, elem_type) => *elem_type,
                    Type::Slice(elem_type) => *elem_type,
                    Type::Error => Type::Error,
                    Type::String(_) => {
                        let (_lvalue_name, lvalue_span) = self.get_lvalue_name_and_span(&lvalue);
                        self.errors.push(TypeCheckError::StringIndexAssign { span: lvalue_span });
                        Type::Error
                    }
                    other => {
                        // TODO: Need a better span here
                        self.errors.push(TypeCheckError::TypeMismatch {
                            expected_typ: "array".to_string(),
                            expr_typ: other.to_string(),
                            expr_span: assign_span,
                        });
                        Type::Error
                    }
                };

                let array = Box::new(lvalue);
                (typ.clone(), HirLValue::Index { array, index: *index, typ, location }, mutable)
            }
            HirLValue::Dereference { lvalue, element_type: _, location } => {
                let (reference_type, lvalue, _) = self.check_lvalue(lvalue, assign_span);
                let lvalue = Box::new(lvalue);
                let location = *location;

                let element_type = Type::type_variable(self.interner.next_type_variable_id());
                let expected_type = Type::MutableReference(Box::new(element_type.clone()));

                self.unify(&reference_type, &expected_type, || TypeCheckError::TypeMismatch {
                    expected_typ: expected_type.to_string(),
                    expr_typ: reference_type.to_string(),
                    expr_span: assign_span,
                });

                // Dereferences are always mutable since we already type checked against a &mut T
                (
                    element_type.clone(),
                    HirLValue::Dereference { lvalue, element_type, location },
                    true,
                )
            }
        }
    }

    fn check_let_stmt(&mut self, let_stmt: HirLetStatement) {
        let resolved_type = self.check_declaration(let_stmt.expression, let_stmt.r#type);

        // Set the type of the pattern to be equal to the annotated type
        self.bind_pattern(&let_stmt.pattern, resolved_type);
    }

    fn check_constrain_stmt(&mut self, stmt: HirConstrainStatement) {
        let expr_type = self.check_expression(&stmt.0);
        let expr_span = self.interner.expr_span(&stmt.0);

        // Must type check the assertion message expression so that we instantiate bindings
        stmt.2.map(|assert_msg_expr| self.check_expression(&assert_msg_expr));

        self.unify(&expr_type, &Type::Bool, || TypeCheckError::TypeMismatch {
            expr_typ: expr_type.to_string(),
            expected_typ: Type::Bool.to_string(),
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

            self.unify_with_coercions(&expr_type, &annotated_type, rhs_expr, || {
                TypeCheckError::TypeMismatch {
                    expected_typ: annotated_type.to_string(),
                    expr_typ: expr_type.to_string(),
                    expr_span,
                }
            });
            if annotated_type.is_unsigned() {
                self.lint_overflowing_uint(&rhs_expr, &annotated_type);
            }
            annotated_type
        } else {
            expr_type
        }
    }

    /// Check if an assignment is overflowing with respect to `annotated_type`
    /// in a declaration statement where `annotated_type` is an unsigned integer
    fn lint_overflowing_uint(&mut self, rhs_expr: &ExprId, annotated_type: &Type) {
        let expr = self.interner.expression(rhs_expr);
        let span = self.interner.expr_span(rhs_expr);
        match expr {
            HirExpression::Literal(HirLiteral::Integer(value, false)) => {
                let v = value.to_u128();
                if let Type::Integer(_, bit_count) = annotated_type {
                    let bit_count: u32 = (*bit_count).into();
                    let max = 1 << bit_count;
                    if v >= max {
                        self.errors.push(TypeCheckError::OverflowingAssignment {
                            expr: value,
                            ty: annotated_type.clone(),
                            range: format!("0..={}", max - 1),
                            span,
                        });
                    };
                };
            }
            HirExpression::Prefix(expr) => {
                self.lint_overflowing_uint(&expr.rhs, annotated_type);
                if matches!(expr.operator, UnaryOp::Minus) {
                    self.errors.push(TypeCheckError::InvalidUnaryOp {
                        kind: "annotated_type".to_string(),
                        span,
                    });
                }
            }
            HirExpression::Infix(expr) => {
                self.lint_overflowing_uint(&expr.lhs, annotated_type);
                self.lint_overflowing_uint(&expr.rhs, annotated_type);
            }
            _ => {}
        }
    }
}
