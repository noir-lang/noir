use std::borrow::{Borrow, BorrowMut};
use std::vec;

use convert_case::{Case, Casing};
use iter_extended::vecmap;
use noirc_frontend::macros_api::FieldElement;
use noirc_frontend::macros_api::{
    BlockExpression, CallExpression, CastExpression, Distinctness, Expression, ExpressionKind,
    ForLoopStatement, ForRange, FunctionDefinition, FunctionReturnType, FunctionVisibility,
    HirContext, HirExpression, HirLiteral, HirStatement, Ident, ImportStatement, IndexExpression,
    LetStatement, Literal, MemberAccessExpression, MethodCallExpression, NoirFunction, NoirStruct,
    Param, Path, PathKind, Pattern, PrefixExpression, SecondaryAttribute, Signedness, Span,
    Statement, StatementKind, StructType, Type, TypeImpl, UnaryOp, UnresolvedType,
    UnresolvedTypeData, Visibility,
};
use noirc_frontend::macros_api::{CrateId, FileId};
use noirc_frontend::macros_api::{MacroError, MacroProcessor};
use noirc_frontend::macros_api::{ModuleDefId, NodeInterner, SortedModule, StructId};
use noirc_frontend::node_interner::{TraitId, TraitImplKind};
use noirc_frontend::Lambda;

pub struct AztecMacro;

impl MacroProcessor for AztecMacro {
    fn process_untyped_ast(
        &self,
        ast: SortedModule,
        crate_id: &CrateId,
        context: &HirContext,
    ) -> Result<SortedModule, (MacroError, FileId)> {
        transform(ast, crate_id, context)
    }

    fn process_typed_ast(
        &self,
        crate_id: &CrateId,
        context: &mut HirContext,
    ) -> Result<(), (MacroError, FileId)> {
        transform_hir(crate_id, context).map_err(|(err, file_id)| (err.into(), file_id))
    }
}

const FUNCTION_TREE_HEIGHT: u32 = 5;
const MAX_CONTRACT_FUNCTIONS: usize = 2_usize.pow(FUNCTION_TREE_HEIGHT);

#[derive(Debug, Clone)]
pub enum AztecMacroError {
    AztecDepNotFound,
    ComputeNoteHashAndNullifierNotFound { span: Span },
    ContractHasTooManyFunctions { span: Span },
    ContractConstructorMissing { span: Span },
    UnsupportedFunctionArgumentType { span: Span, typ: UnresolvedTypeData },
    UnsupportedStorageType { span: Option<Span>, typ: UnresolvedTypeData },
    CouldNotAssignStorageSlots { secondary_message: Option<String> },
    EventError { span: Span, message: String },
}

impl From<AztecMacroError> for MacroError {
    fn from(err: AztecMacroError) -> Self {
        match err {
            AztecMacroError::AztecDepNotFound {} => MacroError {
                primary_message: "Aztec dependency not found. Please add aztec as a dependency in your Cargo.toml. For more information go to https://docs.aztec.network/developers/debugging/aztecnr-errors#aztec-dependency-not-found-please-add-aztec-as-a-dependency-in-your-nargotoml".to_owned(),
                secondary_message: None,
                span: None,
            },
            AztecMacroError::ComputeNoteHashAndNullifierNotFound { span } => MacroError {
                primary_message: "compute_note_hash_and_nullifier function not found. Define it in your contract. For more information go to https://docs.aztec.network/developers/debugging/aztecnr-errors#compute_note_hash_and_nullifier-function-not-found-define-it-in-your-contract".to_owned(),
                secondary_message: None,
                span: Some(span),
            },
            AztecMacroError::ContractHasTooManyFunctions { span } => MacroError {
                primary_message: format!("Contract can only have a maximum of {} functions", MAX_CONTRACT_FUNCTIONS),
                secondary_message: None,
                span: Some(span),
            },
            AztecMacroError::ContractConstructorMissing { span } => MacroError {
                primary_message: "Contract must have a constructor function".to_owned(),
                secondary_message: None,
                span: Some(span),
            },
            AztecMacroError::UnsupportedFunctionArgumentType { span, typ } => MacroError {
                primary_message: format!("Provided parameter type `{typ:?}` is not supported in Aztec contract interface"),
                secondary_message: None,
                span: Some(span),
            },
            AztecMacroError::UnsupportedStorageType { span, typ } => MacroError {
                primary_message: format!("Provided storage type `{typ:?}` is not directly supported in Aztec. Please provide a custom storage implementation"),
                secondary_message: None,
                span,
            },
            AztecMacroError::CouldNotAssignStorageSlots { secondary_message } => MacroError {
                primary_message: "Could not assign storage slots, please provide a custom storage implementation".to_string(),
                secondary_message,
                span: None,
            },
            AztecMacroError::EventError { span, message } => MacroError {
                primary_message: message,
                secondary_message: None,
                span: Some(span),
            },
        }
    }
}

//
//             Helper macros for creating noir ast nodes
//
fn ident(name: &str) -> Ident {
    Ident::new(name.to_string(), Span::default())
}

fn ident_path(name: &str) -> Path {
    Path::from_ident(ident(name))
}

fn path(ident: Ident) -> Path {
    Path::from_ident(ident)
}

fn expression(kind: ExpressionKind) -> Expression {
    Expression::new(kind, Span::default())
}

fn variable(name: &str) -> Expression {
    expression(ExpressionKind::Variable(ident_path(name)))
}

fn variable_ident(identifier: Ident) -> Expression {
    expression(ExpressionKind::Variable(path(identifier)))
}

fn variable_path(path: Path) -> Expression {
    expression(ExpressionKind::Variable(path))
}

fn method_call(object: Expression, method_name: &str, arguments: Vec<Expression>) -> Expression {
    expression(ExpressionKind::MethodCall(Box::new(MethodCallExpression {
        object,
        method_name: ident(method_name),
        arguments,
    })))
}

fn call(func: Expression, arguments: Vec<Expression>) -> Expression {
    expression(ExpressionKind::Call(Box::new(CallExpression { func: Box::new(func), arguments })))
}

fn pattern(name: &str) -> Pattern {
    Pattern::Identifier(ident(name))
}

fn mutable(name: &str) -> Pattern {
    Pattern::Mutable(Box::new(pattern(name)), Span::default(), true)
}

fn mutable_assignment(name: &str, assigned_to: Expression) -> Statement {
    make_statement(StatementKind::Let(LetStatement {
        pattern: mutable(name),
        r#type: make_type(UnresolvedTypeData::Unspecified),
        expression: assigned_to,
    }))
}

