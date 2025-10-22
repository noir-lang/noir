//! # The Elaborator
//!
//! The elaborator is the core semantic analysis phase of the compiler. It transforms collected,
//! unresolved AST items into fully resolved and type-checked HIR through simultaneous and intertwined
//! name resolution, type checking, trait resolution, and macro expansion. It also handles pattern
//! elaboration, generic type processing, comptime interpretation, reference validation, and scope management.
//!
//! ## Architecture Overview
//!
//! The elaborator operates in several distinct phases, processing items in a specific order to handle
//! dependencies correctly:
//!
//! ### Early Resolution
//! 2. Globals - Set up their dependency ordering. Deferred for elaboration later after type resolution.
//!    Globals will be lazily elaborated when other types or expressions bring them into scope.
//! 3. Type aliases - Defined to allow their use in subsequent type definitions
//!
//! ### Type Collection
//! 1. Struct definitions - Collected so their types are available for use
//! 2. Enum definitions - Collected so their types are available for use
//! 3. Trait definitions - Collected so trait bounds can be resolved
//!
//! ### Function metadata and Implementations
//! 1. Function metadata - Signatures collected before bodies are elaborated
//! 2. Trait methods - Method signatures collected from trait definitions
//! 3. Impl blocks - Methods organized into their proper modules based on the impl's type
//! 4. Trait impls - Linked to their corresponding traits and validated
//!
//! ### Global Elaboration
//! - Elaborate any remaining globals which were not brought into scope.
//!   Elaborated after type resolution since they may use struct types which need global type information
//!
//! ### Attribute Processing
//! - Comptime attributes - Executed before function body elaboration, as generated items may change what is in scope or modify functions
//!
//! ### Full Item Elaboration
//! 1. Functions - Function bodies elaborated (resolved & type-checked)
//! 2. Traits - Trait default method implementations elaborated
//! 3. Impls - Implementation method bodies elaborated
//! 4. Trait impls - Trait implementation method bodies elaborated and validated against trait signatures
//!
//! ### Dependency Analysis
//! Detect and report dependency cycles to prevent infinite elaboration loops
//!
//! ## Error Handling
//!
//! The elaborator accumulates errors rather than failing fast, allowing it to report multiple
//! errors in a single compilation pass. Errors are collected throughout elaboration and may be
//! wrapped with additional context when elaborating generated code (e.g., from attributes or
//! comptime calls).

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{
    NamedGeneric, ast::UnresolvedType, graph::CrateGraph,
    hir::def_collector::dc_crate::UnresolvedTrait, usage_tracker::UsageTracker,
    validity::length_is_zero,
};
use crate::{
    Type, TypeVariable,
    ast::{
        BlockExpression, FunctionKind, GenericTypeArgs, Ident, NoirFunction, Param,
        UnresolvedGenerics, UnresolvedTypeData,
    },
    graph::CrateId,
    hir::{
        Context,
        comptime::ComptimeError,
        def_collector::{
            dc_crate::{
                CollectedItems, CompilationError, ImplMap, UnresolvedFunctions, UnresolvedGlobal,
                UnresolvedTraitImpl, UnresolvedTypeAlias,
            },
            errors::DefCollectorErrorKind,
        },
        def_map::{DefMaps, LocalModuleId, MAIN_FUNCTION, ModuleData, ModuleId},
        resolution::errors::ResolverError,
        scope::ScopeForest as GenericScopeForest,
        type_check::TypeCheckError,
    },
    hir_def::{
        expr::{HirCapturedVar, HirIdent},
        function::{FuncMeta, FunctionBody, HirFunction},
        traits::TraitConstraint,
        types::{Kind, ResolvedGeneric},
    },
    node_interner::{
        DefinitionKind, DependencyId, FuncId, FunctionModifiers, GlobalId, NodeInterner, TraitId,
        TraitImplId, TypeAliasId, TypeId,
    },
    parser::{ParserError, ParserErrorReason},
};

mod comptime;
mod enums;
mod expressions;
mod function_context;
mod generics;
mod globals;
mod impls;
mod lints;
mod options;
mod path_resolution;
mod patterns;
mod primitive_types;
mod scope;
mod statements;
mod structs;
mod trait_impls;
mod traits;
pub mod types;
mod unquote;
mod visibility;

