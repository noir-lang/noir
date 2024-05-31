use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use noirc_errors::Location;

use crate::hir_def::types::Type;
use crate::{BinaryTypeOperator, TypeBinding, TypeBindings, TypeVariable, TypeVariableKind};

use super::NodeInterner;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArithId {
    Dummy,
    Hash(u64),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Default)]
pub struct ArithGenericId(usize);

impl ArithGenericId {
    fn offset(&self, offset_amount: usize) -> Self {
        ArithGenericId(self.0 + offset_amount)
    }
}

/// An arithmetic expression can be a variable, constant, or binary operation.
///
/// An ArithExpr::Variable contains a NamedGeneric's TypeVariable and name,
/// as well as the ArithGenericId that points to the corresponding TypeVariable
/// in Type::GenericArith
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ArithExpr {
    Op { kind: ArithOpKind, lhs: Box<ArithExpr>, rhs: Box<ArithExpr> },
    Variable(TypeVariable, Rc<String>, ArithGenericId),
    Constant(u64),
}

impl ArithExpr {
    pub fn try_constant(&self) -> Option<u64> {
        match self {
            Self::Constant(x) => Some(*x),
            _ => None,
        }
    }

    pub fn evaluate(
        &self,
        _interner: &NodeInterner,
        arguments: &Vec<(u64, Type)>,
    ) -> Result<u64, ArithExprError> {
        match self {
            Self::Op { kind, lhs, rhs } => {
                // TODO: interner unused, see https://github.com/noir-lang/noir/issues/5150
                let interner = NodeInterner::default();
                let lhs = lhs.evaluate(&interner, arguments)?;
                let rhs = rhs.evaluate(&interner, arguments)?;
                kind.evaluate(lhs, rhs)
            }
            Self::Variable(binding, name, index) => {
                if let Some((result, _other_var)) = arguments.get(index.0) {
                    // // TODO: assertion fails https://github.com/noir-lang/noir/issues/5150
                    // // (remove other_var if unneeded)
                    //
                    // let mut fresh_bindings = TypeBindings::new();
                    // assert!(Type::NamedGeneric(binding.clone(), name.clone())
                    //     .try_unify(other_var, &mut fresh_bindings, &interner.arith_constraints,)
                    //     .is_ok());

                    Ok(*result)
                } else {
                    Err(ArithExprError::UnboundVariable {
                        binding: binding.clone(),
                        name: name.to_string(),
                    })
                }
            }
            Self::Constant(result) => Ok(*result),
        }
    }

    /// Apply Type::follow_bindings to each named generic
    /// and return the updated version as well as any new generics
    fn follow_bindings(
        &self,
        interner: &NodeInterner,
        offset_amount: &mut usize,
    ) -> (Self, Vec<Type>) {
        match self {
            Self::Op { kind, lhs, rhs } => {
                let (lhs, mut lhs_new_generics) = lhs.follow_bindings(interner, offset_amount);
                let (rhs, mut rhs_new_generics) = rhs.follow_bindings(interner, offset_amount);
                let rhs = rhs.offset_generic_indices(lhs_new_generics.len());
                lhs_new_generics.append(&mut rhs_new_generics);
                (Self::Op { kind: *kind, lhs: Box::new(lhs), rhs: Box::new(rhs) }, lhs_new_generics)
            }
            Self::Variable(binding, name, index) => {
                match Type::NamedGeneric(binding.clone(), name.clone()).follow_bindings() {
                    Type::GenericArith(arith_id, generics) => {
                        let (arith_expr, _location) = interner.get_arith_expression(arith_id);
                        let arith_expr = arith_expr.offset_generic_indices(*offset_amount);
                        *offset_amount = arith_expr.max_generic_index().0;
                        (arith_expr, generics)
                    }

                    Type::NamedGeneric(new_binding, new_name) => (Self::Variable(new_binding, new_name, *index), vec![]),
                    Type::TypeVariable(_new_binding, TypeVariableKind::Constant(value)) => (Self::Constant(value), vec![]),
                    Type::TypeVariable(new_binding, _kind) => {
                        Self::Variable(new_binding, name.clone(), *index).follow_bindings(interner, offset_amount)
                    }
                    Type::Constant(value) => {
                        (ArithExpr::Constant(value), vec![])
                    }
                    other => panic!("ICE: follow_bindings on Type::NamedGeneric produced a result other than a variable or constant: {:?}", other),
                }
            }
            Self::Constant(result) => (Self::Constant(*result), vec![]),
        }
    }

