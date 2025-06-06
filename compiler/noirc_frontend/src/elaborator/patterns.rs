use iter_extended::vecmap;
use noirc_errors::{Located, Location};
use rustc_hash::FxHashSet as HashSet;

use crate::{
    DataType, Kind, Shared, Type, TypeAlias, TypeBindings,
    ast::{
        ERROR_IDENT, Expression, ExpressionKind, GenericTypeArgs, Ident, ItemVisibility, Path,
        PathSegment, Pattern, TypePath,
    },
    elaborator::types::SELF_TYPE_NAME,
    hir::{
        def_collector::dc_crate::CompilationError,
        resolution::{errors::ResolverError, import::PathResolutionError},
        type_check::{
            Source, TypeCheckError,
            generics::{FmtstrPrimitiveType, Generic, StrPrimitiveType},
        },
    },
    hir_def::{
        expr::{HirExpression, HirIdent, HirLiteral, HirMethodReference, ImplKind, TraitMethod},
        stmt::HirPattern,
    },
    node_interner::{
        DefinitionId, DefinitionInfo, DefinitionKind, ExprId, FuncId, GlobalId, TraitImplKind,
    },
    signed_field::SignedField,
};

use super::{
    Elaborator, PrimitiveType, ResolverMeta,
    path_resolution::{PathResolutionItem, TypedPath, TypedPathSegment},
};

