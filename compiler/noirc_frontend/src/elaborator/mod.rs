use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{
    ast::{FunctionKind, GenericTypeArgs, UnresolvedTraitConstraint},
    hir::{
        def_collector::dc_crate::{
            filter_literal_globals, CompilationError, ImplMap, UnresolvedGlobal, UnresolvedStruct,
            UnresolvedTypeAlias,
        },
        def_map::DefMaps,
        resolution::{errors::ResolverError, path_resolver::PathResolver},
        scope::ScopeForest as GenericScopeForest,
        type_check::{generics::TraitGenerics, TypeCheckError},
    },
    hir_def::{
        expr::{HirCapturedVar, HirIdent},
        function::{FunctionBody, Parameters},
        traits::TraitConstraint,
        types::{Generics, Kind, ResolvedGeneric},
    },
    macros_api::{
        BlockExpression, Ident, NodeInterner, NoirFunction, NoirStruct, Pattern,
        SecondaryAttribute, StructId,
    },
    node_interner::{
        DefinitionKind, DependencyId, ExprId, FuncId, GlobalId, ReferenceId, TraitId, TypeAliasId,
    },
    Shared, Type, TypeVariable,
};
use crate::{
    ast::{TraitBound, UnresolvedGeneric, UnresolvedGenerics},
    graph::CrateId,
    hir::{
        def_collector::{dc_crate::CollectedItems, errors::DefCollectorErrorKind},
        def_map::{LocalModuleId, ModuleDefId, ModuleId, MAIN_FUNCTION},
        resolution::{import::PathResolution, path_resolver::StandardPathResolver},
        Context,
    },
    hir_def::function::{FuncMeta, HirFunction},
    macros_api::{Param, Path, UnresolvedTypeData},
    node_interner::TraitImplId,
};
use crate::{
    hir::{
        def_collector::dc_crate::{UnresolvedFunctions, UnresolvedTraitImpl},
        def_map::ModuleData,
    },
    hir_def::traits::TraitImpl,
    macros_api::ItemVisibility,
};

mod comptime;
mod expressions;
mod lints;
mod patterns;
mod scope;
mod statements;
mod trait_impls;
mod traits;
pub mod types;
mod unquote;

use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Location, Span};

use self::traits::check_trait_impl_method_matches_declaration;

/// ResolverMetas are tagged onto each definition to track how many times they are used
#[derive(Debug, PartialEq, Eq)]
pub struct ResolverMeta {
    num_times_used: usize,
    ident: HirIdent,
    warn_if_unused: bool,
}

type ScopeForest = GenericScopeForest<String, ResolverMeta>;

pub struct LambdaContext {
    pub captures: Vec<HirCapturedVar>,
    /// the index in the scope tree
    /// (sometimes being filled by ScopeTree's find method)
    pub scope_index: usize,
}

pub struct Elaborator<'context> {
    scopes: ScopeForest,

    pub(crate) errors: Vec<(CompilationError, FileId)>,

    pub(crate) interner: &'context mut NodeInterner,

    def_maps: &'context mut DefMaps,

    file: FileId,

    in_unsafe_block: bool,
    nested_loops: usize,

    /// Contains a mapping of the current struct or functions's generics to
    /// unique type variables if we're resolving a struct. Empty otherwise.
    /// This is a Vec rather than a map to preserve the order a functions generics
    /// were declared in.
    generics: Vec<ResolvedGeneric>,

    /// When resolving lambda expressions, we need to keep track of the variables
    /// that are captured. We do this in order to create the hidden environment
    /// parameter for the lambda function.
    lambda_stack: Vec<LambdaContext>,

    /// Set to the current type if we're resolving an impl
    self_type: Option<Type>,

    /// The current dependency item we're resolving.
    /// Used to link items to their dependencies in the dependency graph
    current_item: Option<DependencyId>,

    /// If we're currently resolving methods within a trait impl, this will be set
    /// to the corresponding trait impl ID.
    current_trait_impl: Option<TraitImplId>,

    /// The trait  we're currently resolving, if we are resolving one.
    current_trait: Option<TraitId>,

    /// In-resolution names
    ///
    /// This needs to be a set because we can have multiple in-resolution
    /// names when resolving structs that are declared in reverse order of their
    /// dependencies, such as in the following case:
    ///
    /// ```
    /// struct Wrapper {
    ///     value: Wrapped
    /// }
    /// struct Wrapped {
    /// }
    /// ```
    resolving_ids: BTreeSet<StructId>,

    /// Each constraint in the `where` clause of the function currently being resolved.
    trait_bounds: Vec<TraitConstraint>,

    /// This is a stack of function contexts. Most of the time, for each function we
    /// expect this to be of length one, containing each type variable and trait constraint
    /// used in the function. This is also pushed to when a `comptime {}` block is used within
    /// the function. Since it can force us to resolve that block's trait constraints earlier
    /// so that they are resolved when the interpreter is run before the enclosing function
    /// is finished elaborating. When this happens, we need to resolve any type variables
    /// that were made within this block as well so that we can solve these traits.
    function_context: Vec<FunctionContext>,

    /// The current module this elaborator is in.
    /// Initially empty, it is set whenever a new top-level item is resolved.
    local_module: LocalModuleId,

    crate_id: CrateId,

    /// The scope of --debug-comptime, or None if unset
    debug_comptime_in_file: Option<FileId>,

    /// These are the globals that have yet to be elaborated.
    /// This map is used to lazily evaluate these globals if they're encountered before
    /// they are elaborated (e.g. in a function's type or another global's RHS).
    unresolved_globals: BTreeMap<GlobalId, UnresolvedGlobal>,

    /// Temporary flag to enable the experimental arithmetic generics feature
    enable_arithmetic_generics: bool,
}

#[derive(Default)]
struct FunctionContext {
    /// All type variables created in the current function.
    /// This map is used to default any integer type variables at the end of
    /// a function (before checking trait constraints) if a type wasn't already chosen.
    type_variables: Vec<Type>,

    /// Trait constraints are collected during type checking until they are
    /// verified at the end of a function. This is because constraints arise
    /// on each variable, but it is only until function calls when the types
    /// needed for the trait constraint may become known.
    trait_constraints: Vec<(TraitConstraint, ExprId)>,
}

