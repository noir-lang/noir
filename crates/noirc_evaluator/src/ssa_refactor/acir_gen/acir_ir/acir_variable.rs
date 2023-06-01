use crate::brillig::acvm_brillig::BrilligOpcode;

use crate::ssa_refactor::ir::types::NumericType;

use super::{
    errors::AcirGenError,
    generated_acir::GeneratedAcir,
    memory::{ArrayId, Memory},
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
use std::{borrow::Cow, collections::HashMap, hash::Hash};

#[derive(Debug, Default)]
/// Context object which holds the relationship between
/// `Variables`(AcirVar) and types such as `Expression` and `Witness`
/// which are placed into ACIR.
pub(crate) struct AcirContext {
    /// Map which links Variables to AcirVarData.
    ///
    /// This is a common pattern in this codebase
    /// where `AcirVar` can be seen as a pointer to
    /// `AcirVarData`.
    data: HashMap<AcirVar, AcirVarData>,
    /// Map which links `AcirVarData` to Variables.
    ///
    /// This is so that we can lookup
    data_reverse_map: HashMap<AcirVarData, AcirVar>,

    /// An in-memory representation of ACIR.
    ///
    /// This struct will progressively be populated
    /// based on the methods called.
    /// For example, If one was to add two Variables together,
    /// then the `acir_ir` will be populated to assert this
    /// addition.
    acir_ir: GeneratedAcir,

    /// Maps an `AcirVar` to its known bit size.
    variables_to_bit_sizes: HashMap<AcirVar, u32>,
    /// Maps the elements of virtual arrays to their `AcirVar` elements
    memory: Memory,
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
        let var_data = &self.data[&var];
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
    pub(crate) fn inv_var(&mut self, var: AcirVar) -> AcirVar {
        let var_data = &self.data[&var];
        let inverted_witness = match var_data {
            AcirVarData::Witness(witness) => {
                let expr = Expression::from(*witness);
                self.acir_ir.directive_inverse(&expr)
            }
            AcirVarData::Expr(expr) => self.acir_ir.directive_inverse(expr),
            AcirVarData::Const(constant) => {
                // Note that this will return a 0 if the inverse is not available
                return self.add_data(AcirVarData::Const(constant.inverse()));
            }
        };
        let inverted_var = self.add_data(AcirVarData::Witness(inverted_witness));

        let should_be_one = self.mul_var(inverted_var, var);
        self.assert_eq_one(should_be_one);

        inverted_var
    }

    /// Constrains the lhs to be equal to the constant value `1`
    pub(crate) fn assert_eq_one(&mut self, var: AcirVar) {
        let one_var = self.add_constant(FieldElement::one());
        self.assert_eq_var(var, one_var);
    }

    /// Returns an `AcirVar` that is `1` if `lhs` equals `rhs` and
    /// 0 otherwise.
    pub(crate) fn eq_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> AcirVar {
        let lhs_data = &self.data[&lhs];
        let rhs_data = &self.data[&rhs];

        let lhs_expr = lhs_data.to_expression();
        let rhs_expr = rhs_data.to_expression();

        let is_equal_witness = self.acir_ir.is_equal(&lhs_expr, &rhs_expr);
        self.add_data(AcirVarData::Witness(is_equal_witness))
    }

    /// Returns an `AcirVar` that is the XOR result of `lhs` & `rhs`.
    pub(crate) fn xor_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> Result<AcirVar, AcirGenError> {
        let lhs_bit_size = *self
            .variables_to_bit_sizes
            .get(&lhs)
            .expect("ICE: XOR applied to field type, this should be caught by the type system");
        let rhs_bit_size = *self
            .variables_to_bit_sizes
            .get(&lhs)
            .expect("ICE: XOR applied to field type, this should be caught by the type system");
        assert_eq!(lhs_bit_size, rhs_bit_size, "ICE: Operands to XOR require equal bit size");

        let outputs = self.black_box_function(BlackBoxFunc::XOR, vec![lhs, rhs])?;
        let result = outputs[0];
        self.variables_to_bit_sizes.insert(result, lhs_bit_size);
        Ok(result)
    }

    /// Returns an `AcirVar` that is the AND result of `lhs` & `rhs`.
    pub(crate) fn and_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> Result<AcirVar, AcirGenError> {
        let lhs_bit_size = *self
            .variables_to_bit_sizes
            .get(&lhs)
            .expect("ICE: AND applied to field type, this should be caught by the type system");
        let rhs_bit_size = *self
            .variables_to_bit_sizes
            .get(&lhs)
            .expect("ICE: AND applied to field type, this should be caught by the type system");
        assert_eq!(lhs_bit_size, rhs_bit_size, "ICE: Operands to AND require equal bit size");

        let outputs = self.black_box_function(BlackBoxFunc::AND, vec![lhs, rhs])?;
        let result = outputs[0];
        self.variables_to_bit_sizes.insert(result, lhs_bit_size);
        Ok(result)
    }

    /// Returns an `AcirVar` that is the OR result of `lhs` & `rhs`.
    pub(crate) fn or_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> Result<AcirVar, AcirGenError> {
        let lhs_bit_size = *self
            .variables_to_bit_sizes
            .get(&lhs)
            .expect("ICE: OR applied to field type, this should be caught by the type system");
        let rhs_bit_size = *self
            .variables_to_bit_sizes
            .get(&lhs)
            .expect("ICE: OR applied to field type, this should be caught by the type system");
        assert_eq!(lhs_bit_size, rhs_bit_size, "ICE: Operands to OR require equal bit size");
        let bit_size = lhs_bit_size;
        let result = if bit_size == 1 {
            // Operands are booleans
            // a + b - ab
            let sum = self.add_var(lhs, rhs);
            let mul = self.mul_var(lhs, rhs);
            self.sub_var(sum, mul)
        } else {
            // Implement OR in terms of AND
            // max - ((max - a) AND (max -b))
            // Subtracting from max flips the bits, so this is effectively:
            // (NOT a) NAND (NOT b)
            let max = self.add_constant(FieldElement::from((1_u128 << bit_size) - 1));
            let a = self.sub_var(max, lhs);
            let b = self.sub_var(max, rhs);
            // We track the bit sizes of these intermediaries so that blackbox input generation
            // infers them correctly.
            self.variables_to_bit_sizes.insert(a, bit_size);
            self.variables_to_bit_sizes.insert(b, bit_size);
            let output = self.black_box_function(BlackBoxFunc::AND, vec![a, b])?;
            self.sub_var(max, output[0])
        };
        self.variables_to_bit_sizes.insert(result, bit_size);
        Ok(result)
    }

    /// Constrains the `lhs` and `rhs` to be equal.
    pub(crate) fn assert_eq_var(&mut self, lhs: AcirVar, rhs: AcirVar) {
        // TODO: could use sub_var and then assert_eq_zero
        let lhs_data = &self.data[&lhs];
        let rhs_data = &self.data[&rhs];

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
    pub(crate) fn div_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> AcirVar {
        let inv_rhs = self.inv_var(rhs);
        self.mul_var(lhs, inv_rhs)
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the multiplication of `lhs` and `rhs`
    pub(crate) fn mul_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> AcirVar {
        let lhs_data = &self.data[&lhs];
        let rhs_data = &self.data[&rhs];
        match (lhs_data, rhs_data) {
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
        }
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the subtraction of `lhs` and `rhs`
    pub(crate) fn sub_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> AcirVar {
        let neg_rhs = self.neg_var(rhs);
        self.add_var(lhs, neg_rhs)
    }

    /// Adds a new Variable to context whose value will
    /// be constrained to be the addition of `lhs` and `rhs`
    pub(crate) fn add_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> AcirVar {
        let lhs_data = &self.data[&lhs];
        let rhs_data = &self.data[&rhs];
        match (lhs_data, rhs_data) {
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
        }
    }

    /// Adds a new variable that is constrained to be the logical NOT of `x`.
    ///
    /// `x` must be a 1-bit integer (i.e. a boolean)
    pub(crate) fn not_var(&mut self, x: AcirVar) -> AcirVar {
        assert_eq!(
            self.variables_to_bit_sizes.get(&x),
            Some(&1),
            "ICE: NOT op applied to non-bool"
        );
        let data = &self.data[&x];
        // Since `x` can only be 0 or 1, we can derive NOT as 1 - x
        match data {
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
    pub(crate) fn shift_left_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> AcirVar {
        let rhs_data = &self.data[&rhs];

        // Compute 2^{rhs}
        let two_pow_rhs = match rhs_data.as_constant() {
            Some(exponent) => FieldElement::from(2_i128).pow(&exponent),
            None => unimplemented!("rhs must be a constant when doing a right shift"),
        };
        let two_pow_rhs_var = self.add_constant(two_pow_rhs);

        self.mul_var(lhs, two_pow_rhs_var)
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
    pub(crate) fn shift_right_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> AcirVar {
        let rhs_data = &self.data[&rhs];

        // Compute 2^{rhs}
        let two_pow_rhs = match rhs_data.as_constant() {
            Some(exponent) => FieldElement::from(2_i128).pow(&exponent),
            None => unimplemented!("rhs must be a constant when doing a right shift"),
        };
        let two_pow_rhs_var = self.add_constant(two_pow_rhs);

        self.div_var(lhs, two_pow_rhs_var)
    }

    /// Converts the `AcirVar` to a `Witness` if it hasn't been already, and appends it to the
    /// `GeneratedAcir`'s return witnesses.
    pub(crate) fn return_var(&mut self, acir_var: AcirVar) {
        let acir_var_data = self.data.get(&acir_var).expect("ICE: return of undeclared AcirVar");
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
        let data = &self.data[&variable];
        match numeric_type {
            NumericType::Signed { .. } => todo!("signed integer conversion is unimplemented"),
            NumericType::Unsigned { bit_size } => {
                let data_expr = data.to_expression();
                let witness = self.acir_ir.get_or_create_witness(&data_expr);
                self.acir_ir.range_constraint(witness, *bit_size)?;
                // Log the bit size for this variable
                self.variables_to_bit_sizes.insert(variable, *bit_size);
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

    /// Returns an `AcirVar` which will be `1` if lhs >= rhs
    /// and `0` otherwise.
    fn more_than_eq_var(&mut self, lhs: AcirVar, rhs: AcirVar) -> Result<AcirVar, AcirGenError> {
        let lhs_data = &self.data[&lhs];
        let rhs_data = &self.data[&rhs];

        let lhs_expr = lhs_data.to_expression();
        let rhs_expr = rhs_data.to_expression();

        let lhs_bit_size = self.variables_to_bit_sizes.get(&lhs).expect("comparisons cannot be made on variables with no known max bit size. This should have been caught by the frontend");
        let rhs_bit_size = self.variables_to_bit_sizes.get(&rhs).expect("comparisons cannot be made on variables with no known max bit size. This should have been caught by the frontend");

        // This is a conservative choice. Technically, we should just be able to take
        // the bit size of the `lhs` (upper bound), but we need to check/document what happens
        // if the bit_size is not enough to represent both witnesses.
        // An example is the following: (a as u8) >= (b as u32)
        // If the equality is true, then it means that `b` also fits inside
        // of a u8.
        // But its not clear what happens if the equality is false,
        // and we 8 bits to `more_than_eq_comparison`. The conservative
        // choice chosen is to use 32.
        let bit_size = *std::cmp::max(lhs_bit_size, rhs_bit_size);

        let is_greater_than_eq =
            self.acir_ir.more_than_eq_comparison(&lhs_expr, &rhs_expr, bit_size)?;

        Ok(self.add_data(AcirVarData::Witness(is_greater_than_eq)))
    }

    /// Returns an `AcirVar` which will be `1` if lhs < rhs
    /// and `0` otherwise.
    pub(crate) fn less_than_var(
        &mut self,
        lhs: AcirVar,
        rhs: AcirVar,
    ) -> Result<AcirVar, AcirGenError> {
        // Flip the result of calling more than equal method to
        // compute less than.
        let comparison = self.more_than_eq_var(lhs, rhs)?;

        let one = self.add_constant(FieldElement::one());
        let comparison_negated = self.sub_var(one, comparison);

        Ok(comparison_negated)
    }

    /// Calls a Blackbox function on the given inputs and returns a given set of outputs
    /// to represent the result of the blackbox function.
    pub(crate) fn black_box_function(
        &mut self,
        name: BlackBoxFunc,
        inputs: Vec<AcirVar>,
    ) -> Result<Vec<AcirVar>, AcirGenError> {
        // Convert `AcirVar` to `FunctionInput`
        let inputs = self.prepare_inputs_for_black_box_func_call(&inputs)?;

        // Call Black box with `FunctionInput`
        let outputs = self.acir_ir.call_black_box(name, inputs);

        // Convert `Witness` values which are now constrained to be the output of the
        // black box function call into `AcirVar`s.
        //
        // We do not apply range information on the output of the black box function.
        // See issue #1439
        let outputs_var =
            vecmap(&outputs, |witness_index| self.add_data(AcirVarData::Witness(*witness_index)));

        Ok(outputs_var)
    }

    /// Black box function calls expect their inputs to be in a specific data structure (FunctionInput).
    ///
    /// This function will convert `AcirVar` into `FunctionInput` for a blackbox function call.
    fn prepare_inputs_for_black_box_func_call(
        &mut self,
        inputs: &[AcirVar],
    ) -> Result<Vec<FunctionInput>, AcirGenError> {
        let mut witnesses = Vec::new();
        for input in inputs {
            let var_data = &self.data[input];

            // Intrinsics only accept Witnesses. This is not a limitation of the
            // intrinsics, its just how we have defined things. Ideally, we allow
            // constants too.
            let expr = var_data.to_expression();
            let witness = self.acir_ir.get_or_create_witness(&expr);

            // Fetch the number of bits for this variable
            // If it has never been constrained before, then we will
            // encounter None, and so we take the max number of bits for a
            // field element.
            let num_bits = match self.variables_to_bit_sizes.get(input) {
                Some(bits) => {
                    // In Noir, we specify the number of bits to take from the input
                    // by doing the following:
                    //
                    // ```
                    // call_intrinsic(x as u8)
                    // ```
                    //
                    // The `as u8` specifies that we want to take 8 bits from the `x`
                    // variable.
                    //
                    // There were discussions about the SSA IR optimizing out range
                    // constraints. We would want to be careful with it here. For example:
                    //
                    // ```
                    // let x : u32 = y as u32
                    // call_intrinsic(x as u64)
                    // ```
                    // The `x as u64` is redundant since we know that `x` fits within a u32.
                    // However, since the `x as u64` line is being used to tell the intrinsic
                    // to take 64 bits, we cannot remove it.

                    *bits
                }
                None => FieldElement::max_num_bits(),
            };

            witnesses.push(FunctionInput { witness, num_bits });
        }
        Ok(witnesses)
    }

    /// Prints the given `AcirVar`s as witnesses.
    pub(crate) fn print(&mut self, input: Vec<AcirVar>) -> Result<(), AcirGenError> {
        let witnesses = vecmap(input, |acir_var| {
            let var_data = &self.data[&acir_var];
            let expr = var_data.to_expression();
            self.acir_ir.get_or_create_witness(&expr)
        });
        self.acir_ir.call_print(witnesses);
        Ok(())
    }

    /// Terminates the context and takes the resulting `GeneratedAcir`
    pub(crate) fn finish(self) -> GeneratedAcir {
        self.acir_ir
    }

    /// Allocates an array of size `size` and returns a pointer to the array in memory.
    pub(crate) fn allocate_array(&mut self, size: usize) -> ArrayId {
        self.memory.allocate(size)
    }

    /// Stores the given `AcirVar` at the specified address in memory
    pub(crate) fn array_store(
        &mut self,
        array_id: ArrayId,
        index: usize,
        element: AcirVar,
    ) -> Result<(), AcirGenError> {
        self.memory.constant_set(array_id, index, element)
    }

    /// Gets the last stored `AcirVar` at the specified address in memory.
    ///
    /// This errors if nothing was previously stored at the address.
    pub(crate) fn array_load(
        &mut self,
        array_id: ArrayId,
        index: usize,
    ) -> Result<AcirVar, AcirGenError> {
        self.memory.constant_get(array_id, index)
    }

    /// Gets all `AcirVar` elements currently stored at the array.
    ///
    /// This errors if nothing was previously stored any element in the array.
    pub(crate) fn array_load_all(&self, array_id: ArrayId) -> Result<Vec<AcirVar>, AcirGenError> {
        self.memory.constant_get_all(array_id)
    }

    /// Adds `Data` into the context and assigns it a Variable.
    ///
    /// Variable can be seen as an index into the context.
    /// We use a two-way map so that it is efficient to lookup
    /// either the key or the value.
    fn add_data(&mut self, data: AcirVarData) -> AcirVar {
        assert_eq!(self.data.len(), self.data_reverse_map.len());
        if let Some(acir_var) = self.data_reverse_map.get(&data) {
            return *acir_var;
        }

        let id = AcirVar(self.data.len());

        self.data.insert(id, data.clone());
        self.data_reverse_map.insert(data, id);

        id
    }

    pub(crate) fn brillig(&mut self, _code: Vec<BrilligOpcode>) {
        todo!();
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
