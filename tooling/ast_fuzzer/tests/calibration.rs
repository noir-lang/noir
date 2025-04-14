//! Calibration test for the AST program generator, which generates a bunch of random programs,
//! visits all the expressions in the AST, and counts the appearance of the labels we put in
//! the `Freqs` in `ast_fuzzer/src/lib.rs`. Then, we assert that the relative frequency of
//! the different labels is within an acceptable range.
//!
//! We can use this to calibrate the frequency values with some statistical feedback.
//!
//! ```shell
//! cargo test -p noir_ast_fuzzer --test calibration -- --nocapture
//! ```
use std::collections::BTreeMap;

use arbtest::arbtest;
use noir_ast_fuzzer::{Config, arb_program, visit_expr};
use noirc_frontend::monomorphization::ast::Expression;

#[test]
fn arb_program_freqs_in_expected_range() {
    // Counting labels separately for ACIR and Brillig.
    let mut counts: BTreeMap<bool, BTreeMap<&str, usize>> = Default::default();
    let mut program_count = 0;

    arbtest(|u| {
        let program = arb_program(u, Config::default())?;
        for func in program.functions {
            let counts = counts.entry(func.unconstrained).or_default();

            let mut obs = |key| {
                let cnt = counts.entry(key).or_default();
                *cnt = *cnt + 1;
            };

            // Visit the
            visit_expr(&func.body, &mut |expr| {
                let key = match expr {
                    Expression::Literal(_) => "literal",
                    Expression::Block(_) => "block",
                    Expression::Unary(_) => "unary",
                    Expression::Binary(_) => "binary",
                    Expression::For(_) => "for",
                    Expression::Loop(_) => "loop",
                    Expression::While(_) => "while",
                    Expression::If(_) => "if",
                    Expression::Match(_) => "match",
                    Expression::Call(_) => "call",
                    Expression::Let(_) => "let",
                    Expression::Constrain(_, _, _) => "constrain",
                    Expression::Assign(_) => "assign",
                    Expression::Drop(_) => "drop",
                    Expression::Break => "break",
                    Expression::Continue => "continue",
                    Expression::Ident(_)
                    | Expression::Cast(_)
                    | Expression::Tuple(_)
                    | Expression::ExtractTupleField(_, _)
                    | Expression::Index(_)
                    | Expression::Semi(_)
                    | Expression::Clone(_) => {
                        return true;
                    }
                };
                obs(key);
                true
            });
        }
        program_count += 1;
        Ok(())
    })
    .budget_ms(1000)
    .size_min(1 << 12)
    .size_max(1 << 20);

    println!("Generated {program_count} programs.");
    for (c, counts) in counts {
        println!("{} frequencies:", if c { "Brillig" } else { "ACIR" });
        for (key, count) in counts {
            println!("\t{key}: {count}");
        }
    }

    // TODO: Assert relative frequencies
}
