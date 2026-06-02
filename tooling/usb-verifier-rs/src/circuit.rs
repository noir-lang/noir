//! Circuit identity — bytecode and ABI are bundled at compile time so the USB
//! only needs to carry the `proof.json`, not a separate circuit artifact.
//!
//! The bytecode string is the base64-gzip-compressed ACIR as emitted by the
//! Noir compiler for `demo/usb-auth/src/main.nr`.

/// Circuit ABI summary shown for introspection (`--info` flag).
pub struct CircuitInfo {
    pub name: &'static str,
    pub noir_version: &'static str,
    pub public_inputs: &'static [&'static str],
    /// Base64-encoded gzip-compressed ACIR bytecode.
    pub bytecode: &'static str,
}

pub const CIRCUIT: CircuitInfo = CircuitInfo {
    name: "usb_auth",
    noir_version: "0.33.0",
    public_inputs: &["usb_serial", "commitment", "challenge", "user_id_hash"],
    bytecode: "H4sIAAAAAAAA/62T3QrDIAxGTX+210mM1uRurzKZff9H2GCRluJd84EERA6Jh0D45/k7czgCVl9W8V6oc6cLl3FLqZXYiOmNUatkTLluQkJZ8icKc5MkRasWVErcaM/Ku4Enxx4XPxaO/vDurMEceXMfznP3LM59giPL0TWtjixPF3ByAAMvYPs4211/s4ZBvqWD4XIpBAAA",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_public_inputs_include_usb_serial() {
        assert!(CIRCUIT.public_inputs.contains(&"usb_serial"), "hardware binding requires usb_serial as public input");
    }

    #[test]
    fn bytecode_is_non_empty_base64() {
        assert!(!CIRCUIT.bytecode.is_empty());
        // Spot-check: base64 characters only.
        assert!(CIRCUIT.bytecode.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
    }
}
