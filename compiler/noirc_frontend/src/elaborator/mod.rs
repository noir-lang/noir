use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{
    ast::{FunctionKind, UnresolvedTraitConstraint},
    hir::{
        def_collector::{
            dc_crate::{
                filter_literal_globals, CompilationError, ImplMap, UnresolvedGlobal,
                UnresolvedStruct, UnresolvedTypeAlias,
            },
            errors::DuplicateType,
        },
        resolution::{errors::ResolverError, path_resolver::PathResolver, resolver::LambdaContext},
        scope::ScopeForest as GenericScopeForest,
        type_check::{check_trait_impl_method_matches_declaration, TypeCheckError},
    },
    hir_def::{expr::HirIdent, function::Parameters, traits::TraitConstraint},
    macros_api::{
        Ident, NodeInterner, NoirFunction, NoirStruct, Pattern, SecondaryAttribute, StructId,
    },
    node_interner::{DefinitionKind, DependencyId, ExprId, FuncId, TraitId, TypeAliasId},
    Shared, Type, TypeVariable,
};
use crate::{
    ast::{TraitBound, UnresolvedGenerics},
    graph::CrateId,
    hir::{
        def_collector::{dc_crate::CollectedItems, errors::DefCollectorErrorKind},
        def_map::{LocalModuleId, ModuleDefId, ModuleId, MAIN_FUNCTION},
        resolution::{
            errors::PubPosition, import::PathResolution, path_resolver::StandardPathResolver,
        },
        Context,
    },
    hir_def::function::{FuncMeta, HirFunction},
    macros_api::{Param, Path, UnresolvedType, UnresolvedTypeData, Visibility},
    node_interner::TraitImplId,
    token::FunctionAttribute,
    Generics,
};
use crate::{
    hir::{
        def_collector::dc_crate::{UnresolvedFunctions, UnresolvedTraitImpl},
        def_map::{CrateDefMap, ModuleData},
    },
    hir_def::traits::TraitImpl,
    macros_api::ItemVisibility,
};

mod expressions;
mod patterns;
mod scope;
mod statements;
mod traits;
mod types;

use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Location, Span};
use rustc_hash::FxHashSet as HashSet;

/// ResolverMetas are tagged onto each definition to track how many times they are used
#[derive(Debug, PartialEq, Eq)]
pub struct ResolverMeta {
    num_times_used: usize,
    ident: HirIdent,
    warn_if_unused: bool,
}

type ScopeForest = GenericScopeForest<String, ResolverMeta>;

pub struct Elaborator<'context> {
    scopes: ScopeForest,

    errors: Vec<(CompilationError, FileId)>,

    interner: &'context mut NodeInterner,

    def_maps: &'context mut BTreeMap<CrateId, CrateDefMap>,

    file: FileId,

    in_unconstrained_fn: bool,
    nested_loops: usize,

    /// True if the current module is a contract.
    /// This is usually determined by self.path_resolver.module_id(), but it can
    /// be overridden for impls. Impls are an odd case since the methods within resolve
    /// as if they're in the parent module, but should be placed in a child module.
    /// Since they should be within a child module, in_contract is manually set to false
    /// for these so we can still resolve them in the parent module without them being in a contract.
    in_contract: bool,

    /// Contains a mapping of the current struct or functions's generics to
    /// unique type variables if we're resolving a struct. Empty otherwise.
    /// This is a Vec rather than a map to preserve the order a functions generics
    /// were declared in.
    generics: Vec<(Rc<String>, TypeVariable, Span)>,

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

    trait_id: Option<TraitId>,

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

    trait_bounds: Vec<UnresolvedTraitConstraint>,

    current_function: Option<FuncId>,

    /// All type variables created in the current function.
    /// This map is used to default any integer type variables at the end of
    /// a function (before checking trait constraints) if a type wasn't already chosen.
    type_variables: Vec<Type>,

    /// Trait constraints are collected during type checking until they are
    /// verified at the end of a function. This is because constraints arise
    /// on each variable, but it is only until function calls when the types
    /// needed for the trait constraint may become known.
    trait_constraints: Vec<(TraitConstraint, ExprId)>,

    /// The current module this elaborator is in.
    /// Initially empty, it is set whenever a new top-level item is resolved.
    local_module: LocalModuleId,

    crate_id: CrateId,
}

