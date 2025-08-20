use std::{borrow::Cow, sync::Arc};

use acvm::{FieldElement, acir::AcirField};

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{
            ArrayOffset, Binary, BinaryOp, ConstrainError, Endian, Instruction, Intrinsic,
        },
        types::{NumericType, Type},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

use super::simple_optimization::SimpleOptimizationContext;

impl Ssa {
    /// Replaces Shl and Shr instructions with more primitive arithmetic instructions
    /// since our backend doesn't directly support bit shifts.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_bit_shifts(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.remove_bit_shifts();
        }
        self
    }
}

impl Function {
    /// Go through every instruction, replacing bit shifts with more primitive arithmetic
    /// operations.
    pub(crate) fn remove_bit_shifts(&mut self) {
        if self.runtime().is_brillig() {
            return;
        }

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            let Instruction::Binary(Binary { lhs, rhs, operator }) = instruction else {
                return;
            };

            if !matches!(operator, BinaryOp::Shl | BinaryOp::Shr) {
                return;
            }

            let lhs = *lhs;
            let rhs = *rhs;
            let operator = *operator;

            context.remove_current_instruction();

            let old_result = *context.dfg.instruction_results(instruction_id).first().unwrap();

            let mut bitshift_context = Context { context };
            let new_result = if operator == BinaryOp::Shl {
                bitshift_context.insert_wrapping_shift_left(lhs, rhs)
            } else {
                bitshift_context.insert_shift_right(lhs, rhs)
            };

            context.replace_value(old_result, new_result);
        });

        #[cfg(debug_assertions)]
        remove_bit_shifts_post_check(self);
    }
}

struct Context<'m, 'dfg, 'mapping> {
    context: &'m mut SimpleOptimizationContext<'dfg, 'mapping>,
}

impl Context<'_, '_, '_> {
    /// Insert ssa instructions which computes lhs << rhs by doing lhs*2^rhs
    /// and truncate the result to bit_size
    pub(crate) fn insert_wrapping_shift_left(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let typ = self.context.dfg.type_of_value(lhs).unwrap_numeric();
        let max_lhs_bits = self.context.dfg.get_value_max_num_bits(lhs);
        let max_bit_shift_size = self.context.dfg.get_numeric_constant(rhs).map_or_else(
            || {
                // If we don't know `rhs`'s value then it could be anything up to the number
                // of bits in the type, e.g. u32 means shifting by up to 32 bits as otherwise we get overflow.
                self.context.dfg.get_value_max_num_bits(rhs)
            },
            |rhs_constant| {
                // Happy case is that we know precisely by how many bits we're shifting by.
                rhs_constant.to_u128() as u32
            },
        );

        let pow = self.two_pow(rhs);

        // We cap the maximum number of bits here to ensure that we don't try and truncate using a
        // `max_bit_size` greater than what's allowable by the underlying `FieldElement` as this is meaningless.
        //
        // If `max_lhs_bits + max_bit_shift_size` were ever to exceed `FieldElement::max_num_bits()`,
        // then the constraint on `rhs` in `self.two_pow` should be broken.
        let max_bit = std::cmp::min(
            max_lhs_bits.checked_add(max_bit_shift_size).unwrap_or(FieldElement::max_num_bits()),
            FieldElement::max_num_bits(),
        );
        if max_bit <= typ.bit_size() {
            let pow = self.insert_cast(pow, typ);
            // Unchecked mul as it can't overflow
            self.insert_binary(lhs, BinaryOp::Mul { unchecked: true }, pow)
        } else {
            let lhs_field = self.insert_cast(lhs, NumericType::NativeField);
            // Unchecked mul as this is a wrapping operation that we later truncate
            let result = self.insert_binary(lhs_field, BinaryOp::Mul { unchecked: true }, pow);
            let result = self.insert_truncate(result, typ.bit_size(), max_bit);
            self.insert_cast(result, typ)
        }
    }

    /// Insert ssa instructions which computes lhs >> rhs by doing lhs/2^rhs
    /// For negative signed integers, we do the division on the 1-complement representation of lhs,
    /// before converting back the result to the 2-complement representation.
    pub(crate) fn insert_shift_right(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let lhs_typ = self.context.dfg.type_of_value(lhs).unwrap_numeric();

        let pow = self.two_pow(rhs);
        let pow = self.insert_cast(pow, lhs_typ);

        match lhs_typ {
            NumericType::Unsigned { .. } => {
                // unsigned right bit shift is just a normal division
                self.insert_binary(lhs, BinaryOp::Div, pow)
            }
            NumericType::Signed { bit_size } => {
                // Get the sign of the operand; positive signed operand will just do a division as well
                let zero =
                    self.numeric_constant(FieldElement::zero(), NumericType::signed(bit_size));
                let lhs_sign = self.insert_binary(lhs, BinaryOp::Lt, zero);
                let lhs_sign_as_field = self.insert_cast(lhs_sign, NumericType::NativeField);
                let lhs_as_field = self.insert_cast(lhs, NumericType::NativeField);
                // For negative numbers, convert to 1-complement using wrapping addition of a + 1
                // Unchecked add as these are fields
                let one_complement = self.insert_binary(
                    lhs_sign_as_field,
                    BinaryOp::Add { unchecked: true },
                    lhs_as_field,
                );
                let one_complement = self.insert_truncate(one_complement, bit_size, bit_size + 1);
                let one_complement =
                    self.insert_cast(one_complement, NumericType::signed(bit_size));
                // Performs the division on the 1-complement (or the operand if positive)
                let shifted_complement = self.insert_binary(one_complement, BinaryOp::Div, pow);
                // Convert back to 2-complement representation if operand is negative
                let lhs_sign_as_int = self.insert_cast(lhs_sign, lhs_typ);

                // The requirements for this to underflow are all of these:
                // - lhs < 0
                // - ones_complement(lhs) / (2^rhs) == 0
                // As the upper bit is set for the ones complement of negative numbers we'd need 2^rhs
                // to be larger than the lhs bitsize for this to overflow.
                let shifted = self.insert_binary(
                    shifted_complement,
                    BinaryOp::Sub { unchecked: true },
                    lhs_sign_as_int,
                );
                self.insert_truncate(shifted, bit_size, bit_size + 1)
            }

            NumericType::NativeField => unreachable!("Bit shifts are disallowed on `Field` type"),
        }
    }

    /// Computes 2^exponent via square&multiply, using the bits decomposition of exponent
    /// Pseudo-code of the computation:
    /// let mut r = 1;
    /// let exponent_bits = to_bits(exponent);
    /// for i in 1 .. bit_size + 1 {
    ///     let r_squared = r * r;
    ///     let b = exponent_bits[bit_size - i];
    ///     r = if b { 2 * r_squared } else { r_squared };
    /// }
    fn two_pow(&mut self, exponent: ValueId) -> ValueId {
        // Require that exponent < bit_size, ensuring that `pow` returns a value consistent with `lhs`'s type.
        self.enforce_bitshift_rhs_lt_bit_size(exponent);

        if let Some(exponent_const) = self.context.dfg.get_numeric_constant(exponent) {
            let pow = FieldElement::from(2u32).pow(&exponent_const);
            return self.numeric_constant(pow, NumericType::NativeField);
        }

        let to_bits = self.context.dfg.import_intrinsic(Intrinsic::ToBits(Endian::Little));
        let max_exponent_bits = self.context.dfg.type_of_value(exponent).bit_size();
        let result_types = vec![Type::Array(Arc::new(vec![Type::bool()]), max_exponent_bits)];

        // A call to ToBits can only be done with a field argument (exponent is always u8 here)
        let exponent_as_field = self.insert_cast(exponent, NumericType::NativeField);
        let exponent_bits = self.insert_call(to_bits, vec![exponent_as_field], result_types);

        let exponent_bits = exponent_bits[0];
        let one = self.field_constant(FieldElement::one());
        let two = self.field_constant(FieldElement::from(2u32));
        let mut r = one;
        // All operations are unchecked as we're acting on Field types (which are always unchecked)
        for i in 1..max_exponent_bits + 1 {
            let idx = self.numeric_constant(
                FieldElement::from((max_exponent_bits - i) as i128),
                NumericType::length_type(),
            );
            let b = self.insert_array_get(exponent_bits, idx, Type::bool());
            let not_b = self.insert_not(b);
            let b = self.insert_cast(b, NumericType::NativeField);
            let not_b = self.insert_cast(not_b, NumericType::NativeField);

            let r_squared = self.insert_binary(r, BinaryOp::Mul { unchecked: true }, r);
            let r1 = self.insert_binary(r_squared, BinaryOp::Mul { unchecked: true }, not_b);
            let a = self.insert_binary(r_squared, BinaryOp::Mul { unchecked: true }, two);
            let r2 = self.insert_binary(a, BinaryOp::Mul { unchecked: true }, b);
            r = self.insert_binary(r1, BinaryOp::Add { unchecked: true }, r2);
        }

        assert!(
            matches!(self.context.dfg.type_of_value(r).unwrap_numeric(), NumericType::NativeField),
            "ICE: pow is expected to always return a NativeField"
        );

        r
    }

    /// Insert constraints ensuring that the right-hand side of a bit-shift operation
    /// is less than the bit size of the left-hand side.
    fn enforce_bitshift_rhs_lt_bit_size(&mut self, rhs: ValueId) {
        let one = self.numeric_constant(FieldElement::one(), NumericType::bool());
        let rhs_type = self.context.dfg.type_of_value(rhs);

        let assert_message = Some("attempt to bit-shift with overflow".to_owned());

        let (bit_size, bit_size_field) = match rhs_type {
            Type::Numeric(NumericType::Unsigned { bit_size }) => {
                (bit_size, FieldElement::from(bit_size))
            }
            Type::Numeric(NumericType::Signed { bit_size }) => {
                assert!(bit_size > 1, "ICE - i1 is not a valid type");

                (bit_size, FieldElement::from(bit_size - 1))
            }
            _ => unreachable!("check_shift_overflow called with non-numeric type"),
        };

        let unsigned_typ = NumericType::unsigned(bit_size);
        let max = self.numeric_constant(bit_size_field, unsigned_typ);
        let rhs = self.insert_cast(rhs, unsigned_typ);
        let overflow = self.insert_binary(rhs, BinaryOp::Lt, max);
        self.insert_constrain(overflow, one, assert_message.map(Into::into));
    }

    pub(crate) fn field_constant(&mut self, constant: FieldElement) -> ValueId {
        self.context.dfg.make_constant(constant, NumericType::NativeField)
    }

    /// Insert a numeric constant into the current function
    pub(crate) fn numeric_constant(
        &mut self,
        value: impl Into<FieldElement>,
        typ: NumericType,
    ) -> ValueId {
        self.context.dfg.make_constant(value.into(), typ)
    }

    /// Insert a binary instruction at the end of the current block.
    /// Returns the result of the binary instruction.
    pub(crate) fn insert_binary(
        &mut self,
        lhs: ValueId,
        operator: BinaryOp,
        rhs: ValueId,
    ) -> ValueId {
        let instruction = Instruction::Binary(Binary { lhs, rhs, operator });
        self.context.insert_instruction(instruction, None).first()
    }

    /// Insert a not instruction at the end of the current block.
    /// Returns the result of the instruction.
    pub(crate) fn insert_not(&mut self, rhs: ValueId) -> ValueId {
        self.context.insert_instruction(Instruction::Not(rhs), None).first()
    }

    /// Insert a constrain instruction at the end of the current block.
    fn insert_constrain(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        assert_message: Option<ConstrainError>,
    ) {
        self.context.insert_instruction(Instruction::Constrain(lhs, rhs, assert_message), None);
    }

    /// Insert a truncate instruction at the end of the current block.
    /// Returns the result of the truncate instruction.
    pub(crate) fn insert_truncate(
        &mut self,
        value: ValueId,
        bit_size: u32,
        max_bit_size: u32,
    ) -> ValueId {
        self.context
            .insert_instruction(Instruction::Truncate { value, bit_size, max_bit_size }, None)
            .first()
    }

    /// Insert a cast instruction at the end of the current block.
    /// Returns the result of the cast instruction.
    pub(crate) fn insert_cast(&mut self, value: ValueId, typ: NumericType) -> ValueId {
        self.context.insert_instruction(Instruction::Cast(value, typ), None).first()
    }

    /// Insert a call instruction at the end of the current block and return
    /// the results of the call.
    pub(crate) fn insert_call(
        &mut self,
        func: ValueId,
        arguments: Vec<ValueId>,
        result_types: Vec<Type>,
    ) -> Cow<[ValueId]> {
        self.context
            .insert_instruction(Instruction::Call { func, arguments }, Some(result_types))
            .results()
    }

    /// Insert an instruction to extract an element from an array
    pub(crate) fn insert_array_get(
        &mut self,
        array: ValueId,
        index: ValueId,
        element_type: Type,
    ) -> ValueId {
        let element_type = Some(vec![element_type]);
        let offset = ArrayOffset::None;
        let instruction = Instruction::ArrayGet { array, index, offset };
        self.context.insert_instruction(instruction, element_type).first()
    }
}

