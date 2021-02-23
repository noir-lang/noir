// Key is currently {NPComplete_lang}_{OptionalFanIn}_ProofSystem_OrgName
// Org name is needed because more than one implementation of the same proof system may arise

pub(crate) mod backends;
mod tier_one;
mod tier_three;
mod tier_two;
use std::collections::BTreeMap;

use tier_one::TIER_ONE_MAP;
use tier_three::TIER_THREE_MAP;
use tier_two::TIER_TWO_MAP;

use acir::{circuit::Circuit, native_types::Witness};

// re-export acir
pub use acir;
use noir_field::FieldElement;

// Fetches a backend given it's full name
pub fn fetch_by_name(string: &str) -> Option<Box<dyn Backend>> {
    // Check each map to see if we can find the backend name

    if let Some((_, target)) = TIER_ONE_MAP.iter().find(|(name, _)| name == &string) {
        return Some(target.fetch_backend());
    };
    if let Some((_, target)) = TIER_TWO_MAP.iter().find(|(name, _)| name == &string) {
        return Some(target.fetch_backend());
    };

    let (_, target) = TIER_THREE_MAP.iter().find(|(name, _)| name == &string)?;
    Some(target.fetch_backend())
}
pub trait Backend: SmartContract + ProofSystemCompiler {}

pub trait SmartContract {
    // Takes a verification  key and produces a smart contract
    // The platform indicator allows a backend to support multiple smart contract platforms
    //
    // fn verification_key(&self, platform: u8, vk: &[u8], pi : Vec<FieldElement>) -> &[u8] {
    //     todo!("currently the backend is not configured to use this.")
    // }

    /// Takes an ACIR circuit, the number of witnesses and the number of public inputs
    /// Then returns an Ethereum smart contract
    ///
    /// XXX: This will be deprecated in future releases for the above method.
    /// This deprecation may happen in two stages:
    /// The first stage will remove `num_witnesses` and `num_public_inputs` parameters.
    /// If we cannot avoid `num_witnesses`, it can be added into the Circuit struct.
    fn eth_contract_from_cs(
        &self,
        circuit: Circuit,
        num_witnesses: usize,
        num_public_inputs: usize,
    ) -> String;
}

pub trait ProofSystemCompiler {
    /// The NPC language that this proof system directly accepts.
    /// It is possible for ACVM to transpile to different languages, however it is advised to create a new backend
    /// as this in most cases will be inefficient. For this reason, we want to throw a hard error
    /// if the language and proof system does not line up.
    fn np_language(&self) -> Language;

    /// Creates a Proof given the circuit description and the witness values.
    /// It is important to note that the intermediate witnesses for blackbox functions will not generated
    /// This is the responsibility of the proof system.
    ///
    /// See `SmartContract` regarding the removal of `num_witnesses` and `num_public_inputs`
    fn prove_with_meta(
        &self,
        circuit: Circuit,
        witness_values: BTreeMap<Witness, FieldElement>,
        num_witnesses: usize,
        num_public_inputs: usize,
    ) -> Vec<u8>;

    /// Verifies a Proof, given the circuit description.
    ///
    /// XXX: This will be changed in the future to accept a VerifierKey.
    /// At the moment, the Aztec backend API only accepts a constraint system,
    /// which is why this is here.
    ///
    /// See `SmartContract` regarding the removal of `num_witnesses` and `num_public_inputs`
    fn verify_from_cs(
        &self,
        proof: &[u8],
        circuit: Circuit,
        num_witnesses: usize,
        num_public_inputs: usize,
    ) -> bool;
}

/// Supported NP complete languages
/// This might need to be in ACIR instead
pub enum Language {
    R1CS,
    PLONKCSat { width: usize },
}

pub fn hash_constraint_system(cs: &Circuit) {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&format!("{:?}", cs));
    let result = hasher.finalize();
    println!("hash of constraint system : {:x?}", &result[..]);
}
