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
//! 1. Globals - Set up their dependency ordering. Deferred for elaboration later after type resolution.
//!    Globals will be lazily elaborated when other types or expressions bring them into scope.
//! 2. Type aliases - Defined to allow their use in subsequent type definitions
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
    Type,
    ast::UnresolvedGenerics,
    elaborator::types::WildcardDisallowedContext,
    graph::CrateId,
    hir::{
        Context,
        comptime::ComptimeError,
        def_collector::{
            dc_crate::{
                CollectedItems, CompilationError, UnresolvedFunctions, UnresolvedGlobal,
                UnresolvedTraitImpl, UnresolvedTypeAlias,
            },
            errors::DefCollectorErrorKind,
        },
        def_map::{DefMaps, LocalModuleId, ModuleData, ModuleId},
        resolution::errors::ResolverError,
        scope::ScopeForest as GenericScopeForest,
    },
    hir_def::{
        expr::{HirCapturedVar, HirIdent},
        traits::TraitConstraint,
        types::{Kind, ResolvedGeneric},
    },
    node_interner::{
        DependencyId, ExprId, GlobalId, NodeInterner, StmtId, TraitId, TraitImplId, TypeAliasId,
        TypeId,
    },
    parser::{ParserError, ParserErrorReason},
};
use crate::{
    graph::CrateGraph, hir::def_collector::dc_crate::UnresolvedTrait, usage_tracker::UsageTracker,
};
use rustc_hash::FxHashSet as HashSet;

mod comptime;
mod enums;
mod expressions;
mod function;
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
mod variable;
mod visibility;

use function_context::FunctionContext;
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
    /// If we know this lambda to be unconstrained.
    pub unconstrained: bool,
}

/// Determines whether we are in an unsafe block and, if so, whether
/// any unconstrained calls were found in it (because if not we'll warn
/// that the unsafe block is not needed).
#[derive(Copy, Clone)]
enum UnsafeBlockStatus {
    NotInUnsafeBlock,
    InUnsafeBlockWithoutUnconstrainedCalls,
    InUnsafeBlockWithUnconstrainedCalls,
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
    /// Initially None, it is set whenever a new top-level item is resolved.
    local_module: Option<LocalModuleId>,

    /// True if we're elaborating a comptime item such as a comptime function,
    /// block, global, or attribute.
    in_comptime_context: bool,

    /// True if we are elaborating arguments of a function call to an unconstrained function.
    in_unconstrained_args: bool,

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

