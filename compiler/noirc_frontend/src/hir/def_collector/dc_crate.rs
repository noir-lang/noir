use super::dc_mod::collect_defs;
use super::errors::{DefCollectorErrorKind, DuplicateType};
use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleId};
use crate::hir::resolution::errors::ResolverError;

use crate::hir::resolution::import::{resolve_import, ImportDirective};
use crate::hir::resolution::resolver::Resolver;
use crate::hir::resolution::{
    collect_impls, collect_trait_impls, path_resolver, resolve_free_functions, resolve_globals,
    resolve_impls, resolve_structs, resolve_trait_by_path, resolve_trait_impls, resolve_traits,
    resolve_type_aliases,
};
use crate::hir::type_check::{type_check_func, TypeCheckError, TypeChecker};
use crate::hir::Context;

use crate::macros_api::MacroProcessor;
use crate::node_interner::{FuncId, NodeInterner, StmtId, StructId, TraitId, TypeAliasId};

use crate::parser::{ParserError, SortedModule};
use crate::{
    ExpressionKind, Ident, LetStatement, Literal, NoirFunction, NoirStruct, NoirTrait,
    NoirTypeAlias, Path, PathKind, Type, TypeBindings, UnresolvedGenerics,
    UnresolvedTraitConstraint, UnresolvedType,
};
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{CustomDiagnostic, Span};
use std::collections::{BTreeMap, HashMap};

use std::vec;

/// Stores all of the unresolved functions in a particular file/mod
#[derive(Clone)]
pub struct UnresolvedFunctions {
    pub file_id: FileId,
    pub functions: Vec<(LocalModuleId, FuncId, NoirFunction)>,
    pub trait_id: Option<TraitId>,
}

impl UnresolvedFunctions {
    pub fn push_fn(&mut self, mod_id: LocalModuleId, func_id: FuncId, func: NoirFunction) {
        self.functions.push((mod_id, func_id, func));
    }

    pub fn resolve_trait_bounds_trait_ids(
        &mut self,
        def_maps: &BTreeMap<CrateId, CrateDefMap>,
        crate_id: CrateId,
    ) -> Vec<DefCollectorErrorKind> {
        let mut errors = Vec::new();

        for (local_id, _, func) in &mut self.functions {
            let module = ModuleId { krate: crate_id, local_id: *local_id };

            for bound in &mut func.def.where_clause {
                match resolve_trait_by_path(def_maps, module, bound.trait_bound.trait_path.clone())
                {
                    Ok(trait_id) => {
                        bound.trait_bound.trait_id = Some(trait_id);
                    }
                    Err(err) => {
                        errors.push(err);
                    }
                }
            }
        }

        errors
    }
}

pub struct UnresolvedStruct {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub struct_def: NoirStruct,
}

#[derive(Clone)]
pub struct UnresolvedTrait {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub crate_id: CrateId,
    pub trait_def: NoirTrait,
    pub method_ids: HashMap<String, FuncId>,
    pub fns_with_default_impl: UnresolvedFunctions,
}

pub struct UnresolvedTraitImpl {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub trait_id: Option<TraitId>,
    pub trait_generics: Vec<UnresolvedType>,
    pub trait_path: Path,
    pub object_type: UnresolvedType,
    pub methods: UnresolvedFunctions,
    pub generics: UnresolvedGenerics,
    pub where_clause: Vec<UnresolvedTraitConstraint>,
}

#[derive(Clone)]
pub struct UnresolvedTypeAlias {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub type_alias_def: NoirTypeAlias,
}

#[derive(Clone)]
pub struct UnresolvedGlobal {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub stmt_id: StmtId,
    pub stmt_def: LetStatement,
}

/// Given a Crate root, collect all definitions in that crate
pub struct DefCollector {
    pub(crate) def_map: CrateDefMap,
    pub(crate) collected_imports: Vec<ImportDirective>,
    pub(crate) collected_functions: Vec<UnresolvedFunctions>,
    pub(crate) collected_types: BTreeMap<StructId, UnresolvedStruct>,
    pub(crate) collected_type_aliases: BTreeMap<TypeAliasId, UnresolvedTypeAlias>,
    pub(crate) collected_traits: BTreeMap<TraitId, UnresolvedTrait>,
    pub(crate) collected_globals: Vec<UnresolvedGlobal>,
    pub(crate) collected_impls: ImplMap,
    pub(crate) collected_traits_impls: Vec<UnresolvedTraitImpl>,
}

