//! Enum definition collection and variant resolution.

use std::collections::{BTreeMap, BTreeSet};

use iter_extended::{btree_map, try_vecmap, vecmap};
use noirc_errors::Location;
use rangemap::StepLite;
use rustc_hash::FxHashMap as HashMap;

use crate::{
    DataType, EnumVariant as HirEnumVariant, Kind, Shared, Type,
    ast::{
        ConstructorExpression, EnumVariant, Expression, ExpressionKind, FunctionKind, Ident,
        ItemVisibility, Literal, NoirEnumeration, StatementKind, UnresolvedType,
        UnresolvedTypeData,
    },
    elaborator::{
        UnstableFeature,
        path_resolution::PathResolutionItem,
        types::{WildcardAllowed, WildcardDisallowedContext},
    },
    hir::{
        comptime::Value,
        def_collector::dc_crate::UnresolvedEnum,
        resolution::{errors::ResolverError, import::PathResolutionError},
        type_check::TypeCheckError,
    },
    hir_def::{
        expr::{
            Case, Constructor, HirBlockExpression, HirEnumConstructorExpression, HirExpression,
            HirIdent, HirLiteral, HirMatch,
        },
        function::{FuncMeta, FunctionBody, HirFunction, Parameters},
        stmt::{HirLetStatement, HirPattern, HirStatement},
    },
    node_interner::{
        DefinitionId, DefinitionKind, ExprId, FunctionModifiers, GlobalValue, ReferenceId, TypeId,
    },
    shared::Visibility,
    signed_field::SignedField,
    token::Attributes,
};

use super::{Elaborator, TypedPathSegment, path_resolution::PathResolutionTarget};

const WILDCARD_PATTERN: &str = "_";

struct MatchCompiler<'elab, 'ctx> {
    elaborator: &'elab mut Elaborator<'ctx>,
    has_missing_cases: bool,
    // We iterate on this to issue errors later so it needs to be a BTreeMap (versus HashMap) to be
    // deterministic.
    unreachable_cases: BTreeMap<RowBody, Location>,
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
    guard: Option<RowBody>,
    body: RowBody,
    original_body: RowBody,
    location: Location,
}

type RowBody = ExprId;

impl Row {
    fn new(columns: Vec<Column>, guard: Option<RowBody>, body: RowBody, location: Location) -> Row {
        Row { columns, guard, body, original_body: body, location }
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

impl Elaborator<'_> {
    pub(super) fn collect_enum_definitions(&mut self, enums: &BTreeMap<TypeId, UnresolvedEnum>) {
        for (type_id, typ) in enums {
            self.local_module = Some(typ.module_id);
            self.generics.clear();

            let datatype = self.interner.get_type(*type_id);
            let datatype_ref = datatype.borrow();
            let generics = datatype_ref.generic_types();
            self.add_existing_generics(&typ.enum_def.generics, &datatype_ref.generics);

            self.use_unstable_feature(UnstableFeature::Enums, datatype_ref.name.location());
            drop(datatype_ref);

            let self_type = Type::DataType(datatype.clone(), generics);
            let self_type_id = self.interner.push_quoted_type(self_type.clone());
            let location = typ.enum_def.location;
            let unresolved =
                UnresolvedType { typ: UnresolvedTypeData::Resolved(self_type_id), location };

            datatype.borrow_mut().init_variants();
            self.resolving_ids.insert(*type_id);

            let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::EnumVariant);
            for (i, variant) in typ.enum_def.variants.iter().enumerate() {
                let parameters = variant.item.parameters.as_ref();
                let types = parameters.map(|params| {
                    vecmap(params, |typ| self.resolve_type(typ.clone(), wildcard_allowed))
                });
                let name = variant.item.name.clone();

                let is_function = types.is_some();
                let params = types.clone().unwrap_or_default();
                datatype.borrow_mut().push_variant(HirEnumVariant::new(name, params, is_function));

                self.define_enum_variant_constructor(
                    &typ.enum_def,
                    *type_id,
                    &variant.item,
                    types,
                    i,
                    &datatype,
                    &self_type,
                    unresolved.clone(),
                );

                let reference_id = ReferenceId::EnumVariant(*type_id, i);
                let location = variant.item.name.location();
                self.interner.add_definition_location(reference_id, location);
            }

            self.resolving_ids.remove(type_id);
        }
        self.generics.clear();
    }

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
            ItemVisibility::Public,
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
        let let_statement = HirStatement::Expression(global_body);

