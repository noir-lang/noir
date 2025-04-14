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
use noirc_frontend::monomorphization::ast::{Expression, Type};

#[test]
fn arb_program_freqs_in_expected_range() {
    // Counting labels separately for ACIR and Brillig, and then whether it's an expression or a statement.
    let mut counts: BTreeMap<bool, BTreeMap<&str, BTreeMap<&str, usize>>> = Default::default();
    let mut program_count = 0;

    arbtest(|u| {
        let program = arb_program(u, Config::default())?;
        for func in program.functions {
            visit_expr(&func.body, &mut |expr| {
                let Some((group, key)) = classify(expr) else {
                    return true;
                };
                let count = counts
                    .entry(func.unconstrained)
                    .or_default()
                    .entry(group)
                    .or_default()
                    .entry(key)
                    .or_default();
                *count += 1;
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
    for (unconstrained, counts) in &counts {
        println!("{} frequencies:", if *unconstrained { "Brillig" } else { "ACIR" });
        for (group, counts) in counts {
            let total = counts.values().sum::<usize>();
            println!("\t{group} (total {total}):");
            for (key, count) in counts {
                println!(
                    "\t\t{key}:{} {count}\t({}/100)",
                    std::iter::repeat_n(" ", 15 - key.len()).collect::<String>(),
                    count * 100 / total
                );
            }
        }
    }

    let freq_100 = |unconstrained, group: &str, keys: &[&str]| {
        keys.iter().map(|key| counts[&unconstrained][group][key]).sum::<usize>() * 100
            / counts[&unconstrained][group].values().sum::<usize>()
    };

    // Assert relative frequencies
    let loops_a = freq_100(false, "stmt", &["for"]);
    let loops_b = freq_100(true, "stmt", &["for", "loop", "while"]);

    assert!(9 <= loops_a && loops_a <= 11, "ACIR loops: {loops_a}");
    assert!(loops_a - 1 <= loops_b && loops_b <= loops_a + 1, "Brillig loops: {loops_b}");
}

/// Classify the expression into "expr" or "stmt" for frequency settings.
fn classify(expr: &Expression) -> Option<(&'static str, &'static str)> {
    let cat = match expr {
        Expression::Ident(_)
        | Expression::Cast(_)
        | Expression::Tuple(_)
        | Expression::ExtractTupleField(_, _)
        | Expression::Index(_)
        | Expression::Semi(_)
        | Expression::Clone(_) => {
            return None;
        }
        Expression::Literal(_) => ("expr", "literal"),
        Expression::Block(xs) => {
            (xs.last().and_then(classify).map(|(c, _)| c).unwrap_or("stmt"), "block")
        }
        Expression::Unary(_) => ("expr", "unary"),
        Expression::Binary(_) => ("expr", "binary"),
        Expression::For(_) => ("stmt", "for"),
        Expression::Loop(_) => ("stmt", "loop"),
        Expression::While(_) => ("stmt", "while"),
        Expression::If(x) => (if x.typ == Type::Unit { "stmt" } else { "expr" }, "if"),
        Expression::Match(_) => todo!("match"),
        Expression::Call(x) => (if x.return_type == Type::Unit { "stmt" } else { "expr" }, "call"),
        Expression::Let(_) => ("stmt", "let"),
        Expression::Constrain(_, _, _) => ("stmt", "constrain"),
        Expression::Assign(_) => ("stmt", "assign"),
        Expression::Drop(_) => ("stmt", "drop"),
        Expression::Break => ("stmt", "break"),
        Expression::Continue => ("stmt", "continue"),
    };
    Some(cat)
}
