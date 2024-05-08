#![allow(unused)]
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{graph::CrateId, hir::{Context, resolution::path_resolver::StandardPathResolver, def_map::{ModuleId, LocalModuleId}, def_collector::dc_crate::{DefCollector, CollectedItems}}};
use crate::hir::def_map::CrateDefMap;
use crate::{
    ast::{
        ArrayLiteral, ConstructorExpression, FunctionKind, IfExpression, InfixExpression, Lambda,
        UnresolvedTraitConstraint, UnresolvedTypeExpression,
    },
    hir::{
        def_collector::dc_crate::CompilationError,
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
struct ResolverMeta {
    num_times_used: usize,
    ident: HirIdent,
    warn_if_unused: bool,
}

type ScopeForest = GenericScopeForest<String, ResolverMeta>;

pub struct Elaborator<'context> {
    scopes: ScopeForest,

    errors: Vec<(CompilationError, FileId)>,

    interner: &'context mut NodeInterner,

    def_maps: &'context BTreeMap<CrateId, CrateDefMap>,

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
            def_maps: &context.def_maps,
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
        }
    }

    pub fn elaborate(context: &'context mut Context, crate_id: CrateId, items: CollectedItems) -> Vec<(CompilationError, FileId)> {
        let mut this = Self::new(context, crate_id);

        // the resolver filters literal globals first
        for global in items.globals {

        }

        for alias in items.type_aliases {

        }

        for trait_ in items.traits {

        }

        for struct_ in items.types {

        }

        for trait_impl in &items.trait_impls {
            // only collect now
        }

        for impl_ in &items.impls {
            // only collect now
        }

        // resolver resolves non-literal globals here

        for functions in items.functions {
            this.file = functions.file_id;
            for (local_module, id, func) in functions.functions {
                this.local_module = local_module;
                this.elaborate_function(func, id);
            }
        }

        for impl_ in items.impls {

        }

        for trait_impl in items.trait_impls {

        }

        let cycle_errors = this.interner.check_for_dependency_cycles();
        this.errors.extend(cycle_errors);

        this.errors
    }

    fn elaborate_function(&mut self, function: NoirFunction, id: FuncId) {
        self.current_function = Some(id);
        // This is a stub until the elaborator is connected to dc_crate
        match function.kind {
            FunctionKind::LowLevel => todo!(),
            FunctionKind::Builtin => todo!(),
            FunctionKind::Oracle => todo!(),
            FunctionKind::Recursive => todo!(),
            FunctionKind::Normal => {
                let _body = self.elaborate_block(function.def.body);
            }
        }
        self.current_function = None;
    }

    fn push_err(&mut self, error: impl Into<CompilationError>) {
        self.errors.push((error.into(), self.file));
    }
}
