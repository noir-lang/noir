use std::collections::BTreeMap;

use fxhash::FxHashMap as HashMap;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::Location;

use crate::{
    ast::{
        ConstructorExpression, EnumVariant, Expression, ExpressionKind, FunctionKind, Ident,
        Literal, NoirEnumeration, StatementKind, UnresolvedType, Visibility,
    },
    elaborator::path_resolution::PathResolutionItem,
    hir::{comptime::Value, resolution::errors::ResolverError, type_check::TypeCheckError},
    hir_def::{
        expr::{
            Case, Constructor, HirBlockExpression, HirEnumConstructorExpression, HirExpression,
            HirIdent, HirMatch,
        },
        function::{FuncMeta, FunctionBody, HirFunction, Parameters},
        stmt::{HirLetStatement, HirPattern, HirStatement},
    },
    node_interner::{DefinitionId, DefinitionKind, ExprId, FunctionModifiers, GlobalValue, TypeId},
    signed_field::SignedField,
    token::Attributes,
    DataType, Kind, Shared, Type,
};

use super::Elaborator;

impl Elaborator<'_> {
    /// Defines the value of an enum variant that we resolve an enum
    /// variant expression to. E.g. `Foo::Bar` in `Foo::Bar(baz)`.
    ///
    /// If the variant requires arguments we should define a function,
    /// otherwise we define a polymorphic global containing the tag value.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn define_enum_variant_constructor(
        &mut self,
        enum_: &NoirEnumeration,
        type_id: TypeId,
        variant: &EnumVariant,
        variant_arg_types: Option<Vec<Type>>,
        variant_index: usize,
        datatype: &Shared<DataType>,
        self_type: &Type,
        self_type_unresolved: UnresolvedType,
    ) {
        match variant_arg_types {
            Some(args) => self.define_enum_variant_function(
                enum_,
                type_id,
                variant,
                args,
                variant_index,
                datatype,
                self_type,
                self_type_unresolved,
            ),
            None => self.define_enum_variant_global(
                enum_,
                type_id,
                variant,
                variant_index,
                datatype,
                self_type,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn define_enum_variant_global(
        &mut self,
        enum_: &NoirEnumeration,
        type_id: TypeId,
        variant: &EnumVariant,
        variant_index: usize,
        datatype: &Shared<DataType>,
        self_type: &Type,
    ) {
        let name = &variant.name;
        let location = variant.name.location();

        let global_id = self.interner.push_empty_global(
            name.clone(),
            type_id.local_module_id(),
            type_id.krate(),
            name.location().file,
            Vec::new(),
            false,
            false,
        );

        let mut typ = self_type.clone();
        if !datatype.borrow().generics.is_empty() {
            let typevars = vecmap(&datatype.borrow().generics, |generic| generic.type_var.clone());
            typ = Type::Forall(typevars, Box::new(typ));
        }

        let definition_id = self.interner.get_global(global_id).definition_id;
        self.interner.push_definition_type(definition_id, typ.clone());

        let no_parameters = Parameters(Vec::new());
        let global_body =
            self.make_enum_variant_constructor(datatype, variant_index, &no_parameters, location);
        let let_statement = crate::hir_def::stmt::HirStatement::Expression(global_body);

        let statement_id = self.interner.get_global(global_id).let_statement;
        self.interner.replace_statement(statement_id, let_statement);

        self.interner.get_global_mut(global_id).value = GlobalValue::Resolved(
            crate::hir::comptime::Value::Enum(variant_index, Vec::new(), typ),
        );

        Self::get_module_mut(self.def_maps, type_id.module_id())
            .declare_global(name.clone(), enum_.visibility, global_id)
            .ok();
    }

    #[allow(clippy::too_many_arguments)]
    fn define_enum_variant_function(
        &mut self,
        enum_: &NoirEnumeration,
        type_id: TypeId,
        variant: &EnumVariant,
        variant_arg_types: Vec<Type>,
        variant_index: usize,
        datatype: &Shared<DataType>,
        self_type: &Type,
        self_type_unresolved: UnresolvedType,
    ) {
        let name_string = variant.name.to_string();
        let datatype_ref = datatype.borrow();
        let location = variant.name.location();

        let id = self.interner.push_empty_fn();

        let modifiers = FunctionModifiers {
            name: name_string.clone(),
            visibility: enum_.visibility,
            attributes: Attributes { function: None, secondary: Vec::new() },
            is_unconstrained: false,
            generic_count: datatype_ref.generics.len(),
            is_comptime: false,
            name_location: location,
        };
        let definition_id =
            self.interner.push_function_definition(id, modifiers, type_id.module_id(), location);

        let hir_name = HirIdent::non_trait_method(definition_id, location);
        let parameters = self.make_enum_variant_parameters(variant_arg_types, location);

        let body =
            self.make_enum_variant_constructor(datatype, variant_index, &parameters, location);
        self.interner.update_fn(id, HirFunction::unchecked_from_expr(body));

        let function_type =
            datatype_ref.variant_function_type_with_forall(variant_index, datatype.clone());
        self.interner.push_definition_type(definition_id, function_type.clone());

        let meta = FuncMeta {
            name: hir_name,
            kind: FunctionKind::Normal,
            parameters,
            parameter_idents: Vec::new(),
            return_type: crate::ast::FunctionReturnType::Ty(self_type_unresolved),
            return_visibility: Visibility::Private,
            typ: function_type,
            direct_generics: datatype_ref.generics.clone(),
            all_generics: datatype_ref.generics.clone(),
            location,
            has_body: false,
            trait_constraints: Vec::new(),
            type_id: Some(type_id),
            trait_id: None,
            trait_impl: None,
            enum_variant_index: Some(variant_index),
            is_entry_point: false,
            has_inline_attribute: false,
            function_body: FunctionBody::Resolved,
            source_crate: self.crate_id,
            source_module: type_id.local_module_id(),
            source_file: variant.name.location().file,
            self_type: None,
        };

        self.interner.push_fn_meta(meta, id);
        self.interner.add_method(self_type, name_string, id, None);

        let name = variant.name.clone();
        Self::get_module_mut(self.def_maps, type_id.module_id())
            .declare_function(name, enum_.visibility, id)
            .ok();
    }

    // Given:
    // ```
    // enum FooEnum { Foo(u32, u8), ... }
    //
    // fn Foo(a: u32, b: u8) -> FooEnum {}
    // ```
    // Create (pseudocode):
    // ```
    // fn Foo(a: u32, b: u8) -> FooEnum {
    //     // This can't actually be written directly in Noir
    //     FooEnum {
    //         tag: Foo_tag,
    //         Foo: (a, b),
    //         // fields from other variants are zeroed in monomorphization
    //     }
    // }
    // ```
    fn make_enum_variant_constructor(
        &mut self,
        self_type: &Shared<DataType>,
        variant_index: usize,
        parameters: &Parameters,
        location: Location,
    ) -> ExprId {
        // Each parameter of the enum variant function is used as a parameter of the enum
        // constructor expression
        let arguments = vecmap(&parameters.0, |(pattern, typ, _)| match pattern {
            HirPattern::Identifier(ident) => {
                let id = self.interner.push_expr(HirExpression::Ident(ident.clone(), None));
                self.interner.push_expr_type(id, typ.clone());
                self.interner.push_expr_location(id, location);
                id
            }
            _ => unreachable!(),
        });

        let constructor = HirExpression::EnumConstructor(HirEnumConstructorExpression {
            r#type: self_type.clone(),
            arguments,
            variant_index,
        });

        let body = self.interner.push_expr(constructor);
        let enum_generics = self_type.borrow().generic_types();
        let typ = Type::DataType(self_type.clone(), enum_generics);
        self.interner.push_expr_type(body, typ);
        self.interner.push_expr_location(body, location);
        body
    }

    fn make_enum_variant_parameters(
        &mut self,
        parameter_types: Vec<Type>,
        location: Location,
    ) -> Parameters {
        Parameters(vecmap(parameter_types.into_iter().enumerate(), |(i, parameter_type)| {
            let name = format!("${i}");
            let parameter = DefinitionKind::Local(None);
            let id = self.interner.push_definition(name, false, false, parameter, location);
            let pattern = HirPattern::Identifier(HirIdent::non_trait_method(id, location));
            (pattern, parameter_type, Visibility::Private)
        }))
    }

    /// To elaborate the rules of a match we need to go through the pattern first to define all
    /// the variables within, then compile the corresponding branch. For each branch we do this
    /// way we'll need to keep a distinct scope so that branches cannot access the pattern
    /// variables from other branches.
    ///
    /// Returns (rows, result type) where rows is a pattern matrix used to compile the
    /// match into a decision tree.
    pub(super) fn elaborate_match_rules(
        &mut self,
        variable_to_match: DefinitionId,
        rules: Vec<(Expression, Expression)>,
    ) -> (Vec<Row>, Type) {
        let result_type = self.interner.next_type_variable();
        let expected_pattern_type = self.interner.definition_type(variable_to_match);

        let rows = vecmap(rules, |(pattern, branch)| {
            self.push_scope();
            let pattern =
                self.expression_to_pattern(pattern, &expected_pattern_type, &mut Vec::new());
            let columns = vec![Column::new(variable_to_match, pattern)];

            let guard = None;
            let body_location = branch.type_location();
            let (body, body_type) = self.elaborate_expression(branch);

            self.unify(&body_type, &result_type, || TypeCheckError::TypeMismatch {
                expected_typ: result_type.to_string(),
                expr_typ: body_type.to_string(),
                expr_location: body_location,
            });

            self.pop_scope();
            Row::new(columns, guard, body)
        });
        (rows, result_type)
    }

    /// Convert an expression into a Pattern, defining any variables within.
    fn expression_to_pattern(
        &mut self,
        expression: Expression,
        expected_type: &Type,
        variables_defined: &mut Vec<Ident>,
    ) -> Pattern {
        let expr_location = expression.type_location();
        let unify_with_expected_type = |this: &mut Self, actual| {
            this.unify(actual, expected_type, || TypeCheckError::TypeMismatch {
                expected_typ: expected_type.to_string(),
                expr_typ: actual.to_string(),
                expr_location,
            });
        };

        // We want the actual expression's location here, not the innermost one from `type_location()`
        let syntax_error = |this: &mut Self| {
            let errors = ResolverError::InvalidSyntaxInPattern { location: expression.location };
            this.push_err(errors);
            Pattern::Error
        };

        match expression.kind {
            ExpressionKind::Literal(Literal::Integer(value)) => {
                let actual = self.interner.next_type_variable_with_kind(Kind::IntegerOrField);
                unify_with_expected_type(self, &actual);
                Pattern::Int(value)
            }
            ExpressionKind::Literal(Literal::Bool(value)) => {
                unify_with_expected_type(self, &Type::Bool);
                let constructor = if value { Constructor::True } else { Constructor::False };
                Pattern::Constructor(constructor, Vec::new())
            }
            ExpressionKind::Variable(path) => {
                // A variable can be free or bound if it refers to an enum constant:
                // - in `(a, b)`, both variables may be free and should be defined, or
                //   may refer to an enum variant named `a` or `b` in scope.
                // - Possible diagnostics improvement: warn if `a` is defined as a variable
                //   when there is a matching enum variant with name `Foo::a` which can
                //   be imported. The user likely intended to reference the enum variant.
                let path_len = path.segments.len();
                let location = path.location;
                let last_ident = path.last_ident();

                match self.resolve_path_or_error(path) {
                    Ok(resolution) => self.path_resolution_to_constructor(
                        resolution,
                        Vec::new(),
                        expected_type,
                        location,
                        variables_defined,
                    ),
                    Err(_) if path_len == 1 => {
                        // Define the variable
                        let kind = DefinitionKind::Local(None);

                        if let Some(existing) =
                            variables_defined.iter().find(|elem| *elem == &last_ident)
                        {
                            // Allow redefinition of `_` only, to ignore variables
                            if last_ident.0.contents != "_" {
                                let error = ResolverError::VariableAlreadyDefinedInPattern {
                                    existing: existing.clone(),
                                    new_location: last_ident.location(),
                                };
                                self.push_err(error);
                            }
                        } else {
                            variables_defined.push(last_ident.clone());
                        }

                        let id = self.add_variable_decl(last_ident, false, true, true, kind).id;
                        self.interner.push_definition_type(id, expected_type.clone());
                        Pattern::Binding(id)
                    }
                    Err(error) => {
                        self.push_err(error);
                        Pattern::Error
                    }
                }
            }
            ExpressionKind::Call(call) => self.expression_to_constructor(
                *call.func,
                call.arguments,
                expected_type,
                variables_defined,
            ),
            ExpressionKind::Constructor(constructor) => {
                self.constructor_to_pattern(*constructor, variables_defined)
            }
            ExpressionKind::Tuple(fields) => {
                let field_types = vecmap(0..fields.len(), |_| self.interner.next_type_variable());
                let actual = Type::Tuple(field_types.clone());
                unify_with_expected_type(self, &actual);

                let fields = vecmap(fields.into_iter().enumerate(), |(i, field)| {
                    let expected = field_types.get(i).unwrap_or(&Type::Error);
                    self.expression_to_pattern(field, expected, variables_defined)
                });

                Pattern::Constructor(Constructor::Tuple(field_types.clone()), fields)
            }

            ExpressionKind::Parenthesized(expr) => {
                self.expression_to_pattern(*expr, expected_type, variables_defined)
            }
            ExpressionKind::Interned(id) => {
                let kind = self.interner.get_expression_kind(id);
                let expr = Expression::new(kind.clone(), expression.location);
                self.expression_to_pattern(expr, expected_type, variables_defined)
            }
            ExpressionKind::InternedStatement(id) => {
                if let StatementKind::Expression(expr) = self.interner.get_statement_kind(id) {
                    self.expression_to_pattern(expr.clone(), expected_type, variables_defined)
                } else {
                    syntax_error(self)
                }
            }

            ExpressionKind::Literal(_)
            | ExpressionKind::Block(_)
            | ExpressionKind::Prefix(_)
            | ExpressionKind::Index(_)
            | ExpressionKind::MethodCall(_)
            | ExpressionKind::MemberAccess(_)
            | ExpressionKind::Cast(_)
            | ExpressionKind::Infix(_)
            | ExpressionKind::If(_)
            | ExpressionKind::Match(_)
            | ExpressionKind::Constrain(_)
            | ExpressionKind::Lambda(_)
            | ExpressionKind::Quote(_)
            | ExpressionKind::Unquote(_)
            | ExpressionKind::Comptime(_, _)
            | ExpressionKind::Unsafe(_)
            | ExpressionKind::AsTraitPath(_)
            | ExpressionKind::TypePath(_)
            | ExpressionKind::Resolved(_)
            | ExpressionKind::Error => syntax_error(self),
        }
    }

    fn constructor_to_pattern(
        &mut self,
        constructor: ConstructorExpression,
        variables_defined: &mut Vec<Ident>,
    ) -> Pattern {
        let location = constructor.typ.location;
        let typ = self.resolve_type(constructor.typ);

        let Some((struct_name, mut expected_field_types)) =
            self.struct_name_and_field_types(&typ, location)
        else {
            return Pattern::Error;
        };

        let mut fields = BTreeMap::default();
        for (field_name, field) in constructor.fields {
            let Some(field_index) =
                expected_field_types.iter().position(|(name, _)| *name == field_name.0.contents)
            else {
                let error = if fields.contains_key(&field_name.0.contents) {
                    ResolverError::DuplicateField { field: field_name }
                } else {
                    let struct_definition = struct_name.clone();
                    ResolverError::NoSuchField { field: field_name, struct_definition }
                };
                self.push_err(error);
                continue;
            };

            let (field_name, expected_field_type) = expected_field_types.swap_remove(field_index);
            let pattern =
                self.expression_to_pattern(field, &expected_field_type, variables_defined);
            fields.insert(field_name, pattern);
        }

        if !expected_field_types.is_empty() {
            let struct_definition = struct_name;
            let missing_fields = vecmap(expected_field_types, |(name, _)| name);
            let error =
                ResolverError::MissingFields { location, missing_fields, struct_definition };
            self.push_err(error);
        }

        let args = vecmap(fields, |(_name, field)| field);
        Pattern::Constructor(Constructor::Variant(typ, 0), args)
    }

    fn expression_to_constructor(
        &mut self,
        name: Expression,
        args: Vec<Expression>,
        expected_type: &Type,
        variables_defined: &mut Vec<Ident>,
    ) -> Pattern {
        match name.kind {
            ExpressionKind::Variable(path) => {
                let location = path.location;

                match self.resolve_path_or_error(path) {
                    Ok(resolution) => self.path_resolution_to_constructor(
                        resolution,
                        args,
                        expected_type,
                        location,
                        variables_defined,
                    ),
                    Err(error) => {
                        self.push_err(error);
                        let id = self.fresh_match_variable(expected_type.clone(), location);
                        Pattern::Binding(id)
                    }
                }
            }
            ExpressionKind::Parenthesized(expr) => {
                self.expression_to_constructor(*expr, args, expected_type, variables_defined)
            }
            ExpressionKind::Interned(id) => {
                let kind = self.interner.get_expression_kind(id);
                let expr = Expression::new(kind.clone(), name.location);
                self.expression_to_constructor(expr, args, expected_type, variables_defined)
            }
            ExpressionKind::InternedStatement(id) => {
                if let StatementKind::Expression(expr) = self.interner.get_statement_kind(id) {
                    self.expression_to_constructor(
                        expr.clone(),
                        args,
                        expected_type,
                        variables_defined,
                    )
                } else {
                    panic!("Invalid expr kind {name}")
                }
            }
            other => todo!("invalid constructor `{other}`"),
        }
    }

    fn path_resolution_to_constructor(
        &mut self,
        name: PathResolutionItem,
        args: Vec<Expression>,
        expected_type: &Type,
        location: Location,
        variables_defined: &mut Vec<Ident>,
    ) -> Pattern {
        let (actual_type, expected_arg_types, variant_index) = match name {
            PathResolutionItem::Global(id) => {
                // variant constant
                let global = self.interner.get_global(id);
                let variant_index = match global.value {
                    GlobalValue::Resolved(Value::Enum(tag, ..)) => tag,
                    _ => todo!("Value is not an enum constant"),
                };

                let global_type = self.interner.definition_type(global.definition_id);
                let actual_type = global_type.instantiate(self.interner).0;
                (actual_type, Vec::new(), variant_index)
            }
            PathResolutionItem::Method(_type_id, _type_turbofish, func_id) => {
                // TODO(#7430): Take type_turbofish into account when instantiating the function's type
                let meta = self.interner.function_meta(&func_id);
                let Some(variant_index) = meta.enum_variant_index else { todo!("not a variant") };

                let (actual_type, expected_arg_types) = match meta.typ.instantiate(self.interner).0
                {
                    Type::Function(args, ret, _env, _) => (*ret, args),
                    other => unreachable!("Not a function! Found {other}"),
                };

                (actual_type, expected_arg_types, variant_index)
            }
            PathResolutionItem::Module(_) => todo!("path_resolution_to_constructor {name:?}"),
            PathResolutionItem::Type(_) => todo!("path_resolution_to_constructor {name:?}"),
            PathResolutionItem::TypeAlias(_) => todo!("path_resolution_to_constructor {name:?}"),
            PathResolutionItem::Trait(_) => todo!("path_resolution_to_constructor {name:?}"),
            PathResolutionItem::ModuleFunction(_) => {
                todo!("path_resolution_to_constructor {name:?}")
            }
            PathResolutionItem::TypeAliasFunction(_, _, _) => {
                todo!("path_resolution_to_constructor {name:?}")
            }
            PathResolutionItem::TraitFunction(_, _, _) => {
                todo!("path_resolution_to_constructor {name:?}")
            }
        };

        // We must unify the actual type before `expected_arg_types` are used since those
        // are instantiated and rely on this already being unified.
        self.unify(&actual_type, expected_type, || TypeCheckError::TypeMismatch {
            expected_typ: expected_type.to_string(),
            expr_typ: actual_type.to_string(),
            expr_location: location,
        });

        if args.len() != expected_arg_types.len() {
            // error expected N args, found M?
        }

        let args = args.into_iter().zip(expected_arg_types);
        let args = vecmap(args, |(arg, expected_arg_type)| {
            self.expression_to_pattern(arg, &expected_arg_type, variables_defined)
        });
        let constructor = Constructor::Variant(actual_type, variant_index);
        Pattern::Constructor(constructor, args)
    }

    fn struct_name_and_field_types(
        &mut self,
        typ: &Type,
        location: Location,
    ) -> Option<(Ident, Vec<(String, Type)>)> {
        if let Type::DataType(typ, generics) = typ.follow_bindings_shallow().as_ref() {
            if let Some(fields) = typ.borrow().get_fields(generics) {
                return Some((typ.borrow().name.clone(), fields));
            }
        }

        let error = ResolverError::NonStructUsedInConstructor { typ: typ.to_string(), location };
        self.push_err(error);
        None
    }

    /// Compiles the rows of a match expression, outputting a decision tree for the match.
    ///
    /// This is an adaptation of https://github.com/yorickpeterse/pattern-matching-in-rust/tree/main/jacobs2021
    /// which is an implementation of https://julesjacobs.com/notes/patternmatching/patternmatching.pdf
    pub(super) fn elaborate_match_rows(&mut self, rows: Vec<Row>) -> HirMatch {
        self.compile_rows(rows).unwrap_or_else(|error| {
            self.push_err(error);
            HirMatch::Failure
        })
    }

    fn compile_rows(&mut self, mut rows: Vec<Row>) -> Result<HirMatch, ResolverError> {
        if rows.is_empty() {
            eprintln!("Warning: missing case");
            return Ok(HirMatch::Failure);
        }

        self.push_tests_against_bare_variables(&mut rows);

        // If the first row is a match-all we match it and the remaining rows are ignored.
        if rows.first().is_some_and(|row| row.columns.is_empty()) {
            let row = rows.remove(0);

            return Ok(match row.guard {
                None => HirMatch::Success(row.body),
                Some(cond) => {
                    let remaining = self.compile_rows(rows)?;
                    HirMatch::Guard { cond, body: row.body, otherwise: Box::new(remaining) }
                }
            });
        }

        let branch_var = self.branch_variable(&rows);
        let location = self.interner.definition(branch_var).location;

        match self.interner.definition_type(branch_var).follow_bindings_shallow().into_owned() {
            Type::FieldElement | Type::Integer(_, _) => {
                let (cases, fallback) = self.compile_int_cases(rows, branch_var)?;
                Ok(HirMatch::Switch(branch_var, cases, Some(fallback)))
            }
            Type::TypeVariable(typevar) if typevar.is_integer_or_field() => {
                let (cases, fallback) = self.compile_int_cases(rows, branch_var)?;
                Ok(HirMatch::Switch(branch_var, cases, Some(fallback)))
            }

            Type::Array(_, _) => todo!(),
            Type::Slice(_) => todo!(),
            Type::Bool => {
                let cases = vec![
                    (Constructor::False, Vec::new(), Vec::new()),
                    (Constructor::True, Vec::new(), Vec::new()),
                ];

                let (cases, fallback) = self.compile_constructor_cases(rows, branch_var, cases)?;
                Ok(HirMatch::Switch(branch_var, cases, fallback))
            }
            Type::Unit => {
                let cases = vec![(Constructor::Unit, Vec::new(), Vec::new())];
                let (cases, fallback) = self.compile_constructor_cases(rows, branch_var, cases)?;
                Ok(HirMatch::Switch(branch_var, cases, fallback))
            }
            Type::Tuple(fields) => {
                let field_variables = self.fresh_match_variables(fields.clone(), location);
                let cases = vec![(Constructor::Tuple(fields), field_variables, Vec::new())];
                let (cases, fallback) = self.compile_constructor_cases(rows, branch_var, cases)?;
                Ok(HirMatch::Switch(branch_var, cases, fallback))
            }
            Type::DataType(type_def, generics) => {
                let def = type_def.borrow();
                if let Some(variants) = def.get_variants(&generics) {
                    drop(def);
                    let typ = Type::DataType(type_def, generics);

                    let cases = vecmap(variants.iter().enumerate(), |(idx, (_name, args))| {
                        let constructor = Constructor::Variant(typ.clone(), idx);
                        let args = self.fresh_match_variables(args.clone(), location);
                        (constructor, args, Vec::new())
                    });

                    let (cases, fallback) =
                        self.compile_constructor_cases(rows, branch_var, cases)?;
                    Ok(HirMatch::Switch(branch_var, cases, fallback))
                } else if let Some(fields) = def.get_fields(&generics) {
                    drop(def);
                    let typ = Type::DataType(type_def, generics);

                    // Just treat structs as a single-variant type
                    let fields = vecmap(fields, |(_name, typ)| typ);
                    let constructor = Constructor::Variant(typ, 0);
                    let field_variables = self.fresh_match_variables(fields, location);
                    let cases = vec![(constructor, field_variables, Vec::new())];
                    let (cases, fallback) =
                        self.compile_constructor_cases(rows, branch_var, cases)?;
                    Ok(HirMatch::Switch(branch_var, cases, fallback))
                } else {
                    drop(def);
                    let typ = Type::DataType(type_def, generics);
                    todo!("Cannot match on type {typ}")
                }
            }
            typ @ (Type::Alias(_, _)
            | Type::TypeVariable(_)
            | Type::String(_)
            | Type::FmtString(_, _)
            | Type::TraitAsType(_, _, _)
            | Type::NamedGeneric(_, _)
            | Type::CheckedCast { .. }
            | Type::Function(_, _, _, _)
            | Type::MutableReference(_)
            | Type::Forall(_, _)
            | Type::Constant(_, _)
            | Type::Quoted(_)
            | Type::InfixExpr(_, _, _, _)
            | Type::Error) => todo!("Cannot match on type {typ:?}"),
        }
    }

    fn fresh_match_variables(
        &mut self,
        variable_types: Vec<Type>,
        location: Location,
    ) -> Vec<DefinitionId> {
        vecmap(variable_types, |typ| self.fresh_match_variable(typ, location))
    }

    fn fresh_match_variable(&mut self, variable_type: Type, location: Location) -> DefinitionId {
        let name = "internal_match_variable".to_string();
        let kind = DefinitionKind::Local(None);
        let id = self.interner.push_definition(name, false, false, kind, location);
        self.interner.push_definition_type(id, variable_type);
        id
    }

    /// Compiles the cases and fallback cases for integer and range patterns.
    ///
    /// Integers have an infinite number of constructors, so we specialise the
    /// compilation of integer and range patterns.
    fn compile_int_cases(
        &mut self,
        rows: Vec<Row>,
        branch_var: DefinitionId,
    ) -> Result<(Vec<Case>, Box<HirMatch>), ResolverError> {
        let mut raw_cases: Vec<(Constructor, Vec<DefinitionId>, Vec<Row>)> = Vec::new();
        let mut fallback_rows = Vec::new();
        let mut tested: HashMap<(SignedField, SignedField), usize> = HashMap::default();

        for mut row in rows {
            if let Some(col) = row.remove_column(branch_var) {
                let (key, cons) = match col.pattern {
                    Pattern::Int(val) => ((val, val), Constructor::Int(val)),
                    Pattern::Range(start, stop) => ((start, stop), Constructor::Range(start, stop)),
                    Pattern::Error => continue,
                    pattern => {
                        eprintln!("Unexpected pattern for integer type: {pattern:?}");
                        continue;
                    }
                };

                if let Some(index) = tested.get(&key) {
                    raw_cases[*index].2.push(row);
                    continue;
                }

                tested.insert(key, raw_cases.len());

                let mut rows = fallback_rows.clone();

                rows.push(row);
                raw_cases.push((cons, Vec::new(), rows));
            } else {
                for (_, _, rows) in &mut raw_cases {
                    rows.push(row.clone());
                }

                fallback_rows.push(row);
            }
        }

        let cases = try_vecmap(raw_cases, |(cons, vars, rows)| {
            let rows = self.compile_rows(rows)?;
            Ok::<_, ResolverError>(Case::new(cons, vars, rows))
        })?;

        Ok((cases, Box::new(self.compile_rows(fallback_rows)?)))
    }

    /// Compiles the cases and sub cases for the constructor located at the
    /// column of the branching variable.
    ///
    /// What exactly this method does may be a bit hard to understand from the
    /// code, as there's simply quite a bit going on. Roughly speaking, it does
    /// the following:
    ///
    /// 1. It takes the column we're branching on (based on the branching
    ///    variable) and removes it from every row.
    /// 2. We add additional columns to this row, if the constructor takes any
    ///    arguments (which we'll handle in a nested match).
    /// 3. We turn the resulting list of rows into a list of cases, then compile
    ///    those into decision (sub) trees.
    ///
    /// If a row didn't include the branching variable, we simply copy that row
    /// into the list of rows for every constructor to test.
    ///
    /// For this to work, the `cases` variable must be prepared such that it has
    /// a triple for every constructor we need to handle. For an ADT with 10
    /// constructors, that means 10 triples. This is needed so this method can
    /// assign the correct sub matches to these constructors.
    ///
    /// Types with infinite constructors (e.g. int and string) are handled
    /// separately; they don't need most of this work anyway.
    #[allow(clippy::type_complexity)]
    fn compile_constructor_cases(
        &mut self,
        rows: Vec<Row>,
        branch_var: DefinitionId,
        mut cases: Vec<(Constructor, Vec<DefinitionId>, Vec<Row>)>,
    ) -> Result<(Vec<Case>, Option<Box<HirMatch>>), ResolverError> {
        for mut row in rows {
            if let Some(col) = row.remove_column(branch_var) {
                if let Pattern::Constructor(cons, args) = col.pattern {
                    let idx = cons.variant_index();
                    let mut cols = row.columns;

                    for (var, pat) in cases[idx].1.iter().zip(args.into_iter()) {
                        cols.push(Column::new(*var, pat));
                    }

                    cases[idx].2.push(Row::new(cols, row.guard, row.body));
                }
            } else {
                for (_, _, rows) in &mut cases {
                    rows.push(row.clone());
                }
            }
        }

        let cases = try_vecmap(cases, |(cons, vars, rows)| {
            let rows = self.compile_rows(rows)?;
            Ok::<_, ResolverError>(Case::new(cons, vars, rows))
        })?;

        Ok(Self::deduplicate_cases(cases))
    }

    /// Move any cases with duplicate branches into a shared 'else' branch
    fn deduplicate_cases(mut cases: Vec<Case>) -> (Vec<Case>, Option<Box<HirMatch>>) {
        let mut else_case = None;
        let mut ending_cases = Vec::with_capacity(cases.len());
        let mut previous_case: Option<Case> = None;

        // Go through each of the cases, looking for duplicates.
        // This is simplified such that the first (consecutive) duplicates
        // we find we move to an else case. Each case afterward is then compared
        // to the else case. This could be improved in a couple ways:
        // - Instead of the the first consecutive duplicates we find, we could
        //   expand the check to find non-consecutive duplicates as well.
        // - We should also ideally move the most duplicated case to the else
        //   case, not just the first duplicated case we find. I suspect in most
        //   actual code snippets these are the same but it could still be nice to guarantee.
        while let Some(case) = cases.pop() {
            if let Some(else_case) = &else_case {
                if case.body == *else_case {
                    // Delete the current case by not pushing it to `ending_cases`
                    continue;
                } else {
                    ending_cases.push(case);
                }
            } else if let Some(previous) = previous_case {
                if case.body == previous.body {
                    // else_case is known to be None here
                    else_case = Some(previous.body);

                    // Delete both previous_case and case
                    previous_case = None;
                    continue;
                } else {
                    previous_case = Some(case);
                    ending_cases.push(previous);
                }
            } else {
                previous_case = Some(case);
            }
        }

        if let Some(case) = previous_case {
            ending_cases.push(case);
        }

        ending_cases.reverse();
        (ending_cases, else_case.map(Box::new))
    }

    /// Return the variable that was referred to the most in `rows`
    fn branch_variable(&mut self, rows: &[Row]) -> DefinitionId {
        let mut counts = HashMap::default();

        for row in rows {
            for col in &row.columns {
                *counts.entry(&col.variable_to_match).or_insert(0_usize) += 1;
            }
        }

        rows[0]
            .columns
            .iter()
            .map(|col| col.variable_to_match)
            .max_by_key(|var| counts[var])
            .unwrap()
    }

    fn push_tests_against_bare_variables(&mut self, rows: &mut Vec<Row>) {
        for row in rows {
            row.columns.retain(|col| {
                if let Pattern::Binding(variable) = col.pattern {
                    row.body = self.let_binding(variable, col.variable_to_match, row.body);
                    false
                } else {
                    true
                }
            });
        }
    }

    /// Creates:
    /// `{ let <variable> = <rhs>; <body> }`
    fn let_binding(&mut self, variable: DefinitionId, rhs: DefinitionId, body: ExprId) -> ExprId {
        let location = self.interner.definition(rhs).location;

        let r#type = self.interner.definition_type(variable);
        let rhs_type = self.interner.definition_type(rhs);
        let variable = HirIdent::non_trait_method(variable, location);

        let rhs = HirExpression::Ident(HirIdent::non_trait_method(rhs, location), None);
        let rhs = self.interner.push_expr(rhs);
        self.interner.push_expr_type(rhs, rhs_type);
        self.interner.push_expr_location(rhs, location);

        let let_ = HirStatement::Let(HirLetStatement {
            pattern: HirPattern::Identifier(variable),
            r#type,
            expression: rhs,
            attributes: Vec::new(),
            comptime: false,
            is_global_let: false,
        });

        let body_type = self.interner.id_type(body);
        let let_ = self.interner.push_stmt(let_);
        let body = self.interner.push_stmt(HirStatement::Expression(body));

        self.interner.push_stmt_location(let_, location);
        self.interner.push_stmt_location(body, location);

        let block = HirExpression::Block(HirBlockExpression { statements: vec![let_, body] });
        let block = self.interner.push_expr(block);
        self.interner.push_expr_type(block, body_type);
        self.interner.push_expr_location(block, location);
        block
    }
}

/// A Pattern is anything that can appear before the `=>` in a match rule.
#[derive(Debug, Clone)]
enum Pattern {
    /// A pattern checking for a tag and possibly binding variables such as `Some(42)`
    Constructor(Constructor, Vec<Pattern>),
    /// An integer literal pattern such as `4`, `12345`, or `-56`
    Int(SignedField),
    /// A pattern binding a variable such as `a` or `_`
    Binding(DefinitionId),

    /// Multiple patterns combined with `|` where we should match this pattern if any
    /// constituent pattern matches. e.g. `Some(3) | None` or `Some(1) | Some(2) | None`
    #[allow(unused)]
    Or(Vec<Pattern>),

    /// An integer range pattern such as `1..20` which will match any integer n such that
    /// 1 <= n < 20.
    #[allow(unused)]
    Range(SignedField, SignedField),

    /// An error occurred while translating this pattern. This Pattern kind always translates
    /// to a Fail branch in the decision tree, although the compiler is expected to halt
    /// with errors before execution.
    Error,
}

#[derive(Clone)]
struct Column {
    variable_to_match: DefinitionId,
    pattern: Pattern,
}

impl Column {
    fn new(variable_to_match: DefinitionId, pattern: Pattern) -> Self {
        Column { variable_to_match, pattern }
    }
}

#[derive(Clone)]
pub(super) struct Row {
    columns: Vec<Column>,
    guard: Option<ExprId>,
    body: ExprId,
}

impl Row {
    fn new(columns: Vec<Column>, guard: Option<ExprId>, body: ExprId) -> Row {
        Row { columns, guard, body }
    }
}

impl Row {
    fn remove_column(&mut self, variable: DefinitionId) -> Option<Column> {
        self.columns
            .iter()
            .position(|c| c.variable_to_match == variable)
            .map(|idx| self.columns.remove(idx))
    }
}
