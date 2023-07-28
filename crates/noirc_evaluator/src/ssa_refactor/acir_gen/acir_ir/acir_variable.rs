use super::{errors::AcirGenError, generated_acir::GeneratedAcir};
use crate::brillig::brillig_gen::brillig_directive;
use crate::ssa_refactor::acir_gen::{AcirDynamicArray, AcirValue};
use crate::ssa_refactor::ir::types::Type as SsaType;
use crate::ssa_refactor::ir::{instruction::Endian, types::NumericType};
use acvm::acir::circuit::opcodes::{BlockId, MemOp};
use acvm::acir::circuit::Opcode;
use acvm::acir::{
    brillig::Opcode as BrilligOpcode,
    circuit::brillig::{BrilligInputs, BrilligOutputs},
};

use acvm::{
    acir::{
        circuit::opcodes::FunctionInput,
        native_types::{Expression, Witness},
        BlackBoxFunc,
    },
    FieldElement,
};
use iter_extended::vecmap;
use noirc_errors::Location;
use std::collections::HashMap;
use std::{borrow::Cow, hash::Hash};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// High level Type descriptor for Variables.
///
/// One can think of Expression/Witness/Const
/// as low level types which can represent high level types.
///
/// An Expression can represent a u32 for example.
/// We could store this information when we do a range constraint
/// but this information is readily available by the caller so
/// we allow the user to pass it in.
pub(crate) enum AcirType {
    NumericType(NumericType),
    Array(Vec<AcirType>, usize),
}

impl AcirType {
    pub(crate) fn new(typ: NumericType) -> Self {
        Self::NumericType(typ)
    }

    /// Returns the bit size of the underlying type
    pub(crate) fn bit_size(&self) -> u32 {
        match self {
            AcirType::NumericType(numeric_type) => match numeric_type {
                NumericType::Signed { bit_size } => *bit_size,
                NumericType::Unsigned { bit_size } => *bit_size,
                NumericType::NativeField => FieldElement::max_num_bits(),
            },
            AcirType::Array(_, _) => unreachable!("cannot fetch bit size of array type"),
        }
    }

    /// Returns a field type
    pub(crate) fn field() -> Self {
        AcirType::NumericType(NumericType::NativeField)
    }

    /// Returns a boolean type
    fn boolean() -> Self {
        AcirType::NumericType(NumericType::Unsigned { bit_size: 1 })
    }

    /// True if type is signed
    pub(crate) fn is_signed(&self) -> bool {
        let numeric_type = match self {
            AcirType::NumericType(numeric_type) => numeric_type,
            AcirType::Array(_, _) => return false,
        };
        matches!(numeric_type, NumericType::Signed { .. })
    }
}

impl From<SsaType> for AcirType {
    fn from(value: SsaType) -> Self {
        AcirType::from(&value)
    }
}

impl<'a> From<&'a SsaType> for AcirType {
    fn from(value: &SsaType) -> Self {
        match value {
            SsaType::Numeric(numeric_type) => AcirType::NumericType(*numeric_type),
            SsaType::Array(elements, size) => {
                let elements = elements.iter().map(|e| e.into()).collect();
                AcirType::Array(elements, *size)
            }
            _ => unreachable!("The type {value}  cannot be represented in ACIR"),
        }
    }
}

#[derive(Debug, Default)]
/// Context object which holds the relationship between
/// `Variables`(AcirVar) and types such as `Expression` and `Witness`
/// which are placed into ACIR.
pub(crate) struct AcirContext {
    /// Two-way map that links `AcirVar` to `AcirVarData`.
    ///
    /// The vars object is an instance of the `TwoWayMap`, which provides a bidirectional mapping between `AcirVar` and `AcirVarData`.
    vars: HashMap<AcirVar, AcirVarData>,

    /// An in-memory representation of ACIR.
    ///
    /// This struct will progressively be populated
    /// based on the methods called.
    /// For example, If one was to add two Variables together,
    /// then the `acir_ir` will be populated to assert this
    /// addition.
    acir_ir: GeneratedAcir,
}

impl AcirContext {
    /// Adds a constant to the context and assigns a Variable to represent it
    pub(crate) fn add_constant(&mut self, constant: FieldElement) -> AcirVar {
        let constant_data = AcirVarData::Const(constant);
        self.add_data(constant_data)
    }

    /// Adds a Variable to the context, whose exact value is resolved at
    /// runtime.
    pub(crate) fn add_variable(&mut self) -> AcirVar {
        let var_index = self.acir_ir.next_witness_index();

        let var_data = AcirVarData::Witness(var_index);

        self.add_data(var_data)
    }

    pub(crate) fn get_location(&mut self) -> Option<Location> {
        self.acir_ir.current_location
    }

    pub(crate) fn set_location(&mut self, location: Option<Location>) {
        self.acir_ir.current_location = location;
    }

