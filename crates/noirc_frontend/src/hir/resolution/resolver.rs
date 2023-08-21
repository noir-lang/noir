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
use crate::hir_def::expr::{
    HirArrayLiteral, HirBinaryOp, HirBlockExpression, HirCallExpression, HirCapturedVar,
    HirCastExpression, HirConstructorExpression, HirExpression, HirForExpression, HirIdent,
    HirIfExpression, HirIndexExpression, HirInfixExpression, HirLambda, HirLiteral,
    HirMemberAccess, HirMethodCallExpression, HirPrefixExpression,
};
use crate::token::Attribute;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::graph::CrateId;
use crate::hir::def_map::{ModuleDefId, ModuleId, TryFromModuleDefId, MAIN_FUNCTION};
use crate::hir_def::stmt::{HirAssignStatement, HirLValue, HirPattern};
use crate::node_interner::{
    DefinitionId, DefinitionKind, ExprId, FuncId, NodeInterner, StmtId, StructId, TraitId,
};
use crate::{
    hir::{def_map::CrateDefMap, resolution::path_resolver::PathResolver},
    BlockExpression, Expression, ExpressionKind, FunctionKind, Ident, Literal, NoirFunction,
    Statement,
};
use crate::{
    ArrayLiteral, ContractFunctionType, Distinctness, Generics, LValue, NoirStruct, NoirTypeAlias,
    Path, Pattern, Shared, StructType, Trait, Type, TypeAliasType, TypeBinding, TypeVariable,
    UnaryOp, UnresolvedGenerics, UnresolvedType, UnresolvedTypeExpression, Visibility, ERROR_IDENT,
};
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

const SELF_TYPE_NAME: &str = "Self";

type Scope = GenericScope<String, ResolverMeta>;
type ScopeTree = GenericScopeTree<String, ResolverMeta>;
type ScopeForest = GenericScopeForest<String, ResolverMeta>;

pub struct LambdaContext {
    captures: Vec<HirCapturedVar>,
    /// the index in the scope tree
    /// (sometimes being filled by ScopeTree's find method)
    scope_index: usize,
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
    def_maps: &'a HashMap<CrateId, CrateDefMap>,
    interner: &'a mut NodeInterner,
    errors: Vec<ResolverError>,
    file: FileId,

    /// Set to the current type if we're resolving an impl
    self_type: Option<Type>,

    /// Contains a mapping of the current struct or functions's generics to
    /// unique type variables if we're resolving a struct. Empty otherwise.
    /// This is a Vec rather than a map to preserve the order a functions generics
    /// were declared in.
    generics: Vec<(Rc<String>, TypeVariable, Span)>,

    /// When resolving lambda expressions, we need to keep track of the variables
    /// that are captured. We do this in order to create the hidden environment
    /// parameter for the lambda function.
    lambda_stack: Vec<LambdaContext>,
}

/// ResolverMetas are tagged onto each definition to track how many times they are used
#[derive(Debug, PartialEq, Eq)]
struct ResolverMeta {
    num_times_used: usize,
    ident: HirIdent,
    warn_if_unused: bool,
}

