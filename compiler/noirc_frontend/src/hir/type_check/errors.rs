use std::collections::BTreeSet;
use std::rc::Rc;

use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::Location;
use thiserror::Error;

use crate::ast::BinaryOpKind;
use crate::ast::{ConstrainKind, FunctionReturnType, Ident, IntegerBitSize};
use crate::hir::resolution::errors::ResolverError;
use crate::hir_def::traits::TraitConstraint;
use crate::hir_def::types::{BinaryTypeOperator, Kind, Type};
use crate::node_interner::NodeInterner;
use crate::shared::Signedness;
use crate::signed_field::SignedField;
use crate::validity::InvalidType;

/// Rust also only shows 3 maximum, even for short patterns.
pub const MAX_MISSING_CASES: usize = 3;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Source {
    #[error("Binary")]
    Binary,
    #[error("Assignment")]
    Assignment,
    #[error("Return")]
    Return(FunctionReturnType, Location),
    #[error("ArrayIndex")]
    ArrayIndex,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TypeCheckError {
    #[error("Division by zero: {lhs} / {rhs}")]
    DivisionByZero { lhs: SignedField, rhs: SignedField, location: Location },
    #[error("Modulo on Field elements: {lhs} % {rhs}")]
    ModuloOnFields { lhs: SignedField, rhs: SignedField, location: Location },
    #[error("The value `{expr}` cannot fit into `{ty}` which has range `{range}`")]
    IntegerLiteralDoesNotFitItsType {
        expr: SignedField,
        ty: Type,
        range: String,
        location: Location,
    },
    #[error(
        "The value `{value}` cannot fit into `{kind}` which has a maximum size of `{maximum_size}`"
    )]
    OverflowingConstant {
        value: SignedField,
        kind: Kind,
        maximum_size: FieldElement,
        location: Location,
    },
    #[error(
        "The value `{value}` cannot fit into `{kind}` which has a minimum size of `{minimum_size}`"
    )]
    UnderflowingConstant {
        value: SignedField,
        kind: Kind,
        minimum_size: SignedField,
        location: Location,
    },
    #[error("Evaluating `{op}` on `{lhs}`, `{rhs}` failed")]
    FailingBinaryOp { op: BinaryTypeOperator, lhs: String, rhs: String, location: Location },
    #[error("Type {typ:?} cannot be used in a {place:?}")]
    TypeCannotBeUsed { typ: Type, place: &'static str, location: Location },
    #[error("Expected type {expected_typ:?} is not the same as {expr_typ:?}")]
    TypeMismatch { expected_typ: String, expr_typ: String, expr_location: Location },
    #[error("Expected type {expected} is not the same as {actual}")]
    TypeMismatchWithSource { expected: Type, actual: Type, location: Location, source: Source },
    #[error("Expected type {expected_kind:?} is not the same as {expr_kind:?}")]
    TypeKindMismatch { expected_kind: Kind, expr_kind: Kind, expr_location: Location },
    #[error("Evaluating {to} resulted in {to_value}, but {from_value} was expected")]
    TypeCanonicalizationMismatch {
        to: Type,
        from: Type,
        to_value: SignedField,
        from_value: SignedField,
        location: Location,
    },
    #[error("Expected {expected:?} found {found:?}")]
    ArityMisMatch { expected: usize, found: usize, location: Location },
    #[error("Cannot cast type {from}, 'as' is only for primitive field or integer types")]
    InvalidCast { from: Type, location: Location, reason: String },
    #[error("Casting value of type {from} to a smaller type ({to})")]
    DownsizingCast { from: Type, to: Type, location: Location, reason: String },
    #[error("Cannot cast `{typ}` as `bool`")]
    CannotCastNumericToBool { typ: Type, location: Location },
    #[error("Expected a function, but found a(n) {found}")]
    ExpectedFunction { found: Type, location: Location },
    #[error("Type {lhs_type} has no member named {field_name}")]
    AccessUnknownMember { lhs_type: Type, field_name: String, location: Location },
    #[error("Function expects {expected} parameters but {found} were given")]
    ParameterCountMismatch { expected: usize, found: usize, location: Location },
    #[error("{} expects {} or {} parameters but {found} were given", kind, kind.required_arguments_count(), kind.required_arguments_count() + 1)]
    AssertionParameterCountMismatch { kind: ConstrainKind, found: usize, location: Location },
    #[error("{item} expects {expected} generics but {found} were given")]
    GenericCountMismatch { item: String, expected: usize, found: usize, location: Location },
    #[error("{item} has incompatible `unconstrained`")]
    UnconstrainedMismatch { item: String, expected: bool, location: Location },
    #[error("Only integer and Field types may be casted to")]
    UnsupportedCast { location: Location },
    #[error("Only unsigned integer types may be casted to Field")]
    UnsupportedFieldCast { location: Location },
    #[error("Index {index} is out of bounds for this tuple {lhs_type} of length {length}")]
    TupleIndexOutOfBounds { index: usize, lhs_type: Type, length: usize, location: Location },
    #[error("Variable `{name}` must be mutable to be assigned to")]
    VariableMustBeMutable { name: String, location: Location },
    #[error("Cannot mutate immutable variable `{name}`")]
    CannotMutateImmutableVariable { name: String, location: Location },
    #[error("Variable {name} captured in lambda must be a mutable reference")]
    MutableCaptureWithoutRef { name: String, location: Location },
    #[error("Mutable references to array indices are unsupported")]
    MutableReferenceToArrayElement { location: Location },
    #[error("No method named '{method_name}' found for type '{object_type}'")]
    UnresolvedMethodCall { method_name: String, object_type: Type, location: Location },
    #[error("Cannot invoke function field '{method_name}' on type '{object_type}' as a method")]
    CannotInvokeStructFieldFunctionType {
        method_name: String,
        object_type: Type,
        location: Location,
    },
    #[error("Integers must have the same signedness LHS is {sign_x:?}, RHS is {sign_y:?}")]
    IntegerSignedness { sign_x: Signedness, sign_y: Signedness, location: Location },
    #[error("Integers must have the same bit width LHS is {bit_width_x}, RHS is {bit_width_y}")]
    IntegerBitWidth { bit_width_x: IntegerBitSize, bit_width_y: IntegerBitSize, location: Location },
    #[error("Cannot apply unary operator `{operator}` to type `{typ}`")]
    InvalidUnaryOp { operator: &'static str, typ: String, location: Location },
    #[error(
        "Bitwise operations are invalid on Field types. Try casting the operands to a sized integer type first."
    )]
    FieldBitwiseOp { location: Location },
    #[error("Cannot do modulo on Fields, try casting to an integer first")]
    FieldModulo { location: Location },
    #[error("Cannot do not (`!`) on Fields, try casting to an integer first")]
    FieldNot { location: Location },
    #[error("Fields cannot be compared, try casting to an integer first")]
    FieldComparison { location: Location },
    #[error(
        "The bit count in a bit-shift operation must fit in a u8, try casting the right hand side into a u8 first"
    )]
    InvalidShiftSize { location: Location },
    #[error("Cannot `bool {op} bool`")]
    InvalidBoolInfixOp { op: BinaryOpKind, location: Location },
    #[error("Error with additional context")]
    Context { err: Box<TypeCheckError>, ctx: &'static str },
    #[error("Array is not homogeneous")]
    NonHomogeneousArray {
        first_location: Location,
        first_type: String,
        first_index: usize,
        second_location: Location,
        second_type: String,
        second_index: usize,
    },
    #[error("Object type is unknown in method call")]
    TypeAnnotationsNeededForMethodCall { location: Location },
    #[error("Object type is unknown in field access")]
    TypeAnnotationsNeededForFieldAccess { location: Location },
    #[error("Multiple trait impls may apply to this object type")]
    MultipleMatchingImpls { object_type: Type, candidates: Vec<String>, location: Location },
    #[error("use of deprecated function {name}")]
    CallDeprecated { name: String, note: Option<String>, location: Location },
    #[error("{0}")]
    ResolverError(ResolverError),
    #[error("Unused expression result of type {expr_type}")]
    UnusedResultWarning { expr_type: Type, expr_location: Location },
    #[error("Unused expression result of type {expr_type}")]
    UnusedResultError { expr_type: Type, expr_location: Location, message: Option<String> },
    #[error("Expected type {expected_typ:?} is not the same as {actual_typ:?}")]
    TraitMethodParameterTypeMismatch {
        method_name: String,
        expected_typ: String,
        actual_typ: String,
        parameter_location: Location,
        parameter_index: usize,
    },
    #[error("No matching impl found")]
    NoMatchingImplFound(NoMatchingImplFoundError),
    #[error(
        "Constraint for `{typ}: {trait_name}` is not needed, another matching impl is already in scope"
    )]
    UnneededTraitConstraint { trait_name: String, typ: Type, location: Location },
    #[error(
        "Expected {expected_count} generic(s) from this function, but {actual_count} were provided"
    )]
    IncorrectTurbofishGenericCount {
        expected_count: usize,
        actual_count: usize,
        location: Location,
    },
    #[error(
        "Cannot pass a mutable reference from a constrained runtime to an unconstrained runtime"
    )]
    ConstrainedReferenceToUnconstrained { location: Location },
    #[error(
        "Cannot pass a mutable reference from a unconstrained runtime to an constrained runtime"
    )]
    UnconstrainedReferenceToConstrained { location: Location },
    #[error("Vectors cannot be returned from an unconstrained runtime to a constrained runtime")]
    UnconstrainedVectorReturnToConstrained { location: Location },
    #[error("Functions cannot be returned from an unconstrained runtime to a constrained runtime")]
    UnconstrainedFunctionReturnToConstrained { location: Location },
    #[error(
        "Call to unconstrained function is unsafe and must be in an unconstrained function or unsafe block"
    )]
    Unsafe { location: Location },
    #[error("Converting an unconstrained fn to a non-unconstrained fn is unsafe")]
    UnsafeFn { location: Location },
    #[error("Expected a constant, but found `{typ}`")]
    NonConstantEvaluated { typ: Type, location: Location },
    #[error("Only sized types may be used in the entry point to a program")]
    InvalidTypeForEntryPoint { invalid_type: InvalidType, location: Location },
    #[error("Mismatched number of parameters in trait implementation")]
    MismatchTraitImplNumParameters {
        actual_num_parameters: usize,
        expected_num_parameters: usize,
        trait_name: String,
        method_name: String,
        location: Location,
    },
    #[error("Strings do not support indexed assignment")]
    StringIndexAssign { location: Location },
    #[error("Macro calls may only return `Quoted` values")]
    MacroReturningNonExpr { typ: Type, location: Location },
    #[error("`{name}` has already been specified")]
    DuplicateNamedTypeArg { name: Ident, prev_location: Location },
    #[error("`{item}` has no associated type named `{name}`")]
    NoSuchNamedTypeArg { name: Ident, item: String },
    #[error("`{item}` is missing the associated type `{name}`")]
    MissingNamedTypeArg { name: Rc<String>, item: String, location: Location },
    #[error("Internal compiler error: type unspecified for value")]
    UnspecifiedType { location: Location },
    #[error("Binding `{typ}` here to the `_` inside would create a cyclic type")]
    CyclicType { typ: Type, location: Location },
    #[error("Type annotations required before indexing this array or vector")]
    TypeAnnotationsNeededForIndex { location: Location },
    #[error("Unnecessary `unsafe` block")]
    UnnecessaryUnsafeBlock { location: Location },
    #[error("Unnecessary `unsafe` block")]
    NestedUnsafeBlock { location: Location },
    #[error("Unreachable match case")]
    UnreachableCase { location: Location },
    #[error("Missing cases")]
    MissingCases { cases: BTreeSet<String>, location: Location },
    /// This error is used for types like integers which have too many variants to enumerate
    #[error("Missing cases: `{typ}` is non-empty")]
    MissingManyCases { typ: String, location: Location },
    #[error("Expected a tuple with {} elements, found one with {} elements", tuple_types.len(), actual_count)]
    TupleMismatch { tuple_types: Vec<Type>, actual_count: usize, location: Location },
    #[error("Type annotation needed on item")]
    TypeAnnotationNeededOnItem {
        location: Location,
        generic_name: String,
        item_kind: &'static str,
        item_name: String,
        is_numeric: bool,
    },
    #[error("Type annotation needed on array literal")]
    TypeAnnotationNeededOnArrayLiteral { is_array: bool, location: Location },
    #[error("Expecting another error: {message}")]
    ExpectingOtherError { message: String, location: Location },
    #[error("Cannot call `std::verify_proof_with_type` in unconstrained context")]
    VerifyProofWithTypeInBrillig { location: Location },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoMatchingImplFoundError {
    pub(crate) constraints: Vec<(Type, String)>,
    pub location: Location,
}