    /// map over Self::Variable's
    fn map_variables<F>(&self, f: &mut F) -> Self
    where
        F: FnMut(
            &TypeVariable,
            &Rc<String>,
            ArithGenericId,
        ) -> (TypeVariable, Rc<String>, ArithGenericId),
    {
        match self {
            Self::Op { kind, lhs, rhs } => {
                let lhs = Box::new(lhs.map_variables(f));
                let rhs = Box::new(rhs.map_variables(f));
                Self::Op { kind: *kind, lhs, rhs }
            }
            Self::Variable(binding, name, index) => {
                let (new_binding, new_name, new_index) = f(binding, name, *index);
                Self::Variable(new_binding, new_name, new_index)
            }
            Self::Constant(result) => Self::Constant(*result),
        }
    }

    /// normal form: sort nodes at each branch
    fn nf(&self) -> Self {
        match self {
            Self::Op { kind, lhs, rhs } => {
                match kind {
                    // commutative cases
                    ArithOpKind::Add | ArithOpKind::Mul => {
                        let (lhs, rhs) = if lhs <= rhs {
                            (lhs.clone(), rhs.clone())
                        } else {
                            (rhs.clone(), lhs.clone())
                        };
                        Self::Op { kind: *kind, lhs, rhs }
                    }
                    _ => Self::Op { kind: *kind, lhs: lhs.clone(), rhs: rhs.clone() },
                }
            }
            other => other.clone(),
        }
    }

    /// Replace `TypeVariable`s`in Self::Variable with the given Type::TypeVariable's,
    /// indexed by `ArithGenericId``
    fn impute_variables(&self, generics: &[Type]) -> Self {
        self.map_variables(&mut |_var: &TypeVariable, name: &Rc<String>, index: ArithGenericId| {
            let new_var = generics
                .get(index.0)
                .expect("all variables in a GenericArith ArithExpr to be in the included Vec")
                .get_outer_type_variable()
                .expect("all args to GenericArith to be NamedGeneric/TypeVariable's");
            (new_var, name.clone(), index)
        })
    }

    pub(crate) fn to_id(&self) -> ArithId {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        ArithId::Hash(hasher.finish())
    }

    pub(crate) fn offset_generic_indices(&self, offset_amount: usize) -> Self {
        match self {
            Self::Op { kind, lhs, rhs } => {
                let lhs = Box::new(lhs.offset_generic_indices(offset_amount));
                let rhs = Box::new(rhs.offset_generic_indices(offset_amount));
                Self::Op { kind: *kind, lhs, rhs }
            }
            Self::Variable(binding, name, index) => {
                Self::Variable(binding.clone(), name.clone(), index.offset(offset_amount))
            }
            Self::Constant(result) => Self::Constant(*result),
        }
    }

    pub(crate) fn max_generic_index(&self) -> ArithGenericId {
        match self {
            Self::Op { lhs, rhs, .. } => {
                let lhs_max = lhs.max_generic_index();
                let rhs_max = rhs.max_generic_index();
                std::cmp::max(lhs_max, rhs_max)
            }
            Self::Variable(_binding, _name, index) => *index,
            Self::Constant(_result) => ArithGenericId::default(),
        }
    }
}

impl std::fmt::Display for ArithExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArithExpr::Op { kind, lhs, rhs } => write!(f, "{lhs} {kind} {rhs}"),
            ArithExpr::Variable(binding, name, _index) => match &*binding.borrow() {
                TypeBinding::Bound(binding) => binding.fmt(f),
                TypeBinding::Unbound(_) if name.is_empty() => write!(f, "_"),
                TypeBinding::Unbound(_) => write!(f, "{name}"),
            },
            ArithExpr::Constant(x) => x.fmt(f),
        }
    }
}