/// Maps the type and the module id in which the impl is defined to the functions contained in that
/// impl along with the generics declared on the impl itself. This also contains the Span
/// of the object_type of the impl, used to issue an error if the object type fails to resolve.
///
/// Note that because these are keyed by unresolved types, the impl map is one of the few instances
/// of HashMap rather than BTreeMap. For this reason, we should be careful not to iterate over it
/// since it would be non-deterministic.
pub(crate) type ImplMap =
    HashMap<(UnresolvedType, LocalModuleId), Vec<(UnresolvedGenerics, Span, UnresolvedFunctions)>>;

#[derive(Debug, Clone)]
pub enum CompilationError {
    ParseError(ParserError),
    DefinitionError(DefCollectorErrorKind),
    ResolverError(ResolverError),
    TypeError(TypeCheckError),
}

impl From<CompilationError> for CustomDiagnostic {
    fn from(value: CompilationError) -> Self {
        match value {
            CompilationError::ParseError(error) => error.into(),
            CompilationError::DefinitionError(error) => error.into(),
            CompilationError::ResolverError(error) => error.into(),
            CompilationError::TypeError(error) => error.into(),
        }
    }
}

impl From<ParserError> for CompilationError {
    fn from(value: ParserError) -> Self {
        CompilationError::ParseError(value)
    }
}

impl From<DefCollectorErrorKind> for CompilationError {
    fn from(value: DefCollectorErrorKind) -> Self {
        CompilationError::DefinitionError(value)
    }
}

impl From<ResolverError> for CompilationError {
    fn from(value: ResolverError) -> Self {
        CompilationError::ResolverError(value)
    }
}
impl From<TypeCheckError> for CompilationError {
    fn from(value: TypeCheckError) -> Self {
        CompilationError::TypeError(value)
    }
}

impl DefCollector {
    fn new(def_map: CrateDefMap) -> DefCollector {
        DefCollector {
            def_map,
            collected_imports: vec![],
            collected_functions: vec![],
            collected_types: BTreeMap::new(),
            collected_type_aliases: BTreeMap::new(),
            collected_traits: BTreeMap::new(),
            collected_impls: HashMap::new(),
            collected_globals: vec![],
            collected_traits_impls: vec![],
        }
    }