use function_context::FunctionContext;
use iter_extended::vecmap;
use noirc_errors::Location;
pub(crate) use options::ElaboratorOptions;
pub use options::{FrontendOptions, UnstableFeature};
pub use path_resolution::Turbofish;
use path_resolution::{
    PathResolution, PathResolutionItem, PathResolutionMode, PathResolutionTarget,
};

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
        self.set_unresolved_globals_ordering(items.globals);

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
        self.elaborate_remaining_globals();

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

    fn elaborate_functions(&mut self, functions: UnresolvedFunctions) {
        for (_, id, _) in functions.functions {
            self.elaborate_function(id);
        }

        self.generics.clear();
        self.self_type = None;
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

        // Check arg and return-value are not more private than the function they are in.
        if self.should_check_function_args_and_return_are_not_more_private_than_function(
            &func_meta, &modifiers,
        ) {
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
            | FunctionKind::TraitFunctionWithoutBody => {
                if !body.statements.is_empty() {
                    panic!(
                        "Builtin, low-level, and trait function declarations cannot have a body"
                    );
                }
                (HirFunction::empty(), Type::Error)
            }
            FunctionKind::Oracle => {
                if !body.statements.is_empty() {
                    self.push_err(ResolverError::OracleWithBody {
                        location: func_meta.name.location,
                    });
                }
                (HirFunction::empty(), Type::Error)
            }
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
        let is_test_or_fuzz =
            func.attributes().is_test_function() || func.attributes().is_fuzzing_harness();

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
        let wildcard_allowed = false;

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
                _ => self.resolve_type_with_kind(typ, &Kind::Normal, wildcard_allowed),
            };

            self.check_if_type_is_valid_for_program_input(
                &typ,
                is_entry_point || is_test_or_fuzz,
                has_inline_attribute,
                type_location,
            );

            if is_entry_point || is_test_or_fuzz {
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

        let return_type = Box::new(self.use_type(func.return_type(), wildcard_allowed));

        // Temporary allow slices for contract functions, until contracts are re-factored.
        if !func.attributes().has_contract_library_method() {
            self.check_if_type_is_valid_for_program_output(
                &return_type,
                is_entry_point || is_test_or_fuzz,
                has_inline_attribute,
                location,
            );
        }

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
        if is_entry_point {
            if let Some(invalid_type) = typ.program_input_validity() {
                self.push_err(TypeCheckError::InvalidTypeForEntryPoint { invalid_type, location });
                return;
            }
        }

        if has_inline_attribute {
            if let Some(invalid_type) = typ.non_inlined_function_input_validity() {
                self.push_err(TypeCheckError::InvalidTypeForEntryPoint { invalid_type, location });
            }
        }
    }

    fn check_if_type_is_valid_for_program_output(
        &mut self,
        typ: &Type,
        is_entry_point: bool,
        has_inline_attribute: bool,
        location: Location,
    ) {
        match typ {
            Type::Unit => return,
            Type::Array(length, _) | Type::String(length) => {
                if length_is_zero(length) {
                    //returning zero length arrays is allowed
                    return;
                }
            }
            _ => (),
        }

        self.check_if_type_is_valid_for_program_input(
            typ,
            is_entry_point,
            has_inline_attribute,
            location,
        );
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

    pub fn get_module(&self, module: ModuleId) -> &ModuleData {
        let message = "A crate should always be present for a given crate id";
        &self.def_maps.get(&module.krate).expect(message)[module.local_id]
    }

    fn get_module_mut(def_maps: &mut DefMaps, module: ModuleId) -> &mut ModuleData {
        let message = "A crate should always be present for a given crate id";
        &mut def_maps.get_mut(&module.krate).expect(message)[module.local_id]
    }

    fn define_type_alias(&mut self, alias_id: TypeAliasId, alias: UnresolvedTypeAlias) {
        self.local_module = alias.module_id;

        let name = &alias.type_alias_def.name;
        let visibility = alias.type_alias_def.visibility;
        let location = alias.type_alias_def.location;

        let generics = self.add_generics(&alias.type_alias_def.generics);
        self.current_item = Some(DependencyId::Alias(alias_id));
        let wildcard_allowed = false;
        let (typ, num_expr) = if let Some(num_type) = alias.type_alias_def.numeric_type {
            let num_type = self.resolve_type(num_type, wildcard_allowed);
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
                    (
                        self.resolve_type_with_kind(
                            alias.type_alias_def.typ,
                            &kind,
                            wildcard_allowed,
                        ),
                        Some(num_expr),
                    )
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
            (self.use_type(alias.type_alias_def.typ, wildcard_allowed), None)
        };

        if !visibility.is_private() {
            self.check_type_is_not_more_private_then_item(name, visibility, &typ, location);
        }
        self.interner.set_type_alias(alias_id, typ, generics, num_expr);
        self.generics.clear();
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
                let wildcard_allowed = false;
                let self_type = self.resolve_type(self_type.clone(), wildcard_allowed);

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

            let wildcard_allowed = false;
            let self_type = self.resolve_type(unresolved_type, wildcard_allowed);
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
            Monomorphizer::new(elaborator.interner, DebugTypeTracker::default(), false);
        Ok(monomorphizer.expr(expr_id).expect("monomorphization error while converting interpreter execution result, should not be possible"))
    }
}
