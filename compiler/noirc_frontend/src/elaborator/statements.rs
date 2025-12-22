//! Statement elaboration including let bindings, assignments, and control flow.

use noirc_errors::Location;

use crate::{
    Type,
    ast::{
        AssignStatement, ForLoopStatement, ForRange, IntegerBitSize, LValue, LetStatement,
        LoopStatement, Statement, StatementKind, WhileStatement,
    },
    elaborator::{PathResolutionTarget, WildcardDisallowedContext, types::WildcardAllowed},
    hir::{
        def_collector::dc_crate::CompilationError,
        resolution::errors::ResolverError,
        type_check::{Source, TypeCheckError},
    },
    hir_def::{
        expr::{HirBlockExpression, HirExpression, HirIdent},
        stmt::{
            HirAssignStatement, HirForStatement, HirLValue, HirLetStatement, HirPattern,
            HirStatement,
        },
    },
    node_interner::{DefinitionId, DefinitionKind, ExprId, GlobalId, StmtId},
    shared::Signedness,
};

use super::{Elaborator, Loop};

impl Elaborator<'_> {
    fn elaborate_statement_value(&mut self, statement: Statement) -> (HirStatement, Type) {
        self.elaborate_statement_value_with_target_type(statement, None)
    }

    fn elaborate_statement_value_with_target_type(
        &mut self,
        statement: Statement,
        target_type: Option<&Type>,
    ) -> (HirStatement, Type) {
        match statement.kind {
            StatementKind::Let(let_stmt) => self.elaborate_local_let(let_stmt),
            StatementKind::Assign(assign) => self.elaborate_assign(assign),
            StatementKind::For(for_stmt) => self.elaborate_for(for_stmt),
            StatementKind::Loop(loop_) => self.elaborate_loop(loop_),
            StatementKind::While(while_) => self.elaborate_while(while_),
            StatementKind::Break => self.elaborate_jump(true, statement.location),
            StatementKind::Continue => self.elaborate_jump(false, statement.location),
            StatementKind::Comptime(statement) => {
                self.elaborate_comptime_statement(*statement, target_type)
            }
            StatementKind::Expression(expr) => {
                let (expr, typ) = self.elaborate_expression_with_target_type(expr, target_type);
                (HirStatement::Expression(expr), typ)
            }
            StatementKind::Semi(expr) => {
                let (expr, _typ) = self.elaborate_expression(expr);
                (HirStatement::Semi(expr), Type::Unit)
            }
            StatementKind::Interned(id) => {
                let kind = self.interner.get_statement_kind(id);
                let statement = Statement { kind: kind.clone(), location: statement.location };
                self.elaborate_statement_value_with_target_type(statement, target_type)
            }
            StatementKind::Error => (HirStatement::Error, Type::Error),
        }
    }

    pub(crate) fn elaborate_statement(&mut self, statement: Statement) -> (StmtId, Type) {
        self.elaborate_statement_with_target_type(statement, None)
    }

    pub(crate) fn elaborate_statement_with_target_type(
        &mut self,
        statement: Statement,
        target_type: Option<&Type>,
    ) -> (StmtId, Type) {
        let ((id, typ), has_errors) =
            self.with_error_guard(|this| this.elaborate_statement_inner(statement, target_type));

        if has_errors {
            self.interner.stmts_with_errors.insert(id);
        }

        (id, typ)
    }

    fn elaborate_statement_inner(
        &mut self,
        statement: Statement,
        target_type: Option<&Type>,
    ) -> (StmtId, Type) {
        let location = statement.location;
        let (hir_statement, typ) =
            self.elaborate_statement_value_with_target_type(statement, target_type);
        let id = self.interner.push_stmt_full(hir_statement, location);
        (id, typ)
    }

    pub(super) fn elaborate_local_let(&mut self, let_stmt: LetStatement) -> (HirStatement, Type) {
        let (let_statement, typ) = self.elaborate_let(let_stmt, None);
        (HirStatement::Let(let_statement), typ)
    }

    /// Elaborate a local or global let statement.
    /// If this is a global let, the DefinitionId of the global is specified so that
    /// elaborate_pattern can create a Global definition kind with the correct ID
    /// instead of a local one with a fresh ID.
    pub(super) fn elaborate_let(
        &mut self,
        let_stmt: LetStatement,
        global_id: Option<GlobalId>,
    ) -> (HirLetStatement, Type) {
        let no_type = let_stmt.r#type.is_none();
        let wildcard_allowed = if global_id.is_some() {
            WildcardAllowed::No(WildcardDisallowedContext::Global)
        } else {
            WildcardAllowed::Yes
        };
        let annotated_type = self.resolve_inferred_type(let_stmt.r#type, wildcard_allowed);

        let pattern_location = let_stmt.pattern.location();
        let expr_location = let_stmt.expression.location;
        let (expression, expr_type) = if no_type {
            self.elaborate_expression(let_stmt.expression)
        } else {
            self.elaborate_expression_with_target_type(let_stmt.expression, Some(&annotated_type))
        };

        // Require the top-level of a global's type to be fully-specified
        if global_id.is_some() && (no_type || annotated_type.contains_type_variable()) {
            let expected_type = annotated_type.clone();
            let error = ResolverError::UnspecifiedGlobalType {
                pattern_location,
                expr_location,
                expected_type,
            };
            self.push_err(error);
        }

        let definition = match global_id {
            None => {
                debug_assert!(!let_stmt.is_global_let);
                DefinitionKind::Local(Some(expression))
            }
            Some(id) => {
                debug_assert!(let_stmt.is_global_let);
                DefinitionKind::Global(id)
            }
        };

        // Now check if LHS is the same type as the RHS
        // Importantly, we do not coerce any types implicitly
        self.unify_with_coercions(&expr_type, &annotated_type, expression, expr_location, || {
            CompilationError::TypeError(TypeCheckError::TypeMismatch {
                expected_typ: annotated_type.to_string(),
                expr_typ: expr_type.to_string(),
                expr_location,
            })
        });

        let warn_if_unused =
            !let_stmt.attributes.iter().any(|attr| attr.kind.is_allow("unused_variables"));

        let r#type = annotated_type;
        let mut parameter_names_in_list = rustc_hash::FxHashMap::default();
        let pattern = self.elaborate_pattern(
            let_stmt.pattern,
            r#type.clone(),
            definition,
            warn_if_unused,
            &mut parameter_names_in_list,
        );

        let attributes = let_stmt.attributes;
        let comptime = let_stmt.comptime;
        let is_global_let = let_stmt.is_global_let;
        let let_ =
            HirLetStatement::new(pattern, r#type, expression, attributes, comptime, is_global_let);
        (let_, Type::Unit)
    }

    pub(super) fn elaborate_assign(&mut self, assign: AssignStatement) -> (HirStatement, Type) {
        let expr_location = assign.expression.location;
        let (expression, expr_type) = self.elaborate_expression(assign.expression);

        let (lvalue, lvalue_type, mutable, mut new_statements) =
            self.elaborate_lvalue(assign.lvalue);

        if !mutable {
            let (_, name, location) = self.get_lvalue_error_info(&lvalue);
            self.push_err(TypeCheckError::VariableMustBeMutable { name, location });
        } else {
            let (id, name, location) = self.get_lvalue_error_info(&lvalue);
            self.check_can_mutate_lambda_capture(id, name, location);
        }

        self.unify_with_coercions(&expr_type, &lvalue_type, expression, expr_location, || {
            CompilationError::TypeError(TypeCheckError::TypeMismatchWithSource {
                actual: expr_type.clone(),
                expected: lvalue_type.clone(),
                location: expr_location,
                source: Source::Assignment,
            })
        });

        let assign = HirAssignStatement { lvalue, expression };
        let assign = HirStatement::Assign(assign);

        if new_statements.is_empty() {
            (assign, Type::Unit)
        } else {
            let assign = self.interner.push_stmt_full(assign, expr_location);
            new_statements.push(assign);
            let block = HirExpression::Block(HirBlockExpression { statements: new_statements });
            let block = self.interner.push_expr_full(block, expr_location, Type::Unit);
            (HirStatement::Expression(block), Type::Unit)
        }
    }

    pub(super) fn elaborate_for(&mut self, for_loop: ForLoopStatement) -> (HirStatement, Type) {
        let (start, end) = match for_loop.range {
            ForRange::Range(bounds) => bounds.into_half_open(),
            ForRange::Array(_) => {
                let for_stmt =
                    for_loop.range.into_for(for_loop.identifier, for_loop.block, for_loop.location);

                return self.elaborate_statement_value(for_stmt);
            }
        };

        let start_location = start.location;
        let end_location = end.location;

        let (start_range, start_range_type) = self.elaborate_expression(start);
        let (end_range, end_range_type) = self.elaborate_expression(end);
        let (identifier, block) = (for_loop.identifier, for_loop.block);

        let old_loop = std::mem::take(&mut self.current_loop);

        self.current_loop = Some(Loop { is_for: true, has_break: false });
        self.push_scope();

        let kind = DefinitionKind::Local(None);
        let identifier = self.add_variable_decl(
            identifier, false, // mutable
            true,  // allow_shadowing
            true,  // warn_if_unused
            kind,
        );

        // Check that start range and end range have the same types
        self.unify(&start_range_type, &end_range_type, || TypeCheckError::TypeMismatch {
            expected_typ: start_range_type.to_string(),
            expr_typ: end_range_type.to_string(),
            expr_location: end_location,
        });

        let expected_type = self.polymorphic_integer();

        self.unify(&start_range_type, &expected_type, || TypeCheckError::TypeCannotBeUsed {
            typ: start_range_type.clone(),
            place: "for loop",
            location: start_location,
        });

        self.interner.push_definition_type(identifier.id, start_range_type);

        let block_location = block.type_location();
        let (block, block_type) = self.elaborate_expression(block);

        self.unify(&block_type, &Type::Unit, || TypeCheckError::TypeMismatch {
            expected_typ: Type::Unit.to_string(),
            expr_typ: block_type.to_string(),
            expr_location: block_location,
        });

        self.pop_scope();
        self.current_loop = old_loop;

        let statement =
            HirStatement::For(HirForStatement { start_range, end_range, block, identifier });

        (statement, Type::Unit)
    }

    pub(super) fn elaborate_loop(&mut self, loop_: LoopStatement) -> (HirStatement, Type) {
        let LoopStatement { body: block, loop_keyword_location: location } = loop_;
        let in_constrained_function = self.in_constrained_function();
        if in_constrained_function {
            self.push_err(ResolverError::LoopInConstrainedFn { location });
        }

        let old_loop = std::mem::take(&mut self.current_loop);
        self.current_loop = Some(Loop { is_for: false, has_break: false });
        self.push_scope();

        let block_location = block.type_location();
        let (block, block_type) = self.elaborate_expression(block);

        self.unify(&block_type, &Type::Unit, || TypeCheckError::TypeMismatch {
            expected_typ: Type::Unit.to_string(),
            expr_typ: block_type.to_string(),
            expr_location: block_location,
        });

        self.pop_scope();

        let last_loop =
            std::mem::replace(&mut self.current_loop, old_loop).expect("Expected a loop");
        if !last_loop.has_break {
            self.push_err(ResolverError::LoopWithoutBreak { location });
        }

        let statement = HirStatement::Loop(block);

        (statement, Type::Unit)
    }

    pub(super) fn elaborate_while(&mut self, while_: WhileStatement) -> (HirStatement, Type) {
        let in_constrained_function = self.in_constrained_function();
        if in_constrained_function {
            self.push_err(ResolverError::WhileInConstrainedFn {
                location: while_.while_keyword_location,
            });
        }

        let old_loop = std::mem::take(&mut self.current_loop);
        self.current_loop = Some(Loop { is_for: false, has_break: false });
        self.push_scope();

        let location = while_.condition.type_location();
        let (condition, cond_type) = self.elaborate_expression(while_.condition);

        self.unify(&cond_type, &Type::Bool, || TypeCheckError::TypeMismatch {
            expected_typ: Type::Bool.to_string(),
            expr_typ: cond_type.to_string(),
            expr_location: location,
        });

        let block_location = while_.body.type_location();
        let (block, block_type) = self.elaborate_expression(while_.body);

        self.unify(&block_type, &Type::Unit, || TypeCheckError::TypeMismatch {
            expected_typ: Type::Unit.to_string(),
            expr_typ: block_type.to_string(),
            expr_location: block_location,
        });

        self.pop_scope();

        std::mem::replace(&mut self.current_loop, old_loop).expect("Expected a loop");

        let statement = HirStatement::While(condition, block);

        (statement, Type::Unit)
    }

    fn elaborate_jump(&mut self, is_break: bool, location: Location) -> (HirStatement, Type) {
        let in_constrained_function = self.in_constrained_function();

        if in_constrained_function {
            self.push_err(ResolverError::JumpInConstrainedFn { is_break, location });
        }

        if let Some(current_loop) = &mut self.current_loop {
            if is_break {
                current_loop.has_break = true;
            }
        } else {
            self.push_err(ResolverError::JumpOutsideLoop { is_break, location });
        }

        let expr = if is_break { HirStatement::Break } else { HirStatement::Continue };
        (expr, Type::Unit)
    }

    fn get_lvalue_error_info(&self, lvalue: &HirLValue) -> (DefinitionId, String, Location) {
        match lvalue {
            HirLValue::Ident(name, _) => {
                let location = name.location;

                if let Some(definition) = self.interner.try_definition(name.id) {
                    (name.id, definition.name.clone(), location)
                } else {
                    (DefinitionId::dummy_id(), "(undeclared variable)".into(), location)
                }
            }
            HirLValue::MemberAccess { object, .. } => self.get_lvalue_error_info(object),
            HirLValue::Index { array, .. } => self.get_lvalue_error_info(array),
            HirLValue::Dereference { lvalue, .. } => self.get_lvalue_error_info(lvalue),
        }
    }

    /// Elaborates an lvalue returning:
    /// - The HirLValue equivalent of the given `lvalue`
    /// - The type being assigned to
    /// - Whether the underlying variable is mutable
    /// - A vector of new statements which need to prefix the resulting assign statement.
    ///   This hoists out any sub-expressions to simplify sequencing of side-effects.
    fn elaborate_lvalue(&mut self, lvalue: LValue) -> (HirLValue, Type, bool, Vec<StmtId>) {
        match lvalue {
            LValue::Path(path) => {
                let mut mutable = true;
                let location = path.location;
                let path = self.validate_path(path);
                match self.get_ident_from_path_or_error(path.clone()) {
                    Ok(((ident, scope_index), _)) => {
                        self.resolve_local_variable(ident.clone(), scope_index);

                        if let Some(definition) = self.interner.try_definition(ident.id) {
                            mutable = definition.mutable;

                            if definition.comptime && !self.in_comptime_context() {
                                self.push_err(
                                    ResolverError::MutatingComptimeInNonComptimeContext {
                                        name: definition.name.clone(),
                                        location: ident.location,
                                    },
                                );
                            }
                        }

                        let typ =
                            self.interner.definition_type(ident.id).instantiate(self.interner).0;
                        let typ = typ.follow_bindings();

                        self.interner.add_local_reference(ident.id, location);

                        (HirLValue::Ident(ident.clone(), typ.clone()), typ, mutable, Vec::new())
                    }
                    Err(error) => {
                        // We couldn't find a variable or global. Let's see if the identifier refers to something
                        // else, like a module or type.
                        let result = self.resolve_path_inner(
                            path,
                            PathResolutionTarget::Value,
                            super::PathResolutionMode::MarkAsUsed,
                        );
                        if let Ok(result) = result {
                            self.push_errors(result.errors);
                            self.push_err(ResolverError::Expected {
                                location,
                                expected: "value",
                                got: result.item.description(),
                            });
                        } else {
                            self.push_err(error);
                        }

                        let id = DefinitionId::dummy_id();
                        let ident = HirIdent::non_trait_method(id, location);
                        let typ = Type::Error;
                        (HirLValue::Ident(ident.clone(), typ.clone()), typ, mutable, Vec::new())
                    }
                }
            }
            LValue::MemberAccess { object, field_name, location } => {
                let (object, lhs_type, mut mutable, statements) = self.elaborate_lvalue(*object);
                let mut object = Box::new(object);
                let field_name = field_name.clone();

                let object_ref = &mut object;
                let mutable_ref = &mut mutable;

                let dereference_lhs = move |_: &mut Self, _, element_type| {
                    // We must create a temporary value first to move out of object_ref before
                    // we eventually reassign to it.
                    let id = DefinitionId::dummy_id();
                    let ident = HirIdent::non_trait_method(id, location);
                    let tmp_value = HirLValue::Ident(ident, Type::Error);

                    let lvalue = std::mem::replace(object_ref, Box::new(tmp_value));
                    *object_ref = Box::new(HirLValue::Dereference {
                        lvalue,
                        element_type,
                        location,
                        implicitly_added: true,
                    });
                    *mutable_ref = true;
                };

                let name = field_name.as_str();
                let (object_type, field_index) = self
                    .check_field_access(
                        &lhs_type,
                        name,
                        field_name.location(),
                        Some(dereference_lhs),
                    )
                    .unwrap_or((Type::Error, 0));

                let field_index = Some(field_index);
                let typ = object_type.clone();
                let lvalue =
                    HirLValue::MemberAccess { object, field_name, field_index, typ, location };
                (lvalue, object_type, mutable, statements)
            }
            LValue::Index { array, index, location } => {
                let expr_location = index.location;
                let (mut index, index_type) = self.elaborate_expression(index);

                let expected = Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo);
                self.unify(&index_type, &expected, || TypeCheckError::TypeMismatchWithSource {
                    expected: expected.clone(),
                    actual: index_type.clone(),
                    location: expr_location,
                    source: Source::ArrayIndex,
                });

                let (mut lvalue, mut lvalue_type, mut mutable, mut statements) =
                    self.elaborate_lvalue(*array);

                // Push the index expression to the end of the new statements list, referring to it
                // afterward with a let binding. Note that since we recur first then push to the
                // end of the list we're evaluating side-effects such that in `a[i][j]`, `i` will
                // be evaluated first, followed by `j`.
                if let Some((index_definition, new_index)) =
                    self.fresh_definition_for_lvalue_index(index, index_type, expr_location)
                {
                    index = new_index;
                    statements.push(index_definition);
                }

                // Before we check that the lvalue is an array, try to dereference it as many times
                // as needed to unwrap any `&` or `&mut` wrappers.
                while let Type::Reference(element, _) = lvalue_type.follow_bindings() {
                    let element_type = element.as_ref().clone();
                    lvalue = HirLValue::Dereference {
                        lvalue: Box::new(lvalue),
                        element_type,
                        location,
                        implicitly_added: true,
                    };
                    lvalue_type = *element;
                    // We know this value to be mutable now since we found an `&mut`
                    mutable = true;
                }

                let typ = match lvalue_type.follow_bindings() {
                    Type::Array(_, elem_type) => *elem_type,
                    Type::Vector(elem_type) => *elem_type,
                    Type::Error => Type::Error,
                    Type::String(_) => {
                        let (_id, _lvalue_name, lvalue_location) =
                            self.get_lvalue_error_info(&lvalue);
                        self.push_err(TypeCheckError::StringIndexAssign {
                            location: lvalue_location,
                        });
                        Type::Error
                    }
                    Type::TypeVariable(_) => {
                        self.push_err(TypeCheckError::TypeAnnotationsNeededForIndex { location });
                        Type::Error
                    }
                    other => {
                        self.push_err(TypeCheckError::TypeMismatch {
                            expected_typ: "array".to_string(),
                            expr_typ: other.to_string(),
                            expr_location: location,
                        });
                        Type::Error
                    }
                };

                let array = Box::new(lvalue);
                let array_type = typ.clone();
                (HirLValue::Index { array, index, typ, location }, array_type, mutable, statements)
            }
            LValue::Dereference(lvalue, location) => {
                let (lvalue, reference_type, _, statements) = self.elaborate_lvalue(*lvalue);
                let lvalue = Box::new(lvalue);

                let element_type = Type::type_variable(self.interner.next_type_variable_id());

                // Always expect a mutable reference here since we're storing to it
                let expected_type = Type::Reference(Box::new(element_type.clone()), true);

                self.unify(&reference_type, &expected_type, || TypeCheckError::TypeMismatch {
                    expected_typ: expected_type.to_string(),
                    expr_typ: reference_type.to_string(),
                    expr_location: location,
                });

                // Dereferences are always mutable since we already type checked against a &mut T
                let typ = element_type.clone();
                let lvalue = HirLValue::Dereference {
                    lvalue,
                    element_type,
                    location,
                    implicitly_added: false,
                };
                (lvalue, typ, true, statements)
            }
            LValue::Interned(id, location) => {
                let lvalue = self.interner.get_lvalue(id, location).clone();
                self.elaborate_lvalue(lvalue)
            }
        }
    }

    fn fresh_definition_for_lvalue_index(
        &mut self,
        expr: ExprId,
        typ: Type,
        location: Location,
    ) -> Option<(StmtId, ExprId)> {
        // If the original expression trivially cannot have side-effects, don't bother cluttering
        // the output with a let binding. Note that array literals can have side-effects but these
        // would produce type errors anyway.
        if matches!(
            self.interner.expression(&expr),
            HirExpression::Ident(..) | HirExpression::Literal(..)
        ) {
            return None;
        }

        let id = self.interner.push_definition(
            format!("i_{}", self.interner.definition_count()),
            false,
            false,
            DefinitionKind::Local(None),
            location,
        );
        let ident = HirIdent::non_trait_method(id, location);
        let ident_expr = HirExpression::Ident(ident.clone(), None);

        let ident_id = self.interner.push_expr_full(ident_expr, location, typ.clone());

        let pattern = HirPattern::Identifier(ident);
        let let_ = HirStatement::Let(HirLetStatement::basic(pattern, typ, expr));
        let let_ = self.interner.push_stmt_full(let_, location);
        Some((let_, ident_id))
    }

    fn elaborate_comptime_statement(
        &mut self,
        statement: Statement,
        target_type: Option<&Type>,
    ) -> (HirStatement, Type) {
        let location = statement.location;
        let hir_statement = self.elaborate_in_comptime_context(|this| {
            let (hir_statement, typ) = this.elaborate_statement(statement);

            // If the comptime statement is expected to return a specific type, unify their types.
            // This for example allows this code to compile:
            //
            // ```
            // fn foo() -> u8 {
            //   comptime { 1 }
            // }
            // ```
            //
            // If we don't do this, "1" will end up with the default integer or field type,
            // which is Field.
            if let Some(target_type) = target_type {
                this.unify(&typ, target_type, || TypeCheckError::TypeMismatch {
                    expected_typ: target_type.to_string(),
                    expr_typ: typ.to_string(),
                    expr_location: location,
                });
            }

            hir_statement
        });

        // Run the interpreter - it will check if execution has been halted
        let mut interpreter = self.setup_interpreter();
        let value = interpreter.evaluate_statement(hir_statement);

        let (expr, typ) = self.inline_comptime_value(value, location);

        let location = self.interner.id_location(hir_statement);
        self.debug_comptime(location, |interner| expr.to_display_ast(interner).kind);

        (HirStatement::Expression(expr), typ)
    }
}
