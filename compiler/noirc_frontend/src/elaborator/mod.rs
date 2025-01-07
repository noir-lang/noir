use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{
    ast::ItemVisibility, graph::CrateGraph, hir_def::traits::ResolvedTraitBound,
    node_interner::GlobalValue, usage_tracker::UsageTracker, StructField, StructType, TypeBindings,
};
use crate::{
    ast::{
        BlockExpression, FunctionKind, GenericTypeArgs, Ident, NoirFunction, NoirStruct, Param,
        Path, Pattern, TraitBound, UnresolvedGeneric, UnresolvedGenerics,
        UnresolvedTraitConstraint, UnresolvedTypeData, UnsupportedNumericGenericType,
    },
    graph::CrateId,
    hir::{
        def_collector::dc_crate::{
            filter_literal_globals, CompilationError, ImplMap, UnresolvedFunctions,
            UnresolvedGlobal, UnresolvedStruct, UnresolvedTraitImpl, UnresolvedTypeAlias,
        },
        def_collector::{dc_crate::CollectedItems, errors::DefCollectorErrorKind},
        def_map::{DefMaps, ModuleData},
        def_map::{LocalModuleId, ModuleId, MAIN_FUNCTION},
        resolution::errors::ResolverError,
        scope::ScopeForest as GenericScopeForest,
        type_check::{generics::TraitGenerics, TypeCheckError},
        Context,
    },
    hir_def::traits::TraitImpl,
    hir_def::{
        expr::{HirCapturedVar, HirIdent},
        function::{FuncMeta, FunctionBody, HirFunction},
        traits::TraitConstraint,
        types::{Generics, Kind, ResolvedGeneric},
    },
    node_interner::{
        DefinitionKind, DependencyId, ExprId, FuncId, FunctionModifiers, GlobalId, NodeInterner,
        ReferenceId, StructId, TraitId, TraitImplId, TypeAliasId,
    },
    token::SecondaryAttribute,
    Shared, Type, TypeVariable,
};

mod comptime;
mod expressions;
mod lints;
mod path_resolution;
mod patterns;
mod scope;
mod statements;
mod trait_impls;
mod traits;
pub mod types;
mod unquote;

use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Location, Span, Spanned};
use path_resolution::{PathResolution, PathResolutionItem};
use types::bind_ordered_generics;

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

/// Determines whether we are in an unsafe block and, if so, whether
/// any unconstrained calls were found in it (because if not we'll warn
/// that the unsafe block is not needed).
#[derive(Copy, Clone)]
enum UnsafeBlockStatus {
    NotInUnsafeBlock,
    InUnsafeBlockWithoutUnconstrainedCalls,
    InUnsafeBlockWithConstrainedCalls,
}

