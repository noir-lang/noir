// Fix usage of intern and resolve
// In some places, we do intern, however in others we are resolving and interning
// Ideally, I want to separate the interning and resolving abstractly
// so separate functions, but combine them naturally
// This could be possible, if lowering, is given a mutable map/scope as a parameter.
// So that it can match Idents to Ids. This is close to what the Scope map looks like
// Except for the num_times_used parameter.
// We can instead have a map from Ident to Into<IdentId> and implement that trait on ResolverMeta
//
//
// XXX: Change mentions of intern to resolve. In regards to the above comment
//
// XXX: Resolver does not check for unused functions
use acvm::acir::AcirField;

use crate::hir_def::expr::{
    HirArrayLiteral, HirBinaryOp, HirBlockExpression, HirCallExpression, HirCapturedVar,
    HirCastExpression, HirConstructorExpression, HirExpression, HirIdent, HirIfExpression,
    HirIndexExpression, HirInfixExpression, HirLambda, HirLiteral, HirMemberAccess,
    HirMethodCallExpression, HirPrefixExpression, ImplKind,
};

use crate::hir_def::traits::{Trait, TraitConstraint};
use crate::macros_api::SecondaryAttribute;
use crate::token::{Attributes, FunctionAttribute};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::rc::Rc;

use crate::ast::{
    ArrayLiteral, BinaryOpKind, BlockExpression, Expression, ExpressionKind, ForRange,
    FunctionDefinition, FunctionKind, FunctionReturnType, Ident, ItemVisibility, LValue,
    LetStatement, Literal, NoirFunction, NoirStruct, NoirTypeAlias, Param, Path, PathKind, Pattern,
    Statement, StatementKind, TraitBound, UnaryOp, UnresolvedGenerics, UnresolvedTraitConstraint,
    UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression, Visibility, ERROR_IDENT,
};
use crate::graph::CrateId;
use crate::hir::def_map::{ModuleDefId, TryFromModuleDefId, MAIN_FUNCTION};
use crate::hir::{def_map::CrateDefMap, resolution::path_resolver::PathResolver};
use crate::hir_def::stmt::{HirAssignStatement, HirForStatement, HirLValue, HirPattern};
use crate::node_interner::{
    DefinitionId, DefinitionKind, DependencyId, ExprId, FuncId, GlobalId, NodeInterner, StmtId,
    StructId, TraitId, TraitImplId, TraitMethodId, TypeAliasId,
};
use crate::{Generics, Shared, StructType, Type, TypeAlias, TypeVariable, TypeVariableKind};
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Location, Span, Spanned};

use crate::hir::scope::{
    Scope as GenericScope, ScopeForest as GenericScopeForest, ScopeTree as GenericScopeTree,
};
use crate::hir_def::{
    function::{FuncMeta, HirFunction},
    stmt::{HirConstrainStatement, HirLetStatement, HirStatement},
};

use super::errors::{PubPosition, ResolverError};
use super::import::PathResolution;

pub const SELF_TYPE_NAME: &str = "Self";

type Scope = GenericScope<String, ResolverMeta>;
type ScopeTree = GenericScopeTree<String, ResolverMeta>;
type ScopeForest = GenericScopeForest<String, ResolverMeta>;

pub struct LambdaContext {
    pub captures: Vec<HirCapturedVar>,
    /// the index in the scope tree
    /// (sometimes being filled by ScopeTree's find method)
    pub scope_index: usize,
}

/// The primary jobs of the Resolver are to validate that every variable found refers to exactly 1
/// definition in scope, and to convert the AST into the HIR.
///
/// A Resolver is a short-lived struct created to resolve a top-level definition.
/// One of these is created for each function definition and struct definition.
/// This isn't strictly necessary to its function, it could be refactored out in the future.
pub struct Resolver<'a> {
    scopes: ScopeForest,
    path_resolver: &'a dyn PathResolver,
    def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
    trait_id: Option<TraitId>,
    trait_bounds: Vec<UnresolvedTraitConstraint>,
    pub interner: &'a mut NodeInterner,
    errors: Vec<ResolverError>,
    file: FileId,

    /// Set to the current type if we're resolving an impl
    self_type: Option<Type>,

    /// If we're currently resolving methods within a trait impl, this will be set
    /// to the corresponding trait impl ID.
    current_trait_impl: Option<TraitImplId>,

    /// The current dependency item we're resolving.
    /// Used to link items to their dependencies in the dependency graph
    current_item: Option<DependencyId>,

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

    /// True if we're currently resolving an unconstrained function
    in_unconstrained_fn: bool,

    /// How many loops we're currently within.
    /// This increases by 1 at the start of a loop, and decreases by 1 when it ends.
    nested_loops: u32,
}

/// ResolverMetas are tagged onto each definition to track how many times they are used
#[derive(Debug, PartialEq, Eq)]
struct ResolverMeta {
    num_times_used: usize,
    ident: HirIdent,
    warn_if_unused: bool,
}

pub enum ResolvePathError {
    WrongKind,
    NotFound,
}

impl<'a> Resolver<'a> {
    pub fn new(
        interner: &'a mut NodeInterner,
        path_resolver: &'a dyn PathResolver,
        def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
        file: FileId,
    ) -> Resolver<'a> {
        let module_id = path_resolver.module_id();
        let in_contract = module_id.module(def_maps).is_contract;