impl TypeCheckError {
    pub fn add_context(self, ctx: &'static str) -> Self {
        TypeCheckError::Context { err: Box::new(self), ctx }
    }

    pub(crate) fn is_non_constant_evaluated(&self) -> bool {
        matches!(self, TypeCheckError::NonConstantEvaluated { .. })
    }

    pub fn location(&self) -> Location {
        match self {
            TypeCheckError::DivisionByZero { location, .. }
            | TypeCheckError::ModuloOnFields { location, .. }
            | TypeCheckError::IntegerLiteralDoesNotFitItsType { location, .. }
            | TypeCheckError::OverflowingConstant { location, .. }
            | TypeCheckError::UnderflowingConstant { location, .. }
            | TypeCheckError::FailingBinaryOp { location, .. }
            | TypeCheckError::TypeCannotBeUsed { location, .. }
            | TypeCheckError::TypeMismatch { expr_location: location, .. }
            | TypeCheckError::TypeMismatchWithSource { location, .. }
            | TypeCheckError::TypeKindMismatch { expr_location: location, .. }
            | TypeCheckError::TypeCanonicalizationMismatch { location, .. }
            | TypeCheckError::ArityMisMatch { location, .. }
            | TypeCheckError::InvalidCast { location, .. }
            | TypeCheckError::DownsizingCast { location, .. }
            | TypeCheckError::CannotCastNumericToBool { location, .. }
            | TypeCheckError::ExpectedFunction { location, .. }
            | TypeCheckError::AccessUnknownMember { location, .. }
            | TypeCheckError::ParameterCountMismatch { location, .. }
            | TypeCheckError::AssertionParameterCountMismatch { location, .. }
            | TypeCheckError::GenericCountMismatch { location, .. }
            | TypeCheckError::UnconstrainedMismatch { location, .. }
            | TypeCheckError::UnsupportedCast { location }
            | TypeCheckError::UnsupportedFieldCast { location }
            | TypeCheckError::TupleIndexOutOfBounds { location, .. }
            | TypeCheckError::VariableMustBeMutable { location, .. }
            | TypeCheckError::CannotMutateImmutableVariable { location, .. }
            | TypeCheckError::MutableCaptureWithoutRef { location, .. }
            | TypeCheckError::MutableReferenceToArrayElement { location }
            | TypeCheckError::UnresolvedMethodCall { location, .. }
            | TypeCheckError::CannotInvokeStructFieldFunctionType { location, .. }
            | TypeCheckError::IntegerSignedness { location, .. }
            | TypeCheckError::IntegerBitWidth { location, .. }
            | TypeCheckError::InvalidUnaryOp { location, .. }
            | TypeCheckError::FieldBitwiseOp { location }
            | TypeCheckError::FieldModulo { location }
            | TypeCheckError::FieldNot { location }
            | TypeCheckError::FieldComparison { location }
            | TypeCheckError::InvalidShiftSize { location }
            | TypeCheckError::InvalidBoolInfixOp { location, .. }
            | TypeCheckError::NonHomogeneousArray { first_location: location, .. }
            | TypeCheckError::TypeAnnotationsNeededForMethodCall { location }
            | TypeCheckError::TypeAnnotationsNeededForFieldAccess { location }
            | TypeCheckError::MultipleMatchingImpls { location, .. }
            | TypeCheckError::CallDeprecated { location, .. }
            | TypeCheckError::UnusedResultWarning { expr_location: location, .. }
            | TypeCheckError::UnusedResultError { expr_location: location, .. }
            | TypeCheckError::TraitMethodParameterTypeMismatch {
                parameter_location: location,
                ..
            }
            | TypeCheckError::UnneededTraitConstraint { location, .. }
            | TypeCheckError::IncorrectTurbofishGenericCount { location, .. }
            | TypeCheckError::ConstrainedReferenceToUnconstrained { location }
            | TypeCheckError::UnconstrainedReferenceToConstrained { location }
            | TypeCheckError::UnconstrainedVectorReturnToConstrained { location }
            | TypeCheckError::UnconstrainedFunctionReturnToConstrained { location }
            | TypeCheckError::Unsafe { location }
            | TypeCheckError::UnsafeFn { location }
            | TypeCheckError::NonConstantEvaluated { location, .. }
            | TypeCheckError::InvalidTypeForEntryPoint { location, .. }
            | TypeCheckError::MismatchTraitImplNumParameters { location, .. }
            | TypeCheckError::StringIndexAssign { location }
            | TypeCheckError::MacroReturningNonExpr { location, .. }
            | TypeCheckError::MissingNamedTypeArg { location, .. }
            | TypeCheckError::UnspecifiedType { location }
            | TypeCheckError::CyclicType { location, .. }
            | TypeCheckError::TypeAnnotationsNeededForIndex { location }
            | TypeCheckError::UnnecessaryUnsafeBlock { location }
            | TypeCheckError::UnreachableCase { location }
            | TypeCheckError::MissingCases { location, .. }
            | TypeCheckError::MissingManyCases { location, .. }
            | TypeCheckError::NestedUnsafeBlock { location }
            | TypeCheckError::TupleMismatch { location, .. }
            | TypeCheckError::TypeAnnotationNeededOnItem { location, .. }
            | TypeCheckError::TypeAnnotationNeededOnArrayLiteral { location, .. }
            | TypeCheckError::ExpectingOtherError { location, .. }
            | TypeCheckError::VerifyProofWithTypeInBrillig { location } => *location,
            TypeCheckError::DuplicateNamedTypeArg { name: ident, .. }
            | TypeCheckError::NoSuchNamedTypeArg { name: ident, .. } => ident.location(),

            TypeCheckError::NoMatchingImplFound(no_matching_impl_found_error) => {
                no_matching_impl_found_error.location
            }
            TypeCheckError::Context { err, .. } => err.location(),
            TypeCheckError::ResolverError(resolver_error) => resolver_error.location(),
        }
    }
}

