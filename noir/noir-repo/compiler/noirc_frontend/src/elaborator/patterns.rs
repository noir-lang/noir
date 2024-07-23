use iter_extended::vecmap;
use noirc_errors::{Location, Span};
use rustc_hash::FxHashSet as HashSet;

use crate::{
    ast::{UnresolvedType, ERROR_IDENT},
    hir::{
        comptime::Interpreter,
        def_collector::dc_crate::CompilationError,
        resolution::errors::ResolverError,
        type_check::{Source, TypeCheckError},
    },
    hir_def::{
        expr::{HirIdent, ImplKind},
        stmt::HirPattern,
    },
    macros_api::{HirExpression, Ident, Path, Pattern},
    node_interner::{DefinitionId, DefinitionKind, ExprId, FuncId, GlobalId, TraitImplKind},
    Shared, StructType, Type, TypeBindings,
};

use super::{Elaborator, ResolverMeta};

impl<'context> Elaborator<'context> {
    pub(super) fn elaborate_pattern(
        &mut self,
        pattern: Pattern,
        expected_type: Type,
        definition_kind: DefinitionKind,
    ) -> HirPattern {
        self.elaborate_pattern_mut(
            pattern,
            expected_type,
            definition_kind,
            None,
            &mut Vec::new(),
            None,
        )
    }

    /// Equivalent to `elaborate_pattern`, this version just also
    /// adds any new DefinitionIds that were created to the given Vec.
    pub(super) fn elaborate_pattern_and_store_ids(
        &mut self,
        pattern: Pattern,
        expected_type: Type,
        definition_kind: DefinitionKind,
        created_ids: &mut Vec<HirIdent>,
        global_id: Option<GlobalId>,
    ) -> HirPattern {
        self.elaborate_pattern_mut(
            pattern,
            expected_type,
            definition_kind,
            None,
            created_ids,
            global_id,
        )
    }