    /// True if the given AcirVar refers to a constant one value
    pub(crate) fn is_constant_one(&self, var: &AcirVar) -> bool {
        match self.vars[var] {
            AcirVarData::Const(field) => field.is_one(),
            _ => false,
        }
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the negation of `var`.
    ///
    /// Note: `Variables` are immutable.
    pub(crate) fn neg_var(&mut self, var: AcirVar) -> AcirVar {
        let var_data = &self.vars[&var];
        let result_data = if let AcirVarData::Const(constant) = var_data {
            AcirVarData::Const(-*constant)
        } else {
            AcirVarData::Expr(-var_data.to_expression().as_ref())
        };
        self.add_data(result_data)
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the inverse of `var`.
    pub(crate) fn inv_var(
        &mut self,
        var: AcirVar,
        predicate: AcirVar,
    ) -> Result<AcirVar, AcirGenError> {
        let var_data = &self.vars[&var];
        if let AcirVarData::Const(constant) = var_data {
            // Note that this will return a 0 if the inverse is not available
            let result_var = self.add_data(AcirVarData::Const(constant.inverse()));
            return Ok(result_var);
        }

        // Compute the inverse with brillig code
        let inverse_code = brillig_directive::directive_invert();
        let field_type = AcirType::NumericType(NumericType::NativeField);

        let results = self.brillig(
            predicate,
            inverse_code,
            vec![AcirValue::Var(var, field_type.clone())],
            vec![field_type],
        );
        let inverted_var = Self::expect_one_var(results);

        let should_be_one = self.mul_var(inverted_var, var)?;
        self.maybe_eq_predicate(should_be_one, predicate)?;

        Ok(inverted_var)
    }

    // Constrains `var` to be equal to the constant value `1`
    pub(crate) fn assert_eq_one(&mut self, var: AcirVar) -> Result<(), AcirGenError> {
        let one = self.add_constant(FieldElement::one());
        self.assert_eq_var(var, one)
    }

    // Constrains `var` to be equal to predicate if the predicate is true
    // or to be equal to 0 if the predicate is false.
    //
    // Since we multiply `var` by the predicate, this is a no-op if the predicate is false
    pub(crate) fn maybe_eq_predicate(
        &mut self,
        var: AcirVar,
        predicate: AcirVar,
    ) -> Result<(), AcirGenError> {
        let pred_mul_var = self.mul_var(var, predicate)?;
        self.assert_eq_var(pred_mul_var, predicate)
    }

    // Returns the variable from the results, assuming it is the only result
    fn expect_one_var(results: Vec<AcirValue>) -> AcirVar {
        assert_eq!(results.len(), 1);
        match results[0] {
            AcirValue::Var(var, _) => var,
            AcirValue::DynamicArray(_) | AcirValue::Array(_) => {
                unreachable!("ICE - expected a variable")
            }
        }
    }

    /// Returns an `AcirVar` that is `1` if `lhs` equals `rhs` and
    /// 0 otherwise.
    pub(crate) fn eq_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> Result<AcirVar, AcirGenError> {
        let lhs_data = &self.vars[&lhs];
        let rhs_data = &self.vars[&rhs];

        let lhs_expr = lhs_data.to_expression();
        let rhs_expr = rhs_data.to_expression();

        let is_equal_witness = self.acir_ir.is_equal(&lhs_expr, &rhs_expr);
        let result_var = self.add_data(AcirVarData::Witness(is_equal_witness));
        Ok(result_var)
    }

    /// Returns an `AcirVar` that is the XOR result of `lhs` & `rhs`.
    pub(crate) fn xor_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        typ: AcirType,
    ) -> Result<AcirVar, AcirGenError> {
        let inputs = vec![AcirValue::Var(lhs, typ.clone()), AcirValue::Var(rhs, typ)];
        let outputs = self.black_box_function(BlackBoxFunc::XOR, inputs)?;
        Ok(outputs[0])
    }

    /// Returns an `AcirVar` that is the AND result of `lhs` & `rhs`.
    pub(crate) fn and_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        typ: AcirType,
    ) -> Result<AcirVar, AcirGenError> {
        let inputs = vec![AcirValue::Var(lhs, typ.clone()), AcirValue::Var(rhs, typ)];
        let outputs = self.black_box_function(BlackBoxFunc::AND, inputs)?;
        Ok(outputs[0])
    }

