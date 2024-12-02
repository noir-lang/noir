use acvm::FieldElement;
pub use noirc_errors::Span;
use noirc_errors::{CustomDiagnostic as Diagnostic, FileDiagnostic, Location};
use thiserror::Error;

use crate::{
    ast::{Ident, UnsupportedNumericGenericType},
    hir::{comptime::InterpreterError, type_check::TypeCheckError},
    parser::ParserError,
    usage_tracker::UnusedItem,
    Type,
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
    DuplicateDefinition { name: String, first_span: Span, second_span: Span },
    #[error("Unused variable")]
    UnusedVariable { ident: Ident },
    #[error("Unused {}", item.item_type())]
    UnusedItem { ident: Ident, item: UnusedItem },
    #[error("Unconditional recursion")]
    UnconditionalRecursion { name: String, span: Span },
    #[error("Could not find variable in this scope")]
    VariableNotDeclared { name: String, span: Span },
    #[error("path is not an identifier")]
    PathIsNotIdent { span: Span },
    #[error("could not resolve path")]
    PathResolutionError(#[from] PathResolutionError),
    #[error("Expected")]
    Expected { span: Span, expected: &'static str, got: &'static str },
    #[error("Duplicate field in constructor")]
    DuplicateField { field: Ident },
    #[error("No such field in struct")]
    NoSuchField { field: Ident, struct_definition: Ident },
    #[error("Missing fields from struct")]
    MissingFields { span: Span, missing_fields: Vec<String>, struct_definition: Ident },
    #[error("Unneeded 'mut', pattern is already marked as mutable")]
    UnnecessaryMut { first_mut: Span, second_mut: Span },
    #[error("Unneeded 'pub', function is not the main method")]
    UnnecessaryPub { ident: Ident, position: PubPosition },
    #[error("Required 'pub', main function must return public value")]
    NecessaryPub { ident: Ident },
    #[error("Missing expression for declared constant")]
    MissingRhsExpr { name: String, span: Span },
    #[error("Expression invalid in an array length context")]
    InvalidArrayLengthExpr { span: Span },
    #[error("Integer too large to be evaluated in an array length context")]
    IntegerTooLarge { span: Span },
    #[error("No global or generic type parameter found with the given name")]
    NoSuchNumericTypeVariable { path: crate::ast::Path },
    #[error("Closures cannot capture mutable variables")]
    CapturedMutableVariable { span: Span },
    #[error("Test functions are not allowed to have any parameters")]
    TestFunctionHasParameters { span: Span },
    #[error("Only struct types can be used in constructor expressions")]
    NonStructUsedInConstructor { typ: String, span: Span },
    #[error("Only struct types can have generics")]
    NonStructWithGenerics { span: Span },
    #[error("Cannot apply generics on Self type")]
    GenericsOnSelfType { span: Span },
    #[error("Cannot apply generics on an associated type")]
    GenericsOnAssociatedType { span: Span },
    #[error("{0}")]
    ParserError(Box<ParserError>),
    #[error("Cannot create a mutable reference to {variable}, it was declared to be immutable")]
    MutableReferenceToImmutableVariable { variable: String, span: Span },
    #[error("Mutable references to array indices are unsupported")]
    MutableReferenceToArrayElement { span: Span },
    #[error("Numeric constants should be printed without formatting braces")]
    NumericConstantInFormatString { name: String, span: Span },
    #[error("Closure environment must be a tuple or unit type")]
    InvalidClosureEnvironment { typ: Type, span: Span },
    #[error("Nested slices, i.e. slices within an array or slice, are not supported")]
    NestedSlices { span: Span },
    #[error("#[abi(tag)] attribute is only allowed in contracts")]
    AbiAttributeOutsideContract { span: Span },
    #[error("Usage of the `#[foreign]` or `#[builtin]` function attributes are not allowed outside of the Noir standard library")]
    LowLevelFunctionOutsideOfStdlib { ident: Ident },
    #[error(
        "Usage of the `#[oracle]` function attribute is only valid on unconstrained functions"
    )]
    OracleMarkedAsConstrained { ident: Ident },
    #[error("Oracle functions cannot be called directly from constrained functions")]
    UnconstrainedOracleReturnToConstrained { span: Span },
    #[error("Dependency cycle found, '{item}' recursively depends on itself: {cycle} ")]
    DependencyCycle { span: Span, item: String, cycle: String },
    #[error("break/continue are only allowed in unconstrained functions")]
    JumpInConstrainedFn { is_break: bool, span: Span },
    #[error("break/continue are only allowed within loops")]
    JumpOutsideLoop { is_break: bool, span: Span },
    #[error("Only `comptime` globals can be mutable")]
    MutableGlobal { span: Span },
    #[error("Globals must have a specified type")]
    UnspecifiedGlobalType { span: Span, expected_type: Type },
    #[error("Self-referential structs are not supported")]
    SelfReferentialStruct { span: Span },
    #[error("#[no_predicates] attribute is only allowed on constrained functions")]
    NoPredicatesAttributeOnUnconstrained { ident: Ident },
    #[error("#[fold] attribute is only allowed on constrained functions")]
    FoldAttributeOnUnconstrained { ident: Ident },
    #[error("expected type, found numeric generic parameter")]
    NumericGenericUsedForType { name: String, span: Span },
    #[error("Invalid array length construction")]
    ArrayLengthInterpreter { error: InterpreterError },
    #[error("The unquote operator '$' can only be used within a quote expression")]
    UnquoteUsedOutsideQuote { span: Span },
    #[error("Invalid syntax in macro call")]
    InvalidSyntaxInMacroCall { span: Span },
    #[error("Macros must be comptime functions")]
    MacroIsNotComptime { span: Span },
    #[error("Annotation name must refer to a comptime function")]
    NonFunctionInAnnotation { span: Span },
    #[error("Type `{typ}` was inserted into the generics list from a macro, but is not a generic")]
    MacroResultInGenericsListNotAGeneric { span: Span, typ: Type },
    #[error("Named type arguments aren't allowed in a {item_kind}")]
    NamedTypeArgs { span: Span, item_kind: &'static str },
    #[error("Associated constants may only be a field or integer type")]
    AssociatedConstantsMustBeNumeric { span: Span },
    #[error("Computing `{lhs} {op} {rhs}` failed with error {err}")]
    BinaryOpError {
        lhs: FieldElement,
        op: crate::BinaryTypeOperator,
        rhs: FieldElement,
        err: Box<TypeCheckError>,
        span: Span,
    },
    #[error("`quote` cannot be used in runtime code")]
    QuoteInRuntimeCode { span: Span },
    #[error("Comptime-only type `{typ}` cannot be used in runtime code")]
    ComptimeTypeInRuntimeCode { typ: String, span: Span },
    #[error("Comptime variable `{name}` cannot be mutated in a non-comptime context")]
    MutatingComptimeInNonComptimeContext { name: String, span: Span },
    #[error("Failed to parse `{statement}` as an expression")]
    InvalidInternedStatementInExpr { statement: String, span: Span },
    #[error("{0}")]
    UnsupportedNumericGenericType(#[from] UnsupportedNumericGenericType),
    #[error("Type `{typ}` is more private than item `{item}`")]
    TypeIsMorePrivateThenItem { typ: String, item: String, span: Span },
    #[error("Unable to parse attribute `{attribute}`")]
    UnableToParseAttribute { attribute: String, span: Span },
    #[error("Attribute function `{function}` is not a path")]
    AttributeFunctionIsNotAPath { function: String, span: Span },
    #[error("Attribute function `{name}` is not in scope")]
    AttributeFunctionNotInScope { name: String, span: Span },
    #[error("The trait `{missing_trait}` is not implemented for `{type_missing_trait}")]
    TraitNotImplemented {
        impl_trait: String,
        missing_trait: String,
        type_missing_trait: String,
        span: Span,
        missing_trait_location: Location,
    },
}

impl ResolverError {
    pub fn into_file_diagnostic(&self, file: fm::FileId) -> FileDiagnostic {
        Diagnostic::from(self).in_file(file)
    }
}

impl<'a> From<&'a ResolverError> for Diagnostic {
    /// Only user errors can be transformed into a Diagnostic
    /// ICEs will make the compiler panic, as they could affect the
    /// soundness of the generated program
    fn from(error: &'a ResolverError) -> Diagnostic {
        match error {
            ResolverError::DuplicateDefinition { name, first_span, second_span } => {
                let mut diag = Diagnostic::simple_error(
                    format!("duplicate definitions of {name} found"),
                    "first definition found here".to_string(),
                    *first_span,
                );
                diag.add_secondary("second definition found here".to_string(), *second_span);
                diag
            }
            ResolverError::UnusedVariable { ident } => {
                let name = &ident.0.contents;

                let mut diagnostic = Diagnostic::simple_warning(
                    format!("unused variable {name}"),
                    "unused variable ".to_string(),
                    ident.span(),
                );
                diagnostic.unnecessary = true;
                diagnostic
            }
            ResolverError::UnusedItem { ident, item} => {
                let name = &ident.0.contents;
                let item_type = item.item_type();

                let mut diagnostic =
                    if let UnusedItem::Struct(..) = item {
                        Diagnostic::simple_warning(
                            format!("{item_type} `{name}` is never constructed"),
                            format!("{item_type} is never constructed"),
                            ident.span(),
                        )
                    } else {
                        Diagnostic::simple_warning(
                            format!("unused {item_type} {name}"),
                            format!("unused {item_type}"),
                            ident.span(),
                        )
                    };
                diagnostic.unnecessary = true;
                diagnostic
            }
            ResolverError::UnconditionalRecursion { name, span} => {
                Diagnostic::simple_warning(
                    format!("function `{name}` cannot return without recursing"),
                    "function cannot return without recursing".to_string(),
                    *span,
                )
            }
            ResolverError::VariableNotDeclared { name, span } =>  {
                if name == "_" {
                    Diagnostic::simple_error(
                        "in expressions, `_` can only be used on the left-hand side of an assignment".to_string(),
                        "`_` not allowed here".to_string(),
                        *span,
                    )
                } else {
                    Diagnostic::simple_error(
                        format!("cannot find `{name}` in this scope"),
                        "not found in this scope".to_string(),
                        *span,
                    )
                }
            },
            ResolverError::PathIsNotIdent { span } => Diagnostic::simple_error(
                "cannot use path as an identifier".to_string(),
                String::new(),
                *span,
            ),
            ResolverError::PathResolutionError(error) => error.into(),
            ResolverError::Expected { span, expected, got } => Diagnostic::simple_error(
                format!("expected {expected} got {got}"),
                String::new(),
                *span,
            ),
            ResolverError::DuplicateField { field } => Diagnostic::simple_error(
                format!("duplicate field {field}"),
                String::new(),
                field.span(),
            ),
            ResolverError::NoSuchField { field, struct_definition } => {
                Diagnostic::simple_error(
                    format!("no such field {field} defined in struct {struct_definition}"),
                    String::new(),
                    field.span(),
                )
            }
            ResolverError::MissingFields { span, missing_fields, struct_definition } => {
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
                    *span,
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
                let name = &ident.0.contents;

                let mut diag = Diagnostic::simple_warning(
                    format!("unnecessary pub keyword on {position} for function {name}"),
                    format!("unnecessary pub {position}"),
                    ident.0.span(),
                );

                diag.add_note("The `pub` keyword only has effects on arguments to the entry-point function of a program. Thus, adding it to other function parameters can be deceiving and should be removed".to_owned());
                diag
            }
            ResolverError::NecessaryPub { ident } => {
                let name = &ident.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("missing pub keyword on return type of function {name}"),
                    "missing pub on return type".to_string(),
                    ident.0.span(),
                );

                diag.add_note("The `pub` keyword is mandatory for the entry-point function return type because the verifier cannot retrieve private witness and thus the function will not be able to return a 'priv' value".to_owned());
                diag
            }
            ResolverError::MissingRhsExpr { name, span } => Diagnostic::simple_error(
                format!(
                    "no expression specifying the value stored by the constant variable {name}"
                ),
                "expected expression to be stored for let statement".to_string(),
                *span,
            ),
            ResolverError::InvalidArrayLengthExpr { span } => Diagnostic::simple_error(
                "Expression invalid in an array-length context".into(),
                "Array-length expressions can only have simple integer operations and any variables used must be global constants".into(),
                *span,
            ),
            ResolverError::IntegerTooLarge { span } => Diagnostic::simple_error(
                "Integer too large to be evaluated to an array-length".into(),
                "Array-lengths may be a maximum size of usize::MAX, including intermediate calculations".into(),
                *span,
            ),
            ResolverError::NoSuchNumericTypeVariable { path } => Diagnostic::simple_error(
                format!("Cannot find a global or generic type parameter named `{path}`"),
                "Only globals or generic type parameters are allowed to be used as an array type's length".to_string(),
                path.span(),
            ),
            ResolverError::CapturedMutableVariable { span } => Diagnostic::simple_error(
                "Closures cannot capture mutable variables".into(),
                "Mutable variable".into(),
                *span,
            ),
            ResolverError::TestFunctionHasParameters { span } => Diagnostic::simple_error(
                "Test functions cannot have any parameters".into(),
                "Try removing the parameters or moving the test into a wrapper function".into(),
                *span,
            ),
            ResolverError::NonStructUsedInConstructor { typ, span } => Diagnostic::simple_error(
                "Only struct types can be used in constructor expressions".into(),
                format!("{typ} has no fields to construct it with"),
                *span,
            ),
            ResolverError::NonStructWithGenerics { span } => Diagnostic::simple_error(
                "Only struct types can have generic arguments".into(),
                "Try removing the generic arguments".into(),
                *span,
            ),
            ResolverError::GenericsOnSelfType { span } => Diagnostic::simple_error(
                "Cannot apply generics to Self type".into(),
                "Use an explicit type name or apply the generics at the start of the impl instead".into(),
                *span,
            ),
            ResolverError::GenericsOnAssociatedType { span } => Diagnostic::simple_error(
                "Generic Associated Types (GATs) are currently unsupported in Noir".into(),
                "Cannot apply generics to an associated type".into(),
                *span,
            ),
            ResolverError::ParserError(error) => error.as_ref().into(),
            ResolverError::MutableReferenceToImmutableVariable { variable, span } => {
                Diagnostic::simple_error(format!("Cannot mutably reference the immutable variable {variable}"), format!("{variable} is immutable"), *span)
            },
            ResolverError::MutableReferenceToArrayElement { span } => {
                Diagnostic::simple_error("Mutable references to array elements are currently unsupported".into(), "Try storing the element in a fresh variable first".into(), *span)
            },
            ResolverError::NumericConstantInFormatString { name, span } => Diagnostic::simple_error(
                format!("cannot find `{name}` in this scope "),
                "Numeric constants should be printed without formatting braces".to_string(),
                *span,
            ),
            ResolverError::InvalidClosureEnvironment { span, typ } => Diagnostic::simple_error(
                format!("{typ} is not a valid closure environment type"),
                "Closure environment must be a tuple or unit type".to_string(), *span),
            ResolverError::NestedSlices { span } => Diagnostic::simple_error(
                "Nested slices, i.e. slices within an array or slice, are not supported".into(),
                "Try to use a constant sized array or BoundedVec instead".into(),
                *span,
            ),
            ResolverError::AbiAttributeOutsideContract { span } => {
                Diagnostic::simple_error(
                    "#[abi(tag)] attributes can only be used in contracts".to_string(),
                    "misplaced #[abi(tag)] attribute".to_string(),
                    *span,
                )
            },
            ResolverError::LowLevelFunctionOutsideOfStdlib { ident } => Diagnostic::simple_error(
                "Definition of low-level function outside of standard library".into(),
                "Usage of the `#[foreign]` or `#[builtin]` function attributes are not allowed outside of the Noir standard library".into(),
                ident.span(),
            ),
            ResolverError::OracleMarkedAsConstrained { ident } => Diagnostic::simple_warning(
                error.to_string(),
                "Oracle functions must have the `unconstrained` keyword applied".into(),
                ident.span(),
            ),
            ResolverError::UnconstrainedOracleReturnToConstrained { span } => Diagnostic::simple_error(
                error.to_string(),
                "This oracle call must be wrapped in a call to another unconstrained function before being returned to a constrained runtime".into(),
                *span,
            ),
            ResolverError::DependencyCycle { span, item, cycle } => {
                Diagnostic::simple_error(
                    "Dependency cycle found".into(),
                    format!("'{item}' recursively depends on itself: {cycle}"),
                    *span,
                )
            },
            ResolverError::JumpInConstrainedFn { is_break, span } => {
                let item = if *is_break { "break" } else { "continue" };
                Diagnostic::simple_error(
                    format!("{item} is only allowed in unconstrained functions"),
                    "Constrained code must always have a known number of loop iterations".into(),
                    *span,
                )
            },
            ResolverError::JumpOutsideLoop { is_break, span } => {
                let item = if *is_break { "break" } else { "continue" };
                Diagnostic::simple_error(
                    format!("{item} is only allowed within loops"),
                    "".into(),
                    *span,
                )
            },
            ResolverError::MutableGlobal { span } => {
                Diagnostic::simple_error(
                    "Only `comptime` globals may be mutable".into(),
                    String::new(),
                    *span,
                )
            },
            ResolverError::UnspecifiedGlobalType { span, expected_type } => {
                Diagnostic::simple_error(
                    "Globals must have a specified type".to_string(),
                    format!("Inferred type is `{expected_type}`"),
                    *span,
                )
            },
            ResolverError::SelfReferentialStruct { span } => {
                Diagnostic::simple_error(
                    "Self-referential structs are not supported".into(),
                    "".into(),
                    *span,
                )
            },
            ResolverError::NoPredicatesAttributeOnUnconstrained { ident } => {
                let name = &ident.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("misplaced #[no_predicates] attribute on unconstrained function {name}. Only allowed on constrained functions"),
                    "misplaced #[no_predicates] attribute".to_string(),
                    ident.0.span(),
                );

                diag.add_note("The `#[no_predicates]` attribute specifies to the compiler whether it should diverge from auto-inlining constrained functions".to_owned());
                diag
            }
            ResolverError::FoldAttributeOnUnconstrained { ident } => {
                let name = &ident.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("misplaced #[fold] attribute on unconstrained function {name}. Only allowed on constrained functions"),
                    "misplaced #[fold] attribute".to_string(),
                    ident.0.span(),
                );

                diag.add_note("The `#[fold]` attribute specifies whether a constrained function should be treated as a separate circuit rather than inlined into the program entry point".to_owned());
                diag
            }
            ResolverError::NumericGenericUsedForType { name, span } => {
                Diagnostic::simple_error(
                    format!("expected type, found numeric generic parameter {name}"),
                    String::from("not a type"),
                    *span,
                )
            }
            ResolverError::ArrayLengthInterpreter { error } => Diagnostic::from(error),
            ResolverError::UnquoteUsedOutsideQuote { span } => {
                Diagnostic::simple_error(
                    "The unquote operator '$' can only be used within a quote expression".into(),
                    "".into(),
                    *span,
                )
            },
            ResolverError::InvalidSyntaxInMacroCall { span } => {
                Diagnostic::simple_error(
                    "Invalid syntax in macro call".into(),
                    "Macro calls must call a comptime function directly, they cannot use higher-order functions".into(),
                    *span,
                )
            },
            ResolverError::MacroIsNotComptime { span } => {
                Diagnostic::simple_error(
                    "This macro call is to a non-comptime function".into(),
                    "Macro calls must be to comptime functions".into(),
                    *span,
                )
            },
            ResolverError::NonFunctionInAnnotation { span } => {
                Diagnostic::simple_error(
                    "Unknown annotation".into(),
                    "The name of an annotation must refer to a comptime function".into(),
                    *span,
                )
            },
            ResolverError::MacroResultInGenericsListNotAGeneric { span, typ } => {
                Diagnostic::simple_error(
                    format!("Type `{typ}` was inserted into a generics list from a macro, but it is not a generic"),
                    format!("Type `{typ}` is not a generic"),
                    *span,
                )
            }
            ResolverError::NamedTypeArgs { span, item_kind } => {
                Diagnostic::simple_error(
                    format!("Named type arguments aren't allowed on a {item_kind}"),
                    "Named type arguments are only allowed for associated types on traits".to_string(),
                    *span,
                )
            }
            ResolverError::AssociatedConstantsMustBeNumeric { span } => {
                Diagnostic::simple_error(
                    "Associated constants may only be a field or integer type".to_string(),
                    "Only numeric constants are allowed".to_string(),
                    *span,
                )
            }
            ResolverError::BinaryOpError { lhs, op, rhs, err, span } => {
                Diagnostic::simple_error(
                    format!("Computing `{lhs} {op} {rhs}` failed with error {err}"),
                    String::new(),
                    *span,
                )
            }
            ResolverError::QuoteInRuntimeCode { span } => {
                Diagnostic::simple_error(
                    "`quote` cannot be used in runtime code".to_string(),
                    "Wrap this in a `comptime` block or function to use it".to_string(),
                    *span,
                )
            },
            ResolverError::ComptimeTypeInRuntimeCode { typ, span } => {
                Diagnostic::simple_error(
                    format!("Comptime-only type `{typ}` cannot be used in runtime code"),
                    "Comptime-only type used here".to_string(),
                    *span,
                )
            },
            ResolverError::MutatingComptimeInNonComptimeContext { name, span } => {
                Diagnostic::simple_error(
                    format!("Comptime variable `{name}` cannot be mutated in a non-comptime context"),
                    format!("`{name}` mutated here"),
                    *span,
                )
            },
            ResolverError::InvalidInternedStatementInExpr { statement, span } => {
                Diagnostic::simple_error(
                    format!("Failed to parse `{statement}` as an expression"),
                    "The statement was used from a macro here".to_string(),
                    *span,
                )
            },
            ResolverError::UnsupportedNumericGenericType(err) => err.into(),
            ResolverError::TypeIsMorePrivateThenItem { typ, item, span } => {
                Diagnostic::simple_warning(
                    format!("Type `{typ}` is more private than item `{item}`"),
                    String::new(),
                    *span,
                )
            },
            ResolverError::UnableToParseAttribute { attribute, span } => {
                Diagnostic::simple_error(
                    format!("Unable to parse attribute `{attribute}`"),
                    "Attribute should be a function or function call".into(),
                    *span,
                )
            },
            ResolverError::AttributeFunctionIsNotAPath { function, span } => {
                Diagnostic::simple_error(
                    format!("Attribute function `{function}` is not a path"),
                    "An attribute's function should be a single identifier or a path".into(),
                    *span,
                )
            },
            ResolverError::AttributeFunctionNotInScope { name, span } => {
                Diagnostic::simple_error(
                    format!("Attribute function `{name}` is not in scope"),
                    String::new(),
                    *span,
                )
            },
            ResolverError::TraitNotImplemented { impl_trait, missing_trait: the_trait, type_missing_trait: typ, span, missing_trait_location} => {
                let mut diagnostic = Diagnostic::simple_error(
                    format!("The trait bound `{typ}: {the_trait}` is not satisfied"), 
                    format!("The trait `{the_trait}` is not implemented for `{typ}")
                    , *span);
                diagnostic.add_secondary_with_file(format!("required by this bound in `{impl_trait}"), missing_trait_location.span, missing_trait_location.file);
                diagnostic
            },
        }
    }
}