/// Constant < Variable < Op
impl PartialOrd for ArithExpr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Self::Op { kind, lhs, rhs } => match other {
                Self::Op { kind: other_kind, lhs: other_lhs, rhs: other_rhs } => {
                    (kind, lhs, rhs).partial_cmp(&(other_kind, other_lhs, other_rhs))
                }
                Self::Variable(..) => Some(Ordering::Greater),
                Self::Constant(..) => Some(Ordering::Greater),
            },
            Self::Variable(binding, name, index) => match other {
                Self::Op { .. } => Some(Ordering::Less),
                Self::Variable(other_binding, other_name, other_index) => (
                    binding.id().0,
                    name,
                    index,
                )
                    .partial_cmp(&(other_binding.id().0, other_name, other_index)),
                Self::Constant(..) => Some(Ordering::Greater),
            },
            Self::Constant(self_result) => match other {
                Self::Op { .. } => Some(Ordering::Less),
                Self::Variable(..) => Some(Ordering::Less),
                Self::Constant(other_result) => self_result.partial_cmp(other_result),
            },
        }
    }
}

/// A binary operation that's allowed in an ArithExpr
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub enum ArithOpKind {
    Mul,
    Add,
    Sub,
}

impl ArithOpKind {
    /// Returns an error on overflow/underflow
    pub fn evaluate(&self, x: u64, y: u64) -> Result<u64, ArithExprError> {
        match self {
            Self::Mul => Ok(x * y),
            Self::Add => Ok(x + y),
            Self::Sub => x.checked_sub(y).ok_or(ArithExprError::SubUnderflow { lhs: x, rhs: y }),
        }
    }

    pub fn from_binary_type_operator(value: BinaryTypeOperator) -> Option<Self> {
        match value {
            BinaryTypeOperator::Addition => Some(ArithOpKind::Add),
            BinaryTypeOperator::Multiplication => Some(ArithOpKind::Mul),
            BinaryTypeOperator::Subtraction => Some(ArithOpKind::Sub),
            _ => None,
        }
    }
}

impl std::fmt::Display for ArithOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArithOpKind::Mul => write!(f, "*"),
            ArithOpKind::Add => write!(f, "+"),
            ArithOpKind::Sub => write!(f, "-"),
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ArithExprError {
    SubUnderflow { lhs: u64, rhs: u64 },

    UnboundVariable { binding: TypeVariable, name: String },

    EvaluateUnexpectedType { unexpected_type: Type },
}

impl std::fmt::Display for ArithExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SubUnderflow { lhs, rhs } => {
                write!(f, "subtracting {} - {} underflowed", lhs, rhs)
            }
            Self::UnboundVariable { binding, name } => {
                let use_elaborator_notice =
                    // TODO: https://github.com/noir-lang/noir/issues/5149
                    "\nIf you're seeing this error inside of a definition that type checks,\n".to_owned() +
                    "running with '--use-elaborator' may fix it. (See Issue#5149 for more info)";
                if let TypeBinding::Unbound(_) = &*binding.borrow() {
                    write!(
                        f,
                        "unbound variable when resolving generic arithmetic: {}{}",
                        name, use_elaborator_notice
                    )
                } else {
                    write!(
                        f,
                        "unbound variable when resolving generic arithmetic: {}{}",
                        binding.borrow(),
                        use_elaborator_notice
                    )
                }
            }
            Self::EvaluateUnexpectedType { unexpected_type } => {
                write!(f, "unexpected type when evaluating to u64: {}", unexpected_type)
            }
        }
    }
}

/// Whether either the LHS or RHS of an ArithConstraint needs to be interned,
/// which can happen when unifying
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum NeedsInterning {
    Lhs(ArithExpr),
    Rhs(ArithExpr),
    Neither,
}