impl<'a> From<&'a TypeCheckError> for Diagnostic {
    fn from(error: &'a TypeCheckError) -> Diagnostic {
        match error {
            TypeCheckError::TypeCannotBeUsed { typ, place, location } => Diagnostic::simple_error(
                format!("The type {} cannot be used in a {}", &typ, place),
                String::new(),
                *location,
            ),
            TypeCheckError::Context { err, ctx } => {
                let mut diag = Diagnostic::from(err.as_ref());
                diag.add_note(ctx.to_string());
                diag
            }
            TypeCheckError::DivisionByZero { lhs, rhs, location } => Diagnostic::simple_error(
                format!("Division by zero: {lhs} / {rhs}"),
                String::new(),
                *location,
            ),
            TypeCheckError::ModuloOnFields { lhs, rhs, location } => Diagnostic::simple_error(
                format!("Modulo on Field elements: {lhs} % {rhs}"),
                String::new(),
                *location,
            ),
            TypeCheckError::TypeMismatch { expected_typ, expr_typ, expr_location } => {
                Diagnostic::simple_error(
                    format!("Expected type {expected_typ}, found type {expr_typ}"),
                    String::new(),
                    *expr_location,
                )
            }
            TypeCheckError::TypeKindMismatch { expected_kind, expr_kind, expr_location } => {
                // Try to improve the error message for some kind combinations
                match (expected_kind, expr_kind) {
                    (Kind::Normal, Kind::Numeric(_)) => {
                        Diagnostic::simple_error(
                            "Expected type, found numeric generic".into(),
                            "not a type".into(),
                            *expr_location,
                        )
                    }
                    (Kind::Numeric(typ), Kind::Normal) => {
                        Diagnostic::simple_error(
                            "Type provided when a numeric generic was expected".into(),
                            format!("the numeric generic is not of type `{typ}`"),
                            *expr_location,
                        )
                    }
                    (Kind::Numeric(expected_type), Kind::Numeric(found_type)) => {
                        Diagnostic::simple_error(
                            format!("The numeric generic is not of type `{expected_type}`"),
                            format!("expected `{expected_type}`, found `{found_type}`"),
                            *expr_location,
                        )
                    }
                    _ => {
                        Diagnostic::simple_error(
                            format!("Expected kind {expected_kind}, found kind {expr_kind}"),
                            String::new(),
                            *expr_location,
                        )
                    }
                }
            }
            TypeCheckError::TypeCanonicalizationMismatch { to, from, to_value, from_value, location } => {
                Diagnostic::simple_error(
                    format!("Evaluating {to} resulted in {to_value}, but {from_value} was expected"),
                    format!("from evaluating {from} without simplifications"),
                    *location,
                )
            }
            TypeCheckError::TraitMethodParameterTypeMismatch { method_name, expected_typ, actual_typ, parameter_index, parameter_location } => {
                Diagnostic::simple_error(
                    format!("Parameter #{parameter_index} of method `{method_name}` must be of type {expected_typ}, not {actual_typ}"),
                    String::new(),
                    *parameter_location,
                )
            }
            TypeCheckError::NonHomogeneousArray {
                first_location,
                first_type,
                first_index,
                second_location,
                second_type,
                second_index,
            } => {
                let mut diag = Diagnostic::simple_error(
                    format!(
                        "Non homogeneous array, different element types found at indices ({first_index},{second_index})"
                    ),
                    format!("Found type {first_type}"),
                    *first_location,
                );
                diag.add_secondary(format!("but then found type {second_type}"), *second_location);
                diag
            }
            TypeCheckError::ArityMisMatch { expected, found, location } => {
                let plural = if *expected == 1 { "" } else { "s" };
                let msg = format!("Expected {expected} argument{plural}, but found {found}");
                Diagnostic::simple_error(msg, String::new(), *location)
            }
            TypeCheckError::ParameterCountMismatch { expected, found, location } => {
                let empty_or_s = if *expected == 1 { "" } else { "s" };
                let was_or_were = if *found == 1 { "was" } else { "were" };
                let msg = format!("Function expects {expected} parameter{empty_or_s} but {found} {was_or_were} given");
                Diagnostic::simple_error(msg, String::new(), *location)
            }
            TypeCheckError::AssertionParameterCountMismatch { kind, found, location } => {
                let was_or_were = if *found == 1 { "was" } else { "were" };
                let min = kind.required_arguments_count();
                let max = min + 1;
                let msg = format!("{kind} expects {min} or {max} parameters but {found} {was_or_were} given");
                Diagnostic::simple_error(msg, String::new(), *location)
            }
            TypeCheckError::GenericCountMismatch { item, expected, found, location } => {
                let empty_or_s = if *expected == 1 { "" } else { "s" };
                let was_or_were = if *found == 1 { "was" } else { "were" };
                let msg = format!("{item} expects {expected} generic{empty_or_s} but {found} {was_or_were} given");
                Diagnostic::simple_error(msg, String::new(), *location)
            }
            TypeCheckError::UnconstrainedMismatch { item, expected, location } => {
                let msg = if *expected {
                    format!("{item} is expected to be unconstrained")
                } else {
                    format!("{item} is not expected to be unconstrained")
                };
                Diagnostic::simple_error(msg, String::new(), *location)
            }
            TypeCheckError::InvalidCast { location, reason, .. } => {
                Diagnostic::simple_error(error.to_string(), reason.clone(), *location)
            }
            TypeCheckError::DownsizingCast { location, reason, .. } => {
                Diagnostic::simple_warning(error.to_string(), reason.clone(), *location)
            }
            TypeCheckError::CannotCastNumericToBool { typ: _, location } => {
                let secondary = "compare with zero instead: ` != 0`".to_string();
                Diagnostic::simple_error(error.to_string(), secondary, *location)
            }

            TypeCheckError::ExpectedFunction { location, .. }
            | TypeCheckError::AccessUnknownMember { location, .. }
            | TypeCheckError::UnsupportedCast { location }
            | TypeCheckError::UnsupportedFieldCast { location }
            | TypeCheckError::TupleIndexOutOfBounds { location, .. }
            | TypeCheckError::VariableMustBeMutable { location, .. }
            | TypeCheckError::CannotMutateImmutableVariable { location, .. }
            | TypeCheckError::UnresolvedMethodCall { location, .. }
            | TypeCheckError::IntegerSignedness { location, .. }
            | TypeCheckError::IntegerBitWidth { location, .. }
            | TypeCheckError::InvalidUnaryOp { location, .. }
            | TypeCheckError::FieldBitwiseOp { location, .. }
            | TypeCheckError::FieldComparison { location, .. }
            | TypeCheckError::IntegerLiteralDoesNotFitItsType { location, .. }
            | TypeCheckError::OverflowingConstant { location, .. }
            | TypeCheckError::UnderflowingConstant { location, .. }
            | TypeCheckError::FailingBinaryOp { location, .. }
            | TypeCheckError::FieldModulo { location }
            | TypeCheckError::FieldNot { location }
            | TypeCheckError::ConstrainedReferenceToUnconstrained { location }
            | TypeCheckError::UnconstrainedReferenceToConstrained { location }
            | TypeCheckError::UnconstrainedVectorReturnToConstrained { location }
            | TypeCheckError::UnconstrainedFunctionReturnToConstrained { location }
            | TypeCheckError::NonConstantEvaluated { location, .. }
            | TypeCheckError::StringIndexAssign { location }
            | TypeCheckError::InvalidShiftSize { location }
            | TypeCheckError::VerifyProofWithTypeInBrillig { location } => {
                Diagnostic::simple_error(error.to_string(), String::new(), *location)
            }
            TypeCheckError::InvalidBoolInfixOp { op, location } => {
                let primary = match op {
                    BinaryOpKind::Add => "Cannot add a `bool` to a `bool",
                    BinaryOpKind::Subtract => "Cannot subtract a `bool` from a `bool",
                    BinaryOpKind::Multiply => "Cannot multiply a `bool` by a `bool",
                    BinaryOpKind::Divide => "Cannot divide a `bool` by a `bool`",
                    BinaryOpKind::ShiftRight => "No implementation for `bool >> bool`",
                    BinaryOpKind::ShiftLeft => "No implementation for `bool << bool`",
                    BinaryOpKind::Modulo => "Cannot calculate the remainder of a `bool` divided by a `bool`",
                    BinaryOpKind::Equal |
                    BinaryOpKind::NotEqual |
                    BinaryOpKind::Less |
                    BinaryOpKind::LessEqual |
                    BinaryOpKind::Greater |
                    BinaryOpKind::GreaterEqual |
                    BinaryOpKind::And |
                    BinaryOpKind::Or |
                    BinaryOpKind::Xor => panic!("Unexpected op in InvalidBoolInfixOp error: {op}"),
                };
                Diagnostic::simple_error(primary.to_string(), String::new(), *location)
            }
            TypeCheckError::MutableCaptureWithoutRef { name, location } => Diagnostic::simple_error(
                format!("Mutable variable {name} captured in lambda must be a mutable reference"),
                "Use '&mut' instead of 'mut' to capture a mutable variable.".to_string(),
                *location,
            ),
            TypeCheckError::MutableReferenceToArrayElement { location } => {
                Diagnostic::simple_error("Mutable references to array elements are currently unsupported".into(), "Try storing the element in a fresh variable first".into(), *location)
            },
            TypeCheckError::TypeAnnotationsNeededForMethodCall { location } => {
                let mut error = Diagnostic::simple_error(
                    "Object type is unknown in method call".to_string(),
                    "Type must be known by this point to know which method to call".to_string(),
                    *location,
                );
                error.add_note("Try adding a type annotation for the object type before this method call".to_string());
                error
            },
            TypeCheckError::TypeAnnotationsNeededForFieldAccess { location } => {
                let mut error = Diagnostic::simple_error(
                    "Object type is unknown in field access".to_string(),
                    "Type must be known by this point".to_string(),
                    *location,
                );
                error.add_note("Try adding a type annotation for the object type before this expression".to_string());
                error
            },
            TypeCheckError::MultipleMatchingImpls { object_type, candidates, location } => {
                let message = format!("Multiple trait impls match the object type `{object_type}`");
                let secondary = "Ambiguous impl".to_string();
                let mut error = Diagnostic::simple_error(message, secondary, *location);
                for (i, candidate) in candidates.iter().enumerate() {
                    error.add_note(format!("Candidate {}: `{candidate}`", i + 1));
                }
                error
            },
            TypeCheckError::ResolverError(error) => error.into(),
            TypeCheckError::TypeMismatchWithSource { expected, actual, location, source } => {
                let message = match source {
                    Source::Binary => format!("Types in a binary operation should match, but found {expected} and {actual}"),
                    Source::Assignment => {
                        format!("Cannot assign an expression of type {actual} to a value of type {expected}")
                    }
                    Source::Return(ret_ty, expr_location) => {
                        let ret_ty_location = match ret_ty.clone() {
                            FunctionReturnType::Default(location) => location,
                            FunctionReturnType::Ty(ty) => ty.location,
                        };

                        let mut diagnostic = Diagnostic::simple_error(format!("expected type {expected}, found type {actual}"), format!("expected {expected} because of return type"), ret_ty_location);

                        if let FunctionReturnType::Default(_) = ret_ty {
                            diagnostic.add_note(format!("help: try adding a return type: `-> {actual}`"));
                        }

                        diagnostic.add_secondary(format!("{actual} returned here"), *expr_location);

                        return diagnostic
                    },
                    Source::ArrayIndex => format!("Indexing arrays and vectors must be done with `{expected}`, not `{actual}`"),
                };

                Diagnostic::simple_error(message, String::new(), *location)
            }
            TypeCheckError::CallDeprecated { location,  note, .. } => {
                let primary_message = error.to_string();
                let secondary_message = note.clone().unwrap_or_default();

                let mut diagnostic = Diagnostic::simple_warning(primary_message, secondary_message, *location);
                diagnostic.deprecated = true;
                diagnostic
            }
            TypeCheckError::UnusedResultWarning { expr_type, expr_location } => {
                let msg = format!("Unused expression result of type {expr_type}");
                Diagnostic::simple_warning(msg, String::new(), *expr_location)
            }
            TypeCheckError::UnusedResultError { expr_type, expr_location, message } => {
                let unused_message = format!("Unused expression result of type {expr_type} which must be used");
                let (primary, secondary) = match message {
                    Some(message) => (message.clone(), unused_message),
                    None => (unused_message, format!("`{expr_type}` was declared with `#[must_use]`")),
                };
                Diagnostic::simple_error(primary, secondary, *expr_location)
            }
            TypeCheckError::NoMatchingImplFound(error) => error.into(),
            TypeCheckError::UnneededTraitConstraint { trait_name, typ, location } => {
                let msg = format!("Constraint for `{typ}: {trait_name}` is not needed, another matching impl is already in scope");
                Diagnostic::simple_warning(msg, "Unnecessary trait constraint in where clause".into(), *location)
            }
            TypeCheckError::InvalidTypeForEntryPoint { invalid_type, location } => {
                let primary_message = "Invalid type found in the entry point to a program".to_string();
                let mut diagnostic = Diagnostic::simple_error(primary_message, String::new(), *location);
                diagnostic.secondaries.clear();

                if matches!(invalid_type, InvalidType::StructField {..} | InvalidType::Alias {..}) {
                    diagnostic.add_secondary("This type has an invalid entry point type inside it".to_string(), *location);
                }

                diagnostic.add_note("Note: vectors, references, empty arrays, empty strings, or any type containing them may not be used in main, contract functions, test functions, fuzz functions or foldable functions.".to_string());
                add_invalid_type_to_diagnostic(invalid_type, *location, &mut diagnostic);
                diagnostic
            },
            TypeCheckError::MismatchTraitImplNumParameters {
                expected_num_parameters,
                actual_num_parameters,
                trait_name,
                method_name,
                location,
            } => {
                let plural = if *expected_num_parameters == 1 { "" } else { "s" };
                let primary_message = format!(
                    "`{trait_name}::{method_name}` expects {expected_num_parameters} parameter{plural}, but this method has {actual_num_parameters}");
                Diagnostic::simple_error(primary_message, "".to_string(), *location)
            }
            TypeCheckError::IncorrectTurbofishGenericCount { expected_count, actual_count, location } => {
                let expected_plural = if *expected_count == 1 { "" } else { "s" };
                let actual_plural = if *actual_count == 1 { "was" } else { "were" };
                let msg = format!("Expected {expected_count} generic{expected_plural} from this function, but {actual_count} {actual_plural} provided");
                Diagnostic::simple_error(msg, "".into(), *location)
            },
            TypeCheckError::MacroReturningNonExpr { typ, location } =>  {
                let mut error = Diagnostic::simple_error(
                    format!("Expected macro call to return a `Quoted` but found a(n) `{typ}`"),
                    "Macro calls must return quoted values, otherwise there is no code to insert.".into(),
                    *location,
                );
                error.add_secondary("Hint: remove the `!` from the end of the function name.".to_string(), *location);
                error
            },
            TypeCheckError::DuplicateNamedTypeArg { name, prev_location } => {
                let msg = format!("`{name}` has already been specified");
                let mut error = Diagnostic::simple_error(msg.to_string(), "".to_string(), name.location());
                error.add_secondary(format!("`{name}` previously specified here"), *prev_location);
                error
            },
            TypeCheckError::NoSuchNamedTypeArg { name, item } => {
                let msg = format!("`{item}` has no associated type named `{name}`");
                Diagnostic::simple_error(msg.to_string(), "".to_string(), name.location())
            },
            TypeCheckError::MissingNamedTypeArg { name, item, location } => {
                let msg = format!("`{item}` is missing the associated type `{name}`");
                Diagnostic::simple_error(msg.to_string(), "".to_string(), *location)
            },
            TypeCheckError::Unsafe { location } => {
                Diagnostic::simple_error(error.to_string(), String::new(), *location)
            }
            TypeCheckError::UnsafeFn { location } => {
                Diagnostic::simple_error(error.to_string(), String::new(), *location)
            }
            TypeCheckError::UnspecifiedType { location } => {
                Diagnostic::simple_error(error.to_string(), String::new(), *location)
            }
            TypeCheckError::CyclicType { typ: _, location } => {
                Diagnostic::simple_error(error.to_string(), "Cyclic types have unlimited size and are prohibited in Noir".into(), *location)
            }
            TypeCheckError::CannotInvokeStructFieldFunctionType { method_name, object_type, location } => {
                Diagnostic::simple_error(
                    format!("Cannot invoke function field '{method_name}' on type '{object_type}' as a method"),
                    format!("to call the function stored in '{method_name}', surround the field access with parentheses: '(', ')'"),
                    *location,
                )
            },
            TypeCheckError::TypeAnnotationsNeededForIndex { location } => {
                Diagnostic::simple_error(
                    "Type annotations required before indexing this array or vector".into(),
                    "Type annotations needed before this point, can't decide if this is an array or vector".into(),
                    *location,
                )
            },
            TypeCheckError::UnnecessaryUnsafeBlock { location } => {
                Diagnostic::simple_warning(
                    "Unnecessary `unsafe` block".into(),
                    "".into(),
                    *location,
                )
            },
            TypeCheckError::NestedUnsafeBlock { location } => {
                Diagnostic::simple_warning(
                    "Unnecessary `unsafe` block".into(),
                    "Because it's nested inside another `unsafe` block".into(),
                    *location,
                )
            },
            TypeCheckError::UnreachableCase { location } => {
                Diagnostic::simple_warning(
                    "Unreachable match case".into(),
                    "This pattern is redundant with one or more prior patterns".into(),
                    *location,
                )
            },
            TypeCheckError::MissingCases { cases, location } => {
                let s = if cases.len() == 1 { "" } else { "s" };

                let mut not_shown = String::new();
                let mut shown_cases = cases.iter()
                    .map(|s| format!("`{s}`"))
                    .take(MAX_MISSING_CASES)
                    .collect::<Vec<_>>();

                if cases.len() > MAX_MISSING_CASES {
                    shown_cases.truncate(MAX_MISSING_CASES);
                    not_shown = format!(", and {} more not shown", cases.len() - MAX_MISSING_CASES);
                }

                let shown_cases = shown_cases.join(", ");
                let msg = format!("Missing case{s}: {shown_cases}{not_shown}");
                Diagnostic::simple_error(msg, String::new(), *location)
            },
            TypeCheckError::MissingManyCases { typ, location } => {
                let msg = format!("Missing cases: `{typ}` is non-empty");
                let secondary = "Try adding a match-all pattern: `_`".to_string();
                Diagnostic::simple_error(msg, secondary, *location)
            },
            TypeCheckError::TupleMismatch { tuple_types, actual_count, location } => {
                let msg = format!(
                    "Expected a tuple with {} elements, found one with {} elements",
                    tuple_types.len(),
                    actual_count
                );
                let secondary = format!("The expression the tuple is assigned to has type `({})`", vecmap(tuple_types, ToString::to_string).join(","));
                Diagnostic::simple_error(msg, secondary, *location)
            }
            TypeCheckError::TypeAnnotationNeededOnItem {
                location,
                generic_name,
                item_kind,
                item_name,
                is_numeric,
            } => {
                let message = "Type annotation needed".into();
                let type_or_value = if *is_numeric { "value" } else { "type" };
                let secondary = format!(
                    "Could not determine the {type_or_value} of the generic argument `{generic_name}` declared on the {item_kind} `{item_name}`",
                );
                Diagnostic::simple_error(message, secondary, *location)
            }
            TypeCheckError::TypeAnnotationNeededOnArrayLiteral { is_array, location } => {
                let message = "Type annotation needed".into();
                let array_or_vector = if *is_array { "array" } else { "vector" };
                let secondary = format!("Could not determine the type of the {array_or_vector}");
                Diagnostic::simple_error(message, secondary, *location)
            }
            TypeCheckError::ExpectingOtherError { message, location } => {
                let secondary = "".to_string();
                Diagnostic::simple_error(message.to_string(), secondary, *location)
            }
        }
    }
}

