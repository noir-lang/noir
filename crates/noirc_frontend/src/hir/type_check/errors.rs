use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::Span;
use thiserror::Error;

use crate::hir::resolution::errors::ResolverError;
use crate::hir_def::expr::HirBinaryOp;
use crate::hir_def::types::Type;
use crate::Signedness;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Source {
    #[error("Binary")]
    Binary,
    #[error("Assignment")]
    Assignment,
    #[error("ArrayElements")]
    ArrayElements,
    #[error("ArrayLen")]
    ArrayLen,
    #[error("StringLen")]
    StringLen,
    #[error("Comparison")]
    Comparison,
    #[error("BinOp")]
    BinOp,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TypeCheckError {
    #[error("Operator {op:?} cannot be used in a {place:?}")]
    OpCannotBeUsed { op: HirBinaryOp, place: &'static str, span: Span },
    #[error("Type {typ:?} cannot be used in a {place:?}")]
    TypeCannotBeUsed { typ: Type, place: &'static str, span: Span },
    #[error("Expected type {expected_typ:?} is not the same as {expr_typ:?}")]
    TypeMismatch { expected_typ: String, expr_typ: String, expr_span: Span },
    #[error("Expected type {lhs} is not the same as {rhs}")]
    TypeMismatchWithSource { lhs: Type, rhs: Type, span: Span, source: Source },
    #[error("Expected {expected:?} found {found:?}")]
    ArityMisMatch { expected: u16, found: u16, span: Span },
    #[error("Return type in a function cannot be public")]
    PublicReturnType { typ: Type, span: Span },
    #[error("Cannot cast type {from}, 'as' is only for primitive field or integer types")]
    InvalidCast { from: Type, span: Span },
    #[error("Expected a function, but found a(n) {found}")]
    ExpectedFunction { found: Type, span: Span },
    #[error("Type {lhs_type} has no member named {field_name}")]
    AccessUnknownMember { lhs_type: Type, field_name: String, span: Span },
    #[error("Function expects {expected} parameters but {found} given")]
    ParameterCountMismatch { expected: usize, found: usize, span: Span },
    #[error("The value is non-comptime because of this expression, which uses another non-comptime value")]
    NotCompTime { span: Span },
    #[error(
        "The value is comptime because of this expression, which forces the value to be comptime"
    )]
    CompTime { span: Span },
    #[error("Cannot cast to a comptime type, argument to cast is not known at compile-time")]
    CannotCastToComptimeType { span: Span },
    #[error("Only integer and Field types may be casted to")]
    UnsupportedCast { span: Span },
    #[error("Index {index} is out of bounds for this tuple {lhs_type} of length {length}")]
    TupleIndexOutOfBounds { index: usize, lhs_type: Type, length: usize, span: Span },
    #[error("Variable {name} must be mutable to be assigned to")]
    VariableMustBeMutable { name: String, span: Span },
    #[error("No method named '{method_name}' found for type '{object_type}'")]
    UnresolvedMethodCall { method_name: String, object_type: Type, span: Span },
    #[error("Comparisons are invalid on Field types. Try casting the operands to a sized integer type first")]
    InvalidComparisonOnField { span: Span },
    #[error("Integers must have the same signedness LHS is {sign_x:?}, RHS is {sign_y:?}")]
    IntegerSignedness { sign_x: Signedness, sign_y: Signedness, span: Span },
    #[error("Integers must have the same bit width LHS is {bit_width_x}, RHS is {bit_width_y}")]
    IntegerBitWidth { bit_width_x: u32, bit_width_y: u32, span: Span },
    #[error("{kind} cannot be used in an infix operation")]
    InvalidInfixOp { kind: &'static str, span: Span },
    #[error("Bitwise operations are invalid on Field types. Try casting the operands to a sized integer type first.")]
    InvalidBitwiseOperationOnField { span: Span },
    #[error("Integer cannot be used with type {typ}")]
    IntegerTypeMismatch { typ: Type, span: Span },
    #[error("Cannot use an integer and a Field in a binary operation, try converting the Field into an integer first")]
    IntegerAndFieldBinaryOperation { span: Span },
    #[error("Fields cannot be compared, try casting to an integer first")]
    FieldComparison { span: Span },
    #[error("The number of bits to use for this bitwise operation is ambiguous. Either the operand's type or return type should be specified")]
    AmbiguousBitWidth { span: Span },
    #[error("Error with additional context")]
    Context { err: Box<TypeCheckError>, ctx: &'static str },
    #[error("Array is not homogeneous")]
    NonHomogeneousArray {
        first_span: Span,
        first_type: String,
        first_index: usize,
        second_span: Span,
        second_type: String,
        second_index: usize,
    },
    #[error("Cannot infer type of expression, type annotations needed before this point")]
    TypeAnnotationsNeeded { span: Span },
    #[error("use of deprecated function {name}")]
    CallDeprecated { name: String, note: Option<String>, span: Span },
    #[error("{0}")]
    ResolverError(ResolverError),
}

impl TypeCheckError {
    pub fn add_context(self, ctx: &'static str) -> Self {
        TypeCheckError::Context { err: Box::new(self), ctx }
    }
}

impl From<TypeCheckError> for Diagnostic {
    fn from(error: TypeCheckError) -> Diagnostic {
        match error {
            TypeCheckError::TypeCannotBeUsed { typ, place, span } => Diagnostic::simple_error(
                format!("The type {} cannot be used in a {}", &typ, place),
                String::new(),
                span,
            ),
            TypeCheckError::Context { err, ctx } => {
                let mut diag = Diagnostic::from(*err);
                diag.add_note(ctx.to_owned());
                diag
            }
            TypeCheckError::OpCannotBeUsed { op, place, span } => Diagnostic::simple_error(
                format!("The operator {op:?} cannot be used in a {place}"),
                String::new(),
                span,
            ),
            TypeCheckError::TypeMismatch { expected_typ, expr_typ, expr_span } => {
                Diagnostic::simple_error(
                    format!("Expected type {expected_typ}, found type {expr_typ}"),
                    String::new(),
                    expr_span,
                )
            }
            TypeCheckError::NonHomogeneousArray {
                first_span,
                first_type,
                first_index,
                second_span,
                second_type,
                second_index,
            } => {
                let mut diag = Diagnostic::simple_error(
                    format!(
                        "Non homogeneous array, different element types found at indices ({first_index},{second_index})"
                    ),
                    format!("Found type {first_type}"),
                    first_span,
                );
                diag.add_secondary(format!("but then found type {second_type}"), second_span);
                diag
            }
            TypeCheckError::ArityMisMatch { expected, found, span } => {
                let plural = if expected == 1 { "" } else { "s" };
                let msg = format!("Expected {expected} argument{plural}, but found {found}");
                Diagnostic::simple_error(msg, String::new(), span)
            }
            TypeCheckError::ParameterCountMismatch { expected, found, span } => {
                let empty_or_s = if expected == 1 { "" } else { "s" };
                let was_or_were = if found == 1 { "was" } else { "were" };
                let msg = format!("Function expects {expected} parameter{empty_or_s} but {found} {was_or_were} given");
                Diagnostic::simple_error(msg, String::new(), span)
            }
            TypeCheckError::InvalidCast { span, .. }
            | TypeCheckError::ExpectedFunction { span, .. }
            | TypeCheckError::AccessUnknownMember { span, .. }
            | TypeCheckError::CompTime { span }
            | TypeCheckError::NotCompTime { span }
            | TypeCheckError::CannotCastToComptimeType { span }
            | TypeCheckError::UnsupportedCast { span }
            | TypeCheckError::TupleIndexOutOfBounds { span, .. }
            | TypeCheckError::VariableMustBeMutable { span, .. }
            | TypeCheckError::UnresolvedMethodCall { span, .. }
            | TypeCheckError::InvalidComparisonOnField { span }
            | TypeCheckError::IntegerSignedness { span, .. }
            | TypeCheckError::IntegerBitWidth { span, .. }
            | TypeCheckError::InvalidInfixOp { span, .. }
            | TypeCheckError::InvalidBitwiseOperationOnField { span, .. }
            | TypeCheckError::IntegerTypeMismatch { span, .. }
            | TypeCheckError::FieldComparison { span, .. }
            | TypeCheckError::AmbiguousBitWidth { span, .. }
            | TypeCheckError::IntegerAndFieldBinaryOperation { span } => {
                Diagnostic::simple_error(error.to_string(), String::new(), span)
            }
            TypeCheckError::PublicReturnType { typ, span } => Diagnostic::simple_error(
                "Functions cannot declare a public return type".to_string(),
                format!("return type is {typ}"),
                span,
            ),
            TypeCheckError::TypeAnnotationsNeeded { span } => Diagnostic::simple_error(
                "Expression type is ambiguous".to_string(),
                "Type must be known at this point".to_string(),
                span,
            ),
            TypeCheckError::ResolverError(error) => error.into(),
            TypeCheckError::TypeMismatchWithSource { lhs, rhs, span, source } => {
                let message = match source {
                    Source::Binary => format!("Types in a binary operation should match, but found {lhs} and {rhs}"),
                    Source::Assignment => {
                        format!("Cannot assign an expression of type {lhs} to a value of type {rhs}")
                    }
                    Source::ArrayElements => format!("Cannot compare {lhs} and {rhs}, the array element types differ"),
                    Source::ArrayLen => format!("Can only compare arrays of the same length. Here LHS is of length {lhs}, and RHS is {rhs}"),
                    Source::StringLen => format!("Can only compare strings of the same length. Here LHS is of length {lhs}, and RHS is {rhs}"),
                    Source::Comparison => format!("Unsupported types for comparison: {lhs} and {rhs}"),
                    Source::BinOp => format!("Unsupported types for binary operation: {lhs} and {rhs}"),
                };

                Diagnostic::simple_error(message, String::new(), span)
            }
            TypeCheckError::CallDeprecated { span, ref note, .. } => {
                let mut message = error.to_string();

                if let Some(note) = note {
                    message += &format!(": {note}");
                }

                Diagnostic::simple_warning(message, String::new(), span)
            }
        }
    }
}