fn mutable_reference(variable_name: &str) -> Expression {
    expression(ExpressionKind::Prefix(Box::new(PrefixExpression {
        operator: UnaryOp::MutableReference,
        rhs: variable(variable_name),
    })))
}

fn assignment(name: &str, assigned_to: Expression) -> Statement {
    make_statement(StatementKind::Let(LetStatement {
        pattern: pattern(name),
        r#type: make_type(UnresolvedTypeData::Unspecified),
        expression: assigned_to,
    }))
}

fn member_access(lhs: &str, rhs: &str) -> Expression {
    expression(ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
        lhs: variable(lhs),
        rhs: ident(rhs),
    })))
}

fn return_type(path: Path) -> FunctionReturnType {
    let ty = make_type(UnresolvedTypeData::Named(path, vec![], true));
    FunctionReturnType::Ty(ty)
}

fn lambda(parameters: Vec<(Pattern, UnresolvedType)>, body: Expression) -> Expression {
    expression(ExpressionKind::Lambda(Box::new(Lambda {
        parameters,
        return_type: UnresolvedType {
            typ: UnresolvedTypeData::Unspecified,
            span: Some(Span::default()),
        },
        body,
    })))
}

macro_rules! chained_path {
    ( $base:expr ) => {
        {
            ident_path($base)
        }
    };
    ( $base:expr $(, $tail:expr)* ) => {
        {
            let mut base_path = ident_path($base);
            $(
                base_path.segments.push(ident($tail));
            )*
            base_path
        }
    }
}

macro_rules! chained_dep {
    ( $base:expr $(, $tail:expr)* ) => {
        {
            let mut base_path = ident_path($base);
            base_path.kind = PathKind::Dep;
            $(
                base_path.segments.push(ident($tail));
            )*
            base_path
        }
    }
}

