pub use noirc_errors::Span;
use noirc_errors::{CustomDiagnostic as Diagnostic, Location};
use thiserror::Error;

use crate::{
    Kind, Type,
    ast::{Ident, UnsupportedNumericGenericType},
    elaborator::{TypedPath, types::WildcardDisallowedContext},
    hir::{comptime::Value, type_check::TypeCheckError},
    parser::ParserError,
    signed_field::SignedField,
    usage_tracker::UnusedItem,
};

use super::import::PathResolutionError;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PubPosition {
    #[error("parameter")]
    Parameter,
    #[error("return type")]
    ReturnType,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ResolverError {
    #[error("Duplicate definition")]
    DuplicateDefinition { name: String, first_location: Location, second_location: Location },
    #[error("Unused variable")]
    UnusedVariable { ident: Ident },
    #[error("Unused {}", item.item_type())]
    UnusedItem { ident: Ident, item: UnusedItem },
    #[error("Unconditional recursion")]
    UnconditionalRecursion { name: String, location: Location },
    #[error("Could not find variable in this scope")]
    VariableNotDeclared { name: String, location: Location },
    #[error("could not resolve path")]
    PathResolutionError(#[from] PathResolutionError),
    #[error("Expected")]
    Expected { location: Location, expected: &'static str, got: &'static str },
    #[error("Duplicate field in constructor")]
    DuplicateField { field: Ident },
    #[error("No such field in struct")]
    NoSuchField { field: Ident, struct_definition: Ident },
    #[error("Missing fields from struct")]
    MissingFields { location: Location, missing_fields: Vec<String>, struct_definition: Ident },
    #[error("Unneeded 'mut', pattern is already marked as mutable")]
    UnnecessaryMut { first_mut: Location, second_mut: Location },
    #[error("Unneeded 'pub', function is not the main method")]
    UnnecessaryPub { ident: Ident, position: PubPosition },
    #[error("Required 'pub', main function must return public value")]
    NecessaryPub { ident: Ident },
    #[error("No global or generic type parameter found with the given name")]
    NoSuchNumericTypeVariable { path: TypedPath },
    #[error("Only struct types can be used in constructor expressions")]
    NonStructUsedInConstructor { typ: String, location: Location },
    #[error("Cannot apply generics on Self type")]
    GenericsOnSelfType { location: Location },
    #[error("Cannot apply generics on an associated type")]
    GenericsOnAssociatedType { location: Location },
    #[error("{0}")]
    ParserError(Box<ParserError>),
    #[error("Closure environment must be a tuple or unit type")]
    InvalidClosureEnvironment { typ: Type, location: Location },
    #[error("Nested vectors, i.e. vectors within an array or vector, are not supported")]
    NestedVectors { location: Location },
    #[error("#[abi(tag)] attribute is only allowed in contracts")]
    AbiAttributeOutsideContract { location: Location },
    #[error(
        "Usage of the `#[foreign]` or `#[builtin]` function attributes are not allowed outside of the Noir standard library"
    )]
    LowLevelFunctionOutsideOfStdlib { location: Location },
    #[error("Usage of the `#[oracle]` function attribute is only valid on unconstrained functions")]
    OracleMarkedAsConstrained { ident: Ident, location: Location },
    #[error("Oracle functions cannot return multiple vectors")]
    OracleReturnsMultipleVectors { location: Location },
    #[error("Dependency cycle found, '{item}' recursively depends on itself: {cycle} ")]
    DependencyCycle { location: Location, item: String, cycle: String },
    #[error("break/continue are only allowed in unconstrained functions")]
    JumpInConstrainedFn { is_break: bool, location: Location },
    #[error("`loop` is only allowed in unconstrained functions")]
    LoopInConstrainedFn { location: Location },
    #[error("`loop` must have at least one `break` in it")]
    LoopWithoutBreak { location: Location },
    #[error("`while` is only allowed in unconstrained functions")]
    WhileInConstrainedFn { location: Location },
    #[error("break/continue are only allowed within loops")]
    JumpOutsideLoop { is_break: bool, location: Location },
    #[error("Only `comptime` globals can be mutable")]
    MutableGlobal { location: Location },
    #[error("Globals must have a specified type")]
    UnspecifiedGlobalType {
        pattern_location: Location,
        expr_location: Location,
        expected_type: Type,
    },
    #[error("Global failed to evaluate")]
    UnevaluatedGlobalType { location: Location },
    #[error("Globals used in a type position must be non-negative")]
    NegativeGlobalType { location: Location, global_value: Value },
    #[error("Globals used in a type position must be integers")]
    NonIntegralGlobalType { location: Location, global_value: Value },
    #[error("Global value `{global_value}` does not fit its kind's range")]
    GlobalDoesNotFitItsType { location: Location, global_value: SignedField, kind: Kind },
    #[error("Self-referential types are not supported")]
    SelfReferentialType { location: Location },
    #[error("#[no_predicates] attribute is only allowed on constrained functions")]
    NoPredicatesAttributeOnUnconstrained { ident: Ident, location: Location },
    #[error("#[no_predicates] attribute is not allowed on entry point functions")]
    NoPredicatesAttributeOnEntryPoint { ident: Ident, location: Location },
    #[error("#[fold] attribute is only allowed on constrained functions")]
    FoldAttributeOnUnconstrained { ident: Ident, location: Location },
    #[error("The unquote operator '$' can only be used within a quote expression")]
    UnquoteUsedOutsideQuote { location: Location },
    #[error("Invalid syntax in macro call")]
    InvalidSyntaxInMacroCall { location: Location },
    #[error("Macros must be comptime functions")]
    MacroIsNotComptime { location: Location },
    #[error("Annotation name must refer to a comptime function")]
    NonFunctionInAnnotation { location: Location },
    #[error(
        "Type `{typ}` was inserted into the generics vector from a macro, but is not a generic"
    )]
    MacroResultInGenericsVectorNotAGeneric { location: Location, typ: Type },
    #[error("Named type arguments aren't allowed in a {item_kind}")]
    NamedTypeArgs { location: Location, item_kind: &'static str },
    #[error("Associated constants may only be a field or integer type")]
    AssociatedConstantsMustBeNumeric { location: Location },
    #[error("Computing `{lhs} {op} {rhs}` failed with error {err}")]
    BinaryOpError {
        lhs: SignedField,
        op: crate::BinaryTypeOperator,
        rhs: SignedField,
        err: Box<TypeCheckError>,
        location: Location,
    },
    #[error("`quote` cannot be used in runtime code")]
    QuoteInRuntimeCode { location: Location },
    #[error("Comptime-only type `{typ}` cannot be used in runtime code")]
    ComptimeTypeInRuntimeCode { typ: String, location: Location },
    #[error("Comptime-only type `{typ}` cannot be used in non-comptime global")]
    ComptimeTypeInNonComptimeGlobal { typ: String, location: Location },
    #[error("Comptime variable `{name}` cannot be mutated in a non-comptime context")]
    MutatingComptimeInNonComptimeContext { name: String, location: Location },
    #[error("Failed to parse `{statement}` as an expression")]
    InvalidInternedStatementInExpr { statement: String, location: Location },
    #[error("{0}")]
    UnsupportedNumericGenericType(#[from] UnsupportedNumericGenericType),
    #[error("Type `{typ}` is more private than item `{item}`")]
    TypeIsMorePrivateThenItem { typ: String, item: String, location: Location },
    #[error("Attribute function `{function}` is not a path")]
    AttributeFunctionIsNotAPath { function: String, location: Location },
    #[error("Attribute function `{name}` is not in scope")]
    AttributeFunctionNotInScope { name: String, location: Location },
    #[error("The trait `{missing_trait}` is not implemented for `{type_missing_trait}`")]
    TraitNotImplemented {
        impl_trait: String,
        missing_trait: String,
        type_missing_trait: String,
        location: Location,
        missing_trait_location: Location,
    },
    #[error("Expected a trait but found {found}")]
    ExpectedTrait { found: String, location: Location },
    #[error("Invalid syntax in match pattern")]
    InvalidSyntaxInPattern { location: Location },
    #[error("Variable '{existing}' was already defined in the same match pattern")]
    VariableAlreadyDefinedInPattern { existing: Ident, new_location: Location },
    #[error("Only integer globals can be used in match patterns")]
    NonIntegerGlobalUsedInPattern { location: Location },
    #[error("Cannot match on values of type `{typ}`")]
    TypeUnsupportedInMatch { typ: Type, location: Location },
    #[error("Expected a struct, enum, or literal value in pattern, but found a {item}")]
    UnexpectedItemInPattern { location: Location, item: &'static str },
    #[error("Trait `{trait_name}` doesn't have a method named `{method_name}`")]
    NoSuchMethodInTrait { trait_name: String, method_name: String, location: Location },
    #[error("Cannot use a type alias inside a type alias")]
    RecursiveTypeAlias { location: Location },
    #[error("expected numeric expressions, got {typ}")]
    ExpectedNumericExpression { typ: String, location: Location },
    #[error(
        "Indexing an array or vector with a type other than `u32` is deprecated and will soon be an error"
    )]
    NonU32Index { location: Location },
    #[error(
        "The type parameter `{ident}` is not constrained by the impl trait, self type, or predicates"
    )]
    UnconstrainedTypeParameter { ident: Ident },
    #[error("Unreachable statement")]
    UnreachableStatement { location: Location, break_or_continue_location: Location },
    #[error("Associated item constraints are not allowed here")]
    AssociatedItemConstraintsNotAllowedInGenerics { location: Location },
    #[error("Ambiguous associated type")]
    AmbiguousAssociatedType { trait_name: String, associated_type_name: String, location: Location },
    #[error("The placeholder `_` is not allowed within types on item signatures for functions")]
    WildcardTypeDisallowed { location: Location, context: WildcardDisallowedContext },
    #[error("References are not allowed in globals")]
    ReferencesNotAllowedInGlobals { location: Location },
    #[error("Functions marked with #[oracle] must have no body")]
    OracleWithBody { location: Location },
    #[error("Builtin and low-level function declarations cannot have a body")]
    BuiltinWithBody { location: Location },
    #[error("Identifier `{ident}` is bound more than once in the same pattern")]
    PatternBoundMoreThanOnce { ident: Ident },
    #[error("{visibility} attribute is only allowed on entry point functions")]
    DataBusOnNonEntryPoint { visibility: String, ident: Ident },
}

