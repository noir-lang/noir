use super::{errors::AcirGenError, generated_acir::GeneratedAcir};
use crate::ssa_refactor::acir_gen::AcirValue;
use crate::ssa_refactor::ir::types::Type as SsaType;
use crate::ssa_refactor::ir::{instruction::Endian, map::TwoWayMap, types::NumericType};
use acvm::acir::{
    brillig_vm::Opcode as BrilligOpcode,
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
use std::{borrow::Cow, hash::Hash};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// High level Type descriptor for Variables.
///
/// One can think of Expression/Witness/Const
/// as low level types which can represent high level types.
///
/// An Expression can represent a u32 for example.
/// We could store this information when we do a range constraint
/// but this information is readily available by the caller so
/// we allow the user to pass it in.
pub(crate) struct AcirType(NumericType);

impl AcirType {
    pub(crate) fn new(typ: NumericType) -> Self {
        Self(typ)
    }

    /// Returns the bit size of the underlying type
    pub(crate) fn bit_size(&self) -> u32 {
        match self.0 {
            NumericType::Signed { bit_size } => bit_size,
            NumericType::Unsigned { bit_size } => bit_size,
            NumericType::NativeField => FieldElement::max_num_bits(),
        }
    }

    /// Returns a boolean type
    fn boolean() -> Self {
        AcirType(NumericType::Unsigned { bit_size: 1 })
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
            SsaType::Numeric(numeric_type) => AcirType(*numeric_type),
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
    vars: TwoWayMap<AcirVar, AcirVarData>,

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

    /// Adds a new Variable to context whose value will
    /// be constrained to be the negation of `var`.
    ///
    /// Note: `Variables` are immutable.
    pub(crate) fn neg_var(&mut self, var: AcirVar) -> AcirVar {
        let var_data = &self.vars[var];
        match var_data {
            AcirVarData::Witness(witness) => {
                let mut expr = Expression::default();
                expr.push_addition_term(-FieldElement::one(), *witness);

                self.add_data(AcirVarData::Expr(expr))
            }
            AcirVarData::Expr(expr) => self.add_data(AcirVarData::Expr(-expr)),
            AcirVarData::Const(constant) => self.add_data(AcirVarData::Const(-*constant)),
        }
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the inverse of `var`.
    pub(crate) fn inv_var(&mut self, var: AcirVar) -> Result<AcirVar, AcirGenError> {
        let var_data = &self.vars[var];
        let inverted_witness = match var_data {
            AcirVarData::Witness(witness) => {
                let expr = Expression::from(*witness);
                self.acir_ir.directive_inverse(&expr)
            }
            AcirVarData::Expr(expr) => self.acir_ir.directive_inverse(expr),
            AcirVarData::Const(constant) => {
                // Note that this will return a 0 if the inverse is not available
                let result_var = self.add_data(AcirVarData::Const(constant.inverse()));
                return Ok(result_var);
            }
        };
        let inverted_var = self.add_data(AcirVarData::Witness(inverted_witness));

        let should_be_one = self.mul_var(inverted_var, var)?;
        self.assert_eq_one(should_be_one);

        Ok(inverted_var)
    }

    /// Constrains the lhs to be equal to the constant value `1`
    pub(crate) fn assert_eq_one(&mut self, var: AcirVar) {
        let one_var = self.add_constant(FieldElement::one());
        self.assert_eq_var(var, one_var);
    }

    /// Returns an `AcirVar` that is `1` if `lhs` equals `rhs` and
    /// 0 otherwise.
    pub(crate) fn eq_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> Result<AcirVar, AcirGenError> {
        let lhs_data = &self.vars[lhs];
        let rhs_data = &self.vars[rhs];

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
        let inputs = vec![AcirValue::Var(lhs, typ), AcirValue::Var(rhs, typ)];
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
        let inputs = vec![AcirValue::Var(lhs, typ), AcirValue::Var(rhs, typ)];
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
            let inputs = vec![AcirValue::Var(a, typ), AcirValue::Var(b, typ)];
            let outputs = self.black_box_function(BlackBoxFunc::AND, inputs)?;
            self.sub_var(max, outputs[0])
        }
    }

    /// Constrains the `lhs` and `rhs` to be equal.
    pub(crate) fn assert_eq_var(&mut self, lhs: AcirVar, rhs: AcirVar) {
        // TODO: could use sub_var and then assert_eq_zero
        let lhs_data = &self.vars[lhs];
        let rhs_data = &self.vars[rhs];

        match (lhs_data, rhs_data) {
            (AcirVarData::Witness(witness), AcirVarData::Expr(expr))
            | (AcirVarData::Expr(expr), AcirVarData::Witness(witness)) => {
                self.acir_ir.assert_is_zero(expr - *witness);
            }
            (AcirVarData::Witness(witness), AcirVarData::Const(constant))
            | (AcirVarData::Const(constant), AcirVarData::Witness(witness)) => self
                .acir_ir
                .assert_is_zero(&Expression::from(*witness) - &Expression::from(*constant)),
            (AcirVarData::Expr(expr), AcirVarData::Const(constant))
            | (AcirVarData::Const(constant), AcirVarData::Expr(expr)) => {
                self.acir_ir.assert_is_zero(expr.clone() - *constant);
            }
            (AcirVarData::Expr(lhs_expr), AcirVarData::Expr(rhs_expr)) => {
                self.acir_ir.assert_is_zero(lhs_expr - rhs_expr);
            }
            (AcirVarData::Witness(lhs_witness), AcirVarData::Witness(rhs_witness)) => self
                .acir_ir
                .assert_is_zero(&Expression::from(*lhs_witness) - &Expression::from(*rhs_witness)),
            (AcirVarData::Const(lhs_constant), AcirVarData::Const(rhs_constant)) => {
                // TODO: for constants, we add it as a gate.
                // TODO: Assuming users will never want to create unsatisfiable programs
                // TODO: We could return an error here instead
                self.acir_ir.assert_is_zero(Expression::from(FieldElement::from(
                    lhs_constant == rhs_constant,
                )));
            }
        };
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the division of `lhs` and `rhs`
    pub(crate) fn div_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        _typ: AcirType,
    ) -> Result<AcirVar, AcirGenError> {
        let inv_rhs = self.inv_var(rhs)?;
        self.mul_var(lhs, inv_rhs)
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the multiplication of `lhs` and `rhs`
    pub(crate) fn mul_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> Result<AcirVar, AcirGenError> {
        let lhs_data = &self.vars[lhs];
        let rhs_data = &self.vars[rhs];
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
        let lhs_data = &self.vars[lhs];
        let rhs_data = &self.vars[rhs];
        let result = match (lhs_data, rhs_data) {
            (AcirVarData::Witness(witness), AcirVarData::Expr(expr))
            | (AcirVarData::Expr(expr), AcirVarData::Witness(witness)) => {
                self.add_data(AcirVarData::Expr(expr + &Expression::from(*witness)))
            }
            (AcirVarData::Witness(witness), AcirVarData::Const(constant))
            | (AcirVarData::Const(constant), AcirVarData::Witness(witness)) => self.add_data(
                AcirVarData::Expr(&Expression::from(*witness) + &Expression::from(*constant)),
            ),
            (AcirVarData::Expr(expr), AcirVarData::Const(constant))
            | (AcirVarData::Const(constant), AcirVarData::Expr(expr)) => {
                self.add_data(AcirVarData::Expr(expr + &Expression::from(*constant)))
            }
            (AcirVarData::Expr(lhs_expr), AcirVarData::Expr(rhs_expr)) => {
                self.add_data(AcirVarData::Expr(lhs_expr + rhs_expr))
            }
            (AcirVarData::Witness(lhs), AcirVarData::Witness(rhs)) => {
                // TODO: impl Add for Witness which returns an Expression instead of the below
                self.add_data(AcirVarData::Expr(&Expression::from(*lhs) + &Expression::from(*rhs)))
            }
            (AcirVarData::Const(lhs_const), AcirVarData::Const(rhs_const)) => {
                self.add_data(AcirVarData::Const(*lhs_const + *rhs_const))
            }
        };
        Ok(result)
    }

    /// Adds a new variable that is constrained to be the logical NOT of `x`.
    ///
    /// `x` must be a 1-bit integer (i.e. a boolean)
    pub(crate) fn not_var(&mut self, x: AcirVar) -> AcirVar {
        // Since `x` can only be 0 or 1, we can derive NOT as 1 - x
        match &self.vars[x] {
            AcirVarData::Const(constant) => {
                self.add_data(AcirVarData::Expr(&Expression::one() - &Expression::from(*constant)))
            }
            AcirVarData::Expr(expr) => self.add_data(AcirVarData::Expr(&Expression::one() - expr)),
            AcirVarData::Witness(witness) => {
                self.add_data(AcirVarData::Expr(&Expression::one() - *witness))
            }
        }
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
        let rhs_data = &self.vars[rhs];

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
    ) -> Result<(AcirVar, AcirVar), AcirGenError> {
        let predicate = Expression::one();

        let lhs_data = &self.vars[lhs];
        let rhs_data = &self.vars[rhs];

        let lhs_expr = lhs_data.to_expression();
        let rhs_expr = rhs_data.to_expression();

        let (quotient, remainder) =
            self.acir_ir.euclidean_division(&lhs_expr, &rhs_expr, bit_size, &predicate)?;

        let quotient_var = self.add_data(AcirVarData::Witness(quotient));
        let remainder_var = self.add_data(AcirVarData::Witness(remainder));

        Ok((quotient_var, remainder_var))
    }

    /// Returns a variable which is constrained to be `lhs mod rhs`
    pub(crate) fn modulo_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        bit_size: u32,
    ) -> Result<AcirVar, AcirGenError> {
        let (_, remainder) = self.euclidean_division_var(lhs, rhs, bit_size)?;
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
    ) -> Result<AcirVar, AcirGenError> {
        let rhs_data = &self.vars[rhs];

        // Compute 2^{rhs}
        let two_pow_rhs = match rhs_data.as_constant() {
            Some(exponent) => FieldElement::from(2_i128).pow(&exponent),
            None => unimplemented!("rhs must be a constant when doing a right shift"),
        };
        let two_pow_rhs_var = self.add_constant(two_pow_rhs);

        self.div_var(lhs, two_pow_rhs_var, typ)
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
    pub(crate) fn numeric_cast_var(
        &mut self,
        variable: AcirVar,
        numeric_type: &NumericType,
    ) -> Result<AcirVar, AcirGenError> {
        let data = &self.vars[variable];
        match numeric_type {
            NumericType::Signed { .. } => todo!("signed integer conversion is unimplemented"),
            NumericType::Unsigned { bit_size } => {
                let data_expr = data.to_expression();
                let witness = self.acir_ir.get_or_create_witness(&data_expr);
                self.acir_ir.range_constraint(witness, *bit_size)?;
            }
            NumericType::NativeField => {
                // If someone has made a cast to a `Field` type then this is a Noop.
                //
                // The reason for doing this in code is for type safety; ie you have an
                // integer, but a function requires the parameter to be a Field.
            }
        }
        Ok(variable)
    }

    /// Returns an `AcirVar` which will be constrained to be lhs mod 2^{rhs}
    pub(crate) fn truncate_var(
        &mut self,
        lhs: AcirVar,
        rhs: u32,
        max_bit_size: u32,
    ) -> Result<AcirVar, AcirGenError> {
        let lhs_data = &self.vars[lhs];
        let lhs_expr = lhs_data.to_expression();

        let result_expr = self.acir_ir.truncate(&lhs_expr, rhs, max_bit_size)?;

        Ok(self.add_data(AcirVarData::Expr(result_expr)))
    }

    /// Returns an `AcirVar` which will be `1` if lhs >= rhs
    /// and `0` otherwise.
    fn more_than_eq_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
        bit_size: u32,
        predicate: Option<AcirVar>,
    ) -> Result<AcirVar, AcirGenError> {
        let lhs_data = &self.vars[lhs];
        let rhs_data = &self.vars[rhs];

        let lhs_expr = lhs_data.to_expression();
        let rhs_expr = rhs_data.to_expression();

        // TODO: check what happens when we do (a as u8) >= (b as u32)
        // TODO: The frontend should shout in this case

        let predicate = predicate.map(|acir_var| {
            let predicate_data = &self.vars[acir_var];
            predicate_data.to_expression().into_owned()
        });
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
        predicate: Option<AcirVar>,
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

                let domain_constant = self.vars[domain_var]
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
                let var_data = &self.vars[input];

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

        let limb_count = self.vars[limb_count_var]
            .as_constant()
            .expect("ICE: limb_size should be a constant")
            .to_u128() as u32;

        let input_expr = &self.vars[input_var].to_expression();

        let limbs = self.acir_ir.radix_le_decompose(input_expr, radix, limb_count)?;

        let mut limb_vars = vecmap(limbs, |witness| {
            let witness = self.add_data(AcirVarData::Witness(witness));
            AcirValue::Var(witness, result_element_type)
        });

        if endian == Endian::Big {
            limb_vars.reverse();
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
            let var_data = &self.vars[acir_var];
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
        self.vars.insert(id, data)
    }

    pub(crate) fn brillig(
        &mut self,
        code: Vec<BrilligOpcode>,
        inputs: Vec<AcirVar>,
        output_len: usize,
    ) -> Vec<AcirVar> {
        let b_inputs =
            vecmap(inputs, |i| BrilligInputs::Single(self.vars[&i].to_expression().into_owned()));
        let outputs = vecmap(0..output_len, |_| self.acir_ir.next_witness_index());
        let outputs_var =
            vecmap(&outputs, |witness_index| self.add_data(AcirVarData::Witness(*witness_index)));
        let b_outputs = vecmap(outputs, BrilligOutputs::Simple);
        self.acir_ir.brillig(code, b_inputs, b_outputs);

        outputs_var
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

#[test]
fn repeat_op() {
    let mut ctx = AcirContext::default();

    let var_a = ctx.add_variable();
    let var_b = ctx.add_variable();

    // Multiplying the same variables twice should yield
    // the same output.
    let var_c = ctx.mul_var(var_a, var_b);
    let should_be_var_c = ctx.mul_var(var_a, var_b);

    assert_eq!(var_c, should_be_var_c);
}