    /// Collect all of the definitions in a given crate into a CrateDefMap
    /// Modules which are not a part of the module hierarchy starting with
    /// the root module, will be ignored.
    pub fn collect(
        mut def_map: CrateDefMap,
        context: &mut Context,
        ast: SortedModule,
        root_file_id: FileId,
        macro_processors: Vec<&dyn MacroProcessor>,
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
        let crate_id = def_map.krate;

        // Recursively resolve the dependencies
        //
        // Dependencies are fetched from the crate graph
        // Then added these to the context of DefMaps once they are resolved
        //
        let crate_graph = &context.crate_graph[crate_id];

        for dep in crate_graph.dependencies.clone() {
            errors.extend(CrateDefMap::collect_defs(
                dep.crate_id,
                context,
                macro_processors.clone(),
            ));

            let dep_def_root =
                context.def_map(&dep.crate_id).expect("ice: def map was just created").root;
            let module_id = ModuleId { krate: dep.crate_id, local_id: dep_def_root };
            // Add this crate as a dependency by linking it's root module
            def_map.extern_prelude.insert(dep.as_name(), module_id);
        }

        // At this point, all dependencies are resolved and type checked.
        //
        // It is now possible to collect all of the definitions of this crate.
        let crate_root = def_map.root;
        let mut def_collector = DefCollector::new(def_map);

        // Collecting module declarations with ModCollector
        // and lowering the functions
        // i.e. Use a mod collector to collect the nodes at the root module
        // and process them
        errors.extend(collect_defs(
            &mut def_collector,
            ast,
            root_file_id,
            crate_root,
            crate_id,
            context,
        ));

        let submodules = vecmap(def_collector.def_map.modules().iter(), |(index, _)| index);
        // Add the current crate to the collection of DefMaps
        context.def_maps.insert(crate_id, def_collector.def_map);

        inject_prelude(crate_id, context, crate_root, &mut def_collector.collected_imports);
        for submodule in submodules {
            inject_prelude(
                crate_id,
                context,
                LocalModuleId(submodule),
                &mut def_collector.collected_imports,
            );
        }

        // Resolve unresolved imports collected from the crate, one by one.
        for collected_import in def_collector.collected_imports {
            match resolve_import(crate_id, collected_import, &context.def_maps) {
                Ok(resolved_import) => {
                    // Populate module namespaces according to the imports used
                    let current_def_map = context.def_maps.get_mut(&crate_id).unwrap();

                    let name = resolved_import.name;
                    for ns in resolved_import.resolved_namespace.iter_defs() {
                        let result = current_def_map.modules[resolved_import.module_scope.0]
                            .import(name.clone(), ns, resolved_import.is_prelude);

                        if let Err((first_def, second_def)) = result {
                            let err = DefCollectorErrorKind::Duplicate {
                                typ: DuplicateType::Import,
                                first_def,
                                second_def,
                            };
                            errors.push((err.into(), root_file_id));
                        }
                    }
                }
                Err((error, module_id)) => {
                    let current_def_map = context.def_maps.get(&crate_id).unwrap();
                    let file_id = current_def_map.file_id(module_id);
                    let error = DefCollectorErrorKind::PathResolutionError(error);
                    errors.push((error.into(), file_id));
                }
            }
        }

        // We must first resolve and intern the globals before we can resolve any stmts inside each function.
        // Each function uses its own resolver with a newly created ScopeForest, and must be resolved again to be within a function's scope
        //
        // Additionally, we must resolve integer globals before structs since structs may refer to
        // the values of integer globals as numeric generics.
        let (literal_globals, other_globals) =
            filter_literal_globals(def_collector.collected_globals);

        let mut resolved_globals = resolve_globals(context, literal_globals, crate_id);

        errors.extend(resolve_type_aliases(
            context,
            def_collector.collected_type_aliases,
            crate_id,
        ));

        errors.extend(resolve_traits(context, def_collector.collected_traits, crate_id));
        // Must resolve structs before we resolve globals.
        errors.extend(resolve_structs(context, def_collector.collected_types, crate_id));

        // We must wait to resolve non-integer globals until after we resolve structs since structs
        // globals will need to reference the struct type they're initialized to to ensure they are valid.
        resolved_globals.extend(resolve_globals(context, other_globals, crate_id));

        // Bind trait impls to their trait. Collect trait functions, that have a
        // default implementation, which hasn't been overridden.
        errors.extend(collect_trait_impls(
            context,
            crate_id,
            &mut def_collector.collected_traits_impls,
        ));

        // Before we resolve any function symbols we must go through our impls and
        // re-collect the methods within into their proper module. This cannot be
        // done before resolution since we need to be able to resolve the type of the
        // impl since that determines the module we should collect into.
        //
        // These are resolved after trait impls so that struct methods are chosen
        // over trait methods if there are name conflicts.
        errors.extend(collect_impls(context, crate_id, &def_collector.collected_impls));

        // Lower each function in the crate. This is now possible since imports have been resolved
        let file_func_ids = resolve_free_functions(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_functions,
            None,
            &mut errors,
        );

        let file_method_ids = resolve_impls(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_impls,
            &mut errors,
        );
        let file_trait_impls_ids = resolve_trait_impls(
            context,
            def_collector.collected_traits_impls,
            crate_id,
            &mut errors,
        );

        errors.extend(resolved_globals.errors);

        for macro_processor in macro_processors {
            macro_processor.process_typed_ast(&crate_id, context);
        }
        errors.extend(type_check_globals(&mut context.def_interner, resolved_globals.globals));

        // Type check all of the functions in the crate
        errors.extend(type_check_functions(&mut context.def_interner, file_func_ids));
        errors.extend(type_check_functions(&mut context.def_interner, file_method_ids));
        errors.extend(type_check_functions(&mut context.def_interner, file_trait_impls_ids));
        errors
    }
}