impl<'a> Resolver<'a> {
    pub fn new(
        interner: &'a mut NodeInterner,
        path_resolver: &'a dyn PathResolver,
        def_maps: &'a HashMap<CrateId, CrateDefMap>,
        file: FileId,
    ) -> Resolver<'a> {
        Self {
            path_resolver,
            def_maps,
            scopes: ScopeForest::default(),
            interner,
            self_type: None,
            generics: Vec::new(),
            errors: Vec::new(),
            lambda_stack: Vec::new(),
            file,
        }
    }

    pub fn set_self_type(&mut self, self_type: Option<Type>) {
        self.self_type = self_type;
    }

    fn push_err(&mut self, err: ResolverError) {
        self.errors.push(err);
    }

    /// Resolving a function involves interning the metadata
    /// interning any statements inside of the function
    /// and interning the function itself
    /// We resolve and lower the function at the same time
    /// Since lowering would require scope data, unless we add an extra resolution field to the AST
    pub fn resolve_function(
        mut self,
        func: NoirFunction,
        func_id: FuncId,
        module_id: ModuleId,
    ) -> (HirFunction, FuncMeta, Vec<ResolverError>) {
        self.scopes.start_function();

        // Check whether the function has globals in the local module and add them to the scope
        self.resolve_local_globals();

        self.add_generics(&func.def.generics);

        let (hir_func, func_meta) = self.intern_function(func, func_id, module_id);
        let func_scope_tree = self.scopes.end_function();

        self.check_for_unused_variables_in_scope_tree(func_scope_tree);

        (hir_func, func_meta, self.errors)
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
        unused_vars.extend(unused_variables.map(|(_, meta)| meta.ident));
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

        let id = self.interner.push_definition(name.0.contents.clone(), mutable, definition);
        let location = Location::new(name.span(), self.file);
        let ident = HirIdent { location, id };
        let resolver_meta = ResolverMeta { num_times_used: 0, ident, warn_if_unused };

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
        let ident;
        let resolver_meta;

        // This check is necessary to maintain the same definition ids in the interner. Currently, each function uses a new resolver that has its own ScopeForest and thus global scope.
        // We must first check whether an existing definition ID has been inserted as otherwise there will be multiple definitions for the same global statement.
        // This leads to an error in evaluation where the wrong definition ID is selected when evaluating a statement using the global. The check below prevents this error.
        let mut stmt_id = None;
        let global = self.interner.get_all_globals();
        for (global_stmt_id, global_info) in global {
            if global_info.ident == name
                && global_info.local_id == self.path_resolver.local_module_id()
            {
                stmt_id = Some(global_stmt_id);
            }
        }

        if let Some(id) = stmt_id {
            let hir_let_stmt = self.interner.let_statement(&id);
            ident = hir_let_stmt.ident();
            resolver_meta = ResolverMeta { num_times_used: 0, ident, warn_if_unused: true };
        } else {
            let id = self.interner.push_definition(name.0.contents.clone(), false, definition);
            let location = Location::new(name.span(), self.file);
            ident = HirIdent { location, id };
            resolver_meta = ResolverMeta { num_times_used: 0, ident, warn_if_unused: true };
        }

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
            (HirIdent { location, id }, 0)
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
            Ok((HirIdent { location, id }, scope))
        } else {
            Err(ResolverError::VariableNotDeclared {
                name: name.0.contents.clone(),
                span: name.0.span(),
            })
        }
    }

    fn intern_function(
        &mut self,
        func: NoirFunction,
        id: FuncId,
        module_id: ModuleId,
    ) -> (HirFunction, FuncMeta) {
        let func_meta = self.extract_meta(&func, id, module_id);
        let hir_func = match func.kind {
            FunctionKind::Builtin | FunctionKind::LowLevel | FunctionKind::Oracle => {
                HirFunction::empty()
            }
            FunctionKind::Normal => {
                let expr_id = self.intern_block(func.def.body);
                self.interner.push_expr_location(expr_id, func.def.span, self.file);
                HirFunction::unchecked_from_expr(expr_id)
            }
        };

        (hir_func, func_meta)
    }

    /// Translates an UnresolvedType into a Type and appends any
    /// freshly created TypeVariables created to new_variables.
    fn resolve_type_inner(&mut self, typ: UnresolvedType, new_variables: &mut Generics) -> Type {
        match typ {
            UnresolvedType::FieldElement => Type::FieldElement,
            UnresolvedType::Array(size, elem) => {
                let elem = Box::new(self.resolve_type_inner(*elem, new_variables));
                let size = if size.is_none() {
                    Type::NotConstant
                } else {
                    self.resolve_array_size(size, new_variables)
                };
                Type::Array(Box::new(size), elem)
            }
            UnresolvedType::Expression(expr) => self.convert_expression_type(expr),
            UnresolvedType::Integer(sign, bits) => Type::Integer(sign, bits),
            UnresolvedType::Bool => Type::Bool,
            UnresolvedType::String(size) => {
                let resolved_size = self.resolve_array_size(size, new_variables);
                Type::String(Box::new(resolved_size))
            }
            UnresolvedType::FormatString(size, fields) => {
                let resolved_size = self.convert_expression_type(size);
                let fields = self.resolve_type_inner(*fields, new_variables);
                Type::FmtString(Box::new(resolved_size), Box::new(fields))
            }
            UnresolvedType::Unit => Type::Unit,
            UnresolvedType::Unspecified => Type::Error,
            UnresolvedType::Error => Type::Error,
            UnresolvedType::Named(path, args) => self.resolve_named_type(path, args, new_variables),
            UnresolvedType::Tuple(fields) => {
                Type::Tuple(vecmap(fields, |field| self.resolve_type_inner(field, new_variables)))
            }
            UnresolvedType::Function(args, ret, env) => {
                let args = vecmap(args, |arg| self.resolve_type_inner(arg, new_variables));
                let ret = Box::new(self.resolve_type_inner(*ret, new_variables));
                let env = Box::new(self.resolve_type_inner(*env, new_variables));
                Type::Function(args, ret, env)
            }
            UnresolvedType::MutableReference(element) => {
                Type::MutableReference(Box::new(self.resolve_type_inner(*element, new_variables)))
            }
        }
    }

    fn find_generic(&self, target_name: &str) -> Option<&(Rc<String>, TypeVariable, Span)> {
        self.generics.iter().find(|(name, _, _)| name.as_ref() == target_name)
    }

    fn resolve_named_type(
        &mut self,
        path: Path,
        args: Vec<UnresolvedType>,
        new_variables: &mut Generics,
    ) -> Type {
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
        let mut args = vecmap(args, |arg| self.resolve_type_inner(arg, new_variables));

        if let Some(type_alias_type) = self.lookup_type_alias(path.clone()) {
            let expected_generic_count = type_alias_type.generics.len();
            let type_alias_string = type_alias_type.to_string();
            let id = type_alias_type.id;

            self.verify_generics_count(expected_generic_count, &mut args, span, || {
                type_alias_string
            });

            return self.interner.get_type_alias(id).get_type(&args);
        }

        match self.lookup_struct_or_error(path) {
            Some(struct_type) => {
                let expected_generic_count = struct_type.borrow().generics.len();

                self.verify_generics_count(expected_generic_count, &mut args, span, || {
                    struct_type.borrow().to_string()
                });

                Type::Struct(struct_type, args)
            }
            None => Type::Error,
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
                struct_type: type_name(),
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
            Ok(ModuleDefId::GlobalId(id)) => {
                Some(Type::Constant(self.eval_global_as_array_length(id)))
            }
            _ => None,
        }
    }

    fn resolve_array_size(
        &mut self,
        length: Option<UnresolvedTypeExpression>,
        new_variables: &mut Generics,
    ) -> Type {
        match length {
            None => {
                let id = self.interner.next_type_variable_id();
                let typevar = Shared::new(TypeBinding::Unbound(id));
                new_variables.push((id, typevar.clone()));

                // 'Named'Generic is a bit of a misnomer here, we want a type variable that
                // wont be bound over but this one has no name since we do not currently
                // require users to explicitly be generic over array lengths.
                Type::NamedGeneric(typevar, Rc::new("".into()))
            }
            Some(length) => self.convert_expression_type(length),
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
                Ok(id) => return (HirIdent { location, id }, 0),
                Err(_) => error,
            },
            None => match self.lookup_global(path) {
                Ok(id) => return (HirIdent { location, id }, 0),
                Err(error) => error,
            },
        };
        self.push_err(error);
        let id = DefinitionId::dummy_id();
        (HirIdent { location, id }, 0)
    }

    /// Translates an UnresolvedType to a Type
    pub fn resolve_type(&mut self, typ: UnresolvedType) -> Type {
        self.resolve_type_inner(typ, &mut vec![])
    }

    pub fn resolve_type_aliases(
        mut self,
        unresolved: NoirTypeAlias,
    ) -> (Type, Generics, Vec<ResolverError>) {
        let generics = self.add_generics(&unresolved.generics);
        self.resolve_local_globals();
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
        match typ {
            UnresolvedType::Unspecified => self.interner.next_type_variable(),
            other => self.resolve_type_inner(other, &mut vec![]),
        }
    }

    /// Add the given generics to scope.
    /// Each generic will have a fresh Shared<TypeBinding> associated with it.
    pub fn add_generics(&mut self, generics: &UnresolvedGenerics) -> Generics {
        vecmap(generics, |generic| {
            // Map the generic to a fresh type variable
            let id = self.interner.next_type_variable_id();
            let typevar = Shared::new(TypeBinding::Unbound(id));
            let span = generic.0.span();

            // Check for name collisions of this generic
            let name = Rc::new(generic.0.contents.clone());

            if let Some((_, _, first_span)) = self.find_generic(&name) {
                let span = generic.0.span();
                self.errors.push(ResolverError::DuplicateDefinition {
                    name: generic.0.contents.clone(),
                    first_span: *first_span,
                    second_span: span,
                });
            } else {
                self.generics.push((name, typevar.clone(), span));
            }

            (id, typevar)
        })
    }

    pub fn resolve_struct_fields(
        mut self,
        unresolved: NoirStruct,
    ) -> (Generics, Vec<(Ident, Type)>, Vec<ResolverError>) {
        let generics = self.add_generics(&unresolved.generics);

        // Check whether the struct definition has globals in the local module and add them to the scope
        self.resolve_local_globals();

        let fields = vecmap(unresolved.fields, |(ident, typ)| (ident, self.resolve_type(typ)));

        (generics, fields, self.errors)
    }

    fn resolve_local_globals(&mut self) {
        for (stmt_id, global_info) in self.interner.get_all_globals() {
            if global_info.local_id == self.path_resolver.local_module_id() {
                let global_stmt = self.interner.let_statement(&stmt_id);
                let definition = DefinitionKind::Global(global_stmt.expression);
                self.add_global_variable_decl(global_info.ident, definition);
            }
        }
    }

    /// Extract metadata from a NoirFunction
    /// to be used in analysis and intern the function parameters
    /// Prerequisite: self.add_generics() has already been called with the given
    /// function's generics, including any generics from the impl, if any.
    fn extract_meta(
        &mut self,
        func: &NoirFunction,
        func_id: FuncId,
        module_id: ModuleId,
    ) -> FuncMeta {
        let location = Location::new(func.name_ident().span(), self.file);
        let id = self.interner.function_definition_id(func_id);
        let name_ident = HirIdent { id, location };

        let attributes = func.attribute().cloned();

        let mut generics =
            vecmap(self.generics.clone(), |(name, typevar, _)| match &*typevar.borrow() {
                TypeBinding::Unbound(id) => (*id, typevar.clone()),
                TypeBinding::Bound(binding) => {
                    unreachable!("Expected {} to be unbound, but it is bound to {}", name, binding)
                }
            });

        let mut parameters = vec![];
        let mut parameter_types = vec![];

        for (pattern, typ, visibility) in func.parameters().iter().cloned() {
            if visibility == Visibility::Public && !self.pub_allowed(func) {
                self.push_err(ResolverError::UnnecessaryPub {
                    ident: func.name_ident().clone(),
                    position: PubPosition::Parameter,
                });
            }

            let pattern = self.resolve_pattern(pattern, DefinitionKind::Local(None));
            let typ = self.resolve_type_inner(typ, &mut generics);

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

        // 'pub_allowed' also implies 'pub' is required on return types
        if self.pub_allowed(func)
            && return_type.as_ref() != &Type::Unit
            && func.def.return_visibility != Visibility::Public
        {
            self.push_err(ResolverError::NecessaryPub { ident: func.name_ident().clone() });
        }

        if !self.distinct_allowed(func)
            && func.def.return_distinctness != Distinctness::DuplicationAllowed
        {
            self.push_err(ResolverError::DistinctNotAllowed { ident: func.name_ident().clone() });
        }

        if attributes == Some(Attribute::Test) && !parameters.is_empty() {
            self.push_err(ResolverError::TestFunctionHasParameters {
                span: func.name_ident().span(),
            });
        }

        let mut typ = Type::Function(parameter_types, return_type, Box::new(Type::Unit));

        if !generics.is_empty() {
            typ = Type::Forall(generics, Box::new(typ));
        }

        self.interner.push_definition_type(name_ident.id, typ.clone());

        FuncMeta {
            name: name_ident,
            kind: func.kind,
            attributes,
            module_id,
            contract_function_type: self.handle_function_type(func),
            is_internal: self.handle_is_function_internal(func),
            is_unconstrained: func.def.is_unconstrained,
            location,
            typ,
            parameters: parameters.into(),
            return_type: func.def.return_type.clone(),
            return_visibility: func.def.return_visibility,
            return_distinctness: func.def.return_distinctness,
            has_body: !func.def.body.is_empty(),
        }
    }

    /// True if the 'pub' keyword is allowed on parameters in this function
    fn pub_allowed(&self, func: &NoirFunction) -> bool {
        if self.in_contract() {
            !func.def.is_unconstrained
        } else {
            func.name() == MAIN_FUNCTION
        }
    }

    /// True if the `distinct` keyword is allowed on a function's return type
    fn distinct_allowed(&self, func: &NoirFunction) -> bool {
        if self.in_contract() {
            // "open" and "unconstrained" functions are compiled to brillig and thus duplication of
            // witness indices in their abis is not a concern.
            !func.def.is_unconstrained && !func.def.is_open
        } else {
            func.name() == MAIN_FUNCTION
        }
    }

    fn handle_function_type(&mut self, func: &NoirFunction) -> Option<ContractFunctionType> {
        if func.def.is_open {
            if self.in_contract() {
                Some(ContractFunctionType::Open)
            } else {
                self.push_err(ResolverError::ContractFunctionTypeInNormalFunction {
                    span: func.name_ident().span(),
                });
                None
            }
        } else {
            Some(ContractFunctionType::Secret)
        }
    }

    fn handle_is_function_internal(&mut self, func: &NoirFunction) -> Option<bool> {
        if self.in_contract() {
            Some(func.def.is_internal)
        } else {
            if func.def.is_internal {
                self.push_err(ResolverError::ContractFunctionInternalInNormalFunction {
                    span: func.name_ident().span(),
                });
            }
            None
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
        let mut found = HashMap::new();
        for parameter in parameters {
            Self::find_numeric_generics_in_type(parameter, &mut found);
        }
        Self::find_numeric_generics_in_type(return_type, &mut found);
        found.into_iter().collect()
    }

    fn find_numeric_generics_in_type(typ: &Type, found: &mut HashMap<String, Shared<TypeBinding>>) {
        match typ {
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Unit
            | Type::Error
            | Type::TypeVariable(_, _)
            | Type::Constant(_)
            | Type::NamedGeneric(_, _)
            | Type::NotConstant
            | Type::Forall(_, _) => (),

            Type::Array(length, element_type) => {
                if let Type::NamedGeneric(type_variable, name) = length.as_ref() {
                    found.insert(name.to_string(), type_variable.clone());
                }
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

    pub fn resolve_global_let(&mut self, let_stmt: crate::LetStatement) -> HirStatement {
        let expression = self.resolve_expression(let_stmt.expression);
        let definition = DefinitionKind::Global(expression);

        HirStatement::Let(HirLetStatement {
            pattern: self.resolve_pattern(let_stmt.pattern, definition),
            r#type: self.resolve_type(let_stmt.r#type),
            expression,
        })
    }

    pub fn resolve_stmt(&mut self, stmt: Statement) -> HirStatement {
        match stmt {
            Statement::Let(let_stmt) => {
                let expression = self.resolve_expression(let_stmt.expression);
                let definition = DefinitionKind::Local(Some(expression));
                HirStatement::Let(HirLetStatement {
                    pattern: self.resolve_pattern(let_stmt.pattern, definition),
                    r#type: self.resolve_type(let_stmt.r#type),
                    expression,
                })
            }
            Statement::Constrain(constrain_stmt) => {
                let lhs_id = self.resolve_expression(constrain_stmt.0);
                let rhs_id = self.resolve_expression(constrain_stmt.1);

                HirStatement::Constrain(HirConstrainStatement(lhs_id, rhs_id, self.file))
            }
            Statement::Expression(expr) => HirStatement::Expression(self.resolve_expression(expr)),
            Statement::Semi(expr) => HirStatement::Semi(self.resolve_expression(expr)),
            Statement::Assign(assign_stmt) => {
                let identifier = self.resolve_lvalue(assign_stmt.lvalue);
                let expression = self.resolve_expression(assign_stmt.expression);
                let stmt = HirAssignStatement { lvalue: identifier, expression };
                HirStatement::Assign(stmt)
            }
            Statement::Error => HirStatement::Error,
        }
    }

    pub fn intern_stmt(&mut self, stmt: Statement) -> StmtId {
        let hir_stmt = self.resolve_stmt(stmt);
        self.interner.push_stmt(hir_stmt)
    }

    fn resolve_lvalue(&mut self, lvalue: LValue) -> HirLValue {
        match lvalue {
            LValue::Ident(ident) => {
                let ident = self.find_variable_or_default(&ident);
                self.resolve_local_variable(ident.0, ident.1);

                HirLValue::Ident(ident.0, Type::Error)
            }
            LValue::MemberAccess { object, field_name } => {
                let object = Box::new(self.resolve_lvalue(*object));
                HirLValue::MemberAccess { object, field_name, field_index: None, typ: Type::Error }
            }
            LValue::Index { array, index } => {
                let array = Box::new(self.resolve_lvalue(*array));
                let index = self.resolve_expression(index);
                HirLValue::Index { array, index, typ: Type::Error }
            }
            LValue::Dereference(lvalue) => {
                let lvalue = Box::new(self.resolve_lvalue(*lvalue));
                HirLValue::Dereference { lvalue, element_type: Type::Error }
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
                    self.lambda_stack[lambda_index]
                        .captures
                        .push(HirCapturedVar { ident: hir_ident, transitive_capture_index });
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

    pub fn resolve_expression(&mut self, expr: Expression) -> ExprId {
        let hir_expr = match expr.kind {
            ExpressionKind::Literal(literal) => HirExpression::Literal(match literal {
                Literal::Bool(b) => HirLiteral::Bool(b),
                Literal::Array(ArrayLiteral::Standard(elements)) => {
                    let elements = vecmap(elements, |elem| self.resolve_expression(elem));
                    HirLiteral::Array(HirArrayLiteral::Standard(elements))
                }
                Literal::Array(ArrayLiteral::Repeated { repeated_element, length }) => {
                    let span = length.span;
                    let length = UnresolvedTypeExpression::from_expr(*length, span).unwrap_or_else(
                        |error| {
                            self.errors.push(ResolverError::ParserError(Box::new(error)));
                            UnresolvedTypeExpression::Constant(0, span)
                        },
                    );

                    let length = self.convert_expression_type(length);
                    let repeated_element = self.resolve_expression(*repeated_element);

                    HirLiteral::Array(HirArrayLiteral::Repeated { repeated_element, length })
                }
                Literal::Integer(integer) => HirLiteral::Integer(integer),
                Literal::Str(str) => HirLiteral::Str(str),
                Literal::FmtStr(str) => self.resolve_fmt_str_literal(str, expr.span),
                Literal::Unit => HirLiteral::Unit,
            }),
            ExpressionKind::Variable(path) => {
                // If the Path is being used as an Expression, then it is referring to a global from a separate module
                // Otherwise, then it is referring to an Identifier
                // This lookup allows support of such statements: let x = foo::bar::SOME_GLOBAL + 10;
                // If the expression is a singular indent, we search the resolver's current scope as normal.
                let (hir_ident, var_scope_index) = self.get_ident_from_path(path);

                if hir_ident.id != DefinitionId::dummy_id() {
                    match self.interner.definition(hir_ident.id).kind {
                        DefinitionKind::Function(_) => {}
                        DefinitionKind::Global(_) => {}
                        DefinitionKind::GenericType(_) => {
                            // Initialize numeric generics to a polymorphic integer type in case
                            // they're used in expressions. We must do this here since the type
                            // checker does not check definition kinds and otherwise expects
                            // parameters to already be typed.
                            if self.interner.id_type(hir_ident.id) == Type::Error {
                                let typ = Type::polymorphic_integer(self.interner);
                                self.interner.push_definition_type(hir_ident.id, typ);
                            }
                        }
                        DefinitionKind::Local(_) => {
                            // only local variables can be captured by closures.
                            self.resolve_local_variable(hir_ident, var_scope_index);
                        }
                    }
                }

                HirExpression::Ident(hir_ident)
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

                HirExpression::Infix(HirInfixExpression {
                    lhs,
                    operator: HirBinaryOp::new(infix.operator, self.file),
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
                let arguments = vecmap(call_expr.arguments, |arg| self.resolve_expression(arg));
                let location = Location::new(expr.span, self.file);
                HirExpression::MethodCall(HirMethodCallExpression {
                    arguments,
                    method,
                    object,
                    location,
                })
            }
            ExpressionKind::Cast(cast_expr) => HirExpression::Cast(HirCastExpression {
                lhs: self.resolve_expression(cast_expr.lhs),
                r#type: self.resolve_type(cast_expr.r#type),
            }),
            ExpressionKind::For(for_expr) => {
                let start_range = self.resolve_expression(for_expr.start_range);
                let end_range = self.resolve_expression(for_expr.end_range);
                let (identifier, block) = (for_expr.identifier, for_expr.block);

                // TODO: For loop variables are currently mutable by default since we haven't
                //       yet implemented syntax for them to be optionally mutable.
                let (identifier, block_id) = self.in_new_scope(|this| {
                    let decl = this.add_variable_decl(
                        identifier,
                        false,
                        true,
                        DefinitionKind::Local(None),
                    );
                    (decl, this.resolve_expression(block))
                });

                HirExpression::For(HirForExpression {
                    start_range,
                    end_range,
                    block: block_id,
                    identifier,
                })
            }
            ExpressionKind::If(if_expr) => HirExpression::If(HirIfExpression {
                condition: self.resolve_expression(if_expr.condition),
                consequence: self.resolve_expression(if_expr.consequence),
                alternative: if_expr.alternative.map(|e| self.resolve_expression(e)),
            }),
            ExpressionKind::Index(indexed_expr) => HirExpression::Index(HirIndexExpression {
                collection: self.resolve_expression(indexed_expr.collection),
                index: self.resolve_expression(indexed_expr.index),
            }),
            ExpressionKind::Block(block_expr) => self.resolve_block(block_expr),
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
        };

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
            Pattern::Mutable(pattern, span) => {
                if let Some(first_mut) = mutable {
                    self.push_err(ResolverError::UnnecessaryMut { first_mut, second_mut: span });
                }

                let pattern = self.resolve_pattern_mutable(*pattern, Some(span), definition);
                HirPattern::Mutable(Box::new(pattern), span)
            }
            Pattern::Tuple(fields, span) => {
                let fields = vecmap(fields, |field| {
                    self.resolve_pattern_mutable(field, mutable, definition.clone())
                });
                HirPattern::Tuple(fields, span)
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
                HirPattern::Struct(typ, fields, span)
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

    pub fn get_trait(&self, type_id: TraitId) -> Shared<Trait> {
        self.interner.get_trait(type_id)
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
            let let_stmt = self.interner.let_statement(&global);
            return Ok(let_stmt.ident().id);
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

    fn lookup_type_alias(&mut self, path: Path) -> Option<&TypeAliasType> {
        self.lookup(path).ok().map(|id| self.interner.get_type_alias(id))
    }

    fn resolve_path(&mut self, path: Path) -> Result<ModuleDefId, ResolverError> {
        self.path_resolver.resolve(self.def_maps, path).map_err(ResolverError::PathResolutionError)
    }

    fn resolve_block(&mut self, block_expr: BlockExpression) -> HirExpression {
        let statements =
            self.in_new_scope(|this| vecmap(block_expr.0, |stmt| this.intern_stmt(stmt)));
        HirExpression::Block(HirBlockExpression(statements))
    }

    pub fn intern_block(&mut self, block: BlockExpression) -> ExprId {
        let hir_block = self.resolve_block(block);
        self.interner.push_expr(hir_block)
    }

    fn eval_global_as_array_length(&mut self, global: StmtId) -> u64 {
        let stmt = match self.interner.statement(&global) {
            HirStatement::Let(let_expr) => let_expr,
            other => {
                dbg!(other);
                return 0;
            }
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
        match self.interner.expression(&rhs) {
            HirExpression::Literal(HirLiteral::Integer(int)) => {
                int.try_into_u128().ok_or(Some(ResolverError::IntegerTooLarge { span }))
            }
            _other => Err(Some(ResolverError::InvalidArrayLengthExpr { span })),
        }
    }

    fn in_contract(&self) -> bool {
        let module_id = self.path_resolver.module_id();
        module_id.module(self.def_maps).is_contract
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
                let expr_id = self.interner.push_expr(HirExpression::Ident(old_value.ident));
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
        HirExpression::Ident(ident) => {
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

// XXX: These tests repeat a lot of code
// what we should do is have test cases which are passed to a test harness
// A test harness will allow for more expressive and readable tests
#[cfg(test)]
mod test {

    use core::panic;
    use std::collections::HashMap;

    use fm::FileId;
    use iter_extended::vecmap;
    use noirc_errors::Location;

    use crate::hir::def_map::{ModuleData, ModuleId};
    use crate::hir::resolution::errors::ResolverError;
    use crate::hir::resolution::import::PathResolutionError;
    use crate::hir::resolution::resolver::StmtId;

    use crate::graph::CrateId;
    use crate::hir_def::expr::HirExpression;
    use crate::hir_def::function::HirFunction;
    use crate::hir_def::stmt::HirStatement;
    use crate::node_interner::{FuncId, NodeInterner};
    use crate::ParsedModule;
    use crate::{
        hir::def_map::{CrateDefMap, LocalModuleId, ModuleDefId},
        parse_program, Path,
    };

    use super::{PathResolver, Resolver};

    // func_namespace is used to emulate the fact that functions can be imported
    // and functions can be forward declared
    fn init_src_code_resolution(
        src: &str,
    ) -> (ParsedModule, NodeInterner, HashMap<CrateId, CrateDefMap>, FileId, TestPathResolver) {
        let (program, errors) = parse_program(src);
        if !errors.is_empty() {
            panic!("Unexpected parse errors in test code: {:?}", errors);
        }

        let interner: NodeInterner = NodeInterner::default();

        let mut def_maps: HashMap<CrateId, CrateDefMap> = HashMap::new();
        let file = FileId::default();

        let mut modules = arena::Arena::new();
        let location = Location::new(Default::default(), file);
        modules.insert(ModuleData::new(None, location, false));

        let path_resolver = TestPathResolver(HashMap::new());

        def_maps.insert(
            CrateId::dummy_id(),
            CrateDefMap {
                root: path_resolver.local_module_id(),
                modules,
                krate: CrateId::dummy_id(),
                extern_prelude: HashMap::new(),
            },
        );

        (program, interner, def_maps, file, path_resolver)
    }

    // func_namespace is used to emulate the fact that functions can be imported
    // and functions can be forward declared
    fn resolve_src_code(src: &str, func_namespace: Vec<&str>) -> Vec<ResolverError> {
        let (program, mut interner, def_maps, file, mut path_resolver) =
            init_src_code_resolution(src);

        let func_ids = vecmap(&func_namespace, |name| {
            let id = interner.push_fn(HirFunction::empty());
            interner.push_function_definition(name.to_string(), id);
            id
        });

        for (name, id) in func_namespace.into_iter().zip(func_ids) {
            path_resolver.insert_func(name.to_owned(), id);
        }

        let mut errors = Vec::new();
        for func in program.functions {
            let id = interner.push_fn(HirFunction::empty());
            interner.push_function_definition(func.name().to_string(), id);

            let resolver = Resolver::new(&mut interner, &path_resolver, &def_maps, file);
            let (_, _, err) = resolver.resolve_function(func, id, ModuleId::dummy_id());
            errors.extend(err);
        }

        errors
    }

    fn get_program_captures(src: &str) -> Vec<Vec<String>> {
        let (program, mut interner, def_maps, file, mut path_resolver) =
            init_src_code_resolution(src);

        let mut all_captures: Vec<Vec<String>> = Vec::new();
        for func in program.functions {
            let id = interner.push_fn(HirFunction::empty());
            interner.push_function_definition(func.name().to_string(), id);
            path_resolver.insert_func(func.name().to_owned(), id);

            let resolver = Resolver::new(&mut interner, &path_resolver, &def_maps, file);
            let (hir_func, _, _) = resolver.resolve_function(func, id, ModuleId::dummy_id());

            // Iterate over function statements and apply filtering function
            parse_statement_blocks(
                hir_func.block(&interner).statements(),
                &interner,
                &mut all_captures,
            );
        }
        all_captures
    }

    fn parse_statement_blocks(
        stmts: &[StmtId],
        interner: &NodeInterner,
        result: &mut Vec<Vec<String>>,
    ) {
        let mut expr: HirExpression;

        for stmt_id in stmts.iter() {
            let hir_stmt = interner.statement(stmt_id);
            match hir_stmt {
                HirStatement::Expression(expr_id) => {
                    expr = interner.expression(&expr_id);
                }
                HirStatement::Let(let_stmt) => {
                    expr = interner.expression(&let_stmt.expression);
                }
                HirStatement::Assign(assign_stmt) => {
                    expr = interner.expression(&assign_stmt.expression);
                }
                HirStatement::Constrain(constr_stmt) => {
                    expr = interner.expression(&constr_stmt.0);
                }
                HirStatement::Semi(semi_expr) => {
                    expr = interner.expression(&semi_expr);
                }
                HirStatement::Error => panic!("Invalid HirStatement!"),
            }
            get_lambda_captures(expr, interner, result); // TODO: dyn filter function as parameter
        }
    }

    fn get_lambda_captures(
        expr: HirExpression,
        interner: &NodeInterner,
        result: &mut Vec<Vec<String>>,
    ) {
        if let HirExpression::Lambda(lambda_expr) = expr {
            let mut cur_capture = Vec::new();

            for capture in lambda_expr.captures.iter() {
                cur_capture.push(interner.definition(capture.ident.id).name.clone());
            }
            result.push(cur_capture);

            // Check for other captures recursively within the lambda body
            let hir_body_expr = interner.expression(&lambda_expr.body);
            if let HirExpression::Block(block_expr) = hir_body_expr {
                parse_statement_blocks(block_expr.statements(), interner, result);
            }
        }
    }

    #[test]
    fn resolve_empty_function() {
        let src = "
            fn main() {

            }
        ";

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_basic_function() {
        let src = r#"
            fn main(x : Field) {
                let y = x + x;
                assert(y == x);
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_unused_var() {
        let src = r#"
            fn main(x : Field) {
                let y = x + x;
                assert(x == x);
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);

        // There should only be one error
        assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);

        // It should be regarding the unused variable
        match &errors[0] {
            ResolverError::UnusedVariable { ident } => {
                assert_eq!(&ident.0.contents, "y");
            }
            _ => unreachable!("we should only have an unused var error"),
        }
    }

    #[test]
    fn resolve_unresolved_var() {
        let src = r#"
            fn main(x : Field) {
                let y = x + x;
                assert(y == z);
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);

        // There should only be one error
        assert!(errors.len() == 1);

        // It should be regarding the unresolved var `z` (Maybe change to undeclared and special case)
        match &errors[0] {
            ResolverError::VariableNotDeclared { name, span: _ } => assert_eq!(name, "z"),
            _ => unimplemented!("we should only have an unresolved variable"),
        }
    }

    #[test]
    fn unresolved_path() {
        let src = "
            fn main(x : Field) {
                let _z = some::path::to::a::func(x);
            }
        ";

        let mut errors = resolve_src_code(src, vec!["main", "foo"]);
        assert_eq!(errors.len(), 1);
        let err = errors.pop().unwrap();

        path_unresolved_error(err, "func");
    }

    #[test]
    fn resolve_literal_expr() {
        let src = r#"
            fn main(x : Field) {
                let y = 5;
                assert(y == x);
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.is_empty());
    }

    #[test]
    fn multiple_resolution_errors() {
        let src = r#"
            fn main(x : Field) {
               let y = foo::bar(x);
               let z = y + a;
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.len() == 3, "Expected 3 errors, got: {:?}", errors);

        // Errors are:
        // `a` is undeclared
        // `z` is unused
        // `foo::bar` does not exist
        for err in errors {
            match &err {
                ResolverError::UnusedVariable { ident } => {
                    assert_eq!(&ident.0.contents, "z");
                }
                ResolverError::VariableNotDeclared { name, .. } => {
                    assert_eq!(name, "a");
                }
                ResolverError::PathResolutionError(_) => path_unresolved_error(err, "bar"),
                _ => unimplemented!(),
            };
        }
    }

    #[test]
    fn resolve_prefix_expr() {
        let src = r#"
            fn main(x : Field) {
                let _y = -x;
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_for_expr() {
        let src = r#"
            fn main(x : Field) {
                for i in 1..20 {
                    let _z = x + i;
                };
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_call_expr() {
        let src = r#"
            fn main(x : Field) {
                let _z = foo(x);
            }

            fn foo(x : Field) -> Field {
                x
            }
        "#;

        let errors = resolve_src_code(src, vec!["main", "foo"]);
        assert!(errors.is_empty());
    }

    #[test]
    fn resolve_shadowing() {
        let src = r#"
            fn main(x : Field) {
                let x = foo(x);
                let x = x;
                let (x, x) = (x, x);
                let _ = x;
            }

            fn foo(x : Field) -> Field {
                x
            }
        "#;
        let errors = resolve_src_code(src, vec!["main", "foo"]);
        if !errors.is_empty() {
            println!("Unexpected errors: {:?}", errors);
            unreachable!("there should be no errors");
        }
    }

    #[test]
    fn resolve_basic_closure() {
        let src = r#"
            fn main(x : Field) -> pub Field {
                let closure = |y| y + x;
                closure(x)
            }
        "#;

        let errors = resolve_src_code(src, vec!["main", "foo"]);
        if !errors.is_empty() {
            panic!("Unexpected errors: {:?}", errors);
        }
    }

    #[test]
    fn resolve_simplified_closure() {
        // based on bug https://github.com/noir-lang/noir/issues/1088

        let src = r#"fn do_closure(x: Field) -> Field {
            let y = x;
            let ret_capture = || {
              y
            };
            ret_capture()
          }

          fn main(x: Field) {
              assert(do_closure(x) == 100);
          }

          "#;
        let parsed_captures = get_program_captures(src);
        let expected_captures = vec![vec!["y".to_string()]];
        assert_eq!(expected_captures, parsed_captures);
    }

    #[test]
    fn resolve_complex_closures() {
        let src = r#"
            fn main(x: Field) -> pub Field {
                let closure_without_captures = |x| x + x;
                let a = closure_without_captures(1);

                let closure_capturing_a_param = |y| y + x;
                let b = closure_capturing_a_param(2);

                let closure_capturing_a_local_var = |y| y + b;
                let c = closure_capturing_a_local_var(3);

                let closure_with_transitive_captures = |y| {
                    let d = 5;
                    let nested_closure = |z| {
                        let doubly_nested_closure = |w| w + x + b;
                        a + z + y + d + x + doubly_nested_closure(4) + x + y
                    };
                    let res = nested_closure(5);
                    res
                };

                a + b + c + closure_with_transitive_captures(6)
            }
        "#;

        let errors = resolve_src_code(src, vec!["main", "foo"]);
        assert!(errors.is_empty());
        if !errors.is_empty() {
            println!("Unexpected errors: {:?}", errors);
            unreachable!("there should be no errors");
        }

        let expected_captures = vec![
            vec![],
            vec!["x".to_string()],
            vec!["b".to_string()],
            vec!["x".to_string(), "b".to_string(), "a".to_string()],
            vec![
                "x".to_string(),
                "b".to_string(),
                "a".to_string(),
                "y".to_string(),
                "d".to_string(),
            ],
            vec!["x".to_string(), "b".to_string()],
        ];

        let parsed_captures = get_program_captures(src);

        assert_eq!(expected_captures, parsed_captures);
    }

    #[test]
    fn resolve_fmt_strings() {
        let src = r#"
            fn main() {
                let string = f"this is i: {i}";
                println(string);

                println(f"I want to print {0}");

                let new_val = 10;
                println(f"randomstring{new_val}{new_val}");
            }
            fn println<T>(x : T) -> T {
                x
            }
        "#;

        let errors = resolve_src_code(src, vec!["main", "println"]);
        assert!(errors.len() == 2, "Expected 2 errors, got: {:?}", errors);

        for err in errors {
            match &err {
                ResolverError::VariableNotDeclared { name, .. } => {
                    assert_eq!(name, "i");
                }
                ResolverError::NumericConstantInFormatString { name, .. } => {
                    assert_eq!(name, "0");
                }
                _ => unimplemented!(),
            };
        }
    }

    // possible TODO: Create a more sophisticated set of search functions over the HIR, so we can check
    //       that the correct variables are captured in each closure

    fn path_unresolved_error(err: ResolverError, expected_unresolved_path: &str) {
        match err {
            ResolverError::PathResolutionError(PathResolutionError::Unresolved(name)) => {
                assert_eq!(name.to_string(), expected_unresolved_path);
            }
            _ => unimplemented!("expected an unresolved path"),
        }
    }

    struct TestPathResolver(HashMap<String, ModuleDefId>);

    impl PathResolver for TestPathResolver {
        fn resolve(
            &self,
            _def_maps: &HashMap<CrateId, CrateDefMap>,
            path: Path,
        ) -> Result<ModuleDefId, PathResolutionError> {
            // Not here that foo::bar and hello::foo::bar would fetch the same thing
            let name = path.segments.last().unwrap();
            let mod_def = self.0.get(&name.0.contents).cloned();
            mod_def.ok_or_else(move || PathResolutionError::Unresolved(name.clone()))
        }

        fn local_module_id(&self) -> LocalModuleId {
            // This is not LocalModuleId::dummy since we need to use this to index into a Vec
            // later and do not want to push u32::MAX number of elements before we do.
            LocalModuleId(arena::Index::from_raw_parts(0, 0))
        }

        fn module_id(&self) -> ModuleId {
            ModuleId { krate: CrateId::dummy_id(), local_id: self.local_module_id() }
        }
    }

    impl TestPathResolver {
        fn insert_func(&mut self, name: String, func_id: FuncId) {
            self.0.insert(name, func_id.into());
        }
    }
}