impl<'context> Elaborator<'context> {
    pub fn new(context: &'context mut Context, crate_id: CrateId) -> Self {
        Self {
            scopes: ScopeForest::default(),
            errors: Vec::new(),
            interner: &mut context.def_interner,
            def_maps: &mut context.def_maps,
            file: FileId::dummy(),
            in_unconstrained_fn: false,
            nested_loops: 0,
            in_contract: false,
            generics: Vec::new(),
            lambda_stack: Vec::new(),
            self_type: None,
            current_item: None,
            trait_id: None,
            local_module: LocalModuleId::dummy_id(),
            crate_id,
            resolving_ids: BTreeSet::new(),
            trait_bounds: Vec::new(),
            current_function: None,
            type_variables: Vec::new(),
            trait_constraints: Vec::new(),
            current_trait_impl: None,
        }
    }

    pub fn elaborate(
        context: &'context mut Context,
        crate_id: CrateId,
        mut items: CollectedItems,
    ) -> Vec<(CompilationError, FileId)> {
        let mut this = Self::new(context, crate_id);

        // We must first resolve and intern the globals before we can resolve any stmts inside each function.
        // Each function uses its own resolver with a newly created ScopeForest, and must be resolved again to be within a function's scope
        //
        // Additionally, we must resolve integer globals before structs since structs may refer to
        // the values of integer globals as numeric generics.
        let (literal_globals, non_literal_globals) = filter_literal_globals(items.globals);

        for global in literal_globals {
            this.elaborate_global(global);
        }

        for (alias_id, alias) in items.type_aliases {
            this.define_type_alias(alias_id, alias);
        }

        this.define_function_metas(&mut items.functions, &mut items.impls, &mut items.trait_impls);
        this.collect_traits(items.traits);

        // Must resolve structs before we resolve globals.
        this.collect_struct_definitions(items.types);

        // Bind trait impls to their trait. Collect trait functions, that have a
        // default implementation, which hasn't been overridden.
        for trait_impl in &mut items.trait_impls {
            this.collect_trait_impl(trait_impl);
        }

        // Before we resolve any function symbols we must go through our impls and
        // re-collect the methods within into their proper module. This cannot be
        // done during def collection since we need to be able to resolve the type of
        // the impl since that determines the module we should collect into.
        //
        // These are resolved after trait impls so that struct methods are chosen
        // over trait methods if there are name conflicts.
        for ((_self_type, module), impls) in &mut items.impls {
            this.collect_impls(*module, impls);
        }

        // We must wait to resolve non-literal globals until after we resolve structs since struct
        // globals will need to reference the struct type they're initialized to to ensure they are valid.
        for global in non_literal_globals {
            this.elaborate_global(global);
        }

        for functions in items.functions {
            this.elaborate_functions(functions);
        }

        for impls in items.impls.into_values() {
            this.elaborate_impls(impls);
        }

        for trait_impl in items.trait_impls {
            this.elaborate_trait_impl(trait_impl);
        }

        let cycle_errors = this.interner.check_for_dependency_cycles();
        this.errors.extend(cycle_errors);
        this.errors
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
        self.file = functions.file_id;
        self.trait_id = functions.trait_id; // TODO: Resolve?
        self.self_type = functions.self_type;

        for (local_module, id, func) in functions.functions {
            self.local_module = local_module;
            self.recover_generics(|this| this.elaborate_function(func, id));
        }

        self.self_type = None;
        self.trait_id = None;
    }

