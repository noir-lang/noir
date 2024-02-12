pub use noirc_errors::Span;
use noirc_errors::{CustomDiagnostic as Diagnostic, FileDiagnostic};
use thiserror::Error;

use crate::{parser::ParserError, Ident, Type};

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
    #[error("Could not find variable in this scope")]
    VariableNotDeclared { name: String, span: Span },
    #[error("path is not an identifier")]
    PathIsNotIdent { span: Span },
    #[error("could not resolve path")]
    PathResolutionError(PathResolutionError),
    #[error("Expected")]
    Expected { span: Span, expected: String, got: String },
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
    #[error("'distinct' keyword can only be used with main method")]
    DistinctNotAllowed { ident: Ident },
    #[error("Missing expression for declared constant")]
    MissingRhsExpr { name: String, span: Span },
    #[error("Expression invalid in an array length context")]
    InvalidArrayLengthExpr { span: Span },
    #[error("Integer too large to be evaluated in an array length context")]
    IntegerTooLarge { span: Span },
    #[error("No global or generic type parameter found with the given name")]
    NoSuchNumericTypeVariable { path: crate::Path },
    #[error("Closures cannot capture mutable variables")]
    CapturedMutableVariable { span: Span },
    #[error("Test functions are not allowed to have any parameters")]
    TestFunctionHasParameters { span: Span },
    #[error("Only struct types can be used in constructor expressions")]
    NonStructUsedInConstructor { typ: Type, span: Span },
    #[error("Only struct types can have generics")]
    NonStructWithGenerics { span: Span },
    #[error("Cannot apply generics on Self type")]
    GenericsOnSelfType { span: Span },
    #[error("Incorrect amount of arguments to {item_name}")]
    IncorrectGenericCount { span: Span, item_name: String, actual: usize, expected: usize },
    #[error("{0}")]
    ParserError(Box<ParserError>),
    #[error("Function is not defined in a contract yet sets its contract visibility")]
    ContractFunctionTypeInNormalFunction { span: Span },
    #[error("Cannot create a mutable reference to {variable}, it was declared to be immutable")]
    MutableReferenceToImmutableVariable { variable: String, span: Span },
    #[error("Mutable references to array indices are unsupported")]
    MutableReferenceToArrayElement { span: Span },
    #[error("Function is not defined in a contract yet sets is_internal")]
    ContractFunctionInternalInNormalFunction { span: Span },
    #[error("Numeric constants should be printed without formatting braces")]
    NumericConstantInFormatString { name: String, span: Span },
    #[error("Closure environment must be a tuple or unit type")]
    InvalidClosureEnvironment { typ: Type, span: Span },
    #[error("{name} is private and not visible from the current module")]
    PrivateFunctionCalled { name: String, span: Span },
    #[error("{name} is not visible from the current crate")]
    NonCrateFunctionCalled { name: String, span: Span },
    #[error("Only sized types may be used in the entry point to a program")]
    InvalidTypeForEntryPoint { span: Span },
    #[error("Nested slices are not supported")]
    NestedSlices { span: Span },
    #[error("#[recursive] attribute is only allowed on entry points to a program")]
    MisplacedRecursiveAttribute { ident: Ident },
    #[error("Usage of the `#[foreign]` or `#[builtin]` function attributes are not allowed outside of the Noir standard library")]
    LowLevelFunctionOutsideOfStdlib { ident: Ident },
    #[error("Dependency cycle found, '{item}' recursively depends on itself: {cycle} ")]
    DependencyCycle { span: Span, item: String, cycle: String },
}

impl ResolverError {
    pub fn into_file_diagnostic(self, file: fm::FileId) -> FileDiagnostic {
        Diagnostic::from(self).in_file(file)
    }
}