fn cast(lhs: Expression, ty: UnresolvedTypeData) -> Expression {
    expression(ExpressionKind::Cast(Box::new(CastExpression { lhs, r#type: make_type(ty) })))
}

fn make_type(typ: UnresolvedTypeData) -> UnresolvedType {
    UnresolvedType { typ, span: Some(Span::default()) }
}

fn index_array(array: Ident, index: &str) -> Expression {
    expression(ExpressionKind::Index(Box::new(IndexExpression {
        collection: variable_path(path(array)),
        index: variable(index),
    })))
}

fn index_array_variable(array: Expression, index: &str) -> Expression {
    expression(ExpressionKind::Index(Box::new(IndexExpression {
        collection: array,
        index: variable(index),
    })))
}

fn import(path: Path) -> ImportStatement {
    ImportStatement { path, alias: None }
}

//
//                    Create AST Nodes for Aztec
//

/// Traverses every function in the ast, calling `transform_function` which
/// determines if further processing is required
fn transform(
    mut ast: SortedModule,
    crate_id: &CrateId,
    context: &HirContext,
) -> Result<SortedModule, (MacroError, FileId)> {
    // Usage -> mut ast -> aztec_library::transform(&mut ast)

    // Covers all functions in the ast
    for submodule in ast.submodules.iter_mut().filter(|submodule| submodule.is_contract) {
        if transform_module(&mut submodule.contents, crate_id, context)
            .map_err(|(err, file_id)| (err.into(), file_id))?
        {
            check_for_aztec_dependency(crate_id, context)?;
            include_relevant_imports(&mut submodule.contents);
        }
    }
    Ok(ast)
}

//
//                    Transform Hir Nodes for Aztec
//

/// Completes the Hir with data gathered from type resolution
fn transform_hir(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    transform_events(crate_id, context)?;
    assign_storage_slots(crate_id, context)
}

/// Includes an import to the aztec library if it has not been included yet
fn include_relevant_imports(ast: &mut SortedModule) {
    // Create the aztec import path using the assumed chained_dep! macro
    let aztec_import_path = import(chained_dep!("aztec"));

    // Check if the aztec import already exists
    let is_aztec_imported =
        ast.imports.iter().any(|existing_import| existing_import.path == aztec_import_path.path);

    // If aztec is not imported, add the import at the beginning
    if !is_aztec_imported {
        ast.imports.insert(0, aztec_import_path);
    }
}

/// Creates an error alerting the user that they have not downloaded the Aztec-noir library
fn check_for_aztec_dependency(
    crate_id: &CrateId,
    context: &HirContext,
) -> Result<(), (MacroError, FileId)> {
    let crate_graph = &context.crate_graph[crate_id];
    let has_aztec_dependency = crate_graph.dependencies.iter().any(|dep| dep.as_name() == "aztec");
    if has_aztec_dependency {
        Ok(())
    } else {
        Err((AztecMacroError::AztecDepNotFound.into(), crate_graph.root_file_id))
    }
}

// Check to see if the user has defined a storage struct
fn check_for_storage_definition(module: &SortedModule) -> bool {
    module.types.iter().any(|r#struct| r#struct.name.0.contents == "Storage")
}

// Check to see if the user has defined a storage struct
fn check_for_storage_implementation(module: &SortedModule) -> bool {
    module.impls.iter().any(|r#impl| match &r#impl.object_type.typ {
        UnresolvedTypeData::Named(path, _, _) => {
            path.segments.last().is_some_and(|segment| segment.0.contents == "Storage")
        }
        _ => false,
    })
}

// Check if "compute_note_hash_and_nullifier(AztecAddress,Field,Field,Field,[Field; N]) -> [Field; 4]" is defined
fn check_for_compute_note_hash_and_nullifier_definition(module: &SortedModule) -> bool {
    module.functions.iter().any(|func| {
        func.def.name.0.contents == "compute_note_hash_and_nullifier"
                && func.def.parameters.len() == 5
                && match &func.def.parameters[0].typ.typ {
                    UnresolvedTypeData::Named(path, _, _) => path.segments.last().unwrap().0.contents == "AztecAddress",
                    _ => false,
                }
                && func.def.parameters[1].typ.typ == UnresolvedTypeData::FieldElement
                && func.def.parameters[2].typ.typ == UnresolvedTypeData::FieldElement
                && func.def.parameters[3].typ.typ == UnresolvedTypeData::FieldElement
                // checks if the 5th parameter is an array and the Box<UnresolvedType> in
                // Array(Option<UnresolvedTypeExpression>, Box<UnresolvedType>) contains only fields
                && match &func.def.parameters[4].typ.typ {
                    UnresolvedTypeData::Array(_, inner_type) => {
                        matches!(inner_type.typ, UnresolvedTypeData::FieldElement)
                    },
                    _ => false,
                }
                // We check the return type the same way as we did the 5th parameter
                && match &func.def.return_type {
                    FunctionReturnType::Default(_) => false,
                    FunctionReturnType::Ty(unresolved_type) => {
                        match &unresolved_type.typ {
                            UnresolvedTypeData::Array(_, inner_type) => {
                                matches!(inner_type.typ, UnresolvedTypeData::FieldElement)
                            },
                            _ => false,
                        }
                    }
                }
    })
}

/// Checks if an attribute is a custom attribute with a specific name
fn is_custom_attribute(attr: &SecondaryAttribute, attribute_name: &str) -> bool {
    if let SecondaryAttribute::Custom(custom_attr) = attr {
        custom_attr.as_str() == attribute_name
    } else {
        false
    }
}

/// Determines if ast nodes are annotated with aztec attributes.
/// For annotated functions it calls the `transform` function which will perform the required transformations.
/// Returns true if an annotated node is found, false otherwise
fn transform_module(
    module: &mut SortedModule,
    crate_id: &CrateId,
    context: &HirContext,
) -> Result<bool, (AztecMacroError, FileId)> {
    let mut has_transformed_module = false;

    // Check for a user defined storage struct
    let storage_defined = check_for_storage_definition(module);
    let storage_implemented = check_for_storage_implementation(module);

    let crate_graph = &context.crate_graph[crate_id];

    if storage_defined && !storage_implemented {
        generate_storage_implementation(module).map_err(|err| (err, crate_graph.root_file_id))?;
    }

    if storage_defined && !check_for_compute_note_hash_and_nullifier_definition(module) {
        return Err((
            AztecMacroError::ComputeNoteHashAndNullifierNotFound { span: Span::default() },
            crate_graph.root_file_id,
        ));
    }

    for structure in module.types.iter() {
        if structure.attributes.iter().any(|attr| matches!(attr, SecondaryAttribute::Event)) {
            module.impls.push(generate_selector_impl(structure));
            has_transformed_module = true;
        }
    }

    for func in module.functions.iter_mut() {
        for secondary_attribute in func.def.attributes.secondary.clone() {
            let crate_graph = &context.crate_graph[crate_id];
            if is_custom_attribute(&secondary_attribute, "aztec(private)") {
                transform_function("Private", func, storage_defined)
                    .map_err(|err| (err, crate_graph.root_file_id))?;
                has_transformed_module = true;
            } else if is_custom_attribute(&secondary_attribute, "aztec(public)") {
                transform_function("Public", func, storage_defined)
                    .map_err(|err| (err, crate_graph.root_file_id))?;
                has_transformed_module = true;
            } else if is_custom_attribute(&secondary_attribute, "aztec(public-vm)") {
                transform_vm_function(func, storage_defined)
                    .map_err(|err| (err, crate_graph.root_file_id))?;
                has_transformed_module = true;
            }
        }
        // Add the storage struct to the beginning of the function if it is unconstrained in an aztec contract
        if storage_defined && func.def.is_unconstrained {
            transform_unconstrained(func);
            has_transformed_module = true;
        }
    }

    if has_transformed_module {
        // We only want to run these checks if the macro processor has found the module to be an Aztec contract.

        if module.functions.len() > MAX_CONTRACT_FUNCTIONS {
            let crate_graph = &context.crate_graph[crate_id];
            return Err((
                AztecMacroError::ContractHasTooManyFunctions { span: Span::default() },
                crate_graph.root_file_id,
            ));
        }

        let constructor_defined = module.functions.iter().any(|func| func.name() == "constructor");
        if !constructor_defined {
            let crate_graph = &context.crate_graph[crate_id];
            return Err((
                AztecMacroError::ContractConstructorMissing { span: Span::default() },
                crate_graph.root_file_id,
            ));
        }
    }

    Ok(has_transformed_module)
}

/// Auxiliary function to generate the storage constructor for a given field, using
/// the Storage definition as a reference. Supports nesting.
fn generate_storage_field_constructor(
    (type_ident, unresolved_type): &(Ident, UnresolvedType),
    slot: Expression,
) -> Result<Expression, AztecMacroError> {
    let typ = &unresolved_type.typ;
    match typ {
        UnresolvedTypeData::Named(path, generics, _) => {
            let mut new_path = path.clone().to_owned();
            new_path.segments.push(ident("new"));
            match path.segments.last().unwrap().0.contents.as_str() {
                "Map" => Ok(call(
                    variable_path(new_path),
                    vec![
                        variable("context"),
                        slot,
                        lambda(
                            vec![
                                (
                                    pattern("context"),
                                    make_type(UnresolvedTypeData::Named(
                                        chained_path!("aztec", "context", "Context"),
                                        vec![],
                                        true,
                                    )),
                                ),
                                (
                                    Pattern::Identifier(ident("slot")),
                                    make_type(UnresolvedTypeData::FieldElement),
                                ),
                            ],
                            generate_storage_field_constructor(
                                &(type_ident.clone(), generics.iter().last().unwrap().clone()),
                                variable("slot"),
                            )?,
                        ),
                    ],
                )),
                _ => Ok(call(variable_path(new_path), vec![variable("context"), slot])),
            }
        }
        _ => Err(AztecMacroError::UnsupportedStorageType {
            typ: typ.clone(),
            span: Some(type_ident.span()),
        }),
    }
}

// Generates the Storage implementation block from the Storage struct definition if it does not exist
/// From:
///
/// struct Storage {
///     a_map: Map<Field, SomeStoragePrimitive<ASerializableType>>,
///     a_nested_map: Map<Field, Map<Field, SomeStoragePrimitive<ASerializableType>>>,
///     a_field: SomeStoragePrimitive<ASerializableType>,
/// }
///
/// To:
///
/// impl Storage {
///    fn init(context: Context) -> Self {
///        Storage {
///             a_map: Map::new(context, 0, |context, slot| {
///                 SomeStoragePrimitive::new(context, slot)
///             }),
///             a_nested_map: Map::new(context, 0, |context, slot| {
///                 Map::new(context, slot, |context, slot| {
///                     SomeStoragePrimitive::new(context, slot)
///                })
///            }),
///            a_field: SomeStoragePrimitive::new(context, 0),
///         }
///    }
/// }
///
/// Storage slots are generated as 0 and will be populated using the information from the HIR
/// at a later stage.
fn generate_storage_implementation(module: &mut SortedModule) -> Result<(), AztecMacroError> {
    let definition =
        module.types.iter().find(|r#struct| r#struct.name.0.contents == "Storage").unwrap();

    let slot_zero = expression(ExpressionKind::Literal(Literal::Integer(
        FieldElement::from(i128::from(0)),
        false,
    )));

    let field_constructors = definition
        .fields
        .iter()
        .flat_map(|field| {
            generate_storage_field_constructor(field, slot_zero.clone())
                .map(|expression| (field.0.clone(), expression))
        })
        .collect();

    let storage_constructor_statement = make_statement(StatementKind::Expression(expression(
        ExpressionKind::constructor((chained_path!("Storage"), field_constructors)),
    )));

    let init = NoirFunction::normal(FunctionDefinition::normal(
        &ident("init"),
        &vec![],
        &[(
            ident("context"),
            make_type(UnresolvedTypeData::Named(
                chained_path!("aztec", "context", "Context"),
                vec![],
                true,
            )),
        )],
        &BlockExpression(vec![storage_constructor_statement]),
        &[],
        &return_type(chained_path!("Self")),
    ));

    let storage_impl = TypeImpl {
        object_type: UnresolvedType {
            typ: UnresolvedTypeData::Named(chained_path!("Storage"), vec![], true),
            span: Some(Span::default()),
        },
        type_span: Span::default(),
        generics: vec![],
        methods: vec![(init, Span::default())],
    };
    module.impls.push(storage_impl);

    Ok(())
}

/// If it does, it will insert the following things:
/// - A new Input that is provided for a kernel app circuit, named: {Public/Private}ContextInputs
/// - Hashes all of the function input variables
///     - This instantiates a helper function  
fn transform_function(
    ty: &str,
    func: &mut NoirFunction,
    storage_defined: bool,
) -> Result<(), AztecMacroError> {
    let context_name = format!("{}Context", ty);
    let inputs_name = format!("{}ContextInputs", ty);
    let return_type_name = format!("{}CircuitPublicInputs", ty);

    // Add access to the storage struct
    if storage_defined {
        let storage_def = abstract_storage(&ty.to_lowercase(), false);
        func.def.body.0.insert(0, storage_def);
    }

    // Insert the context creation as the first action
    let create_context = create_context(&context_name, &func.def.parameters)?;
    func.def.body.0.splice(0..0, (create_context).iter().cloned());

    // Add the inputs to the params
    let input = create_inputs(&inputs_name);
    func.def.parameters.insert(0, input);

    // Abstract return types such that they get added to the kernel's return_values
    if let Some(return_values) = abstract_return_values(func) {
        func.def.body.0.push(return_values);
    }

    // Push the finish method call to the end of the function
    let finish_def = create_context_finish();
    func.def.body.0.push(finish_def);

    let return_type = create_return_type(&return_type_name);
    func.def.return_type = return_type;
    func.def.return_visibility = Visibility::Public;

    // Distinct return types are only required for private functions
    // Public functions should have open auto-inferred
    match ty {
        "Private" => func.def.return_distinctness = Distinctness::Distinct,
        "Public" => func.def.is_open = true,
        _ => (),
    }

    Ok(())
}

/// Transform a function to work with AVM bytecode
fn transform_vm_function(
    func: &mut NoirFunction,
    _storage_defined: bool,
) -> Result<(), AztecMacroError> {
    // Push Avm context creation to the beginning of the function
    let create_context = create_avm_context()?;
    func.def.body.0.insert(0, create_context);

    // We want the function to be seen as a public function
    func.def.is_open = true;

    // NOTE: the line below is a temporary hack to trigger external transpilation tools
    // It will be removed once the transpiler is integrated into the Noir compiler
    func.def.name.0.contents = format!("avm_{}", func.def.name.0.contents);
    Ok(())
}

/// Transform Unconstrained
///
/// Inserts the following code at the beginning of an unconstrained function
/// ```noir
/// let storage = Storage::init(Context::none());
/// ```
///
/// This will allow developers to access their contract' storage struct in unconstrained functions
fn transform_unconstrained(func: &mut NoirFunction) {
    func.def.body.0.insert(0, abstract_storage("Unconstrained", true));
}

fn collect_crate_structs(crate_id: &CrateId, context: &HirContext) -> Vec<StructId> {
    context
        .def_map(crate_id)
        .expect("ICE: Missing crate in def_map")
        .modules()
        .iter()
        .flat_map(|(_, module)| {
            module.type_definitions().filter_map(|typ| {
                if let ModuleDefId::TypeId(struct_id) = typ {
                    Some(struct_id)
                } else {
                    None
                }
            })
        })
        .collect()
}

fn collect_traits(context: &HirContext) -> Vec<TraitId> {
    let crates = context.crates();
    crates
        .flat_map(|crate_id| context.def_map(&crate_id).map(|def_map| def_map.modules()))
        .flatten()
        .flat_map(|module| {
            module.type_definitions().filter_map(|typ| {
                if let ModuleDefId::TraitId(struct_id) = typ {
                    Some(struct_id)
                } else {
                    None
                }
            })
        })
        .collect()
}

/// Substitutes the signature literal that was introduced in the selector method previously with the actual signature.
fn transform_event(
    struct_id: StructId,
    interner: &mut NodeInterner,
) -> Result<(), (AztecMacroError, FileId)> {
    let struct_type = interner.get_struct(struct_id);
    let selector_id = interner
        .lookup_method(&Type::Struct(struct_type.clone(), vec![]), struct_id, "selector", false)
        .ok_or_else(|| {
            let error = AztecMacroError::EventError {
                span: struct_type.borrow().location.span,
                message: "Selector method not found".to_owned(),
            };
            (error, struct_type.borrow().location.file)
        })?;
    let selector_function = interner.function(&selector_id);

    let compute_selector_statement = interner.statement(
        selector_function.block(interner).statements().first().ok_or_else(|| {
            let error = AztecMacroError::EventError {
                span: struct_type.borrow().location.span,
                message: "Compute selector statement not found".to_owned(),
            };
            (error, struct_type.borrow().location.file)
        })?,
    );

    let compute_selector_expression = match compute_selector_statement {
        HirStatement::Expression(expression_id) => match interner.expression(&expression_id) {
            HirExpression::Call(hir_call_expression) => Some(hir_call_expression),
            _ => None,
        },
        _ => None,
    }
    .ok_or_else(|| {
        let error = AztecMacroError::EventError {
            span: struct_type.borrow().location.span,
            message: "Compute selector statement is not a call expression".to_owned(),
        };
        (error, struct_type.borrow().location.file)
    })?;

    let first_arg_id = compute_selector_expression.arguments.first().ok_or_else(|| {
        let error = AztecMacroError::EventError {
            span: struct_type.borrow().location.span,
            message: "Compute selector statement is not a call expression".to_owned(),
        };
        (error, struct_type.borrow().location.file)
    })?;

    match interner.expression(first_arg_id) {
        HirExpression::Literal(HirLiteral::Str(signature))
            if signature == SIGNATURE_PLACEHOLDER =>
        {
            let selector_literal_id = *first_arg_id;

            let structure = interner.get_struct(struct_id);
            let signature = event_signature(&structure.borrow());
            interner.update_expression(selector_literal_id, |expr| {
                *expr = HirExpression::Literal(HirLiteral::Str(signature.clone()));
            });

            // Also update the type! It might have a different length now than the placeholder.
            interner.push_expr_type(
                selector_literal_id,
                Type::String(Box::new(Type::Constant(signature.len() as u64))),
            );
            Ok(())
        }
        _ => Err((
            AztecMacroError::EventError {
                span: struct_type.borrow().location.span,
                message: "Signature placeholder literal does not match".to_owned(),
            },
            struct_type.borrow().location.file,
        )),
    }
}

fn transform_events(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    for struct_id in collect_crate_structs(crate_id, context) {
        let attributes = context.def_interner.struct_attributes(&struct_id);
        if attributes.iter().any(|attr| matches!(attr, SecondaryAttribute::Event)) {
            transform_event(struct_id, &mut context.def_interner)?;
        }
    }
    Ok(())
}

/// Obtains the serialized length of a type that implements the Serialize trait.
fn get_serialized_length(
    traits: &[TraitId],
    typ: &Type,
    interner: &NodeInterner,
) -> Result<u64, AztecMacroError> {
    let (struct_name, maybe_stored_in_state) = match typ {
        Type::Struct(struct_type, generics) => {
            Ok((struct_type.borrow().name.0.contents.clone(), generics.first()))
        }
        _ => Err(AztecMacroError::CouldNotAssignStorageSlots {
            secondary_message: Some("State storage variable must be a struct".to_string()),
        }),
    }?;
    let stored_in_state =
        maybe_stored_in_state.ok_or(AztecMacroError::CouldNotAssignStorageSlots {
            secondary_message: Some("State storage variable must be generic".to_string()),
        })?;

    let is_note = traits.iter().any(|&trait_id| {
        let r#trait = interner.get_trait(trait_id);
        r#trait.name.0.contents == "NoteInterface"
            && !interner.lookup_all_trait_implementations(stored_in_state, trait_id).is_empty()
    });

    // Maps and (private) Notes always occupy a single slot. Someone could store a Note in PublicState for whatever reason though.
    if struct_name == "Map" || (is_note && struct_name != "PublicState") {
        return Ok(1);
    }

    let serialized_trait_impl_kind = traits
        .iter()
        .find_map(|&trait_id| {
            let r#trait = interner.get_trait(trait_id);
            if r#trait.borrow().name.0.contents == "Serialize"
                && r#trait.borrow().generics.len() == 1
            {
                interner
                    .lookup_all_trait_implementations(stored_in_state, trait_id)
                    .into_iter()
                    .next()
            } else {
                None
            }
        })
        .ok_or(AztecMacroError::CouldNotAssignStorageSlots {
            secondary_message: Some("Stored data must implement Serialize trait".to_string()),
        })?;

    let serialized_trait_impl_id = match serialized_trait_impl_kind {
        TraitImplKind::Normal(trait_impl_id) => Ok(trait_impl_id),
        _ => Err(AztecMacroError::CouldNotAssignStorageSlots { secondary_message: None }),
    }?;

    let serialized_trait_impl_shared = interner.get_trait_implementation(*serialized_trait_impl_id);
    let serialized_trait_impl = serialized_trait_impl_shared.borrow();

    match serialized_trait_impl.trait_generics.first().unwrap() {
        Type::Constant(value) => Ok(*value),
        _ => Err(AztecMacroError::CouldNotAssignStorageSlots { secondary_message: None }),
    }
}

/// Assigns storage slots to the storage struct fields based on the serialized length of the types. This automatic assignment
/// will only trigger if the assigned storage slot is invalid (0 as generated by generate_storage_implementation)
fn assign_storage_slots(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    let traits: Vec<_> = collect_traits(context);
    for struct_id in collect_crate_structs(crate_id, context) {
        let interner: &mut NodeInterner = context.def_interner.borrow_mut();
        let r#struct = interner.get_struct(struct_id);
        let file_id = r#struct.borrow().location.file;
        if r#struct.borrow().name.0.contents == "Storage" && r#struct.borrow().id.krate().is_root()
        {
            let init_id = interner
                .lookup_method(
                    &Type::Struct(interner.get_struct(struct_id), vec![]),
                    struct_id,
                    "init",
                    false,
                )
                .ok_or((
                    AztecMacroError::CouldNotAssignStorageSlots {
                        secondary_message: Some(
                            "Storage struct must have an init function".to_string(),
                        ),
                    },
                    file_id,
                ))?;
            let init_function = interner.function(&init_id).block(interner);
            let init_function_statement_id = init_function.statements().first().ok_or((
                AztecMacroError::CouldNotAssignStorageSlots {
                    secondary_message: Some("Init storage statement not found".to_string()),
                },
                file_id,
            ))?;
            let storage_constructor_statement = interner.statement(init_function_statement_id);

            let storage_constructor_expression = match storage_constructor_statement {
                HirStatement::Expression(expression_id) => {
                    match interner.expression(&expression_id) {
                        HirExpression::Constructor(hir_constructor_expression) => {
                            Ok(hir_constructor_expression)
                        }
                        _ => Err((AztecMacroError::CouldNotAssignStorageSlots {
                            secondary_message: Some(
                                "Storage constructor statement must be a constructor expression"
                                    .to_string(),
                            ),
                        }, file_id))
                    }
                }
                _ => Err((
                    AztecMacroError::CouldNotAssignStorageSlots {
                        secondary_message: Some(
                            "Storage constructor statement must be an expression".to_string(),
                        ),
                    },
                    file_id,
                )),
            }?;

            let mut storage_slot: u64 = 1;
            for (index, (_, expr_id)) in storage_constructor_expression.fields.iter().enumerate() {
                let fields = r#struct.borrow().get_fields(&[]);
                let (_, field_type) = fields.get(index).unwrap();
                let new_call_expression = match interner.expression(expr_id) {
                    HirExpression::Call(hir_call_expression) => Ok(hir_call_expression),
                    _ => Err((
                        AztecMacroError::CouldNotAssignStorageSlots {
                            secondary_message: Some(
                                "Storage field initialization expression is not a call expression"
                                    .to_string(),
                            ),
                        },
                        file_id,
                    )),
                }?;

                let slot_arg_expression = interner.expression(&new_call_expression.arguments[1]);

                let current_storage_slot = match slot_arg_expression {
                    HirExpression::Literal(HirLiteral::Integer(slot, _)) => Ok(slot.to_u128()),
                    _ => Err((
                        AztecMacroError::CouldNotAssignStorageSlots {
                            secondary_message: Some(
                                "Storage slot argument expression must be a literal integer"
                                    .to_string(),
                            ),
                        },
                        file_id,
                    )),
                }?;

                if current_storage_slot != 0 {
                    continue;
                }

                let type_serialized_len = get_serialized_length(&traits, field_type, interner)
                    .map_err(|err| (err, file_id))?;
                interner.update_expression(new_call_expression.arguments[1], |expr| {
                    *expr = HirExpression::Literal(HirLiteral::Integer(
                        FieldElement::from(u128::from(storage_slot)),
                        false,
                    ));
                });

                storage_slot += type_serialized_len;
            }
        }
    }
    Ok(())
}

const SIGNATURE_PLACEHOLDER: &str = "SIGNATURE_PLACEHOLDER";

/// Generates the impl for an event selector
///
/// Inserts the following code:
/// ```noir
/// impl SomeStruct {
///    fn selector() -> FunctionSelector {
///       aztec::protocol_types::abis::function_selector::FunctionSelector::from_signature("SIGNATURE_PLACEHOLDER")
///    }
/// }
/// ```
///
/// This allows developers to emit events without having to write the signature of the event every time they emit it.
/// The signature cannot be known at this point since types are not resolved yet, so we use a signature placeholder.
/// It'll get resolved after by transforming the HIR.
fn generate_selector_impl(structure: &NoirStruct) -> TypeImpl {
    let struct_type =
        make_type(UnresolvedTypeData::Named(path(structure.name.clone()), vec![], true));

    let selector_path =
        chained_path!("aztec", "protocol_types", "abis", "function_selector", "FunctionSelector");
    let mut from_signature_path = selector_path.clone();
    from_signature_path.segments.push(ident("from_signature"));

    let selector_fun_body = BlockExpression(vec![make_statement(StatementKind::Expression(call(
        variable_path(from_signature_path),
        vec![expression(ExpressionKind::Literal(Literal::Str(SIGNATURE_PLACEHOLDER.to_string())))],
    )))]);

    // Define `FunctionSelector` return type
    let return_type =
        FunctionReturnType::Ty(make_type(UnresolvedTypeData::Named(selector_path, vec![], true)));

    let mut selector_fn_def = FunctionDefinition::normal(
        &ident("selector"),
        &vec![],
        &[],
        &selector_fun_body,
        &[],
        &return_type,
    );

    selector_fn_def.visibility = FunctionVisibility::Public;

    // Seems to be necessary on contract modules
    selector_fn_def.return_visibility = Visibility::Public;

    TypeImpl {
        object_type: struct_type,
        type_span: structure.span,
        generics: vec![],
        methods: vec![(NoirFunction::normal(selector_fn_def), Span::default())],
    }
}

/// Helper function that returns what the private context would look like in the ast
/// This should make it available to be consumed within aztec private annotated functions.
///
/// The replaced code:
/// ```noir
/// /// Before
/// fn foo(inputs: PrivateContextInputs) {
///    // ...
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() {
///   // ...
/// }
fn create_inputs(ty: &str) -> Param {
    let context_ident = ident("inputs");
    let context_pattern = Pattern::Identifier(context_ident);

    let path_snippet = ty.to_case(Case::Snake); // e.g. private_context_inputs
    let type_path = chained_path!("aztec", "context", "inputs", &path_snippet, ty);

    let context_type = make_type(UnresolvedTypeData::Named(type_path, vec![], true));
    let visibility = Visibility::Private;

    Param { pattern: context_pattern, typ: context_type, visibility, span: Span::default() }
}

/// Creates the private context object to be accessed within the function, the parameters need to be extracted to be
/// appended into the args hash object.
///
/// The replaced code:
/// ```noir
/// #[aztec(private)]
/// fn foo(structInput: SomeStruct, arrayInput: [u8; 10], fieldInput: Field) -> Field {
///     // Create the hasher object
///     let mut hasher = Hasher::new();
///
///     // struct inputs call serialize on them to add an array of fields
///     hasher.add_multiple(structInput.serialize());
///
///     // Array inputs are iterated over and each element is added to the hasher (as a field)
///     for i in 0..arrayInput.len() {
///         hasher.add(arrayInput[i] as Field);
///     }
///     // Field inputs are added to the hasher
///     hasher.add({ident});
///
///     // Create the context
///     // The inputs (injected by this `create_inputs`) and completed hash object are passed to the context
///     let mut context = PrivateContext::new(inputs, hasher.hash());
/// }
/// ```
fn create_context(ty: &str, params: &[Param]) -> Result<Vec<Statement>, AztecMacroError> {
    let mut injected_expressions: Vec<Statement> = vec![];

    // `let mut hasher = Hasher::new();`
    let let_hasher = mutable_assignment(
        "hasher", // Assigned to
        call(
            variable_path(chained_path!("aztec", "hasher", "Hasher", "new")), // Path
            vec![],                                                           // args
        ),
    );

    // Completes: `let mut hasher = Hasher::new();`
    injected_expressions.push(let_hasher);

    // Iterate over each of the function parameters, adding to them to the hasher
    for Param { pattern, typ, span, .. } in params {
        match pattern {
            Pattern::Identifier(identifier) => {
                // Match the type to determine the padding to do
                let unresolved_type = &typ.typ;
                let expression = match unresolved_type {
                    // `hasher.add_multiple({ident}.serialize())`
                    UnresolvedTypeData::Named(..) => add_struct_to_hasher(identifier),
                    UnresolvedTypeData::Array(_, arr_type) => {
                        add_array_to_hasher(identifier, arr_type)
                    }
                    // `hasher.add({ident})`
                    UnresolvedTypeData::FieldElement => add_field_to_hasher(identifier),
                    // Add the integer to the hasher, casted to a field
                    // `hasher.add({ident} as Field)`
                    UnresolvedTypeData::Integer(..) | UnresolvedTypeData::Bool => {
                        add_cast_to_hasher(identifier)
                    }
                    UnresolvedTypeData::String(..) => {
                        let (var_bytes, id) = str_to_bytes(identifier);
                        injected_expressions.push(var_bytes);
                        add_array_to_hasher(
                            &id,
                            &UnresolvedType {
                                typ: UnresolvedTypeData::Integer(
                                    Signedness::Unsigned,
                                    noirc_frontend::IntegerBitSize::ThirtyTwo,
                                ),
                                span: None,
                            },
                        )
                    }
                    _ => {
                        return Err(AztecMacroError::UnsupportedFunctionArgumentType {
                            typ: unresolved_type.clone(),
                            span: *span,
                        })
                    }
                };
                injected_expressions.push(expression);
            }
            _ => todo!(), // Maybe unreachable?
        }
    }

    // Create the inputs to the context
    let inputs_expression = variable("inputs");
    // `hasher.hash()`
    let hash_call = method_call(
        variable("hasher"), // variable
        "hash",             // method name
        vec![],             // args
    );

    let path_snippet = ty.to_case(Case::Snake); // e.g. private_context

    // let mut context = {ty}::new(inputs, hash);
    let let_context = mutable_assignment(
        "context", // Assigned to
        call(
            variable_path(chained_path!("aztec", "context", &path_snippet, ty, "new")), // Path
            vec![inputs_expression, hash_call],                                         // args
        ),
    );
    injected_expressions.push(let_context);

    // Return all expressions that will be injected by the hasher
    Ok(injected_expressions)
}

/// Creates an mutable avm context
///
/// ```noir
/// /// Before
/// #[aztec(public-vm)]
/// fn foo() -> Field {
///   let mut context = aztec::context::AVMContext::new();
///   let timestamp = context.timestamp();
///   // ...
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() -> Field {
///     let mut timestamp = context.timestamp();
///     // ...
/// }
fn create_avm_context() -> Result<Statement, AztecMacroError> {
    let let_context = mutable_assignment(
        "context", // Assigned to
        call(
            variable_path(chained_path!("aztec", "context", "AVMContext", "new")), // Path
            vec![],                                                                // args
        ),
    );

    Ok(let_context)
}

/// Abstract Return Type
///
/// This function intercepts the function's current return type and replaces it with pushes
/// To the kernel
///
/// The replaced code:
/// ```noir
/// /// Before
/// #[aztec(private)]
/// fn foo() -> protocol_types::abis::private_circuit_public_inputs::PrivateCircuitPublicInputs {
///   // ...
///   let my_return_value: Field = 10;
///   context.return_values.push(my_return_value);
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() -> Field {
///     // ...
///    let my_return_value: Field = 10;
///    my_return_value
/// }
/// ```
/// Similarly; Structs will be pushed to the context, after serialize() is called on them.
/// Arrays will be iterated over and each element will be pushed to the context.
/// Any primitive type that can be cast will be casted to a field and pushed to the context.
fn abstract_return_values(func: &NoirFunction) -> Option<Statement> {
    let current_return_type = func.return_type().typ;
    let len = func.def.body.len();
    let last_statement = &func.def.body.0[len - 1];

    // TODO: (length, type) => We can limit the size of the array returned to be limited by kernel size
    // Doesn't need done until we have settled on a kernel size
    // TODO: support tuples here and in inputs -> convert into an issue

    // Check if the return type is an expression, if it is, we can handle it
    match last_statement {
        Statement { kind: StatementKind::Expression(expression), .. } => {
            match current_return_type {
                // Call serialize on structs, push the whole array, calling push_array
                UnresolvedTypeData::Named(..) => Some(make_struct_return_type(expression.clone())),
                UnresolvedTypeData::Array(..) => Some(make_array_return_type(expression.clone())),
                // Cast these types to a field before pushing
                UnresolvedTypeData::Bool | UnresolvedTypeData::Integer(..) => {
                    Some(make_castable_return_type(expression.clone()))
                }
                UnresolvedTypeData::FieldElement => Some(make_return_push(expression.clone())),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Abstract storage
///
/// For private functions:
/// ```noir
/// #[aztec(private)]
/// fn lol() {
///     let storage = Storage::init(Context::private(context));
/// }
/// ```
///
/// For public functions:
/// ```noir
/// #[aztec(public)]
/// fn lol() {
///    let storage = Storage::init(Context::public(context));
/// }
/// ```
///
/// For unconstrained functions:
/// ```noir
/// unconstrained fn lol() {
///   let storage = Storage::init(Context::none());
/// }
fn abstract_storage(typ: &str, unconstrained: bool) -> Statement {
    let init_context_call = if unconstrained {
        call(
            variable_path(chained_path!("aztec", "context", "Context", "none")), // Path
            vec![],                                                              // args
        )
    } else {
        call(
            variable_path(chained_path!("aztec", "context", "Context", typ)), // Path
            vec![mutable_reference("context")],                               // args
        )
    };

    assignment(
        "storage", // Assigned to
        call(
            variable_path(chained_path!("Storage", "init")), // Path
            vec![init_context_call],                         // args
        ),
    )
}

/// Context Return Values
///
/// Creates an instance to the context return values
/// ```noir
/// `context.return_values`
/// ```
fn context_return_values() -> Expression {
    member_access("context", "return_values")
}

fn make_statement(kind: StatementKind) -> Statement {
    Statement { span: Span::default(), kind }
}

/// Make return Push
///
/// Translates to:
/// `context.return_values.push({push_value})`
fn make_return_push(push_value: Expression) -> Statement {
    make_statement(StatementKind::Semi(method_call(
        context_return_values(),
        "push",
        vec![push_value],
    )))
}

/// Make Return push array
///
/// Translates to:
/// `context.return_values.extend_from_array({push_value})`
fn make_return_extend_from_array(push_value: Expression) -> Statement {
    make_statement(StatementKind::Semi(method_call(
        context_return_values(),
        "extend_from_array",
        vec![push_value],
    )))
}

/// Make struct return type
///
/// Translates to:
/// ```noir
/// `context.return_values.extend_from_array({push_value}.serialize())`
fn make_struct_return_type(expression: Expression) -> Statement {
    let serialized_call = method_call(
        expression,  // variable
        "serialize", // method name
        vec![],      // args
    );
    make_return_extend_from_array(serialized_call)
}

/// Make array return type
///
/// Translates to:
/// ```noir
/// for i in 0..{ident}.len() {
///    context.return_values.push({ident}[i] as Field)
/// }
/// ```
fn make_array_return_type(expression: Expression) -> Statement {
    let inner_cast_expression =
        cast(index_array_variable(expression.clone(), "i"), UnresolvedTypeData::FieldElement);
    let assignment = make_statement(StatementKind::Semi(method_call(
        context_return_values(), // variable
        "push",                  // method name
        vec![inner_cast_expression],
    )));

    create_loop_over(expression, vec![assignment])
}

/// Castable return type
///
/// Translates to:
/// ```noir
/// context.return_values.push({ident} as Field)
/// ```
fn make_castable_return_type(expression: Expression) -> Statement {
    // Cast these types to a field before pushing
    let cast_expression = cast(expression, UnresolvedTypeData::FieldElement);
    make_return_push(cast_expression)
}

/// Create Return Type
///
/// Public functions return protocol_types::abis::public_circuit_public_inputs::PublicCircuitPublicInputs while
/// private functions return protocol_types::abis::private_circuit_public_inputs::::PrivateCircuitPublicInputs
///
/// This call constructs an ast token referencing the above types
/// The name is set in the function above `transform`, hence the
/// whole token name is passed in
///
/// The replaced code:
/// ```noir
///
/// /// Before
/// fn foo() -> protocol_types::abis::private_circuit_public_inputs::PrivateCircuitPublicInputs {
///    // ...
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() {
///  // ...
/// }
fn create_return_type(ty: &str) -> FunctionReturnType {
    let path_snippet = ty.to_case(Case::Snake); // e.g. private_circuit_public_inputs or public_circuit_public_inputs
    let return_path = chained_path!("aztec", "protocol_types", "abis", &path_snippet, ty);
    return_type(return_path)
}

/// Create Context Finish
///
/// Each aztec function calls `context.finish()` at the end of a function
/// to return values required by the kernel.
///
/// The replaced code:
/// ```noir
/// /// Before
/// fn foo() -> protocol_types::abis::private_circuit_public_inputs::PrivateCircuitPublicInputs {
///   // ...
///  context.finish()
/// }
///
/// /// After
/// #[aztec(private)]
/// fn foo() {
///  // ...
/// }
fn create_context_finish() -> Statement {
    let method_call = method_call(
        variable("context"), // variable
        "finish",            // method name
        vec![],              // args
    );
    make_statement(StatementKind::Expression(method_call))
}

//
//                 Methods to create hasher inputs
//

fn add_struct_to_hasher(identifier: &Ident) -> Statement {
    // If this is a struct, we call serialize and add the array to the hasher
    let serialized_call = method_call(
        variable_path(path(identifier.clone())), // variable
        "serialize",                             // method name
        vec![],                                  // args
    );

    make_statement(StatementKind::Semi(method_call(
        variable("hasher"),    // variable
        "add_multiple",        // method name
        vec![serialized_call], // args
    )))
}

fn str_to_bytes(identifier: &Ident) -> (Statement, Ident) {
    // let identifier_as_bytes = identifier.as_bytes();
    let var = variable_ident(identifier.clone());
    let contents = if let ExpressionKind::Variable(p) = &var.kind {
        p.segments.first().cloned().unwrap_or_else(|| panic!("No segments")).0.contents
    } else {
        panic!("Unexpected identifier type")
    };
    let bytes_name = format!("{}_bytes", contents);
    let var_bytes = assignment(&bytes_name, method_call(var, "as_bytes", vec![]));
    let id = Ident::new(bytes_name, Span::default());

    (var_bytes, id)
}

fn create_loop_over(var: Expression, loop_body: Vec<Statement>) -> Statement {
    // If this is an array of primitive types (integers / fields) we can add them each to the hasher
    // casted to a field
    let span = var.span;

    // `array.len()`
    let end_range_expression = method_call(
        var,    // variable
        "len",  // method name
        vec![], // args
    );

    // What will be looped over
    // - `hasher.add({ident}[i] as Field)`
    let for_loop_block = expression(ExpressionKind::Block(BlockExpression(loop_body)));

    // `for i in 0..{ident}.len()`
    make_statement(StatementKind::For(ForLoopStatement {
        range: ForRange::Range(
            expression(ExpressionKind::Literal(Literal::Integer(
                FieldElement::from(i128::from(0)),
                false,
            ))),
            end_range_expression,
        ),
        identifier: ident("i"),
        block: for_loop_block,
        span,
    }))
}

fn add_array_to_hasher(identifier: &Ident, arr_type: &UnresolvedType) -> Statement {
    // If this is an array of primitive types (integers / fields) we can add them each to the hasher
    // casted to a field

    // Wrap in the semi thing - does that mean ended with semi colon?
    // `hasher.add({ident}[i] as Field)`

    let arr_index = index_array(identifier.clone(), "i");
    let (add_expression, hasher_method_name) = match arr_type.typ {
        UnresolvedTypeData::Named(..) => {
            let hasher_method_name = "add_multiple".to_owned();
            let call = method_call(
                // All serialize on each element
                arr_index,   // variable
                "serialize", // method name
                vec![],      // args
            );
            (call, hasher_method_name)
        }
        _ => {
            let hasher_method_name = "add".to_owned();
            let call = cast(
                arr_index,                        // lhs - `ident[i]`
                UnresolvedTypeData::FieldElement, // cast to - `as Field`
            );
            (call, hasher_method_name)
        }
    };

    let block_statement = make_statement(StatementKind::Semi(method_call(
        variable("hasher"),  // variable
        &hasher_method_name, // method name
        vec![add_expression],
    )));

    create_loop_over(variable_ident(identifier.clone()), vec![block_statement])
}

fn add_field_to_hasher(identifier: &Ident) -> Statement {
    // `hasher.add({ident})`
    let ident = variable_path(path(identifier.clone()));
    make_statement(StatementKind::Semi(method_call(
        variable("hasher"), // variable
        "add",              // method name
        vec![ident],        // args
    )))
}

fn add_cast_to_hasher(identifier: &Ident) -> Statement {
    // `hasher.add({ident} as Field)`
    // `{ident} as Field`
    let cast_operation = cast(
        variable_path(path(identifier.clone())), // lhs
        UnresolvedTypeData::FieldElement,        // rhs
    );

    // `hasher.add({ident} as Field)`
    make_statement(StatementKind::Semi(method_call(
        variable("hasher"),   // variable
        "add",                // method name
        vec![cast_operation], // args
    )))
}

/// Computes the aztec signature for a resolved type.
fn signature_of_type(typ: &Type) -> String {
    match typ {
        Type::Integer(Signedness::Signed, bit_size) => format!("i{}", bit_size),
        Type::Integer(Signedness::Unsigned, bit_size) => format!("u{}", bit_size),
        Type::FieldElement => "Field".to_owned(),
        Type::Bool => "bool".to_owned(),
        Type::Array(len, typ) => {
            if let Type::Constant(len) = **len {
                format!("[{};{len}]", signature_of_type(typ))
            } else {
                unimplemented!("Cannot generate signature for array with length type {:?}", typ)
            }
        }
        Type::Struct(def, args) => {
            let fields = def.borrow().get_fields(args);
            let fields = vecmap(fields, |(_, typ)| signature_of_type(&typ));
            format!("({})", fields.join(","))
        }
        Type::Tuple(types) => {
            let fields = vecmap(types, signature_of_type);
            format!("({})", fields.join(","))
        }
        _ => unimplemented!("Cannot generate signature for type {:?}", typ),
    }
}

/// Computes the signature for a resolved event type.
/// It has the form 'EventName(Field,(Field),[u8;2])'
fn event_signature(event: &StructType) -> String {
    let fields = vecmap(event.get_fields(&[]), |(_, typ)| signature_of_type(&typ));
    format!("{}({})", event.name.0.contents, fields.join(","))
}
