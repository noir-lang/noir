#[cfg(test)]
mod tests {
    use acvm::acir::circuit::ExpressionWidth;

    use crate::{
        errors::RuntimeError,
        ssa::{
            opt::assert_normalized_ssa_equals, optimize_all, Ssa, SsaBuilder, SsaEvaluatorOptions,
            SsaLogging,
        },
    };

    fn run_all_passes(ssa: Ssa) -> Result<Ssa, RuntimeError> {
        let options = &SsaEvaluatorOptions {
            ssa_logging: SsaLogging::None,
            enable_brillig_logging: false,
            force_brillig_output: false,
            print_codegen_timings: false,
            expression_width: ExpressionWidth::default(),
            emit_ssa: None,
            skip_underconstrained_check: true,
            skip_brillig_constraints_check: true,
            inliner_aggressiveness: 0,
            max_bytecode_increase_percent: None,
        };

        let builder = SsaBuilder {
            ssa,
            ssa_logging: options.ssa_logging.clone(),
            print_codegen_timings: false,
        };

        optimize_all(builder, options)
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
              jmpif v5 then: b3, else: b2
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

        // After Array Set Optimizations:
        let expected = "
          acir(inline) fn main f0 {
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
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = run_all_passes(ssa).unwrap();
        assert_normalized_ssa_equals(ssa, expected);
    }
}