impl From<ResolverError> for Diagnostic {
    /// Only user errors can be transformed into a Diagnostic
    /// ICEs will make the compiler panic, as they could affect the
    /// soundness of the generated program
    fn from(error: ResolverError) -> Diagnostic {
        match error {
            ResolverError::DuplicateDefinition { name, first_span, second_span } => {
                let mut diag = Diagnostic::simple_error(
                    format!("duplicate definitions of {name} found"),
                    "first definition found here".to_string(),
                    first_span,
                );
                diag.add_secondary("second definition found here".to_string(), second_span);
                diag
            }
            ResolverError::UnusedVariable { ident } => {
                let name = &ident.0.contents;

                Diagnostic::simple_warning(
                    format!("unused variable {name}"),
                    "unused variable ".to_string(),
                    ident.span(),
                )
            }
            ResolverError::VariableNotDeclared { name, span } => Diagnostic::simple_error(
                format!("cannot find `{name}` in this scope "),
                "not found in this scope".to_string(),
                span,
            ),
            ResolverError::PathIsNotIdent { span } => Diagnostic::simple_error(
                "cannot use path as an identifier".to_string(),
                String::new(),
                span,
            ),
            ResolverError::PathResolutionError(error) => error.into(),
            ResolverError::Expected { span, expected, got } => Diagnostic::simple_error(
                format!("expected {expected} got {got}"),
                String::new(),
                span,
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
            ResolverError::MissingFields { span, mut missing_fields, struct_definition } => {
                let plural = if missing_fields.len() != 1 { "s" } else { "" };
                let remaining_fields_names = match &missing_fields[..] {
                    [field1] => field1.clone(),
                    [field1, field2] => format!("{field1} and {field2}"),
                    [field1, field2, field3] => format!("{field1}, {field2} and {field3}"),
                    _ => {
                        let len = missing_fields.len() - 3;
                        let len_plural = if len != 1 {"s"} else {""};

                        let truncated_fields = format!(" and {len} other field{len_plural}");
                        missing_fields.truncate(3);
                        format!("{}{truncated_fields}", missing_fields.join(", "))
                    }
                };

                Diagnostic::simple_error(
                    format!("missing field{plural} {remaining_fields_names} in struct {struct_definition}"),
                    String::new(),
                    span,
                )
            }
            ResolverError::UnnecessaryMut { first_mut, second_mut } => {
                let mut error = Diagnostic::simple_error(
                    "'mut' here is not necessary".to_owned(),
                    "".to_owned(),
                    second_mut,
                );
                error.add_secondary(
                    "Pattern was already made mutable from this 'mut'".to_owned(),
                    first_mut,
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
            ResolverError::DistinctNotAllowed { ident } => {
                let name = &ident.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("Invalid `distinct` keyword on return type of function {name}"),
                    "Invalid distinct on return type".to_string(),
                    ident.0.span(),
                );

                diag.add_note("The `distinct` keyword is only valid when used on the main function of a program, as its only purpose is to ensure that all witness indices that occur in the abi are unique".to_owned());
                diag
            }
            ResolverError::MissingRhsExpr { name, span } => Diagnostic::simple_error(
                format!(
                    "no expression specifying the value stored by the constant variable {name}"
                ),
                "expected expression to be stored for let statement".to_string(),
                span,
            ),
            ResolverError::InvalidArrayLengthExpr { span } => Diagnostic::simple_error(
                "Expression invalid in an array-length context".into(),
                "Array-length expressions can only have simple integer operations and any variables used must be global constants".into(),
                span,
            ),
            ResolverError::IntegerTooLarge { span } => Diagnostic::simple_error(
                "Integer too large to be evaluated to an array-length".into(),
                "Array-lengths may be a maximum size of usize::MAX, including intermediate calculations".into(),
                span,
            ),
            ResolverError::NoSuchNumericTypeVariable { path } => Diagnostic::simple_error(
                format!("Cannot find a global or generic type parameter named `{path}`"),
                "Only globals or generic type parameters are allowed to be used as an array type's length".to_string(),
                path.span(),
            ),
            ResolverError::CapturedMutableVariable { span } => Diagnostic::simple_error(
                "Closures cannot capture mutable variables".into(),
                "Mutable variable".into(),
                span,
            ),
            ResolverError::TestFunctionHasParameters { span } => Diagnostic::simple_error(
                "Test functions cannot have any parameters".into(),
                "Try removing the parameters or moving the test into a wrapper function".into(),
                span,
            ),
            ResolverError::NonStructUsedInConstructor { typ, span } => Diagnostic::simple_error(
                "Only struct types can be used in constructor expressions".into(),
                format!("{typ} has no fields to construct it with"),
                span,
            ),
            ResolverError::NonStructWithGenerics { span } => Diagnostic::simple_error(
                "Only struct types can have generic arguments".into(),
                "Try removing the generic arguments".into(),
                span,
            ),
            ResolverError::GenericsOnSelfType { span } => Diagnostic::simple_error(
                "Cannot apply generics to Self type".into(),
                "Use an explicit type name or apply the generics at the start of the impl instead".into(),
                span,
            ),
            ResolverError::IncorrectGenericCount { span, item_name, actual, expected } => {
                let expected_plural = if expected == 1 { "" } else { "s" };
                let actual_plural = if actual == 1 { "is" } else { "are" };

                Diagnostic::simple_error(
                    format!("`{item_name}` has {expected} generic argument{expected_plural} but {actual} {actual_plural} given here"),
                    "Incorrect number of generic arguments".into(),
                    span,
                )
            }
            ResolverError::ParserError(error) => (*error).into(),
            ResolverError::ContractFunctionTypeInNormalFunction { span } => Diagnostic::simple_error(
                "Only functions defined within contracts can set their contract function type".into(),
                "Non-contract functions cannot be 'open'".into(),
                span,
            ),
            ResolverError::MutableReferenceToImmutableVariable { variable, span } => {
                Diagnostic::simple_error(format!("Cannot mutably reference the immutable variable {variable}"), format!("{variable} is immutable"), span)
            },
            ResolverError::MutableReferenceToArrayElement { span } => {
                Diagnostic::simple_error("Mutable references to array elements are currently unsupported".into(), "Try storing the element in a fresh variable first".into(), span)
            },
            ResolverError::ContractFunctionInternalInNormalFunction { span } => Diagnostic::simple_error(
                "Only functions defined within contracts can set their functions to be internal".into(),
                "Non-contract functions cannot be 'internal'".into(),
                span,
            ),
            ResolverError::NumericConstantInFormatString { name, span } => Diagnostic::simple_error(
                format!("cannot find `{name}` in this scope "),
                "Numeric constants should be printed without formatting braces".to_string(),
                span,
            ),
            ResolverError::InvalidClosureEnvironment { span, typ } => Diagnostic::simple_error(
                format!("{typ} is not a valid closure environment type"),
                "Closure environment must be a tuple or unit type".to_string(), span),
            // This will be upgraded to an error in future versions
            ResolverError::PrivateFunctionCalled { span, name } => Diagnostic::simple_warning(
                format!("{name} is private and not visible from the current module"),
                format!("{name} is private"), span),
            ResolverError::NonCrateFunctionCalled { span, name } => Diagnostic::simple_warning(
                    format!("{name} is not visible from the current crate"),
                    format!("{name} is only visible within its crate"), span),
            ResolverError::InvalidTypeForEntryPoint { span } => Diagnostic::simple_error(
                "Only sized types may be used in the entry point to a program".to_string(),
                "Slices, references, or any type containing them may not be used in main or a contract function".to_string(), span),
            ResolverError::NestedSlices { span } => Diagnostic::simple_error(
                "Nested slices are not supported".into(),
                "Try to use a constant sized array instead".into(),
                span,
            ),
            ResolverError::MisplacedRecursiveAttribute { ident } => {
                let name = &ident.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("misplaced #[recursive] attribute on function {name} rather than the main function"),
                    "misplaced #[recursive] attribute".to_string(),
                    ident.0.span(),
                );

                diag.add_note("The `#[recursive]` attribute specifies to the backend whether it should use a prover which generates proofs that are friendly for recursive verification in another circuit".to_owned());
                diag
            }
            ResolverError::LowLevelFunctionOutsideOfStdlib { ident } => Diagnostic::simple_error(
                "Definition of low-level function outside of standard library".into(),
                "Usage of the `#[foreign]` or `#[builtin]` function attributes are not allowed outside of the Noir standard library".into(),
                ident.span(),
            ),
            ResolverError::DependencyCycle { span, item, cycle } => {
                Diagnostic::simple_error(
                    "Dependency cycle found".into(),
                    format!("'{item}' recursively depends on itself: {cycle}"),
                    span,
                )
            },
        }
    }
}
