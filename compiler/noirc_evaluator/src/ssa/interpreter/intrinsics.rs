use acvm::{AcirField, BlackBoxFunctionSolver, BlackBoxResolutionError, FieldElement};
use bn254_blackbox_solver::derive_generators;
use iter_extended::{try_vecmap, vecmap};
use num_bigint::BigUint;

use crate::ssa::{
    interpreter::NumericValue,
    ir::{
        dfg,
        instruction::{Endian, Intrinsic},
        types::{NumericType, Type},
        value::ValueId,
    },
};

use super::{ArrayValue, IResult, IResults, InternalError, Interpreter, InterpreterError, Value};

impl Interpreter<'_> {
    pub(super) fn call_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        args: &[ValueId],
        results: &[ValueId],
    ) -> IResults {
        match intrinsic {
            Intrinsic::ArrayLen => {
                check_argument_count(args, 1, intrinsic)?;
                let array = self.lookup_array_or_slice(args[0], "call to array_len")?;
                let length = array.elements.borrow().len();
                Ok(vec![Value::Numeric(NumericValue::U32(length as u32))])
            }
            Intrinsic::ArrayAsStrUnchecked => {
                check_argument_count(args, 1, intrinsic)?;
                Ok(vec![self.lookup(args[0])?])
            }
            Intrinsic::AsSlice => {
                check_argument_count(args, 1, intrinsic)?;
                let array = self.lookup_array_or_slice(args[0], "call to as_slice")?;
                let length = array.elements.borrow().len();
                let length = Value::Numeric(NumericValue::U32(length as u32));

                let elements = array.elements.borrow().to_vec();
                let slice = Value::slice(elements, array.element_types.clone());
                Ok(vec![length, slice])
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
                    // or it can have more arguments in case fmtstring or some other non-string value is passed.
                    // For simplicity, we won't build the dynamic message here.
                    let message = if args.len() == 2 {
                        self.lookup_string(args[1], "static_assert")?
                    } else {
                        "static_assert failed".to_string()
                    };
                    Err(InterpreterError::StaticAssertFailed { condition: args[0], message })
                }
            }
            Intrinsic::SlicePushBack => self.slice_push_back(args),
            Intrinsic::SlicePushFront => self.slice_push_front(args),
            Intrinsic::SlicePopBack => self.slice_pop_back(args),
            Intrinsic::SlicePopFront => self.slice_pop_front(args),
            Intrinsic::SliceInsert => self.slice_insert(args),
            Intrinsic::SliceRemove => self.slice_remove(args),
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
                    let result = result.iter().map(|v| (*v as u128).into());
                    let result = Value::array_from_iter(result, NumericType::NativeField)?;
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
                    let result = result.iter().map(|e| (*e as u128).into());
                    let result = Value::array_from_iter(result, NumericType::NativeField)?;
                    Ok(vec![result])
                }
                acvm::acir::BlackBoxFunc::Blake3 => {
                    check_argument_count(args, 1, intrinsic)?;
                    let inputs = self.lookup_bytes(args[0], "call Blake3 BlackBox")?;
                    let results =
                        acvm::blackbox_solver::blake3(&inputs).map_err(Self::convert_error)?;
                    let results = results.iter().map(|e| (*e as u128).into());
                    let results = Value::array_from_iter(results, NumericType::NativeField)?;
                    Ok(vec![results])
                }
                acvm::acir::BlackBoxFunc::EcdsaSecp256k1 => {
                    check_argument_count(args, 4, intrinsic)?;
                    let x = self.lookup_bytes(args[0], "call EcdsaSecp256k1 BlackBox")?;
                    let y = self.lookup_bytes(args[1], "call EcdsaSecp256k1 BlackBox")?;
                    let s = self.lookup_bytes(args[2], "call EcdsaSecp256k1 BlackBox")?;
                    let m = self.lookup_bytes(args[3], "call EcdsaSecp256k1 BlackBox")?;
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
                    let result = acvm::blackbox_solver::ecdsa_secp256k1_verify(
                        &m, x_array, y_array, s_array,
                    )
                    .map_err(Self::convert_error)?;
                    Ok(vec![Value::from_constant(
                        result.into(),
                        NumericType::Unsigned { bit_size: 1 },
                    )?])
                }
                acvm::acir::BlackBoxFunc::EcdsaSecp256r1 => {
                    check_argument_count(args, 4, intrinsic)?;
                    let x = self.lookup_bytes(args[0], "call EcdsaSecp256r1 BlackBox")?;
                    let y = self.lookup_bytes(args[1], "call EcdsaSecp256r1 BlackBox")?;
                    let s = self.lookup_bytes(args[2], "call EcdsaSecp256r1 BlackBox")?;
                    let m = self.lookup_bytes(args[3], "call EcdsaSecp256r1 BlackBox")?;
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
                    let result = acvm::blackbox_solver::ecdsa_secp256r1_verify(
                        &m, x_array, y_array, s_array,
                    )
                    .map_err(Self::convert_error)?;
                    Ok(vec![Value::from_constant(
                        result.into(),
                        NumericType::Unsigned { bit_size: 1 },
                    )?])
                }
                acvm::acir::BlackBoxFunc::MultiScalarMul => {
                    check_argument_count(args, 2, intrinsic)?;
                    let input_points =
                        self.lookup_array_or_slice(args[0], "call to MultiScalarMul blackbox")?;
                    let mut points = Vec::new();
                    for (i, v) in input_points.elements.borrow().iter().enumerate() {
                        if i % 3 == 2 {
                            points.push((v.as_bool().ok_or(
                                InterpreterError::Internal(InternalError::TypeError {
                                    value_id: args[0],
                                    value: v.to_string(),
                                    expected_type: "bool",
                                    instruction: "retrieving is_infinite in call to MultiScalarMul blackbox",
                                })
                            )? as u128).into());
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
                        self.lookup_array_or_slice(args[1], "call to MultiScalarMul blackbox")?;
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
                    let solver = bn254_blackbox_solver::Bn254BlackBoxSolver(false);
                    let result = solver.multi_scalar_mul(&points, &scalars_lo, &scalars_hi);
                    let (a, b, c) = result.map_err(Self::convert_error)?;
                    let result = Value::array_from_iter([a, b, c], NumericType::NativeField)?;
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
                    let results = results.iter().map(|e| (*e as u128).into());
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
                    check_argument_count(args, 6, intrinsic)?;
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
                    let result =
                        solver.ec_add(&lhs.0, &lhs.1, &lhs.2.into(), &rhs.0, &rhs.1, &rhs.2.into());
                    let (x, y, inf) = result.map_err(Self::convert_error)?;
                    let result = Value::array_from_iter([x, y, inf], NumericType::NativeField)?;
                    Ok(vec![result])
                }
                acvm::acir::BlackBoxFunc::BigIntAdd
                | acvm::acir::BlackBoxFunc::BigIntSub
                | acvm::acir::BlackBoxFunc::BigIntMul
                | acvm::acir::BlackBoxFunc::BigIntDiv
                | acvm::acir::BlackBoxFunc::BigIntFromLeBytes
                | acvm::acir::BlackBoxFunc::BigIntToLeBytes => {
                    Err(InterpreterError::Internal(InternalError::UnexpectedInstruction {
                        reason: "unused BigInt BlackBox function",
                    }))
                }
                acvm::acir::BlackBoxFunc::Poseidon2Permutation => {
                    check_argument_count(args, 2, intrinsic)?;
                    let inputs = self
                        .lookup_vec_field(args[0], "call Poseidon2Permutation BlackBox (inputs)")?;
                    let length =
                        self.lookup_u32(args[1], "call Poseidon2Permutation BlackBox (length)")?;
                    let solver = bn254_blackbox_solver::Bn254BlackBoxSolver(false);
                    let result = solver
                        .poseidon2_permutation(&inputs, length)
                        .map_err(Self::convert_error)?;
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
                    let result = state.iter().map(|e| (*e as u128).into());
                    let result = Value::array_from_iter(result, NumericType::NativeField)?;
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
                let generators = derive_generators(&inputs, inputs.len() as u32, index);
                let mut result = Vec::with_capacity(inputs.len());
                for generator in generators.iter() {
                    let x_big: BigUint = generator.x.into();
                    let x = FieldElement::from_le_bytes_reduce(&x_big.to_bytes_le());
                    let y_big: BigUint = generator.y.into();
                    let y = FieldElement::from_le_bytes_reduce(&y_big.to_bytes_le());
                    let generator_slice = Value::array_from_iter(
                        [x, y, generator.infinity.into()],
                        NumericType::NativeField,
                    )?;
                    result.push(generator_slice);
                }
                let results = Value::array(
                    result,
                    vec![Type::Array(
                        std::sync::Arc::new(vec![
                            Type::Numeric(NumericType::NativeField),
                            Type::Numeric(NumericType::NativeField),
                            Type::Numeric(NumericType::NativeField),
                        ]),
                        3,
                    )],
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
            Intrinsic::ArrayRefCount | Intrinsic::SliceRefCount => {
                let array = self.lookup_array_or_slice(args[0], "array/slice ref count")?;
                let rc = *array.rc.borrow();
                Ok(vec![Value::from_constant(rc.into(), NumericType::unsigned(32))?])
            }
        }
    }

    fn convert_error(err: BlackBoxResolutionError) -> InterpreterError {
        let BlackBoxResolutionError::Failed(name, reason) = err;
        InterpreterError::BlackBoxError { name: name.to_string(), reason }
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

    /// (length, slice, elem...) -> (length, slice)
    fn slice_push_back(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_push_back")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_push_back")?;

        // The resulting slice should be cloned - should we check RC here to try mutating it?
        // It'd need to be brillig-only if so since RC is always 1 in acir.
        let mut new_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        for arg in args.iter().skip(2) {
            new_elements.push(self.lookup(*arg)?);
        }

        let new_length = Value::Numeric(NumericValue::U32(length + 1));
        let new_slice = Value::slice(new_elements, element_types);
        Ok(vec![new_length, new_slice])
    }

    /// (length, slice, elem...) -> (length, slice)
    fn slice_push_front(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_push_front")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_push_front")?;
        let slice_elements = slice.elements.clone();
        let element_types = slice.element_types.clone();

        let mut new_elements = try_vecmap(args.iter().skip(2), |arg| self.lookup(*arg))?;
        new_elements.extend_from_slice(&slice_elements.borrow());

        let new_length = Value::Numeric(NumericValue::U32(length + 1));
        let new_slice = Value::slice(new_elements, element_types);
        Ok(vec![new_length, new_slice])
    }

    /// (length, slice) -> (length, slice, elem...)
    fn slice_pop_back(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_pop_back")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_pop_back")?;

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        if slice_elements.is_empty() {
            let instruction = "slice_pop_back";
            return Err(InterpreterError::PoppedFromEmptySlice { slice: args[1], instruction });
        }
        check_slice_can_pop_all_element_types(args[1], &slice)?;

        let mut popped_elements = vecmap(0..element_types.len(), |_| slice_elements.pop().unwrap());
        popped_elements.reverse();

        let new_length = Value::Numeric(NumericValue::U32(length - 1));
        let new_slice = Value::slice(slice_elements, element_types);
        let mut results = vec![new_length, new_slice];
        results.extend(popped_elements);
        Ok(results)
    }

    /// (length, slice) -> (elem..., length, slice)
    fn slice_pop_front(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_pop_front")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_pop_front")?;

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        if slice_elements.is_empty() {
            let instruction = "slice_pop_front";
            return Err(InterpreterError::PoppedFromEmptySlice { slice: args[1], instruction });
        }
        check_slice_can_pop_all_element_types(args[1], &slice)?;

        let mut results = slice_elements.drain(0..element_types.len()).collect::<Vec<_>>();

        let new_length = Value::Numeric(NumericValue::U32(length - 1));
        let new_slice = Value::slice(slice_elements, element_types);
        results.push(new_length);
        results.push(new_slice);
        Ok(results)
    }

    /// (length, slice, index:u32, elem...) -> (length, slice)
    fn slice_insert(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_insert")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_insert")?;
        let index = self.lookup_u32(args[2], "call to slice_insert")?;

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        let mut index = index as usize * element_types.len();
        for arg in args.iter().skip(3) {
            slice_elements.insert(index, self.lookup(*arg)?);
            index += 1;
        }

        let new_length = Value::Numeric(NumericValue::U32(length + 1));
        let new_slice = Value::slice(slice_elements, element_types);
        Ok(vec![new_length, new_slice])
    }

    /// (length, slice, index:u32) -> (length, slice, elem...)
    fn slice_remove(&self, args: &[ValueId]) -> IResults {
        let length = self.lookup_u32(args[0], "call to slice_remove")?;
        let slice = self.lookup_array_or_slice(args[1], "call to slice_remove")?;
        let index = self.lookup_u32(args[2], "call to slice_remove")?;

        let mut slice_elements = slice.elements.borrow().to_vec();
        let element_types = slice.element_types.clone();

        if slice_elements.is_empty() {
            let instruction = "slice_remove";
            return Err(InterpreterError::PoppedFromEmptySlice { slice: args[1], instruction });
        }
        check_slice_can_pop_all_element_types(args[1], &slice)?;

        let index = index as usize * element_types.len();
        let removed: Vec<_> = slice_elements.drain(index..index + element_types.len()).collect();

        let new_length = Value::Numeric(NumericValue::U32(length - 1));
        let new_slice = Value::slice(slice_elements, element_types);
        let mut results = vec![new_length, new_slice];
        results.extend(removed);
        Ok(results)
    }

    /// Print is not an intrinsic but it is treated like one.
    pub(super) fn call_print(&mut self, _args: Vec<Value>) -> IResults {
        // Stub the call for now
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

fn check_slice_can_pop_all_element_types(slice_id: ValueId, slice: &ArrayValue) -> IResult<()> {
    let actual_length = slice.elements.borrow().len();
    if actual_length >= slice.element_types.len() {
        Ok(())
    } else {
        Err(InterpreterError::Internal(InternalError::NotEnoughElementsToPopSliceOfStructs {
            slice_id,
            slice: slice.to_string(),
            actual_length,
            element_types: vecmap(slice.element_types.iter(), ToString::to_string),
        }))
    }
}
