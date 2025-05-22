use noirc_frontend::{
    NamedGeneric, Type, TypeBindings,
    ast::{ItemVisibility, UnaryOp},
    hir::def_map::ModuleDefId,
    hir_def::{
        expr::{
            Constructor, HirArrayLiteral, HirBlockExpression, HirCallExpression, HirExpression,
            HirIdent, HirLambda, HirLiteral, HirMatch, ImplKind,
        },
        stmt::{HirLValue, HirPattern, HirStatement},
    },
    node_interner::{DefinitionId, DefinitionKind, ExprId, StmtId},
    token::FmtStrFragment,
};

use super::ItemPrinter;

impl ItemPrinter<'_, '_> {
    fn show_hir_expression_id(&mut self, expr_id: ExprId) {
        let hir_expr = self.interner.expression(&expr_id);
        self.show_hir_expression(hir_expr);
    }

    fn dereference_hir_expression_id(&self, expr_id: ExprId) -> ExprId {
        let hir_expr = self.interner.expression(&expr_id);
        let HirExpression::Prefix(prefix) = &hir_expr else {
            return expr_id;
        };

        match prefix.operator {
            UnaryOp::Reference { .. } | UnaryOp::Dereference { implicitly_added: true } => {
                prefix.rhs
            }
            UnaryOp::Minus | UnaryOp::Not | UnaryOp::Dereference { implicitly_added: false } => {
                expr_id
            }
        }
    }

    fn show_hir_expression_id_maybe_inside_parens(&mut self, expr_id: ExprId) {
        let hir_expr = self.interner.expression(&expr_id);
        let parens = hir_expression_needs_parentheses(&hir_expr);
        if parens {
            self.push('(');
        }
        self.show_hir_expression(hir_expr);
        if parens {
            self.push(')');
        }
    }

    fn show_hir_expression_id_maybe_inside_curlies(&mut self, expr_id: ExprId) {
        let hir_expr = self.interner.expression(&expr_id);
        let curlies = hir_expression_needs_parentheses(&hir_expr);
        if curlies {
            self.push('{');
        }
        self.show_hir_expression(hir_expr);
        if curlies {
            self.push('}');
        }
    }

