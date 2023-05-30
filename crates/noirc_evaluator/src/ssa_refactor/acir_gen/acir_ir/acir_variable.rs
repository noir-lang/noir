use crate::ssa_refactor::ir::types::NumericType;

use super::{errors::AcirGenError, generated_acir::GeneratedAcir};
use acvm::{
    acir::{
        circuit::opcodes::{BlackBoxFuncCall, FunctionInput},
        native_types::{Expression, Witness},
        BlackBoxFunc,
    },
    FieldElement,
};
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
}

impl AcirContext {
    /// Adds a constant to the context and assigns a Variable to represent it
    pub(crate) fn add_constant(&mut self, constant: FieldElement) -> AcirVar {
        let constant_data = AcirVarData::Const(constant);

        if let Some(var) = self.data_reverse_map.get(&constant_data) {
            return *var;
        };

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
    pub(crate) fn intrinsics(
        &mut self,
        name: BlackBoxFunc,
        inputs: Vec<AcirVar>,
    ) -> Result<Vec<AcirVar>, AcirGenError> {
        let inputs = self.prepare_inputs_for_intrinsics_call(name, &inputs)?;

        let output_count = Self::black_box_expected_output_size(name);
        let outputs = vecmap(0..output_count, |_| self.acir_ir.next_witness_index());
        let outputs_var = vecmap(&outputs, |witness_index| self.add_data(AcirVarData::Witness(*witness_index)));

        let black_box_func_call = match name {
            BlackBoxFunc::AES => unimplemented!("AES is not implemented"),
            BlackBoxFunc::AND => {
                BlackBoxFuncCall::AND { lhs: inputs[0], rhs: inputs[1], output: outputs[0] }
            }
            BlackBoxFunc::XOR => {
                BlackBoxFuncCall::XOR { lhs: inputs[0], rhs: inputs[1], output: outputs[0] }
            }
            BlackBoxFunc::RANGE => BlackBoxFuncCall::RANGE { input: inputs[0] },
            BlackBoxFunc::SHA256 => BlackBoxFuncCall::SHA256 { inputs, outputs },
            BlackBoxFunc::Blake2s => BlackBoxFuncCall::Blake2s { inputs, outputs },
            BlackBoxFunc::HashToField128Security => {
                BlackBoxFuncCall::HashToField128Security { inputs, output: outputs[0] }
            }
            BlackBoxFunc::ComputeMerkleRoot => BlackBoxFuncCall::ComputeMerkleRoot {
                leaf: inputs[0],
                index: inputs[1],
                hash_path: inputs[2..].to_vec(),
                output: outputs[0],
            },
            BlackBoxFunc::SchnorrVerify => BlackBoxFuncCall::SchnorrVerify {
                public_key_x: inputs[0],
                public_key_y: inputs[1],
                // Schnorr signature is two field field elements (r,s)
                signature: vec![inputs[2], inputs[3]],
                message: inputs[4..].to_vec(),
                output: outputs[0],
            },
            BlackBoxFunc::Pedersen => BlackBoxFuncCall::Pedersen { inputs, outputs },
            BlackBoxFunc::EcdsaSecp256k1 => BlackBoxFuncCall::EcdsaSecp256k1 {
                // 32 bytes for each public key co-ordinate
                public_key_x: inputs[0..32].to_vec(),
                public_key_y: inputs[32..64].to_vec(),
                // (r,s) are both 32 bytes each, so signature
                // takes up 64 bytes
                signature: inputs[64..128].to_vec(),
                hashed_message: inputs[128..].to_vec(),
                output: outputs[0],
            },
            BlackBoxFunc::FixedBaseScalarMul => {
                BlackBoxFuncCall::FixedBaseScalarMul { input: inputs[0], outputs }
            }
            BlackBoxFunc::Keccak256 => BlackBoxFuncCall::Keccak256 { inputs, outputs },
        };

        self.acir_ir.push_intrinsic(black_box_func_call);

        Ok(outputs_var)
    }

    /// Checks that the number of inputs being used to call the blackbox function
    /// is correct according to the function definition.
    ///
    /// Some functions expect a variable number of inputs and in such a case,
    /// this method will do nothing.  An example of this is sha256.
    /// In that case, this function will not check anything.
    ///
    /// Since we expect black box functions to be called behind a Noir shim function,
    /// we trigger a compiler error if the inputs do not match.
    ///
    /// An example of Noir shim function is the following:
    /// ```
    /// #[foreign(sha256)]
    /// fn sha256<N>(_input : [u8; N]) -> [u8; 32] {}
    /// ```
    fn intrinsics_check_inputs(&mut self, name: BlackBoxFunc, inputs: &[AcirVar]) {
        let expected_num_inputs = match Self::black_box_func_expected_input_size(name) {
            Some(expected_num_inputs) => expected_num_inputs,
            None => return,
        };
        let got_num_inputs = inputs.len();

        assert_eq!(expected_num_inputs,inputs.len(),"Tried to call black box function {name} with {got_num_inputs} inputs, but this function's definition requires {expected_num_inputs} inputs");
    }

    /// Intrinsic calls expect their inputs to be in a specific data structure (FunctionInput).
    ///
    /// This function will convert `AcirVar` into `FunctionInput` for an intrinsic call.
    fn prepare_inputs_for_intrinsics_call(
        &mut self,
        name: BlackBoxFunc,
        inputs: &[AcirVar],
    ) -> Result<Vec<FunctionInput>, AcirGenError> {
        self.intrinsics_check_inputs(name, inputs);

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
                    // TODO: First lets check to see how these are handled by the IR
                    //
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
                    // TODO(Jake): There were discussions about the SSA IR optimizing out range
                    // TODO constraints. We would want to be careful with it here. For example:
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

            witnesses.push(FunctionInput { witness, num_bits })
        }
        Ok(witnesses)
    }