    fn elaborate_pattern_mut(
        &mut self,
        pattern: Pattern,
        expected_type: Type,
        definition: DefinitionKind,
        mutable: Option<Span>,
        new_definitions: &mut Vec<HirIdent>,
        global_id: Option<GlobalId>,
    ) -> HirPattern {
        match pattern {
            Pattern::Identifier(name) => {
                // If this definition is mutable, do not store the rhs because it will
                // not always refer to the correct value of the variable
                let definition = match (mutable, definition) {
                    (Some(_), DefinitionKind::Local(_)) => DefinitionKind::Local(None),
                    (_, other) => other,
                };
                let ident = if let Some(global_id) = global_id {
                    // Globals don't need to be added to scope, they're already in the def_maps
                    let id = self.interner.get_global(global_id).definition_id;
                    let location = Location::new(name.span(), self.file);
                    HirIdent::non_trait_method(id, location)
                } else {
                    self.add_variable_decl(name, mutable.is_some(), true, definition)
                };
                self.interner.push_definition_type(ident.id, expected_type);
                new_definitions.push(ident.clone());
                HirPattern::Identifier(ident)
            }
            Pattern::Mutable(pattern, span, _) => {
                if let Some(first_mut) = mutable {
                    self.push_err(ResolverError::UnnecessaryMut { first_mut, second_mut: span });
                }

                let pattern = self.elaborate_pattern_mut(
                    *pattern,
                    expected_type,
                    definition,
                    Some(span),
                    new_definitions,
                    global_id,
                );
                let location = Location::new(span, self.file);
                HirPattern::Mutable(Box::new(pattern), location)
            }
            Pattern::Tuple(fields, span) => {
                let field_types = match expected_type.follow_bindings() {
                    Type::Tuple(fields) => fields,
                    Type::Error => Vec::new(),
                    expected_type => {
                        let tuple =
                            Type::Tuple(vecmap(&fields, |_| self.interner.next_type_variable()));

                        self.push_err(TypeCheckError::TypeMismatchWithSource {
                            expected: expected_type,
                            actual: tuple,
                            span,
                            source: Source::Assignment,
                        });
                        Vec::new()
                    }
                };

                let fields = vecmap(fields.into_iter().enumerate(), |(i, field)| {
                    let field_type = field_types.get(i).cloned().unwrap_or(Type::Error);
                    self.elaborate_pattern_mut(
                        field,
                        field_type,
                        definition.clone(),
                        mutable,
                        new_definitions,
                        global_id,
                    )
                });
                let location = Location::new(span, self.file);
                HirPattern::Tuple(fields, location)
            }
            Pattern::Struct(name, fields, span) => self.elaborate_struct_pattern(
                name,
                fields,
                span,
                expected_type,
                definition,
                mutable,
                new_definitions,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn elaborate_struct_pattern(
        &mut self,
        name: Path,
        fields: Vec<(Ident, Pattern)>,
        span: Span,
        expected_type: Type,
        definition: DefinitionKind,
        mutable: Option<Span>,
        new_definitions: &mut Vec<HirIdent>,
    ) -> HirPattern {
        let name_span = name.last_segment().span();
        let is_self_type = name.last_segment().is_self_type_name();

        let error_identifier = |this: &mut Self| {
            // Must create a name here to return a HirPattern::Identifier. Allowing
            // shadowing here lets us avoid further errors if we define ERROR_IDENT
            // multiple times.
            let name = ERROR_IDENT.into();
            let identifier = this.add_variable_decl(name, false, true, definition.clone());
            HirPattern::Identifier(identifier)
        };

        let (struct_type, generics) = match self.lookup_type_or_error(name) {
            Some(Type::Struct(struct_type, generics)) => (struct_type, generics),
            None => return error_identifier(self),
            Some(typ) => {
                self.push_err(ResolverError::NonStructUsedInConstructor { typ, span });
                return error_identifier(self);
            }
        };

        let actual_type = Type::Struct(struct_type.clone(), generics);
        let location = Location::new(span, self.file);

        self.unify(&actual_type, &expected_type, || TypeCheckError::TypeMismatchWithSource {
            expected: expected_type.clone(),
            actual: actual_type.clone(),
            span: location.span,
            source: Source::Assignment,
        });

        let typ = struct_type.clone();
        let fields = self.resolve_constructor_pattern_fields(
            typ,
            fields,
            span,
            expected_type.clone(),
            definition,
            mutable,
            new_definitions,
        );

        let struct_id = struct_type.borrow().id;

        let reference_location = Location::new(name_span, self.file);
        self.interner.add_struct_reference(struct_id, reference_location, is_self_type);

        for (field_index, field) in fields.iter().enumerate() {
            let reference_location = Location::new(field.0.span(), self.file);
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
        struct_type: Shared<StructType>,
        fields: Vec<(Ident, Pattern)>,
        span: Span,
        expected_type: Type,
        definition: DefinitionKind,
        mutable: Option<Span>,
        new_definitions: &mut Vec<HirIdent>,
    ) -> Vec<(Ident, HirPattern)> {
        let mut ret = Vec::with_capacity(fields.len());
        let mut seen_fields = HashSet::default();
        let mut unseen_fields = struct_type.borrow().field_names();

        for (field, pattern) in fields {
            let field_type = expected_type.get_field_type(&field.0.contents).unwrap_or(Type::Error);
            let resolved = self.elaborate_pattern_mut(
                pattern,
                field_type,
                definition.clone(),
                mutable,
                new_definitions,
                None,
            );

            if unseen_fields.contains(&field) {
                unseen_fields.remove(&field);
                seen_fields.insert(field.clone());
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
                span,
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
        definition: DefinitionKind,
    ) -> HirIdent {
        self.add_variable_decl_inner(name, mutable, allow_shadowing, true, definition)
    }

    pub fn add_variable_decl_inner(
        &mut self,
        name: Ident,
        mutable: bool,
        allow_shadowing: bool,
        warn_if_unused: bool,
        definition: DefinitionKind,
    ) -> HirIdent {
        if definition.is_global() {
            return self.add_global_variable_decl(name, definition);
        }

        let location = Location::new(name.span(), self.file);
        let name = name.0.contents;
        let comptime = self.in_comptime_context();
        let id =
            self.interner.push_definition(name.clone(), mutable, comptime, definition, location);
        let ident = HirIdent::non_trait_method(id, location);
        let resolver_meta =
            ResolverMeta { num_times_used: 0, ident: ident.clone(), warn_if_unused };

        let scope = self.scopes.get_mut_scope();
        let old_value = scope.add_key_value(name.clone(), resolver_meta);

        if !allow_shadowing {
            if let Some(old_value) = old_value {
                self.push_err(ResolverError::DuplicateDefinition {
                    name,
                    first_span: old_value.ident.location.span,
                    second_span: location.span,
                });
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
        let second_span = ident.location.span;
        let resolver_meta = ResolverMeta { num_times_used: 0, ident, warn_if_unused };

        let old_value = self.scopes.get_mut_scope().add_key_value(name.clone(), resolver_meta);

        if let Some(old_value) = old_value {
            let first_span = old_value.ident.location.span;
            self.push_err(ResolverError::DuplicateDefinition { name, first_span, second_span });
        }
    }

    pub fn add_global_variable_decl(
        &mut self,
        name: Ident,
        definition: DefinitionKind,
    ) -> HirIdent {
        let comptime = self.in_comptime_context();
        let scope = self.scopes.get_mut_scope();

        // This check is necessary to maintain the same definition ids in the interner. Currently, each function uses a new resolver that has its own ScopeForest and thus global scope.
        // We must first check whether an existing definition ID has been inserted as otherwise there will be multiple definitions for the same global statement.
        // This leads to an error in evaluation where the wrong definition ID is selected when evaluating a statement using the global. The check below prevents this error.
        let mut global_id = None;
        let global = self.interner.get_all_globals();
        for global_info in global {
            if global_info.local_id == self.local_module && global_info.ident == name {
                global_id = Some(global_info.id);
            }
        }

        let (ident, resolver_meta) = if let Some(id) = global_id {
            let global = self.interner.get_global(id);
            let hir_ident = HirIdent::non_trait_method(global.definition_id, global.location);
            let ident = hir_ident.clone();
            let resolver_meta = ResolverMeta { num_times_used: 0, ident, warn_if_unused: true };
            (hir_ident, resolver_meta)
        } else {
            let location = Location::new(name.span(), self.file);
            let name = name.0.contents.clone();
            let id = self.interner.push_definition(name, false, comptime, definition, location);
            let ident = HirIdent::non_trait_method(id, location);
            let resolver_meta =
                ResolverMeta { num_times_used: 0, ident: ident.clone(), warn_if_unused: true };
            (ident, resolver_meta)
        };

        let old_global_value = scope.add_key_value(name.0.contents.clone(), resolver_meta);
        if let Some(old_global_value) = old_global_value {
            self.push_err(ResolverError::DuplicateDefinition {
                name: name.0.contents.clone(),
                first_span: old_global_value.ident.location.span,
                second_span: name.span(),
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
        let variable = scope_tree.find(&name.0.contents);

        let location = Location::new(name.span(), self.file);
        if let Some((variable_found, scope)) = variable {
            variable_found.num_times_used += 1;
            let id = variable_found.ident.id;
            Ok((HirIdent::non_trait_method(id, location), scope))
        } else {
            Err(ResolverError::VariableNotDeclared {
                name: name.0.contents.clone(),
                span: name.0.span(),
            })
        }
    }

    /// Resolve generics using the expected kinds of the function we are calling
    pub(super) fn resolve_turbofish_generics(
        &mut self,
        func_id: &FuncId,
        unresolved_turbofish: Option<Vec<UnresolvedType>>,
        span: Span,
    ) -> Option<Vec<Type>> {
        let direct_generics = self.interner.function_meta(func_id).direct_generics.clone();

        unresolved_turbofish.map(|option_inner| {
            if option_inner.len() != direct_generics.len() {
                let type_check_err = TypeCheckError::IncorrectTurbofishGenericCount {
                    expected_count: direct_generics.len(),
                    actual_count: option_inner.len(),
                    span,
                };
                self.push_err(type_check_err);
            }

            let generics_with_types = direct_generics.iter().zip(option_inner);
            vecmap(generics_with_types, |(generic, unresolved_type)| {
                self.resolve_type_inner(unresolved_type, &generic.kind)
            })
        })
    }

    pub(super) fn elaborate_variable(
        &mut self,
        variable: Path,
        unresolved_turbofish: Option<Vec<UnresolvedType>>,
    ) -> (ExprId, Type) {
        let span = variable.span;
        let expr = self.resolve_variable(variable);
        let definition_id = expr.id;

        let definition_kind =
            self.interner.try_definition(definition_id).map(|definition| definition.kind.clone());

        // Resolve any generics if we the variable we have resolved is a function
        // and if the turbofish operator was used.
        let generics = definition_kind.and_then(|definition_kind| match &definition_kind {
            DefinitionKind::Function(function) => {
                self.resolve_turbofish_generics(function, unresolved_turbofish, span)
            }
            _ => None,
        });

        let id = self.interner.push_expr(HirExpression::Ident(expr.clone(), generics.clone()));

        self.interner.push_expr_location(id, span, self.file);
        let typ = self.type_check_variable(expr, id, generics);
        self.interner.push_expr_type(id, typ.clone());

        // Comptime variables must be replaced with their values
        if let Some(definition) = self.interner.try_definition(definition_id) {
            if definition.comptime && !self.in_comptime_context() {
                let mut interpreter_errors = vec![];
                let mut interpreter = Interpreter::new(
                    self.interner,
                    &mut self.comptime_scopes,
                    self.crate_id,
                    self.debug_comptime_in_file,
                    &mut interpreter_errors,
                );
                let value = interpreter.evaluate(id);
                self.include_interpreter_errors(interpreter_errors);
                return self.inline_comptime_value(value, span);
            }
        }

        (id, typ)
    }

    fn resolve_variable(&mut self, path: Path) -> HirIdent {
        if let Some((method, constraint, assumed)) = self.resolve_trait_generic_path(&path) {
            HirIdent {
                location: Location::new(path.span, self.file),
                id: self.interner.trait_method_id(method),
                impl_kind: ImplKind::TraitMethod(method, constraint, assumed),
            }
        } else {
            // If the Path is being used as an Expression, then it is referring to a global from a separate module
            // Otherwise, then it is referring to an Identifier
            // This lookup allows support of such statements: let x = foo::bar::SOME_GLOBAL + 10;
            // If the expression is a singular indent, we search the resolver's current scope as normal.
            let span = path.span();
            let (hir_ident, var_scope_index) = self.get_ident_from_path(path);

            if hir_ident.id != DefinitionId::dummy_id() {
                match self.interner.definition(hir_ident.id).kind {
                    DefinitionKind::Function(func_id) => {
                        if let Some(current_item) = self.current_item {
                            self.interner.add_function_dependency(current_item, func_id);
                        }

                        self.interner.add_function_reference(func_id, hir_ident.location);
                    }
                    DefinitionKind::Global(global_id) => {
                        if let Some(global) = self.unresolved_globals.remove(&global_id) {
                            self.elaborate_global(global);
                        }
                        if let Some(current_item) = self.current_item {
                            self.interner.add_global_dependency(current_item, global_id);
                        }

                        self.interner.add_global_reference(global_id, hir_ident.location);
                    }
                    DefinitionKind::GenericType(_) => {
                        // Initialize numeric generics to a polymorphic integer type in case
                        // they're used in expressions. We must do this here since type_check_variable
                        // does not check definition kinds and otherwise expects parameters to
                        // already be typed.
                        if self.interner.definition_type(hir_ident.id) == Type::Error {
                            let typ = Type::polymorphic_integer_or_field(self.interner);
                            self.interner.push_definition_type(hir_ident.id, typ);
                        }
                    }
                    DefinitionKind::Local(_) => {
                        // only local variables can be captured by closures.
                        self.resolve_local_variable(hir_ident.clone(), var_scope_index);

                        let reference_location = Location::new(span, self.file);
                        self.interner.add_local_reference(hir_ident.id, reference_location);
                    }
                }
            }

            hir_ident
        }
    }

    pub(super) fn type_check_variable(
        &mut self,
        ident: HirIdent,
        expr_id: ExprId,
        generics: Option<Vec<Type>>,
    ) -> Type {
        let mut bindings = TypeBindings::new();

        // Add type bindings from any constraints that were used.
        // We need to do this first since otherwise instantiating the type below
        // will replace each trait generic with a fresh type variable, rather than
        // the type used in the trait constraint (if it exists). See #4088.
        if let ImplKind::TraitMethod(_, constraint, assumed) = &ident.impl_kind {
            let the_trait = self.interner.get_trait(constraint.trait_id);
            assert_eq!(the_trait.generics.len(), constraint.trait_generics.len());

            for (param, arg) in the_trait.generics.iter().zip(&constraint.trait_generics) {
                // Avoid binding t = t
                if !arg.occurs(param.type_var.id()) {
                    bindings.insert(param.type_var.id(), (param.type_var.clone(), arg.clone()));
                }
            }

            // If the trait impl is already assumed to exist we should add any type bindings for `Self`.
            // Otherwise `self` will be replaced with a fresh type variable, which will require the user
            // to specify a redundant type annotation.
            if *assumed {
                bindings.insert(
                    the_trait.self_type_typevar_id,
                    (the_trait.self_type_typevar.clone(), constraint.typ.clone()),
                );
            }
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

        let span = self.interner.expr_span(&expr_id);
        let location = self.interner.expr_location(&expr_id);
        // This instantiates a trait's generics as well which need to be set
        // when the constraint below is later solved for when the function is
        // finished. How to link the two?
        let (typ, bindings) =
            self.instantiate(t, bindings, generics, function_generic_count, span, location);

        // Push any trait constraints required by this definition to the context
        // to be checked later when the type of this variable is further constrained.
        if let Some(definition) = self.interner.try_definition(ident.id) {
            if let DefinitionKind::Function(function) = definition.kind {
                let function = self.interner.function_meta(&function);

                for mut constraint in function.trait_constraints.clone() {
                    constraint.apply_bindings(&bindings);
                    self.push_trait_constraint(constraint, expr_id);
                }
            }
        }

        if let ImplKind::TraitMethod(_, mut constraint, assumed) = ident.impl_kind {
            constraint.apply_bindings(&bindings);
            if assumed {
                let trait_impl = TraitImplKind::Assumed {
                    object_type: constraint.typ,
                    trait_generics: constraint.trait_generics,
                };
                self.interner.select_impl_for_expression(expr_id, trait_impl);
            } else {
                // Currently only one impl can be selected per expr_id, so this
                // constraint needs to be pushed after any other constraints so
                // that monomorphization can resolve this trait method to the correct impl.
                self.push_trait_constraint(constraint, expr_id);
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
        span: Span,
        location: Location,
    ) -> (Type, TypeBindings) {
        match turbofish_generics {
            Some(turbofish_generics) => {
                if turbofish_generics.len() != function_generic_count {
                    let type_check_err = TypeCheckError::IncorrectTurbofishGenericCount {
                        expected_count: function_generic_count,
                        actual_count: turbofish_generics.len(),
                        span,
                    };
                    self.errors.push((CompilationError::TypeError(type_check_err), location.file));
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

    pub fn get_ident_from_path(&mut self, path: Path) -> (HirIdent, usize) {
        let location = Location::new(path.last_segment().span(), self.file);

        let error = match path.as_ident().map(|ident| self.use_variable(ident)) {
            Some(Ok(found)) => return found,
            // Try to look it up as a global, but still issue the first error if we fail
            Some(Err(error)) => match self.lookup_global(path) {
                Ok(id) => return (HirIdent::non_trait_method(id, location), 0),
                Err(_) => error,
            },
            None => match self.lookup_global(path) {
                Ok(id) => return (HirIdent::non_trait_method(id, location), 0),
                Err(error) => error,
            },
        };
        self.push_err(error);
        let id = DefinitionId::dummy_id();
        (HirIdent::non_trait_method(id, location), 0)
    }
}
