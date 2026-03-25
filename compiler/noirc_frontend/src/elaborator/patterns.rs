//! Pattern elaboration, variable binding, and turbofish generic resolution.

use iter_extended::vecmap;
use noirc_errors::{Located, Location};
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

use crate::elaborator::scope::ItemAsValue;
use crate::hir::def_collector::dc_crate::CompilationError;
use crate::node_interner::DefinitionId;
use crate::{
    DataType, Kind, Type, TypeAlias,
    ast::{ERROR_IDENT, Ident, ItemVisibility, Path, PathSegment, Pattern},
    elaborator::{Turbofish, types::WildcardAllowed},
    hir::{
        resolution::{errors::ResolverError, import::PathResolutionError},
        type_check::{Source, TypeCheckError},
    },
    hir_def::{expr::HirIdent, stmt::HirPattern},
    node_interner::{DefinitionKind, FuncId, TypeAliasId, TypeId},
};

use super::{
    Elaborator, ResolverMeta,
    path_resolution::{PathResolutionItem, TypedPath, TypedPathSegment},
};

/// Represents a variable in the source code.
pub(crate) struct Variable {
    /// The identifier of the variable.
    pub(crate) ident: HirIdent,
    /// The scope index where the variable is declared.
    pub(crate) scope: usize,
}

/// The result of [`Elaborator::get_ident_from_path`] and [`Elaborator::get_ident_from_path_or_error`].
pub(crate) enum IdentFromPath {
    /// A variable was found.
    Variable(Variable),
    /// A definition was found.
    Definition { id: DefinitionId, item: PathResolutionItem },
    /// A type alias that is numeric, infinitely recursive or one that errored, was found.
    TypeAlias(TypeAliasId),
}

