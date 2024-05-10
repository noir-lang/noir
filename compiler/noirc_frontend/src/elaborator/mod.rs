#![allow(unused)]
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{
    ast::{
        ArrayLiteral, ConstructorExpression, FunctionKind, IfExpression, InfixExpression, Lambda,
        UnresolvedTraitConstraint, UnresolvedTypeExpression,
    },
    hir::{
        def_collector::{dc_crate::CompilationError, errors::DuplicateType},
        resolution::{errors::ResolverError, path_resolver::PathResolver, resolver::LambdaContext},
        scope::ScopeForest as GenericScopeForest,
        type_check::TypeCheckError,
    },
    hir_def::{
        expr::{
            HirArrayLiteral, HirBinaryOp, HirBlockExpression, HirCallExpression, HirCastExpression,
            HirConstructorExpression, HirIdent, HirIfExpression, HirIndexExpression,
            HirInfixExpression, HirLambda, HirMemberAccess, HirMethodCallExpression,
            HirMethodReference, HirPrefixExpression,
        },
        traits::TraitConstraint,
    },
    macros_api::{
        BlockExpression, CallExpression, CastExpression, Expression, ExpressionKind, HirExpression,
        HirLiteral, HirStatement, Ident, IndexExpression, Literal, MemberAccessExpression,
        MethodCallExpression, NodeInterner, NoirFunction, PrefixExpression, Statement,
        StatementKind, StructId,
    },
    node_interner::{DefinitionKind, DependencyId, ExprId, FuncId, StmtId, TraitId},
    Shared, StructType, Type, TypeVariable,
};
use crate::{
    ast::{TraitBound, UnresolvedGenerics},
    graph::CrateId,
    hir::{
        def_collector::{
            dc_crate::{CollectedItems, DefCollector},
            errors::DefCollectorErrorKind,
        },
        def_map::{LocalModuleId, ModuleDefId, ModuleId, MAIN_FUNCTION},
        resolution::{
            errors::PubPosition,
            import::{PathResolution, PathResolutionError},
            path_resolver::StandardPathResolver,
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
mod types;

use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Location, Span};
use regex::Regex;
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

        // the resolver filters literal globals first
        for global in items.globals {}

        for alias in items.type_aliases {}

        for trait_ in items.traits {}

        for struct_ in items.types {}

        for trait_impl in &mut items.trait_impls {
            this.collect_trait_impl(trait_impl);
        }

        for ((typ, module), impls) in &items.impls {
            this.collect_impls(typ, *module, impls);
        }

        // resolver resolves non-literal globals here

        for functions in items.functions {
            this.elaborate_functions(functions);
        }

        for ((typ, module), impls) in items.impls {
            this.elaborate_impls(typ, module, impls);
        }

        for trait_impl in items.trait_impls {
            this.elaborate_trait_impl(trait_impl);
        }

        let cycle_errors = this.interner.check_for_dependency_cycles();
        this.errors.extend(cycle_errors);

        this.errors
    }

    fn elaborate_functions(&mut self, functions: UnresolvedFunctions) {
        self.file = functions.file_id;
        self.trait_id = functions.trait_id; // TODO: Resolve?
        for (local_module, id, func) in functions.functions {
            self.local_module = local_module;
            let generics_count = self.generics.len();
            self.elaborate_function(func, id);
            self.generics.truncate(generics_count);
        }
    }

    fn elaborate_function(&mut self, mut function: NoirFunction, id: FuncId) {
        self.current_function = Some(id);
        self.resolve_where_clause(&mut function.def.where_clause);

        // Without this, impl methods can accidentally be placed in contracts. See #3254
        if self.self_type.is_some() {
            self.in_contract = false;
        }

        self.scopes.start_function();
        self.current_item = Some(DependencyId::Function(id));

        // Check whether the function has globals in the local module and add them to the scope
        self.resolve_local_globals();
        self.add_generics(&function.def.generics);

        self.desugar_impl_trait_args(&mut function, id);
        self.trait_bounds = function.def.where_clause.clone();

        let is_low_level_or_oracle = function
            .attributes()
            .function
            .as_ref()
            .map_or(false, |func| func.is_low_level() || func.is_oracle());

        if function.def.is_unconstrained {
            self.in_unconstrained_fn = true;
        }

        let func_meta = self.extract_meta(&function, id);

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

        if !func_meta.can_ignore_return_type() {
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
        for (constraint, expr_id) in std::mem::take(&mut self.trait_constraints) {
            let span = self.interner.expr_span(&expr_id);
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
        if !is_low_level_or_oracle {
            self.check_for_unused_variables_in_scope_tree(func_scope_tree);
        }

        self.trait_bounds.clear();

        self.interner.push_fn_meta(func_meta, id);
        self.interner.update_fn(id, hir_func);
        self.current_function = None;
    }

    /// This turns function parameters of the form:
    /// fn foo(x: impl Bar)
    ///
    /// into
    /// fn foo<T0_impl_Bar>(x: T0_impl_Bar) where T0_impl_Bar: Bar
    fn desugar_impl_trait_args(&mut self, func: &mut NoirFunction, func_id: FuncId) {
        let mut impl_trait_generics = HashSet::default();
        let mut counter: usize = 0;
        for parameter in func.def.parameters.iter_mut() {
            if let UnresolvedTypeData::TraitAsType(path, args) = &parameter.typ.typ {
                let mut new_generic_ident: Ident =
                    format!("T{}_impl_{}", func_id, path.as_string()).into();
                let mut new_generic_path = Path::from_ident(new_generic_ident.clone());
                while impl_trait_generics.contains(&new_generic_ident)
                    || self.lookup_generic_or_global_type(&new_generic_path).is_some()
                {
                    new_generic_ident =
                        format!("T{}_impl_{}_{}", func_id, path.as_string(), counter).into();
                    new_generic_path = Path::from_ident(new_generic_ident.clone());
                    counter += 1;
                }
                impl_trait_generics.insert(new_generic_ident.clone());

                let is_synthesized = true;
                let new_generic_type_data =
                    UnresolvedTypeData::Named(new_generic_path, vec![], is_synthesized);
                let new_generic_type =
                    UnresolvedType { typ: new_generic_type_data.clone(), span: None };
                let new_trait_bound = TraitBound {
                    trait_path: path.clone(),
                    trait_id: None,
                    trait_generics: args.to_vec(),
                };
                let new_trait_constraint = UnresolvedTraitConstraint {
                    typ: new_generic_type,
                    trait_bound: new_trait_bound,
                };

                parameter.typ.typ = new_generic_type_data;
                func.def.generics.push(new_generic_ident);
                func.def.where_clause.push(new_trait_constraint);
            }
        }
        self.add_generics(&impl_trait_generics.into_iter().collect());
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

    fn resolve_local_globals(&mut self) {
        let globals = vecmap(self.interner.get_all_globals(), |global| {
            (global.id, global.local_id, global.ident.clone())
        });
        for (id, local_module_id, name) in globals {
            if local_module_id == self.local_module {
                let definition = DefinitionKind::Global(id);
                self.add_global_variable_decl(name, definition);
            }
        }
    }

    /// TODO: This is currently only respected for generic free functions
    /// there's a bunch of other places where trait constraints can pop up
    fn resolve_trait_constraints(
        &mut self,
        where_clause: &[UnresolvedTraitConstraint],
    ) -> Vec<TraitConstraint> {
        where_clause
            .iter()
            .cloned()
            .filter_map(|constraint| self.resolve_trait_constraint(constraint))
            .collect()
    }

    pub fn resolve_trait_constraint(
        &mut self,
        constraint: UnresolvedTraitConstraint,
    ) -> Option<TraitConstraint> {
        let typ = self.resolve_type(constraint.typ);
        let trait_generics =
            vecmap(constraint.trait_bound.trait_generics, |typ| self.resolve_type(typ));

        let span = constraint.trait_bound.trait_path.span();
        let the_trait = self.lookup_trait_or_error(constraint.trait_bound.trait_path)?;
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
    /// Prerequisite: self.add_generics() has already been called with the given
    /// function's generics, including any generics from the impl, if any.
    fn extract_meta(&mut self, func: &NoirFunction, func_id: FuncId) -> FuncMeta {
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

        let mut generics = vecmap(&self.generics, |(_, typevar, _)| typevar.clone());
        let mut parameters = vec![];
        let mut parameter_types = vec![];

        for Param { visibility, pattern, typ, span: _ } in func.parameters().iter().cloned() {
            if visibility == Visibility::Public && !self.pub_allowed(func) {
                self.push_err(ResolverError::UnnecessaryPub {
                    ident: func.name_ident().clone(),
                    position: PubPosition::Parameter,
                });
            }

            let type_span = typ.span.unwrap_or_else(|| pattern.span());
            let typ = self.resolve_type_inner(typ, &mut generics);
            self.check_if_type_is_valid_for_program_input(
                &typ,
                is_entry_point,
                has_inline_attribute,
                type_span,
            );
            let pattern = self.elaborate_pattern(pattern, typ.clone(), DefinitionKind::Local(None));

            parameters.push((pattern, typ.clone(), visibility));
            parameter_types.push(typ);
        }

        let return_type = Box::new(self.resolve_type(func.return_type()));

        self.declare_numeric_generics(&parameter_types, &return_type);

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

        FuncMeta {
            name: name_ident,
            kind: func.kind,
            location,
            typ,
            direct_generics,
            trait_impl: self.current_trait_impl,
            parameters: parameters.into(),
            return_type: func.def.return_type.clone(),
            return_visibility: func.def.return_visibility,
            has_body: !func.def.body.is_empty(),
            trait_constraints: self.resolve_trait_constraints(&func.def.where_clause),
            is_entry_point,
            has_inline_attribute,
        }
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

    fn declare_numeric_generics(&mut self, params: &[Type], return_type: &Type) {
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
        parameters: &[Type],
        return_type: &Type,
    ) -> Vec<(String, TypeVariable)> {
        let mut found = BTreeMap::new();
        for parameter in parameters {
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

    fn elaborate_impls(
        &mut self,
        typ: UnresolvedType,
        module: LocalModuleId,
        impls: Vec<(Vec<Ident>, Span, UnresolvedFunctions)>,
    ) {
        self.generics.clear();

        for (generics, _, functions) in impls {
            self.file = functions.file_id;
            self.add_generics(&generics);
            let self_type = self.resolve_type(typ.clone());
            self.self_type = Some(self_type.clone());

            let function_ids = vecmap(&functions.functions, |(_, id, _)| *id);
            self.elaborate_functions(functions);

            if self_type != Type::Error {
                for method_id in function_ids {
                    let method_name = self.interner.function_name(&method_id).to_owned();

                    if let Some(first_fn) =
                        self.interner.add_method(&self_type, method_name.clone(), method_id, false)
                    {
                        let error = ResolverError::DuplicateDefinition {
                            name: method_name,
                            first_span: self.interner.function_ident(&first_fn).span(),
                            second_span: self.interner.function_ident(&method_id).span(),
                        };
                        self.push_err(error);
                    }
                }
            }
        }
    }

    fn elaborate_trait_impl(&mut self, trait_impl: UnresolvedTraitImpl) {
        self.file = trait_impl.file_id;
        self.local_module = trait_impl.module_id;

        let unresolved_type = trait_impl.object_type;
        let self_type_span = unresolved_type.span;
        self.add_generics(&trait_impl.generics);

        let trait_generics =
            vecmap(&trait_impl.trait_generics, |generic| self.resolve_type(generic.clone()));

        let self_type = self.resolve_type(unresolved_type.clone());
        let impl_id = self.interner.next_trait_impl_id();

        self.self_type = Some(self_type.clone());
        self.current_trait_impl = Some(impl_id);

        let mut methods = trait_impl.methods.function_ids();

        self.elaborate_functions(trait_impl.methods);

        if matches!(self_type, Type::MutableReference(_)) {
            let span = self_type_span.unwrap_or_else(|| trait_impl.trait_path.span());
            self.push_err(DefCollectorErrorKind::MutableReferenceInTraitImpl { span });
        }

        if let Some(trait_id) = trait_impl.trait_id {
            for func_id in &methods {
                self.interner.set_function_trait(*func_id, self_type.clone(), trait_id);
            }

            let where_clause = trait_impl
                .where_clause
                .into_iter()
                .flat_map(|item| self.resolve_trait_constraint(item))
                .collect();

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
                impl_id,
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

        self.self_type = None;
        self.current_trait_impl = None;
        self.generics.clear();
    }

    fn collect_impls(
        &mut self,
        self_type: &UnresolvedType,
        module: LocalModuleId,
        impls: &[(Vec<Ident>, Span, UnresolvedFunctions)],
    ) {
        self.local_module = module;

        for (generics, span, unresolved) in impls {
            self.file = unresolved.file_id;
            self.declare_method_on_struct(self_type, generics, false, unresolved, *span);
        }
    }

    fn collect_trait_impl(&mut self, trait_impl: &mut UnresolvedTraitImpl) {
        self.local_module = trait_impl.module_id;
        self.file = trait_impl.file_id;
        trait_impl.trait_id = self.resolve_trait_by_path(trait_impl.trait_path.clone());

        if let Some(trait_id) = trait_impl.trait_id {
            self.collect_trait_impl_methods(trait_id, trait_impl);

            let span = trait_impl.object_type.span.expect("All trait self types should have spans");
            let object_type = &trait_impl.object_type;
            let generics = &trait_impl.generics;
            self.declare_method_on_struct(object_type, generics, true, &trait_impl.methods, span);
        }
    }

    fn get_module_mut(&mut self, module: ModuleId) -> &mut ModuleData {
        let message = "A crate should always be present for a given crate id";
        &mut self.def_maps.get_mut(&module.krate).expect(message).modules[module.local_id.0]
    }

    fn declare_method_on_struct(
        &mut self,
        self_type: &UnresolvedType,
        generics: &UnresolvedGenerics,
        is_trait_impl: bool,
        functions: &UnresolvedFunctions,
        span: Span,
    ) {
        let generic_count = self.generics.len();
        self.add_generics(generics);
        let typ = self.resolve_type(self_type.clone());

        if let Type::Struct(struct_type, _generics) = typ {
            let struct_type = struct_type.borrow();

            // `impl`s are only allowed on types defined within the current crate
            if !is_trait_impl && struct_type.id.krate() != self.crate_id {
                let type_name = struct_type.name.to_string();
                self.push_err(DefCollectorErrorKind::ForeignImpl { span, type_name });
                self.generics.truncate(generic_count);
                return;
            }

            // Grab the module defined by the struct type. Note that impls are a case
            // where the module the methods are added to is not the same as the module
            // they are resolved in.
            let module = self.get_module_mut(struct_type.id.module_id());

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
        // Prohibit defining impls for primitive types if we're not in the stdlib
        } else if !is_trait_impl && typ != Type::Error && !self.crate_id.is_stdlib() {
            self.push_err(DefCollectorErrorKind::NonStructTypeInImpl { span });
        }
        self.generics.truncate(generic_count);
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

        let object_crate = match self.resolve_type(trait_impl.object_type.clone()) {
            Type::Struct(struct_type, _) => struct_type.borrow().id.krate(),
            _ => CrateId::Dummy,
        };

        let the_trait = self.interner.get_trait(trait_id);
        if self.crate_id != the_trait.crate_id && self.crate_id != object_crate {
            self.push_err(DefCollectorErrorKind::TraitImplOrphaned {
                span: trait_impl.object_type.span.expect("object type must have a span"),
            });
        }
    }
}
