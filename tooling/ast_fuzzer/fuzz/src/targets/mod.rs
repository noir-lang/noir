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
}