impl Elaborator<'_> {
    pub(super) fn elaborate_pattern(
        &mut self,
        pattern: Pattern,
        expected_type: Type,
        definition_kind: DefinitionKind,
        warn_if_unused: bool,
    ) -> HirPattern {
        self.elaborate_pattern_mut(
            pattern,
            expected_type,
            definition_kind,
            None,
            &mut Vec::new(),
            warn_if_unused,
        )
    }

    /// Equivalent to `elaborate_pattern`, this version just also
    /// adds any new DefinitionIds that were created to the given Vec.
    pub fn elaborate_pattern_and_store_ids(
        &mut self,
        pattern: Pattern,
        expected_type: Type,
        definition_kind: DefinitionKind,
        created_ids: &mut Vec<HirIdent>,
        warn_if_unused: bool,
    ) -> HirPattern {
        self.elaborate_pattern_mut(
            pattern,
            expected_type,
            definition_kind,
            None,
            created_ids,
            warn_if_unused,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn elaborate_pattern_mut(
        &mut self,
        pattern: Pattern,
        expected_type: Type,
        definition: DefinitionKind,
        mutable: Option<Location>,
        new_definitions: &mut Vec<HirIdent>,
        warn_if_unused: bool,
    ) -> HirPattern {
        match pattern {
            Pattern::Identifier(name) => {
                // If this definition is mutable, do not store the rhs because it will
                // not always refer to the correct value of the variable
                let definition = match (mutable, definition) {
                    (Some(_), DefinitionKind::Local(_)) => DefinitionKind::Local(None),
                    (_, other) => other,
                };
                let ident = if let DefinitionKind::Global(global_id) = definition {
                    // Globals don't need to be added to scope, they're already in the def_maps
                    let id = self.interner.get_global(global_id).definition_id;
                    let location = name.location();
                    HirIdent::non_trait_method(id, location)
                } else {
                    self.add_variable_decl(
                        name,
                        mutable.is_some(),
                        true, // allow_shadowing
                        warn_if_unused,
                        definition,
                    )
                };
                self.interner.push_definition_type(ident.id, expected_type);
                new_definitions.push(ident.clone());
                HirPattern::Identifier(ident)
            }
            Pattern::Mutable(pattern, location, _) => {
                if let Some(first_mut) = mutable {
                    self.push_err(ResolverError::UnnecessaryMut {
                        first_mut,
                        second_mut: location,
                    });
                }

                let pattern = self.elaborate_pattern_mut(
                    *pattern,
                    expected_type,
                    definition,
                    Some(location),
                    new_definitions,
                    warn_if_unused,
                );
                HirPattern::Mutable(Box::new(pattern), location)
            }
            Pattern::Tuple(fields, location) => {
                let field_types = match expected_type.follow_bindings() {
                    Type::Tuple(fields) => fields,
                    Type::Error => Vec::new(),
                    expected_type => {
                        let tuple =
                            Type::Tuple(vecmap(&fields, |_| self.interner.next_type_variable()));

                        self.push_err(TypeCheckError::TypeMismatchWithSource {
                            expected: expected_type,
                            actual: tuple,
                            location,
                            source: Source::Assignment,
                        });
                        Vec::new()
                    }
                };

                if fields.len() != field_types.len() {
                    self.push_err(TypeCheckError::TupleMismatch {
                        tuple_types: field_types.clone(),
                        actual_count: fields.len(),
                        location,
                    });
                }

                let fields = vecmap(fields.into_iter().enumerate(), |(i, field)| {
                    let field_type = field_types.get(i).cloned().unwrap_or(Type::Error);
                    self.elaborate_pattern_mut(
                        field,
                        field_type,
                        definition.clone(),
                        mutable,
                        new_definitions,
                        warn_if_unused,
                    )
                });
                HirPattern::Tuple(fields, location)
            }
            Pattern::Struct(name, fields, location) => {
                let name = self.validate_path(name);
                self.elaborate_struct_pattern(
                    name,
                    fields,
                    location,
                    expected_type,
                    definition,
                    mutable,
                    new_definitions,
                )
            }
            Pattern::Parenthesized(pattern, _) => self.elaborate_pattern_mut(
                *pattern,
                expected_type,
                definition,
                mutable,
                new_definitions,
                warn_if_unused,
            ),
            Pattern::Interned(id, _) => {
                let pattern = self.interner.get_pattern(id).clone();
                self.elaborate_pattern_mut(
                    pattern,
                    expected_type,
                    definition,
                    mutable,
                    new_definitions,
                    warn_if_unused,
                )
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn elaborate_struct_pattern(
        &mut self,
        name: TypedPath,
        fields: Vec<(Ident, Pattern)>,
        location: Location,
        expected_type: Type,
        definition: DefinitionKind,
        mutable: Option<Location>,
        new_definitions: &mut Vec<HirIdent>,
    ) -> HirPattern {
        let last_segment = name.last_segment();
        let name_location = last_segment.ident.location();
        let is_self_type = last_segment.ident.is_self_type_name();

        let error_identifier = |this: &mut Self| {
            // Must create a name here to return a HirPattern::Identifier. Allowing
            // shadowing here lets us avoid further errors if we define ERROR_IDENT
            // multiple times.
            let name = ERROR_IDENT.into();
            let identifier = this.add_variable_decl(name, false, true, true, definition.clone());
            HirPattern::Identifier(identifier)
        };

        let (struct_type, generics) = match self.lookup_type_or_error(name) {
            Some(Type::DataType(struct_type, struct_generics))
                if struct_type.borrow().is_struct() =>
            {
                (struct_type, struct_generics)
            }
            None => return error_identifier(self),
            Some(typ) => {
                let typ = typ.to_string();
                self.push_err(ResolverError::NonStructUsedInConstructor { typ, location });
                return error_identifier(self);
            }
        };

        let turbofish_location = last_segment.turbofish_location();

        let generics = self.resolve_struct_turbofish_generics(
            &struct_type.borrow(),
            generics,
            last_segment.generics,
            turbofish_location,
        );

        let actual_type = Type::DataType(struct_type.clone(), generics);

        self.unify(&actual_type, &expected_type, || TypeCheckError::TypeMismatchWithSource {
            expected: expected_type.clone(),
            actual: actual_type.clone(),
            location,
            source: Source::Assignment,
        });

        let typ = struct_type.clone();
        let fields = self.resolve_constructor_pattern_fields(
            typ,
            fields,
            location,
            expected_type.clone(),
            definition,
            mutable,
            new_definitions,
        );

        let struct_id = struct_type.borrow().id;

        self.interner.add_type_reference(struct_id, name_location, is_self_type);

        for (field_index, field) in fields.iter().enumerate() {
            let reference_location = field.0.location();
            self.interner.add_struct_member_reference(struct_id, field_index, reference_location);
        }

        HirPattern::Struct(expected_type, fields, location)
    }

    /// Resolve all the fields of a struct constructor expression.
    /// Ensures all fields are present, none are repeated, and all
    /// are part of the struct.
    #[allow(clippy::too_many_arguments)]
    fn resolve_constructor_pattern_fields(
        &mut self,
        struct_type: Shared<DataType>,
        fields: Vec<(Ident, Pattern)>,
        location: Location,
        expected_type: Type,
        definition: DefinitionKind,
        mutable: Option<Location>,
        new_definitions: &mut Vec<HirIdent>,
    ) -> Vec<(Ident, HirPattern)> {
        let mut ret = Vec::with_capacity(fields.len());
        let mut seen_fields = HashSet::default();
        let mut unseen_fields = struct_type
            .borrow()
            .field_names()
            .expect("This type should already be validated to be a struct");

        for (field, pattern) in fields {
            let (field_type, visibility) = expected_type
                .get_field_type_and_visibility(field.as_str())
                .unwrap_or((Type::Error, ItemVisibility::Public));
            let resolved = self.elaborate_pattern_mut(
                pattern,
                field_type,
                definition.clone(),
                mutable,
                new_definitions,
                true, // warn_if_unused
            );

            if unseen_fields.contains(&field) {
                unseen_fields.remove(&field);
                seen_fields.insert(field.clone());

                self.check_struct_field_visibility(
                    &struct_type.borrow(),
                    field.as_str(),
                    visibility,
                    field.location(),
                );
            } else if seen_fields.contains(&field) {
                // duplicate field
                self.push_err(ResolverError::DuplicateField { field: field.clone() });
            } else {
                // field not required by struct
                self.push_err(ResolverError::NoSuchField {
                    field: field.clone(),
                    struct_definition: struct_type.borrow().name.clone(),
                });
            }

            ret.push((field, resolved));
        }

        if !unseen_fields.is_empty() {
            self.push_err(ResolverError::MissingFields {
                location,
                missing_fields: unseen_fields.into_iter().map(|field| field.to_string()).collect(),
                struct_definition: struct_type.borrow().name.clone(),
            });
        }

        ret
    }

    pub(super) fn add_variable_decl(
        &mut self,
        name: Ident,
        mutable: bool,
        allow_shadowing: bool,
        warn_if_unused: bool,
        definition: DefinitionKind,
    ) -> HirIdent {
        if let DefinitionKind::Global(global_id) = definition {
            return self.add_global_variable_decl(name, global_id);
        }

        let location = name.location();
        let name = name.into_string();
        let comptime = self.in_comptime_context();
        let id =
            self.interner.push_definition(name.clone(), mutable, comptime, definition, location);
        let ident = HirIdent::non_trait_method(id, location);
        let resolver_meta =
            ResolverMeta { num_times_used: 0, ident: ident.clone(), warn_if_unused };

        if name != "_" {
            let scope = self.scopes.get_mut_scope();
            let old_value = scope.add_key_value(name.clone(), resolver_meta);

            if !allow_shadowing {
                if let Some(old_value) = old_value {
                    self.push_err(ResolverError::DuplicateDefinition {
                        name,
                        first_location: old_value.ident.location,
                        second_location: location,
                    });
                }
            }
        }

        ident
    }

    pub fn add_existing_variable_to_scope(
        &mut self,
        name: String,
        ident: HirIdent,
        warn_if_unused: bool,
    ) {
        let second_location = ident.location;
        let resolver_meta = ResolverMeta { num_times_used: 0, ident, warn_if_unused };

        let old_value = self.scopes.get_mut_scope().add_key_value(name.clone(), resolver_meta);

        if let Some(old_value) = old_value {
            let first_location = old_value.ident.location;
            self.push_err(ResolverError::DuplicateDefinition {
                name,
                first_location,
                second_location,
            });
        }
    }

    pub fn add_global_variable_decl(&mut self, name: Ident, global_id: GlobalId) -> HirIdent {
        let scope = self.scopes.get_mut_scope();
        let global = self.interner.get_global(global_id);
        let ident = HirIdent::non_trait_method(global.definition_id, global.location);
        let resolver_meta =
            ResolverMeta { num_times_used: 0, ident: ident.clone(), warn_if_unused: true };

        let old_global_value = scope.add_key_value(name.to_string(), resolver_meta);
        if let Some(old_global_value) = old_global_value {
            self.push_err(ResolverError::DuplicateDefinition {
                first_location: old_global_value.ident.location,
                second_location: name.location(),
                name: name.into_string(),
            });
        }
        ident
    }

    /// Lookup and use the specified variable.
    /// This will increment its use counter by one and return the variable if found.
    /// If the variable is not found, an error is returned.
    pub(super) fn use_variable(
        &mut self,
        name: &Ident,
    ) -> Result<(HirIdent, usize), ResolverError> {
        // Find the definition for this Ident
        let scope_tree = self.scopes.current_scope_tree();
        let variable = scope_tree.find(name.as_str());

        let location = name.location();
        if let Some((variable_found, scope)) = variable {
            variable_found.num_times_used += 1;
            let id = variable_found.ident.id;
            Ok((HirIdent::non_trait_method(id, location), scope))
        } else {
            Err(ResolverError::VariableNotDeclared {
                name: name.to_string(),
                location: name.location(),
            })
        }
    }

    /// Resolve generics using the expected kinds of the function we are calling
    pub(super) fn resolve_function_turbofish_generics(
        &mut self,
        func_id: &FuncId,
        resolved_turbofish: Option<Vec<Located<Type>>>,
        location: Location,
    ) -> Option<Vec<Type>> {
        let direct_generic_kinds =
            vecmap(&self.interner.function_meta(func_id).direct_generics, |generic| generic.kind());

        resolved_turbofish.map(|resolved_turbofish| {
            if resolved_turbofish.len() != direct_generic_kinds.len() {
                let type_check_err = TypeCheckError::IncorrectTurbofishGenericCount {
                    expected_count: direct_generic_kinds.len(),
                    actual_count: resolved_turbofish.len(),
                    location,
                };
                self.push_err(type_check_err);
            }

            self.resolve_turbofish_generics(direct_generic_kinds, resolved_turbofish)
        })
    }

    pub(super) fn resolve_struct_turbofish_generics(
        &mut self,
        struct_type: &DataType,
        generics: Vec<Type>,
        resolved_turbofish: Option<Vec<Located<Type>>>,
        location: Location,
    ) -> Vec<Type> {
        let kinds = vecmap(&struct_type.generics, |generic| generic.kind());
        self.resolve_item_turbofish_generics(
            "struct",
            struct_type.name.as_str(),
            kinds,
            generics,
            resolved_turbofish,
            location,
        )
    }

    pub(super) fn resolve_trait_turbofish_generics(
        &mut self,
        trait_name: &str,
        trait_generic_kinds: Vec<Kind>,
        generics: Vec<Type>,
        resolved_turbofish: Option<Vec<Located<Type>>>,
        location: Location,
    ) -> Vec<Type> {
        self.resolve_item_turbofish_generics(
            "trait",
            trait_name,
            trait_generic_kinds,
            generics,
            resolved_turbofish,
            location,
        )
    }

    pub(super) fn resolve_alias_turbofish_generics(
        &mut self,
        type_alias: &TypeAlias,
        generics: Vec<Type>,
        resolved_turbofish: Option<Vec<Located<Type>>>,
        location: Location,
    ) -> Vec<Type> {
        let kinds = vecmap(&type_alias.generics, |generic| generic.kind());
        self.resolve_item_turbofish_generics(
            "alias",
            type_alias.name.as_str(),
            kinds,
            generics,
            resolved_turbofish,
            location,
        )
    }

    pub(super) fn resolve_item_turbofish_generics(
        &mut self,
        item_kind: &'static str,
        item_name: &str,
        item_generic_kinds: Vec<Kind>,
        generics: Vec<Type>,
        resolved_turbofish: Option<Vec<Located<Type>>>,
        location: Location,
    ) -> Vec<Type> {
        let Some(turbofish_generics) = resolved_turbofish else {
            return generics;
        };

        if turbofish_generics.len() != generics.len() {
            self.push_err(TypeCheckError::GenericCountMismatch {
                item: format!("{item_kind} {item_name}"),
                expected: generics.len(),
                found: turbofish_generics.len(),
                location,
            });
            return generics;
        }

        self.resolve_turbofish_generics(item_generic_kinds, turbofish_generics)
    }

    pub(super) fn resolve_turbofish_generics(
        &mut self,
        kinds: Vec<Kind>,
        turbofish_generics: Vec<Located<Type>>,
    ) -> Vec<Type> {
        let kinds_with_types = kinds.into_iter().zip(turbofish_generics);

        vecmap(kinds_with_types, |(kind, located_type)| {
            let location = located_type.location();
            let typ = located_type.contents;
            let typ = typ.substitute_kind_any_with_kind(&kind);
            self.check_kind(typ, &kind, location)
        })
    }

    pub(super) fn elaborate_variable(&mut self, variable: Path) -> (ExprId, Type) {
        let variable = self.validate_path(variable);
        if let Some((expr_id, typ)) =
            self.elaborate_variable_as_self_method_or_associated_constant(&variable)
        {
            return (expr_id, typ);
        }

        let resolved_turbofish = variable.segments.last().unwrap().generics.clone();

        let location = variable.location;
        let (expr, item) = self.resolve_variable(variable);
        let definition_id = expr.id;

        let (type_generics, self_generic) = if let Some(item) = item {
            self.resolve_item_turbofish_and_self_type(item)
        } else {
            (Vec::new(), None)
        };

        let definition = self.interner.try_definition(definition_id);
        let is_comptime_local = !self.in_comptime_context()
            && definition.is_some_and(DefinitionInfo::is_comptime_local);
        let definition_kind = definition.as_ref().map(|definition| definition.kind.clone());

        let mut bindings = TypeBindings::default();

        // Resolve any generics if we the variable we have resolved is a function
        // and if the turbofish operator was used.
        let generics = if let Some(DefinitionKind::Function(func_id)) = &definition_kind {
            self.resolve_function_turbofish_generics(func_id, resolved_turbofish, location)
        } else {
            None
        };

        if let Some(DefinitionKind::Function(func_id)) = &definition_kind {
            // If there's a self type, bind it to the self type generic
            if let Some(self_generic) = self_generic {
                let func_generics = &self.interner.function_meta(func_id).all_generics;
                let self_resolved_generic =
                    func_generics.iter().find(|generic| generic.name.as_str() == SELF_TYPE_NAME);
                if let Some(self_resolved_generic) = self_resolved_generic {
                    let type_var = &self_resolved_generic.type_var;
                    bindings
                        .insert(type_var.id(), (type_var.clone(), type_var.kind(), self_generic));
                }
            }

            // If this is a function call on a type that has generics, we need to bind those generic types.
            if !type_generics.is_empty() {
                // `all_generics` will always have the enclosing type generics first, so we need to bind those
                let func_generics = &self.interner.function_meta(func_id).all_generics;
                for (type_generic, func_generic) in type_generics.into_iter().zip(func_generics) {
                    let type_var = &func_generic.type_var;
                    bindings
                        .insert(type_var.id(), (type_var.clone(), type_var.kind(), type_generic));
                }
            }
        }

        let id = self.interner.push_expr(HirExpression::Ident(expr.clone(), generics.clone()));

        self.interner.push_expr_location(id, location);
        let typ = self.type_check_variable_with_bindings(expr, id, generics, bindings);
        self.interner.push_expr_type(id, typ.clone());

        // If this variable it a comptime local variable, use its current value as the final expression
        if is_comptime_local {
            let mut interpreter = self.setup_interpreter();
            let value = interpreter.evaluate(id);
            // If the value is an error it means the variable already had an error, so don't report it here again
            // (the error will make no sense, it will say that a non-comptime variable was referenced at runtime
            // but that's not true)
            if value.is_ok() {
                let (id, typ) = self.inline_comptime_value(value, location);
                self.debug_comptime(location, |interner| id.to_display_ast(interner).kind);
                (id, typ)
            } else {
                (id, typ)
            }
        } else {
            (id, typ)
        }
    }

    /// Checks whether `variable` is `Self::method_name` or `Self::AssociatedConstant` when we are inside a trait impl and `Self`
    /// resolves to a primitive type.
    ///
    /// In the first case we elaborate this as if it were a TypePath
    /// (for example, if `Self` is `u32` then we consider this the same as `u32::method_name`).
    /// A regular path lookup won't work here for the same reason `TypePath` exists.
    ///
    /// In the second case we solve the associated constant by looking up its value, later
    /// turning it into a literal.
    fn elaborate_variable_as_self_method_or_associated_constant(
        &mut self,
        variable: &TypedPath,
    ) -> Option<(ExprId, Type)> {
        if !(variable.segments.len() == 2 && variable.segments[0].ident.is_self_type_name()) {
            return None;
        }

        let location = variable.location;
        let name = variable.segments[1].ident.as_str();

        // Check the `Self::AssociatedConstant` case when inside a trait
        if let Some(trait_id) = &self.current_trait {
            let trait_ = self.interner.get_trait(*trait_id);
            if let Some(associated_type) = trait_.get_associated_type(name) {
                if let Kind::Numeric(numeric_type) = associated_type.kind() {
                    // We can produce any value here because this trait method is never going to
                    // produce code (only trait impl methods do)
                    let numeric_type: Type = *numeric_type.clone();
                    let value = SignedField::zero();
                    return Some(self.constant_integer(numeric_type, value, location));
                }
            }
        }

        let Some(self_type) = &self.self_type else {
            return None;
        };

        let Some(trait_impl_id) = &self.current_trait_impl else {
            return None;
        };

        // Check the `Self::AssociatedConstant` case when inside a trait impl
        if let Some((definition_id, numeric_type)) =
            self.interner.get_trait_impl_associated_constant(*trait_impl_id, name).cloned()
        {
            let hir_ident = HirIdent::non_trait_method(definition_id, location);
            let hir_expr = HirExpression::Ident(hir_ident, None);
            let id = self.interner.push_expr(hir_expr);
            self.interner.push_expr_location(id, location);
            self.interner.push_expr_type(id, numeric_type.clone());
            return Some((id, numeric_type));
        }

        // Check the `Self::method_name` case when `Self` is a primitive type
        if matches!(self.self_type, Some(Type::DataType(..))) {
            return None;
        }

        let ident = variable.segments[1].ident.clone();
        let typ_location = variable.segments[0].location;
        Some(self.elaborate_type_path_impl(self_type.clone(), ident, None, typ_location))
    }

    pub(crate) fn validate_path(&mut self, path: Path) -> TypedPath {
        let mut segments = vecmap(path.segments, |segment| self.validate_path_segment(segment));

        if let Some(first_segment) = segments.first_mut() {
            if first_segment.generics.is_some() && first_segment.ident.is_self_type_name() {
                self.push_err(PathResolutionError::TurbofishNotAllowedOnItem {
                    item: "self type".to_string(),
                    location: first_segment.turbofish_location(),
                });
                first_segment.generics = None;
            }
        }

        let kind_location = path.kind_location;
        TypedPath { segments, kind: path.kind, location: path.location, kind_location }
    }

    fn validate_path_segment(&mut self, segment: PathSegment) -> TypedPathSegment {
        let generics = segment.generics.map(|generics| {
            vecmap(generics, |generic| {
                let location = generic.location;
                let typ = self.use_type_with_kind(generic, &Kind::Any);
                Located::from(location, typ)
            })
        });
        TypedPathSegment { ident: segment.ident, generics, location: segment.location }
    }

    fn constant_integer(
        &mut self,
        numeric_type: Type,
        value: SignedField,
        location: Location,
    ) -> (ExprId, Type) {
        let hir_expr = HirExpression::Literal(HirLiteral::Integer(value));
        let id = self.interner.push_expr(hir_expr);
        self.interner.push_expr_location(id, location);
        self.interner.push_expr_type(id, numeric_type.clone());
        (id, numeric_type)
    }

    /// Solve any generics that are part of the path before the function, for example:
    ///
    /// ```noir
    /// foo::Bar::<i32>::baz   
    /// ```
    /// Solve `<i32>` above
    fn resolve_item_turbofish_and_self_type(
        &mut self,
        item: PathResolutionItem,
    ) -> (Vec<Type>, Option<Type>) {
        match item {
            PathResolutionItem::Method(struct_id, Some(generics), _func_id) => {
                let struct_type = self.interner.get_type(struct_id);
                let struct_type = struct_type.borrow();
                let struct_generics = struct_type.instantiate(self.interner);
                let generics = self.resolve_struct_turbofish_generics(
                    &struct_type,
                    struct_generics,
                    Some(generics.generics),
                    generics.location,
                );
                (generics, None)
            }
            PathResolutionItem::SelfMethod(_) => {
                let generics = if let Some(Type::DataType(_, generics)) = &self.self_type {
                    generics.clone()
                } else {
                    Vec::new()
                };
                (generics, None)
            }
            PathResolutionItem::TypeAliasFunction(type_alias_id, generics, _func_id) => {
                let type_alias = self.interner.get_type_alias(type_alias_id);
                let type_alias = type_alias.borrow();
                let alias_generics = vecmap(&type_alias.generics, |generic| {
                    self.interner.next_type_variable_with_kind(generic.kind())
                });

                // First solve the generics on the alias, if any
                let generics = if let Some(generics) = generics {
                    self.resolve_alias_turbofish_generics(
                        &type_alias,
                        alias_generics,
                        Some(generics.generics),
                        generics.location,
                    )
                } else {
                    alias_generics
                };

                // Now instantiate the underlying struct or alias with those generics, the struct might
                // have more generics than those in the alias, like in this example:
                //
                // type Alias<T> = Struct<T, i32>;
                let generics = get_type_alias_generics(&type_alias, &generics);
                (generics, None)
            }
            PathResolutionItem::TraitFunction(trait_id, Some(generics), _func_id) => {
                let trait_ = self.interner.get_trait(trait_id);
                let kinds = vecmap(&trait_.generics, |generic| generic.kind());
                let trait_generics =
                    vecmap(&kinds, |kind| self.interner.next_type_variable_with_kind(kind.clone()));

                let generics = self.resolve_trait_turbofish_generics(
                    &trait_.name.to_string(),
                    kinds,
                    trait_generics,
                    Some(generics.generics),
                    generics.location,
                );
                (generics, None)
            }
            PathResolutionItem::TypeTraitFunction(self_type, trait_id, generics, _func_id) => {
                let generics = if let Some(generics) = generics {
                    let trait_ = self.interner.get_trait(trait_id);
                    let kinds = vecmap(&trait_.generics, |generic| generic.kind());
                    let trait_generics = vecmap(&kinds, |kind| {
                        self.interner.next_type_variable_with_kind(kind.clone())
                    });

                    self.resolve_trait_turbofish_generics(
                        &trait_.name.to_string(),
                        kinds,
                        trait_generics,
                        Some(generics.generics),
                        generics.location,
                    )
                } else {
                    Vec::new()
                };
                (generics, Some(self_type))
            }
            PathResolutionItem::PrimitiveFunction(primitive_type, turbofish, _func_id) => {
                let generics = match primitive_type {
                    PrimitiveType::Bool
                    | PrimitiveType::CtString
                    | PrimitiveType::Expr
                    | PrimitiveType::Field
                    | PrimitiveType::FunctionDefinition
                    | PrimitiveType::I8
                    | PrimitiveType::I16
                    | PrimitiveType::I32
                    | PrimitiveType::I64
                    | PrimitiveType::U1
                    | PrimitiveType::U8
                    | PrimitiveType::U16
                    | PrimitiveType::U32
                    | PrimitiveType::U64
                    | PrimitiveType::U128
                    | PrimitiveType::Module
                    | PrimitiveType::Quoted
                    | PrimitiveType::StructDefinition
                    | PrimitiveType::TraitConstraint
                    | PrimitiveType::TraitDefinition
                    | PrimitiveType::TraitImpl
                    | PrimitiveType::TypeDefinition
                    | PrimitiveType::TypedExpr
                    | PrimitiveType::Type
                    | PrimitiveType::UnresolvedType => {
                        if let Some(turbofish) = turbofish {
                            self.push_err(CompilationError::TypeError(
                                TypeCheckError::GenericCountMismatch {
                                    item: primitive_type.name().to_string(),
                                    expected: 0,
                                    found: turbofish.generics.len(),
                                    location: turbofish.location,
                                },
                            ));
                        }
                        Vec::new()
                    }
                    PrimitiveType::Str => {
                        if let Some(turbofish) = turbofish {
                            let item = StrPrimitiveType;
                            let item_generic_kinds = item.generic_kinds(self.interner);
                            let kind = item_generic_kinds[0].clone();
                            let generics = vec![self.interner.next_type_variable_with_kind(kind)];
                            self.resolve_item_turbofish_generics(
                                item.item_kind(),
                                &item.item_name(self.interner),
                                item_generic_kinds,
                                generics,
                                Some(turbofish.generics),
                                turbofish.location,
                            )
                        } else {
                            Vec::new()
                        }
                    }
                    PrimitiveType::Fmtstr => {
                        if let Some(turbofish) = turbofish {
                            let item_generic_kinds =
                                FmtstrPrimitiveType {}.generic_kinds(self.interner);
                            let kind = item_generic_kinds[0].clone();
                            let generics = vec![self.interner.next_type_variable_with_kind(kind)];
                            self.resolve_item_turbofish_generics(
                                "primitive type",
                                "fmtstr",
                                item_generic_kinds,
                                generics,
                                Some(turbofish.generics),
                                turbofish.location,
                            )
                        } else {
                            Vec::new()
                        }
                    }
                };
                (generics, None)
            }
            PathResolutionItem::Method(_, None, _)
            | PathResolutionItem::TraitFunction(_, None, _)
            | PathResolutionItem::Module(..)
            | PathResolutionItem::Type(..)
            | PathResolutionItem::TypeAlias(..)
            | PathResolutionItem::PrimitiveType(..)
            | PathResolutionItem::Trait(..)
            | PathResolutionItem::Global(..)
            | PathResolutionItem::ModuleFunction(..) => (Vec::new(), None),
        }
    }

    fn resolve_variable(&mut self, path: TypedPath) -> (HirIdent, Option<PathResolutionItem>) {
        if let Some(trait_path_resolution) = self.resolve_trait_generic_path(&path) {
            for error in trait_path_resolution.errors {
                self.push_err(error);
            }

            return (
                HirIdent {
                    location: path.location,
                    id: self.interner.trait_method_id(trait_path_resolution.method.method_id),
                    impl_kind: ImplKind::TraitMethod(trait_path_resolution.method),
                },
                trait_path_resolution.item,
            );
        }

        // If the Path is being used as an Expression, then it is referring to a global from a separate module
        // Otherwise, then it is referring to an Identifier
        // This lookup allows support of such statements: let x = foo::bar::SOME_GLOBAL + 10;
        // If the expression is a singular indent, we search the resolver's current scope as normal.
        let location = path.location;
        let ((hir_ident, var_scope_index), item) = self.get_ident_from_path(path);

        self.handle_hir_ident(&hir_ident, var_scope_index, location);

        (hir_ident, item)
    }

    pub(crate) fn handle_hir_ident(
        &mut self,
        hir_ident: &HirIdent,
        var_scope_index: usize,
        location: Location,
    ) {
        if hir_ident.id == DefinitionId::dummy_id() {
            return;
        }

        match self.interner.definition(hir_ident.id).kind {
            DefinitionKind::Function(func_id) => {
                if let Some(current_item) = self.current_item {
                    self.interner.add_function_dependency(current_item, func_id);
                }

                self.interner.add_function_reference(func_id, hir_ident.location);
            }
            DefinitionKind::Global(global_id) => {
                self.elaborate_global_if_unresolved(&global_id);
                if let Some(current_item) = self.current_item {
                    self.interner.add_global_dependency(current_item, global_id);
                }

                self.interner.add_global_reference(global_id, hir_ident.location);
            }
            DefinitionKind::NumericGeneric(_, ref numeric_typ) => {
                // Initialize numeric generics to a polymorphic integer type in case
                // they're used in expressions. We must do this here since type_check_variable
                // does not check definition kinds and otherwise expects parameters to
                // already be typed.
                if self.interner.definition_type(hir_ident.id) == Type::Error {
                    let type_var_kind = Kind::Numeric(numeric_typ.clone());
                    let typ = self.type_variable_with_kind(type_var_kind);
                    self.interner.push_definition_type(hir_ident.id, typ);
                }
            }
            DefinitionKind::Local(_) => {
                // only local variables can be captured by closures.
                self.resolve_local_variable(hir_ident.clone(), var_scope_index);

                self.interner.add_local_reference(hir_ident.id, location);
            }
            DefinitionKind::AssociatedConstant(..) => {
                // Nothing to do here
            }
        }
    }

    pub(crate) fn type_check_variable(
        &mut self,
        ident: HirIdent,
        expr_id: ExprId,
        generics: Option<Vec<Type>>,
    ) -> Type {
        let bindings = TypeBindings::default();
        self.type_check_variable_with_bindings(ident, expr_id, generics, bindings)
    }

    pub(super) fn type_check_variable_with_bindings(
        &mut self,
        ident: HirIdent,
        expr_id: ExprId,
        generics: Option<Vec<Type>>,
        mut bindings: TypeBindings,
    ) -> Type {
        // Add type bindings from any constraints that were used.
        // We need to do this first since otherwise instantiating the type below
        // will replace each trait generic with a fresh type variable, rather than
        // the type used in the trait constraint (if it exists). See #4088.
        if let ImplKind::TraitMethod(method) = &ident.impl_kind {
            self.bind_generics_from_trait_constraint(
                &method.constraint,
                method.assumed,
                &mut bindings,
            );
        }

        // An identifiers type may be forall-quantified in the case of generic functions.
        // E.g. `fn foo<T>(t: T, field: Field) -> T` has type `forall T. fn(T, Field) -> T`.
        // We must instantiate identifiers at every call site to replace this T with a new type
        // variable to handle generic functions.
        let t = self.interner.id_type_substitute_trait_as_type(ident.id);

        let definition = self.interner.try_definition(ident.id);
        let function_generic_count = definition.map_or(0, |definition| match &definition.kind {
            DefinitionKind::Function(function) => {
                self.interner.function_modifiers(function).generic_count
            }
            _ => 0,
        });

        let location = self.interner.expr_location(&expr_id);

        // This instantiates a trait's generics as well which need to be set
        // when the constraint below is later solved for when the function is
        // finished. How to link the two?
        let (typ, bindings) =
            self.instantiate(t, bindings, generics, function_generic_count, location);

        // Push any trait constraints required by this definition to the context
        // to be checked later when the type of this variable is further constrained.
        if let Some(definition) = self.interner.try_definition(ident.id) {
            if let DefinitionKind::Function(function) = definition.kind {
                let function = self.interner.function_meta(&function);
                for mut constraint in function.all_trait_constraints().cloned().collect::<Vec<_>>()
                {
                    constraint.apply_bindings(&bindings);

                    self.push_trait_constraint(
                        constraint, expr_id,
                        false, // This constraint shouldn't lead to choosing a trait impl method
                    );
                }
            }
        }

        if let ImplKind::TraitMethod(mut method) = ident.impl_kind {
            method.constraint.apply_bindings(&bindings);
            if method.assumed {
                let trait_generics = method.constraint.trait_bound.trait_generics.clone();
                let object_type = method.constraint.typ;
                let trait_impl = TraitImplKind::Assumed { object_type, trait_generics };
                self.interner.select_impl_for_expression(expr_id, trait_impl);
            } else {
                // Currently only one impl can be selected per expr_id, so this
                // constraint needs to be pushed after any other constraints so
                // that monomorphization can resolve this trait method to the correct impl.
                self.push_trait_constraint(
                    method.constraint,
                    expr_id,
                    true, // this constraint should lead to choosing a trait impl method
                );
            }
        }

        self.interner.store_instantiation_bindings(expr_id, bindings);
        typ
    }

    fn instantiate(
        &mut self,
        typ: Type,
        bindings: TypeBindings,
        turbofish_generics: Option<Vec<Type>>,
        function_generic_count: usize,
        location: Location,
    ) -> (Type, TypeBindings) {
        match turbofish_generics {
            Some(turbofish_generics) => {
                if turbofish_generics.len() != function_generic_count {
                    let type_check_err = TypeCheckError::IncorrectTurbofishGenericCount {
                        expected_count: function_generic_count,
                        actual_count: turbofish_generics.len(),
                        location,
                    };
                    self.push_err(CompilationError::TypeError(type_check_err));
                    typ.instantiate_with_bindings(bindings, self.interner)
                } else {
                    // Fetch the count of any implicit generics on the function, such as
                    // for a method within a generic impl.
                    let implicit_generic_count = match &typ {
                        Type::Forall(generics, _) => generics.len() - function_generic_count,
                        _ => 0,
                    };
                    typ.instantiate_with(turbofish_generics, self.interner, implicit_generic_count)
                }
            }
            None => typ.instantiate_with_bindings(bindings, self.interner),
        }
    }

    pub(crate) fn get_ident_from_path(
        &mut self,
        path: TypedPath,
    ) -> ((HirIdent, usize), Option<PathResolutionItem>) {
        let location = Location::new(path.last_ident().span(), path.location.file);
        let use_variable_result = path.as_single_segment().map(|segment| {
            let result = self.use_variable(&segment.ident);
            if result.is_ok() && segment.generics.is_some() {
                let item = "local variables".to_string();
                let location = segment.turbofish_location();
                self.push_err(PathResolutionError::TurbofishNotAllowedOnItem { item, location });
            }
            result
        });

        let error = match use_variable_result {
            Some(Ok(found)) => return (found, None),
            // Try to look it up as a global, but still issue the first error if we fail
            Some(Err(error)) => match self.lookup_global(path) {
                Ok((id, item)) => {
                    return ((HirIdent::non_trait_method(id, location), 0), Some(item));
                }
                Err(_) => error,
            },
            None => match self.lookup_global(path) {
                Ok((id, item)) => {
                    return ((HirIdent::non_trait_method(id, location), 0), Some(item));
                }
                Err(error) => error,
            },
        };
        self.push_err(error);
        let id = DefinitionId::dummy_id();
        ((HirIdent::non_trait_method(id, location), 0), None)
    }

    pub(super) fn elaborate_type_path(&mut self, path: TypePath) -> (ExprId, Type) {
        let typ_location = path.typ.location;
        let turbofish = path.turbofish;
        let typ = self.use_type(path.typ);
        self.elaborate_type_path_impl(typ, path.item, turbofish, typ_location)
    }

    fn elaborate_type_path_impl(
        &mut self,
        typ: Type,
        ident: Ident,
        turbofish: Option<GenericTypeArgs>,
        typ_location: Location,
    ) -> (ExprId, Type) {
        let ident_location = ident.location();
        let check_self_param = false;

        self.interner.push_type_ref_location(&typ, typ_location);

        let Some(method) = self.lookup_method(
            &typ,
            ident.as_str(),
            ident_location,
            typ_location,
            check_self_param,
        ) else {
            let error = Expression::new(ExpressionKind::Error, ident_location);
            return self.elaborate_expression(error);
        };

        let func_id = method
            .func_id(self.interner)
            .expect("Expected trait function to be a DefinitionKind::Function");

        let generics =
            turbofish.map(|turbofish| self.use_type_args(turbofish, func_id, ident_location).0);

        let id = self.interner.function_definition_id(func_id);

        let impl_kind = match method {
            HirMethodReference::FuncId(_) => ImplKind::NotATraitMethod,
            HirMethodReference::TraitMethodId(method_id, generics, _) => {
                let mut constraint =
                    self.interner.get_trait(method_id.trait_id).as_constraint(ident_location);
                constraint.trait_bound.trait_generics = generics;
                ImplKind::TraitMethod(TraitMethod { method_id, constraint, assumed: false })
            }
        };

        let ident = HirIdent { location: ident_location, id, impl_kind };
        let id = self.interner.push_expr(HirExpression::Ident(ident.clone(), generics.clone()));
        self.interner.push_expr_location(id, ident_location);

        let typ = self.type_check_variable(ident, id, generics);
        self.interner.push_expr_type(id, typ.clone());

        (id, typ)
    }
}

fn get_type_alias_generics(type_alias: &TypeAlias, generics: &[Type]) -> Vec<Type> {
    let typ = type_alias.get_type(generics);
    match typ {
        Type::DataType(_, generics) => generics,
        Type::Alias(type_alias, generics) => {
            get_type_alias_generics(&type_alias.borrow(), &generics)
        }
        _ => panic!("Expected type alias to point to struct or alias"),
    }
}
