//! Contains tests related to the `hint` intrinsic.

#[cfg(test)]
mod tests {

    use crate::{
        assert_ssa_snapshot,
        brillig::BrilligOptions,
        errors::RuntimeError,
        ssa::{
            Ssa, SsaBuilder, SsaEvaluatorOptions, SsaLogging,
            opt::{
                DEFAULT_MAX_SPECIALIZATIONS_PER_FN, DEFAULT_SPECIALIZATION_THRESHOLD,
                FORCE_UNROLL_THRESHOLD, MAX_UNROLL_ITERATIONS, constant_folding, inlining,
            },
            primary_passes,
        },
    };

    fn run_all_passes(ssa: Ssa) -> Result<Ssa, RuntimeError> {
        let options = &SsaEvaluatorOptions {
            ssa_logging: SsaLogging::None,
            brillig_options: BrilligOptions::default(),
            print_codegen_timings: false,
            emit_ssa: None,
            skip_underconstrained_check: true,
            enable_brillig_constraints_check_lookback: false,
            skip_brillig_constraints_check: true,
            inliner_aggressiveness: 0,
            max_bytecode_increase_percent: None,
            max_unroll_iterations: MAX_UNROLL_ITERATIONS,
            constant_folding_max_iter: constant_folding::DEFAULT_MAX_ITER,
            small_function_max_instruction: inlining::MAX_SIMPLE_FUNCTION_WEIGHT,
            force_unroll_threshold: FORCE_UNROLL_THRESHOLD,
            specialization_threshold: DEFAULT_SPECIALIZATION_THRESHOLD,
            max_specializations_per_fn: DEFAULT_MAX_SPECIALIZATIONS_PER_FN,
            skip_passes: Default::default(),
            ssa_logging_hide_unchanged: false,
        };

        let builder = SsaBuilder::from_ssa(
            ssa,
            options.ssa_logging.clone(),
            options.ssa_logging_hide_unchanged,
            false,
            None,
        );
        Ok(builder.run_passes(&primary_passes(options))?.finish())
    }

    /// Test that the `std::hint::black_box` function prevents some of the optimizations.
    #[test]
    fn test_black_box_hint() {
        // fn main(sum: u32) {
        //     // This version simplifies into a single `constraint 50 == sum`
        //     assert_eq(loop(5, 10), sum);
        //     // This should preserve additions because `k` is opaque, as if it came from an input.
        //     assert_eq(loop(5, std::hint::black_box(10)), sum);
        // }
        // fn loop(n: u32, k: u32) -> u32 {
        //     let mut sum = 0;
        //     for _ in 0..n {
        //         sum = sum + k;
        //     }
        //     sum
        // }

        // Initial SSA:
        let src = "
          acir(inline) fn main f0 {
            b0(v0: u32):
              v4 = call f1(u32 5, u32 10) -> u32
              v5 = eq v4, v0
              constrain v4 == v0
              v7 = call black_box(u32 10) -> u32
              v9 = call f1(u32 5, v7) -> u32
              v10 = eq v9, v0
              constrain v9 == v0
              return
          }
          acir(inline) fn loop f1 {
            b0(v0: u32, v1: u32):
              v3 = allocate -> &mut u32
              store u32 0 at v3
              jmp b1(u32 0)
            b1(v2: u32):
              v5 = lt v2, v0
              jmpif v5 then: b3(), else: b2()
            b3():
              v7 = load v3 -> u32
              v8 = add v7, v1
              store v8 at v3
              v10 = add v2, u32 1
              jmp b1(v10)
            b2():
              v6 = load v3 -> u32
              return v6
          }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = run_all_passes(ssa).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
          b0(v0: u32):
            constrain u32 50 == v0
            v4 = call black_box(u32 10) -> u32
            v5 = add v4, v4
            v6 = add v5, v4
            v7 = add v6, v4
            v8 = add v7, v4
            constrain v8 == u32 50
            return
        }
        ");
    }

    /// Test that pure brillig calls with constant arguments inside conditional
    /// branches are interpreted before flattening, so their results are
    /// constant when the predicate multiplication happens.
    ///
    /// The ACIR `decompose` wrapper around the brillig `decompose_hint` is
    /// essential: it prevents the brillig call from being inlined away by
    /// preprocessing. Without the pre-flattening constant folding pass,
    /// the brillig hint wouldn't be interpreted and the pre-check on
    /// flattening would catch it.
    ///
    /// Note: the `constrain` and `range_check` instructions from the real
    /// stdlib `decompose` are removed so that `decompose_hint` is classified
    /// as `Pure` (not `PureWithPredicate`). The pre-check only fires for
    /// non-impure brillig calls.
    #[test]
    fn test_brillig_constant_folding_before_flattening() {
        let src = "
            g0 = Field 340282366920938463463374607431768211456
            g1 = Field 53438638232309528389504892708671455233
            g2 = Field 64323764613183177041862057485226039389

            acir(inline) fn main f0 {
              b0(v0: u1):
                jmpif v0 then: b1(), else: b2()
              b1():
                v1, v2 = call f1(Field 340282366920938463463374607431768211456) -> (Field, Field)
                constrain v1 == Field 0
                constrain v2 == Field 1
                jmp b2()
              b2():
                return
            }
            acir(inline) fn decompose f1 {
              b0(v0: Field):
                v1, v2 = call f2(v0) -> (Field, Field)
                call f3(v1)
                call f3(v2)
                v3 = mul Field 340282366920938463463374607431768211456, v2
                v4 = add v1, v3
                call f4(Field 53438638232309528389504892708671455233, Field 64323764613183177041862057485226039389, v1, v2)
                return v1, v2
            }
            brillig(inline) fn decompose_hint f2 {
              b0(v0: Field):
                v1 = truncate v0 to 128 bits, max_bit_size: 254
                v2 = sub v0, v1
                v3 = mul v2, Field 8680525429001239497728366687280168587232520577698044359798894838135247199343
                return v1, v3
            }
            acir(inline) fn assert_max_bit_size f3 {
              b0(v0: Field):
                return
            }
            acir(inline) fn assert_gt_limbs f4 {
              b0(v0: Field, v1: Field, v2: Field, v3: Field):
                v4 = call f5(v0, v2) -> u1
                v5 = sub v0, v2
                v6 = sub v5, Field 1
                v7 = cast v4 as Field
                v8 = mul v7, Field 340282366920938463463374607431768211456
                v9 = add v6, v8
                v10 = sub v1, v3
                v11 = cast v4 as Field
                v12 = sub v10, v11
                call f3(v9)
                call f3(v12)
                return
            }
            brillig(inline) fn lte_hint f5 {
              b0(v0: Field, v1: Field):
                v2 = eq v0, v1
                jmpif v2 then: b1(), else: b2()
              b1():
                jmp b3(u1 1)
              b2():
                v3 = call field_less_than(v0, v1) -> u1
                jmp b3(v3)
              b3(v4: u1):
                return v4
            }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = run_all_passes(ssa).unwrap();
        // With pre-flattening constant folding, the brillig decompose_hint
        // is interpreted with constant 2^128, producing lo=0, hi=1.
        // The constrains become trivially true, simplifying to nothing.
        assert_ssa_snapshot!(ssa, @r"
        g0 = Field 340282366920938463463374607431768211456
        g1 = Field 53438638232309528389504892708671455233
        g2 = Field 64323764613183177041862057485226039389

        acir(inline) pure fn main f0 {
          b0(v3: u1):
            enable_side_effects u1 1
            return
        }
        ");
    }
}
