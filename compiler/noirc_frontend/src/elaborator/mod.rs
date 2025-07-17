use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{
    DataType, NamedGeneric, StructField, TypeBindings,
    ast::{IdentOrQuotedType, ItemVisibility, UnresolvedType},
    graph::CrateGraph,
    hir::def_collector::dc_crate::UnresolvedTrait,
    hir_def::traits::ResolvedTraitBound,
    node_interner::{GlobalValue, QuotedTypeId},
    token::SecondaryAttributeKind,
    usage_tracker::UsageTracker,
};
use crate::{
    EnumVariant, Shared, Type, TypeVariable,
    ast::{
        BlockExpression, FunctionKind, GenericTypeArgs, Ident, NoirFunction, NoirStruct, Param,
        Path, Pattern, TraitBound, UnresolvedGeneric, UnresolvedGenerics,
        UnresolvedTraitConstraint, UnresolvedTypeData, UnsupportedNumericGenericType, Visitor,
    },
    graph::CrateId,
    hir::{
        Context,
        comptime::ComptimeError,
        def_collector::{
            dc_crate::{
                CollectedItems, CompilationError, ImplMap, UnresolvedEnum, UnresolvedFunctions,
                UnresolvedGlobal, UnresolvedStruct, UnresolvedTraitImpl, UnresolvedTypeAlias,
                filter_literal_globals,
            },
            errors::DefCollectorErrorKind,
        },
        def_map::{DefMaps, LocalModuleId, MAIN_FUNCTION, ModuleData, ModuleId},
        resolution::errors::ResolverError,
        scope::ScopeForest as GenericScopeForest,
        type_check::{TypeCheckError, generics::TraitGenerics},
    },
    hir_def::{
        expr::{HirCapturedVar, HirIdent},
        function::{FuncMeta, FunctionBody, HirFunction},
        traits::{TraitConstraint, TraitImpl},
        types::{Generics, Kind, ResolvedGeneric},
    },
    node_interner::{
        DefinitionKind, DependencyId, FuncId, FunctionModifiers, GlobalId, NodeInterner,
        ReferenceId, TraitId, TraitImplId, TypeAliasId, TypeId,
    },
    parser::{ParserError, ParserErrorReason},
};

mod comptime;
mod enums;
mod expressions;
mod function_context;
mod lints;
mod options;
mod path_resolution;
mod patterns;
mod primitive_types;
mod scope;
mod statements;
mod trait_impls;
mod traits;
pub mod types;
mod unquote;

use function_context::FunctionContext;
use fxhash::FxHashMap as HashMap;
use im::HashSet;
use iter_extended::vecmap;
use noirc_errors::{Located, Location};
pub(crate) use options::ElaboratorOptions;
pub use options::{FrontendOptions, UnstableFeature};
pub use path_resolution::Turbofish;
use path_resolution::{
    PathResolution, PathResolutionItem, PathResolutionMode, PathResolutionTarget,
};
use types::bind_ordered_generics;

use self::traits::check_trait_impl_method_matches_declaration;
pub(crate) use path_resolution::{TypedPath, TypedPathSegment};
pub use primitive_types::PrimitiveType;

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

/// Determines whether we are in an unsafe block and, if so, whether
/// any unconstrained calls were found in it (because if not we'll warn
/// that the unsafe block is not needed).
#[derive(Copy, Clone)]
enum UnsafeBlockStatus {
    NotInUnsafeBlock,
    InUnsafeBlockWithoutUnconstrainedCalls,
    InUnsafeBlockWithConstrainedCalls,
}

pub struct Loop {
    pub is_for: bool,
    pub has_break: bool,
}

pub struct Elaborator<'context> {
    scopes: ScopeForest,

    pub(crate) errors: Vec<CompilationError>,

    pub(crate) interner: &'context mut NodeInterner,
    pub(crate) def_maps: &'context mut DefMaps,
    pub(crate) usage_tracker: &'context mut UsageTracker,
    pub(crate) crate_graph: &'context CrateGraph,
    pub(crate) interpreter_output: &'context Option<Rc<RefCell<dyn std::io::Write>>>,

    required_unstable_features: &'context BTreeMap<CrateId, Vec<UnstableFeature>>,

    unsafe_block_status: UnsafeBlockStatus,
    current_loop: Option<Loop>,

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
    resolving_ids: BTreeSet<TypeId>,

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

    /// True if we're elaborating a comptime item such as a comptime function,
    /// block, global, or attribute.
    in_comptime_context: bool,

    crate_id: CrateId,

    /// These are the globals that have yet to be elaborated.
    /// This map is used to lazily evaluate these globals if they're encountered before
    /// they are elaborated (e.g. in a function's type or another global's RHS).
    unresolved_globals: BTreeMap<GlobalId, UnresolvedGlobal>,

    pub(crate) interpreter_call_stack: im::Vector<Location>,

    /// If greater than 0, field visibility errors won't be reported.
    /// This is used when elaborating a comptime expression that is a struct constructor
    /// like `Foo { inner: 5 }`: in that case we already elaborated the code that led to
    /// that comptime value and any visibility errors were already reported.
    silence_field_visibility_errors: usize,

    /// Options from the nargo cli
    options: ElaboratorOptions<'context>,

    /// Sometimes items are elaborated because a function attribute ran and generated items.
    /// The Elaborator keeps track of these reasons so that when an error is produced it will
    /// be wrapped in another error that will include this reason.
    pub(crate) elaborate_reasons: im::Vector<ElaborateReason>,
}

