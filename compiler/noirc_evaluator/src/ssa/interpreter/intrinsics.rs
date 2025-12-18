use std::{hash::BuildHasher, io::Write};

use acvm::{AcirField, BlackBoxFunctionSolver, BlackBoxResolutionError, FieldElement};
use bn254_blackbox_solver::derive_generators;
use iter_extended::{try_vecmap, vecmap};
use noirc_printable_type::{PrintableType, PrintableValueDisplay, decode_printable_value};
use num_bigint::BigUint;

use crate::ssa::ir::{
    dfg,
    instruction::{Endian, Intrinsic},
    types::{NumericType, Type},
    value::ValueId,
};

use super::{ArrayValue, IResult, IResults, InternalError, Interpreter, InterpreterError, Value};

impl<W: Write> Interpreter<'_, W> {
    pub(super) fn call_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        args: &[ValueId],
        results: &[ValueId],
    ) -> IResults {
        match intrinsic {
            Intrinsic::ArrayLen => {
                check_argument_count(args, 1, intrinsic)?;
                let array = self.lookup_array_or_vector(args[0], "call to array_len")?;
                let length = array.elements.borrow().len();
                Ok(vec![Value::u32(length as u32)])
            }
            Intrinsic::ArrayAsStrUnchecked => {
                check_argument_count(args, 1, intrinsic)?;
                Ok(vec![self.lookup(args[0])?])
            }
            Intrinsic::AsVector => {
                check_argument_count(args, 1, intrinsic)?;
                let array = self.lookup_array_or_vector(args[0], "call to as_vector")?;
                let length = array.elements.borrow().len();
                let length = Value::u32(length as u32);

                let elements = array.elements.borrow().to_vec();
                let vector = Value::vector(elements, array.element_types.clone());
                Ok(vec![length, vector])
            }
            Intrinsic::AssertConstant => {
                // Nothing we can do here unfortunately if we want to allow code with
                // assert_constant to still pass pre-inlining and other optimizations.
                Ok(Vec::new())
            }
            Intrinsic::StaticAssert => {
                check_argument_count_is_at_least(args, 2, intrinsic)?;

                let condition = self.lookup_bool(args[0], "static_assert")?;
                if condition {
                    Ok(Vec::new())
                } else {
                    // Static assert can either have 2 arguments, in which case the second one is a string,
                    // or it can have more arguments in case fmtstr or some other non-string value is passed.
                    // For simplicity, we won't build the dynamic message here.
                    let message = if args.len() == 2 {
                        self.lookup_string(args[1], "static_assert")?
                    } else {
                        "static_assert failed".to_string()
                    };
                    Err(InterpreterError::StaticAssertFailed { condition: args[0], message })
                }
            }
            Intrinsic::VectorPushBack => self.vector_push_back(args),
            Intrinsic::VectorPushFront => self.vector_push_front(args),
            Intrinsic::VectorPopBack => self.vector_pop_back(args),
            Intrinsic::VectorPopFront => self.vector_pop_front(args),
            Intrinsic::VectorInsert => self.vector_insert(args),
            Intrinsic::VectorRemove => self.vector_remove(args),
            Intrinsic::ApplyRangeConstraint => {
                Err(InterpreterError::Internal(InternalError::UnexpectedInstruction {
                    reason: "Intrinsic::ApplyRangeConstraint should have been converted to a RangeCheck instruction",
                }))
            }
            Intrinsic::StrAsBytes => {
                // This one is a no-op
                check_argument_count(args, 1, intrinsic)?;
                Ok(vec![self.lookup(args[0])?])
            }
            Intrinsic::AsWitness => {
                // This one is also a no-op, but it doesn't return anything
                check_argument_count(args, 1, intrinsic)?;
                Ok(vec![])
            }
            Intrinsic::ToBits(endian) => {
                check_argument_count(args, 1, intrinsic)?;
                let field = self.lookup_field(args[0], "call to to_bits")?;
                let element_type = NumericType::bool();
                self.to_radix(endian, element_type, args[0], field, 2, results[0])
            }
            Intrinsic::ToRadix(endian) => {
                check_argument_count(args, 2, intrinsic)?;
                let field = self.lookup_field(args[0], "call to to_radix")?;
                let radix = self.lookup_u32(args[1], "call to to_radix")?;
                let element_type = NumericType::Unsigned { bit_size: 8 };
                self.to_radix(endian, element_type, args[0], field, radix, results[0])
            }
            Intrinsic::BlackBox(black_box_func) => match black_box_func {
                acvm::acir::BlackBoxFunc::AES128Encrypt => {
                    check_argument_count(args, 3, intrinsic)?;
                    let inputs = self.lookup_bytes(args[0], "call AES128Encrypt BlackBox")?;
                    let iv = self.lookup_bytes(args[1], "call AES128Encrypt BlackBox")?;
                    let key = self.lookup_bytes(args[2], "call AES128Encrypt BlackBox")?;
                    let iv_len = iv.len();
                    let iv_array: [u8; 16] = iv.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 16,
                            size: iv_len,
                        })
                    })?;
                    let key_len = key.len();
                    let key_array: [u8; 16] = key.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 16,
                            size: key_len,
                        })
                    })?;
                    let result =
                        acvm::blackbox_solver::aes128_encrypt(&inputs, iv_array, key_array)
                            .map_err(Self::convert_error)?;
                    let result = result.iter().map(|v| u128::from(*v).into());
                    let result = Value::array_from_iter(result, NumericType::unsigned(8))?;
                    Ok(vec![result])
                }
                acvm::acir::BlackBoxFunc::AND => {
                    Err(InterpreterError::Internal(InternalError::UnexpectedInstruction {
                        reason: "AND instruction should have already been evaluated",
                    }))
                }
                acvm::acir::BlackBoxFunc::XOR => {
                    Err(InterpreterError::Internal(InternalError::UnexpectedInstruction {
                        reason: "XOR instruction should have already been evaluated",
                    }))
                }
                acvm::acir::BlackBoxFunc::RANGE => {
                    Err(InterpreterError::Internal(InternalError::UnexpectedInstruction {
                        reason: "RANGE instruction should have already been evaluated",
                    }))
                }
                acvm::acir::BlackBoxFunc::Blake2s => {
                    check_argument_count(args, 1, intrinsic)?;
                    let inputs = self.lookup_bytes(args[0], "call Blake2s BlackBox")?;
                    let result =
                        acvm::blackbox_solver::blake2s(&inputs).map_err(Self::convert_error)?;
                    let result = result.iter().map(|e| u128::from(*e).into());
                    let result = Value::array_from_iter(result, NumericType::unsigned(8))?;
                    Ok(vec![result])
                }
                acvm::acir::BlackBoxFunc::Blake3 => {
                    check_argument_count(args, 1, intrinsic)?;
                    let inputs = self.lookup_bytes(args[0], "call Blake3 BlackBox")?;
                    let results =
                        acvm::blackbox_solver::blake3(&inputs).map_err(Self::convert_error)?;
                    let results = results.iter().map(|e| u128::from(*e).into());
                    let results = Value::array_from_iter(results, NumericType::unsigned(8))?;
                    Ok(vec![results])
                }
                acvm::acir::BlackBoxFunc::EcdsaSecp256k1 => {
                    check_argument_count(args, 5, intrinsic)?;
                    let x = self.lookup_bytes(args[0], "call EcdsaSecp256k1 BlackBox")?;
                    let y = self.lookup_bytes(args[1], "call EcdsaSecp256k1 BlackBox")?;
                    let s = self.lookup_bytes(args[2], "call EcdsaSecp256k1 BlackBox")?;
                    let m = self.lookup_bytes(args[3], "call EcdsaSecp256k1 BlackBox")?;
                    let predicate = self.lookup_bool(args[4], "call EcdsaSecp256k1 BlackBox")?;
                    let x_len = x.len();
                    let x_array: &[u8; 32] = &x.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 32,
                            size: x_len,
                        })
                    })?;
                    let y_len = y.len();
                    let y_array: &[u8; 32] = &y.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 32,
                            size: y_len,
                        })
                    })?;
                    let s_len = s.len();
                    let s_array: &[u8; 64] = &s.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 64,
                            size: s_len,
                        })
                    })?;
                    let m_len = m.len();
                    let m_array: &[u8; 32] = &m.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 32,
                            size: m_len,
                        })
                    })?;
                    let result = if predicate {
                        acvm::blackbox_solver::ecdsa_secp256k1_verify(
                            m_array, x_array, y_array, s_array,
                        )
                        .map_err(Self::convert_error)?
                    } else {
                        true
                    };
                    Ok(vec![Value::from_constant(
                        result.into(),
                        NumericType::Unsigned { bit_size: 1 },
                    )?])
                }
                acvm::acir::BlackBoxFunc::EcdsaSecp256r1 => {
                    check_argument_count(args, 5, intrinsic)?;
                    let x = self.lookup_bytes(args[0], "call EcdsaSecp256r1 BlackBox")?;
                    let y = self.lookup_bytes(args[1], "call EcdsaSecp256r1 BlackBox")?;
                    let s = self.lookup_bytes(args[2], "call EcdsaSecp256r1 BlackBox")?;
                    let m = self.lookup_bytes(args[3], "call EcdsaSecp256r1 BlackBox")?;
                    let predicate = self.lookup_bool(args[4], "call EcdsaSecp256r1 BlackBox")?;
                    let x_len = x.len();
                    let x_array: &[u8; 32] = &x.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 32,
                            size: x_len,
                        })
                    })?;
                    let y_len = y.len();
                    let y_array: &[u8; 32] = &y.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 32,
                            size: y_len,
                        })
                    })?;
                    let s_len = s.len();
                    let s_array: &[u8; 64] = &s.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 64,
                            size: s_len,
                        })
                    })?;
                    let m_len = m.len();
                    let m_array: &[u8; 32] = &m.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 32,
                            size: m_len,
                        })
                    })?;

                    let result = if predicate {
                        acvm::blackbox_solver::ecdsa_secp256r1_verify(
                            m_array, x_array, y_array, s_array,
                        )
                        .map_err(Self::convert_error)?
                    } else {
                        true
                    };
                    Ok(vec![Value::from_constant(
                        result.into(),
                        NumericType::Unsigned { bit_size: 1 },
                    )?])
                }
                acvm::acir::BlackBoxFunc::MultiScalarMul => {
                    check_argument_count(args, 3, intrinsic)?;
                    let input_points =
                        self.lookup_array_or_vector(args[0], "call to MultiScalarMul blackbox")?;
                    let mut points = Vec::new();
                    for (i, v) in input_points.elements.borrow().iter().enumerate() {
                        if i % 3 == 2 {
                            points.push(u128::from(v.as_bool().ok_or(
                                InterpreterError::Internal(InternalError::TypeError {
                                    value_id: args[0],
                                    value: v.to_string(),
                                    expected_type: "bool",
                                    instruction: "retrieving is_infinite in call to MultiScalarMul blackbox",
                                })
                            )?).into());
                        } else {
                            points.push(
                            v.as_field().ok_or(
                                InterpreterError::Internal(InternalError::TypeError {
                                    value_id: args[0],
                                    value: v.to_string(),
                                    expected_type: "field",
                                    instruction: "retrieving ec points in call to MultiScalarMul blackbox",
                                })
                            )?);
                        }
                    }
                    let scalars =
                        self.lookup_array_or_vector(args[1], "call to MultiScalarMul blackbox")?;
                    let mut scalars_lo = Vec::new();
                    let mut scalars_hi = Vec::new();
                    for (i, v) in scalars.elements.borrow().iter().enumerate() {
                        if i % 2 == 0 {
                            scalars_lo.push(v.as_field().ok_or(
                                InterpreterError::Internal(InternalError::TypeError {
                                    value_id: args[0],
                                    value: v.to_string(),
                                    expected_type: "Field",
                                    instruction: "retrieving scalars in call to MultiScalarMul blackbox",
                                })
                            )?);
                        } else {
                            scalars_hi.push(v.as_field().ok_or(
                                InterpreterError::Internal(InternalError::TypeError {
                                    value_id: args[0],
                                    value: v.to_string(),
                                    expected_type: "Field",
                                    instruction: "retrieving scalars in call to MultiScalarMul blackbox",
                                })
                            )?);
                        }
                    }
                    let predicate = self.lookup_bool(
                        args[2],
                        "retrieving predicate in call to MultiScalarMul blackbox",
                    )?;

                    let solver = bn254_blackbox_solver::Bn254BlackBoxSolver(false);
                    let result =
                        solver.multi_scalar_mul(&points, &scalars_lo, &scalars_hi, predicate);
                    let (x, y, is_infinite) = result.map_err(Self::convert_error)?;
                    let result = new_embedded_curve_point(x, y, is_infinite)?;
                    Ok(vec![result])
                }
                acvm::acir::BlackBoxFunc::Keccakf1600 => {
                    check_argument_count(args, 1, intrinsic)?;
                    let inputs = self.lookup_vec_u64(args[0], "call to Keccakf1600 BlackBox")?;
                    let input_len = inputs.len();
                    let inputs_array: [u64; 25] = inputs.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 25,
                            size: input_len,
                        })
                    })?;
                    let results = acvm::blackbox_solver::keccakf1600(inputs_array)
                        .map_err(Self::convert_error)?;
                    let results = results.iter().map(|e| u128::from(*e).into());
                    let results =
                        Value::array_from_iter(results, NumericType::Unsigned { bit_size: 64 })?;
                    Ok(vec![results])
                }
                acvm::acir::BlackBoxFunc::RecursiveAggregation => {
                    // Recursive aggregation only updates the backend internal state
                    // from the SSA interpreter, it is a no-op.
                    Ok(vec![])
                }
                acvm::acir::BlackBoxFunc::EmbeddedCurveAdd => {
                    check_argument_count(args, 7, intrinsic)?;
                    let solver = bn254_blackbox_solver::Bn254BlackBoxSolver(false);
                    let lhs = (
                        self.lookup_field(args[0], "call EmbeddedCurveAdd BlackBox")?,
                        self.lookup_field(args[1], "call EmbeddedCurveAdd BlackBox")?,
                        self.lookup_bool(args[2], "call EmbeddedCurveAdd BlackBox")?,
                    );
                    let rhs = (
                        self.lookup_field(args[3], "call EmbeddedCurveAdd BlackBox")?,
                        self.lookup_field(args[4], "call EmbeddedCurveAdd BlackBox")?,
                        self.lookup_bool(args[5], "call EmbeddedCurveAdd BlackBox")?,
                    );
                    let predicate = self.lookup_bool(args[6], "call EmbeddedCurveAdd BlackBox")?;
                    let result = solver.ec_add(
                        &lhs.0,
                        &lhs.1,
                        &lhs.2.into(),
                        &rhs.0,
                        &rhs.1,
                        &rhs.2.into(),
                        predicate,
                    );
                    let (x, y, is_infinite) = result.map_err(Self::convert_error)?;
                    let result = new_embedded_curve_point(x, y, is_infinite)?;
                    Ok(vec![result])
                }

                acvm::acir::BlackBoxFunc::Poseidon2Permutation => {
                    check_argument_count(args, 1, intrinsic)?;
                    let inputs = self
                        .lookup_vec_field(args[0], "call Poseidon2Permutation BlackBox (inputs)")?;
                    let solver = bn254_blackbox_solver::Bn254BlackBoxSolver(false);
                    let result =
                        solver.poseidon2_permutation(&inputs).map_err(Self::convert_error)?;
                    let result = Value::array_from_iter(result, NumericType::NativeField)?;
                    Ok(vec![result])
                }
                acvm::acir::BlackBoxFunc::Sha256Compression => {
                    check_argument_count(args, 2, intrinsic)?;
                    let inputs = self.lookup_vec_u32(args[0], "call Sha256Compression BlackBox")?;
                    let state = self.lookup_vec_u32(args[1], "call Sha256Compression BlackBox")?;
                    let input_len = inputs.len();
                    let inputs: [u32; 16] = inputs.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 16,
                            size: input_len,
                        })
                    })?;
                    let state_len = state.len();
                    let mut state: [u32; 8] = state.try_into().map_err(|_| {
                        InterpreterError::Internal(InternalError::InvalidInputSize {
                            expected_size: 16,
                            size: state_len,
                        })
                    })?;
                    acvm::blackbox_solver::sha256_compression(&mut state, &inputs);
                    let result = state.iter().map(|e| u128::from(*e).into());
                    let result = Value::array_from_iter(result, NumericType::unsigned(32))?;
                    Ok(vec![result])
                }
            },
            Intrinsic::Hint(_) => self.lookup_all(args),
            Intrinsic::IsUnconstrained => {
                check_argument_count(args, 0, intrinsic)?;
                Ok(vec![Value::bool(self.in_unconstrained_context())])
            }
            Intrinsic::DerivePedersenGenerators => {
                check_argument_count(args, 2, intrinsic)?;

                let inputs =
                    self.lookup_bytes(args[0], "call DerivePedersenGenerators BlackBox")?;
                let index = self.lookup_u32(args[1], "call DerivePedersenGenerators BlackBox")?;

                // The definition is:
                //
                // ```noir
                // fn __derive_generators<let N: u32, let M: u32>(
                //     domain_separator_bytes: [u8; M],
                //     starting_index: u32,
                // ) -> [EmbeddedCurvePoint; N] {}
                // ```
                //
                // We need to get N from the return type.
                if results.len() != 1 {
                    return Err(InterpreterError::Internal(
                        InternalError::UnexpectedResultLength {
                            actual_length: results.len(),
                            expected_length: 1,
                            instruction: "call DerivePedersenGenerators BlackBox",
                        },
                    ));
                }

                let result_type = self.dfg().type_of_value(results[0]);
                let Type::Array(_, n) = result_type else {
                    return Err(InterpreterError::Internal(InternalError::UnexpectedResultType {
                        actual_type: result_type.to_string(),
                        expected_type: "array",
                        instruction: "call DerivePedersenGenerators BlackBox",
                    }));
                };

                let generators = derive_generators(&inputs, n, index);
                let mut result = Vec::with_capacity(inputs.len());
                for generator in generators.iter() {
                    let x_big: BigUint = generator.x.into();
                    let x = FieldElement::from_le_bytes_reduce(&x_big.to_bytes_le());
                    let y_big: BigUint = generator.y.into();
                    let y = FieldElement::from_le_bytes_reduce(&y_big.to_bytes_le());
                    result.push(Value::from_constant(x, NumericType::NativeField)?);
                    result.push(Value::from_constant(y, NumericType::NativeField)?);
                    result.push(Value::from_constant(
                        generator.infinity.into(),
                        NumericType::bool(),
                    )?);
                }
                let results = Value::array(
                    result,
                    vec![
                        Type::Numeric(NumericType::NativeField),
                        Type::Numeric(NumericType::NativeField),
                        Type::Numeric(NumericType::bool()),
                    ],
                );
                Ok(vec![results])
            }
            Intrinsic::FieldLessThan => {
                if !self.in_unconstrained_context() {
                    return Err(InterpreterError::Internal(
                        InternalError::FieldLessThanCalledInConstrainedContext,
                    ));
                }
                check_argument_count(args, 2, intrinsic)?;
                let lhs = self.lookup_field(args[0], "lhs of call to field less than")?;
                let rhs = self.lookup_field(args[1], "rhs of call to field less than")?;
                Ok(vec![Value::bool(lhs < rhs)])
            }
            Intrinsic::ArrayRefCount | Intrinsic::VectorRefCount => {
                // `vector_refcount` receives `[length, array]` as input. `array_refcount` gets just `[array]`
                let idx = if matches!(intrinsic, Intrinsic::VectorRefCount) { 1 } else { 0 };
                let array = self.lookup_array_or_vector(args[idx], "array/vector ref count")?;
                let mut rc = *array.rc.borrow();
                // ACIR always returns 0 for the refcounts, and we expect that IncRc and DecRc don't appear in constrained SSA.
                // The interpreter starts with a default ref-count value of 1. If it did not change, treat it as zero to match ACIR.
                if !self.in_unconstrained_context() && rc == 1 {
                    rc = 0;
                }
                Ok(vec![Value::from_constant(rc.into(), NumericType::unsigned(32))?])
            }
        }
    }

    fn convert_error(err: BlackBoxResolutionError) -> InterpreterError {
        let (name, reason) = match err {
            BlackBoxResolutionError::Failed(name, reason) => (name.to_string(), reason),
            BlackBoxResolutionError::AssertFailed(err) => ("Assertion failed".to_string(), err),
        };
        InterpreterError::BlackBoxError { name, reason }
    }

    fn to_radix(
        &self,
        endian: Endian,
        element_type: NumericType,
        field_id: ValueId,
        field: FieldElement,
        radix: u32,
        result: ValueId,
    ) -> IResults {
        let result_type = self.dfg().type_of_value(result);
        let Type::Array(_, limb_count) = result_type else {
            return Err(InterpreterError::Internal(InternalError::TypeError {
                value_id: result,
                value: result_type.to_string(),
                expected_type: "array",
                instruction: "call to to_radix",
            }));
        };

        let Some(limbs) = dfg::simplify::constant_to_radix(endian, field, radix, limb_count) else {
            return Err(InterpreterError::ToRadixFailed { field_id, field, radix });
        };

        let elements = try_vecmap(limbs, |limb| Value::from_constant(limb, element_type))?;
        Ok(vec![Value::array(elements, vec![Type::Numeric(element_type)])])
    }

    /// (length, vector, elem...) -> (length, vector)
    fn vector_push_back(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to vector_push_back")?;
        let vector = self.lookup_array_or_vector(args[1], "call to vector_push_back")?;

        // The resulting vector should be cloned - should we check RC here to try mutating it?
        // It'd need to be brillig-only if so since RC is always 1 in acir.
        let mut new_elements = vector.elements.borrow().to_vec();
        let element_types = vector.element_types.clone();

        // The vector might contain more elements than its length.
        // We need to either insert before the extras, overwrite, or remove them.
        new_elements.truncate(element_types.len() * length as usize);
        for arg in args.iter().skip(2) {
            new_elements.push(self.lookup(*arg)?);
        }

        let new_length = Value::u32(length + 1);
        let new_vector = Value::vector(new_elements, element_types);
        Ok(vec![new_length, new_vector])
    }

    /// (length, vector, elem...) -> (length, vector)
    fn vector_push_front(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to vector_push_front")?;
        let vector = self.lookup_array_or_vector(args[1], "call to vector_push_front")?;
        let vector_elements = vector.elements.clone();
        let element_types = vector.element_types.clone();

        let mut new_elements = try_vecmap(args.iter().skip(2), |arg| self.lookup(*arg))?;
        new_elements.extend_from_slice(&vector_elements.borrow());

        let new_length = Value::u32(length + 1);
        let new_vector = Value::vector(new_elements, element_types);
        Ok(vec![new_length, new_vector])
    }

    /// (length, vector) -> (length, vector, elem...)
    fn vector_pop_back(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to vector_pop_back")?;
        let vector = self.lookup_array_or_vector(args[1], "call to vector_pop_back")?;

        let mut vector_elements = vector.elements.borrow().to_vec();
        let element_types = vector.element_types.clone();

        if vector_elements.is_empty() || length == 0 {
            let instruction = "vector_pop_back";
            return Err(InterpreterError::PoppedFromEmptyVector { vector: args[1], instruction });
        }
        check_vector_can_pop_all_element_types(args[1], &vector)?;

        // The vector might contain more elements than its length.
        // We want the last valid element, ignoring any extras following it.
        // We don't ever access the extras, so we might as well remove any.
        vector_elements.truncate(element_types.len() * length as usize);
        let mut popped_elements = vecmap(0..element_types.len(), |_| vector_elements.pop().unwrap());
        popped_elements.reverse();

        let new_length = Value::u32(length - 1);
        let new_vector = Value::vector(vector_elements, element_types);
        let mut results = vec![new_length, new_vector];
        results.extend(popped_elements);
        Ok(results)
    }

    /// (length, vector) -> (elem..., length, vector)
    fn vector_pop_front(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to vector_pop_front")?;
        let vector = self.lookup_array_or_vector(args[1], "call to vector_pop_front")?;

        let mut vector_elements = vector.elements.borrow().to_vec();
        let element_types = vector.element_types.clone();

        if vector_elements.is_empty() || length == 0 {
            let instruction = "vector_pop_front";
            return Err(InterpreterError::PoppedFromEmptyVector { vector: args[1], instruction });
        }
        check_vector_can_pop_all_element_types(args[1], &vector)?;

        let mut results = vector_elements.drain(0..element_types.len()).collect::<Vec<_>>();

        let new_length = Value::u32(length - 1);
        let new_vector = Value::vector(vector_elements, element_types);
        results.push(new_length);
        results.push(new_vector);
        Ok(results)
    }

    /// (length, vector, index:u32, elem...) -> (length, vector)
    fn vector_insert(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to vector_insert")?;
        let vector = self.lookup_array_or_vector(args[1], "call to vector_insert")?;
        let index = self.lookup_u32(args[2], "call to vector_insert")?;

        let mut vector_elements = vector.elements.borrow().to_vec();
        let element_types = vector.element_types.clone();

        let mut index = index as usize * element_types.len();
        for arg in args.iter().skip(3) {
            vector_elements.insert(index, self.lookup(*arg)?);
            index += 1;
        }

        let new_length = Value::u32(length + 1);
        let new_vector = Value::vector(vector_elements, element_types);
        Ok(vec![new_length, new_vector])
    }

    /// (length, vector, index:u32) -> (length, vector, elem...)
    fn vector_remove(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to vector_remove")?;
        let vector = self.lookup_array_or_vector(args[1], "call to vector_remove")?;
        let index = self.lookup_u32(args[2], "call to vector_remove")?;

        let mut vector_elements = vector.elements.borrow().to_vec();
        let element_types = vector.element_types.clone();

        if vector_elements.is_empty() {
            let instruction = "vector_remove";
            return Err(InterpreterError::PoppedFromEmptyVector { vector: args[1], instruction });
        }
        check_vector_can_pop_all_element_types(args[1], &vector)?;

        let index = index as usize * element_types.len();
        let removed: Vec<_> = vector_elements.drain(index..index + element_types.len()).collect();

        let new_length = Value::u32(length - 1);
        let new_vector = Value::vector(vector_elements, element_types);
        let mut results = vec![new_length, new_vector];
        results.extend(removed);
        Ok(results)
    }

    /// Print is not an intrinsic but it is treated like one.
    pub(super) fn call_print(&mut self, args: Vec<Value>) -> IResults {
        fn get_arg<F, T>(
            args: &[Value],
            idx: usize,
            name: &'static str,
            typ: &'static str,
            f: F,
        ) -> IResult<T>
        where
            F: FnOnce(&Value) -> Option<T>,
        {
            let arg = &args[idx];
            if let Some(v) = f(arg) {
                Ok(v)
            } else {
                Err(InterpreterError::Internal(InternalError::UnexpectedInput {
                    name,
                    expected_type: typ,
                    value: arg.to_string(),
                }))
            }
        }

        let invalid_input_size = |expected_size| {
            Err(InterpreterError::Internal(InternalError::InvalidInputSize {
                expected_size,
                size: args.len(),
            }))
        };

        // We expect at least 4 arguments (tuples are passed as multiple values):
        // * normal: newline, value.0, ..., value.i, meta, false
        // * formatted: newline, msg, N, value1.0, ..., value1.i, ..., valueN.0, ..., valueN.j, meta1, ..., metaN, true
        if args.len() < 4 {
            return invalid_input_size(4);
        }

        let print_newline = get_arg(&args, 0, "print_newline", "bool", |arg| arg.as_bool())?;
        let is_fmt_str = get_arg(&args, args.len() - 1, "is_fmt_str", "bool", |arg| arg.as_bool())?;

        let printable_display = if is_fmt_str {
            let message = value_to_string("message", &args[1])?;
            let num_values =
                get_arg(&args, 2, "num_values", "Field", |arg| arg.as_field())?.to_u128() as usize;

            // We expect at least 4 + num_values * 2 values, because each fragment will have 1 type descriptor, and at least 1 value.
            let min_args = 4 + 2 * num_values;
            if args.len() < min_args {
                return invalid_input_size(min_args);
            }

            // Everything up to the first meta is part of _some_ value.
            // We'll let each parser take as many fields as they need.
            let meta_idx = args.len() - 1 - num_values;
            let input_as_fields = values_to_fields(&args[3..meta_idx]);
            let field_iterator = &mut input_as_fields.into_iter();

            let mut fragments = Vec::new();
            for i in 0..num_values {
                let printable_type = value_to_printable_type(&args[meta_idx + i])?;
                let printable_value = decode_printable_value(field_iterator, &printable_type);
                fragments.push((printable_value, printable_type));
            }
            PrintableValueDisplay::FmtString(message, fragments)
        } else {
            let meta_idx = args.len() - 2;
            let input_as_fields = values_to_fields(&args[1..meta_idx]);
            let printable_type = value_to_printable_type(&args[meta_idx])?;
            let printable_value =
                decode_printable_value(&mut input_as_fields.into_iter(), &printable_type);
            PrintableValueDisplay::Plain(printable_value, printable_type)
        };

        if print_newline {
            writeln!(self.output, "{printable_display}").expect("writeln");
        } else {
            write!(self.output, "{printable_display}").expect("write");
        }

        Ok(Vec::new())
    }
}

