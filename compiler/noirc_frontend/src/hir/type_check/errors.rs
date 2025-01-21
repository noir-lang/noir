use std::rc::Rc;

use acvm::FieldElement;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::Span;
use thiserror::Error;

use crate::ast::{
    BinaryOpKind, ConstrainKind, FunctionReturnType, Ident, IntegerBitSize, Signedness,
};
use crate::hir::resolution::errors::ResolverError;
use crate::hir_def::expr::HirBinaryOp;
use crate::hir_def::traits::TraitConstraint;
use crate::hir_def::types::{BinaryTypeOperator, Kind, Type};
use crate::node_interner::NodeInterner;

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
    #[error("Division by zero: {lhs} / {rhs}")]
    DivisionByZero { lhs: FieldElement, rhs: FieldElement, span: Span },
    #[error("Modulo on Field elements: {lhs} % {rhs}")]
    ModuloOnFields { lhs: FieldElement, rhs: FieldElement, span: Span },
    #[error("The value `{expr:?}` cannot fit into `{ty}` which has range `{range}`")]
    OverflowingAssignment { expr: FieldElement, ty: Type, range: String, span: Span },
    #[error(
        "The value `{value}` cannot fit into `{kind}` which has a maximum size of `{maximum_size}`"
    )]
    OverflowingConstant { value: FieldElement, kind: Kind, maximum_size: FieldElement, span: Span },
    #[error("Evaluating `{op}` on `{lhs}`, `{rhs}` failed")]
    FailingBinaryOp { op: BinaryTypeOperator, lhs: i128, rhs: i128, span: Span },
    #[error("Type {typ:?} cannot be used in a {place:?}")]
    TypeCannotBeUsed { typ: Type, place: &'static str, span: Span },
    #[error("Expected type {expected_typ:?} is not the same as {expr_typ:?}")]
    TypeMismatch { expected_typ: String, expr_typ: String, expr_span: Span },
    #[error("Expected type {expected} is not the same as {actual}")]
    TypeMismatchWithSource { expected: Type, actual: Type, span: Span, source: Source },
    #[error("Expected type {expected_kind:?} is not the same as {expr_kind:?}")]
    TypeKindMismatch { expected_kind: String, expr_kind: String, expr_span: Span },
    #[error("Evaluating {to} resulted in {to_value}, but {from_value} was expected")]
    TypeCanonicalizationMismatch {
        to: Type,
        from: Type,
        to_value: FieldElement,
        from_value: FieldElement,
        span: Span,
    },
    #[error("Expected {expected:?} found {found:?}")]
    ArityMisMatch { expected: usize, found: usize, span: Span },
    #[error("Return type in a function cannot be public")]
    PublicReturnType { typ: Type, span: Span },
    #[error("Cannot cast type {from}, 'as' is only for primitive field or integer types")]
    InvalidCast { from: Type, span: Span, reason: String },
    #[error("Casting value of type {from} to a smaller type ({to})")]
    DownsizingCast { from: Type, to: Type, span: Span, reason: String },
    #[error("Expected a function, but found a(n) {found}")]
    ExpectedFunction { found: Type, span: Span },
    #[error("Type {lhs_type} has no member named {field_name}")]
    AccessUnknownMember { lhs_type: Type, field_name: String, span: Span },
    #[error("Function expects {expected} parameters but {found} were given")]
    ParameterCountMismatch { expected: usize, found: usize, span: Span },
    #[error("{} expects {} or {} parameters but {found} were given", kind, kind.required_arguments_count(), kind.required_arguments_count() + 1)]
    AssertionParameterCountMismatch { kind: ConstrainKind, found: usize, span: Span },
    #[error("{item} expects {expected} generics but {found} were given")]
    GenericCountMismatch { item: String, expected: usize, found: usize, span: Span },
    #[error("{item} has incompatible `unconstrained`")]
    UnconstrainedMismatch { item: String, expected: bool, span: Span },
    #[error("Only integer and Field types may be casted to")]
    UnsupportedCast { span: Span },
    #[error("Index {index} is out of bounds for this tuple {lhs_type} of length {length}")]
    TupleIndexOutOfBounds { index: usize, lhs_type: Type, length: usize, span: Span },
    #[error("Variable `{name}` must be mutable to be assigned to")]
    VariableMustBeMutable { name: String, span: Span },
    #[error("Cannot mutate immutable variable `{name}`")]
    CannotMutateImmutableVariable { name: String, span: Span },
    #[error("No method named '{method_name}' found for type '{object_type}'")]
    UnresolvedMethodCall { method_name: String, object_type: Type, span: Span },
    #[error("Cannot invoke function field '{method_name}' on type '{object_type}' as a method")]
    CannotInvokeStructFieldFunctionType { method_name: String, object_type: Type, span: Span },
    #[error("Integers must have the same signedness LHS is {sign_x:?}, RHS is {sign_y:?}")]
    IntegerSignedness { sign_x: Signedness, sign_y: Signedness, span: Span },
    #[error("Integers must have the same bit width LHS is {bit_width_x}, RHS is {bit_width_y}")]
    IntegerBitWidth { bit_width_x: IntegerBitSize, bit_width_y: IntegerBitSize, span: Span },
    #[error("{kind} cannot be used in an infix operation")]
    InvalidInfixOp { kind: &'static str, span: Span },
    #[error("{kind} cannot be used in a unary operation")]
    InvalidUnaryOp { kind: String, span: Span },
    #[error("Bitwise operations are invalid on Field types. Try casting the operands to a sized integer type first.")]
    FieldBitwiseOp { span: Span },
    #[error("Integer cannot be used with type {typ}")]
    IntegerTypeMismatch { typ: Type, span: Span },
    #[error("Cannot use an integer and a Field in a binary operation, try converting the Field into an integer first")]
    IntegerAndFieldBinaryOperation { span: Span },
    #[error("Cannot do modulo on Fields, try casting to an integer first")]
    FieldModulo { span: Span },
    #[error("Cannot do not (`!`) on Fields, try casting to an integer first")]
    FieldNot { span: Span },
    #[error("Fields cannot be compared, try casting to an integer first")]
    FieldComparison { span: Span },
    #[error("The bit count in a bit-shift operation must fit in a u8, try casting the right hand side into a u8 first")]
    InvalidShiftSize { span: Span },
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
    #[error("Object type is unknown in method call")]
    TypeAnnotationsNeededForMethodCall { span: Span },
    #[error("Object type is unknown in field access")]
    TypeAnnotationsNeededForFieldAccess { span: Span },
    #[error("Multiple trait impls may apply to this object type")]
    MultipleMatchingImpls { object_type: Type, candidates: Vec<String>, span: Span },
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
    NoMatchingImplFound(NoMatchingImplFoundError),
    #[error("Constraint for `{typ}: {trait_name}` is not needed, another matching impl is already in scope")]
    UnneededTraitConstraint { trait_name: String, typ: Type, span: Span },
    #[error(
        "Expected {expected_count} generic(s) from this function, but {actual_count} were provided"
    )]
    IncorrectTurbofishGenericCount { expected_count: usize, actual_count: usize, span: Span },
    #[error(
        "Cannot pass a mutable reference from a constrained runtime to an unconstrained runtime"
    )]
    ConstrainedReferenceToUnconstrained { span: Span },
    #[error(
        "Cannot pass a mutable reference from a unconstrained runtime to an constrained runtime"
    )]
    UnconstrainedReferenceToConstrained { span: Span },
    #[error("Slices cannot be returned from an unconstrained runtime to a constrained runtime")]
    UnconstrainedSliceReturnToConstrained { span: Span },
    #[error("Call to unconstrained function is unsafe and must be in an unconstrained function or unsafe block")]
    Unsafe { span: Span },
    #[error("Converting an unconstrained fn to a non-unconstrained fn is unsafe")]
    UnsafeFn { span: Span },
    #[error("Expected a constant, but found `{typ}`")]
    NonConstantEvaluated { typ: Type, span: Span },
    #[error("Slices must have constant length")]
    NonConstantSliceLength { span: Span },
    #[error("Only sized types may be used in the entry point to a program")]
    InvalidTypeForEntryPoint { span: Span },
    #[error("Mismatched number of parameters in trait implementation")]
    MismatchTraitImplNumParameters {
        actual_num_parameters: usize,
        expected_num_parameters: usize,
        trait_name: String,
        method_name: String,
        span: Span,
    },
    #[error("Strings do not support indexed assignment")]
    StringIndexAssign { span: Span },
    #[error("Macro calls may only return `Quoted` values")]
    MacroReturningNonExpr { typ: Type, span: Span },
    #[error("`{name}` has already been specified")]
    DuplicateNamedTypeArg { name: Ident, prev_span: Span },
    #[error("`{item}` has no associated type named `{name}`")]
    NoSuchNamedTypeArg { name: Ident, item: String },
    #[error("`{item}` is missing the associated type `{name}`")]
    MissingNamedTypeArg { name: Rc<String>, item: String, span: Span },
    #[error("Internal compiler error: type unspecified for value")]
    UnspecifiedType { span: Span },
    #[error("Binding `{typ}` here to the `_` inside would create a cyclic type")]
    CyclicType { typ: Type, span: Span },
    #[error("Type annotations required before indexing this array or slice")]
    TypeAnnotationsNeededForIndex { span: Span },
    #[error("Unnecessary `unsafe` block")]
    UnnecessaryUnsafeBlock { span: Span },
    #[error("Unnecessary `unsafe` block")]
    NestedUnsafeBlock { span: Span },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoMatchingImplFoundError {
    pub(crate) constraints: Vec<(Type, String)>,
    pub span: Span,
}