/// An arithmetic constraint, composed of the parameters from two Type::GenericArith's and an
/// optional NeedsInterning case.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ArithConstraint {
    pub lhs: ArithId,
    pub lhs_generics: Vec<Type>,
    pub rhs: ArithId,
    pub rhs_generics: Vec<Type>,
    pub needs_interning: NeedsInterning,
}

impl ArithConstraint {
    pub(crate) fn evaluate_generics_to_u64(
        generics: &[Type],
        location: &Location,
        interner: &NodeInterner,
    ) -> Result<Vec<(u64, Type)>, ArithExprError> {
        generics
            .iter()
            .cloned()
            .map(|generic| {
                generic.evaluate_to_u64(location, interner).map(|result| (result, generic))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn validate(self, interner: &NodeInterner) -> Result<(), ArithConstraintError> {
        let (lhs, rhs) = match &self.needs_interning {
            NeedsInterning::Lhs(lhs_expr) => (
                (lhs_expr.clone(), Location::dummy()),
                interner.get_arith_expression(self.rhs).clone(),
            ),
            NeedsInterning::Rhs(rhs_expr) => (
                interner.get_arith_expression(self.lhs).clone(),
                (rhs_expr.clone(), Location::dummy()),
            ),
            NeedsInterning::Neither => (
                interner.get_arith_expression(self.lhs).clone(),
                interner.get_arith_expression(self.rhs).clone(),
            ),
        };
        let (lhs_expr, lhs_location) = lhs;
        let (rhs_expr, rhs_location) = rhs;

        // follow NamedGeneric bindings
        let mut current_generic_index_offset = 0;
        let (lhs_expr, lhs_new_generics) =
            lhs_expr.follow_bindings(interner, &mut current_generic_index_offset);
        let (rhs_expr, rhs_new_generics) =
            rhs_expr.follow_bindings(interner, &mut current_generic_index_offset);
        rhs_expr.offset_generic_indices(lhs_new_generics.len());

        let lhs_generics: Vec<_> = self.lhs_generics.into_iter().chain(lhs_new_generics).collect();
        let rhs_generics: Vec<_> = self.rhs_generics.into_iter().chain(rhs_new_generics).collect();

        match Self::evaluate_generics_to_u64(&lhs_generics, &lhs_location, interner).and_then(
            |lhs_generics| {
                let rhs_generics =
                    Self::evaluate_generics_to_u64(&rhs_generics, &rhs_location, interner)?;
                Ok((lhs_generics, rhs_generics))
            },
        ) {
            // all generics resolved
            Ok((lhs_generics, rhs_generics)) => {
                match (
                    lhs_expr.evaluate(interner, &lhs_generics),
                    rhs_expr.evaluate(interner, &rhs_generics),
                ) {
                    (Ok(lhs_evaluated), Ok(rhs_evaluated)) => {
                        if lhs_evaluated == rhs_evaluated {
                            Ok(())
                        } else {
                            Err(ArithConstraintError::EvaluatedToDifferentValues {
                                lhs_evaluated,
                                rhs_evaluated,
                                location: rhs_location,
                                other_location: lhs_location,
                            })
                        }
                    }
                    (lhs_result, rhs_result) => Err(ArithConstraintError::FailedToEvaluate {
                        lhs_expr,
                        rhs_expr,
                        lhs_result,
                        rhs_result,
                        location: lhs_location,
                        other_location: rhs_location,
                    }),
                }
            }
            Err(arith_expr_error) => {
                let mut fresh_bindings = TypeBindings::new();
                let generics_match = lhs_generics.iter().zip(rhs_generics.iter()).all(
                    |(lhs_generic, rhs_generic)| {
                        lhs_generic
                            .try_unify(
                                rhs_generic,
                                &mut fresh_bindings,
                                &interner.arith_constraints,
                            )
                            .is_ok()
                    },
                );
                Type::apply_type_bindings(fresh_bindings);

                if generics_match {
                    // impute the unified lhs_generics into both ArithExpr's
                    let lhs_expr = lhs_expr.impute_variables(&lhs_generics).nf();
                    let rhs_expr = rhs_expr.impute_variables(&lhs_generics).nf();
                    if lhs_expr == rhs_expr {
                        Ok(())
                    } else {
                        Err(ArithConstraintError::DistinctExpressions {
                            lhs_expr: lhs_expr.clone(),
                            rhs_expr: rhs_expr.clone(),
                            generics: lhs_generics.clone(),
                            location: lhs_location,
                            other_location: rhs_location,
                        })
                    }
                } else {
                    Err(ArithConstraintError::ArithExprError {
                        arith_expr_error,
                        location: lhs_location,
                        other_locations: vec![rhs_location],
                    })
                }
            }
        }
    }
}

pub type ArithConstraints = RefCell<Vec<ArithConstraint>>;

#[derive(Debug, thiserror::Error)]
pub enum ArithConstraintError {
    UnresolvedGeneric {
        generic: Type,
        location: Location,
    },
    EvaluatedToDifferentValues {
        lhs_evaluated: u64,
        rhs_evaluated: u64,
        location: Location,
        other_location: Location,
    },
    FailedToEvaluate {
        lhs_expr: ArithExpr,
        rhs_expr: ArithExpr,
        lhs_result: Result<u64, ArithExprError>,
        rhs_result: Result<u64, ArithExprError>,
        location: Location,
        other_location: Location,
    },
    DistinctExpressions {
        lhs_expr: ArithExpr,
        rhs_expr: ArithExpr,
        generics: Vec<Type>,
        location: Location,
        other_location: Location,
    },
    ArithExprError {
        arith_expr_error: ArithExprError,
        location: Location,
        other_locations: Vec<Location>,
    },
}

impl std::fmt::Display for ArithConstraintError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UnresolvedGeneric { generic, .. } => {
                if let Type::NamedGeneric(_, name) = generic {
                    write!(f, "Unresolved generic value: {}", name)
                } else {
                    write!(f, "Unresolved generic value: {}", generic)
                }
            }
            Self::EvaluatedToDifferentValues { lhs_evaluated, rhs_evaluated, .. } => {
                write!(
                    f,
                    "Generic arithmetic evaluated to different values: {} != {}",
                    lhs_evaluated, rhs_evaluated
                )
            }
            Self::FailedToEvaluate { lhs_expr, rhs_expr, lhs_result, rhs_result, .. } => {
                if lhs_result.is_err() {
                    write!(
                        f,
                        "Left hand side of generic arithmetic failed to evaluate: {}\n{}\n",
                        lhs_expr,
                        lhs_result.as_ref().unwrap_err()
                    )?;
                }
                if rhs_result.is_err() {
                    write!(
                        f,
                        "Right hand side of generic arithmetic failed to evaluate: {}\n{}",
                        rhs_expr,
                        rhs_result.as_ref().unwrap_err()
                    )?;
                }
                assert!(
                    lhs_result.is_err() || rhs_result.is_err(),
                    "ArithConstraintError::FailedToEvaluate contains successful evaluation"
                );
                Ok(())
            }
            Self::DistinctExpressions { lhs_expr, rhs_expr, generics, .. } => {
                write!(f, "Generic arithmetic appears to be distinct: {} != {}, where the arguments are: {:?}", lhs_expr, rhs_expr, generics)
            }
            Self::ArithExprError { arith_expr_error, .. } => arith_expr_error.fmt(f),
        }
    }
}

impl ArithConstraintError {
    pub fn location(&self) -> Location {
        match self {
            Self::UnresolvedGeneric { location, .. }
            | Self::EvaluatedToDifferentValues { location, .. }
            | Self::FailedToEvaluate { location, .. }
            | Self::DistinctExpressions { location, .. }
            | Self::ArithExprError { location, .. } => *location,
        }
    }

    pub fn other_locations(&self) -> Vec<Location> {
        match self {
            Self::UnresolvedGeneric { .. } => vec![],

            Self::EvaluatedToDifferentValues { other_location, .. }
            | Self::FailedToEvaluate { other_location, .. }
            | Self::DistinctExpressions { other_location, .. } => vec![*other_location],

            Self::ArithExprError { other_locations, .. } => other_locations.clone(),
        }
    }
}