/// Post-check condition for [Function::remove_bit_shifts].
///
/// Succeeds if:
///   - `func` is not an ACIR function, OR
///   - `func` does not contain any bitshift instructions.
///
/// Otherwise panics.
#[cfg(debug_assertions)]
fn remove_bit_shifts_post_check(func: &Function) {
    // Non-ACIR functions should be unaffected.
    if !func.runtime().is_acir() {
        return;
    }

    // Otherwise there should be no shift-left or shift-right instructions in any reachable block.
    for block_id in func.reachable_blocks() {
        let instruction_ids = func.dfg[block_id].instructions();
        for instruction_id in instruction_ids {
            if matches!(
                func.dfg[*instruction_id],
                Instruction::Binary(Binary { operator: BinaryOp::Shl | BinaryOp::Shr, .. })
            ) {
                panic!("Bitshift instruction still remains in ACIR function");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    mod unsigned {
        use super::*;

        #[test]
        fn removes_shl_with_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u32):
                v2 = shl v0, u32 2
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v1 = cast v0 as Field
            v3 = mul v1, Field 4
            v4 = truncate v3 to 32 bits, max_bit_size: 34
            v5 = cast v4 as u32
            return v5
        }
        ");
        }

        #[test]
        fn removes_shl_with_non_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u32, v1: u32):
                v2 = shl v0, v1
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();

            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: u32, v1: u32):
                v3 = lt v1, u32 32
                constrain v3 == u1 1, "attempt to bit-shift with overflow"
                v5 = cast v1 as Field
                v7 = call to_le_bits(v5) -> [u1; 32]
                v9 = array_get v7, index u32 31 -> u1
                v10 = not v9
                v11 = cast v9 as Field
                v12 = cast v10 as Field
                v14 = mul Field 2, v11
                v15 = add v12, v14
                v17 = array_get v7, index u32 30 -> u1
                v18 = not v17
                v19 = cast v17 as Field
                v20 = cast v18 as Field
                v21 = mul v15, v15
                v22 = mul v21, v20
                v23 = mul v21, Field 2
                v24 = mul v23, v19
                v25 = add v22, v24
                v27 = array_get v7, index u32 29 -> u1
                v28 = not v27
                v29 = cast v27 as Field
                v30 = cast v28 as Field
                v31 = mul v25, v25
                v32 = mul v31, v30
                v33 = mul v31, Field 2
                v34 = mul v33, v29
                v35 = add v32, v34
                v37 = array_get v7, index u32 28 -> u1
                v38 = not v37
                v39 = cast v37 as Field
                v40 = cast v38 as Field
                v41 = mul v35, v35
                v42 = mul v41, v40
                v43 = mul v41, Field 2
                v44 = mul v43, v39
                v45 = add v42, v44
                v47 = array_get v7, index u32 27 -> u1
                v48 = not v47
                v49 = cast v47 as Field
                v50 = cast v48 as Field
                v51 = mul v45, v45
                v52 = mul v51, v50
                v53 = mul v51, Field 2
                v54 = mul v53, v49
                v55 = add v52, v54
                v57 = array_get v7, index u32 26 -> u1
                v58 = not v57
                v59 = cast v57 as Field
                v60 = cast v58 as Field
                v61 = mul v55, v55
                v62 = mul v61, v60
                v63 = mul v61, Field 2
                v64 = mul v63, v59
                v65 = add v62, v64
                v67 = array_get v7, index u32 25 -> u1
                v68 = not v67
                v69 = cast v67 as Field
                v70 = cast v68 as Field
                v71 = mul v65, v65
                v72 = mul v71, v70
                v73 = mul v71, Field 2
                v74 = mul v73, v69
                v75 = add v72, v74
                v77 = array_get v7, index u32 24 -> u1
                v78 = not v77
                v79 = cast v77 as Field
                v80 = cast v78 as Field
                v81 = mul v75, v75
                v82 = mul v81, v80
                v83 = mul v81, Field 2
                v84 = mul v83, v79
                v85 = add v82, v84
                v87 = array_get v7, index u32 23 -> u1
                v88 = not v87
                v89 = cast v87 as Field
                v90 = cast v88 as Field
                v91 = mul v85, v85
                v92 = mul v91, v90
                v93 = mul v91, Field 2
                v94 = mul v93, v89
                v95 = add v92, v94
                v97 = array_get v7, index u32 22 -> u1
                v98 = not v97
                v99 = cast v97 as Field
                v100 = cast v98 as Field
                v101 = mul v95, v95
                v102 = mul v101, v100
                v103 = mul v101, Field 2
                v104 = mul v103, v99
                v105 = add v102, v104
                v107 = array_get v7, index u32 21 -> u1
                v108 = not v107
                v109 = cast v107 as Field
                v110 = cast v108 as Field
                v111 = mul v105, v105
                v112 = mul v111, v110
                v113 = mul v111, Field 2
                v114 = mul v113, v109
                v115 = add v112, v114
                v117 = array_get v7, index u32 20 -> u1
                v118 = not v117
                v119 = cast v117 as Field
                v120 = cast v118 as Field
                v121 = mul v115, v115
                v122 = mul v121, v120
                v123 = mul v121, Field 2
                v124 = mul v123, v119
                v125 = add v122, v124
                v127 = array_get v7, index u32 19 -> u1
                v128 = not v127
                v129 = cast v127 as Field
                v130 = cast v128 as Field
                v131 = mul v125, v125
                v132 = mul v131, v130
                v133 = mul v131, Field 2
                v134 = mul v133, v129
                v135 = add v132, v134
                v137 = array_get v7, index u32 18 -> u1
                v138 = not v137
                v139 = cast v137 as Field
                v140 = cast v138 as Field
                v141 = mul v135, v135
                v142 = mul v141, v140
                v143 = mul v141, Field 2
                v144 = mul v143, v139
                v145 = add v142, v144
                v147 = array_get v7, index u32 17 -> u1
                v148 = not v147
                v149 = cast v147 as Field
                v150 = cast v148 as Field
                v151 = mul v145, v145
                v152 = mul v151, v150
                v153 = mul v151, Field 2
                v154 = mul v153, v149
                v155 = add v152, v154
                v157 = array_get v7, index u32 16 -> u1
                v158 = not v157
                v159 = cast v157 as Field
                v160 = cast v158 as Field
                v161 = mul v155, v155
                v162 = mul v161, v160
                v163 = mul v161, Field 2
                v164 = mul v163, v159
                v165 = add v162, v164
                v167 = array_get v7, index u32 15 -> u1
                v168 = not v167
                v169 = cast v167 as Field
                v170 = cast v168 as Field
                v171 = mul v165, v165
                v172 = mul v171, v170
                v173 = mul v171, Field 2
                v174 = mul v173, v169
                v175 = add v172, v174
                v177 = array_get v7, index u32 14 -> u1
                v178 = not v177
                v179 = cast v177 as Field
                v180 = cast v178 as Field
                v181 = mul v175, v175
                v182 = mul v181, v180
                v183 = mul v181, Field 2
                v184 = mul v183, v179
                v185 = add v182, v184
                v187 = array_get v7, index u32 13 -> u1
                v188 = not v187
                v189 = cast v187 as Field
                v190 = cast v188 as Field
                v191 = mul v185, v185
                v192 = mul v191, v190
                v193 = mul v191, Field 2
                v194 = mul v193, v189
                v195 = add v192, v194
                v197 = array_get v7, index u32 12 -> u1
                v198 = not v197
                v199 = cast v197 as Field
                v200 = cast v198 as Field
                v201 = mul v195, v195
                v202 = mul v201, v200
                v203 = mul v201, Field 2
                v204 = mul v203, v199
                v205 = add v202, v204
                v207 = array_get v7, index u32 11 -> u1
                v208 = not v207
                v209 = cast v207 as Field
                v210 = cast v208 as Field
                v211 = mul v205, v205
                v212 = mul v211, v210
                v213 = mul v211, Field 2
                v214 = mul v213, v209
                v215 = add v212, v214
                v217 = array_get v7, index u32 10 -> u1
                v218 = not v217
                v219 = cast v217 as Field
                v220 = cast v218 as Field
                v221 = mul v215, v215
                v222 = mul v221, v220
                v223 = mul v221, Field 2
                v224 = mul v223, v219
                v225 = add v222, v224
                v227 = array_get v7, index u32 9 -> u1
                v228 = not v227
                v229 = cast v227 as Field
                v230 = cast v228 as Field
                v231 = mul v225, v225
                v232 = mul v231, v230
                v233 = mul v231, Field 2
                v234 = mul v233, v229
                v235 = add v232, v234
                v237 = array_get v7, index u32 8 -> u1
                v238 = not v237
                v239 = cast v237 as Field
                v240 = cast v238 as Field
                v241 = mul v235, v235
                v242 = mul v241, v240
                v243 = mul v241, Field 2
                v244 = mul v243, v239
                v245 = add v242, v244
                v247 = array_get v7, index u32 7 -> u1
                v248 = not v247
                v249 = cast v247 as Field
                v250 = cast v248 as Field
                v251 = mul v245, v245
                v252 = mul v251, v250
                v253 = mul v251, Field 2
                v254 = mul v253, v249
                v255 = add v252, v254
                v257 = array_get v7, index u32 6 -> u1
                v258 = not v257
                v259 = cast v257 as Field
                v260 = cast v258 as Field
                v261 = mul v255, v255
                v262 = mul v261, v260
                v263 = mul v261, Field 2
                v264 = mul v263, v259
                v265 = add v262, v264
                v267 = array_get v7, index u32 5 -> u1
                v268 = not v267
                v269 = cast v267 as Field
                v270 = cast v268 as Field
                v271 = mul v265, v265
                v272 = mul v271, v270
                v273 = mul v271, Field 2
                v274 = mul v273, v269
                v275 = add v272, v274
                v277 = array_get v7, index u32 4 -> u1
                v278 = not v277
                v279 = cast v277 as Field
                v280 = cast v278 as Field
                v281 = mul v275, v275
                v282 = mul v281, v280
                v283 = mul v281, Field 2
                v284 = mul v283, v279
                v285 = add v282, v284
                v287 = array_get v7, index u32 3 -> u1
                v288 = not v287
                v289 = cast v287 as Field
                v290 = cast v288 as Field
                v291 = mul v285, v285
                v292 = mul v291, v290
                v293 = mul v291, Field 2
                v294 = mul v293, v289
                v295 = add v292, v294
                v297 = array_get v7, index u32 2 -> u1
                v298 = not v297
                v299 = cast v297 as Field
                v300 = cast v298 as Field
                v301 = mul v295, v295
                v302 = mul v301, v300
                v303 = mul v301, Field 2
                v304 = mul v303, v299
                v305 = add v302, v304
                v307 = array_get v7, index u32 1 -> u1
                v308 = not v307
                v309 = cast v307 as Field
                v310 = cast v308 as Field
                v311 = mul v305, v305
                v312 = mul v311, v310
                v313 = mul v311, Field 2
                v314 = mul v313, v309
                v315 = add v312, v314
                v317 = array_get v7, index u32 0 -> u1
                v318 = not v317
                v319 = cast v317 as Field
                v320 = cast v318 as Field
                v321 = mul v315, v315
                v322 = mul v321, v320
                v323 = mul v321, Field 2
                v324 = mul v323, v319
                v325 = add v322, v324
                v326 = cast v0 as Field
                v327 = mul v326, v325
                v328 = truncate v327 to 32 bits, max_bit_size: 64
                v329 = cast v328 as u32
                return v329
            }
            "#);
        }

        #[test]
        fn does_not_generate_invalid_truncation_on_overflowing_bitshift() {
            // We want to ensure that the `max_bit_size` of the truncation does not exceed the field size.
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u32):
                v2 = shl v0, u32 255
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: u32):
                constrain u1 0 == u1 1, "attempt to bit-shift with overflow"
                v3 = cast v0 as Field
                v5 = mul v3, Field -7768683996859727954953724731427871339010100868427821011365820555770860666883
                v6 = truncate v5 to 32 bits, max_bit_size: 254
                v7 = cast v6 as u32
                return v7
            }
            "#);
        }

        #[test]
        fn removes_shr_with_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u32):
                v2 = shr v0, u32 2
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @r"
            acir(inline) fn main f0 {
              b0(v0: u32):
                v2 = div v0, u32 4
                return v2
            }
            ");
        }

        #[test]
        fn removes_shr_with_non_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u32, v1: u32):
                v2 = shr v0, v1
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();

            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: u32, v1: u32):
                v3 = lt v1, u32 32
                constrain v3 == u1 1, "attempt to bit-shift with overflow"
                v5 = cast v1 as Field
                v7 = call to_le_bits(v5) -> [u1; 32]
                v9 = array_get v7, index u32 31 -> u1
                v10 = not v9
                v11 = cast v9 as Field
                v12 = cast v10 as Field
                v14 = mul Field 2, v11
                v15 = add v12, v14
                v17 = array_get v7, index u32 30 -> u1
                v18 = not v17
                v19 = cast v17 as Field
                v20 = cast v18 as Field
                v21 = mul v15, v15
                v22 = mul v21, v20
                v23 = mul v21, Field 2
                v24 = mul v23, v19
                v25 = add v22, v24
                v27 = array_get v7, index u32 29 -> u1
                v28 = not v27
                v29 = cast v27 as Field
                v30 = cast v28 as Field
                v31 = mul v25, v25
                v32 = mul v31, v30
                v33 = mul v31, Field 2
                v34 = mul v33, v29
                v35 = add v32, v34
                v37 = array_get v7, index u32 28 -> u1
                v38 = not v37
                v39 = cast v37 as Field
                v40 = cast v38 as Field
                v41 = mul v35, v35
                v42 = mul v41, v40
                v43 = mul v41, Field 2
                v44 = mul v43, v39
                v45 = add v42, v44
                v47 = array_get v7, index u32 27 -> u1
                v48 = not v47
                v49 = cast v47 as Field
                v50 = cast v48 as Field
                v51 = mul v45, v45
                v52 = mul v51, v50
                v53 = mul v51, Field 2
                v54 = mul v53, v49
                v55 = add v52, v54
                v57 = array_get v7, index u32 26 -> u1
                v58 = not v57
                v59 = cast v57 as Field
                v60 = cast v58 as Field
                v61 = mul v55, v55
                v62 = mul v61, v60
                v63 = mul v61, Field 2
                v64 = mul v63, v59
                v65 = add v62, v64
                v67 = array_get v7, index u32 25 -> u1
                v68 = not v67
                v69 = cast v67 as Field
                v70 = cast v68 as Field
                v71 = mul v65, v65
                v72 = mul v71, v70
                v73 = mul v71, Field 2
                v74 = mul v73, v69
                v75 = add v72, v74
                v77 = array_get v7, index u32 24 -> u1
                v78 = not v77
                v79 = cast v77 as Field
                v80 = cast v78 as Field
                v81 = mul v75, v75
                v82 = mul v81, v80
                v83 = mul v81, Field 2
                v84 = mul v83, v79
                v85 = add v82, v84
                v87 = array_get v7, index u32 23 -> u1
                v88 = not v87
                v89 = cast v87 as Field
                v90 = cast v88 as Field
                v91 = mul v85, v85
                v92 = mul v91, v90
                v93 = mul v91, Field 2
                v94 = mul v93, v89
                v95 = add v92, v94
                v97 = array_get v7, index u32 22 -> u1
                v98 = not v97
                v99 = cast v97 as Field
                v100 = cast v98 as Field
                v101 = mul v95, v95
                v102 = mul v101, v100
                v103 = mul v101, Field 2
                v104 = mul v103, v99
                v105 = add v102, v104
                v107 = array_get v7, index u32 21 -> u1
                v108 = not v107
                v109 = cast v107 as Field
                v110 = cast v108 as Field
                v111 = mul v105, v105
                v112 = mul v111, v110
                v113 = mul v111, Field 2
                v114 = mul v113, v109
                v115 = add v112, v114
                v117 = array_get v7, index u32 20 -> u1
                v118 = not v117
                v119 = cast v117 as Field
                v120 = cast v118 as Field
                v121 = mul v115, v115
                v122 = mul v121, v120
                v123 = mul v121, Field 2
                v124 = mul v123, v119
                v125 = add v122, v124
                v127 = array_get v7, index u32 19 -> u1
                v128 = not v127
                v129 = cast v127 as Field
                v130 = cast v128 as Field
                v131 = mul v125, v125
                v132 = mul v131, v130
                v133 = mul v131, Field 2
                v134 = mul v133, v129
                v135 = add v132, v134
                v137 = array_get v7, index u32 18 -> u1
                v138 = not v137
                v139 = cast v137 as Field
                v140 = cast v138 as Field
                v141 = mul v135, v135
                v142 = mul v141, v140
                v143 = mul v141, Field 2
                v144 = mul v143, v139
                v145 = add v142, v144
                v147 = array_get v7, index u32 17 -> u1
                v148 = not v147
                v149 = cast v147 as Field
                v150 = cast v148 as Field
                v151 = mul v145, v145
                v152 = mul v151, v150
                v153 = mul v151, Field 2
                v154 = mul v153, v149
                v155 = add v152, v154
                v157 = array_get v7, index u32 16 -> u1
                v158 = not v157
                v159 = cast v157 as Field
                v160 = cast v158 as Field
                v161 = mul v155, v155
                v162 = mul v161, v160
                v163 = mul v161, Field 2
                v164 = mul v163, v159
                v165 = add v162, v164
                v167 = array_get v7, index u32 15 -> u1
                v168 = not v167
                v169 = cast v167 as Field
                v170 = cast v168 as Field
                v171 = mul v165, v165
                v172 = mul v171, v170
                v173 = mul v171, Field 2
                v174 = mul v173, v169
                v175 = add v172, v174
                v177 = array_get v7, index u32 14 -> u1
                v178 = not v177
                v179 = cast v177 as Field
                v180 = cast v178 as Field
                v181 = mul v175, v175
                v182 = mul v181, v180
                v183 = mul v181, Field 2
                v184 = mul v183, v179
                v185 = add v182, v184
                v187 = array_get v7, index u32 13 -> u1
                v188 = not v187
                v189 = cast v187 as Field
                v190 = cast v188 as Field
                v191 = mul v185, v185
                v192 = mul v191, v190
                v193 = mul v191, Field 2
                v194 = mul v193, v189
                v195 = add v192, v194
                v197 = array_get v7, index u32 12 -> u1
                v198 = not v197
                v199 = cast v197 as Field
                v200 = cast v198 as Field
                v201 = mul v195, v195
                v202 = mul v201, v200
                v203 = mul v201, Field 2
                v204 = mul v203, v199
                v205 = add v202, v204
                v207 = array_get v7, index u32 11 -> u1
                v208 = not v207
                v209 = cast v207 as Field
                v210 = cast v208 as Field
                v211 = mul v205, v205
                v212 = mul v211, v210
                v213 = mul v211, Field 2
                v214 = mul v213, v209
                v215 = add v212, v214
                v217 = array_get v7, index u32 10 -> u1
                v218 = not v217
                v219 = cast v217 as Field
                v220 = cast v218 as Field
                v221 = mul v215, v215
                v222 = mul v221, v220
                v223 = mul v221, Field 2
                v224 = mul v223, v219
                v225 = add v222, v224
                v227 = array_get v7, index u32 9 -> u1
                v228 = not v227
                v229 = cast v227 as Field
                v230 = cast v228 as Field
                v231 = mul v225, v225
                v232 = mul v231, v230
                v233 = mul v231, Field 2
                v234 = mul v233, v229
                v235 = add v232, v234
                v237 = array_get v7, index u32 8 -> u1
                v238 = not v237
                v239 = cast v237 as Field
                v240 = cast v238 as Field
                v241 = mul v235, v235
                v242 = mul v241, v240
                v243 = mul v241, Field 2
                v244 = mul v243, v239
                v245 = add v242, v244
                v247 = array_get v7, index u32 7 -> u1
                v248 = not v247
                v249 = cast v247 as Field
                v250 = cast v248 as Field
                v251 = mul v245, v245
                v252 = mul v251, v250
                v253 = mul v251, Field 2
                v254 = mul v253, v249
                v255 = add v252, v254
                v257 = array_get v7, index u32 6 -> u1
                v258 = not v257
                v259 = cast v257 as Field
                v260 = cast v258 as Field
                v261 = mul v255, v255
                v262 = mul v261, v260
                v263 = mul v261, Field 2
                v264 = mul v263, v259
                v265 = add v262, v264
                v267 = array_get v7, index u32 5 -> u1
                v268 = not v267
                v269 = cast v267 as Field
                v270 = cast v268 as Field
                v271 = mul v265, v265
                v272 = mul v271, v270
                v273 = mul v271, Field 2
                v274 = mul v273, v269
                v275 = add v272, v274
                v277 = array_get v7, index u32 4 -> u1
                v278 = not v277
                v279 = cast v277 as Field
                v280 = cast v278 as Field
                v281 = mul v275, v275
                v282 = mul v281, v280
                v283 = mul v281, Field 2
                v284 = mul v283, v279
                v285 = add v282, v284
                v287 = array_get v7, index u32 3 -> u1
                v288 = not v287
                v289 = cast v287 as Field
                v290 = cast v288 as Field
                v291 = mul v285, v285
                v292 = mul v291, v290
                v293 = mul v291, Field 2
                v294 = mul v293, v289
                v295 = add v292, v294
                v297 = array_get v7, index u32 2 -> u1
                v298 = not v297
                v299 = cast v297 as Field
                v300 = cast v298 as Field
                v301 = mul v295, v295
                v302 = mul v301, v300
                v303 = mul v301, Field 2
                v304 = mul v303, v299
                v305 = add v302, v304
                v307 = array_get v7, index u32 1 -> u1
                v308 = not v307
                v309 = cast v307 as Field
                v310 = cast v308 as Field
                v311 = mul v305, v305
                v312 = mul v311, v310
                v313 = mul v311, Field 2
                v314 = mul v313, v309
                v315 = add v312, v314
                v317 = array_get v7, index u32 0 -> u1
                v318 = not v317
                v319 = cast v317 as Field
                v320 = cast v318 as Field
                v321 = mul v315, v315
                v322 = mul v321, v320
                v323 = mul v321, Field 2
                v324 = mul v323, v319
                v325 = add v322, v324
                v326 = cast v325 as u32
                v327 = div v0, v326
                return v327
            }
            "#);
        }
    }

    mod signed {
        use super::*;
        #[test]
        fn removes_shl_with_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: i32):
                v2 = shl v0, i32 2
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: i32):
            v1 = cast v0 as Field
            v3 = mul v1, Field 4
            v4 = truncate v3 to 32 bits, max_bit_size: 34
            v5 = cast v4 as i32
            return v5
        }
        ");
        }

        #[test]
        fn removes_shl_with_non_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: i32, v1: i32):
                v2 = shl v0, v1
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();

            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: i32, v1: i32):
                v2 = cast v1 as u32
                v4 = lt v2, u32 31
                constrain v4 == u1 1, "attempt to bit-shift with overflow"
                v6 = cast v1 as Field
                v8 = call to_le_bits(v6) -> [u1; 32]
                v9 = array_get v8, index u32 31 -> u1
                v10 = not v9
                v11 = cast v9 as Field
                v12 = cast v10 as Field
                v14 = mul Field 2, v11
                v15 = add v12, v14
                v17 = array_get v8, index u32 30 -> u1
                v18 = not v17
                v19 = cast v17 as Field
                v20 = cast v18 as Field
                v21 = mul v15, v15
                v22 = mul v21, v20
                v23 = mul v21, Field 2
                v24 = mul v23, v19
                v25 = add v22, v24
                v27 = array_get v8, index u32 29 -> u1
                v28 = not v27
                v29 = cast v27 as Field
                v30 = cast v28 as Field
                v31 = mul v25, v25
                v32 = mul v31, v30
                v33 = mul v31, Field 2
                v34 = mul v33, v29
                v35 = add v32, v34
                v37 = array_get v8, index u32 28 -> u1
                v38 = not v37
                v39 = cast v37 as Field
                v40 = cast v38 as Field
                v41 = mul v35, v35
                v42 = mul v41, v40
                v43 = mul v41, Field 2
                v44 = mul v43, v39
                v45 = add v42, v44
                v47 = array_get v8, index u32 27 -> u1
                v48 = not v47
                v49 = cast v47 as Field
                v50 = cast v48 as Field
                v51 = mul v45, v45
                v52 = mul v51, v50
                v53 = mul v51, Field 2
                v54 = mul v53, v49
                v55 = add v52, v54
                v57 = array_get v8, index u32 26 -> u1
                v58 = not v57
                v59 = cast v57 as Field
                v60 = cast v58 as Field
                v61 = mul v55, v55
                v62 = mul v61, v60
                v63 = mul v61, Field 2
                v64 = mul v63, v59
                v65 = add v62, v64
                v67 = array_get v8, index u32 25 -> u1
                v68 = not v67
                v69 = cast v67 as Field
                v70 = cast v68 as Field
                v71 = mul v65, v65
                v72 = mul v71, v70
                v73 = mul v71, Field 2
                v74 = mul v73, v69
                v75 = add v72, v74
                v77 = array_get v8, index u32 24 -> u1
                v78 = not v77
                v79 = cast v77 as Field
                v80 = cast v78 as Field
                v81 = mul v75, v75
                v82 = mul v81, v80
                v83 = mul v81, Field 2
                v84 = mul v83, v79
                v85 = add v82, v84
                v87 = array_get v8, index u32 23 -> u1
                v88 = not v87
                v89 = cast v87 as Field
                v90 = cast v88 as Field
                v91 = mul v85, v85
                v92 = mul v91, v90
                v93 = mul v91, Field 2
                v94 = mul v93, v89
                v95 = add v92, v94
                v97 = array_get v8, index u32 22 -> u1
                v98 = not v97
                v99 = cast v97 as Field
                v100 = cast v98 as Field
                v101 = mul v95, v95
                v102 = mul v101, v100
                v103 = mul v101, Field 2
                v104 = mul v103, v99
                v105 = add v102, v104
                v107 = array_get v8, index u32 21 -> u1
                v108 = not v107
                v109 = cast v107 as Field
                v110 = cast v108 as Field
                v111 = mul v105, v105
                v112 = mul v111, v110
                v113 = mul v111, Field 2
                v114 = mul v113, v109
                v115 = add v112, v114
                v117 = array_get v8, index u32 20 -> u1
                v118 = not v117
                v119 = cast v117 as Field
                v120 = cast v118 as Field
                v121 = mul v115, v115
                v122 = mul v121, v120
                v123 = mul v121, Field 2
                v124 = mul v123, v119
                v125 = add v122, v124
                v127 = array_get v8, index u32 19 -> u1
                v128 = not v127
                v129 = cast v127 as Field
                v130 = cast v128 as Field
                v131 = mul v125, v125
                v132 = mul v131, v130
                v133 = mul v131, Field 2
                v134 = mul v133, v129
                v135 = add v132, v134
                v137 = array_get v8, index u32 18 -> u1
                v138 = not v137
                v139 = cast v137 as Field
                v140 = cast v138 as Field
                v141 = mul v135, v135
                v142 = mul v141, v140
                v143 = mul v141, Field 2
                v144 = mul v143, v139
                v145 = add v142, v144
                v147 = array_get v8, index u32 17 -> u1
                v148 = not v147
                v149 = cast v147 as Field
                v150 = cast v148 as Field
                v151 = mul v145, v145
                v152 = mul v151, v150
                v153 = mul v151, Field 2
                v154 = mul v153, v149
                v155 = add v152, v154
                v157 = array_get v8, index u32 16 -> u1
                v158 = not v157
                v159 = cast v157 as Field
                v160 = cast v158 as Field
                v161 = mul v155, v155
                v162 = mul v161, v160
                v163 = mul v161, Field 2
                v164 = mul v163, v159
                v165 = add v162, v164
                v167 = array_get v8, index u32 15 -> u1
                v168 = not v167
                v169 = cast v167 as Field
                v170 = cast v168 as Field
                v171 = mul v165, v165
                v172 = mul v171, v170
                v173 = mul v171, Field 2
                v174 = mul v173, v169
                v175 = add v172, v174
                v177 = array_get v8, index u32 14 -> u1
                v178 = not v177
                v179 = cast v177 as Field
                v180 = cast v178 as Field
                v181 = mul v175, v175
                v182 = mul v181, v180
                v183 = mul v181, Field 2
                v184 = mul v183, v179
                v185 = add v182, v184
                v187 = array_get v8, index u32 13 -> u1
                v188 = not v187
                v189 = cast v187 as Field
                v190 = cast v188 as Field
                v191 = mul v185, v185
                v192 = mul v191, v190
                v193 = mul v191, Field 2
                v194 = mul v193, v189
                v195 = add v192, v194
                v197 = array_get v8, index u32 12 -> u1
                v198 = not v197
                v199 = cast v197 as Field
                v200 = cast v198 as Field
                v201 = mul v195, v195
                v202 = mul v201, v200
                v203 = mul v201, Field 2
                v204 = mul v203, v199
                v205 = add v202, v204
                v207 = array_get v8, index u32 11 -> u1
                v208 = not v207
                v209 = cast v207 as Field
                v210 = cast v208 as Field
                v211 = mul v205, v205
                v212 = mul v211, v210
                v213 = mul v211, Field 2
                v214 = mul v213, v209
                v215 = add v212, v214
                v217 = array_get v8, index u32 10 -> u1
                v218 = not v217
                v219 = cast v217 as Field
                v220 = cast v218 as Field
                v221 = mul v215, v215
                v222 = mul v221, v220
                v223 = mul v221, Field 2
                v224 = mul v223, v219
                v225 = add v222, v224
                v227 = array_get v8, index u32 9 -> u1
                v228 = not v227
                v229 = cast v227 as Field
                v230 = cast v228 as Field
                v231 = mul v225, v225
                v232 = mul v231, v230
                v233 = mul v231, Field 2
                v234 = mul v233, v229
                v235 = add v232, v234
                v237 = array_get v8, index u32 8 -> u1
                v238 = not v237
                v239 = cast v237 as Field
                v240 = cast v238 as Field
                v241 = mul v235, v235
                v242 = mul v241, v240
                v243 = mul v241, Field 2
                v244 = mul v243, v239
                v245 = add v242, v244
                v247 = array_get v8, index u32 7 -> u1
                v248 = not v247
                v249 = cast v247 as Field
                v250 = cast v248 as Field
                v251 = mul v245, v245
                v252 = mul v251, v250
                v253 = mul v251, Field 2
                v254 = mul v253, v249
                v255 = add v252, v254
                v257 = array_get v8, index u32 6 -> u1
                v258 = not v257
                v259 = cast v257 as Field
                v260 = cast v258 as Field
                v261 = mul v255, v255
                v262 = mul v261, v260
                v263 = mul v261, Field 2
                v264 = mul v263, v259
                v265 = add v262, v264
                v267 = array_get v8, index u32 5 -> u1
                v268 = not v267
                v269 = cast v267 as Field
                v270 = cast v268 as Field
                v271 = mul v265, v265
                v272 = mul v271, v270
                v273 = mul v271, Field 2
                v274 = mul v273, v269
                v275 = add v272, v274
                v277 = array_get v8, index u32 4 -> u1
                v278 = not v277
                v279 = cast v277 as Field
                v280 = cast v278 as Field
                v281 = mul v275, v275
                v282 = mul v281, v280
                v283 = mul v281, Field 2
                v284 = mul v283, v279
                v285 = add v282, v284
                v287 = array_get v8, index u32 3 -> u1
                v288 = not v287
                v289 = cast v287 as Field
                v290 = cast v288 as Field
                v291 = mul v285, v285
                v292 = mul v291, v290
                v293 = mul v291, Field 2
                v294 = mul v293, v289
                v295 = add v292, v294
                v297 = array_get v8, index u32 2 -> u1
                v298 = not v297
                v299 = cast v297 as Field
                v300 = cast v298 as Field
                v301 = mul v295, v295
                v302 = mul v301, v300
                v303 = mul v301, Field 2
                v304 = mul v303, v299
                v305 = add v302, v304
                v307 = array_get v8, index u32 1 -> u1
                v308 = not v307
                v309 = cast v307 as Field
                v310 = cast v308 as Field
                v311 = mul v305, v305
                v312 = mul v311, v310
                v313 = mul v311, Field 2
                v314 = mul v313, v309
                v315 = add v312, v314
                v317 = array_get v8, index u32 0 -> u1
                v318 = not v317
                v319 = cast v317 as Field
                v320 = cast v318 as Field
                v321 = mul v315, v315
                v322 = mul v321, v320
                v323 = mul v321, Field 2
                v324 = mul v323, v319
                v325 = add v322, v324
                v326 = cast v0 as Field
                v327 = mul v326, v325
                v328 = truncate v327 to 32 bits, max_bit_size: 64
                v329 = cast v328 as i32
                return v329
            }
            "#);
        }

        #[test]
        fn removes_shr_with_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: i32):
                v2 = shr v0, i32 2
                return v2
            }
        ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @r"
            acir(inline) fn main f0 {
              b0(v0: i32):
                v2 = lt v0, i32 0
                v3 = cast v2 as Field
                v4 = cast v0 as Field
                v5 = add v3, v4
                v6 = truncate v5 to 32 bits, max_bit_size: 33
                v7 = cast v6 as i32
                v9 = div v7, i32 4
                v10 = cast v2 as i32
                v11 = unchecked_sub v9, v10
                v12 = truncate v11 to 32 bits, max_bit_size: 33
                return v12
            }
            ");
        }

        #[test]
        fn removes_shr_with_non_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: i32, v1: i32):
                v2 = shr v0, v1
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();

            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: i32, v1: i32):
                v2 = cast v1 as u32
                v4 = lt v2, u32 31
                constrain v4 == u1 1, "attempt to bit-shift with overflow"
                v6 = cast v1 as Field
                v8 = call to_le_bits(v6) -> [u1; 32]
                v9 = array_get v8, index u32 31 -> u1
                v10 = not v9
                v11 = cast v9 as Field
                v12 = cast v10 as Field
                v14 = mul Field 2, v11
                v15 = add v12, v14
                v17 = array_get v8, index u32 30 -> u1
                v18 = not v17
                v19 = cast v17 as Field
                v20 = cast v18 as Field
                v21 = mul v15, v15
                v22 = mul v21, v20
                v23 = mul v21, Field 2
                v24 = mul v23, v19
                v25 = add v22, v24
                v27 = array_get v8, index u32 29 -> u1
                v28 = not v27
                v29 = cast v27 as Field
                v30 = cast v28 as Field
                v31 = mul v25, v25
                v32 = mul v31, v30
                v33 = mul v31, Field 2
                v34 = mul v33, v29
                v35 = add v32, v34
                v37 = array_get v8, index u32 28 -> u1
                v38 = not v37
                v39 = cast v37 as Field
                v40 = cast v38 as Field
                v41 = mul v35, v35
                v42 = mul v41, v40
                v43 = mul v41, Field 2
                v44 = mul v43, v39
                v45 = add v42, v44
                v47 = array_get v8, index u32 27 -> u1
                v48 = not v47
                v49 = cast v47 as Field
                v50 = cast v48 as Field
                v51 = mul v45, v45
                v52 = mul v51, v50
                v53 = mul v51, Field 2
                v54 = mul v53, v49
                v55 = add v52, v54
                v57 = array_get v8, index u32 26 -> u1
                v58 = not v57
                v59 = cast v57 as Field
                v60 = cast v58 as Field
                v61 = mul v55, v55
                v62 = mul v61, v60
                v63 = mul v61, Field 2
                v64 = mul v63, v59
                v65 = add v62, v64
                v67 = array_get v8, index u32 25 -> u1
                v68 = not v67
                v69 = cast v67 as Field
                v70 = cast v68 as Field
                v71 = mul v65, v65
                v72 = mul v71, v70
                v73 = mul v71, Field 2
                v74 = mul v73, v69
                v75 = add v72, v74
                v77 = array_get v8, index u32 24 -> u1
                v78 = not v77
                v79 = cast v77 as Field
                v80 = cast v78 as Field
                v81 = mul v75, v75
                v82 = mul v81, v80
                v83 = mul v81, Field 2
                v84 = mul v83, v79
                v85 = add v82, v84
                v87 = array_get v8, index u32 23 -> u1
                v88 = not v87
                v89 = cast v87 as Field
                v90 = cast v88 as Field
                v91 = mul v85, v85
                v92 = mul v91, v90
                v93 = mul v91, Field 2
                v94 = mul v93, v89
                v95 = add v92, v94
                v97 = array_get v8, index u32 22 -> u1
                v98 = not v97
                v99 = cast v97 as Field
                v100 = cast v98 as Field
                v101 = mul v95, v95
                v102 = mul v101, v100
                v103 = mul v101, Field 2
                v104 = mul v103, v99
                v105 = add v102, v104
                v107 = array_get v8, index u32 21 -> u1
                v108 = not v107
                v109 = cast v107 as Field
                v110 = cast v108 as Field
                v111 = mul v105, v105
                v112 = mul v111, v110
                v113 = mul v111, Field 2
                v114 = mul v113, v109
                v115 = add v112, v114
                v117 = array_get v8, index u32 20 -> u1
                v118 = not v117
                v119 = cast v117 as Field
                v120 = cast v118 as Field
                v121 = mul v115, v115
                v122 = mul v121, v120
                v123 = mul v121, Field 2
                v124 = mul v123, v119
                v125 = add v122, v124
                v127 = array_get v8, index u32 19 -> u1
                v128 = not v127
                v129 = cast v127 as Field
                v130 = cast v128 as Field
                v131 = mul v125, v125
                v132 = mul v131, v130
                v133 = mul v131, Field 2
                v134 = mul v133, v129
                v135 = add v132, v134
                v137 = array_get v8, index u32 18 -> u1
                v138 = not v137
                v139 = cast v137 as Field
                v140 = cast v138 as Field
                v141 = mul v135, v135
                v142 = mul v141, v140
                v143 = mul v141, Field 2
                v144 = mul v143, v139
                v145 = add v142, v144
                v147 = array_get v8, index u32 17 -> u1
                v148 = not v147
                v149 = cast v147 as Field
                v150 = cast v148 as Field
                v151 = mul v145, v145
                v152 = mul v151, v150
                v153 = mul v151, Field 2
                v154 = mul v153, v149
                v155 = add v152, v154
                v157 = array_get v8, index u32 16 -> u1
                v158 = not v157
                v159 = cast v157 as Field
                v160 = cast v158 as Field
                v161 = mul v155, v155
                v162 = mul v161, v160
                v163 = mul v161, Field 2
                v164 = mul v163, v159
                v165 = add v162, v164
                v167 = array_get v8, index u32 15 -> u1
                v168 = not v167
                v169 = cast v167 as Field
                v170 = cast v168 as Field
                v171 = mul v165, v165
                v172 = mul v171, v170
                v173 = mul v171, Field 2
                v174 = mul v173, v169
                v175 = add v172, v174
                v177 = array_get v8, index u32 14 -> u1
                v178 = not v177
                v179 = cast v177 as Field
                v180 = cast v178 as Field
                v181 = mul v175, v175
                v182 = mul v181, v180
                v183 = mul v181, Field 2
                v184 = mul v183, v179
                v185 = add v182, v184
                v187 = array_get v8, index u32 13 -> u1
                v188 = not v187
                v189 = cast v187 as Field
                v190 = cast v188 as Field
                v191 = mul v185, v185
                v192 = mul v191, v190
                v193 = mul v191, Field 2
                v194 = mul v193, v189
                v195 = add v192, v194
                v197 = array_get v8, index u32 12 -> u1
                v198 = not v197
                v199 = cast v197 as Field
                v200 = cast v198 as Field
                v201 = mul v195, v195
                v202 = mul v201, v200
                v203 = mul v201, Field 2
                v204 = mul v203, v199
                v205 = add v202, v204
                v207 = array_get v8, index u32 11 -> u1
                v208 = not v207
                v209 = cast v207 as Field
                v210 = cast v208 as Field
                v211 = mul v205, v205
                v212 = mul v211, v210
                v213 = mul v211, Field 2
                v214 = mul v213, v209
                v215 = add v212, v214
                v217 = array_get v8, index u32 10 -> u1
                v218 = not v217
                v219 = cast v217 as Field
                v220 = cast v218 as Field
                v221 = mul v215, v215
                v222 = mul v221, v220
                v223 = mul v221, Field 2
                v224 = mul v223, v219
                v225 = add v222, v224
                v227 = array_get v8, index u32 9 -> u1
                v228 = not v227
                v229 = cast v227 as Field
                v230 = cast v228 as Field
                v231 = mul v225, v225
                v232 = mul v231, v230
                v233 = mul v231, Field 2
                v234 = mul v233, v229
                v235 = add v232, v234
                v237 = array_get v8, index u32 8 -> u1
                v238 = not v237
                v239 = cast v237 as Field
                v240 = cast v238 as Field
                v241 = mul v235, v235
                v242 = mul v241, v240
                v243 = mul v241, Field 2
                v244 = mul v243, v239
                v245 = add v242, v244
                v247 = array_get v8, index u32 7 -> u1
                v248 = not v247
                v249 = cast v247 as Field
                v250 = cast v248 as Field
                v251 = mul v245, v245
                v252 = mul v251, v250
                v253 = mul v251, Field 2
                v254 = mul v253, v249
                v255 = add v252, v254
                v257 = array_get v8, index u32 6 -> u1
                v258 = not v257
                v259 = cast v257 as Field
                v260 = cast v258 as Field
                v261 = mul v255, v255
                v262 = mul v261, v260
                v263 = mul v261, Field 2
                v264 = mul v263, v259
                v265 = add v262, v264
                v267 = array_get v8, index u32 5 -> u1
                v268 = not v267
                v269 = cast v267 as Field
                v270 = cast v268 as Field
                v271 = mul v265, v265
                v272 = mul v271, v270
                v273 = mul v271, Field 2
                v274 = mul v273, v269
                v275 = add v272, v274
                v277 = array_get v8, index u32 4 -> u1
                v278 = not v277
                v279 = cast v277 as Field
                v280 = cast v278 as Field
                v281 = mul v275, v275
                v282 = mul v281, v280
                v283 = mul v281, Field 2
                v284 = mul v283, v279
                v285 = add v282, v284
                v287 = array_get v8, index u32 3 -> u1
                v288 = not v287
                v289 = cast v287 as Field
                v290 = cast v288 as Field
                v291 = mul v285, v285
                v292 = mul v291, v290
                v293 = mul v291, Field 2
                v294 = mul v293, v289
                v295 = add v292, v294
                v297 = array_get v8, index u32 2 -> u1
                v298 = not v297
                v299 = cast v297 as Field
                v300 = cast v298 as Field
                v301 = mul v295, v295
                v302 = mul v301, v300
                v303 = mul v301, Field 2
                v304 = mul v303, v299
                v305 = add v302, v304
                v307 = array_get v8, index u32 1 -> u1
                v308 = not v307
                v309 = cast v307 as Field
                v310 = cast v308 as Field
                v311 = mul v305, v305
                v312 = mul v311, v310
                v313 = mul v311, Field 2
                v314 = mul v313, v309
                v315 = add v312, v314
                v317 = array_get v8, index u32 0 -> u1
                v318 = not v317
                v319 = cast v317 as Field
                v320 = cast v318 as Field
                v321 = mul v315, v315
                v322 = mul v321, v320
                v323 = mul v321, Field 2
                v324 = mul v323, v319
                v325 = add v322, v324
                v326 = cast v325 as i32
                v328 = lt v0, i32 0
                v329 = cast v328 as Field
                v330 = cast v0 as Field
                v331 = add v329, v330
                v332 = truncate v331 to 32 bits, max_bit_size: 33
                v333 = cast v332 as i32
                v334 = div v333, v326
                v335 = cast v328 as i32
                v336 = unchecked_sub v334, v335
                v337 = truncate v336 to 32 bits, max_bit_size: 33
                return v337
            }
            "#);
        }
    }

    #[test]
    fn follows_canonical_block_ordering() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v4 = shr u8 1, u8 98
            v6 = eq v4, u8 0
            jmpif v6 then: b7, else: b8
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            v11 = eq v9, u8 1
            jmpif v11 then: b4, else: b5
          b4():
            jmp b6()
          b5():
            jmp b6()
          b6():
            return
          b7():
            jmp b9()
          b8():
            jmp b9()
          b9():
            v7 = eq v4, u8 1
            jmpif v7 then: b10, else: b11
          b10():
            jmp b12()
          b11():
            jmp b12()
          b12():
            v9 = shr u8 1, u8 99
            v10 = eq v9, u8 0
            jmpif v10 then: b1, else: b2
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_bit_shifts();

        // We expect v9 in b3 to be resolved to `u8 0`. Even though b12 has a higher value,
        // it comes before b3 in the block ordering.
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "attempt to bit-shift with overflow"
            v4 = div u8 1, u8 0
            v5 = eq v4, u8 0
            jmpif v5 then: b7, else: b8
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            v9 = eq v7, u8 1
            jmpif v9 then: b4, else: b5
          b4():
            jmp b6()
          b5():
            jmp b6()
          b6():
            return
          b7():
            jmp b9()
          b8():
            jmp b9()
          b9():
            v6 = eq v4, u8 1
            jmpif v6 then: b10, else: b11
          b10():
            jmp b12()
          b11():
            jmp b12()
          b12():
            constrain u1 0 == u1 1, "attempt to bit-shift with overflow"
            v7 = div u8 1, u8 0
            v8 = eq v7, u8 0
            jmpif v8 then: b1, else: b2
        }
        "#);
    }
}