        Self {
            path_resolver,
            def_maps,
            trait_id: None,
            trait_bounds: Vec::new(),
            scopes: ScopeForest::default(),
            interner,
            self_type: None,
            generics: Vec::new(),
            errors: Vec::new(),
            lambda_stack: Vec::new(),
            current_trait_impl: None,
            current_item: None,
            resolving_ids: BTreeSet::new(),
            file,
            in_contract,
            in_unconstrained_fn: false,
            nested_loops: 0,
        }
    }

    pub fn set_self_type(&mut self, self_type: Option<Type>) {
        self.self_type = self_type;
    }

    pub fn set_trait_id(&mut self, trait_id: Option<TraitId>) {
        self.trait_id = trait_id;
    }

    pub fn set_trait_impl_id(&mut self, impl_id: Option<TraitImplId>) {
        self.current_trait_impl = impl_id;
    }

    pub fn get_self_type(&mut self) -> Option<&Type> {
        self.self_type.as_ref()
    }

    fn push_err(&mut self, err: ResolverError) {
        self.errors.push(err);
    }

    /// This turns function parameters of the form:
    /// fn foo(x: impl Bar)
    ///
    /// into
    /// fn foo<T0_impl_Bar>(x: T0_impl_Bar) where T0_impl_Bar: Bar
    fn desugar_impl_trait_args(&mut self, func: &mut NoirFunction, func_id: FuncId) {
        let mut impl_trait_generics = HashSet::new();
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

    /// Resolving a function involves interning the metadata
    /// interning any statements inside of the function
    /// and interning the function itself
    /// We resolve and lower the function at the same time
    /// Since lowering would require scope data, unless we add an extra resolution field to the AST
    pub fn resolve_function(
        mut self,
        mut func: NoirFunction,
        func_id: FuncId,
    ) -> (HirFunction, FuncMeta, Vec<ResolverError>) {
        self.scopes.start_function();
        self.current_item = Some(DependencyId::Function(func_id));

        // Check whether the function has globals in the local module and add them to the scope
        self.resolve_local_globals();
        self.add_generics(&func.def.generics);

        self.desugar_impl_trait_args(&mut func, func_id);
        self.trait_bounds = func.def.where_clause.clone();

        let is_low_level_or_oracle = func
            .attributes()
            .function
            .as_ref()
            .map_or(false, |func| func.is_low_level() || func.is_oracle());
        let (hir_func, func_meta) = self.intern_function(func, func_id);
        let func_scope_tree = self.scopes.end_function();

        // The arguments to low-level and oracle functions are always unused so we do not produce warnings for them.
        if !is_low_level_or_oracle {
            self.check_for_unused_variables_in_scope_tree(func_scope_tree);
        }

        self.trait_bounds.clear();
        (hir_func, func_meta, self.errors)
    }

    pub fn resolve_trait_function(
        &mut self,
        name: &Ident,
        generics: &UnresolvedGenerics,
        parameters: &[(Ident, UnresolvedType)],
        return_type: &FunctionReturnType,
        where_clause: &[UnresolvedTraitConstraint],
        func_id: FuncId,
    ) -> (HirFunction, FuncMeta) {
        self.scopes.start_function();

        // Check whether the function has globals in the local module and add them to the scope
        self.resolve_local_globals();

        self.trait_bounds = where_clause.to_vec();

        let kind = FunctionKind::Normal;
        let def = FunctionDefinition {
            name: name.clone(),
            attributes: Attributes::empty(),
            is_unconstrained: false,
            is_comptime: false,
            visibility: ItemVisibility::Public, // Trait functions are always public
            generics: generics.clone(),
            parameters: vecmap(parameters, |(name, typ)| Param {
                visibility: Visibility::Private,
                pattern: Pattern::Identifier(name.clone()),
                typ: typ.clone(),
                span: name.span(),
            }),
            body: BlockExpression { statements: Vec::new() },
            span: name.span(),
            where_clause: where_clause.to_vec(),
            return_type: return_type.clone(),
            return_visibility: Visibility::Private,
        };

        let (hir_func, func_meta) = self.intern_function(NoirFunction { kind, def }, func_id);
        let _ = self.scopes.end_function();
        // Don't check the scope tree for unused variables, they can't be used in a declaration anyway.
        self.trait_bounds.clear();
        (hir_func, func_meta)
    }

    fn check_for_unused_variables_in_scope_tree(&mut self, scope_decls: ScopeTree) {
        let mut unused_vars = Vec::new();
        for scope in scope_decls.0.into_iter() {
            Resolver::check_for_unused_variables_in_local_scope(scope, &mut unused_vars);
        }

        for unused_var in unused_vars.iter() {
            if let Some(definition_info) = self.interner.try_definition(unused_var.id) {
                let name = &definition_info.name;
                if name != ERROR_IDENT && !definition_info.is_global() {
                    let ident = Ident(Spanned::from(unused_var.location.span, name.to_owned()));
                    self.push_err(ResolverError::UnusedVariable { ident });
                }
            }
        }
    }

    fn check_for_unused_variables_in_local_scope(decl_map: Scope, unused_vars: &mut Vec<HirIdent>) {
        let unused_variables = decl_map.filter(|(variable_name, metadata)| {
            let has_underscore_prefix = variable_name.starts_with('_'); // XXX: This is used for development mode, and will be removed
            metadata.warn_if_unused && metadata.num_times_used == 0 && !has_underscore_prefix
        });
        unused_vars.extend(unused_variables.map(|(_, meta)| meta.ident.clone()));
    }

    /// Run the given function in a new scope.
    fn in_new_scope<T, F: FnOnce(&mut Self) -> T>(&mut self, f: F) -> T {
        self.scopes.start_scope();
        let ret = f(self);
        let scope = self.scopes.end_scope();
        self.check_for_unused_variables_in_scope_tree(scope.into());
        ret
    }

    fn add_variable_decl(
        &mut self,
        name: Ident,
        mutable: bool,
        allow_shadowing: bool,
        definition: DefinitionKind,
    ) -> HirIdent {
        self.add_variable_decl_inner(name, mutable, allow_shadowing, true, definition)
    }

    fn add_variable_decl_inner(
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
        let id =
            self.interner.push_definition(name.0.contents.clone(), mutable, definition, location);
        let ident = HirIdent::non_trait_method(id, location);
        let resolver_meta =
            ResolverMeta { num_times_used: 0, ident: ident.clone(), warn_if_unused };

        let scope = self.scopes.get_mut_scope();
        let old_value = scope.add_key_value(name.0.contents.clone(), resolver_meta);

        if !allow_shadowing {
            if let Some(old_value) = old_value {
                self.push_err(ResolverError::DuplicateDefinition {
                    name: name.0.contents,
                    first_span: old_value.ident.location.span,
                    second_span: location.span,
                });
            }
        }

        ident
    }

    fn add_global_variable_decl(&mut self, name: Ident, definition: DefinitionKind) -> HirIdent {
        let scope = self.scopes.get_mut_scope();

        // This check is necessary to maintain the same definition ids in the interner. Currently, each function uses a new resolver that has its own ScopeForest and thus global scope.
        // We must first check whether an existing definition ID has been inserted as otherwise there will be multiple definitions for the same global statement.
        // This leads to an error in evaluation where the wrong definition ID is selected when evaluating a statement using the global. The check below prevents this error.
        let mut global_id = None;
        let global = self.interner.get_all_globals();
        for global_info in global {
            if global_info.ident == name
                && global_info.local_id == self.path_resolver.local_module_id()
            {
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
            let id =
                self.interner.push_definition(name.0.contents.clone(), false, definition, location);
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

    // Checks for a variable having been declared before
    // variable declaration and definition cannot be separate in Noir
    // Once the variable has been found, intern and link `name` to this definition
    // return the IdentId of `name`
    //
    // If a variable is not found, then an error is logged and a dummy id
    // is returned, for better error reporting UX
    fn find_variable_or_default(&mut self, name: &Ident) -> (HirIdent, usize) {
        self.find_variable(name).unwrap_or_else(|error| {
            self.push_err(error);
            let id = DefinitionId::dummy_id();
            let location = Location::new(name.span(), self.file);
            (HirIdent::non_trait_method(id, location), 0)
        })
    }

    fn find_variable(&mut self, name: &Ident) -> Result<(HirIdent, usize), ResolverError> {
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

    fn intern_function(&mut self, func: NoirFunction, id: FuncId) -> (HirFunction, FuncMeta) {
        let func_meta = self.extract_meta(&func, id);

        if func.def.is_unconstrained {
            self.in_unconstrained_fn = true;
        }

        let hir_func = match func.kind {
            FunctionKind::Builtin | FunctionKind::LowLevel | FunctionKind::Oracle => {
                HirFunction::empty()
            }
            FunctionKind::Normal | FunctionKind::Recursive => {
                let expr_id = self.intern_block(func.def.body);
                self.interner.push_expr_location(expr_id, func.def.span, self.file);
                HirFunction::unchecked_from_expr(expr_id)
            }
        };

        (hir_func, func_meta)
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

    /// Translates an UnresolvedType into a Type and appends any
    /// freshly created TypeVariables created to new_variables.
    fn resolve_type_inner(&mut self, typ: UnresolvedType) -> Type {
        use crate::ast::UnresolvedTypeData::*;

        let resolved_type = match typ.typ {
            FieldElement => Type::FieldElement,
            Array(size, elem) => {
                let elem = Box::new(self.resolve_type_inner(*elem));
                let size = self.convert_expression_type(size);
                Type::Array(Box::new(size), elem)
            }
            Slice(elem) => {
                let elem = Box::new(self.resolve_type_inner(*elem));
                Type::Slice(elem)
            }
            Expression(expr) => self.convert_expression_type(expr),
            Integer(sign, bits) => Type::Integer(sign, bits),
            Bool => Type::Bool,
            String(size) => {
                let resolved_size = self.convert_expression_type(size);
                Type::String(Box::new(resolved_size))
            }
            FormatString(size, fields) => {
                let resolved_size = self.convert_expression_type(size);
                let fields = self.resolve_type_inner(*fields);
                Type::FmtString(Box::new(resolved_size), Box::new(fields))
            }
            Code => Type::Code,
            Unit => Type::Unit,
            Unspecified => Type::Error,
            Error => Type::Error,
            Named(path, args, _) => self.resolve_named_type(path, args),
            TraitAsType(path, args) => self.resolve_trait_as_type(path, args),

            Tuple(fields) => Type::Tuple(vecmap(fields, |field| self.resolve_type_inner(field))),
            Function(args, ret, env) => {
                let args = vecmap(args, |arg| self.resolve_type_inner(arg));
                let ret = Box::new(self.resolve_type_inner(*ret));

                // expect() here is valid, because the only places we don't have a span are omitted types
                // e.g. a function without return type implicitly has a spanless UnresolvedType::Unit return type
                // To get an invalid env type, the user must explicitly specify the type, which will have a span
                let env_span =
                    env.span.expect("Unexpected missing span for closure environment type");

                let env = Box::new(self.resolve_type_inner(*env));

                match *env {
                    Type::Unit | Type::Tuple(_) | Type::NamedGeneric(_, _) => {
                        Type::Function(args, ret, env)
                    }
                    _ => {
                        self.push_err(ResolverError::InvalidClosureEnvironment {
                            typ: *env,
                            span: env_span,
                        });
                        Type::Error
                    }
                }
            }
            MutableReference(element) => {
                Type::MutableReference(Box::new(self.resolve_type_inner(*element)))
            }
            Parenthesized(typ) => self.resolve_type_inner(*typ),
        };

        if let Type::Struct(_, _) = resolved_type {
            if let Some(unresolved_span) = typ.span {
                // Record the location of the type reference
                self.interner.push_type_ref_location(
                    resolved_type.clone(),
                    Location::new(unresolved_span, self.file),
                );
            }
        }
        resolved_type
    }

    fn find_generic(&self, target_name: &str) -> Option<&(Rc<String>, TypeVariable, Span)> {
        self.generics.iter().find(|(name, _, _)| name.as_ref() == target_name)
    }

    fn resolve_named_type(&mut self, path: Path, args: Vec<UnresolvedType>) -> Type {
        if args.is_empty() {
            if let Some(typ) = self.lookup_generic_or_global_type(&path) {
                return typ;
            }
        }

        // Check if the path is a type variable first. We currently disallow generics on type
        // variables since we do not support higher-kinded types.
        if path.segments.len() == 1 {
            let name = &path.last_segment().0.contents;

            if name == SELF_TYPE_NAME {
                if let Some(self_type) = self.self_type.clone() {
                    if !args.is_empty() {
                        self.push_err(ResolverError::GenericsOnSelfType { span: path.span() });
                    }
                    return self_type;
                }
            }
        }

        let span = path.span();
        let mut args = vecmap(args, |arg| self.resolve_type_inner(arg));

        if let Some(type_alias) = self.lookup_type_alias(path.clone()) {
            let type_alias = type_alias.borrow();
            let expected_generic_count = type_alias.generics.len();
            let type_alias_string = type_alias.to_string();
            let id = type_alias.id;

            self.verify_generics_count(expected_generic_count, &mut args, span, || {
                type_alias_string
            });

            if let Some(item) = self.current_item {
                self.interner.add_type_alias_dependency(item, id);
            }

            // Collecting Type Alias references [Location]s to be used by LSP in order
            // to resolve the definition of the type alias
            self.interner.add_type_alias_ref(id, Location::new(span, self.file));

            // Because there is no ordering to when type aliases (and other globals) are resolved,
            // it is possible for one to refer to an Error type and issue no error if it is set
            // equal to another type alias. Fixing this fully requires an analysis to create a DFG
            // of definition ordering, but for now we have an explicit check here so that we at
            // least issue an error that the type was not found instead of silently passing.
            let alias = self.interner.get_type_alias(id);
            return Type::Alias(alias, args);
        }

        match self.lookup_struct_or_error(path) {
            Some(struct_type) => {
                if self.resolving_ids.contains(&struct_type.borrow().id) {
                    self.push_err(ResolverError::SelfReferentialStruct {
                        span: struct_type.borrow().name.span(),
                    });

                    return Type::Error;
                }

                let expected_generic_count = struct_type.borrow().generics.len();
                if !self.in_contract
                    && self
                        .interner
                        .struct_attributes(&struct_type.borrow().id)
                        .iter()
                        .any(|attr| matches!(attr, SecondaryAttribute::Abi(_)))
                {
                    self.push_err(ResolverError::AbiAttributeOutsideContract {
                        span: struct_type.borrow().name.span(),
                    });
                }
                self.verify_generics_count(expected_generic_count, &mut args, span, || {
                    struct_type.borrow().to_string()
                });

                if let Some(current_item) = self.current_item {
                    let dependency_id = struct_type.borrow().id;
                    self.interner.add_type_dependency(current_item, dependency_id);
                }

                Type::Struct(struct_type, args)
            }
            None => Type::Error,
        }
    }

    fn resolve_trait_as_type(&mut self, path: Path, args: Vec<UnresolvedType>) -> Type {
        let args = vecmap(args, |arg| self.resolve_type_inner(arg));

        if let Some(t) = self.lookup_trait_or_error(path) {
            Type::TraitAsType(t.id, Rc::new(t.name.to_string()), args)
        } else {
            Type::Error
        }
    }

    fn verify_generics_count(
        &mut self,
        expected_count: usize,
        args: &mut Vec<Type>,
        span: Span,
        type_name: impl FnOnce() -> String,
    ) {
        if args.len() != expected_count {
            self.errors.push(ResolverError::IncorrectGenericCount {
                span,
                item_name: type_name(),
                actual: args.len(),
                expected: expected_count,
            });

            // Fix the generic count so we can continue typechecking
            args.resize_with(expected_count, || Type::Error);
        }
    }

    fn lookup_generic_or_global_type(&mut self, path: &Path) -> Option<Type> {
        if path.segments.len() == 1 {
            let name = &path.last_segment().0.contents;
            if let Some((name, var, _)) = self.find_generic(name) {
                return Some(Type::NamedGeneric(var.clone(), name.clone()));
            }
        }

        // If we cannot find a local generic of the same name, try to look up a global
        match self.path_resolver.resolve(self.def_maps, path.clone()) {
            Ok(PathResolution { module_def_id: ModuleDefId::GlobalId(id), error }) => {
                if let Some(current_item) = self.current_item {
                    self.interner.add_global_dependency(current_item, id);
                }

                if let Some(error) = error {
                    self.push_err(error.into());
                }
                Some(Type::Constant(self.eval_global_as_array_length(id, path)))
            }
            _ => None,
        }
    }

    fn convert_expression_type(&mut self, length: UnresolvedTypeExpression) -> Type {
        match length {
            UnresolvedTypeExpression::Variable(path) => {
                self.lookup_generic_or_global_type(&path).unwrap_or_else(|| {
                    self.push_err(ResolverError::NoSuchNumericTypeVariable { path });
                    Type::Constant(0)
                })
            }
            UnresolvedTypeExpression::Constant(int, _) => Type::Constant(int),
            UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, _) => {
                let (lhs_span, rhs_span) = (lhs.span(), rhs.span());
                let lhs = self.convert_expression_type(*lhs);
                let rhs = self.convert_expression_type(*rhs);

                match (lhs, rhs) {
                    (Type::Constant(lhs), Type::Constant(rhs)) => {
                        Type::Constant(op.function()(lhs, rhs))
                    }
                    (lhs, _) => {
                        let span =
                            if !matches!(lhs, Type::Constant(_)) { lhs_span } else { rhs_span };
                        self.push_err(ResolverError::InvalidArrayLengthExpr { span });
                        Type::Constant(0)
                    }
                }
            }
        }
    }

    fn get_ident_from_path(&mut self, path: Path) -> (HirIdent, usize) {
        let location = Location::new(path.span(), self.file);

        let error = match path.as_ident().map(|ident| self.find_variable(ident)) {
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

    /// Translates an UnresolvedType to a Type
    pub fn resolve_type(&mut self, typ: UnresolvedType) -> Type {
        let span = typ.span;
        let resolved_type = self.resolve_type_inner(typ);
        if resolved_type.is_nested_slice() {
            self.errors.push(ResolverError::NestedSlices { span: span.unwrap() });
        }
        resolved_type
    }

    pub fn resolve_type_alias(
        mut self,
        unresolved: NoirTypeAlias,
        alias_id: TypeAliasId,
    ) -> (Type, Generics, Vec<ResolverError>) {
        let generics = self.add_generics(&unresolved.generics);
        self.resolve_local_globals();

        self.current_item = Some(DependencyId::Alias(alias_id));
        let typ = self.resolve_type(unresolved.typ);

        (typ, generics, self.errors)
    }

    pub fn take_errors(self) -> Vec<ResolverError> {
        self.errors
    }

    /// Return the current generics.
    /// Needed to keep referring to the same type variables across many
    /// methods in a single impl.
    pub fn get_generics(&self) -> &[(Rc<String>, TypeVariable, Span)] {
        &self.generics
    }

    /// Set the current generics that are in scope.
    /// Unlike add_generics, this function will not create any new type variables,
    /// opting to reuse the existing ones it is directly given.
    pub fn set_generics(&mut self, generics: Vec<(Rc<String>, TypeVariable, Span)>) {
        self.generics = generics;
    }

    /// Translates a (possibly Unspecified) UnresolvedType to a Type.
    /// Any UnresolvedType::Unspecified encountered are replaced with fresh type variables.
    fn resolve_inferred_type(&mut self, typ: UnresolvedType) -> Type {
        match &typ.typ {
            UnresolvedTypeData::Unspecified => self.interner.next_type_variable(),
            _ => self.resolve_type(typ),
        }
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
                self.errors.push(ResolverError::DuplicateDefinition {
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

    /// Add the given existing generics to scope.
    /// This is useful for adding the same generics to many items. E.g. apply impl generics
    /// to each function in the impl or trait generics to each item in the trait.
    pub fn add_existing_generics(&mut self, names: &UnresolvedGenerics, generics: &Generics) {
        assert_eq!(names.len(), generics.len());

        for (name, typevar) in names.iter().zip(generics) {
            self.add_existing_generic(&name.0.contents, name.0.span(), typevar.clone());
        }
    }

    pub fn add_existing_generic(&mut self, name: &str, span: Span, typevar: TypeVariable) {
        // Check for name collisions of this generic
        let rc_name = Rc::new(name.to_owned());

        if let Some((_, _, first_span)) = self.find_generic(&rc_name) {
            self.errors.push(ResolverError::DuplicateDefinition {
                name: name.to_owned(),
                first_span: *first_span,
                second_span: span,
            });
        } else {
            self.generics.push((rc_name, typevar, span));
        }
    }

    pub fn resolve_struct_fields(
        mut self,
        unresolved: NoirStruct,
        struct_id: StructId,
    ) -> (Generics, Vec<(Ident, Type)>, Vec<ResolverError>) {
        let generics = self.add_generics(&unresolved.generics);

        // Check whether the struct definition has globals in the local module and add them to the scope
        self.resolve_local_globals();

        self.current_item = Some(DependencyId::Struct(struct_id));

        self.resolving_ids.insert(struct_id);
        let fields = vecmap(unresolved.fields, |(ident, typ)| (ident, self.resolve_type(typ)));
        self.resolving_ids.remove(&struct_id);

        (generics, fields, self.errors)
    }

    fn resolve_local_globals(&mut self) {
        let globals = vecmap(self.interner.get_all_globals(), |global| {
            (global.id, global.local_id, global.ident.clone())
        });
        for (id, local_module_id, name) in globals {
            if local_module_id == self.path_resolver.local_module_id() {
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

        let generics = vecmap(&self.generics, |(_, typevar, _)| typevar.clone());
        let mut parameters = vec![];
        let mut parameter_types = vec![];

        for Param { visibility, pattern, typ, span: _ } in func.parameters().iter().cloned() {
            if visibility == Visibility::Public && !self.pub_allowed(func) {
                self.push_err(ResolverError::UnnecessaryPub {
                    ident: func.name_ident().clone(),
                    position: PubPosition::Parameter,
                });
            }

            let pattern = self.resolve_pattern(pattern, DefinitionKind::Local(None));
            let typ = self.resolve_type_inner(typ);

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
        if !self.path_resolver.module_id().krate.is_stdlib() && is_low_level_function {
            let error =
                ResolverError::LowLevelFunctionOutsideOfStdlib { ident: func.name_ident().clone() };
            self.push_err(error);
        }

        // 'pub' is required on return types for entry point functions
        if self.is_entry_point_function(func)
            && return_type.as_ref() != &Type::Unit
            && func.def.return_visibility == Visibility::Private
        {
            self.push_err(ResolverError::NecessaryPub { ident: func.name_ident().clone() });
        }
        // '#[recursive]' attribute is only allowed for entry point functions
        if !self.is_entry_point_function(func) && func.kind == FunctionKind::Recursive {
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
            is_entry_point: self.is_entry_point_function(func),
            has_inline_attribute,

            // This is only used by the elaborator
            all_generics: Vec::new(),
            is_trait_function: false,
            parameter_idents: Vec::new(),
        }
    }

    /// Override whether this name resolver is within a contract or not.
    /// This will affect which types are allowed as parameters to methods as well
    /// as which modifiers are allowed on a function.
    pub(crate) fn set_in_contract(&mut self, in_contract: bool) {
        self.in_contract = in_contract;
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

    fn inline_attribute_allowed(&self, func: &NoirFunction) -> bool {
        // Inline attributes are only relevant for constrained functions
        // as all unconstrained functions are not inlined
        !func.def.is_unconstrained
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

    pub fn resolve_global_let(
        &mut self,
        let_stmt: LetStatement,
        global_id: GlobalId,
    ) -> HirStatement {
        self.current_item = Some(DependencyId::Global(global_id));
        let expression = self.resolve_expression(let_stmt.expression);
        let definition = DefinitionKind::Global(global_id);

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

        HirStatement::Let(HirLetStatement {
            pattern: self.resolve_pattern(let_stmt.pattern, definition),
            r#type: self.resolve_type(let_stmt.r#type),
            expression,
            attributes: let_stmt.attributes,
            comptime: let_stmt.comptime,
        })
    }

    pub fn resolve_stmt(&mut self, stmt: StatementKind, span: Span) -> HirStatement {
        match stmt {
            StatementKind::Let(let_stmt) => {
                let expression = self.resolve_expression(let_stmt.expression);
                let definition = DefinitionKind::Local(Some(expression));
                HirStatement::Let(HirLetStatement {
                    pattern: self.resolve_pattern(let_stmt.pattern, definition),
                    r#type: self.resolve_type(let_stmt.r#type),
                    expression,
                    attributes: let_stmt.attributes,
                    comptime: let_stmt.comptime,
                })
            }
            StatementKind::Constrain(constrain_stmt) => {
                let expr_id = self.resolve_expression(constrain_stmt.0);
                let assert_message_expr_id =
                    constrain_stmt.1.map(|assert_expr_id| self.resolve_expression(assert_expr_id));

                HirStatement::Constrain(HirConstrainStatement(
                    expr_id,
                    self.file,
                    assert_message_expr_id,
                ))
            }
            StatementKind::Expression(expr) => {
                HirStatement::Expression(self.resolve_expression(expr))
            }
            StatementKind::Semi(expr) => HirStatement::Semi(self.resolve_expression(expr)),
            StatementKind::Assign(assign_stmt) => {
                let identifier = self.resolve_lvalue(assign_stmt.lvalue);
                let expression = self.resolve_expression(assign_stmt.expression);
                let stmt = HirAssignStatement { lvalue: identifier, expression };
                HirStatement::Assign(stmt)
            }
            StatementKind::For(for_loop) => {
                match for_loop.range {
                    ForRange::Range(start_range, end_range) => {
                        let start_range = self.resolve_expression(start_range);
                        let end_range = self.resolve_expression(end_range);
                        let (identifier, block) = (for_loop.identifier, for_loop.block);

                        self.nested_loops += 1;

                        // TODO: For loop variables are currently mutable by default since we haven't
                        //       yet implemented syntax for them to be optionally mutable.
                        let (identifier, block) = self.in_new_scope(|this| {
                            let decl = this.add_variable_decl(
                                identifier,
                                false,
                                true,
                                DefinitionKind::Local(None),
                            );
                            (decl, this.resolve_expression(block))
                        });

                        self.nested_loops -= 1;

                        HirStatement::For(HirForStatement {
                            start_range,
                            end_range,
                            block,
                            identifier,
                        })
                    }
                    range @ ForRange::Array(_) => {
                        let for_stmt =
                            range.into_for(for_loop.identifier, for_loop.block, for_loop.span);
                        self.resolve_stmt(for_stmt.kind, for_loop.span)
                    }
                }
            }
            StatementKind::Break => {
                self.check_break_continue(true, span);
                HirStatement::Break
            }
            StatementKind::Continue => {
                self.check_break_continue(false, span);
                HirStatement::Continue
            }
            StatementKind::Error => HirStatement::Error,
            StatementKind::Comptime(statement) => {
                let hir_statement = self.resolve_stmt(statement.kind, statement.span);
                let statement_id = self.interner.push_stmt(hir_statement);
                self.interner.push_stmt_location(statement_id, statement.span, self.file);
                HirStatement::Comptime(statement_id)
            }
        }
    }

    pub fn intern_stmt(&mut self, stmt: Statement) -> StmtId {
        let hir_stmt = self.resolve_stmt(stmt.kind, stmt.span);
        let id = self.interner.push_stmt(hir_stmt);
        self.interner.push_stmt_location(id, stmt.span, self.file);
        id
    }

    fn resolve_lvalue(&mut self, lvalue: LValue) -> HirLValue {
        match lvalue {
            LValue::Ident(ident) => {
                let ident = self.find_variable_or_default(&ident);
                self.resolve_local_variable(ident.0.clone(), ident.1);

                HirLValue::Ident(ident.0, Type::Error)
            }
            LValue::MemberAccess { object, field_name, span } => HirLValue::MemberAccess {
                object: Box::new(self.resolve_lvalue(*object)),
                field_name,
                location: Location::new(span, self.file),
                field_index: None,
                typ: Type::Error,
            },
            LValue::Index { array, index, span } => {
                let array = Box::new(self.resolve_lvalue(*array));
                let index = self.resolve_expression(index);
                let location = Location::new(span, self.file);
                HirLValue::Index { array, index, location, typ: Type::Error }
            }
            LValue::Dereference(lvalue, span) => {
                let lvalue = Box::new(self.resolve_lvalue(*lvalue));
                let location = Location::new(span, self.file);
                HirLValue::Dereference { lvalue, location, element_type: Type::Error }
            }
        }
    }

    fn resolve_local_variable(&mut self, hir_ident: HirIdent, var_scope_index: usize) {
        let mut transitive_capture_index: Option<usize> = None;

        for lambda_index in 0..self.lambda_stack.len() {
            if self.lambda_stack[lambda_index].scope_index > var_scope_index {
                // Beware: the same variable may be captured multiple times, so we check
                // for its presence before adding the capture below.
                let pos = self.lambda_stack[lambda_index]
                    .captures
                    .iter()
                    .position(|capture| capture.ident.id == hir_ident.id);

                if pos.is_none() {
                    self.lambda_stack[lambda_index].captures.push(HirCapturedVar {
                        ident: hir_ident.clone(),
                        transitive_capture_index,
                    });
                }

                if lambda_index + 1 < self.lambda_stack.len() {
                    // There is more than one closure between the current scope and
                    // the scope of the variable, so this is a propagated capture.
                    // We need to track the transitive capture index as we go up in
                    // the closure stack.
                    transitive_capture_index = Some(pos.unwrap_or(
                        // If this was a fresh capture, we added it to the end of
                        // the captures vector:
                        self.lambda_stack[lambda_index].captures.len() - 1,
                    ));
                }
            }
        }
    }

    fn resolve_array_literal(&mut self, array_literal: ArrayLiteral) -> HirArrayLiteral {
        match array_literal {
            ArrayLiteral::Standard(elements) => {
                let elements = vecmap(elements, |elem| self.resolve_expression(elem));
                HirArrayLiteral::Standard(elements)
            }
            ArrayLiteral::Repeated { repeated_element, length } => {
                let span = length.span;
                let length =
                    UnresolvedTypeExpression::from_expr(*length, span).unwrap_or_else(|error| {
                        self.errors.push(ResolverError::ParserError(Box::new(error)));
                        UnresolvedTypeExpression::Constant(0, span)
                    });

                let length = self.convert_expression_type(length);
                let repeated_element = self.resolve_expression(*repeated_element);

                HirArrayLiteral::Repeated { repeated_element, length }
            }
        }
    }

    pub fn resolve_expression(&mut self, expr: Expression) -> ExprId {
        let hir_expr = match expr.kind {
            ExpressionKind::Literal(literal) => HirExpression::Literal(match literal {
                Literal::Bool(b) => HirLiteral::Bool(b),
                Literal::Array(array_literal) => {
                    HirLiteral::Array(self.resolve_array_literal(array_literal))
                }
                Literal::Slice(array_literal) => {
                    HirLiteral::Slice(self.resolve_array_literal(array_literal))
                }
                Literal::Integer(integer, sign) => HirLiteral::Integer(integer, sign),
                Literal::Str(str) => HirLiteral::Str(str),
                Literal::RawStr(str, _) => HirLiteral::Str(str),
                Literal::FmtStr(str) => self.resolve_fmt_str_literal(str, expr.span),
                Literal::Unit => HirLiteral::Unit,
            }),
            ExpressionKind::Variable(path, generics) => {
                let generics =
                    generics.map(|generics| vecmap(generics, |typ| self.resolve_type(typ)));

                if let Some((method, constraint, assumed)) = self.resolve_trait_generic_path(&path)
                {
                    HirExpression::Ident(
                        HirIdent {
                            location: Location::new(expr.span, self.file),
                            id: self.interner.trait_method_id(method),
                            impl_kind: ImplKind::TraitMethod(method, constraint, assumed),
                        },
                        generics,
                    )
                } else {
                    // If the Path is being used as an Expression, then it is referring to a global from a separate module
                    // Otherwise, then it is referring to an Identifier
                    // This lookup allows support of such statements: let x = foo::bar::SOME_GLOBAL + 10;
                    // If the expression is a singular indent, we search the resolver's current scope as normal.
                    let (hir_ident, var_scope_index) = self.get_ident_from_path(path);

                    if hir_ident.id != DefinitionId::dummy_id() {
                        match self.interner.definition(hir_ident.id).kind {
                            DefinitionKind::Function(id) => {
                                if let Some(current_item) = self.current_item {
                                    self.interner.add_function_dependency(current_item, id);
                                }
                            }
                            DefinitionKind::Global(global_id) => {
                                if let Some(current_item) = self.current_item {
                                    self.interner.add_global_dependency(current_item, global_id);
                                }
                            }
                            DefinitionKind::GenericType(_) => {
                                // Initialize numeric generics to a polymorphic integer type in case
                                // they're used in expressions. We must do this here since the type
                                // checker does not check definition kinds and otherwise expects
                                // parameters to already be typed.
                                if self.interner.definition_type(hir_ident.id) == Type::Error {
                                    let typ = Type::polymorphic_integer_or_field(self.interner);
                                    self.interner.push_definition_type(hir_ident.id, typ);
                                }
                            }
                            DefinitionKind::Local(_) => {
                                // only local variables can be captured by closures.
                                self.resolve_local_variable(hir_ident.clone(), var_scope_index);
                            }
                        }
                    }

                    HirExpression::Ident(hir_ident, generics)
                }
            }
            ExpressionKind::Prefix(prefix) => {
                let operator = prefix.operator;
                let rhs = self.resolve_expression(prefix.rhs);

                if operator == UnaryOp::MutableReference {
                    if let Err(error) = verify_mutable_reference(self.interner, rhs) {
                        self.errors.push(error);
                    }
                }

                HirExpression::Prefix(HirPrefixExpression { operator, rhs })
            }
            ExpressionKind::Infix(infix) => {
                let lhs = self.resolve_expression(infix.lhs);
                let rhs = self.resolve_expression(infix.rhs);
                let trait_id = self.interner.get_operator_trait_method(infix.operator.contents);

                HirExpression::Infix(HirInfixExpression {
                    lhs,
                    operator: HirBinaryOp::new(infix.operator, self.file),
                    trait_method_id: trait_id,
                    rhs,
                })
            }
            ExpressionKind::Call(call_expr) => {
                // Get the span and name of path for error reporting
                let func = self.resolve_expression(*call_expr.func);

                let arguments = vecmap(call_expr.arguments, |arg| self.resolve_expression(arg));
                let location = Location::new(expr.span, self.file);
                HirExpression::Call(HirCallExpression { func, arguments, location })
            }
            ExpressionKind::MethodCall(call_expr) => {
                let method = call_expr.method_name;
                let object = self.resolve_expression(call_expr.object);

                // Cannot verify the generic count here equals the expected count since we don't
                // know which definition `method` refers to until it is resolved during type checking.
                let generics = call_expr
                    .generics
                    .map(|generics| vecmap(generics, |typ| self.resolve_type(typ)));

                let arguments = vecmap(call_expr.arguments, |arg| self.resolve_expression(arg));
                let location = Location::new(expr.span, self.file);
                HirExpression::MethodCall(HirMethodCallExpression {
                    method,
                    object,
                    generics,
                    arguments,
                    location,
                })
            }
            ExpressionKind::Cast(cast_expr) => HirExpression::Cast(HirCastExpression {
                lhs: self.resolve_expression(cast_expr.lhs),
                r#type: self.resolve_type(cast_expr.r#type),
            }),
            ExpressionKind::If(if_expr) => HirExpression::If(HirIfExpression {
                condition: self.resolve_expression(if_expr.condition),
                consequence: self.resolve_expression(if_expr.consequence),
                alternative: if_expr.alternative.map(|e| self.resolve_expression(e)),
            }),
            ExpressionKind::Index(indexed_expr) => HirExpression::Index(HirIndexExpression {
                collection: self.resolve_expression(indexed_expr.collection),
                index: self.resolve_expression(indexed_expr.index),
            }),
            ExpressionKind::Block(block_expr) => {
                HirExpression::Block(self.resolve_block(block_expr))
            }
            ExpressionKind::Constructor(constructor) => {
                let span = constructor.type_name.span();

                match self.lookup_type_or_error(constructor.type_name) {
                    Some(Type::Struct(r#type, struct_generics)) => {
                        let typ = r#type.clone();
                        let fields = constructor.fields;
                        let resolve_expr = Resolver::resolve_expression;
                        let fields =
                            self.resolve_constructor_fields(typ, fields, span, resolve_expr);
                        HirExpression::Constructor(HirConstructorExpression {
                            fields,
                            r#type,
                            struct_generics,
                        })
                    }
                    Some(typ) => {
                        self.push_err(ResolverError::NonStructUsedInConstructor { typ, span });
                        HirExpression::Error
                    }
                    None => HirExpression::Error,
                }
            }
            ExpressionKind::MemberAccess(access) => {
                // Validating whether the lhs actually has the rhs as a field
                // needs to wait until type checking when we know the type of the lhs
                HirExpression::MemberAccess(HirMemberAccess {
                    lhs: self.resolve_expression(access.lhs),
                    rhs: access.rhs,
                    // This is only used when lhs is a reference and we want to return a reference to rhs
                    is_offset: false,
                })
            }
            ExpressionKind::Error => HirExpression::Error,
            ExpressionKind::Tuple(elements) => {
                let elements = vecmap(elements, |elem| self.resolve_expression(elem));
                HirExpression::Tuple(elements)
            }
            // We must stay in the same function scope as the parent function to allow for closures
            // to capture variables. This is currently limited to immutable variables.
            ExpressionKind::Lambda(lambda) => self.in_new_scope(|this| {
                let scope_index = this.scopes.current_scope_index();

                this.lambda_stack.push(LambdaContext { captures: Vec::new(), scope_index });

                let parameters = vecmap(lambda.parameters, |(pattern, typ)| {
                    let parameter = DefinitionKind::Local(None);
                    (this.resolve_pattern(pattern, parameter), this.resolve_inferred_type(typ))
                });

                let return_type = this.resolve_inferred_type(lambda.return_type);
                let body = this.resolve_expression(lambda.body);

                let lambda_context = this.lambda_stack.pop().unwrap();

                HirExpression::Lambda(HirLambda {
                    parameters,
                    return_type,
                    body,
                    captures: lambda_context.captures,
                })
            }),
            ExpressionKind::Parenthesized(sub_expr) => return self.resolve_expression(*sub_expr),

            // The quoted expression isn't resolved since we don't want errors if variables aren't defined
            ExpressionKind::Quote(block) => HirExpression::Quote(block),
            ExpressionKind::Comptime(block) => HirExpression::Comptime(self.resolve_block(block)),
        };

        // If these lines are ever changed, make sure to change the early return
        // in the ExpressionKind::Variable case as well
        let expr_id = self.interner.push_expr(hir_expr);
        self.interner.push_expr_location(expr_id, expr.span, self.file);
        expr_id
    }

    fn resolve_pattern(&mut self, pattern: Pattern, definition: DefinitionKind) -> HirPattern {
        self.resolve_pattern_mutable(pattern, None, definition)
    }

    fn resolve_pattern_mutable(
        &mut self,
        pattern: Pattern,
        mutable: Option<Span>,
        definition: DefinitionKind,
    ) -> HirPattern {
        match pattern {
            Pattern::Identifier(name) => {
                // If this definition is mutable, do not store the rhs because it will
                // not always refer to the correct value of the variable
                let definition = match (mutable, definition) {
                    (Some(_), DefinitionKind::Local(_)) => DefinitionKind::Local(None),
                    (_, other) => other,
                };
                let id = self.add_variable_decl(name, mutable.is_some(), true, definition);
                HirPattern::Identifier(id)
            }
            Pattern::Mutable(pattern, span, _) => {
                if let Some(first_mut) = mutable {
                    self.push_err(ResolverError::UnnecessaryMut { first_mut, second_mut: span });
                }

                let pattern = self.resolve_pattern_mutable(*pattern, Some(span), definition);
                let location = Location::new(span, self.file);
                HirPattern::Mutable(Box::new(pattern), location)
            }
            Pattern::Tuple(fields, span) => {
                let fields = vecmap(fields, |field| {
                    self.resolve_pattern_mutable(field, mutable, definition.clone())
                });
                let location = Location::new(span, self.file);
                HirPattern::Tuple(fields, location)
            }
            Pattern::Struct(name, fields, span) => {
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

                let resolve_field = |this: &mut Self, pattern| {
                    this.resolve_pattern_mutable(pattern, mutable, definition.clone())
                };

                let typ = struct_type.clone();
                let fields = self.resolve_constructor_fields(typ, fields, span, resolve_field);

                let typ = Type::Struct(struct_type, generics);
                let location = Location::new(span, self.file);
                HirPattern::Struct(typ, fields, location)
            }
        }
    }

    /// Resolve all the fields of a struct constructor expression.
    /// Ensures all fields are present, none are repeated, and all
    /// are part of the struct.
    ///
    /// This is generic to allow it to work for constructor expressions
    /// and constructor patterns.
    fn resolve_constructor_fields<T, U>(
        &mut self,
        struct_type: Shared<StructType>,
        fields: Vec<(Ident, T)>,
        span: Span,
        mut resolve_function: impl FnMut(&mut Self, T) -> U,
    ) -> Vec<(Ident, U)> {
        let mut ret = Vec::with_capacity(fields.len());
        let mut seen_fields = HashSet::new();
        let mut unseen_fields = struct_type.borrow().field_names();

        for (field, expr) in fields {
            let resolved = resolve_function(self, expr);

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

    pub fn get_struct(&self, type_id: StructId) -> Shared<StructType> {
        self.interner.get_struct(type_id)
    }

    pub fn get_trait_mut(&mut self, trait_id: TraitId) -> &mut Trait {
        self.interner.get_trait_mut(trait_id)
    }

    fn lookup<T: TryFromModuleDefId>(&mut self, path: Path) -> Result<T, ResolverError> {
        let span = path.span();
        let id = self.resolve_path(path)?;
        T::try_from(id).ok_or_else(|| ResolverError::Expected {
            expected: T::description(),
            got: id.as_str().to_owned(),
            span,
        })
    }

    fn lookup_global(&mut self, path: Path) -> Result<DefinitionId, ResolverError> {
        let span = path.span();
        let id = self.resolve_path(path)?;

        if let Some(function) = TryFromModuleDefId::try_from(id) {
            return Ok(self.interner.function_definition_id(function));
        }

        if let Some(global) = TryFromModuleDefId::try_from(id) {
            let global = self.interner.get_global(global);
            return Ok(global.definition_id);
        }

        let expected = "global variable".into();
        let got = "local variable".into();
        Err(ResolverError::Expected { span, expected, got })
    }

    /// Lookup a given struct type by name.
    fn lookup_struct_or_error(&mut self, path: Path) -> Option<Shared<StructType>> {
        match self.lookup(path) {
            Ok(struct_id) => Some(self.get_struct(struct_id)),
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    /// Lookup a given trait by name/path.
    fn lookup_trait_or_error(&mut self, path: Path) -> Option<&mut Trait> {
        match self.lookup(path) {
            Ok(trait_id) => Some(self.get_trait_mut(trait_id)),
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    /// Looks up a given type by name.
    /// This will also instantiate any struct types found.
    fn lookup_type_or_error(&mut self, path: Path) -> Option<Type> {
        let ident = path.as_ident();
        if ident.map_or(false, |i| i == SELF_TYPE_NAME) {
            if let Some(typ) = &self.self_type {
                return Some(typ.clone());
            }
        }

        match self.lookup(path) {
            Ok(struct_id) => {
                let struct_type = self.get_struct(struct_id);
                let generics = struct_type.borrow().instantiate(self.interner);
                Some(Type::Struct(struct_type, generics))
            }
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    fn lookup_type_alias(&mut self, path: Path) -> Option<Shared<TypeAlias>> {
        self.lookup(path).ok().map(|id| self.interner.get_type_alias(id))
    }

    // this resolves Self::some_static_method, inside an impl block (where we don't have a concrete self_type)
    fn resolve_trait_static_method_by_self(
        &mut self,
        path: &Path,
    ) -> Option<(TraitMethodId, TraitConstraint, bool)> {
        let trait_id = self.trait_id?;

        if path.kind == PathKind::Plain && path.segments.len() == 2 {
            let name = &path.segments[0].0.contents;
            let method = &path.segments[1];

            if name == SELF_TYPE_NAME {
                let the_trait = self.interner.get_trait(trait_id);
                let method = the_trait.find_method(method.0.contents.as_str())?;

                let constraint = TraitConstraint {
                    typ: self.self_type.clone()?,
                    trait_generics: Type::from_generics(&the_trait.generics),
                    trait_id,
                };
                return Some((method, constraint, false));
            }
        }
        None
    }

    // this resolves TraitName::some_static_method
    fn resolve_trait_static_method(
        &mut self,
        path: &Path,
    ) -> Option<(TraitMethodId, TraitConstraint, bool)> {
        if path.kind == PathKind::Plain && path.segments.len() == 2 {
            let method = &path.segments[1];

            let mut trait_path = path.clone();
            trait_path.pop();
            let trait_id = self.lookup(trait_path).ok()?;
            let the_trait = self.interner.get_trait(trait_id);

            let method = the_trait.find_method(method.0.contents.as_str())?;
            let constraint = TraitConstraint {
                typ: Type::TypeVariable(
                    the_trait.self_type_typevar.clone(),
                    TypeVariableKind::Normal,
                ),
                trait_generics: Type::from_generics(&the_trait.generics),
                trait_id,
            };
            return Some((method, constraint, false));
        }
        None
    }

    // This resolves a static trait method T::trait_method by iterating over the where clause
    //
    // Returns the trait method, trait constraint, and whether the impl is assumed from a where
    // clause. This is always true since this helper searches where clauses for a generic constraint.
    // E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    fn resolve_trait_method_by_named_generic(
        &mut self,
        path: &Path,
    ) -> Option<(TraitMethodId, TraitConstraint, bool)> {
        if path.segments.len() != 2 {
            return None;
        }

        for UnresolvedTraitConstraint { typ, trait_bound } in self.trait_bounds.clone() {
            if let UnresolvedTypeData::Named(constraint_path, _, _) = &typ.typ {
                // if `path` is `T::method_name`, we're looking for constraint of the form `T: SomeTrait`
                if constraint_path.segments.len() == 1
                    && path.segments[0] != constraint_path.last_segment()
                {
                    continue;
                }

                if let Ok(ModuleDefId::TraitId(trait_id)) =
                    self.resolve_path(trait_bound.trait_path.clone())
                {
                    let the_trait = self.interner.get_trait(trait_id);
                    if let Some(method) =
                        the_trait.find_method(path.segments.last().unwrap().0.contents.as_str())
                    {
                        let constraint = TraitConstraint {
                            trait_id,
                            typ: self.resolve_type(typ.clone()),
                            trait_generics: vecmap(trait_bound.trait_generics, |typ| {
                                self.resolve_type(typ)
                            }),
                        };
                        return Some((method, constraint, true));
                    }
                }
            }
        }
        None
    }

    // Try to resolve the given trait method path.
    //
    // Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    // E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    fn resolve_trait_generic_path(
        &mut self,
        path: &Path,
    ) -> Option<(TraitMethodId, TraitConstraint, bool)> {
        self.resolve_trait_static_method_by_self(path)
            .or_else(|| self.resolve_trait_static_method(path))
            .or_else(|| self.resolve_trait_method_by_named_generic(path))
    }

    fn resolve_path(&mut self, path: Path) -> Result<ModuleDefId, ResolverError> {
        let path_resolution = self.path_resolver.resolve(self.def_maps, path)?;

        if let Some(error) = path_resolution.error {
            self.push_err(error.into());
        }

        Ok(path_resolution.module_def_id)
    }

    fn resolve_block(&mut self, block_expr: BlockExpression) -> HirBlockExpression {
        let statements =
            self.in_new_scope(|this| vecmap(block_expr.statements, |stmt| this.intern_stmt(stmt)));
        HirBlockExpression { statements }
    }

    pub fn intern_block(&mut self, block: BlockExpression) -> ExprId {
        let hir_block = HirExpression::Block(self.resolve_block(block));
        self.interner.push_expr(hir_block)
    }

    fn eval_global_as_array_length(&mut self, global: GlobalId, path: &Path) -> u64 {
        let Some(stmt) = self.interner.get_global_let_statement(global) else {
            let path = path.clone();
            self.push_err(ResolverError::NoSuchNumericTypeVariable { path });
            return 0;
        };

        let length = stmt.expression;
        let span = self.interner.expr_span(&length);
        let result = self.try_eval_array_length_id(length, span);

        match result.map(|length| length.try_into()) {
            Ok(Ok(length_value)) => return length_value,
            Ok(Err(_cast_err)) => self.push_err(ResolverError::IntegerTooLarge { span }),
            Err(Some(error)) => self.push_err(error),
            Err(None) => (),
        }
        0
    }

    fn try_eval_array_length_id(
        &self,
        rhs: ExprId,
        span: Span,
    ) -> Result<u128, Option<ResolverError>> {
        // Arbitrary amount of recursive calls to try before giving up
        let fuel = 100;
        self.try_eval_array_length_id_with_fuel(rhs, span, fuel)
    }

    fn try_eval_array_length_id_with_fuel(
        &self,
        rhs: ExprId,
        span: Span,
        fuel: u32,
    ) -> Result<u128, Option<ResolverError>> {
        if fuel == 0 {
            // If we reach here, it is likely from evaluating cyclic globals. We expect an error to
            // be issued for them after name resolution so issue no error now.
            return Err(None);
        }

        match self.interner.expression(&rhs) {
            HirExpression::Literal(HirLiteral::Integer(int, false)) => {
                int.try_into_u128().ok_or(Some(ResolverError::IntegerTooLarge { span }))
            }
            HirExpression::Ident(ident, _) => {
                let definition = self.interner.definition(ident.id);
                match definition.kind {
                    DefinitionKind::Global(global_id) => {
                        let let_statement = self.interner.get_global_let_statement(global_id);
                        if let Some(let_statement) = let_statement {
                            let expression = let_statement.expression;
                            self.try_eval_array_length_id_with_fuel(expression, span, fuel - 1)
                        } else {
                            Err(Some(ResolverError::InvalidArrayLengthExpr { span }))
                        }
                    }
                    _ => Err(Some(ResolverError::InvalidArrayLengthExpr { span })),
                }
            }
            HirExpression::Infix(infix) => {
                let lhs = self.try_eval_array_length_id_with_fuel(infix.lhs, span, fuel - 1)?;
                let rhs = self.try_eval_array_length_id_with_fuel(infix.rhs, span, fuel - 1)?;

                match infix.operator.kind {
                    BinaryOpKind::Add => Ok(lhs + rhs),
                    BinaryOpKind::Subtract => Ok(lhs - rhs),
                    BinaryOpKind::Multiply => Ok(lhs * rhs),
                    BinaryOpKind::Divide => Ok(lhs / rhs),
                    BinaryOpKind::Equal => Ok((lhs == rhs) as u128),
                    BinaryOpKind::NotEqual => Ok((lhs != rhs) as u128),
                    BinaryOpKind::Less => Ok((lhs < rhs) as u128),
                    BinaryOpKind::LessEqual => Ok((lhs <= rhs) as u128),
                    BinaryOpKind::Greater => Ok((lhs > rhs) as u128),
                    BinaryOpKind::GreaterEqual => Ok((lhs >= rhs) as u128),
                    BinaryOpKind::And => Ok(lhs & rhs),
                    BinaryOpKind::Or => Ok(lhs | rhs),
                    BinaryOpKind::Xor => Ok(lhs ^ rhs),
                    BinaryOpKind::ShiftRight => Ok(lhs >> rhs),
                    BinaryOpKind::ShiftLeft => Ok(lhs << rhs),
                    BinaryOpKind::Modulo => Ok(lhs % rhs),
                }
            }
            _other => Err(Some(ResolverError::InvalidArrayLengthExpr { span })),
        }
    }

    fn resolve_fmt_str_literal(&mut self, str: String, call_expr_span: Span) -> HirLiteral {
        let re = Regex::new(r"\{([a-zA-Z0-9_]+)\}")
            .expect("ICE: an invalid regex pattern was used for checking format strings");
        let mut fmt_str_idents = Vec::new();
        for field in re.find_iter(&str) {
            let matched_str = field.as_str();
            let ident_name = &matched_str[1..(matched_str.len() - 1)];

            let scope_tree = self.scopes.current_scope_tree();
            let variable = scope_tree.find(ident_name);
            if let Some((old_value, _)) = variable {
                old_value.num_times_used += 1;
                let ident = HirExpression::Ident(old_value.ident.clone(), None);
                let expr_id = self.interner.push_expr(ident);
                self.interner.push_expr_location(expr_id, call_expr_span, self.file);
                fmt_str_idents.push(expr_id);
            } else if ident_name.parse::<usize>().is_ok() {
                self.errors.push(ResolverError::NumericConstantInFormatString {
                    name: ident_name.to_owned(),
                    span: call_expr_span,
                });
            } else {
                self.errors.push(ResolverError::VariableNotDeclared {
                    name: ident_name.to_owned(),
                    span: call_expr_span,
                });
            }
        }
        HirLiteral::FmtStr(str, fmt_str_idents)
    }

    fn check_break_continue(&mut self, is_break: bool, span: Span) {
        if !self.in_unconstrained_fn {
            self.push_err(ResolverError::JumpInConstrainedFn { is_break, span });
        }
        if self.nested_loops == 0 {
            self.push_err(ResolverError::JumpOutsideLoop { is_break, span });
        }
    }
}

/// Gives an error if a user tries to create a mutable reference
/// to an immutable variable.
pub fn verify_mutable_reference(interner: &NodeInterner, rhs: ExprId) -> Result<(), ResolverError> {
    match interner.expression(&rhs) {
        HirExpression::MemberAccess(member_access) => {
            verify_mutable_reference(interner, member_access.lhs)
        }
        HirExpression::Index(_) => {
            let span = interner.expr_span(&rhs);
            Err(ResolverError::MutableReferenceToArrayElement { span })
        }
        HirExpression::Ident(ident, _) => {
            if let Some(definition) = interner.try_definition(ident.id) {
                if !definition.mutable {
                    return Err(ResolverError::MutableReferenceToImmutableVariable {
                        span: interner.expr_span(&rhs),
                        variable: definition.name.clone(),
                    });
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