    fn elaborate_function(&mut self, function: NoirFunction, id: FuncId) {
        self.current_function = Some(id);

        // Without this, impl methods can accidentally be placed in contracts. See #3254
        if self.self_type.is_some() {
            self.in_contract = false;
        }

        self.scopes.start_function();
        self.current_item = Some(DependencyId::Function(id));

        self.trait_bounds = function.def.where_clause.clone();

        if function.def.is_unconstrained {
            self.in_unconstrained_fn = true;
        }

        let func_meta = self.interner.func_meta.get(&id);
        let func_meta = func_meta
            .expect("FuncMetas should be declared before a function is elaborated")
            .clone();

        // The DefinitionIds for each parameter were already created in define_function_meta
        // so we need to reintroduce the same IDs into scope here.
        for parameter in &func_meta.parameter_idents {
            let name = self.interner.definition_name(parameter.id).to_owned();
            self.add_existing_variable_to_scope(name, parameter.clone());
        }

        self.generics = func_meta.all_generics.clone();
        self.declare_numeric_generics(&func_meta.parameters, func_meta.return_type());
        self.add_trait_constraints_to_scope(&func_meta);

        let (hir_func, body_type) = match function.kind {
            FunctionKind::Builtin | FunctionKind::LowLevel | FunctionKind::Oracle => {
                (HirFunction::empty(), Type::Error)
            }
            FunctionKind::Normal | FunctionKind::Recursive => {
                let block_span = function.def.span;
                let (block, body_type) = self.elaborate_block(function.def.body);
                let expr_id = self.intern_expr(block, block_span);
                self.interner.push_expr_type(expr_id, body_type.clone());
                (HirFunction::unchecked_from_expr(expr_id), body_type)
            }
        };

        // Don't verify the return type for builtin functions & trait function declarations
        if !func_meta.is_stub() {
            self.type_check_function_body(body_type, &func_meta, hir_func.as_expr());
        }

        // Default any type variables that still need defaulting.
        // This is done before trait impl search since leaving them bindable can lead to errors
        // when multiple impls are available. Instead we default first to choose the Field or u64 impl.
        for typ in &self.type_variables {
            if let Type::TypeVariable(variable, kind) = typ.follow_bindings() {
                let msg = "TypeChecker should only track defaultable type vars";
                variable.bind(kind.default_type().expect(msg));
            }
        }

        // Verify any remaining trait constraints arising from the function body
        for (mut constraint, expr_id) in std::mem::take(&mut self.trait_constraints) {
            let span = self.interner.expr_span(&expr_id);

            if matches!(&constraint.typ, Type::MutableReference(_)) {
                let (_, dereferenced_typ) =
                    self.insert_auto_dereferences(expr_id, constraint.typ.clone());
                constraint.typ = dereferenced_typ;
            }

            self.verify_trait_constraint(
                &constraint.typ,
                constraint.trait_id,
                &constraint.trait_generics,
                expr_id,
                span,
            );
        }

        // Now remove all the `where` clause constraints we added
        for constraint in &func_meta.trait_constraints {
            self.interner.remove_assumed_trait_implementations_for_trait(constraint.trait_id);
        }

        let func_scope_tree = self.scopes.end_function();

        // The arguments to low-level and oracle functions are always unused so we do not produce warnings for them.
        if !func_meta.is_stub() {
            self.check_for_unused_variables_in_scope_tree(func_scope_tree);
        }

        self.trait_bounds.clear();

        self.interner.update_fn(id, hir_func);
        self.current_function = None;
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
        trait_generics: Vec<UnresolvedType>,
        generics: &mut Vec<TypeVariable>,
        trait_constraints: &mut Vec<TraitConstraint>,
    ) -> Type {
        let new_generic_id = self.interner.next_type_variable_id();
        let new_generic = TypeVariable::unbound(new_generic_id);
        generics.push(new_generic.clone());

        let name = format!("impl {trait_path}");
        let generic_type = Type::NamedGeneric(new_generic, Rc::new(name));
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
            // Map the generic to a fresh type variable
            let id = self.interner.next_type_variable_id();
            let typevar = TypeVariable::unbound(id);
            let span = generic.0.span();

            // Check for name collisions of this generic
            let name = Rc::new(generic.0.contents.clone());

            if let Some((_, _, first_span)) = self.find_generic(&name) {
                self.push_err(ResolverError::DuplicateDefinition {
                    name: generic.0.contents.clone(),
                    first_span: *first_span,
                    second_span: span,
                });
            } else {
                self.generics.push((name, typevar.clone(), span));
            }

            typevar
        })
    }

    fn push_err(&mut self, error: impl Into<CompilationError>) {
        self.errors.push((error.into(), self.file));
    }

    fn resolve_where_clause(&mut self, clause: &mut [UnresolvedTraitConstraint]) {
        for bound in clause {
            if let Some(trait_id) = self.resolve_trait_by_path(bound.trait_bound.trait_path.clone())
            {
                bound.trait_bound.trait_id = Some(trait_id);
            }
        }
    }

    fn resolve_trait_by_path(&mut self, path: Path) -> Option<TraitId> {
        let path_resolver = StandardPathResolver::new(self.module_id());

        let error = match path_resolver.resolve(self.def_maps, path.clone()) {
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

    fn resolve_trait_bound(&mut self, bound: &TraitBound, typ: Type) -> Option<TraitConstraint> {
        let trait_generics = vecmap(&bound.trait_generics, |typ| self.resolve_type(typ.clone()));

        let span = bound.trait_path.span();
        let the_trait = self.lookup_trait_or_error(bound.trait_path.clone())?;
        let trait_id = the_trait.id;

        let expected_generics = the_trait.generics.len();
        let actual_generics = trait_generics.len();

        if actual_generics != expected_generics {
            let item_name = the_trait.name.to_string();
            self.push_err(ResolverError::IncorrectGenericCount {
                span,
                item_name,
                actual: actual_generics,
                expected: expected_generics,
            });
        }

        Some(TraitConstraint { typ, trait_id, trait_generics })
    }

    /// Extract metadata from a NoirFunction
    /// to be used in analysis and intern the function parameters
    /// Prerequisite: any implicit generics, including any generics from the impl,
    /// have already been added to scope via `self.add_generics`.
    fn define_function_meta(
        &mut self,
        func: &mut NoirFunction,
        func_id: FuncId,
        is_trait_function: bool,
    ) {
        self.current_function = Some(func_id);
        self.resolve_where_clause(&mut func.def.where_clause);

        // Without this, impl methods can accidentally be placed in contracts. See #3254
        if self.self_type.is_some() {
            self.in_contract = false;
        }

        self.scopes.start_function();
        self.current_item = Some(DependencyId::Function(func_id));

        let location = Location::new(func.name_ident().span(), self.file);
        let id = self.interner.function_definition_id(func_id);
        let name_ident = HirIdent::non_trait_method(id, location);

        let attributes = func.attributes().clone();
        let has_no_predicates_attribute = attributes.is_no_predicates();
        let should_fold = attributes.is_foldable();
        if !self.inline_attribute_allowed(func) {
            if has_no_predicates_attribute {
                self.push_err(ResolverError::NoPredicatesAttributeOnUnconstrained {
                    ident: func.name_ident().clone(),
                });
            } else if should_fold {
                self.push_err(ResolverError::FoldAttributeOnUnconstrained {
                    ident: func.name_ident().clone(),
                });
            }
        }
        // Both the #[fold] and #[no_predicates] alter a function's inline type and code generation in similar ways.
        // In certain cases such as type checking (for which the following flag will be used) both attributes
        // indicate we should code generate in the same way. Thus, we unify the attributes into one flag here.
        let has_inline_attribute = has_no_predicates_attribute || should_fold;
        let is_entry_point = self.is_entry_point_function(func);

        self.add_generics(&func.def.generics);

        let mut trait_constraints = self.resolve_trait_constraints(&func.def.where_clause);

        let mut generics = vecmap(&self.generics, |(_, typevar, _)| typevar.clone());
        let mut parameters = Vec::new();
        let mut parameter_types = Vec::new();
        let mut parameter_idents = Vec::new();

        for Param { visibility, pattern, typ, span: _ } in func.parameters().iter().cloned() {
            if visibility == Visibility::Public && !self.pub_allowed(func) {
                self.push_err(ResolverError::UnnecessaryPub {
                    ident: func.name_ident().clone(),
                    position: PubPosition::Parameter,
                });
            }

            let type_span = typ.span.unwrap_or_else(|| pattern.span());

            let typ = match typ.typ {
                UnresolvedTypeData::TraitAsType(path, args) => {
                    self.desugar_impl_trait_arg(path, args, &mut generics, &mut trait_constraints)
                }
                _ => self.resolve_type_inner(typ),
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
            );

            parameters.push((pattern, typ.clone(), visibility));
            parameter_types.push(typ);
        }

        let return_type = Box::new(self.resolve_type(func.return_type()));

        if !self.pub_allowed(func) && func.def.return_visibility == Visibility::Public {
            self.push_err(ResolverError::UnnecessaryPub {
                ident: func.name_ident().clone(),
                position: PubPosition::ReturnType,
            });
        }

        let is_low_level_function =
            attributes.function.as_ref().map_or(false, |func| func.is_low_level());

        if !self.crate_id.is_stdlib() && is_low_level_function {
            let error =
                ResolverError::LowLevelFunctionOutsideOfStdlib { ident: func.name_ident().clone() };
            self.push_err(error);
        }

        // 'pub' is required on return types for entry point functions
        if is_entry_point
            && return_type.as_ref() != &Type::Unit
            && func.def.return_visibility == Visibility::Private
        {
            self.push_err(ResolverError::NecessaryPub { ident: func.name_ident().clone() });
        }
        // '#[recursive]' attribute is only allowed for entry point functions
        if !is_entry_point && func.kind == FunctionKind::Recursive {
            self.push_err(ResolverError::MisplacedRecursiveAttribute {
                ident: func.name_ident().clone(),
            });
        }

        if matches!(attributes.function, Some(FunctionAttribute::Test { .. }))
            && !parameters.is_empty()
        {
            self.push_err(ResolverError::TestFunctionHasParameters {
                span: func.name_ident().span(),
            });
        }

        let mut typ = Type::Function(parameter_types, return_type, Box::new(Type::Unit));

        if !generics.is_empty() {
            typ = Type::Forall(generics, Box::new(typ));
        }

        self.interner.push_definition_type(name_ident.id, typ.clone());

        let direct_generics = func.def.generics.iter();
        let direct_generics = direct_generics
            .filter_map(|generic| self.find_generic(&generic.0.contents))
            .map(|(name, typevar, _span)| (name.clone(), typevar.clone()))
            .collect();

        let meta = FuncMeta {
            name: name_ident,
            kind: func.kind,
            location,
            typ,
            direct_generics,
            all_generics: self.generics.clone(),
            trait_impl: self.current_trait_impl,
            parameters: parameters.into(),
            parameter_idents,
            return_type: func.def.return_type.clone(),
            return_visibility: func.def.return_visibility,
            has_body: !func.def.body.is_empty(),
            trait_constraints,
            is_entry_point,
            is_trait_function,
            has_inline_attribute,
        };

        self.interner.push_fn_meta(meta, func_id);
        self.current_function = None;
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

    fn inline_attribute_allowed(&self, func: &NoirFunction) -> bool {
        // Inline attributes are only relevant for constrained functions
        // as all unconstrained functions are not inlined
        !func.def.is_unconstrained
    }

    /// True if the 'pub' keyword is allowed on parameters in this function
    /// 'pub' on function parameters is only allowed for entry point functions
    fn pub_allowed(&self, func: &NoirFunction) -> bool {
        self.is_entry_point_function(func) || func.attributes().is_foldable()
    }

    fn is_entry_point_function(&self, func: &NoirFunction) -> bool {
        if self.in_contract {
            func.attributes().is_contract_entry_point()
        } else {
            func.name() == MAIN_FUNCTION
        }
    }

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
            if let Some((name, _, span)) =
                self.generics.iter().find(|(name, _, _)| name.as_ref() == &name_to_find)
            {
                let ident = Ident::new(name.to_string(), *span);
                let definition = DefinitionKind::GenericType(type_variable);
                self.add_variable_decl_inner(ident, false, false, false, definition);
            }
        }
    }

    fn find_numeric_generics(
        parameters: &Parameters,
        return_type: &Type,
    ) -> Vec<(String, TypeVariable)> {
        let mut found = BTreeMap::new();
        for (_, parameter, _) in &parameters.0 {
            Self::find_numeric_generics_in_type(parameter, &mut found);
        }
        Self::find_numeric_generics_in_type(return_type, &mut found);
        found.into_iter().collect()
    }

    fn find_numeric_generics_in_type(typ: &Type, found: &mut BTreeMap<String, TypeVariable>) {
        match typ {
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Unit
            | Type::Error
            | Type::TypeVariable(_, _)
            | Type::Constant(_)
            | Type::NamedGeneric(_, _)
            | Type::Code
            | Type::Forall(_, _) => (),

            Type::TraitAsType(_, _, args) => {
                for arg in args {
                    Self::find_numeric_generics_in_type(arg, found);
                }
            }

            Type::Array(length, element_type) => {
                if let Type::NamedGeneric(type_variable, name) = length.as_ref() {
                    found.insert(name.to_string(), type_variable.clone());
                }
                Self::find_numeric_generics_in_type(element_type, found);
            }

            Type::Slice(element_type) => {
                Self::find_numeric_generics_in_type(element_type, found);
            }

            Type::Tuple(fields) => {
                for field in fields {
                    Self::find_numeric_generics_in_type(field, found);
                }
            }

            Type::Function(parameters, return_type, _env) => {
                for parameter in parameters {
                    Self::find_numeric_generics_in_type(parameter, found);
                }
                Self::find_numeric_generics_in_type(return_type, found);
            }

            Type::Struct(struct_type, generics) => {
                for (i, generic) in generics.iter().enumerate() {
                    if let Type::NamedGeneric(type_variable, name) = generic {
                        if struct_type.borrow().generic_is_numeric(i) {
                            found.insert(name.to_string(), type_variable.clone());
                        }
                    } else {
                        Self::find_numeric_generics_in_type(generic, found);
                    }
                }
            }
            Type::Alias(alias, generics) => {
                for (i, generic) in generics.iter().enumerate() {
                    if let Type::NamedGeneric(type_variable, name) = generic {
                        if alias.borrow().generic_is_numeric(i) {
                            found.insert(name.to_string(), type_variable.clone());
                        }
                    } else {
                        Self::find_numeric_generics_in_type(generic, found);
                    }
                }
            }
            Type::MutableReference(element) => Self::find_numeric_generics_in_type(element, found),
            Type::String(length) => {
                if let Type::NamedGeneric(type_variable, name) = length.as_ref() {
                    found.insert(name.to_string(), type_variable.clone());
                }
            }
            Type::FmtString(length, fields) => {
                if let Type::NamedGeneric(type_variable, name) = length.as_ref() {
                    found.insert(name.to_string(), type_variable.clone());
                }
                Self::find_numeric_generics_in_type(fields, found);
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

    fn elaborate_impls(&mut self, impls: Vec<(Vec<Ident>, Span, UnresolvedFunctions)>) {
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
        impls: &mut [(Vec<Ident>, Span, UnresolvedFunctions)],
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
        trait_impl.trait_id = self.resolve_trait_by_path(trait_impl.trait_path.clone());

        let self_type = trait_impl.methods.self_type.clone();
        let self_type =
            self_type.expect("Expected struct type to be set before collect_trait_impl");

        self.self_type = Some(self_type.clone());
        let self_type_span = trait_impl.object_type.span;

        if matches!(self_type, Type::MutableReference(_)) {
            let span = self_type_span.unwrap_or_else(|| trait_impl.trait_path.span());
            self.push_err(DefCollectorErrorKind::MutableReferenceInTraitImpl { span });
        }

        if let Some(trait_id) = trait_impl.trait_id {
            self.generics = trait_impl.resolved_generics.clone();
            self.collect_trait_impl_methods(trait_id, trait_impl);

            let span = trait_impl.object_type.span.expect("All trait self types should have spans");
            self.declare_methods_on_struct(true, &mut trait_impl.methods, span);

            let methods = trait_impl.methods.function_ids();
            for func_id in &methods {
                self.interner.set_function_trait(*func_id, self_type.clone(), trait_id);
            }

            let where_clause = trait_impl
                .where_clause
                .iter()
                .flat_map(|item| self.resolve_trait_constraint(item))
                .collect();

            let trait_generics = trait_impl.resolved_trait_generics.clone();

            let resolved_trait_impl = Shared::new(TraitImpl {
                ident: trait_impl.trait_path.last_segment().clone(),
                typ: self_type.clone(),
                trait_id,
                trait_generics: trait_generics.clone(),
                file: trait_impl.file_id,
                where_clause,
                methods,
            });

            let generics = vecmap(&self.generics, |(_, type_variable, _)| type_variable.clone());

            if let Err((prev_span, prev_file)) = self.interner.add_trait_implementation(
                self_type.clone(),
                trait_id,
                trait_generics,
                trait_impl.impl_id.expect("impl_id should be set in define_function_metas"),
                generics,
                resolved_trait_impl,
            ) {
                self.push_err(DefCollectorErrorKind::OverlappingImpl {
                    typ: self_type.clone(),
                    span: self_type_span.unwrap_or_else(|| trait_impl.trait_path.span()),
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

    fn get_module_mut(&mut self, module: ModuleId) -> &mut ModuleData {
        let message = "A crate should always be present for a given crate id";
        &mut self.def_maps.get_mut(&module.krate).expect(message).modules[module.local_id.0]
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
            let module = self.get_module_mut(struct_ref.id.module_id());

            for (_, method_id, method) in &functions.functions {
                // If this method was already declared, remove it from the module so it cannot
                // be accessed with the `TypeName::method` syntax. We'll check later whether the
                // object types in each method overlap or not. If they do, we issue an error.
                // If not, that is specialization which is allowed.
                let name = method.name_ident().clone();
                if module.declare_function(name, ItemVisibility::Public, *method_id).is_err() {
                    module.remove_function(method.name_ident());
                }
            }

            self.declare_struct_methods(self_type, &function_ids);
        // We can define methods on primitive types only if we're in the stdlib
        } else if !is_trait_impl && *self_type != Type::Error {
            if self.crate_id.is_stdlib() {
                self.declare_struct_methods(self_type, &function_ids);
            } else {
                self.push_err(DefCollectorErrorKind::NonStructTypeInImpl { span });
            }
        }
    }

    fn declare_struct_methods(&mut self, self_type: &Type, function_ids: &[FuncId]) {
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

    fn collect_trait_impl_methods(
        &mut self,
        trait_id: TraitId,
        trait_impl: &mut UnresolvedTraitImpl,
    ) {
        self.local_module = trait_impl.module_id;
        self.file = trait_impl.file_id;

        // In this Vec methods[i] corresponds to trait.methods[i]. If the impl has no implementation
        // for a particular method, the default implementation will be added at that slot.
        let mut ordered_methods = Vec::new();

        // check whether the trait implementation is in the same crate as either the trait or the type
        self.check_trait_impl_crate_coherence(trait_id, trait_impl);

        // set of function ids that have a corresponding method in the trait
        let mut func_ids_in_trait = HashSet::default();

        // Temporarily take ownership of the trait's methods so we can iterate over them
        // while also mutating the interner
        let the_trait = self.interner.get_trait_mut(trait_id);
        let methods = std::mem::take(&mut the_trait.methods);

        for method in &methods {
            let overrides: Vec<_> = trait_impl
                .methods
                .functions
                .iter()
                .filter(|(_, _, f)| f.name() == method.name.0.contents)
                .collect();

            if overrides.is_empty() {
                if let Some(default_impl) = &method.default_impl {
                    // copy 'where' clause from unresolved trait impl
                    let mut default_impl_clone = default_impl.clone();
                    default_impl_clone.def.where_clause.extend(trait_impl.where_clause.clone());

                    let func_id = self.interner.push_empty_fn();
                    let module = self.module_id();
                    let location = Location::new(default_impl.def.span, trait_impl.file_id);
                    self.interner.push_function(func_id, &default_impl.def, module, location);
                    self.define_function_meta(&mut default_impl_clone, func_id, false);
                    func_ids_in_trait.insert(func_id);
                    ordered_methods.push((
                        method.default_impl_module_id,
                        func_id,
                        *default_impl_clone,
                    ));
                } else {
                    self.push_err(DefCollectorErrorKind::TraitMissingMethod {
                        trait_name: self.interner.get_trait(trait_id).name.clone(),
                        method_name: method.name.clone(),
                        trait_impl_span: trait_impl
                            .object_type
                            .span
                            .expect("type must have a span"),
                    });
                }
            } else {
                for (_, func_id, _) in &overrides {
                    func_ids_in_trait.insert(*func_id);
                }

                if overrides.len() > 1 {
                    self.push_err(DefCollectorErrorKind::Duplicate {
                        typ: DuplicateType::TraitAssociatedFunction,
                        first_def: overrides[0].2.name_ident().clone(),
                        second_def: overrides[1].2.name_ident().clone(),
                    });
                }

                ordered_methods.push(overrides[0].clone());
            }
        }

        // Restore the methods that were taken before the for loop
        let the_trait = self.interner.get_trait_mut(trait_id);
        the_trait.set_methods(methods);

        // Emit MethodNotInTrait error for methods in the impl block that
        // don't have a corresponding method signature defined in the trait
        for (_, func_id, func) in &trait_impl.methods.functions {
            if !func_ids_in_trait.contains(func_id) {
                let trait_name = the_trait.name.clone();
                let impl_method = func.name_ident().clone();
                let error = DefCollectorErrorKind::MethodNotInTrait { trait_name, impl_method };
                self.errors.push((error.into(), self.file));
            }
        }

        trait_impl.methods.functions = ordered_methods;
        trait_impl.methods.trait_id = Some(trait_id);
    }

    fn check_trait_impl_crate_coherence(
        &mut self,
        trait_id: TraitId,
        trait_impl: &UnresolvedTraitImpl,
    ) {
        self.local_module = trait_impl.module_id;
        self.file = trait_impl.file_id;

        let object_crate = match &trait_impl.resolved_object_type {
            Some(Type::Struct(struct_type, _)) => struct_type.borrow().id.krate(),
            _ => CrateId::Dummy,
        };

        let the_trait = self.interner.get_trait(trait_id);
        if self.crate_id != the_trait.crate_id && self.crate_id != object_crate {
            self.push_err(DefCollectorErrorKind::TraitImplOrphaned {
                span: trait_impl.object_type.span.expect("object type must have a span"),
            });
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

    fn collect_struct_definitions(&mut self, structs: BTreeMap<StructId, UnresolvedStruct>) {
        // This is necessary to avoid cloning the entire struct map
        // when adding checks after each struct field is resolved.
        let struct_ids = structs.keys().copied().collect::<Vec<_>>();

        // Resolve each field in each struct.
        // Each struct should already be present in the NodeInterner after def collection.
        for (type_id, typ) in structs {
            self.file = typ.file_id;
            self.local_module = typ.module_id;
            let (generics, fields) = self.resolve_struct_fields(typ.struct_def, type_id);

            self.interner.update_struct(type_id, |struct_def| {
                struct_def.set_fields(fields);
                struct_def.generics = generics;
            });
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
        unresolved: NoirStruct,
        struct_id: StructId,
    ) -> (Generics, Vec<(Ident, Type)>) {
        self.recover_generics(|this| {
            let generics = this.add_generics(&unresolved.generics);

            this.current_item = Some(DependencyId::Struct(struct_id));

            this.resolving_ids.insert(struct_id);
            let fields = vecmap(unresolved.fields, |(ident, typ)| (ident, this.resolve_type(typ)));
            this.resolving_ids.remove(&struct_id);

            (generics, fields)
        })
    }

    fn elaborate_global(&mut self, global: UnresolvedGlobal) {
        self.local_module = global.module_id;
        self.file = global.file_id;

        let global_id = global.global_id;
        self.current_item = Some(DependencyId::Global(global_id));
        let let_stmt = global.stmt_def;

        if !self.in_contract
            && let_stmt.attributes.iter().any(|attr| matches!(attr, SecondaryAttribute::Abi(_)))
        {
            let span = let_stmt.pattern.span();
            self.push_err(ResolverError::AbiAttributeOutsideContract { span });
        }

        if !let_stmt.comptime && matches!(let_stmt.pattern, Pattern::Mutable(..)) {
            let span = let_stmt.pattern.span();
            self.push_err(ResolverError::MutableGlobal { span });
        }

        self.elaborate_global_let(let_stmt, global_id);

        // Avoid defaulting the types of globals here since they may be used in any function.
        // Otherwise we may prematurely default to a Field inside the next function if this
        // global was unused there, even if it is consistently used as a u8 everywhere else.
        self.type_variables.clear();
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
                self.add_generics(generics);
                let self_type = self.resolve_type(self_type.clone());
                function_set.self_type = Some(self_type.clone());
                self.self_type = Some(self_type);
                self.define_function_metas_for_functions(function_set);
                self.generics.clear();
            }
        }

        for trait_impl in trait_impls {
            self.file = trait_impl.file_id;
            self.local_module = trait_impl.module_id;

            let unresolved_type = &trait_impl.object_type;
            self.add_generics(&trait_impl.generics);
            trait_impl.resolved_generics = self.generics.clone();

            let trait_generics =
                vecmap(&trait_impl.trait_generics, |generic| self.resolve_type(generic.clone()));
            trait_impl.resolved_trait_generics = trait_generics;

            let self_type = self.resolve_type(unresolved_type.clone());

            self.self_type = Some(self_type.clone());
            trait_impl.methods.self_type = Some(self_type);

            let impl_id = self.interner.next_trait_impl_id();
            self.current_trait_impl = Some(impl_id);

            self.define_function_metas_for_functions(&mut trait_impl.methods);

            trait_impl.resolved_object_type = self.self_type.take();
            trait_impl.impl_id = self.current_trait_impl.take();
            self.generics.clear();
        }
    }

    fn define_function_metas_for_functions(&mut self, function_set: &mut UnresolvedFunctions) {
        self.file = function_set.file_id;

        for (local_module, id, func) in &mut function_set.functions {
            self.local_module = *local_module;
            self.recover_generics(|this| {
                this.define_function_meta(func, *id, false);
            });
        }
    }
}
