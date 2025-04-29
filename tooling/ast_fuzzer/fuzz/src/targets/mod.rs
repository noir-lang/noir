pub mod acir_vs_brillig;
pub mod init_vs_final;
pub mod orig_vs_mutant;

#[cfg(test)]
mod tests {
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
    pub fn should_ignore_on_ci() -> bool {
        // TODO: Enable on CI once the following are fixed:
        // #8229, #8230, #8231, #8236, #8261, #8262
        std::env::var("CI").is_ok()
    }
}
