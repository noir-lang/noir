use std::collections::BTreeSet;
use std::rc::Rc;

use acvm::FieldElement;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::Location;
use thiserror::Error;

use crate::ast::{
    BinaryOpKind, ConstrainKind, FunctionReturnType, Ident, IntegerBitSize, Signedness,
};
use crate::hir::resolution::errors::ResolverError;
use crate::hir_def::expr::HirBinaryOp;
use crate::hir_def::traits::TraitConstraint;
use crate::hir_def::types::{BinaryTypeOperator, Kind, Type};
use crate::node_interner::NodeInterner;
use crate::signed_field::SignedField;

/// Rust also only shows 3 maximum, even for short patterns.
pub const MAX_MISSING_CASES: usize = 3;

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
    Return(FunctionReturnType, Location),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TypeCheckError {
    #[error("Operator {op:?} cannot be used in a {place:?}")]
    OpCannotBeUsed { op: HirBinaryOp, place: &'static str, location: Location },
    #[error("Division by zero: {lhs} / {rhs}")]
    DivisionByZero { lhs: FieldElement, rhs: FieldElement, location: Location },
    #[error("Modulo on Field elements: {lhs} % {rhs}")]
    ModuloOnFields { lhs: FieldElement, rhs: FieldElement, location: Location },
    #[error("The value `{expr}` cannot fit into `{ty}` which has range `{range}`")]
    OverflowingAssignment { expr: SignedField, ty: Type, range: String, location: Location },
    #[error(
        "The value `{value}` cannot fit into `{kind}` which has a maximum size of `{maximum_size}`"
    )]
    OverflowingConstant {
        value: FieldElement,
        kind: Kind,
        maximum_size: FieldElement,
        location: Location,
    },
    #[error("Evaluating `{op}` on `{lhs}`, `{rhs}` failed")]
    FailingBinaryOp { op: BinaryTypeOperator, lhs: i128, rhs: i128, location: Location },
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
        to_value: FieldElement,
        from_value: FieldElement,
        location: Location,
    },
    #[error("Expected {expected:?} found {found:?}")]
    ArityMisMatch { expected: usize, found: usize, location: Location },
    #[error("Return type in a function cannot be public")]
    PublicReturnType { typ: Type, location: Location },
    #[error("Cannot cast type {from}, 'as' is only for primitive field or integer types")]
    InvalidCast { from: Type, location: Location, reason: String },
    #[error("Casting value of type {from} to a smaller type ({to})")]
    DownsizingCast { from: Type, to: Type, location: Location, reason: String },
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
    #[error("{kind} cannot be used in an infix operation")]
    InvalidInfixOp { kind: &'static str, location: Location },
    #[error("{kind} cannot be used in a unary operation")]
    InvalidUnaryOp { kind: String, location: Location },
    #[error(
        "Bitwise operations are invalid on Field types. Try casting the operands to a sized integer type first."
    )]
    FieldBitwiseOp { location: Location },
    #[error("Integer cannot be used with type {typ}")]
    IntegerTypeMismatch { typ: Type, location: Location },
    #[error(
        "Cannot use an integer and a Field in a binary operation, try converting the Field into an integer first"
    )]
    IntegerAndFieldBinaryOperation { location: Location },
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
    #[error(
        "The number of bits to use for this bitwise operation is ambiguous. Either the operand's type or return type should be specified"
    )]
    AmbiguousBitWidth { location: Location },
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
    UnusedResultError { expr_type: Type, expr_location: Location },
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
    #[error("Slices cannot be returned from an unconstrained runtime to a constrained runtime")]
    UnconstrainedSliceReturnToConstrained { location: Location },
    #[error(
        "Call to unconstrained function is unsafe and must be in an unconstrained function or unsafe block"
    )]
    Unsafe { location: Location },
    #[error("Converting an unconstrained fn to a non-unconstrained fn is unsafe")]
    UnsafeFn { location: Location },
    #[error("Expected a constant, but found `{typ}`")]
    NonConstantEvaluated { typ: Type, location: Location },
    #[error("Slices must have constant length")]
    NonConstantSliceLength { location: Location },
    #[error("Only sized types may be used in the entry point to a program")]
    InvalidTypeForEntryPoint { location: Location },
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
    #[error("Type annotations required before indexing this array or slice")]
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
            TypeCheckError::OpCannotBeUsed { location, .. }
            | TypeCheckError::DivisionByZero { location, .. }
            | TypeCheckError::ModuloOnFields { location, .. }
            | TypeCheckError::OverflowingAssignment { location, .. }
            | TypeCheckError::OverflowingConstant { location, .. }
            | TypeCheckError::FailingBinaryOp { location, .. }
            | TypeCheckError::TypeCannotBeUsed { location, .. }
            | TypeCheckError::TypeMismatch { expr_location: location, .. }
            | TypeCheckError::TypeMismatchWithSource { location, .. }
            | TypeCheckError::TypeKindMismatch { expr_location: location, .. }
            | TypeCheckError::TypeCanonicalizationMismatch { location, .. }
            | TypeCheckError::ArityMisMatch { location, .. }
            | TypeCheckError::PublicReturnType { location, .. }
            | TypeCheckError::InvalidCast { location, .. }
            | TypeCheckError::DownsizingCast { location, .. }
            | TypeCheckError::ExpectedFunction { location, .. }
            | TypeCheckError::AccessUnknownMember { location, .. }
            | TypeCheckError::ParameterCountMismatch { location, .. }
            | TypeCheckError::AssertionParameterCountMismatch { location, .. }
            | TypeCheckError::GenericCountMismatch { location, .. }
            | TypeCheckError::UnconstrainedMismatch { location, .. }
            | TypeCheckError::UnsupportedCast { location }
            | TypeCheckError::TupleIndexOutOfBounds { location, .. }
            | TypeCheckError::VariableMustBeMutable { location, .. }
            | TypeCheckError::CannotMutateImmutableVariable { location, .. }
            | TypeCheckError::MutableCaptureWithoutRef { location, .. }
            | TypeCheckError::MutableReferenceToArrayElement { location }
            | TypeCheckError::UnresolvedMethodCall { location, .. }
            | TypeCheckError::CannotInvokeStructFieldFunctionType { location, .. }
            | TypeCheckError::IntegerSignedness { location, .. }
            | TypeCheckError::IntegerBitWidth { location, .. }
            | TypeCheckError::InvalidInfixOp { location, .. }
            | TypeCheckError::InvalidUnaryOp { location, .. }
            | TypeCheckError::FieldBitwiseOp { location }
            | TypeCheckError::IntegerTypeMismatch { location, .. }
            | TypeCheckError::IntegerAndFieldBinaryOperation { location }
            | TypeCheckError::FieldModulo { location }
            | TypeCheckError::FieldNot { location }
            | TypeCheckError::FieldComparison { location }
            | TypeCheckError::InvalidShiftSize { location }
            | TypeCheckError::AmbiguousBitWidth { location }
            | TypeCheckError::NonHomogeneousArray { first_location: location, .. }
            | TypeCheckError::TypeAnnotationsNeededForMethodCall { location }
            | TypeCheckError::TypeAnnotationsNeededForFieldAccess { location }
            | TypeCheckError::MultipleMatchingImpls { location, .. }
            | TypeCheckError::CallDeprecated { location, .. }
            | TypeCheckError::UnusedResultError { expr_location: location, .. }
            | TypeCheckError::TraitMethodParameterTypeMismatch {
                parameter_location: location,
                ..
            }
            | TypeCheckError::UnneededTraitConstraint { location, .. }
            | TypeCheckError::IncorrectTurbofishGenericCount { location, .. }
            | TypeCheckError::ConstrainedReferenceToUnconstrained { location }
            | TypeCheckError::UnconstrainedReferenceToConstrained { location }
            | TypeCheckError::UnconstrainedSliceReturnToConstrained { location }
            | TypeCheckError::Unsafe { location }
            | TypeCheckError::UnsafeFn { location }
            | TypeCheckError::NonConstantEvaluated { location, .. }
            | TypeCheckError::NonConstantSliceLength { location }
            | TypeCheckError::InvalidTypeForEntryPoint { location }
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
            | TypeCheckError::NestedUnsafeBlock { location } => *location,

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
            TypeCheckError::OpCannotBeUsed { op, place, location } => Diagnostic::simple_error(
                format!("The operator {op:?} cannot be used in a {place}"),
                String::new(),
                *location,
            ),
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

            TypeCheckError::ExpectedFunction { location, .. }
            | TypeCheckError::AccessUnknownMember { location, .. }
            | TypeCheckError::UnsupportedCast { location }
            | TypeCheckError::TupleIndexOutOfBounds { location, .. }
            | TypeCheckError::VariableMustBeMutable { location, .. }
            | TypeCheckError::CannotMutateImmutableVariable { location, .. }
            | TypeCheckError::UnresolvedMethodCall { location, .. }
            | TypeCheckError::IntegerSignedness { location, .. }
            | TypeCheckError::IntegerBitWidth { location, .. }
            | TypeCheckError::InvalidInfixOp { location, .. }
            | TypeCheckError::InvalidUnaryOp { location, .. }
            | TypeCheckError::FieldBitwiseOp { location, .. }
            | TypeCheckError::IntegerTypeMismatch { location, .. }
            | TypeCheckError::FieldComparison { location, .. }
            | TypeCheckError::AmbiguousBitWidth { location, .. }
            | TypeCheckError::IntegerAndFieldBinaryOperation { location }
            | TypeCheckError::OverflowingAssignment { location, .. }
            | TypeCheckError::OverflowingConstant { location, .. }
            | TypeCheckError::FailingBinaryOp { location, .. }
            | TypeCheckError::FieldModulo { location }
            | TypeCheckError::FieldNot { location }
            | TypeCheckError::ConstrainedReferenceToUnconstrained { location }
            | TypeCheckError::UnconstrainedReferenceToConstrained { location }
            | TypeCheckError::UnconstrainedSliceReturnToConstrained { location }
            | TypeCheckError::NonConstantEvaluated { location, .. }
            | TypeCheckError::NonConstantSliceLength { location }
            | TypeCheckError::StringIndexAssign { location }
            | TypeCheckError::InvalidShiftSize { location } => {
                Diagnostic::simple_error(error.to_string(), String::new(), *location)
            }
            TypeCheckError::MutableCaptureWithoutRef { name, location } => Diagnostic::simple_error(
                format!("Mutable variable {name} captured in lambda must be a mutable reference"),
                "Use '&mut' instead of 'mut' to capture a mutable variable.".to_string(),
                *location,
            ),
            TypeCheckError::MutableReferenceToArrayElement { location } => {
                Diagnostic::simple_error("Mutable references to array elements are currently unsupported".into(), "Try storing the element in a fresh variable first".into(), *location)
            },
            TypeCheckError::PublicReturnType { typ, location } => Diagnostic::simple_error(
                "Functions cannot declare a public return type".to_string(),
                format!("return type is {typ}"),
                *location,
            ),
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
                    Source::ArrayElements => format!("Cannot compare {expected} and {actual}, the array element types differ"),
                    Source::ArrayLen => format!("Can only compare arrays of the same length. Here LHS is of length {expected}, and RHS is {actual}"),
                    Source::StringLen => format!("Can only compare strings of the same length. Here LHS is of length {expected}, and RHS is {actual}"),
                    Source::Comparison => format!("Unsupported types for comparison: {expected} and {actual}"),
                    Source::BinOp(kind) => format!("Unsupported types for operator `{kind}`: {expected} and {actual}"),
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
            TypeCheckError::UnusedResultError { expr_type, expr_location } => {
                let msg = format!("Unused expression result of type {expr_type}");
                Diagnostic::simple_warning(msg, String::new(), *expr_location)
            }
            TypeCheckError::NoMatchingImplFound(error) => error.into(),
            TypeCheckError::UnneededTraitConstraint { trait_name, typ, location } => {
                let msg = format!("Constraint for `{typ}: {trait_name}` is not needed, another matching impl is already in scope");
                Diagnostic::simple_warning(msg, "Unnecessary trait constraint in where clause".into(), *location)
            }
            TypeCheckError::InvalidTypeForEntryPoint { location } => Diagnostic::simple_error(
                "Only sized types may be used in the entry point to a program".to_string(),
                "Slices, references, or any type containing them may not be used in main, contract functions, or foldable functions".to_string(), *location),
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
                    "Type annotations required before indexing this array or slice".into(), 
                    "Type annotations needed before this point, can't decide if this is an array or slice".into(),
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