impl ResolverError {
    pub fn location(&self) -> Location {
        match self {
            ResolverError::DuplicateDefinition { second_location: location, .. }
            | ResolverError::UnconditionalRecursion { location, .. }
            | ResolverError::Expected { location, .. }
            | ResolverError::VariableNotDeclared { location, .. }
            | ResolverError::MissingFields { location, .. }
            | ResolverError::UnnecessaryMut { second_mut: location, .. }
            | ResolverError::TypeIsMorePrivateThenItem { location, .. }
            | ResolverError::AttributeFunctionIsNotAPath { location, .. }
            | ResolverError::AttributeFunctionNotInScope { location, .. }
            | ResolverError::TraitNotImplemented { location, .. }
            | ResolverError::ExpectedTrait { location, .. }
            | ResolverError::NonStructUsedInConstructor { location, .. }
            | ResolverError::GenericsOnSelfType { location }
            | ResolverError::GenericsOnAssociatedType { location }
            | ResolverError::InvalidClosureEnvironment { location, .. }
            | ResolverError::NestedVectors { location }
            | ResolverError::AbiAttributeOutsideContract { location }
            | ResolverError::DependencyCycle { location, .. }
            | ResolverError::JumpInConstrainedFn { location, .. }
            | ResolverError::LoopInConstrainedFn { location }
            | ResolverError::LoopWithoutBreak { location }
            | ResolverError::WhileInConstrainedFn { location }
            | ResolverError::JumpOutsideLoop { location, .. }
            | ResolverError::MutableGlobal { location }
            | ResolverError::UnspecifiedGlobalType { pattern_location: location, .. }
            | ResolverError::UnevaluatedGlobalType { location }
            | ResolverError::NegativeGlobalType { location, .. }
            | ResolverError::NonIntegralGlobalType { location, .. }
            | ResolverError::GlobalDoesNotFitItsType { location, .. }
            | ResolverError::SelfReferentialType { location }
            | ResolverError::UnquoteUsedOutsideQuote { location }
            | ResolverError::InvalidSyntaxInMacroCall { location }
            | ResolverError::MacroIsNotComptime { location }
            | ResolverError::NonFunctionInAnnotation { location }
            | ResolverError::MacroResultInGenericsVectorNotAGeneric { location, .. }
            | ResolverError::NamedTypeArgs { location, .. }
            | ResolverError::AssociatedConstantsMustBeNumeric { location }
            | ResolverError::BinaryOpError { location, .. }
            | ResolverError::QuoteInRuntimeCode { location }
            | ResolverError::ComptimeTypeInRuntimeCode { location, .. }
            | ResolverError::ComptimeTypeInNonComptimeGlobal { location, .. }
            | ResolverError::MutatingComptimeInNonComptimeContext { location, .. }
            | ResolverError::InvalidInternedStatementInExpr { location, .. }
            | ResolverError::InvalidSyntaxInPattern { location }
            | ResolverError::NonIntegerGlobalUsedInPattern { location, .. }
            | ResolverError::TypeUnsupportedInMatch { location, .. }
            | ResolverError::UnexpectedItemInPattern { location, .. }
            | ResolverError::NoSuchMethodInTrait { location, .. }
            | ResolverError::VariableAlreadyDefinedInPattern { new_location: location, .. }
            | ResolverError::ExpectedNumericExpression { location, .. }
            | ResolverError::RecursiveTypeAlias { location } => *location,
            ResolverError::NonU32Index { location }
            | ResolverError::NoPredicatesAttributeOnUnconstrained { location, .. }
            | ResolverError::NoPredicatesAttributeOnEntryPoint { location, .. }
            | ResolverError::FoldAttributeOnUnconstrained { location, .. }
            | ResolverError::OracleMarkedAsConstrained { location, .. }
            | ResolverError::OracleReturnsMultipleVectors { location, .. }
            | ResolverError::LowLevelFunctionOutsideOfStdlib { location }
            | ResolverError::UnreachableStatement { location, .. }
            | ResolverError::AssociatedItemConstraintsNotAllowedInGenerics { location }
            | ResolverError::AmbiguousAssociatedType { location, .. }
            | ResolverError::WildcardTypeDisallowed { location, .. }
            | ResolverError::ReferencesNotAllowedInGlobals { location }
            | ResolverError::OracleWithBody { location }
            | ResolverError::BuiltinWithBody { location } => *location,
            ResolverError::UnusedVariable { ident }
            | ResolverError::UnusedItem { ident, .. }
            | ResolverError::DuplicateField { field: ident }
            | ResolverError::NoSuchField { field: ident, .. }
            | ResolverError::UnnecessaryPub { ident, .. }
            | ResolverError::NecessaryPub { ident }
            | ResolverError::UnconstrainedTypeParameter { ident }
            | ResolverError::DataBusOnNonEntryPoint { ident, .. }
            | ResolverError::PatternBoundMoreThanOnce { ident } => ident.location(),
            ResolverError::PathResolutionError(path_resolution_error) => {
                path_resolution_error.location()
            }
            ResolverError::NoSuchNumericTypeVariable { path } => path.location,
            ResolverError::ParserError(parser_error) => parser_error.location(),
            ResolverError::UnsupportedNumericGenericType(unsupported_numeric_generic_type) => {
                unsupported_numeric_generic_type.location
            }
        }
    }
}

