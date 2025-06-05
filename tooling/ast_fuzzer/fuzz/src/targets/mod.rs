pub mod acir_vs_brillig;
pub mod comptime_vs_brillig;
pub mod min_vs_full;
pub mod orig_vs_morph;
pub mod pass_vs_prev;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use arbitrary::Unstructured;
    use color_eyre::eyre;

    pub fn seed_from_env() -> Option<u64> {
        let Ok(seed) = std::env::var("NOIR_ARBTEST_SEED") else { return None };
        let seed = u64::from_str_radix(seed.trim_start_matches("0x"), 16)
            .unwrap_or_else(|e| panic!("failed to parse seed '{seed}': {e}"));
        Some(seed)
    }

    /// We can use this to disable the proptests on CI until we fix known bugs.
    ///
    /// The tests should always be enabled locally. They can be run with:
    ///
    /// ```ignore
    /// cargo test -p noir_ast_fuzzer_fuzz
    /// ```
    #[allow(unused)]
    pub fn is_running_in_ci() -> bool {
        std::env::var("CI").is_ok()
    }

    /// `cargo fuzz` takes a long time to ramp up the complexity.
    /// This test catches crash bugs much faster.
    ///
    /// Run it with for example:
    /// ```ignore
    /// NOIR_ARBTEST_SEED=0x6819c61400001000 \
    /// NOIR_AST_FUZZER_SHOW_AST=1 \
    /// cargo test -p noir_ast_fuzzer_fuzz acir_vs_brillig
    /// ```
    pub fn fuzz_with_arbtest(f: impl Fn(&mut Unstructured) -> eyre::Result<()>) {
        let _ = env_logger::try_init();

        let mut prop = arbtest::arbtest(|u| {
            f(u).unwrap();
            Ok(())
        })
        .budget(Duration::from_secs(10))
        .size_min(1 << 12)
        .size_max(1 << 20);

        if let Some(seed) = seed_from_env() {
            prop = prop.seed(seed);
        }

        prop.run();
    }
}