    /// This function will return the number of inputs that a blackbox function
    /// expects. Returning `None` if there is no expectation.
    ///
    /// TODO(NOTE): We should not merge this PR, while this is here,
    /// TODO: the ideal way to do this will be to recognize that each blackbox function
    /// TODO is heterogenous and then check the input lengths in each functions context.
    fn black_box_func_expected_input_size(name: BlackBoxFunc) -> Option<usize> {
        match name {
            // Bitwise opcodes will take in 2 parameters
            BlackBoxFunc::AND | BlackBoxFunc::XOR => Some(2),
            // All of the hash/cipher methods will take in a
            // variable number of inputs.
            // Note: one can view `ComputeMerkleRoot` as a hash function.
            BlackBoxFunc::ComputeMerkleRoot
            | BlackBoxFunc::AES
            | BlackBoxFunc::Keccak256
            | BlackBoxFunc::SHA256
            | BlackBoxFunc::Blake2s
            | BlackBoxFunc::Pedersen
            | BlackBoxFunc::HashToField128Security => None,

            // Can only apply a range constraint to one
            // witness at a time.
            BlackBoxFunc::RANGE => Some(1),

            // Signature verification algorithms will take in a variable
            // number of inputs, since the message/hashed-message can vary in size.
            BlackBoxFunc::SchnorrVerify | BlackBoxFunc::EcdsaSecp256k1 => None,
            // Inputs for fixed based scalar multiplication
            // is just a scalar
            BlackBoxFunc::FixedBaseScalarMul => Some(1),
        }
    }
    /// This function will return the number of outputs that a blackbox function
    /// expects. Returning `None` if there is no expectation.
    ///
    /// TODO: With recursion, this becomes quite annoying, since that
    /// TODO: is backend specific
    fn black_box_expected_output_size(name: BlackBoxFunc) -> u32 {
        match name {
            // Bitwise opcodes will return 1 parameter which is the output
            // or the operation.
            BlackBoxFunc::AND | BlackBoxFunc::XOR => 1,
            // 32 byte hash algorithms
            BlackBoxFunc::Keccak256 | BlackBoxFunc::SHA256 | BlackBoxFunc::Blake2s => 32,
            // Hash to field returns a field element
            BlackBoxFunc::HashToField128Security => 1,
            // Pedersen returns a point
            BlackBoxFunc::Pedersen => 2,
            // A merkle root is represented as a single field element
            BlackBoxFunc::ComputeMerkleRoot => 1,
            // The output of AES128 is 16 bytes
            BlackBoxFunc::AES => 1,
            // Can only apply a range constraint to one
            // witness at a time.
            BlackBoxFunc::RANGE => 0,
            // Signature verification algorithms will return a boolean
            BlackBoxFunc::SchnorrVerify | BlackBoxFunc::EcdsaSecp256k1 => 1,
            // Output of fixed based scalar mul over the embedded curve
            // will be 2 field elements representing the point.
            BlackBoxFunc::FixedBaseScalarMul => 2,
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
        assert_eq!(self.data.len(), self.data_reverse_map.len());

        let id = AcirVar(self.data.len());

        self.data.insert(id, data.clone());
        self.data_reverse_map.insert(data, id);

        id
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
