use acvm::FieldElement;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::Span;
use thiserror::Error;

use crate::hir::resolution::errors::ResolverError;
use crate::hir_def::expr::HirBinaryOp;
use crate::hir_def::types::Type;
use crate::BinaryOpKind;
use crate::FunctionReturnType;
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
    #[error("{0}")]
    BinOp(BinaryOpKind),
    #[error("Return")]
    Return(FunctionReturnType, Span),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TypeCheckError {
    #[error("Operator {op:?} cannot be used in a {place:?}")]
    OpCannotBeUsed { op: HirBinaryOp, place: &'static str, span: Span },
    #[error("The literal `{expr:?}` cannot fit into `{ty}` which has range `{range}`")]
    OverflowingAssignment { expr: FieldElement, ty: Type, range: String, span: Span },
    #[error("Type {typ:?} cannot be used in a {place:?}")]
    TypeCannotBeUsed { typ: Type, place: &'static str, span: Span },
    #[error("Expected type {expected_typ:?} is not the same as {expr_typ:?}")]
    TypeMismatch { expected_typ: String, expr_typ: String, expr_span: Span },
    #[error("Expected type {expected} is not the same as {actual}")]
    TypeMismatchWithSource { expected: Type, actual: Type, span: Span, source: Source },
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
    #[error("{kind} cannot be used in a unary operation")]
    InvalidUnaryOp { kind: String, span: Span },
    #[error("Bitwise operations are invalid on Field types. Try casting the operands to a sized integer type first.")]
    InvalidBitwiseOperationOnField { span: Span },
    #[error("Integer cannot be used with type {typ}")]
    IntegerTypeMismatch { typ: Type, span: Span },
    #[error("Cannot use an integer and a Field in a binary operation, try converting the Field into an integer first")]
    IntegerAndFieldBinaryOperation { span: Span },
    #[error("Cannot do modulo on Fields, try casting to an integer first")]
    FieldModulo { span: Span },
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
    #[error("Unused expression result of type {expr_type}")]
    UnusedResultError { expr_type: Type, expr_span: Span },
    #[error("Expected type {expected_typ:?} is not the same as {actual_typ:?}")]
    TraitMethodParameterTypeMismatch {
        method_name: String,
        expected_typ: String,
        actual_typ: String,
        parameter_span: Span,
        parameter_index: usize,
    },
    #[error("No matching impl found")]
    NoMatchingImplFound { constraints: Vec<(Type, String)>, span: Span },
    #[error("Constraint for `{typ}: {trait_name}` is not needed, another matching impl is already in scope")]
    UnneededTraitConstraint { trait_name: String, typ: Type, span: Span },
    #[error(
        "Cannot pass a mutable reference from a constrained runtime to an unconstrained runtime"
    )]
    ConstrainedReferenceToUnconstrained { span: Span },
    #[error("Slices cannot be returned from an unconstrained runtime to a constrained runtime")]
    UnconstrainedSliceReturnToConstrained { span: Span },
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
            TypeCheckError::TraitMethodParameterTypeMismatch { method_name, expected_typ, actual_typ, parameter_index, parameter_span } => {
                Diagnostic::simple_error(
                    format!("Parameter #{parameter_index} of method `{method_name}` must be of type {expected_typ}, not {actual_typ}"),
                    String::new(),
                    parameter_span,
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
            | TypeCheckError::UnsupportedCast { span }
            | TypeCheckError::TupleIndexOutOfBounds { span, .. }
            | TypeCheckError::VariableMustBeMutable { span, .. }
            | TypeCheckError::UnresolvedMethodCall { span, .. }
            | TypeCheckError::InvalidComparisonOnField { span }
            | TypeCheckError::IntegerSignedness { span, .. }
            | TypeCheckError::IntegerBitWidth { span, .. }
            | TypeCheckError::InvalidInfixOp { span, .. }
            | TypeCheckError::InvalidUnaryOp { span, .. }
            | TypeCheckError::InvalidBitwiseOperationOnField { span, .. }
            | TypeCheckError::IntegerTypeMismatch { span, .. }
            | TypeCheckError::FieldComparison { span, .. }
            | TypeCheckError::AmbiguousBitWidth { span, .. }
            | TypeCheckError::IntegerAndFieldBinaryOperation { span }
            | TypeCheckError::OverflowingAssignment { span, .. }
            | TypeCheckError::FieldModulo { span }
            | TypeCheckError::ConstrainedReferenceToUnconstrained { span }
            | TypeCheckError::UnconstrainedSliceReturnToConstrained { span } => {
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
            TypeCheckError::TypeMismatchWithSource { expected, actual, span, source } => {
                let message = match source {
                    Source::Binary => format!("Types in a binary operation should match, but found {expected} and {actual}"),
                    Source::Assignment => {
                        format!("Cannot assign an expression of type {actual} to a value of type {expected}")
                    }
                    Source::ArrayElements => format!("Cannot compare {expected} and {actual}, the array element types differ"),
                    Source::ArrayLen => format!("Can only compare arrays of the same length. Here LHS is of length {expected}, and RHS is {actual}"),
                    Source::StringLen => format!("Can only compare strings of the same length. Here LHS is of length {expected}, and RHS is {actual}"),
                    Source::Comparison => format!("Unsupported types for comparison: {expected} and {actual}"),
                    Source::BinOp(kind) => format!("Unsupported types for operator `{kind}`: {expected} and {actual}"),
                    Source::Return(ret_ty, expr_span) => {
                        let ret_ty_span = match ret_ty.clone() {
                            FunctionReturnType::Default(span) => span,
                            FunctionReturnType::Ty(ty) => ty.span.unwrap(),
                        };

                        let mut diagnostic = Diagnostic::simple_error(format!("expected type {expected}, found type {actual}"), format!("expected {expected} because of return type"), ret_ty_span);

                        if let FunctionReturnType::Default(_) = ret_ty {
                            diagnostic.add_note(format!("help: try adding a return type: `-> {actual}`"));
                        }

                        diagnostic.add_secondary(format!("{actual} returned here"), expr_span);

                        return diagnostic
                    },
                };

                Diagnostic::simple_error(message, String::new(), span)
            }
            TypeCheckError::CallDeprecated { span, ref note, .. } => {
                let primary_message = error.to_string();
                let secondary_message = note.clone().unwrap_or_default();

                Diagnostic::simple_warning(primary_message, secondary_message, span)
            }
            TypeCheckError::UnusedResultError { expr_type, expr_span } => {
                let msg = format!("Unused expression result of type {expr_type}");
                Diagnostic::simple_warning(msg, String::new(), expr_span)
            }
            TypeCheckError::NoMatchingImplFound { constraints, span } => {
                assert!(!constraints.is_empty());
                let msg = format!("No matching impl found for `{}: {}`", constraints[0].0, constraints[0].1);
                let mut diagnostic = Diagnostic::from_message(&msg);

                diagnostic.add_secondary(format!("No impl for `{}: {}`", constraints[0].0, constraints[0].1), span);

                // These must be notes since secondaries are unordered
                for (typ, trait_name) in &constraints[1..] {
                    diagnostic.add_note(format!("Required by `{typ}: {trait_name}`"));
                }

                diagnostic
            }
            TypeCheckError::UnneededTraitConstraint { trait_name, typ, span } => {
                let msg = format!("Constraint for `{typ}: {trait_name}` is not needed, another matching impl is already in scope");
                Diagnostic::simple_warning(msg, "Unnecessary trait constraint in where clause".into(), span)
            }
        }
    }
}
