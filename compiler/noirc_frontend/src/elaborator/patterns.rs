//! Pattern elaboration, variable binding, and turbofish generic resolution.

use iter_extended::vecmap;
use noirc_errors::{Located, Location};
use rustc_hash::FxHashSet as HashSet;

use crate::{
    DataType, Kind, Shared, Type, TypeAlias,
    ast::{ERROR_IDENT, Ident, ItemVisibility, Path, PathSegment, Pattern},
    elaborator::Turbofish,
    hir::{
        resolution::{errors::ResolverError, import::PathResolutionError},
        type_check::{Source, TypeCheckError},
    },
    hir_def::{expr::HirIdent, stmt::HirPattern},
    node_interner::{DefinitionId, DefinitionKind, FuncId, GlobalId, TypeAliasId, TypeId},
};

use super::{
    Elaborator, ResolverMeta,
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
                let wildcard_allowed = true;
                let typ = self.use_type_with_kind(generic, &Kind::Any, wildcard_allowed);
                Located::from(location, typ)
            })
        });
        TypedPathSegment { ident: segment.ident, generics, location: segment.location }
    }

    pub(super) fn resolve_struct_id_turbofish_generics(
        &mut self,
        struct_id: TypeId,
        mut turbofish: Option<Turbofish>,
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
            )
        } else {
            struct_generics
        }
    }

    pub(super) fn resolve_type_alias_id_turbofish_generics(
        &mut self,
        type_alias_id: TypeAliasId,
        generics: Option<Turbofish>,
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
            )
        } else {
            alias_generics
        }
    }

    pub(crate) fn get_ident_from_path(
        &mut self,
        path: TypedPath,
    ) -> ((HirIdent, usize), Option<PathResolutionItem>) {
        let location = Location::new(path.last_ident().span(), path.location.file);

        self.get_ident_from_path_or_error(path).unwrap_or_else(|error| {
            self.push_err(error);
            let id = DefinitionId::dummy_id();
            ((HirIdent::non_trait_method(id, location), 0), None)
        })
    }

    pub(crate) fn get_ident_from_path_or_error(
        &mut self,
        path: TypedPath,
    ) -> Result<((HirIdent, usize), Option<PathResolutionItem>), ResolverError> {
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
            Some(Ok(found)) => return Ok((found, None)),
            // Try to look it up as a global, but still issue the first error if we fail
            Some(Err(error)) => match self.lookup_global(path) {
                Ok((id, item)) => {
                    return Ok(((HirIdent::non_trait_method(id, location), 0), Some(item)));
                }
                Err(_) => error,
            },
            None => match self.lookup_global(path) {
                Ok((dummy_id, PathResolutionItem::TypeAlias(type_alias_id)))
                    if dummy_id == DefinitionId::dummy_id() =>
                {
                    // Allow path which resolves to a type alias
                    return Ok((
                        (HirIdent::non_trait_method(dummy_id, location), 4),
                        Some(PathResolutionItem::TypeAlias(type_alias_id)),
                    ));
                }
                Ok((id, item)) => {
                    return Ok(((HirIdent::non_trait_method(id, location), 0), Some(item)));
                }
                Err(error) => error,
            },
        };

        Err(error)
    }
}