    pub(super) fn show_hir_expression(&mut self, hir_expr: HirExpression) {
        match hir_expr {
            HirExpression::Ident(hir_ident, generics) => {
                self.show_hir_ident(hir_ident);
                if let Some(generics) = generics {
                    let use_colons = true;
                    self.show_generic_types(&generics, use_colons);
                }
            }
            HirExpression::Literal(hir_literal) => {
                self.show_hir_literal(hir_literal);
            }
            HirExpression::Block(hir_block_expression) => {
                self.show_hir_block_expression(hir_block_expression);
            }
            HirExpression::Prefix(hir_prefix_expression) => match hir_prefix_expression.operator {
                UnaryOp::Minus => {
                    self.push('-');
                    self.show_hir_expression_id_maybe_inside_parens(hir_prefix_expression.rhs);
                }
                UnaryOp::Not => {
                    self.push('!');
                    self.show_hir_expression_id_maybe_inside_parens(hir_prefix_expression.rhs);
                }
                UnaryOp::Reference { mutable } => {
                    if mutable {
                        self.push_str("&mut ");
                    } else {
                        self.push_str("&");
                    }
                    self.show_hir_expression_id(hir_prefix_expression.rhs);
                }
                UnaryOp::Dereference { implicitly_added } => {
                    if implicitly_added {
                        self.show_hir_expression_id(hir_prefix_expression.rhs);
                    } else {
                        self.push('*');
                        self.show_hir_expression_id_maybe_inside_parens(hir_prefix_expression.rhs);
                    }
                }
            },
            HirExpression::Infix(hir_infix_expression) => {
                self.show_hir_expression_id_maybe_inside_parens(hir_infix_expression.lhs);
                self.push(' ');
                self.push_str(&hir_infix_expression.operator.kind.to_string());
                self.push(' ');
                self.show_hir_expression_id_maybe_inside_parens(hir_infix_expression.rhs);
            }
            HirExpression::Index(hir_index_expression) => {
                self.show_hir_expression_id_maybe_inside_parens(hir_index_expression.collection);
                self.push('[');
                self.show_hir_expression_id(hir_index_expression.index);
                self.push(']');
            }
            HirExpression::Constructor(hir_constructor_expression) => {
                // let data_type = hir_constructor_expression.r#type.borrow();
                let typ = Type::DataType(
                    hir_constructor_expression.r#type.clone(),
                    hir_constructor_expression.struct_generics.clone(),
                );
                if self.self_type.as_ref() == Some(&typ) {
                    self.push_str("Self");
                } else {
                    let data_type = hir_constructor_expression.r#type.borrow();

                    let use_import = true;
                    self.show_reference_to_module_def_id(
                        ModuleDefId::TypeId(data_type.id),
                        data_type.visibility,
                        use_import,
                    );

                    let use_colons = true;
                    self.show_generic_types(
                        &hir_constructor_expression.struct_generics,
                        use_colons,
                    );
                }

                self.push_str(" { ");
                self.show_separated_by_comma(
                    &hir_constructor_expression.fields,
                    |this, (name, value)| {
                        this.push_str(&name.to_string());
                        this.push_str(": ");
                        this.show_hir_expression_id(*value);
                    },
                );
                self.push('}');
            }
            HirExpression::EnumConstructor(constructor) => {
                let data_type = constructor.r#type.borrow();
                let use_import = true;
                self.show_reference_to_module_def_id(
                    ModuleDefId::TypeId(data_type.id),
                    data_type.visibility,
                    use_import,
                );

                let variant = data_type.variant_at(constructor.variant_index);
                self.push_str("::");
                self.push_str(&variant.name.to_string());
                if variant.is_function {
                    self.push('(');
                    self.show_hir_expression_ids_separated_by_comma(&constructor.arguments);
                    self.push(')');
                }
            }
            HirExpression::MemberAccess(hir_member_access) => {
                self.show_hir_expression_id_maybe_inside_parens(hir_member_access.lhs);
                self.push('.');
                self.push_str(&hir_member_access.rhs.to_string());
            }
            HirExpression::Call(hir_call_expression) => {
                if self.try_show_hir_call_as_method(&hir_call_expression) {
                    return;
                }

                let func = self.interner.expression(&hir_call_expression.func);

                // Special case: a call on a member access must have parentheses around it
                if matches!(func, HirExpression::MemberAccess(..)) {
                    self.push('(');
                    self.show_hir_expression_id(hir_call_expression.func);
                    self.push(')');
                } else {
                    self.show_hir_expression_id_maybe_inside_parens(hir_call_expression.func);
                }

                if hir_call_expression.is_macro_call {
                    self.push('!');
                }
                self.push('(');
                self.show_hir_expression_ids_separated_by_comma(&hir_call_expression.arguments);
                self.push(')');
            }
            HirExpression::Constrain(hir_constrain_expression) => {
                self.push_str("assert(");
                self.show_hir_expression_id(hir_constrain_expression.0);
                if let Some(message_id) = hir_constrain_expression.2 {
                    self.push_str(", ");
                    self.show_hir_expression_id(message_id);
                }
                self.push(')');
            }
            HirExpression::Cast(hir_cast_expression) => {
                self.show_hir_expression_id_maybe_inside_parens(hir_cast_expression.lhs);
                self.push_str(" as ");
                self.show_type(&hir_cast_expression.r#type);
            }
            HirExpression::If(hir_if_expression) => {
                self.push_str("if ");
                self.show_hir_expression_id(hir_if_expression.condition);
                self.push(' ');
                self.show_hir_expression_id(hir_if_expression.consequence);
                if let Some(alternative) = hir_if_expression.alternative {
                    self.push_str(" else ");
                    self.show_hir_expression_id(alternative);
                }
            }
            HirExpression::Match(hir_match) => self.show_hir_match(hir_match),
            HirExpression::Tuple(expr_ids) => {
                let len = expr_ids.len();
                self.push('(');
                self.show_hir_expression_ids_separated_by_comma(&expr_ids);
                if len == 1 {
                    self.push(',');
                }
                self.push(')');
            }
            HirExpression::Lambda(hir_lambda) => self.show_hir_lambda(hir_lambda),
            HirExpression::Quote(tokens) => {
                self.show_quoted(&tokens.0);
            }
            HirExpression::Unsafe(hir_block_expression) => {
                // The safety comment was already outputted for the enclosing statement
                self.push_str("unsafe ");
                self.show_hir_block_expression(hir_block_expression);
            }
            HirExpression::Error => unreachable!("error nodes should not happen"),
            HirExpression::Unquote(_) => unreachable!("unquote should not happen"),
        }
    }

    pub(super) fn show_hir_lambda(&mut self, hir_lambda: HirLambda) {
        self.push('|');
        self.show_separated_by_comma(&hir_lambda.parameters, |this, (parameter, typ)| {
            this.show_hir_pattern(parameter.clone());
            this.push_str(": ");
            this.show_type(typ);
        });
        self.push_str("| ");
        if hir_lambda.return_type != Type::Unit {
            self.push_str("-> ");
            self.show_type(&hir_lambda.return_type);
            self.push_str(" ");
        }
        self.show_hir_expression_id_maybe_inside_curlies(hir_lambda.body);
    }

    fn show_hir_match(&mut self, hir_match: HirMatch) {
        match hir_match {
            HirMatch::Success(expr_id) => self.show_hir_expression_id(expr_id),
            HirMatch::Failure { .. } => {
                unreachable!("At this point code should not have errors")
            }
            HirMatch::Guard { cond, body, otherwise } => {
                self.push_str("if ");
                self.show_hir_expression_id(cond);
                self.push(' ');
                self.show_hir_expression_id(body);
                self.push_str(" else ");
                self.show_hir_match(*otherwise);
            }
            HirMatch::Switch(variable, cases, default) => {
                self.push_str("match ");
                self.show_definition_id(variable);
                self.push_str(" {\n");
                self.increase_indent();
                for case in cases {
                    let typ = self.interner.definition_type(variable).follow_bindings();
                    self.write_indent();

                    if !matches!(typ, Type::Tuple(..)) {
                        self.show_constructor(case.constructor);
                    }

                    if !case.arguments.is_empty() {
                        if let Some(fields) = get_type_fields(&typ) {
                            self.push('{');
                            self.show_separated_by_comma(
                                &case.arguments.into_iter().zip(fields).collect::<Vec<_>>(),
                                |this, (argument, (name, _, _))| {
                                    this.push_str(name);
                                    this.push_str(": ");
                                    this.show_definition_id(*argument);
                                },
                            );
                            self.push('}');
                        } else {
                            self.push('(');
                            self.show_separated_by_comma(&case.arguments, |this, argument| {
                                this.show_definition_id(*argument);
                            });
                            self.push(')');
                        }
                    }
                    self.push_str(" => ");
                    self.show_hir_match(case.body);
                    self.push(',');
                    self.push('\n');
                }

                if let Some(default) = default {
                    self.write_indent();
                    self.push_str("_ => ");
                    self.show_hir_match(*default);
                    self.push(',');
                    self.push('\n');
                }

                self.decrease_indent();
                self.write_indent();
                self.push('}');
            }
        }
    }

    fn show_constructor(&mut self, constructor: Constructor) {
        match constructor {
            Constructor::True => self.push_str("true"),
            Constructor::False => self.push_str("false"),
            Constructor::Unit => self.push_str("()"),
            Constructor::Int(signed_field) => self.push_str(&signed_field.to_string()),
            Constructor::Tuple(items) => {
                let len = items.len();
                self.push('(');
                self.show_types_separated_by_comma(&items);
                if len == 1 {
                    self.push(',');
                }
                self.push(')');
            }
            Constructor::Variant(typ, index) => {
                self.show_type_name_as_data_type(&typ);

                let Type::DataType(data_type, _) = typ.follow_bindings() else {
                    panic!("Expected data type")
                };
                let data_type = data_type.borrow();
                if data_type.is_enum() {
                    let variant = data_type.variant_at(index);
                    self.push_str("::");
                    self.push_str(&variant.name.to_string());
                }
            }
            Constructor::Range(from, to) => {
                self.push_str(&from.to_string());
                self.push_str("..");
                self.push_str(&to.to_string());
            }
        }
    }

    fn try_show_hir_call_as_method(&mut self, hir_call_expression: &HirCallExpression) -> bool {
        let arguments = &hir_call_expression.arguments;

        // If there are no arguments this is definitely not a method call
        if arguments.is_empty() {
            return false;
        }

        // A method call must have `func` be a HirIdent
        let HirExpression::Ident(hir_ident, generics) =
            self.interner.expression(&hir_call_expression.func)
        else {
            return false;
        };

        // That HirIdent must be a function reference
        let definition = self.interner.definition(hir_ident.id);
        let DefinitionKind::Function(func_id) = definition.kind else {
            return false;
        };

        // Is this `self.foo()` where `self` is currently a trait?
        // If so, show it as `self.foo()` instead of `Self::foo(self)`.
        let mut method_on_trait_self = false;

        // Special case: assumed trait method
        if let ImplKind::TraitMethod(trait_method) = hir_ident.impl_kind {
            if trait_method.assumed {
                if let Type::NamedGeneric(NamedGeneric { name, .. }) = &trait_method.constraint.typ
                {
                    if name.to_string() == "Self" {
                        method_on_trait_self = true;
                    }
                }

                if !method_on_trait_self {
                    self.show_type(&trait_method.constraint.typ);
                    self.push_str("::");
                    self.push_str(self.interner.function_name(&func_id));
                    if let Some(generics) = generics {
                        let use_colons = true;
                        self.show_generic_types(&generics, use_colons);
                    }
                    self.push('(');
                    self.show_hir_expression_ids_separated_by_comma(arguments);
                    self.push(')');
                    return true;
                }
            }
        }

        // The function must have a self type
        let func_meta = self.interner.function_meta(&func_id);

        // Don't do this for trait methods (refer to the trait name instead)
        if func_meta.trait_id.is_some() && !method_on_trait_self {
            return false;
        }

        let Some(self_type) = &func_meta.self_type else {
            return false;
        };

        // And it must have parameters
        if func_meta.parameters.is_empty() {
            return false;
        }

        // The first parameter must unify with the self type (as-is or after removing `&mut`)
        let param_type = func_meta.parameters.0[0].1.follow_bindings();
        let param_type = if let Type::Reference(typ, ..) = param_type { *typ } else { param_type };

        let mut bindings = TypeBindings::default();
        if self_type.try_unify(&param_type, &mut bindings).is_err() {
            return false;
        }

        let first_argument = self.dereference_hir_expression_id(arguments[0]);
        self.show_hir_expression_id_maybe_inside_parens(first_argument);
        self.push('.');
        self.push_str(self.interner.function_name(&func_id));
        if let Some(generics) = generics {
            let use_colons = true;
            self.show_generic_types(&generics, use_colons);
        }
        self.push('(');
        self.show_hir_expression_ids_separated_by_comma(&arguments[1..]);
        self.push(')');

        true
    }

    fn show_hir_block_expression(&mut self, block: HirBlockExpression) {
        self.push_str("{\n");
        self.increase_indent();
        let len = block.statements.len();
        for (index, statement) in block.statements.into_iter().enumerate() {
            self.write_indent();
            self.show_hir_statement_id(statement);

            // For some reason some statements in the middle of a block end up being `Expression`
            // and not `Semi`, so we add a semicolon, if needed, to produce valid syntax.
            if index != len - 1 && !self.string.ends_with(';') {
                self.push(';');
            }

            self.push_str("\n");
        }
        self.decrease_indent();
        self.write_indent();
        self.push('}');
    }

    fn show_hir_expression_ids_separated_by_comma(&mut self, expr_ids: &[ExprId]) {
        self.show_separated_by_comma(expr_ids, |this, expr_id| {
            this.show_hir_expression_id(*expr_id);
        });
    }

    fn show_hir_statement_id(&mut self, stmt_id: StmtId) {
        let statement = self.interner.statement(&stmt_id);
        self.show_hir_statement(statement);
    }

    fn show_hir_statement(&mut self, statement: HirStatement) {
        // A safety comment can be put before a statement and it applies to any `unsafe`
        // expression inside it. Here we check if the statement has `unsafe` in it and
        // put a safety comment right before it. When printing an `Unsafe` expression
        // we'll never include a safety comment at that point.
        let has_unsafe = self.statement_has_unsafe(&statement);
        if has_unsafe {
            self.push_str("// Safety: comment added by `nargo expand`\n");
            self.write_indent();
        }

        match statement {
            HirStatement::Let(hir_let_statement) => {
                self.push_str("let ");
                self.show_hir_pattern(hir_let_statement.pattern);
                self.push_str(": ");
                self.show_type(&hir_let_statement.r#type);
                self.push_str(" = ");
                self.show_hir_expression_id(hir_let_statement.expression);
                self.push(';');
            }
            HirStatement::Assign(hir_assign_statement) => {
                self.show_hir_lvalue(hir_assign_statement.lvalue);
                self.push_str(" = ");
                self.show_hir_expression_id(hir_assign_statement.expression);
                self.push(';');
            }
            HirStatement::For(hir_for_statement) => {
                self.push_str("for ");
                self.show_hir_ident(hir_for_statement.identifier);
                self.push_str(" in ");
                self.show_hir_expression_id(hir_for_statement.start_range);
                self.push_str("..");
                self.show_hir_expression_id(hir_for_statement.end_range);
                self.push(' ');
                self.show_hir_expression_id(hir_for_statement.block);
            }
            HirStatement::Loop(expr_id) => {
                self.push_str("loop ");
                self.show_hir_expression_id(expr_id);
            }
            HirStatement::While(condition, body) => {
                self.push_str("while ");
                self.show_hir_expression_id(condition);
                self.push(' ');
                self.show_hir_expression_id(body);
            }
            HirStatement::Break => {
                self.push_str("break;");
            }
            HirStatement::Continue => {
                self.push_str("continue;");
            }
            HirStatement::Expression(expr_id) => {
                self.show_hir_expression_id(expr_id);
            }
            HirStatement::Semi(expr_id) => {
                self.show_hir_expression_id(expr_id);
                self.push(';');
            }
            HirStatement::Comptime(_) => unreachable!("comptime should not happen"),
            HirStatement::Error => unreachable!("error should not happen"),
        }
    }

    fn show_hir_literal(&mut self, literal: HirLiteral) {
        match literal {
            HirLiteral::Array(hir_array_literal) => {
                self.push_str("[");
                self.show_hir_array_literal(hir_array_literal);
                self.push(']');
            }
            HirLiteral::Slice(hir_array_literal) => {
                self.push_str("&[");
                self.show_hir_array_literal(hir_array_literal);
                self.push(']');
            }
            HirLiteral::Bool(value) => {
                self.push_str(&value.to_string());
            }
            HirLiteral::Integer(signed_field) => {
                self.push_str(&signed_field.to_string());
            }
            HirLiteral::Str(string) => {
                self.push_str(&format!("{:?}", string));
            }
            HirLiteral::FmtStr(fmt_str_fragments, _expr_ids, _) => {
                self.push_str("f\"");
                for fragment in fmt_str_fragments {
                    match fragment {
                        FmtStrFragment::String(string) => {
                            let string = string
                                .replace('\\', "\\\\")
                                .replace('\n', "\\n")
                                .replace('\t', "\\t")
                                .replace('{', "{{")
                                .replace('}', "}}");
                            self.push_str(&string);
                        }
                        FmtStrFragment::Interpolation(string, _) => {
                            self.push('{');
                            self.push_str(&string);
                            self.push('}');
                        }
                    }
                }
                self.push('"');
            }
            HirLiteral::Unit => {
                self.push_str("()");
            }
        }
    }

    fn show_hir_array_literal(&mut self, array: HirArrayLiteral) {
        match array {
            HirArrayLiteral::Standard(expr_ids) => {
                self.show_hir_expression_ids_separated_by_comma(&expr_ids);
            }
            HirArrayLiteral::Repeated { repeated_element, length } => {
                self.show_hir_expression_id(repeated_element);
                self.push_str("; ");
                self.show_type(&length);
            }
        }
    }

    fn show_hir_lvalue(&mut self, lvalue: HirLValue) {
        match lvalue {
            HirLValue::Ident(hir_ident, _) => {
                self.show_hir_ident(hir_ident);
            }
            HirLValue::MemberAccess { object, field_name, field_index: _, typ: _, location: _ } => {
                self.show_hir_lvalue(*object);
                self.push('.');
                self.push_str(&field_name.to_string());
            }
            HirLValue::Index { array, index, typ: _, location: _ } => {
                self.show_hir_lvalue(*array);
                self.push('[');
                self.show_hir_expression_id(index);
                self.push(']');
            }
            HirLValue::Dereference { lvalue, implicitly_added, element_type: _, location: _ } => {
                if implicitly_added {
                    self.show_hir_lvalue(*lvalue);
                } else {
                    // Even though parentheses aren't always required, it's tricky to
                    // figure out exactly when so we always include them.
                    self.push_str("*(");
                    self.show_hir_lvalue(*lvalue);
                    self.push(')');
                }
            }
        }
    }

    fn show_hir_pattern(&mut self, pattern: HirPattern) {
        match pattern {
            HirPattern::Identifier(hir_ident) => self.show_hir_ident(hir_ident),
            HirPattern::Mutable(hir_pattern, _) => {
                self.push_str("mut ");
                self.show_hir_pattern(*hir_pattern);
            }
            HirPattern::Tuple(hir_patterns, _location) => {
                let len = hir_patterns.len();
                self.push('(');
                self.show_separated_by_comma(&hir_patterns, |this, pattern| {
                    this.show_hir_pattern(pattern.clone());
                });
                if len == 1 {
                    self.push(',');
                }
                self.push(')');
            }
            HirPattern::Struct(typ, items, _location) => {
                self.show_type_name_as_data_type(&typ);
                self.push_str(" {\n");
                self.increase_indent();
                self.show_separated_by_comma(&items, |this, (name, pattern)| {
                    this.push_str(&name.to_string());
                    this.push_str(": ");
                    this.show_hir_pattern(pattern.clone());
                });
                self.push('\n');
                self.decrease_indent();
                self.write_indent();
                self.push('}');
            }
        }
    }

    fn show_definition_id(&mut self, definition_id: DefinitionId) {
        let location = self.interner.definition(definition_id).location;
        let ident = HirIdent::non_trait_method(definition_id, location);
        self.show_hir_ident(ident);
    }

    fn show_hir_ident(&mut self, ident: HirIdent) {
        let definition = self.interner.definition(ident.id);
        match definition.kind {
            DefinitionKind::Function(func_id) => {
                let func_meta = self.interner.function_meta(&func_id);
                if func_meta.self_type.is_some() && func_meta.self_type == self.self_type {
                    // No need to fully-qualify the function name if its self type is the current self type
                    let name = self.interner.function_name(&func_id);
                    self.push_str("Self::");
                    self.push_str(name);
                } else {
                    let use_import = true;
                    let visibility = self.interner.function_visibility(func_id);
                    self.show_reference_to_module_def_id(
                        ModuleDefId::FunctionId(func_id),
                        visibility,
                        use_import,
                    );
                }
            }
            DefinitionKind::Global(global_id) => {
                let global_info = self.interner.get_global(global_id);
                let typ = self.interner.definition_type(global_info.definition_id);

                // Special case: the global is an enum value
                let typ = if let Type::Forall(_, typ) = typ { *typ } else { typ };
                if let Type::DataType(data_type, _generics) = &typ {
                    let data_type = data_type.borrow();
                    if data_type.is_enum() {
                        self.show_type_name_as_data_type(&typ);
                        self.push_str("::");
                    }
                }
                let use_import = true;
                self.show_reference_to_module_def_id(
                    ModuleDefId::GlobalId(global_id),
                    global_info.visibility,
                    use_import,
                );
            }
            DefinitionKind::Local(..) | DefinitionKind::NumericGeneric(..) => {
                let name = self.interner.definition_name(ident.id);

                // The compiler uses '$' for some internal identifiers.
                // We replace them with "___" to make sure they have valid syntax, even though
                // there's a tiny change they might collide with user code (unlikely, really).
                //
                // In other cases these internal names have spaces.
                let name = name.replace(['$', ' '], "___");

                self.push_str(&name);
            }
        }
    }

    fn statement_id_has_unsafe(&self, stmt_id: StmtId) -> bool {
        let statement = self.interner.statement(&stmt_id);
        self.statement_has_unsafe(&statement)
    }

    fn statement_has_unsafe(&self, statement: &HirStatement) -> bool {
        match statement {
            HirStatement::Let(hir_let_statement) => {
                self.expression_id_has_unsafe(hir_let_statement.expression)
            }
            HirStatement::Assign(hir_assign_statement) => {
                self.expression_id_has_unsafe(hir_assign_statement.expression)
            }
            HirStatement::For(hir_for_statement) => {
                // We don't check the block, as the block consists of statements and we
                // can put the safety comment on top of the ones that have unsafe
                self.expression_id_has_unsafe(hir_for_statement.start_range)
                    || self.expression_id_has_unsafe(hir_for_statement.end_range)
            }
            HirStatement::Loop(expr_id) => self.expression_id_has_unsafe(*expr_id),
            HirStatement::While(expr_id, expr_id2) => {
                self.expression_id_has_unsafe(*expr_id) || self.expression_id_has_unsafe(*expr_id2)
            }
            HirStatement::Break => false,
            HirStatement::Continue => false,
            HirStatement::Expression(expr_id) => self.expression_id_has_unsafe(*expr_id),
            HirStatement::Semi(expr_id) => self.expression_id_has_unsafe(*expr_id),
            HirStatement::Comptime(stmt_id) => self.statement_id_has_unsafe(*stmt_id),
            HirStatement::Error => false,
        }
    }

    fn expression_id_has_unsafe(&self, expr_id: ExprId) -> bool {
        let hir_expr = self.interner.expression(&expr_id);
        self.expression_has_unsafe(hir_expr)
    }

    fn expression_has_unsafe(&self, expr: HirExpression) -> bool {
        match expr {
            HirExpression::Ident(..) => false,
            HirExpression::Literal(hir_literal) => match hir_literal {
                HirLiteral::Array(hir_array_literal) | HirLiteral::Slice(hir_array_literal) => {
                    match hir_array_literal {
                        HirArrayLiteral::Standard(expr_ids) => {
                            expr_ids.iter().any(|expr_id| self.expression_id_has_unsafe(*expr_id))
                        }
                        HirArrayLiteral::Repeated { repeated_element, length: _ } => {
                            self.expression_id_has_unsafe(repeated_element)
                        }
                    }
                }
                HirLiteral::FmtStr(_, expr_ids, _) => {
                    expr_ids.iter().any(|expr_id| self.expression_id_has_unsafe(*expr_id))
                }
                HirLiteral::Bool(_)
                | HirLiteral::Integer(..)
                | HirLiteral::Str(_)
                | HirLiteral::Unit => false,
            },
            HirExpression::Block(_) => {
                // A block consists of statements so if any of those have `unsafe`, those
                // should have the safety comment, not this wrapping statement
                false
            }
            HirExpression::Prefix(hir_prefix_expression) => {
                self.expression_id_has_unsafe(hir_prefix_expression.rhs)
            }
            HirExpression::Infix(hir_infix_expression) => {
                self.expression_id_has_unsafe(hir_infix_expression.lhs)
                    || self.expression_id_has_unsafe(hir_infix_expression.rhs)
            }
            HirExpression::Index(hir_index_expression) => {
                self.expression_id_has_unsafe(hir_index_expression.collection)
                    || self.expression_id_has_unsafe(hir_index_expression.index)
            }
            HirExpression::Constructor(hir_constructor_expression) => hir_constructor_expression
                .fields
                .iter()
                .any(|(_, expr_id)| self.expression_id_has_unsafe(*expr_id)),
            HirExpression::EnumConstructor(hir_enum_constructor_expression) => {
                hir_enum_constructor_expression
                    .arguments
                    .iter()
                    .any(|expr_id| self.expression_id_has_unsafe(*expr_id))
            }
            HirExpression::MemberAccess(hir_member_access) => {
                self.expression_id_has_unsafe(hir_member_access.lhs)
            }
            HirExpression::Call(hir_call_expression) => {
                self.expression_id_has_unsafe(hir_call_expression.func)
                    || hir_call_expression
                        .arguments
                        .iter()
                        .any(|expr_id| self.expression_id_has_unsafe(*expr_id))
            }
            HirExpression::Constrain(hir_constrain_expression) => {
                self.expression_id_has_unsafe(hir_constrain_expression.0)
                    || hir_constrain_expression
                        .2
                        .is_some_and(|expr_id| self.expression_id_has_unsafe(expr_id))
            }
            HirExpression::Cast(hir_cast_expression) => {
                self.expression_id_has_unsafe(hir_cast_expression.lhs)
            }
            HirExpression::If(hir_if_expression) => {
                self.expression_id_has_unsafe(hir_if_expression.condition)
                    || self.expression_id_has_unsafe(hir_if_expression.consequence)
                    || hir_if_expression
                        .alternative
                        .is_some_and(|expr_id| self.expression_id_has_unsafe(expr_id))
            }
            HirExpression::Match(hir_match) => self.hir_match_has_unsafe(&hir_match),
            HirExpression::Tuple(expr_ids) => {
                expr_ids.iter().any(|expr_id| self.expression_id_has_unsafe(*expr_id))
            }
            HirExpression::Lambda(hir_lambda) => self.expression_id_has_unsafe(hir_lambda.body),
            HirExpression::Quote(..) | HirExpression::Unquote(..) => false,
            HirExpression::Unsafe(..) => true,
            HirExpression::Error => false,
        }
    }

    fn hir_match_has_unsafe(&self, hir_match: &HirMatch) -> bool {
        match hir_match {
            HirMatch::Success(expr_id) => self.expression_id_has_unsafe(*expr_id),
            HirMatch::Failure { .. } => false,
            HirMatch::Guard { cond, body, otherwise } => {
                self.expression_id_has_unsafe(*cond)
                    || self.expression_id_has_unsafe(*body)
                    || self.hir_match_has_unsafe(otherwise)
            }
            HirMatch::Switch(_, cases, hir_match) => {
                cases.iter().any(|case| self.hir_match_has_unsafe(&case.body))
                    || hir_match
                        .as_ref()
                        .is_some_and(|hir_match| self.hir_match_has_unsafe(hir_match))
            }
        }
    }
}

fn hir_expression_needs_parentheses(hir_expr: &HirExpression) -> bool {
    match hir_expr {
        HirExpression::Infix(..) | HirExpression::Cast(..) | HirExpression::Lambda(..) => true,
        HirExpression::Ident(..)
        | HirExpression::Literal(..)
        | HirExpression::Block(..)
        | HirExpression::Prefix(..)
        | HirExpression::Index(..)
        | HirExpression::Constructor(..)
        | HirExpression::EnumConstructor(..)
        | HirExpression::MemberAccess(..)
        | HirExpression::Call(..)
        | HirExpression::Constrain(..)
        | HirExpression::If(..)
        | HirExpression::Match(..)
        | HirExpression::Tuple(..)
        | HirExpression::Quote(..)
        | HirExpression::Unquote(..)
        | HirExpression::Unsafe(..)
        | HirExpression::Error => false,
    }
}

fn get_type_fields(typ: &Type) -> Option<Vec<(String, Type, ItemVisibility)>> {
    match typ.follow_bindings() {
        Type::DataType(data_type, generics) => {
            let data_type = data_type.borrow();
            data_type.get_fields(&generics)
        }
        _ => None,
    }
}