impl<'a> From<&'a ResolverError> for Diagnostic {
    /// Only user errors can be transformed into a Diagnostic
    /// ICEs will make the compiler panic, as they could affect the
    /// soundness of the generated program
    fn from(error: &'a ResolverError) -> Diagnostic {
        match error {
            ResolverError::DuplicateDefinition { name, first_location, second_location} => {
                let mut diag = Diagnostic::simple_error(
                    format!("duplicate definitions of {name} found"),
                    "second definition found here".to_string(),
                    *second_location,
                );
                diag.add_secondary("first definition found here".to_string(), *first_location);
                diag
            }
            ResolverError::UnusedVariable { ident } => {
                let mut diagnostic = Diagnostic::simple_warning(
                    format!("unused variable {ident}"),
                    "unused variable".to_string(),
                    ident.location(),
                );
                diagnostic.unnecessary = true;
                diagnostic
            }
            ResolverError::UnusedItem { ident, item} => {
                let item_type = item.item_type();

                let mut diagnostic =
                    if let UnusedItem::Struct(..) = item {
                        Diagnostic::simple_warning(
                            format!("{item_type} `{ident}` is never constructed"),
                            format!("{item_type} is never constructed"),
                            ident.location(),
                        )
                    } else {
                        Diagnostic::simple_warning(
                            format!("unused {item_type} {ident}"),
                            format!("unused {item_type}"),
                            ident.location(),
                        )
                    };
                diagnostic.unnecessary = true;
                diagnostic
            }
            ResolverError::UnconditionalRecursion { name, location} => {
                Diagnostic::simple_warning(
                    format!("function `{name}` cannot return without recursing"),
                    "function cannot return without recursing".to_string(),
                    *location,
                )
            }
            ResolverError::VariableNotDeclared { name, location } =>  {
                if name == "_" {
                    Diagnostic::simple_error(
                        "in expressions, `_` can only be used on the left-hand side of an assignment".to_string(),
                        "`_` not allowed here".to_string(),
                        *location,
                    )
                } else {
                    Diagnostic::simple_error(
                        format!("cannot find `{name}` in this scope"),
                        "not found in this scope".to_string(),
                        *location,
                    )
                }
            }
            ResolverError::PathResolutionError(error) => error.into(),
            ResolverError::Expected { location, expected, got } => Diagnostic::simple_error(
                format!("expected {expected} got {got}"),
                String::new(),
                *location,
            ),
            ResolverError::DuplicateField { field } => Diagnostic::simple_error(
                format!("duplicate field {field}"),
                String::new(),
                field.location(),
            ),
            ResolverError::NoSuchField { field, struct_definition } => {
                Diagnostic::simple_error(
                    format!("no such field {field} defined in struct {struct_definition}"),
                    String::new(),
                    field.location(),
                )
            }
            ResolverError::MissingFields { location, missing_fields, struct_definition } => {
                let plural = if missing_fields.len() != 1 { "s" } else { "" };
                let remaining_fields_names = match &missing_fields[..] {
                    [field1] => field1.clone(),
                    [field1, field2] => format!("{field1} and {field2}"),
                    [field1, field2, field3] => format!("{field1}, {field2} and {field3}"),
                    _ => {
                        let len = missing_fields.len() - 3;
                        let len_plural = if len != 1 {"s"} else {""};

                        let truncated_fields = format!(" and {len} other field{len_plural}");
                        let missing_fields = &missing_fields[0..3];
                        format!("{}{truncated_fields}", missing_fields.join(", "))
                    }
                };

                Diagnostic::simple_error(
                    format!("missing field{plural} {remaining_fields_names} in struct {struct_definition}"),
                    String::new(),
                    *location,
                )
            }
            ResolverError::UnnecessaryMut { first_mut, second_mut } => {
                let mut error = Diagnostic::simple_error(
                    "'mut' here is not necessary".to_owned(),
                    "".to_owned(),
                    *second_mut,
                );
                error.add_secondary(
                    "Pattern was already made mutable from this 'mut'".to_owned(),
                    *first_mut,
                );
                error
            }
            ResolverError::UnnecessaryPub { ident, position } => {
                let mut diag = Diagnostic::simple_error(
                    format!("unnecessary pub keyword on {position} for function {ident}"),
                    format!("unnecessary pub {position}"),
                    ident.location(),
                );
                diag.add_note("The `pub` keyword only has effects on arguments to the entry-point function of a program. Thus, adding it to other function parameters can be deceiving and should be removed".to_owned());
                diag
            }
            ResolverError::NecessaryPub { ident } => {
                let mut diag = Diagnostic::simple_error(
                    format!("missing pub keyword on return type of function {ident}"),
                    "missing pub on return type".to_string(),
                    ident.location(),
                );
                diag.add_note("The `pub` keyword is mandatory for the entry-point function return type because the verifier cannot retrieve private witness and thus the function will not be able to return a 'priv' value".to_owned());
                diag
            }
            ResolverError::NoSuchNumericTypeVariable { path } => Diagnostic::simple_error(
                format!("Cannot find a global or generic type parameter named `{path}`"),
                "Only globals or generic type parameters are allowed to be used as an array type's length".to_string(),
                path.location,
            ),
            ResolverError::NonStructUsedInConstructor { typ, location } => Diagnostic::simple_error(
                "Only struct types can be used in constructor expressions".into(),
                format!("{typ} has no fields to construct it with"),
                *location,
            ),
            ResolverError::GenericsOnSelfType { location } => Diagnostic::simple_error(
                "Cannot apply generics to Self type".into(),
                "Use an explicit type name or apply the generics at the start of the impl instead".into(),
                *location,
            ),
            ResolverError::GenericsOnAssociatedType { location } => Diagnostic::simple_error(
                "Generic Associated Types (GATs) are currently unsupported in Noir".into(),
                "Cannot apply generics to an associated type".into(),
                *location,
            ),
            ResolverError::ParserError(error) => error.as_ref().into(),
            ResolverError::InvalidClosureEnvironment { location, typ } => Diagnostic::simple_error(
                format!("{typ} is not a valid closure environment type"),
                "Closure environment must be a tuple or unit type".to_string(), *location
            ),
            ResolverError::NestedVectors { location } => Diagnostic::simple_error(
                "Nested vectors, i.e. vectors within an array or vector, are not supported".into(),
                "Try to use a constant sized array or BoundedVec instead".into(),
                *location,
            ),
            ResolverError::AbiAttributeOutsideContract { location } => {
                Diagnostic::simple_error(
                    "#[abi(tag)] attributes can only be used in contracts".to_string(),
                    "misplaced #[abi(tag)] attribute".to_string(),
                    *location,
                )
            },
            ResolverError::LowLevelFunctionOutsideOfStdlib { location } => Diagnostic::simple_error(
                "Definition of low-level function outside of standard library".into(),
                "Usage of the `#[foreign]` or `#[builtin]` function attributes are not allowed outside of the Noir standard library".into(),
                *location,
            ),
            ResolverError::OracleMarkedAsConstrained { ident, location } => {
                let mut diagnostic = Diagnostic::simple_error(
                    error.to_string(),
                    String::new(),
                    *location,
                );
                diagnostic.add_secondary("Oracle functions must have the `unconstrained` keyword applied".into(), ident.location());
                diagnostic
            },
            ResolverError::OracleReturnsMultipleVectors { location } => {
                Diagnostic::simple_error(
                    error.to_string(),
                    String::new(),
                    *location,
                )
            },
            ResolverError::DependencyCycle { location, item, cycle } => {
                Diagnostic::simple_error(
                    "Dependency cycle found".into(),
                    format!("'{item}' recursively depends on itself: {cycle}"),
                    *location,
                )
            },
            ResolverError::JumpInConstrainedFn { is_break, location } => {
                let item = if *is_break { "break" } else { "continue" };
                Diagnostic::simple_error(
                    format!("{item} is only allowed in unconstrained functions"),
                    "Constrained code must always have a known number of loop iterations".into(),
                    *location,
                )
            },
            ResolverError::LoopInConstrainedFn { location } => {
                Diagnostic::simple_error(
                    "`loop` is only allowed in unconstrained functions".into(),
                    "Constrained code must always have a known number of loop iterations".into(),
                    *location,
                )
            },
            ResolverError::LoopWithoutBreak { location } => {
                Diagnostic::simple_error(
                    "`loop` must have at least one `break` in it".into(),
                    "Infinite loops are disallowed".into(),
                    *location,
                )
            },
            ResolverError::WhileInConstrainedFn { location } => {
                Diagnostic::simple_error(
                    "`while` is only allowed in unconstrained functions".into(),
                    "Constrained code must always have a known number of loop iterations".into(),
                    *location,
                )
            },
            ResolverError::JumpOutsideLoop { is_break, location } => {
                let item = if *is_break { "break" } else { "continue" };
                Diagnostic::simple_error(
                    format!("{item} is only allowed within loops"),
                    "".into(),
                    *location,
                )
            },
            ResolverError::MutableGlobal { location } => {
                Diagnostic::simple_error(
                    "Only `comptime` globals may be mutable".into(),
                    String::new(),
                    *location,
                )
            },
            ResolverError::UnspecifiedGlobalType { pattern_location, expr_location, expected_type } => {
                let mut diagnostic = Diagnostic::simple_error(
                    "Globals must have a specified type".to_string(),
                    String::new(),
                    *pattern_location,
                );
                diagnostic.add_secondary(format!("Inferred type is `{expected_type}`"), *expr_location);
                diagnostic
            },
            ResolverError::UnevaluatedGlobalType { location } => {
                Diagnostic::simple_error(
                    "Global failed to evaluate".to_string(),
                    String::new(),
                    *location,
                )
            }
            ResolverError::NegativeGlobalType { location, global_value } => {
                Diagnostic::simple_error(
                    "Globals used in a type position must be non-negative".to_string(),
                    format!("But found value `{global_value:?}`"),
                    *location,
                )
            }
            ResolverError::NonIntegralGlobalType { location, global_value } => {
                Diagnostic::simple_error(
                    "Globals used in a type position must be integers".to_string(),
                    format!("But found value `{global_value:?}`"),
                    *location,
                )
            }
            ResolverError::GlobalDoesNotFitItsType { location, global_value, kind } => {
                Diagnostic::simple_error(
                    format!("Global value `{global_value}` is larger than its kind's maximum value"),
                    format!("Global's kind inferred to be `{kind}`"),
                    *location,
                )
            }
            ResolverError::SelfReferentialType { location } => {
                Diagnostic::simple_error(
                    "Self-referential types are not supported".into(),
                    "".into(),
                    *location,
                )
            },
            ResolverError::NoPredicatesAttributeOnUnconstrained { ident, location } => {
                let mut diag = Diagnostic::simple_error(
                    format!("misplaced #[no_predicates] attribute on unconstrained function {ident}. Only allowed on constrained functions"),
                    "misplaced #[no_predicates] attribute".to_string(),
                    *location,
                );

                diag.add_note("The `#[no_predicates]` attribute specifies to the compiler whether it should diverge from auto-inlining constrained functions".to_owned());
                diag
            }
            ResolverError::NoPredicatesAttributeOnEntryPoint { ident, location } => {
                let mut diag = Diagnostic::simple_error(
                    format!("#[no_predicates] attribute is not allowed on entry point function {ident}"),
                    "#[no_predicates] attribute not allowed on entry points".to_string(),
                    *location,
                );

                diag.add_note("The `#[no_predicates]` attribute is used to prevent inlining of a function into the entry point, but applying it to the entry point itself has no effect".to_owned());
                diag
            }
            ResolverError::FoldAttributeOnUnconstrained { ident, location } => {
                let mut diag = Diagnostic::simple_error(
                    format!("misplaced #[fold] attribute on unconstrained function {ident}. Only allowed on constrained functions"),
                    "misplaced #[fold] attribute".to_string(),
                    *location,
                );

                diag.add_note("The `#[fold]` attribute specifies whether a constrained function should be treated as a separate circuit rather than inlined into the program entry point".to_owned());
                diag
            }
            ResolverError::UnquoteUsedOutsideQuote { location } => {
                Diagnostic::simple_error(
                    "The unquote operator '$' can only be used within a quote expression".into(),
                    "".into(),
                    *location,
                )
            },
            ResolverError::InvalidSyntaxInMacroCall { location } => {
                Diagnostic::simple_error(
                    "Invalid syntax in macro call".into(),
                    "Macro calls must call a comptime function directly, they cannot use higher-order functions".into(),
                    *location,
                )
            },
            ResolverError::MacroIsNotComptime { location } => {
                Diagnostic::simple_error(
                    "This macro call is to a non-comptime function".into(),
                    "Macro calls must be to comptime functions".into(),
                    *location,
                )
            },
            ResolverError::NonFunctionInAnnotation { location } => {
                Diagnostic::simple_error(
                    "Unknown annotation".into(),
                    "The name of an annotation must refer to a comptime function".into(),
                    *location,
                )
            },
            ResolverError::MacroResultInGenericsVectorNotAGeneric { location, typ } => {
                Diagnostic::simple_error(
                    format!("Type `{typ}` was inserted into a generics vector from a macro, but it is not a generic"),
                    format!("Type `{typ}` is not a generic"),
                    *location,
                )
            }
            ResolverError::NamedTypeArgs { location, item_kind } => {
                Diagnostic::simple_error(
                    format!("Named type arguments aren't allowed on a {item_kind}"),
                    "Named type arguments are only allowed for associated types on traits".to_string(),
                    *location,
                )
            }
            ResolverError::AssociatedConstantsMustBeNumeric { location } => {
                Diagnostic::simple_error(
                    "Associated constants may only be a field or integer type".to_string(),
                    "Only numeric constants are allowed".to_string(),
                    *location,
                )
            }
            ResolverError::BinaryOpError { lhs, op, rhs, err, location } => {
                Diagnostic::simple_error(
                    format!("Computing `{lhs} {op} {rhs}` failed with error {err}"),
                    String::new(),
                    *location,
                )
            }
            ResolverError::QuoteInRuntimeCode { location } => {
                Diagnostic::simple_error(
                    "`quote` cannot be used in runtime code".to_string(),
                    "Wrap this in a `comptime` block or function to use it".to_string(),
                    *location,
                )
            },
            ResolverError::ComptimeTypeInRuntimeCode { typ, location } => {
                Diagnostic::simple_error(
                    format!("Comptime-only type `{typ}` cannot be used in runtime code"),
                    "Comptime-only type used here".to_string(),
                    *location,
                )
            },
            ResolverError::ComptimeTypeInNonComptimeGlobal { typ, location } => {
                Diagnostic::simple_error(
                    format!("Comptime-only type `{typ}` cannot be used in non-comptime global"),
                    "Comptime-only type used here".to_string(),
                    *location,
                )
            },
            ResolverError::MutatingComptimeInNonComptimeContext { name, location } => {
                Diagnostic::simple_error(
                    format!("Comptime variable `{name}` cannot be mutated in a non-comptime context"),
                    format!("`{name}` mutated here"),
                    *location,
                )
            },
            ResolverError::InvalidInternedStatementInExpr { statement, location } => {
                Diagnostic::simple_error(
                    format!("Failed to parse `{statement}` as an expression"),
                    "The statement was used from a macro here".to_string(),
                    *location,
                )
            },
            ResolverError::UnsupportedNumericGenericType(err) => err.into(),
            ResolverError::TypeIsMorePrivateThenItem { typ, item, location } => {
                Diagnostic::simple_error(
                    format!("Type `{typ}` is more private than item `{item}`"),
                    String::new(),
                    *location,
                )
            },
            ResolverError::AttributeFunctionIsNotAPath { function, location } => {
                Diagnostic::simple_error(
                    format!("Attribute function `{function}` is not a path"),
                    "An attribute's function should be a single identifier or a path".into(),
                    *location,
                )
            },
            ResolverError::AttributeFunctionNotInScope { name, location } => {
                Diagnostic::simple_error(
                    format!("Attribute function `{name}` is not in scope"),
                    String::new(),
                    *location,
                )
            },
            ResolverError::TraitNotImplemented { impl_trait, missing_trait: the_trait, type_missing_trait: typ, location, missing_trait_location} => {
                let mut diagnostic = Diagnostic::simple_error(
                    format!("The trait bound `{typ}: {the_trait}` is not satisfied"),
                    format!("The trait `{the_trait}` is not implemented for `{typ}`")
                    , *location);
                diagnostic.add_secondary(format!("required by this bound in `{impl_trait}`"), *missing_trait_location);
                diagnostic
            },
            ResolverError::ExpectedTrait { found, location  } => {
                Diagnostic::simple_error(
                    format!("Expected a trait, found {found}"),
                    String::new(),
                    *location,
                )
            }
            ResolverError::InvalidSyntaxInPattern { location } => {
                Diagnostic::simple_error(
                    "Invalid syntax in match pattern".into(),
                    "Only literal, constructor, and variable patterns are allowed".into(),
                    *location,
                )
            },
            ResolverError::VariableAlreadyDefinedInPattern { existing, new_location } => {
                let message = format!("Variable `{existing}` was already defined in the same match pattern");
                let secondary = format!("`{existing}` redefined here");
                let mut error = Diagnostic::simple_error(message, secondary, *new_location);
                error.add_secondary(format!("`{existing}` was previously defined here"), existing.location());
                error
            },
            ResolverError::NonIntegerGlobalUsedInPattern { location } => {
                let message = "Only integer or boolean globals can be used in match patterns".to_string();
                let secondary = "This global is not an integer or boolean".to_string();
                Diagnostic::simple_error(message, secondary, *location)
            },
            ResolverError::TypeUnsupportedInMatch { typ, location } => {
                Diagnostic::simple_error(
                    format!("Cannot match on values of type `{typ}`"),
                    String::new(),
                    *location,
                )
            },
            ResolverError::UnexpectedItemInPattern { item, location } => {
                Diagnostic::simple_error(
                    format!("Expected a struct, enum, or literal pattern, but found a {item}"),
                    String::new(),
                    *location,
                )
            },
            ResolverError::NoSuchMethodInTrait { trait_name, method_name, location } => {
                Diagnostic::simple_error(
                    format!("Trait `{trait_name}` has no method named `{method_name}`"),
                    String::new(),
                    *location,
                )
            },
            ResolverError::RecursiveTypeAlias { location } => {
                Diagnostic::simple_error(
                    "Cannot use a type alias inside a type alias".to_string(),
                    String::new(),
                    *location,
                )
            },
            ResolverError::ExpectedNumericExpression { typ, location } => {
                Diagnostic::simple_error(
                    format!("Expected a numeric expression, but got `{typ}`"),
                    String::new(),
                    *location,
                )
            },
            ResolverError::NonU32Index { location } => {
                Diagnostic::simple_warning(
                    "Indexing an array or vector with a type other than `u32` is deprecated and will soon be an error".to_string(),
                    String::new(),
                    *location,
                )
            },
            ResolverError::UnconstrainedTypeParameter { ident} => {
                Diagnostic::simple_error(
                    format!("The type parameter `{ident}` is not constrained by the impl trait, self type, or predicates"),
                    format!("Hint: remove the `{ident}` type parameter"),
                    ident.location(),
                )
            }
            ResolverError::UnreachableStatement { location, break_or_continue_location} => {
                let mut diagnostic = Diagnostic::simple_warning(
                    "Unreachable statement".to_string(),
                    "Unreachable statement".to_string(),
                    *location,
                );
                diagnostic.add_secondary("Any code following this expression is unreachable".to_string(), *break_or_continue_location);
                diagnostic
            }
            ResolverError::AssociatedItemConstraintsNotAllowedInGenerics { location} => {
                Diagnostic::simple_error(
                    "Associated item constraints are not allowed here".to_string(),
                    "Consider removing this associated item binding".to_string(),
                    *location,
                )
            }
            ResolverError::AmbiguousAssociatedType { trait_name, associated_type_name, location } => {
                Diagnostic::simple_error(
                    "Ambiguous associated type".to_string(),
                    format!("If there were a type named `Example` that implemented `{trait_name}`, you could use the fully-qualified path: `<Example as {trait_name}>::{associated_type_name}`"),
                    *location,
                )
            }
            ResolverError::WildcardTypeDisallowed { location, context: reason } => {
                let context = match reason {
                    WildcardDisallowedContext::AssociatedType => "associated type definitions",
                    WildcardDisallowedContext::Cast => "casts",
                    WildcardDisallowedContext::EnumVariant => "enum variant definitions",
                    WildcardDisallowedContext::FunctionReturn => "function return types",
                    WildcardDisallowedContext::FunctionParameter => "function parameter types",
                    WildcardDisallowedContext::Global => "global definitions",
                    WildcardDisallowedContext::ImplType => "impl types",
                    WildcardDisallowedContext::NumericGeneric => "numeric generics",
                    WildcardDisallowedContext::QuotedAsType => "the argument to Quoted::as_type",
                    WildcardDisallowedContext::StructField => "struct field types",
                    WildcardDisallowedContext::TraitAsType => "impl trait types",
                    WildcardDisallowedContext::TraitBound => "trait bounds",
                    WildcardDisallowedContext::TraitConstraint => "trait constraints",
                    WildcardDisallowedContext::TraitImplType => "trait impl types",
                    WildcardDisallowedContext::TypeAlias => "type alias definitions",
                };
                Diagnostic::simple_error(
                    format!("The placeholder `_` is not allowed in {context}"),
                    String::new(),
                    *location,
                )
            }
            ResolverError::ReferencesNotAllowedInGlobals { location } => {
                Diagnostic::simple_error(
                    "References are not allowed in globals".to_string(),
                    String::new(),
                    *location,
                )
            }
            ResolverError::OracleWithBody { location } => {
                Diagnostic::simple_error(
                    "Functions marked with #[oracle] must have no body".to_string(),
                    "This function body will never be run so should be removed".to_string(),
                    *location,
                )
            }
            ResolverError::BuiltinWithBody { location } => {
                Diagnostic::simple_error(
                    "Builtin and low-level function declarations cannot have a body".to_string(),
                    "This function body should be removed".to_string(),
                    *location,
                )
            }
            ResolverError::PatternBoundMoreThanOnce { ident } => {
                Diagnostic::simple_error(
                    format!("Identifier `{ident}` is bound more than once in the same pattern"),
                    "Used in a pattern more than once".to_string(),
                    ident.location(),
                )
            }
            ResolverError::DataBusOnNonEntryPoint { visibility, ident } => {
                let mut diag = Diagnostic::simple_error(
                    format!("unnecessary {visibility} attribute for function {ident}"),
                    format!("unnecessary {visibility}"),
                    ident.location(),
                );
                diag.add_note(
                    format!("The {visibility} attribute only has effects for the entry-point function of a program. Thus, adding it to other function can be deceiving and should be removed)"));
                diag
            },
        }
    }
}