#[derive(Copy, Clone)]
pub enum ElaborateReason {
    /// A function attribute generated an item that's being elaborated.
    RunningAttribute(Location),
    /// Evaluating a comptime call like `Module::add_item`
    EvaluatingComptimeCall(&'static str, Location),
}

impl ElaborateReason {
    fn to_macro_error(self, error: CompilationError) -> ComptimeError {
        match self {
            ElaborateReason::RunningAttribute(location) => {
                ComptimeError::ErrorRunningAttribute { error: Box::new(error), location }
            }
            ElaborateReason::EvaluatingComptimeCall(method_name, location) => {
                let error = Box::new(error);
                ComptimeError::ErrorEvaluatingComptimeCall { method_name, error, location }
            }
        }
    }
}

impl<'context> Elaborator<'context> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        interner: &'context mut NodeInterner,
        def_maps: &'context mut DefMaps,
        usage_tracker: &'context mut UsageTracker,
        crate_graph: &'context CrateGraph,
        interpreter_output: &'context Option<Rc<RefCell<dyn std::io::Write>>>,
        required_unstable_features: &'context BTreeMap<CrateId, Vec<UnstableFeature>>,
        crate_id: CrateId,
        interpreter_call_stack: im::Vector<Location>,
        options: ElaboratorOptions<'context>,
        elaborate_reasons: im::Vector<ElaborateReason>,
    ) -> Self {
        Self {
            scopes: ScopeForest::default(),
            errors: Vec::new(),
            interner,
            def_maps,
            usage_tracker,
            crate_graph,
            interpreter_output,
            required_unstable_features,
            unsafe_block_status: UnsafeBlockStatus::NotInUnsafeBlock,
            current_loop: None,
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
            unresolved_globals: BTreeMap::new(),
            current_trait: None,
            interpreter_call_stack,
            in_comptime_context: false,
            silence_field_visibility_errors: 0,
            options,
            elaborate_reasons,
        }
    }

    pub fn from_context(
        context: &'context mut Context,
        crate_id: CrateId,
        options: ElaboratorOptions<'context>,
    ) -> Self {
        Self::new(
            &mut context.def_interner,
            &mut context.def_maps,
            &mut context.usage_tracker,
            &context.crate_graph,
            &context.interpreter_output,
            &context.required_unstable_features,
            crate_id,
            im::Vector::new(),
            options,
            im::Vector::new(),
        )
    }

    pub fn elaborate(
        context: &'context mut Context,
        crate_id: CrateId,
        items: CollectedItems,
        options: ElaboratorOptions<'context>,
    ) -> Vec<CompilationError> {
        Self::elaborate_and_return_self(context, crate_id, items, options).errors
    }

    pub fn elaborate_and_return_self(
        context: &'context mut Context,
        crate_id: CrateId,
        items: CollectedItems,
        options: ElaboratorOptions<'context>,
    ) -> Self {
        let mut this = Self::from_context(context, crate_id, options);
        this.elaborate_items(items);
        this.check_and_pop_function_context();
        this
    }

    pub(crate) fn elaborate_items(&mut self, mut items: CollectedItems) {
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

        // Must resolve types before we resolve globals.
        self.collect_struct_definitions(&items.structs);
        self.collect_enum_definitions(&items.enums);
        self.collect_traits(&mut items.traits);

        self.define_function_metas(&mut items.functions, &mut items.impls, &mut items.trait_impls);

        self.collect_trait_methods(&mut items.traits);

        // Before we resolve any function symbols we must go through our impls and
        // re-collect the methods within into their proper module. This cannot be
        // done during def collection since we need to be able to resolve the type of
        // the impl since that determines the module we should collect into.
        for ((self_type, module), impls) in &mut items.impls {
            self.collect_impls(*module, impls, self_type);
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
        self.run_attributes(
            &items.traits,
            &items.structs,
            &items.functions,
            &items.module_attributes,
        );

        for functions in items.functions {
            self.elaborate_functions(functions);
        }

        self.elaborate_traits(items.traits);

        for impls in items.impls.into_values() {
            self.elaborate_impls(impls);
        }

        for trait_impl in items.trait_impls {
            self.elaborate_trait_impl(trait_impl);
        }

        self.push_errors(self.interner.check_for_dependency_cycles());
    }

    /// True if we should use pedantic ACVM solving
    pub fn pedantic_solving(&self) -> bool {
        self.options.pedantic_solving
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
            if let Kind::Numeric(typ) = &generic.kind() {
                let definition =
                    DefinitionKind::NumericGeneric(generic.type_var.clone(), typ.clone());
                let ident = Ident::new(generic.name.to_string(), generic.location);
                let hir_ident = self.add_variable_decl(
                    ident, false, // mutable
                    false, // allow_shadowing
                    false, // warn_if_unused
                    definition,
                );
                self.interner.push_definition_type(hir_ident.id, *typ.clone());
            }
        }

        self.generics = all_generics;
    }

    pub(crate) fn elaborate_function(&mut self, id: FuncId) {
        let func_meta = self.interner.func_meta.get_mut(&id);
        let func_meta =
            func_meta.expect("FuncMetas should be declared before a function is elaborated");

        let (kind, body, body_location) = match func_meta.take_body() {
            FunctionBody::Unresolved(kind, body, location) => (kind, body, location),
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
        self.self_type = func_meta.self_type.clone();
        self.current_trait_impl = func_meta.trait_impl;

        self.scopes.start_function();
        let old_item = self.current_item.replace(DependencyId::Function(id));

        self.trait_bounds = func_meta.all_trait_constraints().cloned().collect();
        self.push_function_context();

        let modifiers = self.interner.function_modifiers(&id).clone();

        self.run_function_lints(&func_meta, &modifiers);

        // Check arg and return-value visibility of standalone functions.
        if self.should_check_function_visibility(&func_meta, &modifiers) {
            let name = Ident::new(
                self.interner.definition_name(func_meta.name.id).to_string(),
                func_meta.name.location,
            );
            for (_, typ, _) in func_meta.parameters.iter() {
                self.check_type_is_not_more_private_then_item(
                    &name,
                    modifiers.visibility,
                    typ,
                    name.location(),
                );
            }
            self.check_type_is_not_more_private_then_item(
                &name,
                modifiers.visibility,
                func_meta.return_type(),
                name.location(),
            );
        }

        self.introduce_generics_into_scope(func_meta.all_generics.clone());

        // The DefinitionIds for each parameter were already created in define_function_meta
        // so we need to reintroduce the same IDs into scope here.
        for parameter in &func_meta.parameter_idents {
            let name = self.interner.definition_name(parameter.id).to_owned();
            if name == "_" {
                continue;
            }
            let warn_if_unused = !(func_meta.trait_impl.is_some() && name == "self");
            self.add_existing_variable_to_scope(name, parameter.clone(), warn_if_unused);
        }

        self.add_trait_constraints_to_scope(func_meta.all_trait_constraints(), func_meta.location);

        let (hir_func, body_type) = match kind {
            FunctionKind::Builtin
            | FunctionKind::LowLevel
            | FunctionKind::Oracle
            | FunctionKind::TraitFunctionWithoutBody => (HirFunction::empty(), Type::Error),
            FunctionKind::Normal => {
                let return_type = func_meta.return_type();
                let (block, body_type) = self.elaborate_block(body, Some(return_type));
                let expr_id = self.intern_expr(block, body_location);
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

        self.remove_trait_constraints_from_scope(func_meta.all_trait_constraints());

        let func_scope_tree = self.scopes.end_function();

        // The arguments to low-level and oracle functions are always unused so we do not produce warnings for them.
        if !func_meta.is_stub() {
            self.check_for_unused_variables_in_scope_tree(func_scope_tree);
        }

        // Check that the body can return without calling the function.
        if let FunctionKind::Normal = kind {
            self.run_lint(|elaborator| {
                lints::unbounded_recursion(
                    elaborator.interner,
                    id,
                    || elaborator.interner.definition_name(func_meta.name.id),
                    func_meta.name.location,
                    hir_func.as_expr(),
                )
                .map(Into::into)
            });
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

        let new_generic = TypeVariable::unbound(new_generic_id, Kind::Normal);
        generics.push(new_generic.clone());

        let name = format!("impl {trait_path}");
        let generic_type = Type::NamedGeneric(NamedGeneric {
            type_var: new_generic,
            name: Rc::new(name),
            implicit: false,
        });
        let trait_bound = TraitBound { trait_path, trait_id: None, trait_generics };

        if let Some(trait_bound) = self.resolve_trait_bound(&trait_bound) {
            let new_constraint = TraitConstraint { typ: generic_type.clone(), trait_bound };
            trait_constraints.push(new_constraint);
        }

        generic_type
    }

    /// Add the given generics to scope.
    /// Each generic will have a fresh `Shared<TypeBinding>` associated with it.
    pub fn add_generics(&mut self, generics: &UnresolvedGenerics) -> Generics {
        vecmap(generics, |generic| {
            let mut is_error = false;
            let (type_var, name) = match self.resolve_generic(generic) {
                Ok(values) => values,
                Err(error) => {
                    self.push_err(error);
                    is_error = true;
                    let id = self.interner.next_type_variable_id();
                    let kind = self.resolve_generic_kind(generic);
                    (TypeVariable::unbound(id, kind), Rc::new("(error)".into()))
                }
            };

            let location = generic.location();
            let name_owned = name.as_ref().clone();
            let resolved_generic = ResolvedGeneric { name, type_var, location };

            // Check for name collisions of this generic
            // Checking `is_error` here prevents DuplicateDefinition errors when
            // we have multiple generics from macros which fail to resolve and
            // are all given the same default name "(error)".
            if !is_error {
                if let Some(generic) = self.find_generic(&name_owned) {
                    self.push_err(ResolverError::DuplicateDefinition {
                        name: name_owned,
                        first_location: generic.location,
                        second_location: location,
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
    ) -> Result<(TypeVariable, Rc<String>), ResolverError> {
        // Map the generic to a fresh type variable
        match generic.ident() {
            IdentOrQuotedType::Ident(ident) => {
                let id = self.interner.next_type_variable_id();
                let kind = self.resolve_generic_kind(generic);
                let typevar = TypeVariable::unbound(id, kind);
                let name = Rc::new(ident.to_string());
                Ok((typevar, name))
            }
            IdentOrQuotedType::Quoted(id, location) => {
                match self.interner.get_quoted_type(*id).follow_bindings() {
                    Type::NamedGeneric(NamedGeneric { type_var, name, .. }) => {
                        Ok((type_var.clone(), name))
                    }
                    other => Err(ResolverError::MacroResultInGenericsListNotAGeneric {
                        location: *location,
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
            let unresolved_typ = typ.clone();
            let typ = if unresolved_typ.is_type_expression() {
                self.resolve_type_with_kind(
                    unresolved_typ.clone(),
                    &Kind::numeric(Type::default_int_type()),
                )
            } else {
                self.resolve_type(unresolved_typ.clone())
            };
            if !matches!(typ, Type::FieldElement | Type::Integer(_, _)) {
                let unsupported_typ_err =
                    ResolverError::UnsupportedNumericGenericType(UnsupportedNumericGenericType {
                        name: ident.ident().map(|name| name.to_string()),
                        typ: typ.to_string(),
                        location: unresolved_typ.location,
                    });

                self.push_err(unsupported_typ_err);
            }
            Kind::numeric(typ)
        } else {
            Kind::Normal
        }
    }

    pub(crate) fn push_err(&mut self, error: impl Into<CompilationError>) {
        let error: CompilationError = error.into();
        self.errors.push(error);
    }

    pub(crate) fn push_errors(&mut self, errors: impl IntoIterator<Item = CompilationError>) {
        self.errors.extend(errors);
    }

    fn run_lint(&mut self, lint: impl Fn(&Elaborator) -> Option<CompilationError>) {
        if let Some(error) = lint(self) {
            self.push_err(error);
        }
    }

    pub(crate) fn resolve_module_by_path(&mut self, path: TypedPath) -> Option<ModuleId> {
        match self.resolve_path_as_type(path) {
            Ok(PathResolution { item: PathResolutionItem::Module(module_id), errors })
                if errors.is_empty() =>
            {
                Some(module_id)
            }
            _ => None,
        }
    }

    fn resolve_trait_by_path(&mut self, path: TypedPath) -> Option<TraitId> {
        let error = match self.resolve_path_as_type(path.clone()) {
            Ok(PathResolution { item: PathResolutionItem::Trait(trait_id), errors }) => {
                for error in errors {
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

    /// Resolve the given trait constraints and add them to scope as we go.
    /// This second step is necessary to resolve subsequent constraints such
    /// as `<T as Foo>::Bar: Eq` which may lookup an impl which was assumed
    /// by a previous constraint.
    ///
    /// If these constraints are unwanted afterward they should be manually
    /// removed from the interner.
    fn resolve_trait_constraints(
        &mut self,
        where_clause: &[UnresolvedTraitConstraint],
    ) -> Vec<TraitConstraint> {
        where_clause
            .iter()
            .filter_map(|constraint| self.resolve_trait_constraint(constraint))
            .collect()
    }

    /// Expands any traits in a where clause to mention all associated types if they were
    /// elided by the user. See `add_missing_named_generics` for more  detail.
    ///
    /// Returns all newly created generics to be added to this function/trait/impl.
    fn desugar_trait_constraints(
        &mut self,
        where_clause: &mut [UnresolvedTraitConstraint],
    ) -> Vec<(ResolvedGeneric, Vec<ResolvedTraitBound>)> {
        where_clause
            .iter_mut()
            .flat_map(|constraint| {
                self.add_missing_named_generics(&constraint.typ, &mut constraint.trait_bound)
            })
            .collect()
    }

    /// For each associated type that isn't mentioned in a trait bound, this adds
    /// the type as an implicit generic to the where clause and returns the newly
    /// created generics in a vector to add to the function/trait/impl later.
    /// For example, this will turn a function using a trait with 2 associated types:
    ///
    /// `fn foo<T>() where T: Foo { ... }`
    ///
    /// into:
    /// `fn foo<T>() where T: Foo<Bar = A, Baz = B> { ... }`
    ///
    /// with a vector of `<A, B>` returned so that the caller can then modify the function to:
    /// `fn foo<T, A, B>() where T: Foo<Bar = A, Baz = B> { ... }`
    fn add_missing_named_generics(
        &mut self,
        object: &UnresolvedType,
        bound: &mut TraitBound,
    ) -> Vec<(ResolvedGeneric, Vec<ResolvedTraitBound>)> {
        let mut added_generics = Vec::new();
        let trait_path = self.validate_path(bound.trait_path.clone());

        let Ok(PathResolutionItem::Trait(trait_id)) =
            self.resolve_path_or_error(trait_path, PathResolutionTarget::Type)
        else {
            return Vec::new();
        };

        let the_trait = self.get_trait_mut(trait_id);

        if the_trait.associated_types.len() > bound.trait_generics.named_args.len() {
            let trait_name = the_trait.name.to_string();
            let associated_type_bounds = the_trait.associated_type_bounds.clone();

            for associated_type in &the_trait.associated_types.clone() {
                if !bound
                    .trait_generics
                    .named_args
                    .iter()
                    .any(|(name, _)| name.as_str() == *associated_type.name.as_ref())
                {
                    // This generic isn't contained in the bound's named arguments,
                    // so add it by creating a fresh type variable.
                    let new_generic_id = self.interner.next_type_variable_id();
                    let kind = associated_type.type_var.kind();
                    let type_var = TypeVariable::unbound(new_generic_id, kind);

                    let location = bound.trait_path.location;
                    let name = format!("<{object} as {trait_name}>::{}", associated_type.name);
                    let name = Rc::new(name);
                    let typ = Type::NamedGeneric(NamedGeneric {
                        type_var: type_var.clone(),
                        name: name.clone(),
                        implicit: true,
                    });
                    let typ = self.interner.push_quoted_type(typ);
                    let typ = UnresolvedTypeData::Resolved(typ).with_location(location);
                    let ident = Ident::new(associated_type.name.as_ref().clone(), location);

                    let associated_type_bounds = associated_type_bounds
                        .get(associated_type.name.as_str())
                        .cloned()
                        .unwrap_or_default();

                    bound.trait_generics.named_args.push((ident, typ));
                    added_generics.push((
                        ResolvedGeneric { name, location, type_var },
                        associated_type_bounds,
                    ));
                }
            }
        }

        added_generics
    }

    /// Resolves a trait constraint and adds it to scope as an assumed impl.
    /// This second step is necessary to resolve subsequent constraints such
    /// as `<T as Foo>::Bar: Eq` which may lookup an impl which was assumed
    /// by a previous constraint.
    fn resolve_trait_constraint(
        &mut self,
        constraint: &UnresolvedTraitConstraint,
    ) -> Option<TraitConstraint> {
        let typ = self.resolve_type(constraint.typ.clone());
        let trait_bound = self.resolve_trait_bound(&constraint.trait_bound)?;
        let location = constraint.trait_bound.trait_path.location;

        self.add_trait_bound_to_scope(location, &typ, &trait_bound, trait_bound.trait_id);

        Some(TraitConstraint { typ, trait_bound })
    }

    pub fn resolve_trait_bound(&mut self, bound: &TraitBound) -> Option<ResolvedTraitBound> {
        self.resolve_trait_bound_inner(bound, PathResolutionMode::MarkAsReferenced)
    }

    pub fn use_trait_bound(&mut self, bound: &TraitBound) -> Option<ResolvedTraitBound> {
        self.resolve_trait_bound_inner(bound, PathResolutionMode::MarkAsUsed)
    }

    fn resolve_trait_bound_inner(
        &mut self,
        bound: &TraitBound,
        mode: PathResolutionMode,
    ) -> Option<ResolvedTraitBound> {
        let trait_path = self.validate_path(bound.trait_path.clone());
        let the_trait = self.lookup_trait_or_error(trait_path)?;
        let trait_id = the_trait.id;
        let location = bound.trait_path.location;

        let (ordered, named) =
            self.resolve_type_args_inner(bound.trait_generics.clone(), trait_id, location, mode);

        let trait_generics = TraitGenerics { ordered, named };
        Some(ResolvedTraitBound { trait_id, trait_generics, location })
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
        extra_trait_constraints: &[(TraitConstraint, Location)],
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

        let location = func.name_ident().location();
        let id = self.interner.function_definition_id(func_id);
        let name_ident = HirIdent::non_trait_method(id, location);

        let is_entry_point = self.is_entry_point_function(func, in_contract);

        // Both the #[fold] and #[no_predicates] alter a function's inline type and code generation in similar ways.
        // In certain cases such as type checking (for which the following flag will be used) both attributes
        // indicate we should code generate in the same way. Thus, we unify the attributes into one flag here.
        let has_no_predicates_attribute = func.attributes().is_no_predicates();
        let should_fold = func.attributes().is_foldable();
        let has_inline_attribute = has_no_predicates_attribute || should_fold;
        let is_pub_allowed = self.pub_allowed(func, in_contract);
        self.add_generics(&func.def.generics);

        let func_generics = vecmap(&self.generics, |generic| generic.type_var.clone());

        let associated_generics = self.desugar_trait_constraints(&mut func.def.where_clause);

        let mut generics = Vec::with_capacity(associated_generics.len());
        let mut associated_generics_trait_constraints = Vec::new();

        for (associated_generic, bounds) in associated_generics {
            for bound in bounds {
                let typ = Type::TypeVariable(associated_generic.type_var.clone());
                let location = associated_generic.location;
                self.add_trait_bound_to_scope(location, &typ, &bound, bound.trait_id);
                associated_generics_trait_constraints
                    .push(TraitConstraint { typ, trait_bound: bound });
            }

            generics.push(associated_generic.type_var);
        }

        for (extra_constraint, location) in extra_trait_constraints {
            let bound = &extra_constraint.trait_bound;
            self.add_trait_bound_to_scope(*location, &extra_constraint.typ, bound, bound.trait_id);
        }

        // We put associated generics first, as they are implicit and implicit generics
        // come before explicit generics (see `Type::instantiate_with`).
        generics.extend(func_generics);

        let mut trait_constraints = self.resolve_trait_constraints(&func.def.where_clause);
        let mut extra_trait_constraints =
            vecmap(extra_trait_constraints, |(constraint, _)| constraint.clone());
        extra_trait_constraints.extend(associated_generics_trait_constraints);

        let mut parameters = Vec::new();
        let mut parameter_types = Vec::new();
        let mut parameter_idents = Vec::new();

        for Param { visibility, pattern, typ, location: _ } in func.parameters().iter().cloned() {
            self.run_lint(|_| {
                lints::unnecessary_pub_argument(func, visibility, is_pub_allowed).map(Into::into)
            });

            let type_location = typ.location;
            let typ = match typ.typ {
                UnresolvedTypeData::TraitAsType(path, args) => {
                    self.desugar_impl_trait_arg(path, args, &mut generics, &mut trait_constraints)
                }
                // Function parameters have Kind::Normal
                _ => self.resolve_type_with_kind(typ, &Kind::Normal),
            };

            self.check_if_type_is_valid_for_program_input(
                &typ,
                is_entry_point,
                has_inline_attribute,
                type_location,
            );

            if is_entry_point {
                self.mark_type_as_used(&typ);
            }

            let pattern = self.elaborate_pattern_and_store_ids(
                pattern,
                typ.clone(),
                DefinitionKind::Local(None),
                &mut parameter_idents,
                true, // warn_if_unused
            );

            parameters.push((pattern, typ.clone(), visibility));
            parameter_types.push(typ);
        }

        let return_type = Box::new(self.use_type(func.return_type()));

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
            .filter_map(|generic| {
                generic.ident().ident().and_then(|name| self.find_generic(name.as_str())).cloned()
            })
            .collect();

        let statements = std::mem::take(&mut func.def.body.statements);
        let body = BlockExpression { statements };

        let struct_id = if let Some(Type::DataType(struct_type, _)) = &self.self_type {
            Some(struct_type.borrow().id)
        } else {
            None
        };

        // Remove the traits assumed by `resolve_trait_constraints` from scope
        self.remove_trait_constraints_from_scope(
            trait_constraints.iter().chain(extra_trait_constraints.iter()),
        );

        let meta = FuncMeta {
            name: name_ident,
            kind: func.kind,
            location,
            typ,
            direct_generics,
            all_generics: self.generics.clone(),
            type_id: struct_id,
            trait_id,
            trait_impl: self.current_trait_impl,
            enum_variant_index: None,
            parameters: parameters.into(),
            parameter_idents,
            return_type: func.def.return_type.clone(),
            return_visibility: func.def.return_visibility,
            has_body: !func.def.body.is_empty(),
            trait_constraints,
            extra_trait_constraints,
            is_entry_point,
            has_inline_attribute,
            source_crate: self.crate_id,
            source_module: self.local_module,
            function_body: FunctionBody::Unresolved(func.kind, body, func.def.location),
            self_type: self.self_type.clone(),
            source_file: location.file,
        };

        self.interner.push_fn_meta(meta, func_id);
        self.scopes.end_function();
        self.current_item = None;
    }

    fn mark_type_as_used(&mut self, typ: &Type) {
        match typ {
            Type::Array(_n, typ) => self.mark_type_as_used(typ),
            Type::Slice(typ) => self.mark_type_as_used(typ),
            Type::Tuple(types) => {
                for typ in types {
                    self.mark_type_as_used(typ);
                }
            }
            Type::DataType(datatype, generics) => {
                self.mark_struct_as_constructed(datatype.clone());
                for generic in generics {
                    self.mark_type_as_used(generic);
                }
                if let Some(fields) = datatype.borrow().get_fields(generics) {
                    for (_, typ, _) in fields {
                        self.mark_type_as_used(&typ);
                    }
                } else if let Some(variants) = datatype.borrow().get_variants(generics) {
                    for (_, variant_types) in variants {
                        for typ in variant_types {
                            self.mark_type_as_used(&typ);
                        }
                    }
                }
            }
            Type::Alias(alias_type, generics) => {
                self.mark_type_as_used(&alias_type.borrow().get_type(generics));
            }
            Type::CheckedCast { from, to } => {
                self.mark_type_as_used(from);
                self.mark_type_as_used(to);
            }
            Type::Reference(typ, _) => {
                self.mark_type_as_used(typ);
            }
            Type::InfixExpr(left, _op, right, _) => {
                self.mark_type_as_used(left);
                self.mark_type_as_used(right);
            }
            Type::FieldElement
            | Type::Integer(..)
            | Type::Bool
            | Type::String(_)
            | Type::FmtString(_, _)
            | Type::Unit
            | Type::Quoted(..)
            | Type::Constant(..)
            | Type::TraitAsType(..)
            | Type::TypeVariable(..)
            | Type::NamedGeneric(..)
            | Type::Function(..)
            | Type::Forall(..)
            | Type::Error => (),
        }
    }

    fn run_function_lints(&mut self, func: &FuncMeta, modifiers: &FunctionModifiers) {
        self.run_lint(|_| lints::inlining_attributes(func, modifiers).map(Into::into));
        self.run_lint(|_| lints::missing_pub(func, modifiers).map(Into::into));
        self.run_lint(|_| {
            let pub_allowed = func.is_entry_point || modifiers.attributes.is_foldable();
            lints::unnecessary_pub_return(func, modifiers, pub_allowed).map(Into::into)
        });
        self.run_lint(|_| lints::oracle_not_marked_unconstrained(func, modifiers).map(Into::into));
        self.run_lint(|elaborator| {
            lints::low_level_function_outside_stdlib(modifiers, elaborator.crate_id).map(Into::into)
        });
    }

    /// Only sized types are valid to be used as main's parameters or the parameters to a contract
    /// function. If the given type is not sized (e.g. contains a slice or NamedGeneric type), an
    /// error is issued.
    fn check_if_type_is_valid_for_program_input(
        &mut self,
        typ: &Type,
        is_entry_point: bool,
        has_inline_attribute: bool,
        location: Location,
    ) {
        if (is_entry_point && !typ.is_valid_for_program_input())
            || (has_inline_attribute && !typ.is_valid_non_inlined_function_input())
        {
            self.push_err(TypeCheckError::InvalidTypeForEntryPoint { location });
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

    fn add_trait_constraints_to_scope<'a>(
        &mut self,
        constraints: impl Iterator<Item = &'a TraitConstraint>,
        location: Location,
    ) {
        for constraint in constraints {
            self.add_trait_bound_to_scope(
                location,
                &constraint.typ,
                &constraint.trait_bound,
                constraint.trait_bound.trait_id,
            );
        }

        // Also assume `self` implements the current trait if we are inside a trait definition
        if let Some(trait_id) = self.current_trait {
            let the_trait = self.interner.get_trait(trait_id);
            let constraint = the_trait.as_constraint(the_trait.name.location());
            let self_type =
                self.self_type.clone().expect("Expected a self type if there's a current trait");
            self.add_trait_bound_to_scope(
                location,
                &self_type,
                &constraint.trait_bound,
                constraint.trait_bound.trait_id,
            );
        }
    }

    fn remove_trait_constraints_from_scope<'a>(
        &mut self,
        constraints: impl Iterator<Item = &'a TraitConstraint>,
    ) {
        for constraint in constraints {
            self.interner
                .remove_assumed_trait_implementations_for_trait(constraint.trait_bound.trait_id);
        }

        // Also remove the assumed trait implementation for `self` if this is a trait definition
        if let Some(trait_id) = self.current_trait {
            self.interner.remove_assumed_trait_implementations_for_trait(trait_id);
        }
    }

    fn add_trait_bound_to_scope(
        &mut self,
        location: Location,
        object: &Type,
        trait_bound: &ResolvedTraitBound,
        starting_trait_id: TraitId,
    ) {
        let trait_id = trait_bound.trait_id;
        let generics = trait_bound.trait_generics.clone();

        if !self.interner.add_assumed_trait_implementation(object.clone(), trait_id, generics) {
            if let Some(the_trait) = self.interner.try_get_trait(trait_id) {
                let trait_name = the_trait.name.to_string();
                let typ = object.clone();
                self.push_err(TypeCheckError::UnneededTraitConstraint {
                    trait_name,
                    typ,
                    location,
                });
            }
        }

        // Also add assumed implementations for the parent traits, if any
        if let Some(trait_bounds) =
            self.interner.try_get_trait(trait_id).map(|the_trait| the_trait.trait_bounds.clone())
        {
            for parent_trait_bound in trait_bounds {
                // Avoid looping forever in case there are cycles
                if parent_trait_bound.trait_id == starting_trait_id {
                    continue;
                }

                let parent_trait_bound =
                    self.instantiate_parent_trait_bound(trait_bound, &parent_trait_bound);
                self.add_trait_bound_to_scope(
                    location,
                    object,
                    &parent_trait_bound,
                    starting_trait_id,
                );
            }
        }
    }

    fn elaborate_traits(&mut self, traits: BTreeMap<TraitId, UnresolvedTrait>) {
        for (trait_id, unresolved_trait) in traits {
            self.current_trait = Some(trait_id);
            self.elaborate_functions(unresolved_trait.fns_with_default_impl);
        }
        self.current_trait = None;
    }

    fn elaborate_impls(&mut self, impls: Vec<(UnresolvedGenerics, Location, UnresolvedFunctions)>) {
        for (_, _, functions) in impls {
            self.recover_generics(|this| this.elaborate_functions(functions));
        }
    }

    fn elaborate_trait_impl(&mut self, trait_impl: UnresolvedTraitImpl) {
        self.local_module = trait_impl.module_id;

        self.generics = trait_impl.resolved_generics.clone();
        self.current_trait_impl = trait_impl.impl_id;

        self.add_trait_impl_assumed_trait_implementations(trait_impl.impl_id);
        self.check_trait_impl_where_clause_matches_trait_where_clause(&trait_impl);
        self.check_parent_traits_are_implemented(&trait_impl);
        self.remove_trait_impl_assumed_trait_implementations(trait_impl.impl_id);

        for (module, function, noir_function) in &trait_impl.methods.functions {
            self.local_module = *module;
            let errors = check_trait_impl_method_matches_declaration(
                self.interner,
                *function,
                noir_function,
            );
            self.push_errors(errors.into_iter().map(|error| error.into()));
        }

        self.elaborate_functions(trait_impl.methods);

        self.self_type = None;
        self.current_trait_impl = None;
        self.generics.clear();
    }

    fn add_trait_impl_assumed_trait_implementations(&mut self, impl_id: Option<TraitImplId>) {
        if let Some(impl_id) = impl_id {
            if let Some(trait_implementation) = self.interner.try_get_trait_implementation(impl_id)
            {
                for trait_constrain in &trait_implementation.borrow().where_clause {
                    let trait_bound = &trait_constrain.trait_bound;
                    self.add_trait_bound_to_scope(
                        trait_bound.location,
                        &trait_constrain.typ,
                        trait_bound,
                        trait_bound.trait_id,
                    );
                }
            }
        }
    }

    fn remove_trait_impl_assumed_trait_implementations(&mut self, impl_id: Option<TraitImplId>) {
        if let Some(impl_id) = impl_id {
            if let Some(trait_implementation) = self.interner.try_get_trait_implementation(impl_id)
            {
                for trait_constrain in &trait_implementation.borrow().where_clause {
                    self.interner.remove_assumed_trait_implementations_for_trait(
                        trait_constrain.trait_bound.trait_id,
                    );
                }
            }
        }
    }

    fn check_trait_impl_where_clause_matches_trait_where_clause(
        &mut self,
        trait_impl: &UnresolvedTraitImpl,
    ) {
        let Some(trait_id) = trait_impl.trait_id else {
            return;
        };

        let Some(the_trait) = self.interner.try_get_trait(trait_id) else {
            return;
        };

        if the_trait.where_clause.is_empty() {
            return;
        }

        let impl_trait = the_trait.name.to_string();

        let mut bindings = TypeBindings::default();
        bind_ordered_generics(
            &the_trait.generics,
            &trait_impl.resolved_trait_generics,
            &mut bindings,
        );

        // Check that each of the trait's where clause constraints is satisfied
        for trait_constraint in the_trait.where_clause.clone() {
            let Some(trait_constraint_trait) =
                self.interner.try_get_trait(trait_constraint.trait_bound.trait_id)
            else {
                continue;
            };

            let trait_constraint_type = trait_constraint.typ.substitute(&bindings);
            let trait_bound = &trait_constraint.trait_bound;

            if self
                .interner
                .try_lookup_trait_implementation(
                    &trait_constraint_type,
                    trait_bound.trait_id,
                    &trait_bound.trait_generics.ordered,
                    &trait_bound.trait_generics.named,
                )
                .is_err()
            {
                let missing_trait =
                    format!("{}{}", trait_constraint_trait.name, trait_bound.trait_generics);
                self.push_err(ResolverError::TraitNotImplemented {
                    impl_trait: impl_trait.clone(),
                    missing_trait,
                    type_missing_trait: trait_constraint_type.to_string(),
                    location: trait_impl.object_type.location,
                    missing_trait_location: trait_bound.location,
                });
            }
        }
    }

    fn check_parent_traits_are_implemented(&mut self, trait_impl: &UnresolvedTraitImpl) {
        let Some(trait_id) = trait_impl.trait_id else {
            return;
        };

        let Some(object_type) = &trait_impl.resolved_object_type else {
            return;
        };

        let Some(the_trait) = self.interner.try_get_trait(trait_id) else {
            return;
        };

        if the_trait.trait_bounds.is_empty() {
            return;
        }

        let impl_trait = the_trait.name.to_string();

        let mut bindings = TypeBindings::default();
        bind_ordered_generics(
            &the_trait.generics,
            &trait_impl.resolved_trait_generics,
            &mut bindings,
        );

        // Note: we only check if the immediate parents are implemented, we don't check recursively.
        // Why? If a parent isn't implemented, we get an error. If a parent is implemented, we'll
        // do the same check for the parent, so this trait's parents parents will be checked, so the
        // recursion is guaranteed.
        for parent_trait_bound in the_trait.trait_bounds.clone() {
            let Some(parent_trait) = self.interner.try_get_trait(parent_trait_bound.trait_id)
            else {
                continue;
            };

            let parent_trait_bound = ResolvedTraitBound {
                trait_generics: parent_trait_bound
                    .trait_generics
                    .map(|typ| typ.substitute(&bindings)),
                ..parent_trait_bound
            };

            if self
                .interner
                .try_lookup_trait_implementation(
                    object_type,
                    parent_trait_bound.trait_id,
                    &parent_trait_bound.trait_generics.ordered,
                    &parent_trait_bound.trait_generics.named,
                )
                .is_err()
            {
                let missing_trait =
                    format!("{}{}", parent_trait.name, parent_trait_bound.trait_generics);
                self.push_err(ResolverError::TraitNotImplemented {
                    impl_trait: impl_trait.clone(),
                    missing_trait,
                    type_missing_trait: trait_impl.object_type.to_string(),
                    location: trait_impl.object_type.location,
                    missing_trait_location: parent_trait_bound.location,
                });
            }
        }
    }

    fn collect_impls(
        &mut self,
        module: LocalModuleId,
        impls: &mut [(UnresolvedGenerics, Location, UnresolvedFunctions)],
        self_type: &UnresolvedType,
    ) {
        self.local_module = module;

        for (generics, location, unresolved) in impls {
            self.check_generics_appear_in_type(generics, self_type);

            let old_generic_count = self.generics.len();
            self.add_generics(generics);
            self.declare_methods_on_struct(None, unresolved, *location);
            self.generics.truncate(old_generic_count);
        }
    }

    fn collect_trait_impl(&mut self, trait_impl: &mut UnresolvedTraitImpl) {
        self.local_module = trait_impl.module_id;
        self.current_trait_impl = trait_impl.impl_id;

        let self_type = trait_impl.methods.self_type.clone();
        let self_type =
            self_type.expect("Expected struct type to be set before collect_trait_impl");

        self.self_type = Some(self_type.clone());
        let self_type_location = trait_impl.object_type.location;

        if matches!(self_type, Type::Reference(..)) {
            self.push_err(DefCollectorErrorKind::ReferenceInTraitImpl {
                location: self_type_location,
            });
        }

        if let Some(trait_id) = trait_impl.trait_id {
            self.generics = trait_impl.resolved_generics.clone();

            let where_clause = self.resolve_trait_constraints(&trait_impl.where_clause);

            // Now solve the actual type of associated types
            // (before this we only declared them without knowing their type)
            if let Some(trait_impl_id) = trait_impl.impl_id {
                let unresolved_associated_types =
                    std::mem::take(&mut trait_impl.unresolved_associated_types);
                let mut unresolved_associated_types =
                    unresolved_associated_types.into_iter().collect::<HashMap<_, _>>();

                let associated_types =
                    self.interner.get_associated_types_for_impl(trait_impl_id).to_vec();
                for associated_type in &associated_types {
                    let Type::NamedGeneric(named_generic) = &associated_type.typ else {
                        // This can happen if the associated type is specified directly in the impl trait generics,
                        // This can't be done in code, but it could happen with unquoted types.
                        continue;
                    };

                    let Some(unresolved_type) =
                        unresolved_associated_types.remove(&associated_type.name)
                    else {
                        // This too can happen if the associated type is specified directly in the impl trait generics,
                        // like `impl<H> BuildHasher<H = H>`, where `H` is a named generic but its resolution isn't delayed.
                        // This can't be done in code, but it could happen with unquoted types.
                        continue;
                    };
                    let resolved_type =
                        self.resolve_type_with_kind(unresolved_type, &associated_type.typ.kind());
                    named_generic.type_var.bind(resolved_type);
                }
            }

            let trait_ = self.interner.get_trait(trait_id);

            // If there are bounds on the trait's associated types, check them now
            let associated_type_bounds = &trait_.associated_type_bounds;
            if !associated_type_bounds.is_empty() {
                let associated_type_bounds = associated_type_bounds.clone();
                let named_generics = self
                    .interner
                    .get_associated_types_for_impl(trait_impl.impl_id.unwrap())
                    .to_vec();
                for named_generic in named_generics {
                    let Some(bounds) = associated_type_bounds.get(named_generic.name.as_str())
                    else {
                        continue;
                    };
                    let object_type = &named_generic.typ;
                    for bound in bounds {
                        if let Err(error) = self.interner.lookup_trait_implementation(
                            object_type,
                            bound.trait_id,
                            &bound.trait_generics.ordered,
                            &bound.trait_generics.named,
                        ) {
                            self.push_trait_constraint_error(
                                object_type,
                                error,
                                named_generic.name.location(),
                            );
                        }
                    }
                }
            }

            self.remove_trait_constraints_from_scope(where_clause.iter());

            self.collect_trait_impl_methods(trait_id, trait_impl, &where_clause);

            let location = trait_impl.object_type.location;
            self.declare_methods_on_struct(Some(trait_id), &mut trait_impl.methods, location);

            let trait_visibility = self.interner.get_trait(trait_id).visibility;

            let methods = trait_impl.methods.function_ids();
            for func_id in &methods {
                self.interner.set_function_trait(*func_id, self_type.clone(), trait_id);

                // A trait impl method has the same visibility as its trait
                let modifiers = self.interner.function_modifiers_mut(func_id);
                modifiers.visibility = trait_visibility;
            }

            let trait_generics = trait_impl.resolved_trait_generics.clone();
            let ident = match &trait_impl.r#trait.typ {
                UnresolvedTypeData::Named(trait_path, _, _) => trait_path.last_ident(),
                UnresolvedTypeData::Resolved(quoted_type_id) => {
                    let typ = self.interner.get_quoted_type(*quoted_type_id);
                    let name = if let Type::TraitAsType(_, name, _) = typ {
                        name.to_string()
                    } else {
                        typ.to_string()
                    };
                    Ident::new(name, trait_impl.r#trait.location)
                }
                _ => {
                    // We don't error in this case because an error will be produced later on when
                    // solving the trait impl trait type
                    Ident::new(trait_impl.r#trait.to_string(), trait_impl.r#trait.location)
                }
            };

            let resolved_trait_impl = Shared::new(TraitImpl {
                ident,
                location,
                typ: self_type.clone(),
                trait_id,
                trait_generics,
                file: trait_impl.file_id,
                crate_id: self.crate_id,
                where_clause,
                methods,
            });

            let generics = vecmap(&self.generics, |generic| generic.type_var.clone());

            if let Err(prev_location) = self.interner.add_trait_implementation(
                self_type.clone(),
                trait_id,
                trait_impl.impl_id.expect("impl_id should be set in define_function_metas"),
                generics,
                resolved_trait_impl,
            ) {
                self.push_err(DefCollectorErrorKind::OverlappingImpl {
                    typ: self_type.clone(),
                    location: self_type_location,
                    prev_location,
                });
            }
        }

        self.generics.clear();

        self.current_trait_impl = None;
        self.self_type = None;
    }

    pub fn get_module(&self, module: ModuleId) -> &ModuleData {
        let message = "A crate should always be present for a given crate id";
        &self.def_maps.get(&module.krate).expect(message)[module.local_id]
    }

    fn get_module_mut(def_maps: &mut DefMaps, module: ModuleId) -> &mut ModuleData {
        let message = "A crate should always be present for a given crate id";
        &mut def_maps.get_mut(&module.krate).expect(message)[module.local_id]
    }

    fn declare_methods_on_struct(
        &mut self,
        trait_id: Option<TraitId>,
        functions: &mut UnresolvedFunctions,
        location: Location,
    ) {
        let self_type = functions.self_type.as_ref();
        let self_type =
            self_type.expect("Expected struct type to be set before declare_methods_on_struct");

        let function_ids = functions.function_ids();

        if let Type::DataType(struct_type, _) = &self_type {
            let struct_ref = struct_type.borrow();

            // `impl`s are only allowed on types defined within the current crate
            if trait_id.is_none() && struct_ref.id.krate() != self.crate_id {
                let type_name = struct_ref.name.to_string();
                self.push_err(DefCollectorErrorKind::ForeignImpl { location, type_name });
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
                let result = if let Some(trait_id) = trait_id {
                    module.declare_trait_function(name, *method_id, trait_id)
                } else {
                    module.declare_function(name, method.def.visibility, *method_id)
                };
                if result.is_err() {
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
            if trait_id.is_none() {
                self.declare_methods(self_type, &function_ids);
            }
        // We can define methods on primitive types only if we're in the stdlib
        } else if trait_id.is_none() && *self_type != Type::Error {
            if self.crate_id.is_stdlib() {
                // Trait impl methods are already declared in NodeInterner::add_trait_implementation
                if trait_id.is_none() {
                    self.declare_methods(self_type, &function_ids);
                }
            } else {
                self.push_err(DefCollectorErrorKind::NonStructTypeInImpl { location });
            }
        }
    }

    fn declare_methods(&mut self, self_type: &Type, function_ids: &[FuncId]) {
        for method_id in function_ids {
            let method_name = self.interner.function_name(method_id).to_owned();

            if let Some(first_fn) =
                self.interner.add_method(self_type, method_name.clone(), *method_id, None)
            {
                let first_location = self.interner.function_ident(&first_fn).location();
                let second_location = self.interner.function_ident(method_id).location();
                let error = ResolverError::DuplicateDefinition {
                    name: method_name,
                    first_location,
                    second_location,
                };
                self.push_err(error);
            }
        }
    }

    fn define_type_alias(&mut self, alias_id: TypeAliasId, alias: UnresolvedTypeAlias) {
        self.local_module = alias.module_id;

        let name = &alias.type_alias_def.name;
        let visibility = alias.type_alias_def.visibility;
        let location = alias.type_alias_def.location;

        let generics = self.add_generics(&alias.type_alias_def.generics);
        self.current_item = Some(DependencyId::Alias(alias_id));

        let (typ, num_expr) = if let Some(num_type) = alias.type_alias_def.numeric_type {
            let num_type = self.resolve_type(num_type);
            let kind = Kind::numeric(num_type);
            let num_expr = alias.type_alias_def.typ.typ.try_into_expression();

            if let Some(num_expr) = num_expr {
                // Checks that the expression only references generics and constants
                if !num_expr.is_valid_expression() {
                    self.errors.push(CompilationError::ResolverError(
                        ResolverError::RecursiveTypeAlias {
                            location: alias.type_alias_def.numeric_location,
                        },
                    ));
                    (Type::Error, None)
                } else {
                    (self.resolve_type_with_kind(alias.type_alias_def.typ, &kind), Some(num_expr))
                }
            } else {
                self.errors.push(CompilationError::ResolverError(
                    ResolverError::ExpectedNumericExpression {
                        typ: alias.type_alias_def.typ.typ.to_string(),
                        location,
                    },
                ));
                (Type::Error, None)
            }
        } else {
            (self.use_type(alias.type_alias_def.typ), None)
        };

        if visibility != ItemVisibility::Private {
            self.check_type_is_not_more_private_then_item(name, visibility, &typ, location);
        }
        self.interner.set_type_alias(alias_id, typ, generics, num_expr);
        self.generics.clear();
    }

    /// Find the struct in the parent module so we can know its visibility
    fn find_struct_visibility(&self, struct_type: &DataType) -> Option<ItemVisibility> {
        let parent_module_id = struct_type.id.parent_module_id(self.def_maps);
        let parent_module_data = self.get_module(parent_module_id);
        let per_ns = parent_module_data.find_name(&struct_type.name);
        per_ns.types.map(|(_, vis, _)| vis)
    }

    /// Check whether a functions return value and args should be checked for private type visibility.
    fn should_check_function_visibility(
        &self,
        func_meta: &FuncMeta,
        modifiers: &FunctionModifiers,
    ) -> bool {
        // Private functions don't leak anything.
        if modifiers.visibility == ItemVisibility::Private {
            return false;
        }
        // Implementing public traits on private types is okay, they can't be used unless the type itself is accessible.
        if func_meta.trait_impl.is_some() {
            return false;
        }
        // Public struct functions should not expose private types.
        if let Some(struct_visibility) = func_meta.type_id.and_then(|id| {
            let struct_def = self.get_type(id);
            let struct_def = struct_def.borrow();
            self.find_struct_visibility(&struct_def)
        }) {
            return struct_visibility != ItemVisibility::Private;
        }
        // Standalone functions should be checked
        true
    }

    /// Check that an item such as a struct field or type alias is not more visible than the type it refers to.
    fn check_type_is_not_more_private_then_item(
        &mut self,
        name: &Ident,
        visibility: ItemVisibility,
        typ: &Type,
        location: Location,
    ) {
        match typ {
            Type::DataType(struct_type, generics) => {
                let struct_type = struct_type.borrow();
                let struct_module_id = struct_type.id.module_id();

                // We only check this in types in the same crate. If it's in a different crate
                // then it's either accessible (all good) or it's not, in which case a different
                // error will happen somewhere else, but no need to error again here.
                if struct_module_id.krate == self.crate_id {
                    if let Some(aliased_visibility) = self.find_struct_visibility(&struct_type) {
                        if aliased_visibility < visibility {
                            self.push_err(ResolverError::TypeIsMorePrivateThenItem {
                                typ: struct_type.name.to_string(),
                                item: name.to_string(),
                                location,
                            });
                        }
                    }
                }

                for generic in generics {
                    self.check_type_is_not_more_private_then_item(
                        name, visibility, generic, location,
                    );
                }
            }
            Type::Tuple(types) => {
                for typ in types {
                    self.check_type_is_not_more_private_then_item(name, visibility, typ, location);
                }
            }
            Type::Alias(alias_type, generics) => {
                self.check_type_is_not_more_private_then_item(
                    name,
                    visibility,
                    &alias_type.borrow().get_type(generics),
                    location,
                );
            }
            Type::CheckedCast { from, to } => {
                self.check_type_is_not_more_private_then_item(name, visibility, from, location);
                self.check_type_is_not_more_private_then_item(name, visibility, to, location);
            }
            Type::Function(args, return_type, env, _) => {
                for arg in args {
                    self.check_type_is_not_more_private_then_item(name, visibility, arg, location);
                }
                self.check_type_is_not_more_private_then_item(
                    name,
                    visibility,
                    return_type,
                    location,
                );
                self.check_type_is_not_more_private_then_item(name, visibility, env, location);
            }
            Type::Reference(typ, _) | Type::Array(_, typ) | Type::Slice(typ) => {
                self.check_type_is_not_more_private_then_item(name, visibility, typ, location);
            }
            Type::InfixExpr(left, _op, right, _) => {
                self.check_type_is_not_more_private_then_item(name, visibility, left, location);
                self.check_type_is_not_more_private_then_item(name, visibility, right, location);
            }
            Type::FieldElement
            | Type::Integer(..)
            | Type::Bool
            | Type::String(..)
            | Type::FmtString(..)
            | Type::Unit
            | Type::Quoted(..)
            | Type::TypeVariable(..)
            | Type::Forall(..)
            | Type::TraitAsType(..)
            | Type::Constant(..)
            | Type::NamedGeneric(..)
            | Type::Error => (),
        }
    }

    fn collect_struct_definitions(&mut self, structs: &BTreeMap<TypeId, UnresolvedStruct>) {
        // This is necessary to avoid cloning the entire struct map
        // when adding checks after each struct field is resolved.
        let struct_ids = structs.keys().copied().collect::<Vec<_>>();

        // Resolve each field in each struct.
        // Each struct should already be present in the NodeInterner after def collection.
        for (type_id, typ) in structs {
            self.local_module = typ.module_id;

            let fields = self.resolve_struct_fields(&typ.struct_def, *type_id);

            if typ.struct_def.is_abi() {
                for field in &fields {
                    self.mark_type_as_used(&field.typ);
                }
            }

            // Check that the a public struct doesn't have a private type as a public field.
            if typ.struct_def.visibility != ItemVisibility::Private {
                for field in &fields {
                    let ident = Ident::from(Located::from(
                        field.name.location(),
                        format!("{}::{}", typ.struct_def.name, field.name),
                    ));
                    self.check_type_is_not_more_private_then_item(
                        &ident,
                        field.visibility,
                        &field.typ,
                        field.name.location(),
                    );
                }
            }

            if self.interner.is_in_lsp_mode() {
                for (field_index, field) in fields.iter().enumerate() {
                    let location = field.name.location();
                    let reference_id = ReferenceId::StructMember(*type_id, field_index);
                    self.interner.add_definition_location(reference_id, location, None);
                }
            }

            self.interner.update_type(*type_id, |struct_def| {
                struct_def.set_fields(fields);
            });
        }

        // Check whether the struct fields have nested slices
        // We need to check after all structs are resolved to
        // make sure every struct's fields is accurately set.
        for id in struct_ids {
            let struct_type = self.interner.get_type(id);

            // Only handle structs without generics as any generics args will be checked
            // after monomorphization when performing SSA codegen
            if struct_type.borrow().generics.is_empty() {
                let fields = struct_type.borrow().get_fields(&[]).unwrap();
                for (_, field_type, _) in fields.iter() {
                    if field_type.is_nested_slice() {
                        let location = struct_type.borrow().location;
                        self.push_err(ResolverError::NestedSlices { location });
                    }
                }
            }
        }
    }

    pub fn resolve_struct_fields(
        &mut self,
        unresolved: &NoirStruct,
        struct_id: TypeId,
    ) -> Vec<StructField> {
        self.recover_generics(|this| {
            this.current_item = Some(DependencyId::Struct(struct_id));

            this.resolving_ids.insert(struct_id);

            let struct_def = this.interner.get_type(struct_id);
            this.add_existing_generics(&unresolved.generics, &struct_def.borrow().generics);

            let fields = vecmap(&unresolved.fields, |field| {
                let ident = &field.item.name;
                let typ = &field.item.typ;
                let visibility = field.item.visibility;
                StructField { visibility, name: ident.clone(), typ: this.resolve_type(typ.clone()) }
            });

            this.resolving_ids.remove(&struct_id);

            fields
        })
    }

    fn collect_enum_definitions(&mut self, enums: &BTreeMap<TypeId, UnresolvedEnum>) {
        for (type_id, typ) in enums {
            self.local_module = typ.module_id;
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
            let module_id = ModuleId { krate: self.crate_id, local_id: typ.module_id };
            self.resolving_ids.insert(*type_id);

            for (i, variant) in typ.enum_def.variants.iter().enumerate() {
                let parameters = variant.item.parameters.as_ref();
                let types =
                    parameters.map(|params| vecmap(params, |typ| self.resolve_type(typ.clone())));
                let name = variant.item.name.clone();

                let is_function = types.is_some();
                let params = types.clone().unwrap_or_default();
                datatype.borrow_mut().push_variant(EnumVariant::new(name, params, is_function));

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
                self.interner.add_definition_location(reference_id, location, Some(module_id));
            }

            self.resolving_ids.remove(type_id);
        }
        self.generics.clear();
    }

    fn elaborate_global(&mut self, global: UnresolvedGlobal) {
        let old_module = std::mem::replace(&mut self.local_module, global.module_id);
        let old_item = self.current_item.take();

        let global_id = global.global_id;
        self.current_item = Some(DependencyId::Global(global_id));
        let let_stmt = global.stmt_def;

        let name = if self.interner.is_in_lsp_mode() {
            Some(let_stmt.pattern.name_ident().to_string())
        } else {
            None
        };

        let location = let_stmt.pattern.location();

        if !self.in_contract() {
            for attr in &let_stmt.attributes {
                if matches!(attr.kind, SecondaryAttributeKind::Abi(_)) {
                    self.push_err(ResolverError::AbiAttributeOutsideContract {
                        location: attr.location,
                    });
                }
            }
        }

        if !let_stmt.comptime && matches!(let_stmt.pattern, Pattern::Mutable(..)) {
            self.push_err(ResolverError::MutableGlobal { location });
        }

        let (let_statement, _typ) = self
            .elaborate_in_comptime_context(|this| this.elaborate_let(let_stmt, Some(global_id)));

        let statement_id = self.interner.get_global(global_id).let_statement;
        self.interner.replace_statement(statement_id, let_statement);

        self.elaborate_comptime_global(global_id);

        if let Some(name) = name {
            self.interner.register_global(
                global_id,
                name,
                location,
                global.visibility,
                self.module_id(),
            );
        }

        self.local_module = old_module;
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
            let error: CompilationError = error.into();
            self.push_err(error);
        } else {
            let value = interpreter
                .lookup_id(definition_id, location)
                .expect("The global should be defined since evaluate_let did not error");

            self.debug_comptime(location, |interner| value.display(interner).to_string());

            self.interner.get_global_mut(global_id).value = GlobalValue::Resolved(value);
        }
    }

    /// If the given global is unresolved, elaborate it and return true
    fn elaborate_global_if_unresolved(&mut self, global_id: &GlobalId) -> bool {
        if let Some(global) = self.unresolved_globals.remove(global_id) {
            self.elaborate_global(global);
            true
        } else {
            false
        }
    }

    fn define_function_metas(
        &mut self,
        functions: &mut [UnresolvedFunctions],
        impls: &mut ImplMap,
        trait_impls: &mut [UnresolvedTraitImpl],
    ) {
        for function_set in functions {
            self.define_function_metas_for_functions(function_set, &[]);
        }

        for ((self_type, local_module), function_sets) in impls {
            self.local_module = *local_module;

            for (generics, _, function_set) in function_sets {
                self.add_generics(generics);
                let self_type = self.resolve_type(self_type.clone());

                function_set.self_type = Some(self_type.clone());
                self.self_type = Some(self_type);
                self.define_function_metas_for_functions(function_set, &[]);
                self.self_type = None;
                self.generics.clear();
            }
        }

        for trait_impl in trait_impls {
            self.local_module = trait_impl.module_id;

            let (trait_id, mut trait_generics, path_location) = match &trait_impl.r#trait.typ {
                UnresolvedTypeData::Named(trait_path, trait_generics, _) => {
                    let mut trait_generics = trait_generics.clone();
                    let location = trait_path.location;
                    let trait_path = self.validate_path(trait_path.clone());
                    let trait_id = self.resolve_trait_by_path(trait_path);

                    // Check and remove and any generic that is specifying an associated item
                    if !trait_generics.named_args.is_empty() {
                        if let Some(trait_id) = trait_id {
                            let associated_types =
                                self.interner.get_trait(trait_id).associated_types.clone();
                            trait_generics.named_args.retain(|(name, typ)| {
                                let associated_type = associated_types.iter().find(|associated_type| {
                                    associated_type.name.as_str() == name.as_str()
                                });
                                if associated_type.is_some() {
                                    let location = name.location().merge(typ.location);
                                    self.push_err(
                                        ResolverError::AssociatedItemConstraintsNotAllowedInGenerics {
                                            location,
                                        },
                                    );
                                    false
                                } else {
                                    true
                                }
                            });
                        }
                    }

                    (trait_id, trait_generics.clone(), location)
                }
                UnresolvedTypeData::Resolved(quoted_type_id) => {
                    let typ = self.interner.get_quoted_type(*quoted_type_id);
                    let location = trait_impl.r#trait.location;
                    let Type::TraitAsType(trait_id, _, trait_generics) = typ else {
                        let found = typ.to_string();
                        self.push_err(ResolverError::ExpectedTrait { location, found });
                        continue;
                    };

                    // In order to take associated types into account we turn these resolved generics
                    // into unresolved ones, but ones that point to solved types.
                    let trait_id = *trait_id;
                    let trait_generics = trait_generics.clone();
                    let trait_generics = GenericTypeArgs {
                        ordered_args: vecmap(&trait_generics.ordered, |typ| {
                            let quoted_type_id = self.interner.push_quoted_type(typ.clone());
                            let typ = UnresolvedTypeData::Resolved(quoted_type_id);
                            UnresolvedType { typ, location }
                        }),
                        named_args: vecmap(&trait_generics.named, |named_type| {
                            let quoted_type_id =
                                self.interner.push_quoted_type(named_type.typ.clone());
                            let typ = UnresolvedTypeData::Resolved(quoted_type_id);
                            (named_type.name.clone(), UnresolvedType { typ, location })
                        }),
                        kinds: Vec::new(),
                    };

                    (Some(trait_id), trait_generics, location)
                }
                _ => {
                    let location = trait_impl.r#trait.location;
                    let found = trait_impl.r#trait.typ.to_string();
                    self.push_err(ResolverError::ExpectedTrait { location, found });
                    (None, GenericTypeArgs::default(), location)
                }
            };

            trait_impl.trait_id = trait_id;
            let unresolved_type = trait_impl.object_type.clone();

            self.add_generics(&trait_impl.generics);
            trait_impl.resolved_generics = self.generics.clone();

            let new_generics = self.desugar_trait_constraints(&mut trait_impl.where_clause);
            let mut new_generics_trait_constraints = Vec::new();
            for (new_generic, bounds) in new_generics {
                for bound in bounds {
                    let typ = Type::TypeVariable(new_generic.type_var.clone());
                    let location = new_generic.location;
                    self.add_trait_bound_to_scope(location, &typ, &bound, bound.trait_id);
                    new_generics_trait_constraints
                        .push((TraitConstraint { typ, trait_bound: bound }, location));
                }
                trait_impl.resolved_generics.push(new_generic.clone());
                self.generics.push(new_generic);
            }

            // We need to resolve the where clause before any associated types to be
            // able to resolve trait as type syntax, eg. `<T as Foo>` in case there
            // is a where constraint for `T: Foo`.
            let constraints = self.resolve_trait_constraints(&trait_impl.where_clause);

            for (_, _, method) in trait_impl.methods.functions.iter_mut() {
                // Attach any trait constraints on the impl to the function
                method.def.where_clause.append(&mut trait_impl.where_clause.clone());
            }

            let impl_id = self.interner.next_trait_impl_id();
            self.current_trait_impl = Some(impl_id);

            // Add each associated type to the list of named type arguments
            let associated_types = self.take_unresolved_associated_types(trait_impl);

            // Put every associated type behind a type variable (inside a NamedGeneric).
            // This way associated types can be referred to even if their actual value (for associated constants)
            // is not known yet. This is to allow associated constants to refer to associated constants
            // in other trait impls.
            let associated_types_behind_type_vars =
                vecmap(&associated_types, |(name, _typ, kind)| {
                    let new_generic_id = self.interner.next_type_variable_id();
                    let type_var = TypeVariable::unbound(new_generic_id, kind.clone());
                    let typ = Type::NamedGeneric(NamedGeneric {
                        type_var: type_var.clone(),
                        name: Rc::new(name.to_string()),
                        implicit: false,
                    });
                    let typ = self.interner.push_quoted_type(typ);
                    let typ = UnresolvedTypeData::Resolved(typ).with_location(name.location());
                    (name.clone(), typ)
                });

            trait_generics.named_args.extend(associated_types_behind_type_vars);

            let associated_types = vecmap(associated_types, |(name, typ, _kind)| (name, typ));

            let (ordered_generics, named_generics) = trait_impl
                .trait_id
                .map(|trait_id| {
                    // Check for missing generics & associated types for the trait being implemented
                    self.resolve_trait_args_from_trait_impl(trait_generics, trait_id, path_location)
                })
                .unwrap_or_default();

            trait_impl.resolved_trait_generics = ordered_generics;
            self.interner.set_associated_types_for_impl(impl_id, named_generics);

            self.remove_trait_constraints_from_scope(
                constraints
                    .iter()
                    .chain(new_generics_trait_constraints.iter().map(|(constraint, _)| constraint)),
            );

            let self_type = self.resolve_type(unresolved_type);
            self.self_type = Some(self_type.clone());
            trait_impl.methods.self_type = Some(self_type);

            self.define_function_metas_for_functions(
                &mut trait_impl.methods,
                &new_generics_trait_constraints,
            );

            trait_impl.resolved_object_type = self.self_type.take();
            trait_impl.impl_id = self.current_trait_impl.take();
            trait_impl.unresolved_associated_types = associated_types;
            self.generics.clear();

            if let Some(trait_id) = trait_id {
                let (location, is_self_type_name) = match &trait_impl.r#trait.typ {
                    UnresolvedTypeData::Named(trait_path, _, _) => {
                        let trait_name = trait_path.last_ident();
                        (trait_name.location(), trait_name.is_self_type_name())
                    }
                    _ => (trait_impl.r#trait.location, false),
                };
                self.interner.add_trait_reference(trait_id, location, is_self_type_name);
            }
        }
    }

    fn define_function_metas_for_functions(
        &mut self,
        function_set: &mut UnresolvedFunctions,
        extra_constraints: &[(TraitConstraint, Location)],
    ) {
        for (local_module, id, func) in &mut function_set.functions {
            self.local_module = *local_module;
            self.recover_generics(|this| {
                this.define_function_meta(func, *id, None, extra_constraints);
            });
        }
    }

    /// True if we're currently within a constrained function.
    /// Defaults to `true` if the current function is unknown.
    fn in_constrained_function(&self) -> bool {
        !self.in_comptime_context()
            && self.current_item.is_none_or(|id| match id {
                DependencyId::Function(id) => {
                    !self.interner.function_modifiers(&id).is_unconstrained
                }
                _ => true,
            })
    }

    /// Check that all the generics show up in `self_type` (if they don't, we produce an error)
    fn check_generics_appear_in_type(
        &mut self,
        generics: &[UnresolvedGeneric],
        self_type: &UnresolvedType,
    ) {
        if generics.is_empty() {
            return;
        }

        // Turn each generic into an Ident
        let mut idents = HashSet::new();
        for generic in generics {
            match generic.ident() {
                IdentOrQuotedType::Ident(ident) => {
                    idents.insert(ident.clone());
                }
                IdentOrQuotedType::Quoted(quoted_type_id, location) => {
                    if let Type::NamedGeneric(NamedGeneric { name, .. }) =
                        self.interner.get_quoted_type(*quoted_type_id).follow_bindings()
                    {
                        idents.insert(Ident::new(name.to_string(), *location));
                    }
                }
            }
        }

        // Remove the ones that show up in `self_type`
        let mut visitor =
            RemoveGenericsAppearingInTypeVisitor { interner: self.interner, idents: &mut idents };
        self_type.accept(&mut visitor);

        // The ones that remain are not mentioned in the impl: it's an error.
        for ident in idents {
            self.push_err(ResolverError::UnconstrainedTypeParameter { ident });
        }
    }

    /// Register a use of the given unstable feature. Errors if the feature has not
    /// been explicitly enabled in this package.
    pub fn use_unstable_feature(&mut self, feature: UnstableFeature, location: Location) {
        // Is the feature globally enabled via CLI options?
        if self.options.enabled_unstable_features.contains(&feature) {
            return;
        }

        // Can crates require unstable features in their manifest?
        let enable_required_unstable_features = self.options.enabled_unstable_features.is_empty()
            && !self.options.disable_required_unstable_features;

        // Is it required by the current crate?
        if enable_required_unstable_features
            && self
                .required_unstable_features
                .get(&self.crate_id)
                .is_some_and(|fs| fs.contains(&feature))
        {
            return;
        }

        let reason = ParserErrorReason::ExperimentalFeature(feature);
        self.push_err(ParserError::with_reason(reason, location));
    }

    /// Run the given function using the resolver and return true if any errors (not warnings)
    /// occurred while running it.
    pub fn errors_occurred_in<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> (bool, T) {
        let previous_errors = self.errors.len();
        let ret = f(self);
        let errored = self.errors.iter().skip(previous_errors).any(|error| error.is_error());
        (errored, ret)
    }
}

struct RemoveGenericsAppearingInTypeVisitor<'a> {
    interner: &'a NodeInterner,
    idents: &'a mut HashSet<Ident>,
}

impl Visitor for RemoveGenericsAppearingInTypeVisitor<'_> {
    fn visit_path(&mut self, path: &Path) {
        if let Some(ident) = path.as_ident() {
            self.idents.remove(ident);
        }
    }

    fn visit_resolved_type(&mut self, id: QuotedTypeId, location: Location) {
        if let Type::NamedGeneric(NamedGeneric { name, .. }) =
            self.interner.get_quoted_type(id).follow_bindings()
        {
            self.idents.remove(&Ident::new(name.as_ref().clone(), location));
        }
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use std::cell::RefCell;
    use std::io::Write;
    use std::rc::Rc;

    use crate::hir::comptime::InterpreterError;
    use crate::{hir::def_collector::dc_crate::CompilationError, parser::ParserError};

    /// The possible errors of interpreting given code
    /// into a monomorphized AST expression.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ElaboratorError {
        Parse(Vec<ParserError>),
        Compile(Vec<CompilationError>),
        Interpret(InterpreterError),
        HIRConvert(InterpreterError),
    }

    /// Interpret source code using the elaborator, without
    /// parsing and compiling it with nargo, converting
    /// the result into a monomorphized AST expression.
    pub fn interpret<W: Write + 'static>(
        src: &str,
        output: Rc<RefCell<W>>,
    ) -> Result<crate::monomorphization::ast::Expression, ElaboratorError> {
        use crate::elaborator::ElaboratorOptions;
        use crate::monomorphization::{Monomorphizer, debug_types::DebugTypeTracker};
        use crate::parse_program;
        use crate::{
            elaborator::Elaborator,
            hir::{
                Context, ParsedFiles,
                def_collector::{dc_crate::DefCollector, dc_mod::collect_defs},
                def_map::{CrateDefMap, ModuleData},
            },
        };
        use fm::{FileId, FileManager};
        use noirc_errors::Location;
        use std::path::PathBuf;

        let file = FileId::default();

        let location = Location::new(Default::default(), file);
        let root_module = ModuleData::new(
            None,
            location,
            Vec::new(),
            Vec::new(),
            false, // is contract
            false, // is struct
        );

        let file_manager = FileManager::new(&PathBuf::new());
        let parsed_files = ParsedFiles::new();
        let mut context = Context::new(file_manager, parsed_files);
        context.def_interner.populate_dummy_operator_traits();
        context.set_comptime_printing(output);

        let krate = context.crate_graph.add_crate_root(FileId::dummy());

        let (module, errors) = parse_program(src, file);
        // Skip parser warnings
        let errors: Vec<_> = errors.iter().filter(|e| !e.is_warning()).cloned().collect();
        if !errors.is_empty() {
            return Err(ElaboratorError::Parse(errors));
        }

        let ast = module.into_sorted();

        let def_map = CrateDefMap::new(krate, root_module);
        let root_module_id = def_map.root();
        let mut collector = DefCollector::new(def_map);

        collect_defs(&mut collector, ast, FileId::dummy(), root_module_id, krate, &mut context);
        context.def_maps.insert(krate, collector.def_map);

        let main = context.get_main_function(&krate).expect("Expected 'main' function");

        let mut elaborator = Elaborator::elaborate_and_return_self(
            &mut context,
            krate,
            collector.items,
            ElaboratorOptions::test_default(),
        );

        // Skip the elaborator's compilation warnings
        let errors: Vec<_> = elaborator.errors.iter().filter(|&e| e.is_error()).cloned().collect();
        if !errors.is_empty() {
            return Err(ElaboratorError::Compile(errors));
        }

        let mut interpreter = elaborator.setup_interpreter();

        // The most straightforward way to convert the interpreter result into
        // an acceptable monomorphized AST expression seems to be converting it
        // into HIR first and then processing it with the monomorphizer
        let expr_id = match interpreter.call_function(
            main,
            Vec::new(),
            Default::default(),
            Location::dummy(),
        ) {
            Err(e) => return Err(ElaboratorError::Interpret(e)),
            Ok(value) => match value.into_hir_expression(elaborator.interner, Location::dummy()) {
                Err(e) => return Err(ElaboratorError::HIRConvert(e)),
                Ok(expr_id) => expr_id,
            },
        };

        let mut monomorphizer =
            Monomorphizer::new(elaborator.interner, DebugTypeTracker::default());
        Ok(monomorphizer.expr(expr_id).expect("monomorphization error while converting interpreter execution result, should not be possible"))
    }
}