impl<'context> Elaborator<'context> {
    pub fn new(
        interner: &'context mut NodeInterner,
        def_maps: &'context mut DefMaps,
        crate_id: CrateId,
        debug_comptime_in_file: Option<FileId>,
        enable_arithmetic_generics: bool,
    ) -> Self {
        Self {
            scopes: ScopeForest::default(),
            errors: Vec::new(),
            interner,
            def_maps,
            file: FileId::dummy(),
            in_unsafe_block: false,
            nested_loops: 0,
            generics: Vec::new(),
            lambda_stack: Vec::new(),
            self_type: None,
            current_item: None,
            local_module: LocalModuleId::dummy_id(),
            crate_id,
            resolving_ids: BTreeSet::new(),
            trait_bounds: Vec::new(),
            function_context: vec![FunctionContext::default()],
            current_trait_impl: None,
            debug_comptime_in_file,
            unresolved_globals: BTreeMap::new(),
            enable_arithmetic_generics,
            current_trait: None,
        }
    }

    pub fn from_context(
        context: &'context mut Context,
        crate_id: CrateId,
        debug_comptime_in_file: Option<FileId>,
        enable_arithmetic_generics: bool,
    ) -> Self {
        Self::new(
            &mut context.def_interner,
            &mut context.def_maps,
            crate_id,
            debug_comptime_in_file,
            enable_arithmetic_generics,
        )
    }

    pub fn elaborate(
        context: &'context mut Context,
        crate_id: CrateId,
        items: CollectedItems,
        debug_comptime_in_file: Option<FileId>,
        enable_arithmetic_generics: bool,
    ) -> Vec<(CompilationError, FileId)> {
        Self::elaborate_and_return_self(
            context,
            crate_id,
            items,
            debug_comptime_in_file,
            enable_arithmetic_generics,
        )
        .errors
    }

    pub fn elaborate_and_return_self(
        context: &'context mut Context,
        crate_id: CrateId,
        items: CollectedItems,
        debug_comptime_in_file: Option<FileId>,
        enable_arithmetic_generics: bool,
    ) -> Self {
        let mut this = Self::from_context(
            context,
            crate_id,
            debug_comptime_in_file,
            enable_arithmetic_generics,
        );
        this.elaborate_items(items);
        this.check_and_pop_function_context();
        this
    }

    fn elaborate_items(&mut self, mut items: CollectedItems) {
        // We must first resolve and intern the globals before we can resolve any stmts inside each function.
        // Each function uses its own resolver with a newly created ScopeForest, and must be resolved again to be within a function's scope
        //
        // Additionally, we must resolve integer globals before structs since structs may refer to
        // the values of integer globals as numeric generics.
        let (literal_globals, non_literal_globals) = filter_literal_globals(items.globals);
        for global in non_literal_globals {
            self.unresolved_globals.insert(global.global_id, global);
        }

        for global in literal_globals {
            self.elaborate_global(global);
        }

        for (alias_id, alias) in items.type_aliases {
            self.define_type_alias(alias_id, alias);
        }

        // Must resolve structs before we resolve globals.
        self.collect_struct_definitions(&items.types);

        self.define_function_metas(&mut items.functions, &mut items.impls, &mut items.trait_impls);

        self.collect_traits(&items.traits);

        // Before we resolve any function symbols we must go through our impls and
        // re-collect the methods within into their proper module. This cannot be
        // done during def collection since we need to be able to resolve the type of
        // the impl since that determines the module we should collect into.
        for ((_self_type, module), impls) in &mut items.impls {
            self.collect_impls(*module, impls);
        }

        // Bind trait impls to their trait. Collect trait functions, that have a
        // default implementation, which hasn't been overridden.
        for trait_impl in &mut items.trait_impls {
            self.collect_trait_impl(trait_impl);
        }

        // We must wait to resolve non-literal globals until after we resolve structs since struct
        // globals will need to reference the struct type they're initialized to ensure they are valid.
        while let Some((_, global)) = self.unresolved_globals.pop_first() {
            self.elaborate_global(global);
        }

        // We have to run any comptime attributes on functions before the function is elaborated
        // since the generated items are checked beforehand as well.
        let generated_items = self.run_attributes(&items.traits, &items.types, &items.functions);

        // After everything is collected, we can elaborate our generated items.
        // It may be better to inline these within `items` entirely since elaborating them
        // all here means any globals will not see these. Inlining them completely within `items`
        // means we must be more careful about missing any additional items that need to be already
        // elaborated. E.g. if a new struct is created, we've already passed the code path to
        // elaborate them.
        if !generated_items.is_empty() {
            self.elaborate_items(generated_items);
        }

        for functions in items.functions {
            self.elaborate_functions(functions);
        }

        for impls in items.impls.into_values() {
            self.elaborate_impls(impls);
        }

        for trait_impl in items.trait_impls {
            self.elaborate_trait_impl(trait_impl);
        }

        self.errors.extend(self.interner.check_for_dependency_cycles());
    }