impl TypeCheckError {
    pub fn add_context(self, ctx: &'static str) -> Self {
        TypeCheckError::Context { err: Box::new(self), ctx }
    }

    pub(crate) fn is_non_constant_evaluated(&self) -> bool {
        matches!(self, TypeCheckError::NonConstantEvaluated { .. })
    }
}

impl<'a> From<&'a TypeCheckError> for Diagnostic {
    fn from(error: &'a TypeCheckError) -> Diagnostic {
        match error {
            TypeCheckError::TypeCannotBeUsed { typ, place, span } => Diagnostic::simple_error(
                format!("The type {} cannot be used in a {}", &typ, place),
                String::new(),
                *span,
            ),
            TypeCheckError::Context { err, ctx } => {
                let mut diag = Diagnostic::from(err.as_ref());
                diag.add_note(ctx.to_string());
                diag
            }
            TypeCheckError::OpCannotBeUsed { op, place, span } => Diagnostic::simple_error(
                format!("The operator {op:?} cannot be used in a {place}"),
                String::new(),
                *span,
            ),
            TypeCheckError::DivisionByZero { lhs, rhs, span } => Diagnostic::simple_error(
                format!("Division by zero: {lhs} / {rhs}"),
                String::new(),
                *span,
            ),
            TypeCheckError::ModuloOnFields { lhs, rhs, span } => Diagnostic::simple_error(
                format!("Modulo on Field elements: {lhs} % {rhs}"),
                String::new(),
                *span,
            ),
            TypeCheckError::TypeMismatch { expected_typ, expr_typ, expr_span } => {
                Diagnostic::simple_error(
                    format!("Expected type {expected_typ}, found type {expr_typ}"),
                    String::new(),
                    *expr_span,
                )
            }
            TypeCheckError::TypeKindMismatch { expected_kind, expr_kind, expr_span } => {
                Diagnostic::simple_error(
                    format!("Expected kind {expected_kind}, found kind {expr_kind}"),
                    String::new(),
                    *expr_span,
                )
            }
            TypeCheckError::TypeCanonicalizationMismatch { to, from, to_value, from_value, span } => {
                Diagnostic::simple_error(
                    format!("Evaluating {to} resulted in {to_value}, but {from_value} was expected"),
                    format!("from evaluating {from} without simplifications"),
                    *span,
                )
            }
            TypeCheckError::TraitMethodParameterTypeMismatch { method_name, expected_typ, actual_typ, parameter_index, parameter_span } => {
                Diagnostic::simple_error(
                    format!("Parameter #{parameter_index} of method `{method_name}` must be of type {expected_typ}, not {actual_typ}"),
                    String::new(),
                    *parameter_span,
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
                    *first_span,
                );
                diag.add_secondary(format!("but then found type {second_type}"), *second_span);
                diag
            }
            TypeCheckError::ArityMisMatch { expected, found, span } => {
                let plural = if *expected == 1 { "" } else { "s" };
                let msg = format!("Expected {expected} argument{plural}, but found {found}");
                Diagnostic::simple_error(msg, String::new(), *span)
            }
            TypeCheckError::ParameterCountMismatch { expected, found, span } => {
                let empty_or_s = if *expected == 1 { "" } else { "s" };
                let was_or_were = if *found == 1 { "was" } else { "were" };
                let msg = format!("Function expects {expected} parameter{empty_or_s} but {found} {was_or_were} given");
                Diagnostic::simple_error(msg, String::new(), *span)
            }
            TypeCheckError::AssertionParameterCountMismatch { kind, found, span } => {
                let was_or_were = if *found == 1 { "was" } else { "were" };
                let min = kind.required_arguments_count();
                let max = min + 1;
                let msg = format!("{kind} expects {min} or {max} parameters but {found} {was_or_were} given");
                Diagnostic::simple_error(msg, String::new(), *span)
            }
            TypeCheckError::GenericCountMismatch { item, expected, found, span } => {
                let empty_or_s = if *expected == 1 { "" } else { "s" };
                let was_or_were = if *found == 1 { "was" } else { "were" };
                let msg = format!("{item} expects {expected} generic{empty_or_s} but {found} {was_or_were} given");
                Diagnostic::simple_error(msg, String::new(), *span)
            }
            TypeCheckError::UnconstrainedMismatch { item, expected, span } => {
                let msg = if *expected {
                    format!("{item} is expected to be unconstrained")
                } else {
                    format!("{item} is not expected to be unconstrained")
                };
                Diagnostic::simple_error(msg, String::new(), *span)
            }
            TypeCheckError::InvalidCast { span, reason, .. } => {
                Diagnostic::simple_error(error.to_string(), reason.clone(), *span)
            }
            TypeCheckError::DownsizingCast { span, reason, .. } => {
                Diagnostic::simple_warning(error.to_string(), reason.clone(), *span)
            }

            TypeCheckError::ExpectedFunction { span, .. }
            | TypeCheckError::AccessUnknownMember { span, .. }
            | TypeCheckError::UnsupportedCast { span }
            | TypeCheckError::TupleIndexOutOfBounds { span, .. }
            | TypeCheckError::VariableMustBeMutable { span, .. }
            | TypeCheckError::CannotMutateImmutableVariable { span, .. }
            | TypeCheckError::UnresolvedMethodCall { span, .. }
            | TypeCheckError::IntegerSignedness { span, .. }
            | TypeCheckError::IntegerBitWidth { span, .. }
            | TypeCheckError::InvalidInfixOp { span, .. }
            | TypeCheckError::InvalidUnaryOp { span, .. }
            | TypeCheckError::FieldBitwiseOp { span, .. }
            | TypeCheckError::IntegerTypeMismatch { span, .. }
            | TypeCheckError::FieldComparison { span, .. }
            | TypeCheckError::AmbiguousBitWidth { span, .. }
            | TypeCheckError::IntegerAndFieldBinaryOperation { span }
            | TypeCheckError::OverflowingAssignment { span, .. }
            | TypeCheckError::OverflowingConstant { span, .. }
            | TypeCheckError::FailingBinaryOp { span, .. }
            | TypeCheckError::FieldModulo { span }
            | TypeCheckError::FieldNot { span }
            | TypeCheckError::ConstrainedReferenceToUnconstrained { span }
            | TypeCheckError::UnconstrainedReferenceToConstrained { span }
            | TypeCheckError::UnconstrainedSliceReturnToConstrained { span }
            | TypeCheckError::NonConstantEvaluated { span, .. }
            | TypeCheckError::NonConstantSliceLength { span }
            | TypeCheckError::StringIndexAssign { span }
            | TypeCheckError::InvalidShiftSize { span } => {
                Diagnostic::simple_error(error.to_string(), String::new(), *span)
            }
            TypeCheckError::PublicReturnType { typ, span } => Diagnostic::simple_error(
                "Functions cannot declare a public return type".to_string(),
                format!("return type is {typ}"),
                *span,
            ),
            TypeCheckError::TypeAnnotationsNeededForMethodCall { span } => {
                let mut error = Diagnostic::simple_error(
                    "Object type is unknown in method call".to_string(),
                    "Type must be known by this point to know which method to call".to_string(),
                    *span,
                );
                error.add_note("Try adding a type annotation for the object type before this method call".to_string());
                error
            },
            TypeCheckError::TypeAnnotationsNeededForFieldAccess { span } => {
                let mut error = Diagnostic::simple_error(
                    "Object type is unknown in field access".to_string(),
                    "Type must be known by this point".to_string(),
                    *span,
                );
                error.add_note("Try adding a type annotation for the object type before this expression".to_string());
                error
            },
            TypeCheckError::MultipleMatchingImpls { object_type, candidates, span } => {
                let message = format!("Multiple trait impls match the object type `{object_type}`");
                let secondary = "Ambiguous impl".to_string();
                let mut error = Diagnostic::simple_error(message, secondary, *span);
                for (i, candidate) in candidates.iter().enumerate() {
                    error.add_note(format!("Candidate {}: `{candidate}`", i + 1));
                }
                error
            },
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
                            FunctionReturnType::Ty(ty) => ty.span,
                        };

                        let mut diagnostic = Diagnostic::simple_error(format!("expected type {expected}, found type {actual}"), format!("expected {expected} because of return type"), ret_ty_span);

                        if let FunctionReturnType::Default(_) = ret_ty {
                            diagnostic.add_note(format!("help: try adding a return type: `-> {actual}`"));
                        }

                        diagnostic.add_secondary(format!("{actual} returned here"), *expr_span);

                        return diagnostic
                    },
                };

                Diagnostic::simple_error(message, String::new(), *span)
            }
            TypeCheckError::CallDeprecated { span, ref note, .. } => {
                let primary_message = error.to_string();
                let secondary_message = note.clone().unwrap_or_default();

                let mut diagnostic = Diagnostic::simple_warning(primary_message, secondary_message, *span);
                diagnostic.deprecated = true;
                diagnostic
            }
            TypeCheckError::UnusedResultError { expr_type, expr_span } => {
                let msg = format!("Unused expression result of type {expr_type}");
                Diagnostic::simple_warning(msg, String::new(), *expr_span)
            }
            TypeCheckError::NoMatchingImplFound(error) => error.into(),
            TypeCheckError::UnneededTraitConstraint { trait_name, typ, span } => {
                let msg = format!("Constraint for `{typ}: {trait_name}` is not needed, another matching impl is already in scope");
                Diagnostic::simple_warning(msg, "Unnecessary trait constraint in where clause".into(), *span)
            }
            TypeCheckError::InvalidTypeForEntryPoint { span } => Diagnostic::simple_error(
                "Only sized types may be used in the entry point to a program".to_string(),
                "Slices, references, or any type containing them may not be used in main, contract functions, or foldable functions".to_string(), *span),
            TypeCheckError::MismatchTraitImplNumParameters {
                expected_num_parameters,
                actual_num_parameters,
                trait_name,
                method_name,
                span,
            } => {
                let plural = if *expected_num_parameters == 1 { "" } else { "s" };
                let primary_message = format!(
                    "`{trait_name}::{method_name}` expects {expected_num_parameters} parameter{plural}, but this method has {actual_num_parameters}");
                Diagnostic::simple_error(primary_message, "".to_string(), *span)
            }
            TypeCheckError::IncorrectTurbofishGenericCount { expected_count, actual_count, span } => {
                let expected_plural = if *expected_count == 1 { "" } else { "s" };
                let actual_plural = if *actual_count == 1 { "was" } else { "were" };
                let msg = format!("Expected {expected_count} generic{expected_plural} from this function, but {actual_count} {actual_plural} provided");
                Diagnostic::simple_error(msg, "".into(), *span)
            },
            TypeCheckError::MacroReturningNonExpr { typ, span } =>  {
                let mut error = Diagnostic::simple_error(
                    format!("Expected macro call to return a `Quoted` but found a(n) `{typ}`"),
                    "Macro calls must return quoted values, otherwise there is no code to insert.".into(),
                    *span,
                );
                error.add_secondary("Hint: remove the `!` from the end of the function name.".to_string(), *span);
                error
            },
            TypeCheckError::DuplicateNamedTypeArg { name, prev_span } => {
                let msg = format!("`{name}` has already been specified");
                let mut error = Diagnostic::simple_error(msg.to_string(), "".to_string(), name.span());
                error.add_secondary(format!("`{name}` previously specified here"), *prev_span);
                error
            },
            TypeCheckError::NoSuchNamedTypeArg { name, item } => {
                let msg = format!("`{item}` has no associated type named `{name}`");
                Diagnostic::simple_error(msg.to_string(), "".to_string(), name.span())
            },
            TypeCheckError::MissingNamedTypeArg { name, item, span } => {
                let msg = format!("`{item}` is missing the associated type `{name}`");
                Diagnostic::simple_error(msg.to_string(), "".to_string(), *span)
            },
            TypeCheckError::Unsafe { span } => {
                Diagnostic::simple_error(error.to_string(), String::new(), *span)
            }
            TypeCheckError::UnsafeFn { span } => {
                Diagnostic::simple_error(error.to_string(), String::new(), *span)
            }
            TypeCheckError::UnspecifiedType { span } => {
                Diagnostic::simple_error(error.to_string(), String::new(), *span)
            }
            TypeCheckError::CyclicType { typ: _, span } => {
                Diagnostic::simple_error(error.to_string(), "Cyclic types have unlimited size and are prohibited in Noir".into(), *span)
            }
            TypeCheckError::CannotInvokeStructFieldFunctionType { method_name, object_type, span } => {
                Diagnostic::simple_error(
                    format!("Cannot invoke function field '{method_name}' on type '{object_type}' as a method"), 
                    format!("to call the function stored in '{method_name}', surround the field access with parentheses: '(', ')'"),
                    *span,
                )
            },
            TypeCheckError::TypeAnnotationsNeededForIndex { span } => {
                Diagnostic::simple_error(
                    "Type annotations required before indexing this array or slice".into(), 
                    "Type annotations needed before this point, can't decide if this is an array or slice".into(),
                    *span,
                )
            },
            TypeCheckError::UnnecessaryUnsafeBlock { span } => {
                Diagnostic::simple_warning(
                    "Unnecessary `unsafe` block".into(), 
                    "".into(),
                    *span,
                )
            },
            TypeCheckError::NestedUnsafeBlock { span } => {
                Diagnostic::simple_warning(
                    "Unnecessary `unsafe` block".into(), 
                    "Because it's nested inside another `unsafe` block".into(),
                    *span,
                )
            },
        }
    }
}