impl<'a> From<&'a NoMatchingImplFoundError> for Diagnostic {
    fn from(error: &'a NoMatchingImplFoundError) -> Self {
        let constraints = &error.constraints;
        let location = error.location;

        assert!(!constraints.is_empty());
        let msg =
            format!("No matching impl found for `{}: {}`", constraints[0].0, constraints[0].1);
        let mut diagnostic = Diagnostic::from_message(&msg, location.file);

        let secondary = format!("No impl for `{}: {}`", constraints[0].0, constraints[0].1);
        diagnostic.add_secondary(secondary, location);

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
        location: Location,
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

        Some(Self { constraints, location })
    }
}

fn add_invalid_type_to_diagnostic(
    invalid_type: &InvalidType,
    location: Location,
    diagnostic: &mut Diagnostic,
) {
    match invalid_type {
        InvalidType::Primitive(typ) => match typ {
            // Use a slightly better message for common types that might be used as entry point types
            Type::Unit => {
                diagnostic
                    .add_secondary("Unit is not a valid entry point type".to_string(), location);
            }
            Type::Reference(..) => {
                diagnostic.add_secondary(
                    format!("Reference is not a valid entry point type. Found: {typ}"),
                    location,
                );
            }
            Type::Vector(..) => {
                diagnostic.add_secondary(
                    format!("Vector is not a valid entry point type. Found: {typ}"),
                    location,
                );
            }
            _ => {
                diagnostic.add_secondary(format!("Invalid entry point type: {typ}"), location);
            }
        },
        InvalidType::Enum(typ) => {
            diagnostic.add_secondary(
                format!("Enum is not yet allowed as an entry point type. Found: {typ}"),
                location,
            );
        }
        InvalidType::EmptyArray(typ) => {
            diagnostic.add_secondary(
                format!("Empty array is not a valid entry point type. Found: {typ}"),
                location,
            );
        }
        InvalidType::EmptyString(typ) => {
            diagnostic.add_secondary(
                format!("Empty string is not a valid entry point type. Found: {typ}"),
                location,
            );
        }
        InvalidType::StructField { struct_name, field_name, invalid_type } => {
            diagnostic.add_secondary(
                format!("Struct {struct_name} has an invalid entry point type"),
                struct_name.location(),
            );
            diagnostic.add_secondary(
                format!("Field {field_name} has an invalid entry point type"),
                field_name.location(),
            );
            add_invalid_type_to_diagnostic(invalid_type, field_name.location(), diagnostic);
        }
        InvalidType::Alias { alias_name, invalid_type } => {
            diagnostic.add_secondary(
                format!("Alias {alias_name} has an invalid entry point type"),
                alias_name.location(),
            );
            add_invalid_type_to_diagnostic(invalid_type, alias_name.location(), diagnostic);
        }
    }
}