fn inject_prelude(
    crate_id: CrateId,
    context: &Context,
    crate_root: LocalModuleId,
    collected_imports: &mut Vec<ImportDirective>,
) {
    let segments: Vec<_> = "std::prelude"
        .split("::")
        .map(|segment| crate::Ident::new(segment.into(), Span::default()))
        .collect();

    let path =
        Path { segments: segments.clone(), kind: crate::PathKind::Dep, span: Span::default() };

    if !crate_id.is_stdlib() {
        if let Ok(module_def) = path_resolver::resolve_path(
            &context.def_maps,
            ModuleId { krate: crate_id, local_id: crate_root },
            path,
        ) {
            let module_id = module_def.as_module().expect("std::prelude should be a module");
            let prelude = context.module(module_id).scope().names();

            for path in prelude {
                let mut segments = segments.clone();
                segments.push(Ident::new(path.to_string(), Span::default()));

                collected_imports.insert(
                    0,
                    ImportDirective {
                        module_id: crate_root,
                        path: Path { segments, kind: PathKind::Dep, span: Span::default() },
                        alias: None,
                        is_prelude: true,
                    },
                );
            }
        }
    }
}

/// Separate the globals Vec into two. The first element in the tuple will be the
/// literal globals, except for arrays, and the second will be all other globals.
/// We exclude array literals as they can contain complex types
fn filter_literal_globals(
    globals: Vec<UnresolvedGlobal>,
) -> (Vec<UnresolvedGlobal>, Vec<UnresolvedGlobal>) {
    globals.into_iter().partition(|global| match &global.stmt_def.expression.kind {
        ExpressionKind::Literal(literal) => !matches!(literal, Literal::Array(_)),
        _ => false,
    })
}