    /// Runs `f` and if it modifies `self.generics`, `self.generics` is truncated
    /// back to the previous length.
    fn recover_generics<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let generics_count = self.generics.len();
        let ret = f(self);
        self.generics.truncate(generics_count);
        ret
    }

    fn elaborate_functions(&mut self, functions: UnresolvedFunctions) {
        for (_, id, _) in functions.functions {
            self.elaborate_function(id);
        }

        self.generics.clear();
        self.self_type = None;
    }

    fn introduce_generics_into_scope(&mut self, all_generics: Vec<ResolvedGeneric>) {
        // Introduce all numeric generics into scope
        for generic in &all_generics {
            if let Kind::Numeric(typ) = &generic.kind {
                let definition = DefinitionKind::GenericType(generic.type_var.clone());
                let ident = Ident::new(generic.name.to_string(), generic.span);
                let hir_ident =
                    self.add_variable_decl_inner(ident, false, false, false, definition);
                self.interner.push_definition_type(hir_ident.id, *typ.clone());
            }
        }

        self.generics = all_generics;
    }

    pub(crate) fn elaborate_function(&mut self, id: FuncId) {
        let func_meta = self.interner.func_meta.get_mut(&id);
        let func_meta =
            func_meta.expect("FuncMetas should be declared before a function is elaborated");

        let (kind, body, body_span) = match func_meta.take_body() {
            FunctionBody::Unresolved(kind, body, span) => (kind, body, span),
            FunctionBody::Resolved => return,
            // Do not error for the still-resolving case. If there is a dependency cycle,
            // the dependency cycle check will find it later on.
            FunctionBody::Resolving => return,
        };

        let func_meta = func_meta.clone();

        assert_eq!(
            self.crate_id, func_meta.source_crate,
            "Functions in other crates should be already elaborated"
        );

        self.local_module = func_meta.source_module;
        self.file = func_meta.source_file;
        self.self_type = func_meta.self_type.clone();
        self.current_trait_impl = func_meta.trait_impl;

        self.scopes.start_function();
        let old_item = std::mem::replace(&mut self.current_item, Some(DependencyId::Function(id)));

        self.trait_bounds = func_meta.trait_constraints.clone();
        self.function_context.push(FunctionContext::default());

        self.introduce_generics_into_scope(func_meta.all_generics.clone());

        // The DefinitionIds for each parameter were already created in define_function_meta
        // so we need to reintroduce the same IDs into scope here.
        for parameter in &func_meta.parameter_idents {
            let name = self.interner.definition_name(parameter.id).to_owned();
            self.add_existing_variable_to_scope(name, parameter.clone(), true);
        }

        self.declare_numeric_generics(&func_meta.parameters, func_meta.return_type());
        self.add_trait_constraints_to_scope(&func_meta);

        let (hir_func, body_type) = match kind {
            FunctionKind::Builtin | FunctionKind::LowLevel | FunctionKind::Oracle => {
                (HirFunction::empty(), Type::Error)
            }
            FunctionKind::Normal | FunctionKind::Recursive => {
                let (block, body_type) = self.elaborate_block(body);
                let expr_id = self.intern_expr(block, body_span);
                self.interner.push_expr_type(expr_id, body_type.clone());
                (HirFunction::unchecked_from_expr(expr_id), body_type)
            }
        };

        // Don't verify the return type for builtin functions & trait function declarations
        if !func_meta.is_stub() {
            self.type_check_function_body(body_type, &func_meta, hir_func.as_expr());
        }

        // Default any type variables that still need defaulting and
        // verify any remaining trait constraints arising from the function body.
        // This is done before trait impl search since leaving them bindable can lead to errors
        // when multiple impls are available. Instead we default first to choose the Field or u64 impl.
        self.check_and_pop_function_context();

        // Now remove all the `where` clause constraints we added
        for constraint in &func_meta.trait_constraints {
            self.interner.remove_assumed_trait_implementations_for_trait(constraint.trait_id);
        }

        let func_scope_tree = self.scopes.end_function();

        // The arguments to low-level and oracle functions are always unused so we do not produce warnings for them.
        if !func_meta.is_stub() {
            self.check_for_unused_variables_in_scope_tree(func_scope_tree);
        }

        let meta = self
            .interner
            .func_meta
            .get_mut(&id)
            .expect("FuncMetas should be declared before a function is elaborated");

        meta.function_body = FunctionBody::Resolved;

        self.trait_bounds.clear();
        self.interner.update_fn(id, hir_func);
        self.current_item = old_item;
    }

    /// Defaults all type variables used in this function context then solves
    /// all still-unsolved trait constraints in this context.
    fn check_and_pop_function_context(&mut self) {
        let context = self.function_context.pop().expect("Imbalanced function_context pushes");

        for typ in context.type_variables {
            if let Type::TypeVariable(variable, kind) = typ.follow_bindings() {
                let msg = "TypeChecker should only track defaultable type vars";
                variable.bind(kind.default_type().expect(msg));
            }
        }

        for (mut constraint, expr_id) in context.trait_constraints {
            let span = self.interner.expr_span(&expr_id);

            if matches!(&constraint.typ, Type::MutableReference(_)) {
                let (_, dereferenced_typ) =
                    self.insert_auto_dereferences(expr_id, constraint.typ.clone());
                constraint.typ = dereferenced_typ;
            }

            self.verify_trait_constraint(
                &constraint.typ,
                constraint.trait_id,
                &constraint.trait_generics.ordered,
                &constraint.trait_generics.named,
                expr_id,
                span,
            );
        }
    }

    /// This turns function parameters of the form:
    /// `fn foo(x: impl Bar)`
    ///
    /// into
    /// `fn foo<T0_impl_Bar>(x: T0_impl_Bar) where T0_impl_Bar: Bar`
    /// although the fresh type variable is not named internally.
    fn desugar_impl_trait_arg(
        &mut self,
        trait_path: Path,
        trait_generics: GenericTypeArgs,
        generics: &mut Vec<TypeVariable>,
        trait_constraints: &mut Vec<TraitConstraint>,
    ) -> Type {
        let new_generic_id = self.interner.next_type_variable_id();
        let new_generic = TypeVariable::unbound(new_generic_id);
        generics.push(new_generic.clone());

        let name = format!("impl {trait_path}");
        let generic_type = Type::NamedGeneric(new_generic, Rc::new(name), Kind::Normal);
        let trait_bound = TraitBound { trait_path, trait_id: None, trait_generics };

        if let Some(new_constraint) = self.resolve_trait_bound(&trait_bound, generic_type.clone()) {
            trait_constraints.push(new_constraint);
        }

        generic_type
    }

    /// Add the given generics to scope.
    /// Each generic will have a fresh Shared<TypeBinding> associated with it.
    pub fn add_generics(&mut self, generics: &UnresolvedGenerics) -> Generics {
        vecmap(generics, |generic| {
            let mut is_error = false;
            let (type_var, name, kind) = match self.resolve_generic(generic) {
                Ok(values) => values,
                Err(error) => {
                    self.push_err(error);
                    is_error = true;
                    let id = self.interner.next_type_variable_id();
                    (TypeVariable::unbound(id), Rc::new("(error)".into()), Kind::Normal)
                }
            };

            let span = generic.span();
            let name_owned = name.as_ref().clone();
            let resolved_generic = ResolvedGeneric { name, type_var, kind, span };

            // Check for name collisions of this generic
            // Checking `is_error` here prevents DuplicateDefinition errors when
            // we have multiple generics from macros which fail to resolve and
            // are all given the same default name "(error)".
            if !is_error {
                if let Some(generic) = self.find_generic(&name_owned) {
                    self.push_err(ResolverError::DuplicateDefinition {
                        name: name_owned,
                        first_span: generic.span,
                        second_span: span,
                    });
                } else {
                    self.generics.push(resolved_generic.clone());
                }
            }

            resolved_generic
        })
    }

    fn resolve_generic(
        &mut self,
        generic: &UnresolvedGeneric,
    ) -> Result<(TypeVariable, Rc<String>, Kind), ResolverError> {
        // Map the generic to a fresh type variable
        match generic {
            UnresolvedGeneric::Variable(_) | UnresolvedGeneric::Numeric { .. } => {
                let id = self.interner.next_type_variable_id();
                let typevar = TypeVariable::unbound(id);
                let ident = generic.ident();

                let kind = self.resolve_generic_kind(generic);
                let name = Rc::new(ident.0.contents.clone());
                Ok((typevar, name, kind))
            }
            // An already-resolved generic is only possible if it is the result of a
            // previous macro call being inserted into a generics list.
            UnresolvedGeneric::Resolved(id, span) => {
                match self.interner.get_quoted_type(*id).follow_bindings() {
                    Type::NamedGeneric(type_variable, name, kind) => {
                        Ok((type_variable, name, kind))
                    }
                    other => Err(ResolverError::MacroResultInGenericsListNotAGeneric {
                        span: *span,
                        typ: other.clone(),
                    }),
                }
            }
        }
    }

    /// Return the kind of an unresolved generic.
    /// If a numeric generic has been specified, resolve the annotated type to make
    /// sure only primitive numeric types are being used.
    pub(super) fn resolve_generic_kind(&mut self, generic: &UnresolvedGeneric) -> Kind {
        if let UnresolvedGeneric::Numeric { ident, typ } = generic {
            let typ = typ.clone();
            let typ = if typ.is_type_expression() {
                self.resolve_type_inner(typ, &Kind::Numeric(Box::new(Type::default_int_type())))
            } else {
                self.resolve_type(typ.clone())
            };
            if !matches!(typ, Type::FieldElement | Type::Integer(_, _)) {
                let unsupported_typ_err = ResolverError::UnsupportedNumericGenericType {
                    ident: ident.clone(),
                    typ: typ.clone(),
                };
                self.push_err(unsupported_typ_err);
            }
            Kind::Numeric(Box::new(typ))
        } else {
            Kind::Normal
        }
    }

    fn push_err(&mut self, error: impl Into<CompilationError>) {
        self.errors.push((error.into(), self.file));
    }

    fn run_lint(&mut self, lint: impl Fn(&Elaborator) -> Option<CompilationError>) {
        if let Some(error) = lint(self) {
            self.push_err(error);
        }
    }

    pub fn resolve_module_by_path(&self, path: Path) -> Option<ModuleId> {
        let path_resolver = StandardPathResolver::new(self.module_id());

        match path_resolver.resolve(self.def_maps, path.clone(), &mut None) {
            Ok(PathResolution { module_def_id: ModuleDefId::ModuleId(module_id), error }) => {
                if error.is_some() {
                    None
                } else {
                    Some(module_id)
                }
            }
            _ => None,
        }
    }

    fn resolve_trait_by_path(&mut self, path: Path) -> Option<TraitId> {
        let path_resolver = StandardPathResolver::new(self.module_id());

        let error = match path_resolver.resolve(self.def_maps, path.clone(), &mut None) {
            Ok(PathResolution { module_def_id: ModuleDefId::TraitId(trait_id), error }) => {
                if let Some(error) = error {
                    self.push_err(error);
                }
                return Some(trait_id);
            }
            Ok(_) => DefCollectorErrorKind::NotATrait { not_a_trait_name: path },
            Err(_) => DefCollectorErrorKind::TraitNotFound { trait_path: path },
        };
        self.push_err(error);
        None
    }

    /// TODO: This is currently only respected for generic free functions
    /// there's a bunch of other places where trait constraints can pop up
    fn resolve_trait_constraints(
        &mut self,
        where_clause: &[UnresolvedTraitConstraint],
    ) -> Vec<TraitConstraint> {
        where_clause
            .iter()
            .filter_map(|constraint| self.resolve_trait_constraint(constraint))
            .collect()
    }

    pub fn resolve_trait_constraint(
        &mut self,
        constraint: &UnresolvedTraitConstraint,
    ) -> Option<TraitConstraint> {
        let typ = self.resolve_type(constraint.typ.clone());
        self.resolve_trait_bound(&constraint.trait_bound, typ)
    }

    pub fn resolve_trait_bound(
        &mut self,
        bound: &TraitBound,
        typ: Type,
    ) -> Option<TraitConstraint> {
        let the_trait = self.lookup_trait_or_error(bound.trait_path.clone())?;
        let trait_id = the_trait.id;
        let span = bound.trait_path.span;

        let (ordered, named) = self.resolve_type_args(bound.trait_generics.clone(), trait_id, span);

        let trait_generics = TraitGenerics { ordered, named };
        Some(TraitConstraint { typ, trait_id, trait_generics, span })
    }

    /// Extract metadata from a NoirFunction
    /// to be used in analysis and intern the function parameters
    /// Prerequisite: any implicit generics, including any generics from the impl,
    /// have already been added to scope via `self.add_generics`.
    fn define_function_meta(
        &mut self,
        func: &mut NoirFunction,
        func_id: FuncId,
        trait_id: Option<TraitId>,
    ) {
        let in_contract = if self.self_type.is_some() {
            // Without this, impl methods can accidentally be placed in contracts.
            // See: https://github.com/noir-lang/noir/issues/3254
            false
        } else {
            self.in_contract()
        };

        self.scopes.start_function();
        self.current_item = Some(DependencyId::Function(func_id));

        let location = Location::new(func.name_ident().span(), self.file);
        let id = self.interner.function_definition_id(func_id);
        let name_ident = HirIdent::non_trait_method(id, location);

        let is_entry_point = self.is_entry_point_function(func, in_contract);

        self.run_lint(|_| lints::inlining_attributes(func).map(Into::into));
        self.run_lint(|_| lints::missing_pub(func, is_entry_point).map(Into::into));
        self.run_lint(|elaborator| {
            lints::unnecessary_pub_return(func, elaborator.pub_allowed(func, in_contract))
                .map(Into::into)
        });
        self.run_lint(|_| lints::oracle_not_marked_unconstrained(func).map(Into::into));
        self.run_lint(|elaborator| {
            lints::low_level_function_outside_stdlib(func, elaborator.crate_id).map(Into::into)
        });
        self.run_lint(|_| {
            lints::recursive_non_entrypoint_function(func, is_entry_point).map(Into::into)
        });

        // Both the #[fold] and #[no_predicates] alter a function's inline type and code generation in similar ways.
        // In certain cases such as type checking (for which the following flag will be used) both attributes
        // indicate we should code generate in the same way. Thus, we unify the attributes into one flag here.
        let has_no_predicates_attribute = func.attributes().is_no_predicates();
        let should_fold = func.attributes().is_foldable();
        let has_inline_attribute = has_no_predicates_attribute || should_fold;
        let is_pub_allowed = self.pub_allowed(func, in_contract);
        self.add_generics(&func.def.generics);

        let mut trait_constraints = self.resolve_trait_constraints(&func.def.where_clause);

        let mut generics = vecmap(&self.generics, |generic| generic.type_var.clone());
        let mut parameters = Vec::new();
        let mut parameter_types = Vec::new();
        let mut parameter_idents = Vec::new();

        for Param { visibility, pattern, typ, span: _ } in func.parameters().iter().cloned() {
            self.run_lint(|_| {
                lints::unnecessary_pub_argument(func, visibility, is_pub_allowed).map(Into::into)
            });

            let type_span = typ.span;
            let typ = match typ.typ {
                UnresolvedTypeData::TraitAsType(path, args) => {
                    self.desugar_impl_trait_arg(path, args, &mut generics, &mut trait_constraints)
                }
                _ => self.resolve_type_inner(typ, &Kind::Normal),
            };

            self.check_if_type_is_valid_for_program_input(
                &typ,
                is_entry_point,
                has_inline_attribute,
                type_span,
            );

            let pattern = self.elaborate_pattern_and_store_ids(
                pattern,
                typ.clone(),
                DefinitionKind::Local(None),
                &mut parameter_idents,
                None,
            );

            parameters.push((pattern, typ.clone(), visibility));
            parameter_types.push(typ);
        }

        let return_type = Box::new(self.resolve_type(func.return_type()));

        let mut typ = Type::Function(
            parameter_types,
            return_type,
            Box::new(Type::Unit),
            func.def.is_unconstrained,
        );

        if !generics.is_empty() {
            typ = Type::Forall(generics, Box::new(typ));
        }

        self.interner.push_definition_type(name_ident.id, typ.clone());

        let direct_generics = func.def.generics.iter();
        let direct_generics = direct_generics
            .filter_map(|generic| self.find_generic(&generic.ident().0.contents).cloned())
            .collect();

        let statements = std::mem::take(&mut func.def.body.statements);
        let body = BlockExpression { statements };

        let struct_id = if let Some(Type::Struct(struct_type, _)) = &self.self_type {
            Some(struct_type.borrow().id)
        } else {
            None
        };

        let meta = FuncMeta {
            name: name_ident,
            kind: func.kind,
            location,
            typ,
            direct_generics,
            all_generics: self.generics.clone(),
            struct_id,
            trait_id,
            trait_impl: self.current_trait_impl,
            parameters: parameters.into(),
            parameter_idents,
            return_type: func.def.return_type.clone(),
            return_visibility: func.def.return_visibility,
            has_body: !func.def.body.is_empty(),
            trait_constraints,
            is_entry_point,
            has_inline_attribute,
            source_crate: self.crate_id,
            source_module: self.local_module,
            function_body: FunctionBody::Unresolved(func.kind, body, func.def.span),
            self_type: self.self_type.clone(),
            source_file: self.file,
        };

        self.interner.push_fn_meta(meta, func_id);
        self.scopes.end_function();
        self.current_item = None;
    }

    /// Only sized types are valid to be used as main's parameters or the parameters to a contract
    /// function. If the given type is not sized (e.g. contains a slice or NamedGeneric type), an
    /// error is issued.
    fn check_if_type_is_valid_for_program_input(
        &mut self,
        typ: &Type,
        is_entry_point: bool,
        has_inline_attribute: bool,
        span: Span,
    ) {
        if (is_entry_point && !typ.is_valid_for_program_input())
            || (has_inline_attribute && !typ.is_valid_non_inlined_function_input())
        {
            self.push_err(TypeCheckError::InvalidTypeForEntryPoint { span });
        }
    }

    /// True if the `pub` keyword is allowed on parameters in this function
    /// `pub` on function parameters is only allowed for entry point functions
    fn pub_allowed(&self, func: &NoirFunction, in_contract: bool) -> bool {
        self.is_entry_point_function(func, in_contract) || func.attributes().is_foldable()
    }

    /// Returns `true` if the current module is a contract.
    ///
    /// This is usually determined by `self.module_id()`, but it can
    /// be overridden for impls. Impls are an odd case since the methods within resolve
    /// as if they're in the parent module, but should be placed in a child module.
    /// Since they should be within a child module, they should be elaborated as if
    /// `in_contract` is `false` so we can still resolve them in the parent module without them being in a contract.
    fn in_contract(&self) -> bool {
        self.module_is_contract(self.module_id())
    }

    pub(crate) fn module_is_contract(&self, module_id: ModuleId) -> bool {
        module_id.module(self.def_maps).is_contract
    }

    fn is_entry_point_function(&self, func: &NoirFunction, in_contract: bool) -> bool {
        if in_contract {
            func.attributes().is_contract_entry_point()
        } else {
            func.name() == MAIN_FUNCTION
        }
    }

    // TODO(https://github.com/noir-lang/noir/issues/5156): Remove implicit numeric generics
    fn declare_numeric_generics(&mut self, params: &Parameters, return_type: &Type) {
        if self.generics.is_empty() {
            return;
        }

        for (name_to_find, type_variable) in Self::find_numeric_generics(params, return_type) {
            // Declare any generics to let users use numeric generics in scope.
            // Don't issue a warning if these are unused
            //
            // We can fail to find the generic in self.generics if it is an implicit one created
            // by the compiler. This can happen when, e.g. eliding array lengths using the slice
            // syntax [T].
            if let Some(ResolvedGeneric { name, span, kind, .. }) =
                self.generics.iter_mut().find(|generic| generic.name.as_ref() == &name_to_find)
            {
                let scope = self.scopes.get_mut_scope();
                let value = scope.find(&name_to_find);
                if value.is_some() {
                    // With the addition of explicit numeric generics we do not want to introduce numeric generics in this manner
                    // However, this is going to be a big breaking change so for now we simply issue a warning while users have time
                    // to transition to the new syntax
                    // e.g. this code would break with a duplicate definition error:
                    // ```
                    // fn foo<let N: u8>(arr: [Field; N]) { }
                    // ```
                    continue;
                }
                *kind = Kind::Numeric(Box::new(Type::default_int_type()));
                let ident = Ident::new(name.to_string(), *span);
                let definition = DefinitionKind::GenericType(type_variable);
                self.add_variable_decl_inner(ident.clone(), false, false, false, definition);

                self.push_err(ResolverError::UseExplicitNumericGeneric { ident });
            }
        }
    }

    fn add_trait_constraints_to_scope(&mut self, func_meta: &FuncMeta) {
        for constraint in &func_meta.trait_constraints {
            let object = constraint.typ.clone();
            let trait_id = constraint.trait_id;
            let generics = constraint.trait_generics.clone();

            if !self.interner.add_assumed_trait_implementation(object, trait_id, generics) {
                if let Some(the_trait) = self.interner.try_get_trait(trait_id) {
                    let trait_name = the_trait.name.to_string();
                    let typ = constraint.typ.clone();
                    let span = func_meta.location.span;
                    self.push_err(TypeCheckError::UnneededTraitConstraint {
                        trait_name,
                        typ,
                        span,
                    });
                }
            }
        }
    }

    fn elaborate_impls(&mut self, impls: Vec<(UnresolvedGenerics, Span, UnresolvedFunctions)>) {
        for (_, _, functions) in impls {
            self.file = functions.file_id;
            self.recover_generics(|this| this.elaborate_functions(functions));
        }
    }

    fn elaborate_trait_impl(&mut self, trait_impl: UnresolvedTraitImpl) {
        self.file = trait_impl.file_id;
        self.local_module = trait_impl.module_id;

        self.generics = trait_impl.resolved_generics;
        self.current_trait_impl = trait_impl.impl_id;

        for (module, function, _) in &trait_impl.methods.functions {
            self.local_module = *module;
            let errors = check_trait_impl_method_matches_declaration(self.interner, *function);
            self.errors.extend(errors.into_iter().map(|error| (error.into(), self.file)));
        }

        self.elaborate_functions(trait_impl.methods);

        self.self_type = None;
        self.current_trait_impl = None;
        self.generics.clear();
    }

    fn collect_impls(
        &mut self,
        module: LocalModuleId,
        impls: &mut [(UnresolvedGenerics, Span, UnresolvedFunctions)],
    ) {
        self.local_module = module;

        for (generics, span, unresolved) in impls {
            self.file = unresolved.file_id;
            let old_generic_count = self.generics.len();
            self.add_generics(generics);
            self.declare_methods_on_struct(false, unresolved, *span);
            self.generics.truncate(old_generic_count);
        }
    }

    fn collect_trait_impl(&mut self, trait_impl: &mut UnresolvedTraitImpl) {
        self.local_module = trait_impl.module_id;
        self.file = trait_impl.file_id;
        self.current_trait_impl = trait_impl.impl_id;

        let self_type = trait_impl.methods.self_type.clone();
        let self_type =
            self_type.expect("Expected struct type to be set before collect_trait_impl");

        self.self_type = Some(self_type.clone());
        let self_type_span = trait_impl.object_type.span;

        if matches!(self_type, Type::MutableReference(_)) {
            let span = self_type_span;
            self.push_err(DefCollectorErrorKind::MutableReferenceInTraitImpl { span });
        }

        if let Some(trait_id) = trait_impl.trait_id {
            self.generics = trait_impl.resolved_generics.clone();

            let where_clause = self.resolve_trait_constraints(&trait_impl.where_clause);

            self.collect_trait_impl_methods(trait_id, trait_impl, &where_clause);

            let span = trait_impl.object_type.span;
            self.declare_methods_on_struct(true, &mut trait_impl.methods, span);

            let methods = trait_impl.methods.function_ids();
            for func_id in &methods {
                self.interner.set_function_trait(*func_id, self_type.clone(), trait_id);
            }

            let trait_generics = trait_impl.resolved_trait_generics.clone();

            let resolved_trait_impl = Shared::new(TraitImpl {
                ident: trait_impl.trait_path.last_ident(),
                typ: self_type.clone(),
                trait_id,
                trait_generics,
                file: trait_impl.file_id,
                where_clause: where_clause.clone(),
                methods,
            });

            let generics = vecmap(&self.generics, |generic| generic.type_var.clone());

            if let Err((prev_span, prev_file)) = self.interner.add_trait_implementation(
                self_type.clone(),
                trait_id,
                trait_impl.impl_id.expect("impl_id should be set in define_function_metas"),
                generics,
                resolved_trait_impl,
            ) {
                self.push_err(DefCollectorErrorKind::OverlappingImpl {
                    typ: self_type.clone(),
                    span: self_type_span,
                });

                // The 'previous impl defined here' note must be a separate error currently
                // since it may be in a different file and all errors have the same file id.
                self.file = prev_file;
                self.push_err(DefCollectorErrorKind::OverlappingImplNote { span: prev_span });
                self.file = trait_impl.file_id;
            }
        }

        self.generics.clear();

        self.current_trait_impl = None;
        self.self_type = None;
    }

    pub fn get_module(&self, module: ModuleId) -> &ModuleData {
        let message = "A crate should always be present for a given crate id";
        &self.def_maps.get(&module.krate).expect(message).modules[module.local_id.0]
    }

    fn get_module_mut(def_maps: &mut DefMaps, module: ModuleId) -> &mut ModuleData {
        let message = "A crate should always be present for a given crate id";
        &mut def_maps.get_mut(&module.krate).expect(message).modules[module.local_id.0]
    }

    fn declare_methods_on_struct(
        &mut self,
        is_trait_impl: bool,
        functions: &mut UnresolvedFunctions,
        span: Span,
    ) {
        let self_type = functions.self_type.as_ref();
        let self_type =
            self_type.expect("Expected struct type to be set before declare_methods_on_struct");

        let function_ids = functions.function_ids();

        if let Type::Struct(struct_type, _) = &self_type {
            let struct_ref = struct_type.borrow();

            // `impl`s are only allowed on types defined within the current crate
            if !is_trait_impl && struct_ref.id.krate() != self.crate_id {
                let type_name = struct_ref.name.to_string();
                self.push_err(DefCollectorErrorKind::ForeignImpl { span, type_name });
                return;
            }

            // Grab the module defined by the struct type. Note that impls are a case
            // where the module the methods are added to is not the same as the module
            // they are resolved in.
            let module = Self::get_module_mut(self.def_maps, struct_ref.id.module_id());

            for (_, method_id, method) in &functions.functions {
                // If this method was already declared, remove it from the module so it cannot
                // be accessed with the `TypeName::method` syntax. We'll check later whether the
                // object types in each method overlap or not. If they do, we issue an error.
                // If not, that is specialization which is allowed.
                let name = method.name_ident().clone();
                if module.declare_function(name, ItemVisibility::Public, *method_id).is_err() {
                    let existing = module.find_func_with_name(method.name_ident()).expect(
                        "declare_function should only error if there is an existing function",
                    );

                    // Only remove the existing function from scope if it is from a trait impl as
                    // well. If it is from a non-trait impl that should override trait impl methods
                    // anyway so that Foo::bar always resolves to the non-trait impl version.
                    if self.interner.function_meta(&existing).trait_impl.is_some() {
                        module.remove_function(method.name_ident());
                    }
                }
            }

            // Trait impl methods are already declared in NodeInterner::add_trait_implementation
            if !is_trait_impl {
                self.declare_methods(self_type, &function_ids);
            }
        // We can define methods on primitive types only if we're in the stdlib
        } else if !is_trait_impl && *self_type != Type::Error {
            if self.crate_id.is_stdlib() {
                // Trait impl methods are already declared in NodeInterner::add_trait_implementation
                if !is_trait_impl {
                    self.declare_methods(self_type, &function_ids);
                }
            } else {
                self.push_err(DefCollectorErrorKind::NonStructTypeInImpl { span });
            }
        }
    }

    fn declare_methods(&mut self, self_type: &Type, function_ids: &[FuncId]) {
        for method_id in function_ids {
            let method_name = self.interner.function_name(method_id).to_owned();

            if let Some(first_fn) =
                self.interner.add_method(self_type, method_name.clone(), *method_id, false)
            {
                let error = ResolverError::DuplicateDefinition {
                    name: method_name,
                    first_span: self.interner.function_ident(&first_fn).span(),
                    second_span: self.interner.function_ident(method_id).span(),
                };
                self.push_err(error);
            }
        }
    }

    fn define_type_alias(&mut self, alias_id: TypeAliasId, alias: UnresolvedTypeAlias) {
        self.file = alias.file_id;
        self.local_module = alias.module_id;

        let generics = self.add_generics(&alias.type_alias_def.generics);
        self.current_item = Some(DependencyId::Alias(alias_id));
        let typ = self.resolve_type(alias.type_alias_def.typ);
        self.interner.set_type_alias(alias_id, typ, generics);
        self.generics.clear();
    }

    fn collect_struct_definitions(&mut self, structs: &BTreeMap<StructId, UnresolvedStruct>) {
        // This is necessary to avoid cloning the entire struct map
        // when adding checks after each struct field is resolved.
        let struct_ids = structs.keys().copied().collect::<Vec<_>>();

        // Resolve each field in each struct.
        // Each struct should already be present in the NodeInterner after def collection.
        for (type_id, typ) in structs {
            self.file = typ.file_id;
            self.local_module = typ.module_id;

            let fields = self.resolve_struct_fields(&typ.struct_def, *type_id);
            let fields_len = fields.len();
            self.interner.update_struct(*type_id, |struct_def| {
                struct_def.set_fields(fields);

                // TODO(https://github.com/noir-lang/noir/issues/5156): Remove this with implicit numeric generics
                // This is only necessary for resolving named types when implicit numeric generics are used.
                let mut found_names = Vec::new();
                struct_def.find_numeric_generics_in_fields(&mut found_names);
                for generic in struct_def.generics.iter_mut() {
                    for found_generic in found_names.iter() {
                        if found_generic == generic.name.as_str() {
                            if matches!(generic.kind, Kind::Normal) {
                                let ident = Ident::new(generic.name.to_string(), generic.span);
                                self.errors.push((
                                    CompilationError::ResolverError(
                                        ResolverError::UseExplicitNumericGeneric { ident },
                                    ),
                                    self.file,
                                ));
                                generic.kind = Kind::Numeric(Box::new(Type::default_int_type()));
                            }
                            break;
                        }
                    }
                }
            });

            for field_index in 0..fields_len {
                self.interner.add_definition_location(
                    ReferenceId::StructMember(*type_id, field_index),
                    None,
                );
            }
        }

        // Check whether the struct fields have nested slices
        // We need to check after all structs are resolved to
        // make sure every struct's fields is accurately set.
        for id in struct_ids {
            let struct_type = self.interner.get_struct(id);

            // Only handle structs without generics as any generics args will be checked
            // after monomorphization when performing SSA codegen
            if struct_type.borrow().generics.is_empty() {
                let fields = struct_type.borrow().get_fields(&[]);
                for (_, field_type) in fields.iter() {
                    if field_type.is_nested_slice() {
                        let location = struct_type.borrow().location;
                        self.file = location.file;
                        self.push_err(ResolverError::NestedSlices { span: location.span });
                    }
                }
            }
        }
    }

    pub fn resolve_struct_fields(
        &mut self,
        unresolved: &NoirStruct,
        struct_id: StructId,
    ) -> Vec<(Ident, Type)> {
        self.recover_generics(|this| {
            this.current_item = Some(DependencyId::Struct(struct_id));

            this.resolving_ids.insert(struct_id);

            let struct_def = this.interner.get_struct(struct_id);
            this.add_existing_generics(&unresolved.generics, &struct_def.borrow().generics);

            let fields = vecmap(&unresolved.fields, |(ident, typ)| {
                (ident.clone(), this.resolve_type(typ.clone()))
            });

            this.resolving_ids.remove(&struct_id);

            fields
        })
    }

    fn elaborate_global(&mut self, global: UnresolvedGlobal) {
        let old_module = std::mem::replace(&mut self.local_module, global.module_id);
        let old_file = std::mem::replace(&mut self.file, global.file_id);
        let old_item = self.current_item.take();

        let global_id = global.global_id;
        self.current_item = Some(DependencyId::Global(global_id));
        let let_stmt = global.stmt_def;

        let name = if self.interner.is_in_lsp_mode() {
            Some(let_stmt.pattern.name_ident().to_string())
        } else {
            None
        };

        if !self.in_contract()
            && let_stmt.attributes.iter().any(|attr| matches!(attr, SecondaryAttribute::Abi(_)))
        {
            let span = let_stmt.pattern.span();
            self.push_err(ResolverError::AbiAttributeOutsideContract { span });
        }

        if !let_stmt.comptime && matches!(let_stmt.pattern, Pattern::Mutable(..)) {
            let span = let_stmt.pattern.span();
            self.push_err(ResolverError::MutableGlobal { span });
        }

        let comptime = let_stmt.comptime;

        let (let_statement, _typ) = self.elaborate_let(let_stmt, Some(global_id));
        let statement_id = self.interner.get_global(global_id).let_statement;
        self.interner.replace_statement(statement_id, let_statement);

        if comptime {
            self.elaborate_comptime_global(global_id);
        }

        if let Some(name) = name {
            self.interner.register_global(global_id, name, self.module_id());
        }

        self.local_module = old_module;
        self.file = old_file;
        self.current_item = old_item;
    }

    fn elaborate_comptime_global(&mut self, global_id: GlobalId) {
        let let_statement = self
            .interner
            .get_global_let_statement(global_id)
            .expect("Let statement of global should be set by elaborate_global_let");

        let global = self.interner.get_global(global_id);
        let definition_id = global.definition_id;
        let location = global.location;
        let mut interpreter = self.setup_interpreter();

        if let Err(error) = interpreter.evaluate_let(let_statement) {
            self.errors.push(error.into_compilation_error_pair());
        } else {
            let value = interpreter
                .lookup_id(definition_id, location)
                .expect("The global should be defined since evaluate_let did not error");

            self.debug_comptime(location, |interner| {
                interner.get_global(global_id).let_statement.to_display_ast(interner).kind
            });

            self.interner.get_global_mut(global_id).value = Some(value);
        }
    }

    fn define_function_metas(
        &mut self,
        functions: &mut [UnresolvedFunctions],
        impls: &mut ImplMap,
        trait_impls: &mut [UnresolvedTraitImpl],
    ) {
        for function_set in functions {
            self.define_function_metas_for_functions(function_set);
        }

        for ((self_type, local_module), function_sets) in impls {
            self.local_module = *local_module;

            for (generics, _, function_set) in function_sets {
                self.file = function_set.file_id;
                self.add_generics(generics);
                let self_type = self.resolve_type(self_type.clone());
                function_set.self_type = Some(self_type.clone());
                self.self_type = Some(self_type);
                self.define_function_metas_for_functions(function_set);
                self.self_type = None;
                self.generics.clear();
            }
        }

        for trait_impl in trait_impls {
            self.file = trait_impl.file_id;
            self.local_module = trait_impl.module_id;

            let trait_id = self.resolve_trait_by_path(trait_impl.trait_path.clone());
            trait_impl.trait_id = trait_id;
            let unresolved_type = trait_impl.object_type.clone();

            self.add_generics(&trait_impl.generics);
            trait_impl.resolved_generics = self.generics.clone();

            for (_, _, method) in trait_impl.methods.functions.iter_mut() {
                // Attach any trait constraints on the impl to the function
                method.def.where_clause.append(&mut trait_impl.where_clause.clone());
            }

            // Add each associated type to the list of named type arguments
            let mut trait_generics = trait_impl.trait_generics.clone();
            trait_generics.named_args.extend(self.take_unresolved_associated_types(trait_impl));

            let impl_id = self.interner.next_trait_impl_id();
            self.current_trait_impl = Some(impl_id);

            // Fetch trait constraints here
            let (ordered_generics, named_generics) = trait_impl
                .trait_id
                .map(|trait_id| {
                    self.resolve_type_args(trait_generics, trait_id, trait_impl.trait_path.span)
                })
                .unwrap_or_default();

            trait_impl.resolved_trait_generics = ordered_generics;
            self.interner.set_associated_types_for_impl(impl_id, named_generics);

            let self_type = self.resolve_type(unresolved_type);
            self.self_type = Some(self_type.clone());
            trait_impl.methods.self_type = Some(self_type);

            self.define_function_metas_for_functions(&mut trait_impl.methods);

            trait_impl.resolved_object_type = self.self_type.take();
            trait_impl.impl_id = self.current_trait_impl.take();
            self.generics.clear();

            if let Some(trait_id) = trait_id {
                let trait_name = trait_impl.trait_path.last_ident();
                self.interner.add_trait_reference(
                    trait_id,
                    Location::new(trait_name.span(), trait_impl.file_id),
                    trait_name.is_self_type_name(),
                );
            }
        }
    }

    fn define_function_metas_for_functions(&mut self, function_set: &mut UnresolvedFunctions) {
        self.file = function_set.file_id;

        for (local_module, id, func) in &mut function_set.functions {
            self.local_module = *local_module;
            self.recover_generics(|this| {
                this.define_function_meta(func, *id, None);
            });
        }
    }

    /// True if we're currently within a `comptime` block, function, or global
    fn in_comptime_context(&self) -> bool {
        // The first context is the global context, followed by the function-specific context.
        // Any context after that is a `comptime {}` block's.
        if self.function_context.len() > 2 {
            return true;
        }

        match self.current_item {
            Some(DependencyId::Function(id)) => self.interner.function_modifiers(&id).is_comptime,
            Some(DependencyId::Global(id)) => self.interner.get_global_definition(id).comptime,
            _ => false,
        }
    }

    /// True if we're currently within a constrained function.
    /// Defaults to `true` if the current function is unknown.
    fn in_constrained_function(&self) -> bool {
        !self.in_comptime_context()
            && self.current_item.map_or(true, |id| match id {
                DependencyId::Function(id) => {
                    !self.interner.function_modifiers(&id).is_unconstrained
                }
                _ => true,
            })
    }
}