    /// Returns an `AcirVar` that is the OR result of `lhs` & `rhs`.
    pub(crate) fn or_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        typ: AcirType,
    ) -> Result<AcirVar, AcirGenError> {
        let bit_size = typ.bit_size();
        if bit_size == 1 {
            // Operands are booleans
            // a + b - ab
            let sum = self.add_var(lhs, rhs)?;
            let mul = self.mul_var(lhs, rhs)?;
            self.sub_var(sum, mul)
        } else {
            // Implement OR in terms of AND
            // max - ((max - a) AND (max -b))
            // Subtracting from max flips the bits, so this is effectively:
            // (NOT a) NAND (NOT b)
            let max = self.add_constant(FieldElement::from((1_u128 << bit_size) - 1));
            let a = self.sub_var(max, lhs)?;
            let b = self.sub_var(max, rhs)?;
            let inputs = vec![AcirValue::Var(a, typ.clone()), AcirValue::Var(b, typ)];
            let outputs = self.black_box_function(BlackBoxFunc::AND, inputs)?;
            self.sub_var(max, outputs[0])
        }
    }

    /// Constrains the `lhs` and `rhs` to be equal.
    pub(crate) fn assert_eq_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> Result<(), AcirGenError> {
        // TODO: could use sub_var and then assert_eq_zero
        let lhs_data = &self.vars[&lhs];
        let rhs_data = &self.vars[&rhs];
        if let (AcirVarData::Const(lhs_const), AcirVarData::Const(rhs_const)) = (lhs_data, rhs_data)
        {
            if lhs_const == rhs_const {
                // Constraint is always true and need not be added
                Ok(())
            } else {
                // Constraint is always false - this program is unprovable
                Err(AcirGenError::BadConstantEquality {
                    lhs: *lhs_const,
                    rhs: *rhs_const,
                    location: self.get_location(),
                })
            }
        } else {
            self.acir_ir.assert_is_zero(
                lhs_data.to_expression().as_ref() - rhs_data.to_expression().as_ref(),
            );
            Ok(())
        }
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the division of `lhs` and `rhs`
    pub(crate) fn div_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        typ: AcirType,
        predicate: AcirVar,
    ) -> Result<AcirVar, AcirGenError> {
        let numeric_type = match typ {
            AcirType::NumericType(numeric_type) => numeric_type,
            AcirType::Array(_, _) => {
                todo!("cannot divide arrays. This should have been caught by the frontend")
            }
        };
        match numeric_type {
            NumericType::NativeField => {
                let inv_rhs = self.inv_var(rhs, predicate)?;
                self.mul_var(lhs, inv_rhs)
            }
            NumericType::Unsigned { bit_size } => {
                let (quotient_var, _remainder_var) =
                    self.euclidean_division_var(lhs, rhs, bit_size, predicate)?;
                Ok(quotient_var)
            }
            NumericType::Signed { bit_size } => {
                let (quotient_var, _remainder_var) =
                    self.signed_division_var(lhs, rhs, bit_size)?;
                Ok(quotient_var)
            }
        }
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the multiplication of `lhs` and `rhs`
    pub(crate) fn mul_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> Result<AcirVar, AcirGenError> {
        let lhs_data = &self.vars[&lhs];
        let rhs_data = &self.vars[&rhs];
        let result = match (lhs_data, rhs_data) {
            (AcirVarData::Witness(witness), AcirVarData::Expr(expr))
            | (AcirVarData::Expr(expr), AcirVarData::Witness(witness)) => {
                let expr_as_witness = self.acir_ir.get_or_create_witness(expr);
                let mut expr = Expression::default();
                expr.push_multiplication_term(FieldElement::one(), *witness, expr_as_witness);

                self.add_data(AcirVarData::Expr(expr))
            }
            (AcirVarData::Witness(witness), AcirVarData::Const(constant))
            | (AcirVarData::Const(constant), AcirVarData::Witness(witness)) => {
                let mut expr = Expression::default();
                expr.push_addition_term(*constant, *witness);
                self.add_data(AcirVarData::Expr(expr))
            }
            (AcirVarData::Const(constant), AcirVarData::Expr(expr))
            | (AcirVarData::Expr(expr), AcirVarData::Const(constant)) => {
                self.add_data(AcirVarData::Expr(expr * *constant))
            }
            (AcirVarData::Witness(lhs_witness), AcirVarData::Witness(rhs_witness)) => {
                let mut expr = Expression::default();
                expr.push_multiplication_term(FieldElement::one(), *lhs_witness, *rhs_witness);
                self.add_data(AcirVarData::Expr(expr))
            }
            (AcirVarData::Const(lhs_constant), AcirVarData::Const(rhs_constant)) => {
                self.add_data(AcirVarData::Const(*lhs_constant * *rhs_constant))
            }
            (AcirVarData::Expr(lhs_expr), AcirVarData::Expr(rhs_expr)) => {
                let lhs_expr_as_witness = self.acir_ir.get_or_create_witness(lhs_expr);
                let rhs_expr_as_witness = self.acir_ir.get_or_create_witness(rhs_expr);
                let mut expr = Expression::default();
                expr.push_multiplication_term(
                    FieldElement::one(),
                    lhs_expr_as_witness,
                    rhs_expr_as_witness,
                );
                self.add_data(AcirVarData::Expr(expr))
            }
        };
        Ok(result)
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the subtraction of `lhs` and `rhs`
    pub(crate) fn sub_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> Result<AcirVar, AcirGenError> {
        let neg_rhs = self.neg_var(rhs);
        self.add_var(lhs, neg_rhs)
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the addition of `lhs` and `rhs`
    pub(crate) fn add_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> Result<AcirVar, AcirGenError> {
        let lhs_data = &self.vars[&lhs];
        let rhs_data = &self.vars[&rhs];
        let result_data = if let (AcirVarData::Const(lhs_const), AcirVarData::Const(rhs_const)) =
            (lhs_data, rhs_data)
        {
            AcirVarData::Const(*lhs_const + *rhs_const)
        } else {
            let sum = lhs_data.to_expression().as_ref() + rhs_data.to_expression().as_ref();
            AcirVarData::Expr(sum)
        };
        Ok(self.add_data(result_data))
    }

    /// Adds a new variable that is constrained to be the logical NOT of `x`.
    pub(crate) fn not_var(&mut self, x: AcirVar, typ: AcirType) -> Result<AcirVar, AcirGenError> {
        let bit_size = typ.bit_size();
        // Subtracting from max flips the bits
        let max = self.add_constant(FieldElement::from((1_u128 << bit_size) - 1));
        self.sub_var(max, x)
    }

    /// Returns an `AcirVar` that is constrained to be `lhs << rhs`.
    ///
    /// We convert left shifts to multiplications, so this is equivalent to
    /// `lhs * 2^rhs`.
    ///
    /// We currently require `rhs` to be a constant
    /// however this can be extended, see #1478.
    pub(crate) fn shift_left_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        _typ: AcirType,
    ) -> Result<AcirVar, AcirGenError> {
        let rhs_data = &self.vars[&rhs];

        // Compute 2^{rhs}
        let two_pow_rhs = match rhs_data.as_constant() {
            Some(exponent) => FieldElement::from(2_i128).pow(&exponent),
            None => unimplemented!("rhs must be a constant when doing a right shift"),
        };
        let two_pow_rhs_var = self.add_constant(two_pow_rhs);

        self.mul_var(lhs, two_pow_rhs_var)
    }

    /// Returns the quotient and remainder such that lhs = rhs * quotient + remainder
    fn euclidean_division_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        bit_size: u32,
        predicate: AcirVar,
    ) -> Result<(AcirVar, AcirVar), AcirGenError> {
        let lhs_data = &self.vars[&lhs];
        let rhs_data = &self.vars[&rhs];
        let predicate_data = &self.vars[&predicate];

        let lhs_expr = lhs_data.to_expression();
        let rhs_expr = rhs_data.to_expression();
        let predicate_expr = predicate_data.to_expression();

        let (quotient, remainder) =
            self.acir_ir.euclidean_division(&lhs_expr, &rhs_expr, bit_size, &predicate_expr)?;

        let quotient_var = self.add_data(AcirVarData::Witness(quotient));
        let remainder_var = self.add_data(AcirVarData::Witness(remainder));

        Ok((quotient_var, remainder_var))
    }

    /// Returns the quotient and remainder such that lhs = rhs * quotient + remainder
    /// and |remainder| < |rhs|
    /// and remainder has the same sign than lhs
    /// Note that this is not the euclidian division, where we have instead remainder < |rhs|
    ///
    ///
    ///
    ///

    fn signed_division_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        bit_size: u32,
    ) -> Result<(AcirVar, AcirVar), AcirGenError> {
        let lhs_data = &self.vars[&lhs].clone();
        let rhs_data = &self.vars[&rhs].clone();

        let lhs_expr = lhs_data.to_expression();
        let rhs_expr = rhs_data.to_expression();
        let l_witness = self.acir_ir.get_or_create_witness(&lhs_expr);
        let r_witness = self.acir_ir.get_or_create_witness(&rhs_expr);
        assert_ne!(bit_size, 0, "signed integer should have at least one bit");
        let (q, r) =
            self.acir_ir.signed_division(&l_witness.into(), &r_witness.into(), bit_size)?;

        let q_vd = AcirVarData::Expr(q);
        let r_vd = AcirVarData::Expr(r);
        Ok((self.add_data(q_vd), self.add_data(r_vd)))
    }

    /// Returns a variable which is constrained to be `lhs mod rhs`
    pub(crate) fn modulo_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        bit_size: u32,
        predicate: AcirVar,
    ) -> Result<AcirVar, AcirGenError> {
        let (_, remainder) = self.euclidean_division_var(lhs, rhs, bit_size, predicate)?;
        Ok(remainder)
    }

    /// Returns an `AcirVar` that is constrained to be `lhs >> rhs`.
    ///
    /// We convert right shifts to divisions, so this is equivalent to
    /// `lhs / 2^rhs`.
    ///
    /// We currently require `rhs` to be a constant
    /// however this can be extended, see #1478.
    ///
    /// This code is doing a field division instead of an integer division,
    /// see #1479 about how this is expected to change.
    pub(crate) fn shift_right_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        typ: AcirType,
        predicate: AcirVar,
    ) -> Result<AcirVar, AcirGenError> {
        let rhs_data = &self.vars[&rhs];

        // Compute 2^{rhs}
        let two_pow_rhs = match rhs_data.as_constant() {
            Some(exponent) => FieldElement::from(2_i128).pow(&exponent),
            None => unimplemented!("rhs must be a constant when doing a right shift"),
        };
        let two_pow_rhs_var = self.add_constant(two_pow_rhs);

        self.div_var(lhs, two_pow_rhs_var, typ, predicate)
    }

    /// Converts the `AcirVar` to a `Witness` if it hasn't been already, and appends it to the
    /// `GeneratedAcir`'s return witnesses.
    pub(crate) fn return_var(&mut self, acir_var: AcirVar) {
        let acir_var_data = self.vars.get(&acir_var).expect("ICE: return of undeclared AcirVar");
        // TODO: Add caching to prevent expressions from being needlessly duplicated
        let witness = match acir_var_data {
            AcirVarData::Const(constant) => {
                self.acir_ir.get_or_create_witness(&Expression::from(*constant))
            }
            AcirVarData::Expr(expr) => self.acir_ir.get_or_create_witness(expr),
            AcirVarData::Witness(witness) => *witness,
        };
        self.acir_ir.push_return_witness(witness);
    }

    /// Constrains the `AcirVar` variable to be of type `NumericType`.
    pub(crate) fn range_constrain_var(
        &mut self,
        variable: AcirVar,
        numeric_type: &NumericType,
    ) -> Result<AcirVar, AcirGenError> {
        let data = &self.vars[&variable];
        match numeric_type {
            NumericType::Signed { bit_size } | NumericType::Unsigned { bit_size } => {
                let data_expr = data.to_expression();
                let witness = self.acir_ir.get_or_create_witness(&data_expr);
                self.acir_ir.range_constraint(witness, *bit_size)?;
            }
            NumericType::NativeField => {
                // Range constraining a Field is a no-op
            }
        }
        Ok(variable)
    }

    /// Returns an `AcirVar` which will be constrained to be lhs mod 2^{rhs}
    /// In order to do this, we simply perform euclidian division of lhs by 2^{rhs}
    /// The remainder of the division is then lhs mod 2^{rhs}
    pub(crate) fn truncate_var(
        &mut self,
        lhs: AcirVar,
        rhs: u32,
        max_bit_size: u32,
    ) -> Result<AcirVar, AcirGenError> {
        let lhs_data = &self.vars[&lhs];
        let lhs_expr = lhs_data.to_expression();

        // 2^{rhs}
        let divisor = FieldElement::from(2_i128).pow(&FieldElement::from(rhs as i128));
        // Computes lhs = 2^{rhs} * q + r
        let (_, remainder) = self.acir_ir.euclidean_division(
            &lhs_expr,
            &Expression::from_field(divisor),
            max_bit_size,
            &Expression::one(),
        )?;

        Ok(self.add_data(AcirVarData::Expr(Expression::from(remainder))))
    }

    /// Returns an `AcirVar` which will be `1` if lhs >= rhs
    /// and `0` otherwise.
    fn more_than_eq_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        bit_size: u32,
        predicate: AcirVar,
    ) -> Result<AcirVar, AcirGenError> {
        let lhs_data = &self.vars[&lhs];
        let rhs_data = &self.vars[&rhs];

        let lhs_expr = lhs_data.to_expression();
        let rhs_expr = rhs_data.to_expression();

        // TODO: check what happens when we do (a as u8) >= (b as u32)
        // TODO: The frontend should shout in this case

        let predicate_data = &self.vars[&predicate];
        let predicate = predicate_data.to_expression().into_owned();

        let is_greater_than_eq =
            self.acir_ir.more_than_eq_comparison(&lhs_expr, &rhs_expr, bit_size, predicate)?;

        Ok(self.add_data(AcirVarData::Witness(is_greater_than_eq)))
    }

    /// Returns an `AcirVar` which will be `1` if lhs < rhs
    /// and `0` otherwise.
    pub(crate) fn less_than_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        bit_size: u32,
        predicate: AcirVar,
    ) -> Result<AcirVar, AcirGenError> {
        // Flip the result of calling more than equal method to
        // compute less than.
        let comparison = self.more_than_eq_var(lhs, rhs, bit_size, predicate)?;

        let one = self.add_constant(FieldElement::one());
        self.sub_var(one, comparison) // comparison_negated
    }

    /// Calls a Blackbox function on the given inputs and returns a given set of outputs
    /// to represent the result of the blackbox function.
    pub(crate) fn black_box_function(
        &mut self,
        name: BlackBoxFunc,
        mut inputs: Vec<AcirValue>,
    ) -> Result<Vec<AcirVar>, AcirGenError> {
        // Separate out any arguments that should be constants
        let constants = match name {
            BlackBoxFunc::Pedersen => {
                // The last argument of pedersen is the domain separator, which must be a constant
                let domain_var =
                    inputs.pop().expect("ICE: Pedersen call requires domain separator").into_var();

                let domain_constant = self.vars[&domain_var]
                    .as_constant()
                    .expect("ICE: Domain separator must be a constant");

                vec![domain_constant]
            }
            _ => vec![],
        };

        // Convert `AcirVar` to `FunctionInput`
        let inputs = self.prepare_inputs_for_black_box_func_call(inputs)?;

        // Call Black box with `FunctionInput`
        let outputs = self.acir_ir.call_black_box(name, inputs, constants);

        // Convert `Witness` values which are now constrained to be the output of the
        // black box function call into `AcirVar`s.
        //
        // We do not apply range information on the output of the black box function.
        // See issue #1439
        Ok(vecmap(&outputs, |witness_index| self.add_data(AcirVarData::Witness(*witness_index))))
    }

    /// Black box function calls expect their inputs to be in a specific data structure (FunctionInput).
    ///
    /// This function will convert `AcirVar` into `FunctionInput` for a blackbox function call.
    fn prepare_inputs_for_black_box_func_call(
        &mut self,
        inputs: Vec<AcirValue>,
    ) -> Result<Vec<FunctionInput>, AcirGenError> {
        let mut witnesses = Vec::new();
        for input in inputs {
            for (input, typ) in input.flatten() {
                let var_data = &self.vars[&input];

                // Intrinsics only accept Witnesses. This is not a limitation of the
                // intrinsics, its just how we have defined things. Ideally, we allow
                // constants too.
                let expr = var_data.to_expression();
                let witness = self.acir_ir.get_or_create_witness(&expr);
                let num_bits = typ.bit_size();
                witnesses.push(FunctionInput { witness, num_bits });
            }
        }
        Ok(witnesses)
    }

    /// Returns a vector of `AcirVar`s constrained to be the decomposition of the given input
    /// over given radix.
    ///
    /// The `AcirVar`s for the `radix_var` and `limb_count_var` must be a constant
    ///
    /// TODO: support radix larger than field modulus
    pub(crate) fn radix_decompose(
        &mut self,
        endian: Endian,
        input_var: AcirVar,
        radix_var: AcirVar,
        limb_count_var: AcirVar,
        result_element_type: AcirType,
    ) -> Result<Vec<AcirValue>, AcirGenError> {
        let radix =
            self.vars[&radix_var].as_constant().expect("ICE: radix should be a constant").to_u128()
                as u32;

        let limb_count = self.vars[&limb_count_var]
            .as_constant()
            .expect("ICE: limb_size should be a constant")
            .to_u128() as u32;

        let input_expr = &self.vars[&input_var].to_expression();

        let bit_size = u32::BITS - (radix - 1).leading_zeros();
        let limbs = self.acir_ir.radix_le_decompose(input_expr, radix, limb_count, bit_size)?;

        let mut limb_vars = vecmap(limbs, |witness| {
            let witness = self.add_data(AcirVarData::Witness(witness));
            AcirValue::Var(witness, result_element_type.clone())
        });

        if endian == Endian::Big {
            limb_vars.reverse();
        }

        // For legacy reasons (see #617) the to_radix interface supports 256 bits even though
        // FieldElement::max_num_bits() is only 254 bits. Any limbs beyond the specified count
        // become zero padding.
        let max_decomposable_bits: u32 = 256;
        let limb_count_with_padding = max_decomposable_bits / bit_size;
        let zero = self.add_constant(FieldElement::zero());
        while limb_vars.len() < limb_count_with_padding as usize {
            limb_vars.push(AcirValue::Var(zero, result_element_type.clone()));
        }

        Ok(vec![AcirValue::Array(limb_vars.into())])
    }

    /// Returns `AcirVar`s constrained to be the bit decomposition of the provided input
    pub(crate) fn bit_decompose(
        &mut self,
        endian: Endian,
        input_var: AcirVar,
        limb_count_var: AcirVar,
        result_element_type: AcirType,
    ) -> Result<Vec<AcirValue>, AcirGenError> {
        let two_var = self.add_constant(FieldElement::from(2_u128));
        self.radix_decompose(endian, input_var, two_var, limb_count_var, result_element_type)
    }

    /// Prints the given `AcirVar`s as witnesses.
    pub(crate) fn print(&mut self, input: Vec<AcirValue>) -> Result<(), AcirGenError> {
        let input = Self::flatten_values(input);

        let witnesses = vecmap(input, |acir_var| {
            let var_data = &self.vars[&acir_var];
            let expr = var_data.to_expression();
            self.acir_ir.get_or_create_witness(&expr)
        });
        self.acir_ir.call_print(witnesses);
        Ok(())
    }

    /// Flatten the given Vector of AcirValues into a single vector of only variables.
    /// Each AcirValue::Array in the vector is recursively flattened, so each element
    /// will flattened into the resulting Vec. E.g. flatten_values([1, [2, 3]) == [1, 2, 3].
    fn flatten_values(values: Vec<AcirValue>) -> Vec<AcirVar> {
        let mut acir_vars = Vec::with_capacity(values.len());
        for value in values {
            Self::flatten_value(&mut acir_vars, value);
        }
        acir_vars
    }

    /// Recursive helper for flatten_values to flatten a single AcirValue into the result vector.
    pub(crate) fn flatten_value(acir_vars: &mut Vec<AcirVar>, value: AcirValue) {
        match value {
            AcirValue::Var(acir_var, _) => acir_vars.push(acir_var),
            AcirValue::Array(array) => {
                for value in array {
                    Self::flatten_value(acir_vars, value);
                }
            }
            AcirValue::DynamicArray(_) => unreachable!("Cannot flatten a dynamic array"),
        }
    }

    /// Terminates the context and takes the resulting `GeneratedAcir`
    pub(crate) fn finish(self) -> GeneratedAcir {
        self.acir_ir
    }

    /// Adds `Data` into the context and assigns it a Variable.
    ///
    /// Variable can be seen as an index into the context.
    /// We use a two-way map so that it is efficient to lookup
    /// either the key or the value.
    fn add_data(&mut self, data: AcirVarData) -> AcirVar {
        let id = AcirVar(self.vars.len());
        self.vars.insert(id, data);
        id
    }

    pub(crate) fn brillig(
        &mut self,
        predicate: AcirVar,
        code: Vec<BrilligOpcode>,
        inputs: Vec<AcirValue>,
        outputs: Vec<AcirType>,
    ) -> Vec<AcirValue> {
        let b_inputs = vecmap(inputs, |i| match i {
            AcirValue::Var(var, _) => {
                BrilligInputs::Single(self.vars[&var].to_expression().into_owned())
            }
            AcirValue::Array(vars) => {
                let mut var_expressions: Vec<Expression> = Vec::new();
                for var in vars {
                    self.brillig_array_input(&mut var_expressions, var);
                }
                BrilligInputs::Array(var_expressions)
            }
            AcirValue::DynamicArray(_) => {
                let mut var_expressions = Vec::new();
                self.brillig_array_input(&mut var_expressions, i);
                BrilligInputs::Array(var_expressions)
            }
        });

        let mut b_outputs = Vec::new();
        let outputs_var = vecmap(outputs, |output| match output {
            AcirType::NumericType(_) => {
                let witness_index = self.acir_ir.next_witness_index();
                b_outputs.push(BrilligOutputs::Simple(witness_index));
                let var = self.add_data(AcirVarData::Witness(witness_index));
                AcirValue::Var(var, output.clone())
            }
            AcirType::Array(element_types, size) => {
                let (acir_value, witnesses) = self.brillig_array_output(&element_types, size);
                b_outputs.push(BrilligOutputs::Array(witnesses));
                acir_value
            }
        });
        let predicate = self.vars[&predicate].to_expression().into_owned();
        self.acir_ir.brillig(Some(predicate), code, b_inputs, b_outputs);

        outputs_var
    }

    fn brillig_array_input(&mut self, var_expressions: &mut Vec<Expression>, input: AcirValue) {
        match input {
            AcirValue::Var(var, _) => {
                var_expressions.push(self.vars[&var].to_expression().into_owned());
            }
            AcirValue::Array(vars) => {
                for var in vars {
                    self.brillig_array_input(var_expressions, var);
                }
            }
            AcirValue::DynamicArray(AcirDynamicArray { block_id, len }) => {
                for i in 0..len {
                    // We generate witnesses corresponding to the array values
                    let index = AcirValue::Var(
                        self.add_constant(FieldElement::from(i as u128)),
                        AcirType::NumericType(NumericType::NativeField),
                    );
                    let index_var = index.into_var();

                    let value_read_var = self.read_from_memory(block_id, &index_var);
                    let value_read = AcirValue::Var(
                        value_read_var,
                        AcirType::NumericType(NumericType::NativeField),
                    );

                    self.brillig_array_input(var_expressions, value_read);
                }
            }
        }
    }

    /// Recursively create acir values for returned arrays. This is necessary because a brillig returned array can have nested arrays as elements.
    /// A singular array of witnesses is collected for a top level array, by deflattening the assigned witnesses at each level.
    fn brillig_array_output(
        &mut self,
        element_types: &[AcirType],
        size: usize,
    ) -> (AcirValue, Vec<Witness>) {
        let mut witnesses = Vec::new();
        let mut array_values = im::Vector::new();
        for _ in 0..size {
            for element_type in element_types {
                match element_type {
                    AcirType::Array(nested_element_types, nested_size) => {
                        let (nested_acir_value, mut nested_witnesses) =
                            self.brillig_array_output(nested_element_types, *nested_size);
                        witnesses.append(&mut nested_witnesses);
                        array_values.push_back(nested_acir_value);
                    }
                    AcirType::NumericType(_) => {
                        let witness_index = self.acir_ir.next_witness_index();
                        witnesses.push(witness_index);
                        let var = self.add_data(AcirVarData::Witness(witness_index));
                        array_values.push_back(AcirValue::Var(var, element_type.clone()));
                    }
                }
            }
        }
        (AcirValue::Array(array_values), witnesses)
    }

    /// Generate output variables that are constrained to be the sorted inputs
    /// The outputs are the sorted inputs iff
    /// outputs are sorted and
    /// outputs are a permutation of the inputs
    pub(crate) fn sort(
        &mut self,
        inputs: Vec<AcirVar>,
        bit_size: u32,
        predicate: AcirVar,
    ) -> Result<Vec<AcirVar>, AcirGenError> {
        let len = inputs.len();
        // Convert the inputs into expressions
        let inputs_expr = vecmap(inputs, |input| self.vars[&input].to_expression().into_owned());
        // Generate output witnesses
        let outputs_witness = vecmap(0..len, |_| self.acir_ir.next_witness_index());
        let output_expr =
            vecmap(&outputs_witness, |witness_index| Expression::from(*witness_index));
        let outputs_var = vecmap(&outputs_witness, |witness_index| {
            self.add_data(AcirVarData::Witness(*witness_index))
        });

        // Enforce the outputs to be a permutation of the inputs
        self.acir_ir.permutation(&inputs_expr, &output_expr);

        // Enforce the outputs to be sorted
        for i in 0..(outputs_var.len() - 1) {
            self.less_than_constrain(outputs_var[i], outputs_var[i + 1], bit_size, predicate)?;
        }

        Ok(outputs_var)
    }
    /// Converts an AcirVar to a Witness
    fn var_to_witness(&mut self, var: AcirVar) -> Witness {
        let var_data = self.vars.get(&var).expect("ICE: undeclared AcirVar");
        self.acir_ir.get_or_create_witness(&var_data.to_expression())
    }

    /// Constrain lhs to be less than rhs
    fn less_than_constrain(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        bit_size: u32,
        predicate: AcirVar,
    ) -> Result<(), AcirGenError> {
        let lhs_less_than_rhs = self.more_than_eq_var(rhs, lhs, bit_size, predicate)?;
        self.maybe_eq_predicate(lhs_less_than_rhs, predicate)
    }

    /// Returns a Variable that is constrained to be the result of reading
    /// from the memory `block_id` at the given `index`.
    pub(crate) fn read_from_memory(&mut self, block_id: BlockId, index: &AcirVar) -> AcirVar {
        // Fetch the witness corresponding to the index
        let index_witness = self.var_to_witness(*index);

        // Create a Variable to hold the result of the read and extract the corresponding Witness
        let value_read_var = self.add_variable();
        let value_read_witness = self.var_to_witness(value_read_var);

        // Add the memory read operation to the list of opcodes
        let op = MemOp::read_at_mem_index(index_witness.into(), value_read_witness);
        self.acir_ir.opcodes.push(Opcode::MemoryOp { block_id, op });

        value_read_var
    }

    /// Constrains the Variable `value` to be the new value located at `index` in the memory `block_id`.
    pub(crate) fn write_to_memory(&mut self, block_id: BlockId, index: &AcirVar, value: &AcirVar) {
        // Fetch the witness corresponding to the index
        //
        let index_witness = self.var_to_witness(*index);

        // Fetch the witness corresponding to the value to be written
        let value_write_witness = self.var_to_witness(*value);

        // Add the memory write operation to the list of opcodes
        let op = MemOp::write_to_mem_index(index_witness.into(), value_write_witness.into());
        self.acir_ir.opcodes.push(Opcode::MemoryOp { block_id, op });
    }

    /// Initializes an array in memory with the given values `optional_values`.
    /// If `optional_values` is empty, then the array is initialized with zeros.
    pub(crate) fn initialize_array(
        &mut self,
        block_id: BlockId,
        len: usize,
        optional_values: Option<&[AcirValue]>,
    ) {
        // If the optional values are supplied, then we fill the initialized
        // array with those values. If not, then we fill it with zeros.
        let initialized_values = match optional_values {
            None => {
                let zero = self.add_constant(FieldElement::zero());
                let zero_witness = self.var_to_witness(zero);
                vec![zero_witness; len]
            }
            Some(optional_values) => vecmap(optional_values, |value| {
                let value = value.clone().into_var();
                self.var_to_witness(value)
            }),
        };

        self.acir_ir.opcodes.push(Opcode::MemoryInit { block_id, init: initialized_values });
    }
}