        let statement_id = self.interner.get_global(global_id).let_statement;
        self.interner.replace_statement(statement_id, let_statement);

        self.interner.get_global_mut(global_id).value =
            GlobalValue::Resolved(Value::Enum(variant_index, Vec::new(), typ));

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
            extra_trait_constraints: Vec::new(),
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

    /// Given:
    /// ```ignore
    /// enum FooEnum { Foo(u32, u8), ... }
    ///
    /// fn Foo(a: u32, b: u8) -> FooEnum {}
    /// ```
    /// Create (pseudocode):
    /// ```ignore
    /// fn Foo(a: u32, b: u8) -> FooEnum {
    ///     // This can't actually be written directly in Noir
    ///     FooEnum {
    ///         tag: Foo_tag,
    ///         Foo: (a, b),
    ///         // fields from other variants are zeroed in monomorphization
    ///     }
    /// }
    /// ```
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
            HirPattern::Identifier(ident) => self.interner.push_expr_full(
                HirExpression::Ident(ident.clone(), None),
                location,
                typ.clone(),
            ),
            _ => unreachable!(),
        });

        let constructor = HirExpression::EnumConstructor(HirEnumConstructorExpression {
            r#type: self_type.clone(),
            arguments,
            variant_index,
        });

        let enum_generics = self_type.borrow().generic_types();
        let typ = Type::DataType(self_type.clone(), enum_generics);
        self.interner.push_expr_full(constructor, location, typ)
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
            let pattern_location = pattern.location;
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
            Row::new(columns, guard, body, pattern_location)
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
            ExpressionKind::Literal(Literal::Integer(value, suffix)) => {
                let actual = match suffix {
                    Some(suffix) => suffix.as_type(),
                    None => self.interner.next_type_variable_with_kind(Kind::IntegerOrField),
                };
                unify_with_expected_type(self, &actual);

                let expr = HirExpression::Literal(HirLiteral::Integer(value));
                let location = expr_location;
                let expr_id = self.interner.push_expr_full(expr, location, actual.clone());
                self.push_integer_literal_expr_id(expr_id);

                Pattern::Int(value)
            }
            ExpressionKind::Literal(Literal::Bool(value)) => {
                unify_with_expected_type(self, &Type::Bool);
                let constructor = if value { Constructor::True } else { Constructor::False };
                Pattern::Constructor(constructor, Vec::new())
            }
            ExpressionKind::Variable(path) => {
                let path = self.validate_path(path);

                // A variable can be free or bound if it refers to an enum constant:
                // - in `(a, b)`, both variables may be free and should be defined, or
                //   may refer to an enum variant named `a` or `b` in scope.
                // - Possible diagnostics improvement: warn if `a` is defined as a variable
                //   when there is a matching enum variant with name `Foo::a` which can
                //   be imported. The user likely intended to reference the enum variant.
                let location = path.location;

                // Setting this to `Some` allows us to shadow globals with the same name.
                // We should avoid this if there is a `::` in the path since that means the
                // user is trying to resolve to a non-local item.

                let shadow_existing = path.as_single_segment().cloned();

                match self.resolve_path_or_error(path, PathResolutionTarget::Value) {
                    Ok(resolution) => self.path_resolution_to_constructor(
                        resolution,
                        shadow_existing,
                        Vec::new(),
                        expected_type,
                        location,
                        variables_defined,
                    ),
                    Err(error) => {
                        if let Some(name) = shadow_existing {
                            if name.generics.is_some() {
                                self.push_err(PathResolutionError::TurbofishNotAllowedOnItem {
                                    item: "local variables".to_string(),
                                    location: name.turbofish_location(),
                                });
                            }

                            self.define_pattern_variable(
                                name.ident,
                                expected_type,
                                variables_defined,
                            )
                        } else {
                            self.push_err(error);
                            Pattern::Error
                        }
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

    fn define_pattern_variable(
        &mut self,
        name: Ident,
        expected_type: &Type,
        variables_defined: &mut Vec<Ident>,
    ) -> Pattern {
        // Define the variable
        let kind = DefinitionKind::Local(None);

        if let Some(existing) = variables_defined.iter().find(|elem| *elem == &name) {
            // Allow redefinition of `_` only, to ignore variables
            if name.as_str() != WILDCARD_PATTERN {
                self.push_err(ResolverError::VariableAlreadyDefinedInPattern {
                    existing: existing.clone(),
                    new_location: name.location(),
                });
            }
        } else {
            variables_defined.push(name.clone());
        }

        let id = self.add_variable_decl(name, false, true, true, kind).id;
        self.interner.push_definition_type(id, expected_type.clone());
        Pattern::Binding(id)
    }

    fn constructor_to_pattern(
        &mut self,
        constructor: ConstructorExpression,
        variables_defined: &mut Vec<Ident>,
    ) -> Pattern {
        let location = constructor.typ.location;
        let wildcard_allowed = WildcardAllowed::Yes;
        let typ = self.resolve_type(constructor.typ, wildcard_allowed);

        let Some((struct_name, mut expected_field_types)) =
            self.struct_name_and_field_types(&typ, location)
        else {
            return Pattern::Error;
        };

        let mut fields = BTreeMap::default();
        for (field_name, field) in constructor.fields {
            let Some(field_index) =
                expected_field_types.iter().position(|(name, _, _)| *name == field_name.as_str())
            else {
                let error = if fields.contains_key(field_name.as_str()) {
                    ResolverError::DuplicateField { field: field_name }
                } else {
                    let struct_definition = struct_name.clone();
                    ResolverError::NoSuchField { field: field_name, struct_definition }
                };
                self.push_err(error);
                continue;
            };

            let (field_name, expected_field_type, _) =
                expected_field_types.swap_remove(field_index);
            let pattern =
                self.expression_to_pattern(field, &expected_field_type, variables_defined);
            fields.insert(field_name, pattern);
        }

        if !expected_field_types.is_empty() {
            let struct_definition = struct_name;
            let missing_fields = vecmap(expected_field_types, |(name, _, _)| name);
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
        let syntax_error = |this: &mut Self| {
            this.push_err(ResolverError::InvalidSyntaxInPattern { location: name.location });
            Pattern::Error
        };

        match name.kind {
            ExpressionKind::Variable(path) => {
                let location = path.location;
                let path = self.validate_path(path);

                match self.resolve_path_or_error(path, PathResolutionTarget::Value) {
                    // Use None for `name` here - we don't want to define a variable if this
                    // resolves to an existing item.
                    Ok(resolution) => self.path_resolution_to_constructor(
                        resolution,
                        None,
                        args,
                        expected_type,
                        location,
                        variables_defined,
                    ),
                    Err(error) => {
                        self.push_err(error);
                        Pattern::Error
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
                    syntax_error(self)
                }
            }
            _ => syntax_error(self),
        }
    }

    /// Convert a PathResolutionItem - usually an enum variant or global - to a Constructor.
    /// If `name` is `Some`, we'll define a Pattern::Binding instead of erroring if the
    /// item doesn't resolve to a variant or global. This would shadow an existing
    /// value such as a free function. Generally this is desired unless the variable was
    /// a path with multiple components such as `foo::bar` which should always be treated as
    /// a path to an existing item.
    fn path_resolution_to_constructor(
        &mut self,
        resolution: PathResolutionItem,
        name: Option<TypedPathSegment>,
        args: Vec<Expression>,
        expected_type: &Type,
        location: Location,
        variables_defined: &mut Vec<Ident>,
    ) -> Pattern {
        let (actual_type, expected_arg_types, variant_index) = match &resolution {
            PathResolutionItem::Global(id) => {
                // variant constant
                self.elaborate_global_if_unresolved(id);
                let global = self.interner.get_global(*id);
                let variant_index = match &global.value {
                    GlobalValue::Resolved(Value::Enum(tag, ..)) => *tag,
                    // This may be a global constant. Treat it like a normal constant
                    GlobalValue::Resolved(value) => {
                        let value = value.clone();
                        return self.global_constant_to_integer_constructor(
                            value,
                            expected_type,
                            location,
                        );
                    }
                    // We tried to resolve this value above so there must have been an error
                    // in doing so. Avoid reporting an additional error.
                    _ => return Pattern::Error,
                };

                let global_type = self.interner.definition_type(global.definition_id);
                let actual_type = global_type.instantiate(self.interner).0;
                (actual_type, Vec::new(), variant_index)
            }
            PathResolutionItem::Method(_, _, func_id) | PathResolutionItem::SelfMethod(func_id) => {
                // TODO(#7430): Take type_turbofish into account when instantiating the function's type
                let meta = self.interner.function_meta(func_id);
                let Some(variant_index) = meta.enum_variant_index else {
                    let item = resolution.description();
                    self.push_err(ResolverError::UnexpectedItemInPattern { location, item });
                    return Pattern::Error;
                };

                let (actual_type, expected_arg_types) = match meta.typ.instantiate(self.interner).0
                {
                    Type::Function(args, ret, _env, _) => (*ret, args),
                    other => unreachable!("Not a function! Found {other}"),
                };

                (actual_type, expected_arg_types, variant_index)
            }
            PathResolutionItem::Module(_)
            | PathResolutionItem::Type(_)
            | PathResolutionItem::TypeAlias(_)
            | PathResolutionItem::PrimitiveType(_)
            | PathResolutionItem::Trait(_)
            | PathResolutionItem::TraitAssociatedType(..)
            | PathResolutionItem::ModuleFunction(_)
            | PathResolutionItem::TypeAliasFunction(..)
            | PathResolutionItem::TraitFunction(..)
            | PathResolutionItem::TypeTraitFunction(..)
            | PathResolutionItem::PrimitiveFunction(..) => {
                // This variable refers to an existing item
                if let Some(name) = name {
                    // If name is set, shadow the existing item
                    if name.generics.is_some() {
                        self.push_err(PathResolutionError::TurbofishNotAllowedOnItem {
                            item: "local variables".to_string(),
                            location: name.turbofish_location(),
                        });
                    }
                    return self.define_pattern_variable(
                        name.ident,
                        expected_type,
                        variables_defined,
                    );
                } else {
                    let item = resolution.description();
                    self.push_err(ResolverError::UnexpectedItemInPattern { location, item });
                    return Pattern::Error;
                }
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
            let expected = expected_arg_types.len();
            let found = args.len();
            self.push_err(TypeCheckError::ArityMisMatch { expected, found, location });
            return Pattern::Error;
        }

        let args = args.into_iter().zip(expected_arg_types);
        let args = vecmap(args, |(arg, expected_arg_type)| {
            self.expression_to_pattern(arg, &expected_arg_type, variables_defined)
        });
        let constructor = Constructor::Variant(actual_type, variant_index);
        Pattern::Constructor(constructor, args)
    }

    fn global_constant_to_integer_constructor(
        &mut self,
        constant: Value,
        expected_type: &Type,
        location: Location,
    ) -> Pattern {
        let actual_type = constant.get_type();
        self.unify(&actual_type, expected_type, || TypeCheckError::TypeMismatch {
            expected_typ: expected_type.to_string(),
            expr_typ: actual_type.to_string(),
            expr_location: location,
        });

        let value = match constant {
            Value::Bool(value) => SignedField::positive(value),
            Value::Field(value) => value,
            Value::I8(value) => SignedField::from_signed(value),
            Value::I16(value) => SignedField::from_signed(value),
            Value::I32(value) => SignedField::from_signed(value),
            Value::I64(value) => SignedField::from_signed(value),
            Value::U1(value) => SignedField::positive(value),
            Value::U8(value) => SignedField::positive(u128::from(value)),
            Value::U16(value) => SignedField::positive(u128::from(value)),
            Value::U32(value) => SignedField::positive(value),
            Value::U64(value) => SignedField::positive(value),
            Value::U128(value) => SignedField::positive(value),
            Value::Zeroed(_) => SignedField::positive(0u32),
            _ => {
                self.push_err(ResolverError::NonIntegerGlobalUsedInPattern { location });
                return Pattern::Error;
            }
        };

        Pattern::Int(value)
    }

    #[allow(clippy::type_complexity)]
    fn struct_name_and_field_types(
        &mut self,
        typ: &Type,
        location: Location,
    ) -> Option<(Ident, Vec<(String, Type, ItemVisibility)>)> {
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
    /// This is an adaptation of <https://github.com/yorickpeterse/pattern-matching-in-rust/tree/main/jacobs2021>
    /// which is an implementation of <https://julesjacobs.com/notes/patternmatching/patternmatching.pdf>
    pub(super) fn elaborate_match_rows(
        &mut self,
        rows: Vec<Row>,
        type_matched_on: &Type,
        location: Location,
    ) -> HirMatch {
        MatchCompiler::run(self, rows, type_matched_on, location)
    }
}

impl<'elab, 'ctx> MatchCompiler<'elab, 'ctx> {
    fn run(
        elaborator: &'elab mut Elaborator<'ctx>,
        rows: Vec<Row>,
        type_matched_on: &Type,
        location: Location,
    ) -> HirMatch {
        let mut compiler = Self {
            elaborator,
            has_missing_cases: false,
            unreachable_cases: rows.iter().map(|row| (row.body, row.location)).collect(),
        };

        let hir_match = compiler.compile_rows(rows).unwrap_or_else(|error| {
            compiler.elaborator.push_err(error);
            HirMatch::Failure { missing_case: false }
        });

        if compiler.has_missing_cases {
            compiler.issue_missing_cases_error(&hir_match, type_matched_on, location);
        }

        if !compiler.unreachable_cases.is_empty() {
            compiler.issue_unreachable_cases_warning();
        }

        hir_match
    }

    fn compile_rows(&mut self, mut rows: Vec<Row>) -> Result<HirMatch, ResolverError> {
        if rows.is_empty() {
            self.has_missing_cases = true;
            return Ok(HirMatch::Failure { missing_case: true });
        }

        self.push_tests_against_bare_variables(&mut rows);

        // If the first row is a match-all we match it and the remaining rows are ignored.
        if rows.first().is_some_and(|row| row.columns.is_empty()) {
            let row = rows.remove(0);

            return Ok(match row.guard {
                None => {
                    self.unreachable_cases.remove(&row.original_body);
                    HirMatch::Success(row.body)
                }
                Some(cond) => {
                    let remaining = self.compile_rows(rows)?;
                    HirMatch::Guard { cond, body: row.body, otherwise: Box::new(remaining) }
                }
            });
        }

        let branch_var = self.branch_variable(&rows);
        let location = self.elaborator.interner.definition(branch_var).location;

        let definition_type = self.elaborator.interner.definition_type(branch_var);
        match definition_type.follow_bindings_shallow().into_owned() {
            Type::FieldElement | Type::Integer(_, _) => {
                let (cases, fallback) = self.compile_int_cases(rows, branch_var)?;
                Ok(HirMatch::Switch(branch_var, cases, Some(fallback)))
            }
            Type::TypeVariable(typevar) if typevar.is_integer() || typevar.is_integer_or_field() => {
                let (cases, fallback) = self.compile_int_cases(rows, branch_var)?;
                Ok(HirMatch::Switch(branch_var, cases, Some(fallback)))
            }

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
                    let fields = vecmap(fields, |(_name, typ, _)| typ);
                    let constructor = Constructor::Variant(typ, 0);
                    let field_variables = self.fresh_match_variables(fields, location);
                    let cases = vec![(constructor, field_variables, Vec::new())];
                    let (cases, fallback) =
                        self.compile_constructor_cases(rows, branch_var, cases)?;
                    Ok(HirMatch::Switch(branch_var, cases, fallback))
                } else {
                    drop(def);
                    let typ = Type::DataType(type_def, generics);
                    Err(ResolverError::TypeUnsupportedInMatch { typ, location })
                }
            }
            // We could match on these types in the future
            typ @ (Type::Array(_, _)
            | Type::Vector(_)
            | Type::String(_)
            // But we'll never be able to match on these
            | Type::Alias(_, _)
            | Type::TypeVariable(_)
            | Type::FmtString(_, _)
            | Type::TraitAsType(_, _, _)
            | Type::NamedGeneric(_)
            | Type::CheckedCast { .. }
            | Type::Function(_, _, _, _)
            | Type::Reference(..)
            | Type::Forall(_, _)
            | Type::Constant(_, _)
            | Type::Quoted(_)
            | Type::InfixExpr(_, _, _, _)
            | Type::Error) => {
                Err(ResolverError::TypeUnsupportedInMatch { typ, location })
            },
        }
    }

    fn fresh_match_variables(
        &mut self,
        variable_types: Vec<Type>,
        location: Location,
    ) -> Vec<DefinitionId> {
        vecmap(variable_types.into_iter().enumerate(), |(index, typ)| {
            self.fresh_match_variable(index, typ, location)
        })
    }

    fn fresh_match_variable(
        &mut self,
        index: usize,
        variable_type: Type,
        location: Location,
    ) -> DefinitionId {
        let name = format!("internal_match_variable_{index}");
        let kind = DefinitionKind::Local(None);
        let id = self.elaborator.interner.push_definition(name, false, false, kind, location);
        self.elaborator.interner.push_definition_type(id, variable_type);
        id
    }

    /// Compiles the cases and fallback cases for integer and range patterns.
    ///
    /// Integers have an infinite number of constructors, so we specialize the
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
                    // Any other pattern shouldn't have an integer type and we expect a type
                    // check error to already have been issued.
                    _ => continue,
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
    /// 3. We turn the resulting vector of rows into a vector of cases, then compile
    ///    those into decision (sub) trees.
    ///
    /// If a row didn't include the branching variable, we simply copy that row
    /// into the vector of rows for every constructor to test.
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

                    cases[idx].2.push(Row::new(cols, row.guard, row.body, row.location));
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
        let location = self.elaborator.interner.definition(rhs).location;

        let r#type = self.elaborator.interner.definition_type(variable);
        let rhs_type = self.elaborator.interner.definition_type(rhs);
        let variable = HirIdent::non_trait_method(variable, location);

        let rhs = HirExpression::Ident(HirIdent::non_trait_method(rhs, location), None);
        let rhs = self.elaborator.interner.push_expr_full(rhs, location, rhs_type);

        let let_ = HirStatement::Let(HirLetStatement {
            pattern: HirPattern::Identifier(variable),
            r#type,
            expression: rhs,
            attributes: Vec::new(),
            comptime: false,
            is_global_let: false,
        });

        let body_type = self.elaborator.interner.id_type(body);
        let let_ = self.elaborator.interner.push_stmt_full(let_, location);
        let body =
            self.elaborator.interner.push_stmt_full(HirStatement::Expression(body), location);

        let block = HirExpression::Block(HirBlockExpression { statements: vec![let_, body] });
        self.elaborator.interner.push_expr_full(block, location, body_type)
    }

    /// Any case that isn't branched to when the match is finished must be covered by another
    /// case and is thus redundant.
    fn issue_unreachable_cases_warning(&mut self) {
        for location in self.unreachable_cases.values().copied() {
            self.elaborator.push_err(TypeCheckError::UnreachableCase { location });
        }
    }

    /// Traverse the resulting HirMatch to build counter-examples of values which would
    /// not be covered by the match.
    fn issue_missing_cases_error(
        &mut self,
        tree: &HirMatch,
        type_matched_on: &Type,
        location: Location,
    ) {
        let starting_id = match tree {
            HirMatch::Switch(id, ..) => *id,
            _ => return self.issue_missing_cases_error_for_type(type_matched_on, location),
        };

        let mut cases = BTreeSet::new();
        self.find_missing_values(tree, &mut Default::default(), &mut cases, starting_id);

        // It's possible to trigger this matching on an empty enum like `enum Void {}`
        if !cases.is_empty() {
            self.elaborator.push_err(TypeCheckError::MissingCases { cases, location });
        }
    }

    /// Issue a missing cases error if necessary for the given type, assuming that no
    /// case of the type is covered. This is the case for empty matches `match foo {}`.
    /// Note that this is expected not to error if the given type is an enum with zero variants.
    fn issue_missing_cases_error_for_type(&mut self, type_matched_on: &Type, location: Location) {
        let typ = type_matched_on.follow_bindings_shallow();
        if let Type::DataType(shared, generics) = typ.as_ref() {
            if let Some(variants) = shared.borrow().get_variants(generics) {
                let cases: BTreeSet<_> = variants.into_iter().map(|(name, _)| name).collect();
                if !cases.is_empty() {
                    self.elaborator.push_err(TypeCheckError::MissingCases { cases, location });
                }
                return;
            }
        }
        let typ = typ.to_string();
        self.elaborator.push_err(TypeCheckError::MissingManyCases { typ, location });
    }

    fn find_missing_values(
        &self,
        tree: &HirMatch,
        env: &mut HashMap<DefinitionId, (String, Vec<DefinitionId>)>,
        missing_cases: &mut BTreeSet<String>,
        starting_id: DefinitionId,
    ) {
        match tree {
            HirMatch::Success(_) | HirMatch::Failure { missing_case: false } => (),
            HirMatch::Guard { otherwise, .. } => {
                self.find_missing_values(otherwise, env, missing_cases, starting_id);
            }
            HirMatch::Failure { missing_case: true } => {
                let case = Self::construct_missing_case(starting_id, env);
                missing_cases.insert(case);
            }
            HirMatch::Switch(definition_id, cases, else_case) => {
                for case in cases {
                    let name = case.constructor.to_string();
                    env.insert(*definition_id, (name, case.arguments.clone()));
                    self.find_missing_values(&case.body, env, missing_cases, starting_id);
                }

                if let Some(else_case) = else_case {
                    let typ = self.elaborator.interner.definition_type(*definition_id);

                    for case in self.missing_cases(cases, &typ) {
                        env.insert(*definition_id, case);
                        self.find_missing_values(else_case, env, missing_cases, starting_id);
                    }
                }

                env.remove(definition_id);
            }
        }
    }

    fn missing_cases(&self, cases: &[Case], typ: &Type) -> Vec<(String, Vec<DefinitionId>)> {
        // We expect `cases` to come from a `Switch` which should always have
        // at least 2 cases, otherwise it should be a Success or Failure node.
        let first = &cases[0];

        if matches!(&first.constructor, Constructor::Int(_) | Constructor::Range(..)) {
            return self.missing_integer_cases(cases, typ);
        }

        let all_constructors = first.constructor.all_constructors();
        let mut all_constructors =
            btree_map(all_constructors, |(constructor, arg_count)| (constructor, arg_count));

        for case in cases {
            all_constructors.remove(&case.constructor);
        }

        vecmap(all_constructors, |(constructor, arg_count)| {
            // Safety: this id should only be used in `env` of `find_missing_values` which
            //         only uses it for display and defaults to "_" on unknown ids.
            let args = vecmap(0..arg_count, |_| DefinitionId::dummy_id());
            (constructor.to_string(), args)
        })
    }

    fn missing_integer_cases(
        &self,
        cases: &[Case],
        typ: &Type,
    ) -> Vec<(String, Vec<DefinitionId>)> {
        // We could give missed cases for field ranges of `0 .. field_modulus` but since the field
        // used in Noir may change we recommend a match-all pattern instead.
        // If the type is a type variable, we don't know exactly which integer type this may
        // resolve to so also just suggest a catch-all in that case.
        if typ.is_field() || typ.is_bindable() {
            return vec![(WILDCARD_PATTERN.to_string(), Vec::new())];
        }

        let mut missing_cases = rangemap::RangeInclusiveSet::new();

        let int_max = SignedField::positive(typ.integral_maximum_size().unwrap());
        let int_min = typ.integral_minimum_size().unwrap();
        missing_cases.insert(int_min..=int_max);

        for case in cases {
            match &case.constructor {
                Constructor::Int(signed_field) => {
                    missing_cases.remove(*signed_field..=*signed_field);
                }
                Constructor::Range(start, end) => {
                    // Our ranges are exclusive, so adjust for that
                    missing_cases.remove(*start..=end.sub_one());
                }
                _ => unreachable!(
                    "missing_integer_cases should only be called with Int or Range constructors"
                ),
            }
        }

        vecmap(missing_cases, |range| {
            if range.start() == range.end() {
                (format!("{}", range.start()), Vec::new())
            } else {
                (format!("{}..={}", range.start(), range.end()), Vec::new())
            }
        })
    }

    fn construct_missing_case(
        starting_id: DefinitionId,
        env: &HashMap<DefinitionId, (String, Vec<DefinitionId>)>,
    ) -> String {
        let Some((constructor, arguments)) = env.get(&starting_id) else {
            return WILDCARD_PATTERN.to_string();
        };

        let no_arguments = arguments.is_empty();

        let args = vecmap(arguments, |arg| Self::construct_missing_case(*arg, env)).join(", ");

        if no_arguments { constructor.clone() } else { format!("{constructor}({args})") }
    }
}