fn type_check_globals(
    interner: &mut NodeInterner,
    global_ids: Vec<(FileId, StmtId)>,
) -> Vec<(CompilationError, fm::FileId)> {
    global_ids
        .iter()
        .flat_map(|(file_id, stmt_id)| {
            TypeChecker::check_global(stmt_id, interner)
                .iter()
                .cloned()
                .map(|e| (e.into(), *file_id))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn type_check_functions(
    interner: &mut NodeInterner,
    file_func_ids: Vec<(FileId, FuncId)>,
) -> Vec<(CompilationError, fm::FileId)> {
    file_func_ids
        .iter()
        .flat_map(|(file, func)| {
            type_check_func(interner, *func)
                .iter()
                .cloned()
                .map(|e| (e.into(), *file))
                .collect::<Vec<_>>()
        })
        .collect()
}

// TODO(vitkov): Move this out of here and into type_check
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_methods_signatures(
    resolver: &mut Resolver,
    impl_methods: &[(FileId, FuncId)],
    trait_id: TraitId,
    trait_name_span: Span,
    // These are the generics on the trait itself from the impl.
    // E.g. in `impl Foo<A, B> for Bar<B, C>`, this is `vec![A, B]`.
    trait_generics: Vec<UnresolvedType>,
    trait_impl_generic_count: usize,
    file_id: FileId,
    errors: &mut Vec<(CompilationError, FileId)>,
) {
    let self_type = resolver.get_self_type().expect("trait impl must have a Self type").clone();
    let trait_generics = vecmap(trait_generics, |typ| resolver.resolve_type(typ));

    // Temporarily bind the trait's Self type to self_type so we can type check
    let the_trait = resolver.interner.get_trait_mut(trait_id);
    the_trait.self_type_typevar.bind(self_type);

    if trait_generics.len() != the_trait.generics.len() {
        let error = DefCollectorErrorKind::MismatchGenericCount {
            actual_generic_count: trait_generics.len(),
            expected_generic_count: the_trait.generics.len(),
            // Preferring to use 'here' over a more precise term like 'this reference'
            // to try to make the error easier to understand for newer users.
            location: "here it",
            origin: the_trait.name.to_string(),
            span: trait_name_span,
        };
        errors.push((error.into(), file_id));
    }

    // We also need to bind the traits generics to the trait's generics on the impl
    for (generic, binding) in the_trait.generics.iter().zip(trait_generics) {
        generic.bind(binding);
    }

    // Temporarily take the trait's methods so we can use both them and a mutable reference
    // to the interner within the loop.
    let trait_methods = std::mem::take(&mut the_trait.methods);

    for (file_id, func_id) in impl_methods {
        let func_name = resolver.interner.function_name(func_id).to_owned();

        // This is None in the case where the impl block has a method that's not part of the trait.
        // If that's the case, a `MethodNotInTrait` error has already been thrown, and we can ignore
        // the impl method, since there's nothing in the trait to match its signature against.
        if let Some(trait_method) =
            trait_methods.iter().find(|method| method.name.0.contents == func_name)
        {
            let impl_method = resolver.interner.function_meta(func_id);

            let impl_method_generic_count =
                impl_method.typ.generic_count() - trait_impl_generic_count;

            // We subtract 1 here to account for the implicit generic `Self` type that is on all
            // traits (and thus trait methods) but is not required (or allowed) for users to specify.
            let the_trait = resolver.interner.get_trait(trait_id);
            let trait_method_generic_count =
                trait_method.generics().len() - 1 - the_trait.generics.len();

            if impl_method_generic_count != trait_method_generic_count {
                let trait_name = resolver.interner.get_trait(trait_id).name.clone();

                let error = DefCollectorErrorKind::MismatchGenericCount {
                    actual_generic_count: impl_method_generic_count,
                    expected_generic_count: trait_method_generic_count,
                    origin: format!("{}::{}", trait_name, func_name),
                    location: "this method",
                    span: impl_method.location.span,
                };
                errors.push((error.into(), *file_id));
            }

            // This instantiation is technically not needed. We could bind each generic in the
            // trait function to the impl's corresponding generic but to do so we'd have to rely
            // on the trait function's generics being first in the generic list, since the same
            // list also contains the generic `Self` variable, and any generics on the trait itself.
            //
            // Instantiating the impl method's generics here instead is a bit less precise but
            // doesn't rely on any orderings that may be changed.
            let impl_function_type = impl_method.typ.instantiate(resolver.interner).0;

            let mut bindings = TypeBindings::new();
            let mut typecheck_errors = Vec::new();

            if let Type::Function(impl_params, impl_return, _) = impl_function_type.as_monotype() {
                if trait_method.arguments().len() != impl_params.len() {
                    let error = DefCollectorErrorKind::MismatchTraitImplementationNumParameters {
                        actual_num_parameters: impl_method.parameters.0.len(),
                        expected_num_parameters: trait_method.arguments().len(),
                        trait_name: resolver.interner.get_trait(trait_id).name.to_string(),
                        method_name: func_name.to_string(),
                        span: impl_method.location.span,
                    };
                    errors.push((error.into(), *file_id));
                }

                // Check the parameters of the impl method against the parameters of the trait method
                let args = trait_method.arguments().iter();
                let args_and_params = args.zip(impl_params).zip(&impl_method.parameters.0);

                for (parameter_index, ((expected, actual), (hir_pattern, _, _))) in
                    args_and_params.enumerate()
                {
                    if expected.try_unify(actual, &mut bindings).is_err() {
                        typecheck_errors.push(TypeCheckError::TraitMethodParameterTypeMismatch {
                            method_name: func_name.to_string(),
                            expected_typ: expected.to_string(),
                            actual_typ: actual.to_string(),
                            parameter_span: hir_pattern.span(),
                            parameter_index: parameter_index + 1,
                        });
                    }
                }

                if trait_method.return_type().try_unify(impl_return, &mut bindings).is_err() {
                    let impl_method = resolver.interner.function_meta(func_id);
                    let ret_type_span = impl_method.return_type.get_type().span;
                    let expr_span = ret_type_span.expect("return type must always have a span");

                    let expected_typ = trait_method.return_type().to_string();
                    let expr_typ = impl_method.return_type().to_string();
                    let error = TypeCheckError::TypeMismatch { expr_typ, expected_typ, expr_span };
                    typecheck_errors.push(error);
                }
            } else {
                unreachable!(
                    "impl_function_type is not a function type, it is: {impl_function_type}"
                );
            }

            errors.extend(typecheck_errors.iter().cloned().map(|e| (e.into(), *file_id)));
        }
    }

    // Now unbind `Self` and the trait's generics
    let the_trait = resolver.interner.get_trait_mut(trait_id);
    the_trait.set_methods(trait_methods);
    the_trait.self_type_typevar.unbind(the_trait.self_type_typevar_id);

    for generic in &the_trait.generics {
        generic.unbind(generic.id());
    }
}