    /// Set to true when the interpreter encounters an errored expression/statement,
    /// causing all subsequent comptime evaluation to be skipped.
    pub(crate) comptime_evaluation_halted: bool,
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
            local_module: None,
            crate_id,
            resolving_ids: BTreeSet::new(),
            trait_bounds: Vec::new(),
            function_context: vec![FunctionContext::default()],
            current_trait_impl: None,
            unresolved_globals: BTreeMap::new(),
            current_trait: None,
            interpreter_call_stack,
            in_comptime_context: false,
            in_unconstrained_args: false,
            silence_field_visibility_errors: 0,
            options,
            elaborate_reasons,
            comptime_evaluation_halted: false,
        }
    }

    pub(crate) fn local_module(&self) -> LocalModuleId {
        self.local_module.expect("local_module is unset")
    }

    /// Returns `true` if the current local module is the crate root,
    /// and we are not inside an impl or trait impl.
    pub(crate) fn is_at_crate_root(&self) -> bool {
        self.self_type.is_none()
            && self.current_trait.is_none()
            && self.current_trait_impl.is_none()
            && self.local_module.is_some_and(|id| id == self.def_maps[&self.crate_id].root())
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

        // Before we resolve any function symbols we must go through our impls and
        // re-collect the methods within into their proper module. This cannot be
        // done during def collection since we need to be able to resolve the type of
        // the impl since that determines the module we should collect into.
        for ((self_type, module), impls) in &mut items.impls {
            self.collect_impls(*module, impls, self_type);
        }

        self.collect_trait_methods(&mut items.traits);

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

    pub(crate) fn push_err(&mut self, error: impl Into<CompilationError>) {
        let error: CompilationError = error.into();
        // Filter out internal control flow errors that should not be displayed
        if !error.should_be_filtered() {
            self.errors.push(error);
        }
    }

    pub(crate) fn push_errors<E: Into<CompilationError>>(
        &mut self,
        errors: impl IntoIterator<Item = E>,
    ) {
        for error in errors {
            self.push_err(error);
        }
    }

    /// Run a given function while also tracking whether any new errors were generated as a result.
    pub(crate) fn with_error_guard<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> (T, bool) {
        // Count actual errors (ignore warnings)
        let initial_error_count = self.errors.len();
        let result = f(self);
        let has_new_errors = self.errors[initial_error_count..].iter().any(|e| e.is_error());
        (result, has_new_errors)
    }

    fn run_lint(&mut self, lint: impl Fn(&Elaborator) -> Option<CompilationError>) {
        if let Some(error) = lint(self) {
            self.push_err(error);
        }
    }

    pub(crate) fn resolve_module_by_path(&mut self, path: TypedPath) -> Option<ModuleId> {
        match self.resolve_path_as_type(path) {
            Ok(PathResolution { item: PathResolutionItem::Module(module_id), errors }) => {
                self.push_errors(errors);
                Some(module_id)
            }
            _ => None,
        }
    }

    fn resolve_trait_by_path(&mut self, path: TypedPath) -> Option<TraitId> {
        let error = match self.resolve_path_as_type(path.clone()) {
            Ok(PathResolution { item: PathResolutionItem::Trait(trait_id), errors }) => {
                self.push_errors(errors);
                return Some(trait_id);
            }
            Ok(_) => DefCollectorErrorKind::NotATrait { not_a_trait_name: path },
            Err(_) => DefCollectorErrorKind::TraitNotFound { trait_path: path },
        };
        self.push_err(error);
        None
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
        self.local_module = Some(trait_impl.module_id);

        self.generics = trait_impl.resolved_generics.clone();
        self.current_trait_impl = trait_impl.impl_id;

        self.add_trait_impl_assumed_trait_implementations(trait_impl.impl_id);
        self.check_trait_impl_where_clause_matches_trait_where_clause(&trait_impl);
        self.check_parent_traits_are_implemented(&trait_impl);
        self.remove_trait_impl_assumed_trait_implementations(trait_impl.impl_id);

        for (module, function, noir_function) in &trait_impl.methods.functions {
            self.local_module = Some(*module);
            let errors = check_trait_impl_method_matches_declaration(
                self.interner,
                *function,
                noir_function,
            );
            self.push_errors(errors);
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
        self.local_module = Some(alias.module_id);

        let name = &alias.type_alias_def.name;
        let visibility = alias.type_alias_def.visibility;
        let location = alias.type_alias_def.location;

        let generics = self.add_generics(&alias.type_alias_def.generics);
        self.current_item = Some(DependencyId::Alias(alias_id));
        let wildcard_allowed = types::WildcardAllowed::No(WildcardDisallowedContext::TypeAlias);
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

    /// True if we're currently within a constrained function or lambda.
    /// Defaults to `true` if the current function is unknown.
    fn in_constrained_function(&self) -> bool {
        if self.in_comptime_context() {
            return false;
        }

        let in_unconstrained_function = self.current_item.is_some_and(|id| {
            if let DependencyId::Function(id) = id {
                self.interner.function_modifiers(&id).is_unconstrained
            } else {
                false
            }
        });

        let in_unconstrained_lambda = self.lambda_stack.last().is_some_and(|ctx| ctx.unconstrained);

        !in_unconstrained_function && !in_unconstrained_lambda
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