pub struct Elaborator<'context> {
    scopes: ScopeForest,

    pub(crate) errors: Vec<(CompilationError, FileId)>,

    pub(crate) interner: &'context mut NodeInterner,
    pub(crate) def_maps: &'context mut DefMaps,
    pub(crate) usage_tracker: &'context mut UsageTracker,
    pub(crate) crate_graph: &'context CrateGraph,

    pub(crate) file: FileId,

    unsafe_block_status: UnsafeBlockStatus,
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

    /// True if we're elaborating a comptime item such as a comptime function,
    /// block, global, or attribute.
    in_comptime_context: bool,

    crate_id: CrateId,

    /// The scope of --debug-comptime, or None if unset
    debug_comptime_in_file: Option<FileId>,

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

    /// Use pedantic ACVM solving
    pedantic_solving: bool,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        interner: &'context mut NodeInterner,
        def_maps: &'context mut DefMaps,
        usage_tracker: &'context mut UsageTracker,
        crate_graph: &'context CrateGraph,
        crate_id: CrateId,
        debug_comptime_in_file: Option<FileId>,
        interpreter_call_stack: im::Vector<Location>,
        pedantic_solving: bool,
    ) -> Self {
        Self {
            scopes: ScopeForest::default(),
            errors: Vec::new(),
            interner,
            def_maps,
            usage_tracker,
            crate_graph,
            file: FileId::dummy(),
            unsafe_block_status: UnsafeBlockStatus::NotInUnsafeBlock,
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
            current_trait: None,
            interpreter_call_stack,
            in_comptime_context: false,
            silence_field_visibility_errors: 0,
            pedantic_solving,
        }
    }

    pub fn from_context(
        context: &'context mut Context,
        crate_id: CrateId,
        debug_comptime_in_file: Option<FileId>,
        pedantic_solving: bool,
    ) -> Self {
        Self::new(
            &mut context.def_interner,
            &mut context.def_maps,
            &mut context.usage_tracker,
            &context.crate_graph,
            crate_id,
            debug_comptime_in_file,
            im::Vector::new(),
            pedantic_solving,
        )
    }

    pub fn elaborate(
        context: &'context mut Context,
        crate_id: CrateId,
        items: CollectedItems,
        debug_comptime_in_file: Option<FileId>,
        pedantic_solving: bool,
    ) -> Vec<(CompilationError, FileId)> {
        Self::elaborate_and_return_self(
            context,
            crate_id,
            items,
            debug_comptime_in_file,
            pedantic_solving,
        )
        .errors
    }

    pub fn elaborate_and_return_self(
        context: &'context mut Context,
        crate_id: CrateId,
        items: CollectedItems,
        debug_comptime_in_file: Option<FileId>,
        pedantic_solving: bool,
    ) -> Self {
        let mut this =
            Self::from_context(context, crate_id, debug_comptime_in_file, pedantic_solving);
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
        self.run_attributes(
            &items.traits,
            &items.types,
            &items.functions,
            &items.module_attributes,
        );

        for functions in items.functions {
            self.elaborate_functions(functions);
        }

        for (trait_id, unresolved_trait) in items.traits {
            self.current_trait = Some(trait_id);
            self.elaborate_functions(unresolved_trait.fns_with_default_impl);
        }
        self.current_trait = None;

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
            if let Kind::Numeric(typ) = &generic.kind() {
                let definition =
                    DefinitionKind::NumericGeneric(generic.type_var.clone(), typ.clone());
                let ident = Ident::new(generic.name.to_string(), generic.span);
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

        let modifiers = self.interner.function_modifiers(&id).clone();

        self.run_function_lints(&func_meta, &modifiers);

        // Check arg and return-value visibility of standalone functions.
        if self.should_check_function_visibility(&func_meta, &modifiers) {
            let name = Ident(Spanned::from(
                func_meta.name.location.span,
                self.interner.definition_name(func_meta.name.id).to_string(),
            ));
            for (_, typ, _) in func_meta.parameters.iter() {
                self.check_type_is_not_more_private_then_item(
                    &name,
                    modifiers.visibility,
                    typ,
                    name.span(),
                );
            }
            self.check_type_is_not_more_private_then_item(
                &name,
                modifiers.visibility,
                func_meta.return_type(),
                name.span(),
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

        self.add_trait_constraints_to_scope(&func_meta);

        let (hir_func, body_type) = match kind {
            FunctionKind::Builtin
            | FunctionKind::LowLevel
            | FunctionKind::Oracle
            | FunctionKind::TraitFunctionWithoutBody => (HirFunction::empty(), Type::Error),
            FunctionKind::Normal => {
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

        self.remove_trait_constraints_from_scope(&func_meta);

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
                    func_meta.name.location.span,
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

    /// Defaults all type variables used in this function context then solves
    /// all still-unsolved trait constraints in this context.
    fn check_and_pop_function_context(&mut self) {
        let context = self.function_context.pop().expect("Imbalanced function_context pushes");

        for typ in context.type_variables {
            if let Type::TypeVariable(variable) = typ.follow_bindings() {
                let msg = "TypeChecker should only track defaultable type vars";
                variable.bind(variable.kind().default_type().expect(msg));
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
                constraint.trait_bound.trait_id,
                &constraint.trait_bound.trait_generics.ordered,
                &constraint.trait_bound.trait_generics.named,
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

        let new_generic = TypeVariable::unbound(new_generic_id, Kind::Normal);
        generics.push(new_generic.clone());

        let name = format!("impl {trait_path}");
        let generic_type = Type::NamedGeneric(new_generic, Rc::new(name));
        let trait_bound = TraitBound { trait_path, trait_id: None, trait_generics };

        if let Some(trait_bound) = self.resolve_trait_bound(&trait_bound) {
            let new_constraint = TraitConstraint { typ: generic_type.clone(), trait_bound };
            trait_constraints.push(new_constraint);
        }

        generic_type
    }

    /// Add the given generics to scope.
    /// Each generic will have a fresh Shared<TypeBinding> associated with it.
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

            let span = generic.span();
            let name_owned = name.as_ref().clone();
            let resolved_generic = ResolvedGeneric { name, type_var, span };

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
    ) -> Result<(TypeVariable, Rc<String>), ResolverError> {
        // Map the generic to a fresh type variable
        match generic {
            UnresolvedGeneric::Variable(_) | UnresolvedGeneric::Numeric { .. } => {
                let id = self.interner.next_type_variable_id();
                let kind = self.resolve_generic_kind(generic);
                let typevar = TypeVariable::unbound(id, kind);
                let ident = generic.ident();
                let name = Rc::new(ident.0.contents.clone());
                Ok((typevar, name))
            }
            // An already-resolved generic is only possible if it is the result of a
            // previous macro call being inserted into a generics list.
            UnresolvedGeneric::Resolved(id, span) => {
                match self.interner.get_quoted_type(*id).follow_bindings() {
                    Type::NamedGeneric(type_variable, name) => Ok((type_variable.clone(), name)),
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
            let unresolved_typ = typ.clone();
            let typ = if unresolved_typ.is_type_expression() {
                self.resolve_type_inner(
                    unresolved_typ.clone(),
                    &Kind::numeric(Type::default_int_type()),
                )
            } else {
                self.resolve_type(unresolved_typ.clone())
            };
            if !matches!(typ, Type::FieldElement | Type::Integer(_, _)) {
                let unsupported_typ_err =
                    ResolverError::UnsupportedNumericGenericType(UnsupportedNumericGenericType {
                        ident: ident.clone(),
                        typ: unresolved_typ.typ.clone(),
                    });
                self.push_err(unsupported_typ_err);
            }
            Kind::numeric(typ)
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

    pub fn resolve_module_by_path(&mut self, path: Path) -> Option<ModuleId> {
        match self.resolve_path(path.clone()) {
            Ok(PathResolution { item: PathResolutionItem::Module(module_id), errors }) => {
                if errors.is_empty() {
                    Some(module_id)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn resolve_trait_by_path(&mut self, path: Path) -> Option<TraitId> {
        let error = match self.resolve_path(path.clone()) {
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
        let trait_bound = self.resolve_trait_bound(&constraint.trait_bound)?;
        Some(TraitConstraint { typ, trait_bound })
    }

    pub fn resolve_trait_bound(&mut self, bound: &TraitBound) -> Option<ResolvedTraitBound> {
        let the_trait = self.lookup_trait_or_error(bound.trait_path.clone())?;
        let trait_id = the_trait.id;
        let span = bound.trait_path.span;

        let (ordered, named) = self.resolve_type_args(bound.trait_generics.clone(), trait_id, span);

        let trait_generics = TraitGenerics { ordered, named };
        Some(ResolvedTraitBound { trait_id, trait_generics, span })
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
                // Function parameters have Kind::Normal
                _ => self.resolve_type_inner(typ, &Kind::Normal),
            };

            self.check_if_type_is_valid_for_program_input(
                &typ,
                is_entry_point,
                has_inline_attribute,
                type_span,
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

    fn mark_type_as_used(&mut self, typ: &Type) {
        match typ {
            Type::Array(_n, typ) => self.mark_type_as_used(typ),
            Type::Slice(typ) => self.mark_type_as_used(typ),
            Type::Tuple(types) => {
                for typ in types {
                    self.mark_type_as_used(typ);
                }
            }
            Type::Struct(struct_type, generics) => {
                self.mark_struct_as_constructed(struct_type.clone());
                for generic in generics {
                    self.mark_type_as_used(generic);
                }
                for (_, typ) in struct_type.borrow().get_fields(generics) {
                    self.mark_type_as_used(&typ);
                }
            }
            Type::Alias(alias_type, generics) => {
                self.mark_type_as_used(&alias_type.borrow().get_type(generics));
            }
            Type::CheckedCast { from, to } => {
                self.mark_type_as_used(from);
                self.mark_type_as_used(to);
            }
            Type::MutableReference(typ) => {
                self.mark_type_as_used(typ);
            }
            Type::InfixExpr(left, _op, right) => {
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
            lints::low_level_function_outside_stdlib(func, modifiers, elaborator.crate_id)
                .map(Into::into)
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

    fn add_trait_constraints_to_scope(&mut self, func_meta: &FuncMeta) {
        for constraint in &func_meta.trait_constraints {
            self.add_trait_bound_to_scope(
                func_meta,
                &constraint.typ,
                &constraint.trait_bound,
                constraint.trait_bound.trait_id,
            );
        }

        // Also assume `self` implements the current trait if we are inside a trait definition
        if let Some(trait_id) = self.current_trait {
            let the_trait = self.interner.get_trait(trait_id);
            let constraint = the_trait.as_constraint(the_trait.name.span());
            let self_type =
                self.self_type.clone().expect("Expected a self type if there's a current trait");
            self.add_trait_bound_to_scope(
                func_meta,
                &self_type,
                &constraint.trait_bound,
                constraint.trait_bound.trait_id,
            );
        }
    }

    fn remove_trait_constraints_from_scope(&mut self, func_meta: &FuncMeta) {
        for constraint in &func_meta.trait_constraints {
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
        func_meta: &FuncMeta,
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
                let span = func_meta.location.span;
                self.push_err(TypeCheckError::UnneededTraitConstraint { trait_name, typ, span });
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
                    func_meta,
                    object,
                    &parent_trait_bound,
                    starting_trait_id,
                );
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

        self.generics = trait_impl.resolved_generics.clone();
        self.current_trait_impl = trait_impl.impl_id;

        self.add_trait_impl_assumed_trait_implementations(trait_impl.impl_id);
        self.check_trait_impl_where_clause_matches_trait_where_clause(&trait_impl);
        self.check_parent_traits_are_implemented(&trait_impl);
        self.remove_trait_impl_assumed_trait_implementations(trait_impl.impl_id);

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

    fn add_trait_impl_assumed_trait_implementations(&mut self, impl_id: Option<TraitImplId>) {
        if let Some(impl_id) = impl_id {
            if let Some(trait_implementation) = self.interner.try_get_trait_implementation(impl_id)
            {
                for trait_constrain in &trait_implementation.borrow().where_clause {
                    let trait_bound = &trait_constrain.trait_bound;
                    self.interner.add_assumed_trait_implementation(
                        trait_constrain.typ.clone(),
                        trait_bound.trait_id,
                        trait_bound.trait_generics.clone(),
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
        let the_trait_file = the_trait.location.file;

        let mut bindings = TypeBindings::new();
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
                    span: trait_impl.object_type.span,
                    missing_trait_location: Location::new(trait_bound.span, the_trait_file),
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
        let the_trait_file = the_trait.location.file;

        let mut bindings = TypeBindings::new();
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
                    span: trait_impl.object_type.span,
                    missing_trait_location: Location::new(parent_trait_bound.span, the_trait_file),
                });
            }
        }
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
            self.declare_methods_on_struct(None, unresolved, *span);
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
            self.declare_methods_on_struct(Some(trait_id), &mut trait_impl.methods, span);

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
                where_clause,
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
        trait_id: Option<TraitId>,
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
            if trait_id.is_none() && struct_ref.id.krate() != self.crate_id {
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

        let name = &alias.type_alias_def.name;
        let visibility = alias.type_alias_def.visibility;
        let span = alias.type_alias_def.typ.span;

        let generics = self.add_generics(&alias.type_alias_def.generics);
        self.current_item = Some(DependencyId::Alias(alias_id));
        let typ = self.resolve_type(alias.type_alias_def.typ);

        if visibility != ItemVisibility::Private {
            self.check_type_is_not_more_private_then_item(name, visibility, &typ, span);
        }

        self.interner.set_type_alias(alias_id, typ, generics);
        self.generics.clear();
    }

    /// Find the struct in the parent module so we can know its visibility
    fn find_struct_visibility(&self, struct_type: &StructType) -> Option<ItemVisibility> {
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
        if let Some(struct_visibility) = func_meta.struct_id.and_then(|id| {
            let struct_def = self.get_struct(id);
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
        span: Span,
    ) {
        match typ {
            Type::Struct(struct_type, generics) => {
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
                                span,
                            });
                        }
                    }
                }

                for generic in generics {
                    self.check_type_is_not_more_private_then_item(name, visibility, generic, span);
                }
            }
            Type::Tuple(types) => {
                for typ in types {
                    self.check_type_is_not_more_private_then_item(name, visibility, typ, span);
                }
            }
            Type::Alias(alias_type, generics) => {
                self.check_type_is_not_more_private_then_item(
                    name,
                    visibility,
                    &alias_type.borrow().get_type(generics),
                    span,
                );
            }
            Type::CheckedCast { from, to } => {
                self.check_type_is_not_more_private_then_item(name, visibility, from, span);
                self.check_type_is_not_more_private_then_item(name, visibility, to, span);
            }
            Type::Function(args, return_type, env, _) => {
                for arg in args {
                    self.check_type_is_not_more_private_then_item(name, visibility, arg, span);
                }
                self.check_type_is_not_more_private_then_item(name, visibility, return_type, span);
                self.check_type_is_not_more_private_then_item(name, visibility, env, span);
            }
            Type::MutableReference(typ) | Type::Array(_, typ) | Type::Slice(typ) => {
                self.check_type_is_not_more_private_then_item(name, visibility, typ, span);
            }
            Type::InfixExpr(left, _op, right) => {
                self.check_type_is_not_more_private_then_item(name, visibility, left, span);
                self.check_type_is_not_more_private_then_item(name, visibility, right, span);
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

            if typ.struct_def.is_abi() {
                for field in &fields {
                    self.mark_type_as_used(&field.typ);
                }
            }

            // Check that the a public struct doesn't have a private type as a public field.
            if typ.struct_def.visibility != ItemVisibility::Private {
                for field in &fields {
                    let ident = Ident(Spanned::from(
                        field.name.span(),
                        format!("{}::{}", typ.struct_def.name, field.name),
                    ));
                    self.check_type_is_not_more_private_then_item(
                        &ident,
                        field.visibility,
                        &field.typ,
                        field.name.span(),
                    );
                }
            }

            let fields_len = fields.len();
            self.interner.update_struct(*type_id, |struct_def| {
                struct_def.set_fields(fields);
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
    ) -> Vec<StructField> {
        self.recover_generics(|this| {
            this.current_item = Some(DependencyId::Struct(struct_id));

            this.resolving_ids.insert(struct_id);

            let struct_def = this.interner.get_struct(struct_id);
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

        let (let_statement, _typ) = self
            .elaborate_in_comptime_context(|this| this.elaborate_let(let_stmt, Some(global_id)));

        let statement_id = self.interner.get_global(global_id).let_statement;
        self.interner.replace_statement(statement_id, let_statement);

        self.elaborate_comptime_global(global_id);

        if let Some(name) = name {
            self.interner.register_global(global_id, name, global.visibility, self.module_id());
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