fn check_argument_count(
    args: &[ValueId],
    expected_count: usize,
    intrinsic: Intrinsic,
) -> IResult<()> {
    if args.len() != expected_count {
        Err(InterpreterError::Internal(InternalError::IntrinsicArgumentCountMismatch {
            intrinsic,
            arguments: args.len(),
            parameters: expected_count,
        }))
    } else {
        Ok(())
    }
}

fn check_argument_count_is_at_least(
    args: &[ValueId],
    expected_count: usize,
    intrinsic: Intrinsic,
) -> IResult<()> {
    if args.len() < expected_count {
        Err(InterpreterError::Internal(InternalError::IntrinsicMinArgumentCountMismatch {
            intrinsic,
            arguments: args.len(),
            parameters: expected_count,
        }))
    } else {
        Ok(())
    }
}

fn check_vector_can_pop_all_element_types(vector_id: ValueId, vector: &ArrayValue) -> IResult<()> {
    let actual_length = vector.elements.borrow().len();
    if actual_length >= vector.element_types.len() {
        Ok(())
    } else {
        Err(InterpreterError::Internal(InternalError::NotEnoughElementsToPopVectorOfStructs {
            vector_id,
            vector: vector.to_string(),
            actual_length,
            element_types: vecmap(vector.element_types.iter(), ToString::to_string),
        }))
    }
}