impl Elaborator<'_> {
    /// Elaborate a pattern, which can appear in a `let <pattern> = <expr>`, or a `match` statement.
    ///
    /// The `definition_kind` specifies the kind of variables the pattern will create, e.g. a local or global variable.
    ///
    /// The `expected_type` is always known, because we can first infer the type of the `<expr>` and try to match it to
    /// the pattern.
    ///
    /// `parameter_names_in_list` keeps track of parameter names, and their location, across multiple
    /// patterns in a list. If a name is found multiple times, an error is captured.
    pub(super) fn elaborate_pattern(
        &mut self,
        pattern: Pattern,
        expected_type: Type,
        definition_kind: DefinitionKind,
        warn_if_unused: bool,
        warn_if_not_mutated: bool,
        parameter_names_in_list: &mut HashMap<String, Location>,
    ) -> HirPattern {
        self.elaborate_pattern_mut(
            pattern,
            expected_type,
            definition_kind,
            None,
            &mut Vec::new(),
            warn_if_unused,
            warn_if_not_mutated,
            &mut HashSet::default(),
            parameter_names_in_list,
        )
    }

    /// Equivalent to `elaborate_pattern`, this version just also
    /// adds any new `DefinitionIds` that were created to the given `Vec`.
    ///
    /// `parameter_names_in_list` keeps track of parameter names, and their location, across multiple
    /// patterns in a list. If a name is found multiple times, an error is captured.
    #[allow(clippy::too_many_arguments)]
    pub fn elaborate_pattern_and_store_ids(
        &mut self,
        pattern: Pattern,
        expected_type: Type,
        definition_kind: DefinitionKind,
        created_ids: &mut Vec<HirIdent>,
        warn_if_unused: bool,
        warn_if_not_mutated: bool,
        parameter_names_in_list: &mut HashMap<String, Location>,
    ) -> HirPattern {
        self.elaborate_pattern_mut(
            pattern,
            expected_type,
            definition_kind,
            None,
            created_ids,
            warn_if_unused,
            warn_if_not_mutated,
            &mut HashSet::default(),
            parameter_names_in_list,
        )
    }

    /// Elaborate the (potentially mutable) pattern and add the variables
    /// it created to the scope if necessary.
    ///
    /// - `pattern_names` keeps track of parameter names within this single pattern (or an outer
    ///   one, when called recursively). If a name is found multiple times, an error is captured.
    /// - `parameter_names_in_list` keeps track of parameter names, and their location, across multiple
    ///   patterns in a list. If a name is found multiple times, an error is captured.
    #[allow(clippy::too_many_arguments)]
    fn elaborate_pattern_mut(
        &mut self,
        pattern: Pattern,
        expected_type: Type,
        definition: DefinitionKind,
        // Location of the `mut` keyword.
        mutable: Option<Location>,
        new_definitions: &mut Vec<HirIdent>,
        warn_if_unused: bool,
        warn_if_not_mutated: bool,
        pattern_names: &mut HashSet<String>,
        parameter_names_in_list: &mut HashMap<String, Location>,
    ) -> HirPattern {
        match pattern {
            // e.g. let <ident> = ...;
            Pattern::Identifier(name) => {
                // If this definition is mutable, do not store the RHS because it will
                // not always refer to the correct value of the variable.
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
                    if name.as_str() != "_" {
                        if !pattern_names.insert(name.to_string()) {
                            self.push_err(ResolverError::PatternBoundMoreThanOnce {
                                ident: name.clone(),
                            });
                        } else if let Some(first_location) =
                            parameter_names_in_list.insert(name.to_string(), name.location())
                        {
                            self.push_err(ResolverError::DuplicateDefinition {
                                name: name.to_string(),
                                first_location,
                                second_location: name.location(),
                            });
                        }
                    }

                    self.add_variable_decl(
                        name,
                        mutable.is_some(),
                        true, // allow_shadowing
                        warn_if_unused,
                        warn_if_not_mutated,
                        definition,
                    )
                };
                self.interner.push_definition_type(ident.id, expected_type);
                new_definitions.push(ident.clone());
                HirPattern::Identifier(ident)
            }
            // e.g. let mut <pattern> = ...;
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
                    warn_if_not_mutated,
                    pattern_names,
                    parameter_names_in_list,
                );
                HirPattern::Mutable(Box::new(pattern), location)
            }
            // e.g. let (<pattern 0>, <pattern 1>, ...) = ...;
            Pattern::Tuple(fields, location) => {
                // Returns Some for valid tuple types (where arity checking makes sense),
                // None when we've already issued an error or have an invalid type.
                let field_types = match expected_type.follow_bindings() {
                    Type::Tuple(fields) => Some(fields),
                    Type::Error => None,
                    expected_type => {
                        let tuple =
                            Type::Tuple(vecmap(&fields, |_| self.interner.next_type_variable()));

                        self.push_err(TypeCheckError::TypeMismatchWithSource {
                            expected: expected_type,
                            actual: tuple,
                            location,
                            source: Source::Assignment,
                        });
                        None
                    }
                };

                // Only check tuple arity if the expected type was actually a tuple.
                // If it wasn't, we've already issued a type mismatch error above.
                if let Some(field_types) = &field_types
                    && fields.len() != field_types.len()
                {
                    self.push_err(TypeCheckError::TupleMismatch {
                        tuple_types: field_types.clone(),
                        actual_count: fields.len(),
                        location,
                    });
                }

                let fields = vecmap(fields.into_iter().enumerate(), |(i, field)| {
                    let field_type = field_types
                        .as_ref()
                        .and_then(|types| types.get(i).cloned())
                        .unwrap_or(Type::Error);
                    self.elaborate_pattern_mut(
                        field,
                        field_type,
                        definition.clone(),
                        mutable,
                        new_definitions,
                        warn_if_unused,
                        warn_if_not_mutated,
                        pattern_names,
                        parameter_names_in_list,
                    )
                });
                HirPattern::Tuple(fields, location)
            }
            // e.g. let <name> { <field 0>: <pattern 0>, <field 1>: <pattern 0>, ... } = ...'
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
                    pattern_names,
                    parameter_names_in_list,
                )
            }
            // e.g. let (<pattern>) = ...;
            Pattern::Parenthesized(pattern, _) => self.elaborate_pattern_mut(
                *pattern,
                expected_type,
                definition,
                mutable,
                new_definitions,
                warn_if_unused,
                warn_if_not_mutated,
                pattern_names,
                parameter_names_in_list,
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
                    warn_if_not_mutated,
                    pattern_names,
                    parameter_names_in_list,
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
        pattern_names: &mut HashSet<String>,
        parameter_names_in_list: &mut HashMap<String, Location>,
    ) -> HirPattern {
        let last_segment = name.last_segment();
        let name_location = last_segment.ident.location();
        let is_self_type = last_segment.ident.is_self_type_name();

        let error_identifier = |this: &mut Self| {
            // Must create a name here to return a HirPattern::Identifier. Allowing
            // shadowing here lets us avoid further errors if we define ERROR_IDENT
            // multiple times.
            let name = ERROR_IDENT.into();
            let identifier =
                this.add_variable_decl(name, false, true, true, true, definition.clone());
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

        let mut errors = Vec::new();
        let generics = self.resolve_struct_turbofish_generics(
            &struct_type.borrow(),
            generics,
            last_segment.generics,
            turbofish_location,
            &mut errors,
        );
        self.push_errors(errors);

        let actual_type = Type::DataType(struct_type.clone(), generics);

        self.unify(&actual_type, &expected_type, || TypeCheckError::TypeMismatchWithSource {
            expected: expected_type.clone(),
            actual: actual_type.clone(),
            location,
            source: Source::Assignment,
        });

        let fields = self.resolve_constructor_pattern_fields(
            fields,
            location,
            actual_type.clone(),
            definition,
            mutable,
            new_definitions,
            pattern_names,
            parameter_names_in_list,
        );

        let struct_id = struct_type.borrow().id;

        self.interner.add_type_reference(struct_id, name_location, is_self_type);

        for (field_index, field) in fields.iter().enumerate() {
            let reference_location = field.0.location();
            self.interner.add_struct_member_reference(struct_id, field_index, reference_location);
        }

        HirPattern::Struct(actual_type, fields, location)
    }

    /// Resolve all the fields of a struct constructor expression.
    /// Ensures all fields are present, none are repeated, and all
    /// are part of the struct.
    #[allow(clippy::too_many_arguments)]
    fn resolve_constructor_pattern_fields(
        &mut self,
        fields: Vec<(Ident, Pattern)>,
        location: Location,
        typ: Type,
        definition: DefinitionKind,
        mutable: Option<Location>,
        new_definitions: &mut Vec<HirIdent>,
        pattern_names: &mut HashSet<String>,
        parameter_names_in_list: &mut HashMap<String, Location>,
    ) -> Vec<(Ident, HirPattern)> {
        let mut ret = Vec::with_capacity(fields.len());
        let mut seen_fields = HashSet::default();
        let Type::DataType(struct_type, _) = &typ else {
            unreachable!("Should be validated as struct before getting here")
        };
        let mut unseen_fields = struct_type
            .borrow()
            .field_names()
            .expect("This type should already be validated to be a struct");

        for (field, pattern) in fields {
            let (field_type, visibility) = typ
                .get_field_type_and_visibility(field.as_str())
                .unwrap_or((Type::Error, ItemVisibility::Public));
            let resolved = self.elaborate_pattern_mut(
                pattern,
                field_type,
                definition.clone(),
                mutable,
                new_definitions,
                true, // warn_if_unused
                true, // warn_if_not_mutated
                pattern_names,
                parameter_names_in_list,
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

    /// Add a local or const numeric variable declaration to the scope,
    /// unless the name is `"_"`.
    ///
    /// Returns the created identifier.
    ///
    /// Panics if the `definition` is [DefinitionKind::Global].
    pub(super) fn add_variable_decl(
        &mut self,
        name: Ident,
        mutable: bool,
        allow_shadowing: bool,
        warn_if_unused: bool,
        warn_if_not_mutated: bool,
        definition: DefinitionKind,
    ) -> HirIdent {
        if let DefinitionKind::Global(_) = definition {
            unreachable!("ICE: globals don't need to be added to the scope");
        }

        let location = name.location();
        let name = name.into_string();
        let comptime = self.in_comptime_context();
        let id =
            self.interner.push_definition(name.clone(), mutable, comptime, definition, location);
        let ident = HirIdent::non_trait_method(id, location);

        self.add_existing_variable_to_scope(
            name,
            ident.clone(),
            warn_if_unused,
            warn_if_not_mutated,
            allow_shadowing,
        );

        ident
    }

    /// Add a [ResolverMeta] to the last scope for a given [HirIdent], which already has its definition interned,
    /// unless its name is `"_"`.
    pub fn add_existing_variable_to_scope(
        &mut self,
        name: String,
        ident: HirIdent,
        warn_if_unused: bool,
        warn_if_not_mutated: bool,
        allow_shadowing: bool,
    ) {
        if name == "_" {
            return;
        }

        let second_location = ident.location;
        let resolver_meta = ResolverMeta {
            used: false,
            mutated: false,
            ident,
            warn_if_unused,
            warn_if_not_mutated,
        };

        let old_value = self.scopes.get_mut_scope().add_key_value(name.clone(), resolver_meta);

        if !allow_shadowing && let Some(old_value) = old_value {
            self.push_err(ResolverError::DuplicateDefinition {
                name,
                first_location: old_value.ident.location,
                second_location,
            });
        }
    }

    /// Lookup and use the specified local variable.
    /// This will increment its use counter by one and return the variable if found.
    /// If the variable is not found, an error is returned.
    ///
    /// This method is private and is expected to be called through [Self::get_ident_from_path_or_error].
    fn use_variable(&mut self, name: &Ident) -> Result<Variable, ResolverError> {
        // Find the definition for this Ident
        let scope_tree = self.scopes.current_scope_tree();
        let variable = scope_tree.find(name.as_str());

        let location = name.location();
        if let Some((variable_found, scope)) = variable {
            variable_found.used = true;
            let id = variable_found.ident.id;
            let ident = HirIdent::non_trait_method(id, location);
            let variable = Variable { ident, scope };
            Ok(variable)
        } else if self.parent_runtime_variables.contains(name.as_str()) {
            Err(ResolverError::RuntimeVarReferencedInComptime {
                name: name.to_string(),
                location: name.location(),
            })
        } else {
            Err(ResolverError::VariableNotDeclared {
                name: name.to_string(),
                location: name.location(),
            })
        }
    }

    /// Resolve generics using the expected kinds of the function we are calling.
    ///
    /// Looks up the generics of the function in [FuncMeta][crate::hir_def::function::FuncMeta].
    ///
    /// If there is no turbofish, it returns `None`.
    pub(super) fn resolve_function_turbofish_generics(
        &mut self,
        func_id: &FuncId,
        resolved_turbofish: Option<Vec<Located<Type>>>,
        location: Location,
    ) -> Option<Vec<Type>> {
        resolved_turbofish.map(|mut resolved_turbofish| {
            let direct_generic_kinds =
                vecmap(&self.interner.function_meta(func_id).direct_generics, |generic| {
                    generic.kind()
                });
            let expected = direct_generic_kinds.len();
            let actual = resolved_turbofish.len();

            if actual != expected {
                self.push_err(TypeCheckError::IncorrectTurbofishGenericCount {
                    expected_count: expected,
                    actual_count: actual,
                    location,
                });

                // Pad so the result has the expected length for future checks
                resolved_turbofish.resize(expected, Located::from(location, Type::Error));
            }

            self.resolve_turbofish_generics(direct_generic_kinds, resolved_turbofish)
        })
    }

    /// Resolve generics using the generic kinds of a struct [DataType].
    ///
    /// If there are no turbofish, returns the generics of the struct itself, as constructed by the caller.
    pub(super) fn resolve_struct_turbofish_generics(
        &mut self,
        struct_type: &DataType,
        generics: Vec<Type>,
        resolved_turbofish: Option<Vec<Located<Type>>>,
        location: Location,
        errors: &mut Vec<CompilationError>,
    ) -> Vec<Type> {
        let kinds = vecmap(&struct_type.generics, |generic| generic.kind());
        self.resolve_item_turbofish_generics(
            "struct",
            struct_type.name.as_str(),
            kinds,
            generics,
            resolved_turbofish,
            location,
            errors,
        )
    }

    /// Resolve generics using the generics and generic kinds of a [Trait][crate::hir_def::traits::Trait].
    ///
    /// If there are no turbofish, returns the generics of the trait itself, as constructed by the caller.
    pub(super) fn resolve_trait_turbofish_generics(
        &mut self,
        trait_name: &str,
        trait_generic_kinds: Vec<Kind>,
        generics: Vec<Type>,
        resolved_turbofish: Option<Vec<Located<Type>>>,
        location: Location,
        errors: &mut Vec<CompilationError>,
    ) -> Vec<Type> {
        self.resolve_item_turbofish_generics(
            "trait",
            trait_name,
            trait_generic_kinds,
            generics,
            resolved_turbofish,
            location,
            errors,
        )
    }

    /// Resolve generics using the generic and generic kinds of a [TypeAlias].
    ///
    /// If there are no turbofish, returns the generics of the trait itself, as constructed by the caller.
    pub(super) fn resolve_alias_turbofish_generics(
        &mut self,
        type_alias: &TypeAlias,
        generics: Vec<Type>,
        resolved_turbofish: Option<Vec<Located<Type>>>,
        location: Location,
        errors: &mut Vec<CompilationError>,
    ) -> Vec<Type> {
        let kinds = vecmap(&type_alias.generics, |generic| generic.kind());
        self.resolve_item_turbofish_generics(
            "alias",
            type_alias.name.as_str(),
            kinds,
            generics,
            resolved_turbofish,
            location,
            errors,
        )
    }

    /// Given the generic [Kind]s of a type and its own declared generic [Type]s,
    /// check if we have a non-empty turbofish with the expected number of generics,
    /// and if so try unify them with the expected kinds, otherwise return the default
    /// generics of the type.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn resolve_item_turbofish_generics(
        &mut self,
        item_kind: &'static str,
        item_name: &str,
        item_generic_kinds: Vec<Kind>,
        generics: Vec<Type>,
        resolved_turbofish: Option<Vec<Located<Type>>>,
        location: Location,
        errors: &mut Vec<CompilationError>,
    ) -> Vec<Type> {
        assert_eq!(
            generics.len(),
            item_generic_kinds.len(),
            "ICE: generics count should match the expected kinds"
        );

        let Some(turbofish_generics) = resolved_turbofish else {
            return generics;
        };

        if turbofish_generics.len() != generics.len() {
            errors.push(
                TypeCheckError::GenericCountMismatch {
                    item: format!("{item_kind} {item_name}"),
                    expected: generics.len(),
                    found: turbofish_generics.len(),
                    location,
                }
                .into(),
            );
            return generics;
        }

        self.resolve_turbofish_generics(item_generic_kinds, turbofish_generics)
    }

    /// Given the generic [Kind]s of a type, and the list of generic types in a non-empty turbofish,
    /// which have already been verified to match the expected number of generics, run type checking
    /// to ensure each turbofish generic matches the expected kind, and return the unified types.
    fn resolve_turbofish_generics(
        &mut self,
        kinds: Vec<Kind>,
        turbofish_generics: Vec<Located<Type>>,
    ) -> Vec<Type> {
        // Use zip (not zip_eq) since callers like resolve_function_turbofish_generics
        // may push an error for mismatched counts but still call this function.
        let kinds_with_types = kinds.into_iter().zip(turbofish_generics);

        vecmap(kinds_with_types, |(kind, located_type)| {
            let location = located_type.location();
            let typ = located_type.contents;
            self.check_type_kind(typ, &kind, location)
        })
    }

    /// Create a validated [TypedPath] from a [Path] by resolving all generics in every [PathSegment] in it.
    ///
    /// Pushes an error if the first segment is `Self` and it has turbofish generics.
    pub(crate) fn validate_path(&mut self, path: Path) -> TypedPath {
        let mut segments = vecmap(path.segments, |segment| self.validate_path_segment(segment));

        if let Some(first_segment) = segments.first_mut()
            && first_segment.generics.is_some()
            && first_segment.ident.is_self_type_name()
        {
            self.push_err(PathResolutionError::TurbofishNotAllowedOnItem {
                item: "self type".to_string(),
                location: first_segment.turbofish_location(),
            });
            first_segment.generics = None;
        }

        TypedPath {
            segments,
            kind: path.kind,
            location: path.location,
            kind_location: path.kind_location,
        }
    }

    /// Create a validated [TypedPathSegment] from a [PathSegment] by resolving all turbofish generics
    /// in it with [Kind::Any], allowing wildcards, and marking them as _used_.
    fn validate_path_segment(&mut self, segment: PathSegment) -> TypedPathSegment {
        let generics = segment.generics.map(|generics| {
            vecmap(generics, |generic| {
                let location = generic.location;
                let wildcard_allowed = WildcardAllowed::Yes;
                let typ = self.use_type_with_kind(generic, &Kind::Any, wildcard_allowed);
                Located::from(location, typ)
            })
        });
        TypedPathSegment { ident: segment.ident, generics, location: segment.location }
    }

    /// Get the [DataType] of a [TypeId] and call [Elaborator::resolve_struct_turbofish_generics].
    pub(super) fn resolve_struct_id_turbofish_generics(
        &mut self,
        struct_id: TypeId,
        mut turbofish: Option<Turbofish>,
        errors: &mut Vec<CompilationError>,
    ) -> Vec<Type> {
        let struct_type = self.interner.get_type(struct_id);
        let struct_type = struct_type.borrow();
        let struct_generics = struct_type.instantiate(self.interner);
        if let Some(turbofish) = turbofish.take() {
            self.resolve_struct_turbofish_generics(
                &struct_type,
                struct_generics,
                Some(turbofish.generics),
                turbofish.location,
                errors,
            )
        } else {
            struct_generics
        }
    }

    /// Get the [TypeAlias] of a [TypeAliasId] and call [Elaborator::resolve_alias_turbofish_generics].
    pub(super) fn resolve_type_alias_id_turbofish_generics(
        &mut self,
        type_alias_id: TypeAliasId,
        generics: Option<Turbofish>,
        errors: &mut Vec<CompilationError>,
    ) -> Vec<Type> {
        let type_alias = self.interner.get_type_alias(type_alias_id);
        let type_alias = type_alias.borrow();
        let alias_generics = vecmap(&type_alias.generics, |generic| {
            self.interner.next_type_variable_with_kind(generic.kind())
        });

        if let Some(generics) = generics {
            self.resolve_alias_turbofish_generics(
                &type_alias,
                alias_generics,
                Some(generics.generics),
                generics.location,
                errors,
            )
        } else {
            alias_generics
        }
    }

    /// Resolve a [TypedPath] into a local or global [HirIdent].
    ///
    /// If it cannot be found, then it pushes the error and returns [None].
    pub(crate) fn get_ident_from_path(&mut self, path: TypedPath) -> Option<IdentFromPath> {
        match self.get_ident_from_path_or_error(path) {
            Ok(value) => Some(value),
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    /// Resolve a [TypedPath] into a local or global [HirIdent], or return `Err` if it could not be found.
    pub(crate) fn get_ident_from_path_or_error(
        &mut self,
        path: TypedPath,
    ) -> Result<IdentFromPath, ResolverError> {
        // If the path is a single segment, try to resolve it as a local variable first
        let use_variable_error = match path.as_single_segment() {
            Some(segment) => match self.use_variable(&segment.ident) {
                Ok(variable) => {
                    // Succeed even if the variable has turbofish on it, but report an error for that
                    if segment.generics.is_some() {
                        let item = "local variables".to_string();
                        let location = segment.turbofish_location();
                        let error =
                            PathResolutionError::TurbofishNotAllowedOnItem { item, location };
                        self.push_err(error);
                    }
                    return Ok(IdentFromPath::Variable(variable));
                }
                Err(error) => Some(error),
            },
            None => None,
        };

        match self.lookup_item_as_value(path) {
            Ok(ItemAsValue::Definition { id, item }) => Ok(IdentFromPath::Definition { id, item }),
            Ok(ItemAsValue::TypeAlias(type_alias_id)) => {
                Ok(IdentFromPath::TypeAlias(type_alias_id))
            }
            Err(ResolverError::PathResolutionError(PathResolutionError::Unresolved(ident))) => {
                // If we can't resolve a path, but we have an error from trying to resolve a variable
                // (in which case the path was a single segment), prefer saying "variable not found"
                // instead of "Cannot resolve '...' in path".
                match use_variable_error {
                    Some(error) => Err(error),
                    None => Err(ResolverError::PathResolutionError(
                        PathResolutionError::Unresolved(ident),
                    )),
                }
            }
            Err(item_as_value_error) => {
                match use_variable_error {
                    // If the path was "_" then we want to preserve that error as it's clearer
                    // than the error of an item not being found (it will mention that "_" is
                    // not valid as an expression).
                    Some(ResolverError::VariableNotDeclared { name, .. }) if name != "_" => {
                        Err(item_as_value_error)
                    }
                    Some(use_variable_error) => Err(use_variable_error),
                    None => Err(item_as_value_error),
                }
            }
        }
    }
}
