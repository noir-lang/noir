pub use noirc_errors::Span;
use noirc_errors::{CustomDiagnostic as Diagnostic, FileDiagnostic};
use thiserror::Error;

use crate::{parser::ParserError, Ident, Type};

use super::import::PathResolutionError;

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
    UnnecessaryPub { ident: Ident },
    #[error("Required 'pub', main function must return public value")]
    NecessaryPub { ident: Ident },
    #[error("'distinct' keyword can only be used with main method")]
    DistinctNotAllowed { ident: Ident },
    #[error("Expected const value where non-constant value was used")]
    ExpectedComptimeVariable { name: String, span: Span },
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
    #[error("Incorrect amount of arguments to generic type constructor")]
    IncorrectGenericCount { span: Span, struct_type: String, actual: usize, expected: usize },
    #[error("{0}")]
    ParserError(Box<ParserError>),
    #[error("Function is not defined in a contract yet sets its contract visibility")]
    ContractFunctionTypeInNormalFunction { span: Span },
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
                let mut error = Diagnostic::simple_error(
                    format!("no such field {field} defined in struct {struct_definition}"),
                    String::new(),
                    field.span(),
                );

                error.add_secondary(
                    format!("{struct_definition} defined here with no {field} field"),
                    struct_definition.span(),
                );
                error
            }
            ResolverError::MissingFields { span, missing_fields, struct_definition } => {
                let plural = if missing_fields.len() != 1 { "s" } else { "" };
                let missing_fields = missing_fields.join(", ");

                let mut error = Diagnostic::simple_error(
                    format!("missing field{plural}: {missing_fields}"),
                    String::new(),
                    span,
                );

                error.add_secondary(
                    format!("{struct_definition} defined here"),
                    struct_definition.span(),
                );
                error
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
            ResolverError::UnnecessaryPub { ident } => {
                let name = &ident.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("unnecessary pub keyword on parameter for function {name}"),
                    "unnecessary pub parameter".to_string(),
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
            ResolverError::ExpectedComptimeVariable { name, span } => Diagnostic::simple_error(
                format!("expected constant variable where non-constant variable {name} was used"),
                "expected const variable".to_string(),
                span,
            ),
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
            ResolverError::IncorrectGenericCount { span, struct_type, actual, expected } => {
                let expected_plural = if expected == 1 { "" } else { "s" };
                let actual_plural = if actual == 1 { "is" } else { "are" };

                Diagnostic::simple_error(
                    format!("The struct type {struct_type} has {expected} generic{expected_plural} but {actual} {actual_plural} given here"),
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
        }
    }
}