/// Enum representing the possible values that a
/// Variable can be given.
#[derive(Debug, Eq, Clone)]
enum AcirVarData {
    Witness(Witness),
    Expr(Expression),
    Const(FieldElement),
}

impl PartialEq for AcirVarData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Witness(l0), Self::Witness(r0)) => l0 == r0,
            (Self::Expr(l0), Self::Expr(r0)) => l0 == r0,
            (Self::Const(l0), Self::Const(r0)) => l0 == r0,
            _ => false,
        }
    }
}

// TODO: check/test this hash impl
impl std::hash::Hash for AcirVarData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl AcirVarData {
    /// Returns a FieldElement, if the underlying `AcirVarData`
    /// represents a constant.
    pub(crate) fn as_constant(&self) -> Option<FieldElement> {
        if let AcirVarData::Const(field) = self {
            return Some(*field);
        }
        None
    }
    /// Converts all enum variants to an Expression.
    pub(crate) fn to_expression(&self) -> Cow<Expression> {
        match self {
            AcirVarData::Witness(witness) => Cow::Owned(Expression::from(*witness)),
            AcirVarData::Expr(expr) => Cow::Borrowed(expr),
            AcirVarData::Const(constant) => Cow::Owned(Expression::from(*constant)),
        }
    }
}

/// A Reference to an `AcirVarData`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AcirVar(usize);