impl<'a> From<&'a NoMatchingImplFoundError> for Diagnostic {
    fn from(error: &'a NoMatchingImplFoundError) -> Self {
        let constraints = &error.constraints;
        let span = error.span;

        assert!(!constraints.is_empty());
        let msg =
            format!("No matching impl found for `{}: {}`", constraints[0].0, constraints[0].1);
        let mut diagnostic = Diagnostic::from_message(&msg);

        let secondary = format!("No impl for `{}: {}`", constraints[0].0, constraints[0].1);
        diagnostic.add_secondary(secondary, span);

        // These must be notes since secondaries are unordered
        for (typ, trait_name) in &constraints[1..] {
            diagnostic.add_note(format!("Required by `{typ}: {trait_name}`"));
        }

        diagnostic
    }
}

impl NoMatchingImplFoundError {
    pub fn new(
        interner: &NodeInterner,
        failing_constraints: Vec<TraitConstraint>,
        span: Span,
    ) -> Option<Self> {
        // Don't show any errors where try_get_trait returns None.
        // This can happen if a trait is used that was never declared.
        let constraints = failing_constraints
            .into_iter()
            .map(|constraint| {
                let r#trait = interner.try_get_trait(constraint.trait_bound.trait_id)?;
                let name = format!("{}{}", r#trait.name, constraint.trait_bound.trait_generics);
                Some((constraint.typ, name))
            })
            .collect::<Option<Vec<_>>>()?;

        Some(Self { constraints, span })
    }
}