fn new_embedded_curve_point(
    x: FieldElement,
    y: FieldElement,
    is_infinite: FieldElement,
) -> IResult<Value> {
    let x = Value::from_constant(x, NumericType::NativeField)?;
    let y = Value::from_constant(y, NumericType::NativeField)?;
    let is_infinite = Value::from_constant(is_infinite, NumericType::bool())?;
    Ok(Value::array(
        vec![x, y, is_infinite],
        vec![
            Type::Numeric(NumericType::NativeField),
            Type::Numeric(NumericType::NativeField),
            Type::Numeric(NumericType::bool()),
        ],
    ))
}

/// Convert a vector of [Value] to a flattened vector of [FieldElement] for printing.
///
/// It takes a vector, rather than individual values, so that it can try to
/// pair up `u32` fields indicating the size of a `Vector` with its elements
/// following in the next value, and truncate the data to the semantic length.
fn values_to_fields(values: &[Value]) -> Vec<FieldElement> {
    fn go<'a, I>(values: I, fields: &mut Vec<FieldElement>)
    where
        I: Iterator<Item = &'a Value>,
    {
        let mut vector_length: Option<usize> = None;
        for value in values {
            match value {
                Value::Numeric(numeric_value) => fields.push(numeric_value.convert_to_field()),
                Value::Reference(reference_value) => {
                    if let Some(value) = reference_value.element.borrow().as_ref() {
                        go(std::iter::once(value), fields);
                    }
                }
                Value::ArrayOrVector(array_value) => {
                    let length = match vector_length {
                        Some(length) if array_value.is_vector => {
                            length * array_value.element_types.len()
                        }
                        _ => array_value.elements.borrow().len(),
                    };
                    go(array_value.elements.borrow().iter().take(length), fields);
                }
                Value::Function(id) => {
                    // Based on `decode_printable_value` it will expect consume the environment as well,
                    // but that's catered for the by the SSA generation: the env is passed as separate values.
                    fields.push(FieldElement::from(id.to_u32()));
                }
                Value::ForeignFunction(x) => {
                    // The actual display of functions only shows the type, but expects the ID.
                    // Send a hash so we can interpret the Initial SSA until we wrap these values with a normal function.
                    let hash = rustc_hash::FxBuildHasher.hash_one(x);
                    fields.push(FieldElement::from(hash));
                }
                Value::Intrinsic(x) => {
                    // Same as foreign functions: just pass something so we can handle the initial SSA even if somehow an intrinsic makes it here.
                    let hash = rustc_hash::FxBuildHasher.hash_one(x);
                    fields.push(FieldElement::from(hash));
                }
            }
            // Chamber the length for a potential vector following it.
            if let Some(length) = value.as_u32() {
                vector_length = Some(length as usize);
            } else {
                vector_length = None;
            }
        }
    }
    let mut fields = Vec::new();
    go(values.iter(), &mut fields);
    fields
}

/// Parse a [Value] as [PrintableType].
fn value_to_printable_type(value: &Value) -> IResult<PrintableType> {
    let name = "type_metadata";
    let json = value_to_string(name, value)?;
    let printable_type = serde_json::from_str::<PrintableType>(&json).map_err(|e| {
        InterpreterError::Internal(InternalError::ParsingError {
            name,
            expected_type: "PrintableType",
            value: json,
            error: e.to_string(),
        })
    })?;
    Ok(printable_type)
}

/// Parse a value as `[u8]` and convert to UTF-8 `String`.
fn value_to_string(name: &'static str, value: &Value) -> IResult<String> {
    let arr = value.as_array_or_vector().and_then(|arr| {
        arr.elements.borrow().iter().map(|v| v.as_u8()).collect::<Option<Vec<_>>>()
    });
    let Some(bz) = arr else {
        return Err(InterpreterError::Internal(InternalError::UnexpectedInput {
            name,
            expected_type: "[u8]",
            value: value.to_string(),
        }));
    };
    let Some(s) = String::from_utf8(bz).ok() else {
        return Err(InterpreterError::Internal(InternalError::UnexpectedInput {
            name,
            expected_type: "String",
            value: value.to_string(),
        }));
    };
    Ok(s)
}
