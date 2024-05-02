//! A pass to convert the Ast to Hir, performing
//! little to no other transformations besides removing
//! unnecessary nodes like `ExpressionKind::Parenthesized`

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

fn intern_function(interner: &mut NodeInterner, func: NoirFunction, id: FuncId) -> (HirFunction, FuncMeta) {
    let func_meta = interner.extract_meta(&func, id);

    let hir_func = match func.kind {
        FunctionKind::Builtin | FunctionKind::LowLevel | FunctionKind::Oracle => {
            HirFunction::empty()
        }
        FunctionKind::Normal | FunctionKind::Recursive => {
            let expr_id = intern_block(interner, func.def.body);
            interner.push_expr_location(expr_id, func.def.span, interner.file);
            HirFunction::unchecked_from_expr(expr_id)
        }
    };

    (hir_func, func_meta)
}

pub fn resolve_trait_constraint(
    interner: &mut NodeInterner,
    constraint: UnresolvedTraitConstraint,
) -> Option<TraitConstraint> {
    let typ = intern_type(interner, constraint.typ);
    let trait_generics =
        vecmap(constraint.trait_bound.trait_generics, |typ| intern_type(interner, typ));

    let span = constraint.trait_bound.trait_path.span();
    let the_trait = interner.lookup_trait_or_error(constraint.trait_bound.trait_path)?;
    let trait_id = the_trait.id;

    let expected_generics = the_trait.generics.len();
    let actual_generics = trait_generics.len();

    if actual_generics != expected_generics {
        let item_name = the_trait.name.to_string();
        interner.push_err(ResolverError::IncorrectGenericCount {
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
fn resolve_type_inner(interner: &mut NodeInterner, typ: UnresolvedType, new_variables: &mut Generics) -> Type {
    use crate::ast::UnresolvedTypeData::*;

    let resolved_type = match typ.typ {
        FieldElement => Type::FieldElement,
        Array(size, elem) => {
            let elem = Box::new(interner.resolve_type_inner(*elem, new_variables));
            let size = interner.resolve_array_size(Some(size), new_variables);
            Type::Array(Box::new(size), elem)
        }
        Slice(elem) => {
            let elem = Box::new(interner.resolve_type_inner(*elem, new_variables));
            Type::Slice(elem)
        }
        Expression(expr) => interner.convert_expression_type(expr),
        Integer(sign, bits) => Type::Integer(sign, bits),
        Bool => Type::Bool,
        String(size) => {
            let resolved_size = interner.resolve_array_size(size, new_variables);
            Type::String(Box::new(resolved_size))
        }
        FormatString(size, fields) => {
            let resolved_size = interner.convert_expression_type(size);
            let fields = interner.resolve_type_inner(*fields, new_variables);
            Type::FmtString(Box::new(resolved_size), Box::new(fields))
        }
        Code => Type::Code,
        Unit => Type::Unit,
        Unspecified => Type::Error,
        Error => Type::Error,
        Named(path, args, _) => interner.resolve_named_type(path, args, new_variables),
        TraitAsType(path, args) => interner.resolve_trait_as_type(path, args, new_variables),

        Tuple(fields) => {
            Type::Tuple(vecmap(fields, |field| interner.resolve_type_inner(field, new_variables)))
        }
        Function(args, ret, env) => {
            let args = vecmap(args, |arg| interner.resolve_type_inner(arg, new_variables));
            let ret = Box::new(interner.resolve_type_inner(*ret, new_variables));

            // expect() here is valid, because the only places we don't have a span are omitted types
            // e.g. a function without return type implicitly has a spanless UnresolvedType::Unit return type
            // To get an invalid env type, the user must explicitly specify the type, which will have a span
            let env_span =
                env.span.expect("Unexpected missing span for closure environment type");

            let env = Box::new(interner.resolve_type_inner(*env, new_variables));

            match *env {
                Type::Unit | Type::Tuple(_) | Type::NamedGeneric(_, _) => {
                    Type::Function(args, ret, env)
                }
                _ => {
                    interner.push_err(ResolverError::InvalidClosureEnvironment {
                        typ: *env,
                        span: env_span,
                    });
                    Type::Error
                }
            }
        }
        MutableReference(element) => {
            Type::MutableReference(Box::new(interner.resolve_type_inner(*element, new_variables)))
        }
        Parenthesized(typ) => interner.resolve_type_inner(*typ, new_variables),
    };

    if let Type::Struct(_, _) = resolved_type {
        if let Some(unresolved_span) = typ.span {
            // Record the location of the type reference
            interner.push_type_ref_location(
                resolved_type.clone(),
                Location::new(unresolved_span, interner.file),
            );
        }
    }
    resolved_type
}

fn find_generic(interner: &NodeInterner, target_name: &str) -> Option<&(Rc<String>, TypeVariable, Span)> {
    interner.generics.iter().find(|(name, _, _)| name.as_ref() == target_name)
}

fn resolve_named_type(
    interner: &mut NodeInterner,
    path: Path,
    args: Vec<UnresolvedType>,
    new_variables: &mut Generics,
) -> Type {
    if args.is_empty() {
        if let Some(typ) = interner.lookup_generic_or_global_type(&path) {
            return typ;
        }
    }

    // Check if the path is a type variable first. We currently disallow generics on type
    // variables since we do not support higher-kinded types.
    if path.segments.len() == 1 {
        let name = &path.last_segment().0.contents;

        if name == SELF_TYPE_NAME {
            if let Some(self_type) = interner.self_type.clone() {
                if !args.is_empty() {
                    interner.push_err(ResolverError::GenericsOnSelfType { span: path.span() });
                }
                return self_type;
            }
        }
    }

    let span = path.span();
    let mut args = vecmap(args, |arg| interner.resolve_type_inner(arg, new_variables));

    if let Some(type_alias) = interner.lookup_type_alias(path.clone()) {
        let type_alias = type_alias.borrow();
        let expected_generic_count = type_alias.generics.len();
        let type_alias_string = type_alias.to_string();
        let id = type_alias.id;

        interner.verify_generics_count(expected_generic_count, &mut args, span, || {
            type_alias_string
        });

        if let Some(item) = interner.current_item {
            interner.add_type_alias_dependency(item, id);
        }

        // Collecting Type Alias references [Location]s to be used by LSP in order
        // to resolve the definition of the type alias
        interner.add_type_alias_ref(id, Location::new(span, interner.file));

        // Because there is no ordering to when type aliases (and other globals) are resolved,
        // it is possible for one to refer to an Error type and issue no error if it is set
        // equal to another type alias. Fixing this fully requires an analysis to create a DFG
        // of definition ordering, but for now we have an explicit check here so that we at
        // least issue an error that the type was not found instead of silently passing.
        let alias = interner.get_type_alias(id);
        return Type::Alias(alias, args);
    }

    match interner.lookup_struct_or_error(path) {
        Some(struct_type) => {
            if interner.resolving_ids.contains(&struct_type.borrow().id) {
                interner.push_err(ResolverError::SelfReferentialStruct {
                    span: struct_type.borrow().name.span(),
                });

                return Type::Error;
            }

            let expected_generic_count = struct_type.borrow().generics.len();
            if !interner.in_contract
                && self
                    .interner
                    .struct_attributes(&struct_type.borrow().id)
                    .iter()
                    .any(|attr| matches!(attr, SecondaryAttribute::Abi(_)))
            {
                interner.push_err(ResolverError::AbiAttributeOutsideContract {
                    span: struct_type.borrow().name.span(),
                });
            }
            interner.verify_generics_count(expected_generic_count, &mut args, span, || {
                struct_type.borrow().to_string()
            });

            if let Some(current_item) = interner.current_item {
                let dependency_id = struct_type.borrow().id;
                interner.add_type_dependency(current_item, dependency_id);
            }

            Type::Struct(struct_type, args)
        }
        None => Type::Error,
    }
}

fn resolve_trait_as_type(
    interner: &mut NodeInterner,
    path: Path,
    args: Vec<UnresolvedType>,
    new_variables: &mut Generics,
) -> Type {
    let args = vecmap(args, |arg| interner.resolve_type_inner(arg, new_variables));

    if let Some(t) = interner.lookup_trait_or_error(path) {
        Type::TraitAsType(t.id, Rc::new(t.name.to_string()), args)
    } else {
        Type::Error
    }
}

fn verify_generics_count(
    interner: &mut NodeInterner,
    expected_count: usize,
    args: &mut Vec<Type>,
    span: Span,
    type_name: impl FnOnce() -> String,
) {
    if args.len() != expected_count {
        interner.errors.push(ResolverError::IncorrectGenericCount {
            span,
            item_name: type_name(),
            actual: args.len(),
            expected: expected_count,
        });

        // Fix the generic count so we can continue typechecking
        args.resize_with(expected_count, || Type::Error);
    }
}

fn lookup_generic_or_global_type(interner: &mut NodeInterner, path: &Path) -> Option<Type> {
    if path.segments.len() == 1 {
        let name = &path.last_segment().0.contents;
        if let Some((name, var, _)) = interner.find_generic(name) {
            return Some(Type::NamedGeneric(var.clone(), name.clone()));
        }
    }

    // If we cannot find a local generic of the same name, try to look up a global
    match interner.path_resolver.resolve(interner.def_maps, path.clone()) {
        Ok(PathResolution { module_def_id: ModuleDefId::GlobalId(id), error }) => {
            if let Some(current_item) = interner.current_item {
                interner.add_global_dependency(current_item, id);
            }

            if let Some(error) = error {
                interner.push_err(error.into());
            }
            Some(Type::Constant(interner.eval_global_as_array_length(id, path)))
        }
        _ => None,
    }
}

fn resolve_array_size(
    interner: &mut NodeInterner,
    length: Option<UnresolvedTypeExpression>,
    new_variables: &mut Generics,
) -> Type {
    match length {
        None => {
            let id = interner.next_type_variable_id();
            let typevar = TypeVariable::unbound(id);
            new_variables.push(typevar.clone());

            // 'Named'Generic is a bit of a misnomer here, we want a type variable that
            // wont be bound over but this one has no name since we do not currently
            // require users to explicitly be generic over array lengths.
            Type::NamedGeneric(typevar, Rc::new("".into()))
        }
        Some(length) => interner.convert_expression_type(length),
    }
}

fn convert_expression_type(interner: &mut NodeInterner, length: UnresolvedTypeExpression) -> Type {
    match length {
        UnresolvedTypeExpression::Variable(path) => {
            interner.lookup_generic_or_global_type(&path).unwrap_or_else(|| {
                interner.push_err(ResolverError::NoSuchNumericTypeVariable { path });
                Type::Constant(0)
            })
        }
        UnresolvedTypeExpression::Constant(int, _) => Type::Constant(int),
        UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, _) => {
            let (lhs_span, rhs_span) = (lhs.span(), rhs.span());
            let lhs = interner.convert_expression_type(*lhs);
            let rhs = interner.convert_expression_type(*rhs);

            match (lhs, rhs) {
                (Type::Constant(lhs), Type::Constant(rhs)) => {
                    Type::Constant(op.function()(lhs, rhs))
                }
                (lhs, _) => {
                    let span =
                        if !matches!(lhs, Type::Constant(_)) { lhs_span } else { rhs_span };
                    interner.push_err(ResolverError::InvalidArrayLengthExpr { span });
                    Type::Constant(0)
                }
            }
        }
    }
}

fn get_ident_from_path(interner: &mut NodeInterner, path: Path) -> (HirIdent, usize) {
    let location = Location::new(path.span(), interner.file);

    let error = match path.as_ident().map(|ident| interner.find_variable(ident)) {
        Some(Ok(found)) => return found,
        // Try to look it up as a global, but still issue the first error if we fail
        Some(Err(error)) => match interner.lookup_global(path) {
            Ok(id) => return (HirIdent::non_trait_method(id, location), 0),
            Err(_) => error,
        },
        None => match interner.lookup_global(path) {
            Ok(id) => return (HirIdent::non_trait_method(id, location), 0),
            Err(error) => error,
        },
    };
    interner.push_err(error);
    let id = DefinitionId::dummy_id();
    (HirIdent::non_trait_method(id, location), 0)
}

/// Translates an UnresolvedType to a Type
pub fn resolve_type(interner: &mut NodeInterner, typ: UnresolvedType) -> Type {
    let span = typ.span;
    let resolved_type = interner.resolve_type_inner(typ, &mut vec![]);
    if resolved_type.is_nested_slice() {
        interner.errors.push(ResolverError::NestedSlices { span: span.unwrap() });
    }

    resolved_type
}

pub fn resolve_type_alias(
    mut self,
    unresolved: NoirTypeAlias,
    alias_id: TypeAliasId,
) -> (Type, Generics, Vec<ResolverError>) {
    let generics = interner.add_generics(&unresolved.generics);
    interner.resolve_local_globals();

    interner.current_item = Some(DependencyId::Alias(alias_id));
    let typ = intern_type(interner, unresolved.typ);

    (typ, generics, interner.errors)
}

pub fn take_errors(self) -> Vec<ResolverError> {
    interner.errors
}

/// Return the current generics.
/// Needed to keep referring to the same type variables across many
/// methods in a single impl.
pub fn get_generics(interner: &NodeInterner) -> &[(Rc<String>, TypeVariable, Span)] {
    interner: &NodeInterner.generics
}

/// Set the current generics that are in scope.
/// Unlike add_generics, this function will not create any new type variables,
/// opting to reuse the existing ones it is directly given.
pub fn set_generics(interner: &mut NodeInterner, generics: Vec<(Rc<String>, TypeVariable, Span)>) {
    interner.generics = generics;
}

/// Translates a (possibly Unspecified) UnresolvedType to a Type.
/// Any UnresolvedType::Unspecified encountered are replaced with fresh type variables.
fn resolve_inferred_type(interner: &mut NodeInterner, typ: UnresolvedType) -> Type {
    match &typ.typ {
        UnresolvedTypeData::Unspecified => interner.next_type_variable(),
        _ => interner.resolve_type_inner(typ, &mut vec![]),
    }
}

/// Add the given generics to scope.
/// Each generic will have a fresh Shared<TypeBinding> associated with it.
pub fn add_generics(interner: &mut NodeInterner, generics: &UnresolvedGenerics) -> Generics {
    vecmap(generics, |generic| {
        // Map the generic to a fresh type variable
        let id = interner.next_type_variable_id();
        let typevar = TypeVariable::unbound(id);
        let span = generic.0.span();

        // Check for name collisions of this generic
        let name = Rc::new(generic.0.contents.clone());

        if let Some((_, _, first_span)) = interner.find_generic(&name) {
            interner.errors.push(ResolverError::DuplicateDefinition {
                name: generic.0.contents.clone(),
                first_span: *first_span,
                second_span: span,
            });
        } else {
            interner.generics.push((name, typevar.clone(), span));
        }

        typevar
    })
}

/// Add the given existing generics to scope.
/// This is useful for adding the same generics to many items. E.g. apply impl generics
/// to each function in the impl or trait generics to each item in the trait.
pub fn add_existing_generics(interner: &mut NodeInterner, names: &UnresolvedGenerics, generics: &Generics) {
    assert_eq!(names.len(), generics.len());

    for (name, typevar) in names.iter().zip(generics) {
        interner.add_existing_generic(&name.0.contents, name.0.span(), typevar.clone());
    }
}

pub fn add_existing_generic(interner: &mut NodeInterner, name: &str, span: Span, typevar: TypeVariable) {
    // Check for name collisions of this generic
    let rc_name = Rc::new(name.to_owned());

    if let Some((_, _, first_span)) = interner.find_generic(&rc_name) {
        interner.errors.push(ResolverError::DuplicateDefinition {
            name: name.to_owned(),
            first_span: *first_span,
            second_span: span,
        });
    } else {
        interner.generics.push((rc_name, typevar, span));
    }
}

pub fn resolve_struct_fields(
    mut self,
    unresolved: NoirStruct,
    struct_id: StructId,
) -> (Generics, Vec<(Ident, Type)>, Vec<ResolverError>) {
    let generics = interner.add_generics(&unresolved.generics);

    // Check whether the struct definition has globals in the local module and add them to the scope
    interner.resolve_local_globals();

    interner.current_item = Some(DependencyId::Struct(struct_id));

    interner.resolving_ids.insert(struct_id);
    let fields = vecmap(unresolved.fields, |(ident, typ)| (ident, intern_type(interner, typ)));
    interner.resolving_ids.remove(&struct_id);

    (generics, fields, interner.errors)
}

fn resolve_local_globals(interner: &mut NodeInterner) {
    let globals = vecmap(interner.get_all_globals(), |global| {
        (global.id, global.local_id, global.ident.clone())
    });
    for (id, local_module_id, name) in globals {
        if local_module_id == interner.path_resolver.local_module_id() {
            let definition = DefinitionKind::Global(id);
            interner.add_global_variable_decl(name, definition);
        }
    }
}

/// TODO: This is currently only respected for generic free functions
/// there's a bunch of other places where trait constraints can pop up
fn resolve_trait_constraints(
    interner: &mut NodeInterner,
    where_clause: &[UnresolvedTraitConstraint],
) -> Vec<TraitConstraint> {
    where_clause
        .iter()
        .cloned()
        .filter_map(|constraint| interner.resolve_trait_constraint(constraint))
        .collect()
}

/// Extract metadata from a NoirFunction
/// to be used in analysis and intern the function parameters
/// Prerequisite: interner.add_generics() has already been called with the given
/// function's generics, including any generics from the impl, if any.
fn extract_meta(interner: &mut NodeInterner, func: &NoirFunction, func_id: FuncId) -> FuncMeta {
    let location = Location::new(func.name_ident().span(), interner.file);
    let id = interner.function_definition_id(func_id);
    let name_ident = HirIdent::non_trait_method(id, location);

    let attributes = func.attributes().clone();
    let has_no_predicates_attribute = attributes.is_no_predicates();
    let should_fold = attributes.is_foldable();
    if !interner.inline_attribute_allowed(func) {
        if has_no_predicates_attribute {
            interner.push_err(ResolverError::NoPredicatesAttributeOnUnconstrained {
                ident: func.name_ident().clone(),
            });
        } else if should_fold {
            interner.push_err(ResolverError::FoldAttributeOnUnconstrained {
                ident: func.name_ident().clone(),
            });
        }
    }
    // Both the #[fold] and #[no_predicates] alter a function's inline type and code generation in similar ways.
    // In certain cases such as type checking (for which the following flag will be used) both attributes
    // indicate we should code generate in the same way. Thus, we unify the attributes into one flag here.
    let has_inline_attribute = has_no_predicates_attribute || should_fold;

    let mut generics = vecmap(interner: &NodeInterner.generics, |(_, typevar, _)| typevar.clone());
    let mut parameters = vec![];
    let mut parameter_types = vec![];

    for Param { visibility, pattern, typ, span: _ } in func.parameters().iter().cloned() {
        if visibility == Visibility::Public && !interner.pub_allowed(func) {
            interner.push_err(ResolverError::UnnecessaryPub {
                ident: func.name_ident().clone(),
                position: PubPosition::Parameter,
            });
        }

        let pattern = intern_pattern(interner, pattern, DefinitionKind::Local(None));
        let typ = interner.resolve_type_inner(typ, &mut generics);

        parameters.push((pattern, typ.clone(), visibility));
        parameter_types.push(typ);
    }

    let return_type = Box::new(intern_type(interner, func.return_type()));

    interner.declare_numeric_generics(&parameter_types, &return_type);

    if !interner.pub_allowed(func) && func.def.return_visibility == Visibility::Public {
        interner.push_err(ResolverError::UnnecessaryPub {
            ident: func.name_ident().clone(),
            position: PubPosition::ReturnType,
        });
    }
    let is_low_level_function =
        attributes.function.as_ref().map_or(false, |func| func.is_low_level());
    if !interner.path_resolver.module_id().krate.is_stdlib() && is_low_level_function {
        let error =
            ResolverError::LowLevelFunctionOutsideOfStdlib { ident: func.name_ident().clone() };
        interner.push_err(error);
    }

    // 'pub' is required on return types for entry point functions
    if interner.is_entry_point_function(func)
        && return_type.as_ref() != &Type::Unit
        && func.def.return_visibility == Visibility::Private
    {
        interner.push_err(ResolverError::NecessaryPub { ident: func.name_ident().clone() });
    }
    // '#[recursive]' attribute is only allowed for entry point functions
    if !interner.is_entry_point_function(func) && func.kind == FunctionKind::Recursive {
        interner.push_err(ResolverError::MisplacedRecursiveAttribute {
            ident: func.name_ident().clone(),
        });
    }

    if matches!(attributes.function, Some(FunctionAttribute::Test { .. }))
        && !parameters.is_empty()
    {
        interner.push_err(ResolverError::TestFunctionHasParameters {
            span: func.name_ident().span(),
        });
    }

    let mut typ = Type::Function(parameter_types, return_type, Box::new(Type::Unit));

    if !generics.is_empty() {
        typ = Type::Forall(generics, Box::new(typ));
    }

    interner.push_definition_type(name_ident.id, typ.clone());

    let direct_generics = func.def.generics.iter();
    let direct_generics = direct_generics
        .filter_map(|generic| interner.find_generic(&generic.0.contents))
        .map(|(name, typevar, _span)| (name.clone(), typevar.clone()))
        .collect();

    FuncMeta {
        name: name_ident,
        kind: func.kind,
        location,
        typ,
        direct_generics,
        trait_impl: interner.current_trait_impl,
        parameters: parameters.into(),
        return_type: func.def.return_type.clone(),
        return_visibility: func.def.return_visibility,
        has_body: !func.def.body.is_empty(),
        trait_constraints: interner.resolve_trait_constraints(&func.def.where_clause),
        is_entry_point: interner.is_entry_point_function(func),
        has_inline_attribute,
    }
}

/// Override whether this name resolver is within a contract or not.
/// This will affect which types are allowed as parameters to methods as well
/// as which modifiers are allowed on a function.
pub(crate) fn set_in_contract(interner: &mut NodeInterner, in_contract: bool) {
    interner.in_contract = in_contract;
}

/// True if the 'pub' keyword is allowed on parameters in this function
/// 'pub' on function parameters is only allowed for entry point functions
fn pub_allowed(interner: &NodeInterner, func: &NoirFunction) -> bool {
    interner.is_entry_point_function(func) || func.attributes().is_foldable()
}

fn is_entry_point_function(interner: &NodeInterner, func: &NoirFunction) -> bool {
    if interner.in_contract {
        func.attributes().is_contract_entry_point()
    } else {
        func.name() == MAIN_FUNCTION
    }
}

fn inline_attribute_allowed(interner: &NodeInterner, func: &NoirFunction) -> bool {
    // Inline attributes are only relevant for constrained functions
    // as all unconstrained functions are not inlined
    !func.def.is_unconstrained
}

fn declare_numeric_generics(interner: &mut NodeInterner, params: &[Type], return_type: &Type) {
    if interner.generics.is_empty() {
        return;
    }

    for (name_to_find, type_variable) in Self::find_numeric_generics(params, return_type) {
        // Declare any generics to let users use numeric generics in scope.
        // Don't issue a warning if these are unused
        //
        // We can fail to find the generic in interner.generics if it is an implicit one created
        // by the compiler. This can happen when, e.g. eliding array lengths using the slice
        // syntax [T].
        if let Some((name, _, span)) =
            interner.generics.iter().find(|(name, _, _)| name.as_ref() == &name_to_find)
        {
            let ident = Ident::new(name.to_string(), *span);
            let definition = DefinitionKind::GenericType(type_variable);
            interner.add_variable_decl_inner(ident, false, false, false, definition);
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
    interner: &mut NodeInterner,
    let_stmt: LetStatement,
    global_id: GlobalId,
) -> HirStatement {
    interner.current_item = Some(DependencyId::Global(global_id));
    let expression = intern_expression(interner, let_stmt.expression);
    let definition = DefinitionKind::Global(global_id);

    if !interner.in_contract
        && let_stmt.attributes.iter().any(|attr| matches!(attr, SecondaryAttribute::Abi(_)))
    {
        let span = let_stmt.pattern.span();
        interner.push_err(ResolverError::AbiAttributeOutsideContract { span });
    }

    if !let_stmt.comptime && matches!(let_stmt.pattern, Pattern::Mutable(..)) {
        let span = let_stmt.pattern.span();
        interner.push_err(ResolverError::MutableGlobal { span });
    }

    HirStatement::Let(HirLetStatement {
        pattern: intern_pattern(interner, let_stmt.pattern, definition),
        r#type: intern_type(interner, let_stmt.r#type),
        expression,
        attributes: let_stmt.attributes,
        comptime: let_stmt.comptime,
    })
}

pub fn resolve_stmt(interner: &mut NodeInterner, stmt: StatementKind, span: Span) -> HirStatement {
    match stmt {
        StatementKind::Let(let_stmt) => {
            let expression = intern_expression(interner, let_stmt.expression);
            let definition = DefinitionKind::Local(Some(expression));
            HirStatement::Let(HirLetStatement {
                pattern: intern_pattern(interner, let_stmt.pattern, definition),
                r#type: intern_type(interner, let_stmt.r#type),
                expression,
                attributes: let_stmt.attributes,
                comptime: let_stmt.comptime,
            })
        }
        StatementKind::Constrain(constrain_stmt) => {
            let span = constrain_stmt.0.span;
            let assert_msg_call_expr_id =
                interner.resolve_assert_message(constrain_stmt.1, span, constrain_stmt.0.clone());
            let expr_id = intern_expression(interner, constrain_stmt.0);

            HirStatement::Constrain(HirConstrainStatement(
                expr_id,
                interner.file,
                assert_msg_call_expr_id,
            ))
        }
        StatementKind::Expression(expr) => {
            HirStatement::Expression(intern_expression(interner, expr))
        }
        StatementKind::Semi(expr) => HirStatement::Semi(intern_expression(interner, expr)),
        StatementKind::Assign(assign_stmt) => {
            let identifier = intern_lvalue(interner, assign_stmt.lvalue);
            let expression = intern_expression(interner, assign_stmt.expression);
            let stmt = HirAssignStatement { lvalue: identifier, expression };
            HirStatement::Assign(stmt)
        }
        StatementKind::For(for_loop) => {
            match for_loop.range {
                ForRange::Range(start_range, end_range) => {
                    let start_range = intern_expression(interner, start_range);
                    let end_range = intern_expression(interner, end_range);
                    let (identifier, block) = (for_loop.identifier, for_loop.block);

                    interner.nested_loops += 1;

                    // TODO: For loop variables are currently mutable by default since we haven't
                    //       yet implemented syntax for them to be optionally mutable.
                    let (identifier, block) = interner.in_new_scope(|this| {
                        let decl = this.add_variable_decl(
                            identifier,
                            false,
                            true,
                            DefinitionKind::Local(None),
                        );
                        (decl, this.resolve_expression(block))
                    });

                    interner.nested_loops -= 1;

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
                    resolve_stmt(interner, for_stmt, for_loop.span)
                }
            }
        }
        StatementKind::Break => HirStatement::Break,
        StatementKind::Continue => HirStatement::Continue,
        StatementKind::Error => HirStatement::Error,
        StatementKind::Comptime(statement) => {
            let hir_statement = resolve_stmt(interner, statement.kind, statement.span);
            let statement_id = interner.push_stmt(hir_statement);
            HirStatement::Comptime(statement_id)
        }
    }
}

pub fn intern_stmt(interner: &mut NodeInterner, stmt: Statement) -> StmtId {
    let hir_stmt = resolve_stmt(interner, stmt.kind, stmt.span);
    interner.push_stmt(hir_stmt)
}

fn intern_lvalue(interner: &mut NodeInterner, lvalue: LValue) -> HirLValue {
    match lvalue {
        LValue::Ident(ident) => {
            let ident = interner.find_variable_or_default(&ident);
            interner.resolve_local_variable(ident.0.clone(), ident.1);

            HirLValue::Ident(ident.0, Type::Error)
        }
        LValue::MemberAccess { object, field_name, span } => HirLValue::MemberAccess {
            object: Box::new(intern_lvalue(interner, *object)),
            field_name,
            location: Location::new(span, interner.file),
            field_index: None,
            typ: Type::Error,
        },
        LValue::Index { array, index, span } => {
            let array = Box::new(intern_lvalue(interner, *array));
            let index = intern_expression(interner, index);
            let location = Location::new(span, interner.file);
            HirLValue::Index { array, index, location, typ: Type::Error }
        }
        LValue::Dereference(lvalue, span) => {
            let lvalue = Box::new(intern_lvalue(interner, *lvalue));
            let location = Location::new(span, interner.file);
            HirLValue::Dereference { lvalue, location, element_type: Type::Error }
        }
    }
}

fn intern_array_literal(interner: &mut NodeInterner, array_literal: ArrayLiteral) -> HirArrayLiteral {
    match array_literal {
        ArrayLiteral::Standard(elements) => {
            let elements = vecmap(elements, |elem| intern_expression(interner, elem));
            HirArrayLiteral::Standard(elements)
        }
        ArrayLiteral::Repeated { repeated_element, length } => {
            let span = length.span;
            let length =
                UnresolvedTypeExpression::from_expr(*length, span).unwrap_or_else(|error| {
                    interner.errors.push(ResolverError::ParserError(Box::new(error)));
                    UnresolvedTypeExpression::Constant(0, span)
                });

            let length = interner.convert_expression_type(length);
            let repeated_element = intern_expression(interner, *repeated_element);

            HirArrayLiteral::Repeated { repeated_element, length }
        }
    }
}

pub fn intern_expression(interner: &mut NodeInterner, expr: Expression) -> ExprId {
    let hir_expr = match expr.kind {
        ExpressionKind::Literal(literal) => HirExpression::Literal(match literal {
            Literal::Bool(b) => HirLiteral::Bool(b),
            Literal::Array(array_literal) => {
                HirLiteral::Array(intern_array_literal(interner, array_literal))
            }
            Literal::Slice(array_literal) => {
                HirLiteral::Slice(intern_array_literal(interner, array_literal))
            }
            Literal::Integer(integer, sign) => HirLiteral::Integer(integer, sign),
            Literal::Str(str) => HirLiteral::Str(str),
            Literal::RawStr(str, _) => HirLiteral::Str(str),
            Literal::FmtStr(str) => interner.resolve_fmt_str_literal(str, expr.span),
            Literal::Unit => HirLiteral::Unit,
        }),
        ExpressionKind::Variable(path) => {
            if let Some((method, constraint, assumed)) = interner.resolve_trait_generic_path(&path)
            {
                HirExpression::Ident(HirIdent {
                    location: Location::new(expr.span, interner.file),
                    id: interner.trait_method_id(method),
                    impl_kind: ImplKind::TraitMethod(method, constraint, assumed),
                })
            } else {
                // If the Path is being used as an Expression, then it is referring to a global from a separate module
                // Otherwise, then it is referring to an Identifier
                // This lookup allows support of such statements: let x = foo::bar::SOME_GLOBAL + 10;
                // If the expression is a singular indent, we search the resolver's current scope as normal.
                let (hir_ident, var_scope_index) = interner.get_ident_from_path(path);

                if hir_ident.id != DefinitionId::dummy_id() {
                    match interner.definition(hir_ident.id).kind {
                        DefinitionKind::Function(id) => {
                            if let Some(current_item) = interner.current_item {
                                interner.add_function_dependency(current_item, id);
                            }
                        }
                        DefinitionKind::Global(global_id) => {
                            if let Some(current_item) = interner.current_item {
                                interner.add_global_dependency(current_item, global_id);
                            }
                        }
                        DefinitionKind::GenericType(_) => {
                            // Initialize numeric generics to a polymorphic integer type in case
                            // they're used in expressions. We must do this here since the type
                            // checker does not check definition kinds and otherwise expects
                            // parameters to already be typed.
                            if interner.definition_type(hir_ident.id) == Type::Error {
                                let typ = Type::polymorphic_integer_or_field(interner);
                                interner.push_definition_type(hir_ident.id, typ);
                            }
                        }
                        DefinitionKind::Local(_) => {
                            // only local variables can be captured by closures.
                            interner.resolve_local_variable(hir_ident.clone(), var_scope_index);
                        }
                    }
                }

                HirExpression::Ident(hir_ident)
            }
        }
        ExpressionKind::Prefix(prefix) => {
            let operator = prefix.operator;
            let rhs = intern_expression(interner, prefix.rhs);

            if operator == UnaryOp::MutableReference {
                if let Err(error) = verify_mutable_reference(interner, rhs) {
                    interner.errors.push(error);
                }
            }

            HirExpression::Prefix(HirPrefixExpression { operator, rhs })
        }
        ExpressionKind::Infix(infix) => {
            let lhs = intern_expression(interner, infix.lhs);
            let rhs = intern_expression(interner, infix.rhs);
            let trait_id = interner.get_operator_trait_method(infix.operator.contents);

            HirExpression::Infix(HirInfixExpression {
                lhs,
                operator: HirBinaryOp::new(infix.operator, interner.file),
                trait_method_id: trait_id,
                rhs,
            })
        }
        ExpressionKind::Call(call_expr) => {
            // Get the span and name of path for error reporting
            let func = intern_expression(interner, *call_expr.func);

            let arguments = vecmap(call_expr.arguments, |arg| intern_expression(interner, arg));
            let location = Location::new(expr.span, interner.file);
            HirExpression::Call(HirCallExpression { func, arguments, location })
        }
        ExpressionKind::MethodCall(call_expr) => {
            let method = call_expr.method_name;
            let object = intern_expression(interner, call_expr.object);
            let arguments = vecmap(call_expr.arguments, |arg| intern_expression(interner, arg));
            let location = Location::new(expr.span, interner.file);
            HirExpression::MethodCall(HirMethodCallExpression {
                arguments,
                method,
                object,
                location,
            })
        }
        ExpressionKind::Cast(cast_expr) => HirExpression::Cast(HirCastExpression {
            lhs: intern_expression(interner, cast_expr.lhs),
            r#type: intern_type(interner, cast_expr.r#type),
        }),
        ExpressionKind::If(if_expr) => HirExpression::If(HirIfExpression {
            condition: intern_expression(interner, if_expr.condition),
            consequence: intern_expression(interner, if_expr.consequence),
            alternative: if_expr.alternative.map(|e| intern_expression(interner, e)),
        }),
        ExpressionKind::Index(indexed_expr) => HirExpression::Index(HirIndexExpression {
            collection: intern_expression(interner, indexed_expr.collection),
            index: intern_expression(interner, indexed_expr.index),
        }),
        ExpressionKind::Block(block_expr) => {
            HirExpression::Block(interner.resolve_block(block_expr))
        }
        ExpressionKind::Constructor(constructor) => {
            let span = constructor.type_name.span();

            match interner.lookup_type_or_error(constructor.type_name) {
                Some(Type::Struct(r#type, struct_generics)) => {
                    let typ = r#type.clone();
                    let fields = constructor.fields;
                    let resolve_expr = Resolver::resolve_expression;
                    let fields =
                        interner.resolve_constructor_fields(typ, fields, span, resolve_expr);
                    HirExpression::Constructor(HirConstructorExpression {
                        fields,
                        r#type,
                        struct_generics,
                    })
                }
                Some(typ) => {
                    interner.push_err(ResolverError::NonStructUsedInConstructor { typ, span });
                    HirExpression::Error
                }
                None => HirExpression::Error,
            }
        }
        ExpressionKind::MemberAccess(access) => {
            // Validating whether the lhs actually has the rhs as a field
            // needs to wait until type checking when we know the type of the lhs
            HirExpression::MemberAccess(HirMemberAccess {
                lhs: intern_expression(interner, access.lhs),
                rhs: access.rhs,
                // This is only used when lhs is a reference and we want to return a reference to rhs
                is_offset: false,
            })
        }
        ExpressionKind::Error => HirExpression::Error,
        ExpressionKind::Tuple(elements) => {
            let elements = vecmap(elements, |elem| intern_expression(interner, elem));
            HirExpression::Tuple(elements)
        }
        // We must stay in the same function scope as the parent function to allow for closures
        // to capture variables. This is currently limited to immutable variables.
        ExpressionKind::Lambda(lambda) => interner.in_new_scope(|this| {
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
        ExpressionKind::Parenthesized(sub_expr) => return intern_expression(interner, *sub_expr),

        // The quoted expression isn't resolved since we don't want errors if variables aren't defined
        ExpressionKind::Quote(block) => HirExpression::Quote(block),
        ExpressionKind::Comptime(block) => HirExpression::Comptime(interner.resolve_block(block)),
    };

    // If these lines are ever changed, make sure to change the early return
    // in the ExpressionKind::Variable case as well
    let expr_id = interner.push_expr(hir_expr);
    interner.push_expr_location(expr_id, expr.span, interner.file);
    expr_id
}

fn resolve_pattern(interner: &mut NodeInterner, pattern: Pattern, definition: DefinitionKind) -> HirPattern {
    interner.resolve_pattern_mutable(pattern, None, definition)
}

fn resolve_pattern_mutable(
    interner: &mut NodeInterner,
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
            let id = interner.add_variable_decl(name, mutable.is_some(), true, definition);
            HirPattern::Identifier(id)
        }
        Pattern::Mutable(pattern, span, _) => {
            if let Some(first_mut) = mutable {
                interner.push_err(ResolverError::UnnecessaryMut { first_mut, second_mut: span });
            }

            let pattern = interner.resolve_pattern_mutable(*pattern, Some(span), definition);
            let location = Location::new(span, interner.file);
            HirPattern::Mutable(Box::new(pattern), location)
        }
        Pattern::Tuple(fields, span) => {
            let fields = vecmap(fields, |field| {
                interner.resolve_pattern_mutable(field, mutable, definition.clone())
            });
            let location = Location::new(span, interner.file);
            HirPattern::Tuple(fields, location)
        }
        Pattern::Struct(name, fields, span) => {
            let error_identifier = |this: &mut NodeInterner| {
                // Must create a name here to return a HirPattern::Identifier. Allowing
                // shadowing here lets us avoid further errors if we define ERROR_IDENT
                // multiple times.
                let name = ERROR_IDENT.into();
                let identifier = this.add_variable_decl(name, false, true, definition.clone());
                HirPattern::Identifier(identifier)
            };

            let (struct_type, generics) = match interner.lookup_type_or_error(name) {
                Some(Type::Struct(struct_type, generics)) => (struct_type, generics),
                None => return error_identifier(self),
                Some(typ) => {
                    interner.push_err(ResolverError::NonStructUsedInConstructor { typ, span });
                    return error_identifier(self);
                }
            };

            let resolve_field = |this: &mut NodeInterner, pattern| {
                this.resolve_pattern_mutable(pattern, mutable, definition.clone())
            };

            let typ = struct_type.clone();
            let fields = interner.resolve_constructor_fields(typ, fields, span, resolve_field);

            let typ = Type::Struct(struct_type, generics);
            let location = Location::new(span, interner.file);
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
    interner: &mut NodeInterner,
    struct_type: Shared<StructType>,
    fields: Vec<(Ident, T)>,
    span: Span,
    mut resolve_function: impl FnMut(&mut NodeInterner, T) -> U,
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
            interner.push_err(ResolverError::DuplicateField { field: field.clone() });
        } else {
            // field not required by struct
            interner.push_err(ResolverError::NoSuchField {
                field: field.clone(),
                struct_definition: struct_type.borrow().name.clone(),
            });
        }

        ret.push((field, resolved));
    }

    if !unseen_fields.is_empty() {
        interner.push_err(ResolverError::MissingFields {
            span,
            missing_fields: unseen_fields.into_iter().map(|field| field.to_string()).collect(),
            struct_definition: struct_type.borrow().name.clone(),
        });
    }

    ret
}

pub fn get_struct(interner: &NodeInterner, type_id: StructId) -> Shared<StructType> {
    interner.get_struct(type_id)
}

pub fn get_trait_mut(interner: &mut NodeInterner, trait_id: TraitId) -> &mut Trait {
    interner.get_trait_mut(trait_id)
}

fn lookup<T: TryFromModuleDefId>(interner: &mut NodeInterner, path: Path) -> Result<T, ResolverError> {
    let span = path.span();
    let id = interner.resolve_path(path)?;
    T::try_from(id).ok_or_else(|| ResolverError::Expected {
        expected: T::description(),
        got: id.as_str().to_owned(),
        span,
    })
}

fn lookup_global(interner: &mut NodeInterner, path: Path) -> Result<DefinitionId, ResolverError> {
    let span = path.span();
    let id = interner.resolve_path(path)?;

    if let Some(function) = TryFromModuleDefId::try_from(id) {
        return Ok(interner.function_definition_id(function));
    }

    if let Some(global) = TryFromModuleDefId::try_from(id) {
        let global = interner.get_global(global);
        return Ok(global.definition_id);
    }

    let expected = "global variable".into();
    let got = "local variable".into();
    Err(ResolverError::Expected { span, expected, got })
}

/// Lookup a given struct type by name.
fn lookup_struct_or_error(interner: &mut NodeInterner, path: Path) -> Option<Shared<StructType>> {
    match interner.lookup(path) {
        Ok(struct_id) => Some(interner.get_struct(struct_id)),
        Err(error) => {
            interner.push_err(error);
            None
        }
    }
}

/// Lookup a given trait by name/path.
fn lookup_trait_or_error(interner: &mut NodeInterner, path: Path) -> Option<&mut Trait> {
    match interner.lookup(path) {
        Ok(trait_id) => Some(interner.get_trait_mut(trait_id)),
        Err(error) => {
            interner.push_err(error);
            None
        }
    }
}

/// Looks up a given type by name.
/// This will also instantiate any struct types found.
fn lookup_type_or_error(interner: &mut NodeInterner, path: Path) -> Option<Type> {
    let ident = path.as_ident();
    if ident.map_or(false, |i| i == SELF_TYPE_NAME) {
        if let Some(typ) = interner: &NodeInterner.self_type {
            return Some(typ.clone());
        }
    }

    match interner.lookup(path) {
        Ok(struct_id) => {
            let struct_type = interner.get_struct(struct_id);
            let generics = struct_type.borrow().instantiate(interner);
            Some(Type::Struct(struct_type, generics))
        }
        Err(error) => {
            interner.push_err(error);
            None
        }
    }
}

fn lookup_type_alias(interner: &mut NodeInterner, path: Path) -> Option<Shared<TypeAlias>> {
    interner.lookup(path).ok().map(|id| interner.get_type_alias(id))
}

// this resolves Self::some_static_method, inside an impl block (where we don't have a concrete self_type)
fn resolve_trait_static_method_by_self(
    interner: &mut NodeInterner,
    path: &Path,
) -> Option<(TraitMethodId, TraitConstraint, bool)> {
    let trait_id = interner.trait_id?;

    if path.kind == PathKind::Plain && path.segments.len() == 2 {
        let name = &path.segments[0].0.contents;
        let method = &path.segments[1];

        if name == SELF_TYPE_NAME {
            let the_trait = interner.get_trait(trait_id);
            let method = the_trait.find_method(method.0.contents.as_str())?;

            let constraint = TraitConstraint {
                typ: interner.self_type.clone()?,
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
    interner: &mut NodeInterner,
    path: &Path,
) -> Option<(TraitMethodId, TraitConstraint, bool)> {
    if path.kind == PathKind::Plain && path.segments.len() == 2 {
        let method = &path.segments[1];

        let mut trait_path = path.clone();
        trait_path.pop();
        let trait_id = interner.lookup(trait_path).ok()?;
        let the_trait = interner.get_trait(trait_id);

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
    interner: &mut NodeInterner,
    path: &Path,
) -> Option<(TraitMethodId, TraitConstraint, bool)> {
    if path.segments.len() != 2 {
        return None;
    }

    for UnresolvedTraitConstraint { typ, trait_bound } in interner.trait_bounds.clone() {
        if let UnresolvedTypeData::Named(constraint_path, _, _) = &typ.typ {
            // if `path` is `T::method_name`, we're looking for constraint of the form `T: SomeTrait`
            if constraint_path.segments.len() == 1
                && path.segments[0] != constraint_path.last_segment()
            {
                continue;
            }

            if let Ok(ModuleDefId::TraitId(trait_id)) =
                interner.resolve_path(trait_bound.trait_path.clone())
            {
                let the_trait = interner.get_trait(trait_id);
                if let Some(method) =
                    the_trait.find_method(path.segments.last().unwrap().0.contents.as_str())
                {
                    let constraint = TraitConstraint {
                        trait_id,
                        typ: intern_type(interner, typ.clone()),
                        trait_generics: vecmap(trait_bound.trait_generics, |typ| {
                            intern_type(interner, typ)
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
    interner: &mut NodeInterner,
    path: &Path,
) -> Option<(TraitMethodId, TraitConstraint, bool)> {
    interner.resolve_trait_static_method_by_self(path)
        .or_else(|| interner.resolve_trait_static_method(path))
        .or_else(|| interner.resolve_trait_method_by_named_generic(path))
}

fn resolve_path(interner: &mut NodeInterner, path: Path) -> Result<ModuleDefId, ResolverError> {
    let path_resolution = interner.path_resolver.resolve(interner.def_maps, path)?;

    if let Some(error) = path_resolution.error {
        interner.push_err(error.into());
    }

    Ok(path_resolution.module_def_id)
}

fn resolve_block(interner: &mut NodeInterner, block_expr: BlockExpression) -> HirBlockExpression {
    let statements =
        interner.in_new_scope(|this| vecmap(block_expr.statements, |stmt| this.intern_stmt(stmt)));
    HirBlockExpression { statements }
}

pub fn intern_block(interner: &mut NodeInterner, block: BlockExpression) -> ExprId {
    let hir_block = HirExpression::Block(interner.resolve_block(block));
    interner.push_expr(hir_block)
}

fn eval_global_as_array_length(interner: &mut NodeInterner, global: GlobalId, path: &Path) -> u64 {
    let Some(stmt) = interner.get_global_let_statement(global) else {
        let path = path.clone();
        interner.push_err(ResolverError::NoSuchNumericTypeVariable { path });
        return 0;
    };

    let length = stmt.expression;
    let span = interner.expr_span(&length);
    let result = interner.try_eval_array_length_id(length, span);

    match result.map(|length| length.try_into()) {
        Ok(Ok(length_value)) => return length_value,
        Ok(Err(_cast_err)) => interner.push_err(ResolverError::IntegerTooLarge { span }),
        Err(Some(error)) => interner.push_err(error),
        Err(None) => (),
    }
    0
}

fn try_eval_array_length_id(
    interner: &NodeInterner,
    rhs: ExprId,
    span: Span,
) -> Result<u128, Option<ResolverError>> {
    // Arbitrary amount of recursive calls to try before giving up
    let fuel = 100;
    interner.try_eval_array_length_id_with_fuel(rhs, span, fuel)
}

fn try_eval_array_length_id_with_fuel(
    interner: &NodeInterner,
    rhs: ExprId,
    span: Span,
    fuel: u32,
) -> Result<u128, Option<ResolverError>> {
    if fuel == 0 {
        // If we reach here, it is likely from evaluating cyclic globals. We expect an error to
        // be issued for them after name resolution so issue no error now.
        return Err(None);
    }

    match interner.expression(&rhs) {
        HirExpression::Literal(HirLiteral::Integer(int, false)) => {
            int.try_into_u128().ok_or(Some(ResolverError::IntegerTooLarge { span }))
        }
        HirExpression::Ident(ident) => {
            let definition = interner.definition(ident.id);
            match definition.kind {
                DefinitionKind::Global(global_id) => {
                    let let_statement = interner.get_global_let_statement(global_id);
                    if let Some(let_statement) = let_statement {
                        let expression = let_statement.expression;
                        interner.try_eval_array_length_id_with_fuel(expression, span, fuel - 1)
                    } else {
                        Err(Some(ResolverError::InvalidArrayLengthExpr { span }))
                    }
                }
                _ => Err(Some(ResolverError::InvalidArrayLengthExpr { span })),
            }
        }
        HirExpression::Infix(infix) => {
            let lhs = interner.try_eval_array_length_id_with_fuel(infix.lhs, span, fuel - 1)?;
            let rhs = interner.try_eval_array_length_id_with_fuel(infix.rhs, span, fuel - 1)?;

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

fn resolve_fmt_str_literal(interner: &mut NodeInterner, str: String, call_expr_span: Span) -> HirLiteral {
    let re = Regex::new(r"\{([a-zA-Z0-9_]+)\}")
        .expect("ICE: an invalid regex pattern was used for checking format strings");
    let mut fmt_str_idents = Vec::new();
    for field in re.find_iter(&str) {
        let matched_str = field.as_str();
        let ident_name = &matched_str[1..(matched_str.len() - 1)];

        let scope_tree = interner.scopes.current_scope_tree();
        let variable = scope_tree.find(ident_name);
        if let Some((old_value, _)) = variable {
            old_value.num_times_used += 1;
            let ident = HirExpression::Ident(old_value.ident.clone());
            let expr_id = interner.push_expr(ident);
            interner.push_expr_location(expr_id, call_expr_span, interner.file);
            fmt_str_idents.push(expr_id);
        } else if ident_name.parse::<usize>().is_ok() {
            interner.errors.push(ResolverError::NumericConstantInFormatString {
                name: ident_name.to_owned(),
                span: call_expr_span,
            });
        } else {
            interner.errors.push(ResolverError::VariableNotDeclared {
                name: ident_name.to_owned(),
                span: call_expr_span,
            });
        }
    }
    HirLiteral::FmtStr(str, fmt_str_idents)
}
