//! This integration test aims to check that the `nargo codegen-verifier` will successfully create a
//! file containing a verifier for a simple program.
//! A proof is then generated and then verified against this contract.

use acvm::FieldElement;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{process::Command, sync::Arc};

use assert_fs::prelude::{FileWriteStr, PathAssert, PathChild};

use ethers::{
    prelude::{abigen, ContractFactory, SignerMiddleware},
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    solc::{CompilerInput, CompilerOutput, Solc},
    utils::Anvil,
};

// `UltraVerifier` uses pragma `>=0.8.4` however compilation fails for versions below `0.8.7`.
const SOLC_VERSION: &str = "0.8.14";

abigen!(
    UltraVerifier,
    r#"[
        function verify(bytes calldata _proof, bytes32[] calldata _publicInputs) external view returns (bool)
        function getVerificationKeyHash() public pure returns (bytes32)
    ]"#,
);

#[tokio::test]
async fn simple_verifier_codegen() {
    let test_dir = assert_fs::TempDir::new().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    // Create trivial program
    let project_name = "hello_world";
    let project_dir = test_dir.child(project_name);

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("new").arg(project_name);
    cmd.assert().success();

    std::env::set_current_dir(&project_dir).unwrap();

    // Run `nargo codegen-verifier`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("codegen-verifier");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Contract successfully created and located at"));

    let contract_file = project_dir.child("contract").child("hello_world").child("plonk_vk.sol");
    contract_file.assert(predicate::path::is_file());

    // Compile verifier contract.
    let solc = Solc::find_or_install_svm_version(SOLC_VERSION).expect("should find svm");

    let mut compiler_input =
        CompilerInput::new(contract_file).expect("could not load contracts").remove(0);
    compiler_input.settings.evm_version = Some(ethers::solc::EvmVersion::London);
    // TODO: `UltraVerifier` does not compile unless optimizer is enabled.
    // See https://github.com/AztecProtocol/aztec-verifier-contracts/issues/19
    compiler_input.settings.optimizer.enabled = Some(true);

    let compiler_output: CompilerOutput =
        solc.compile(&compiler_input).expect("Could not compile contracts");
    let (abi, bytecode, _) = compiler_output
        .find("UltraVerifier")
        .expect("could not find UltraVerifier contract")
        .into_parts_or_default();
    assert!(!bytecode.is_empty());

    // Spin up node and deploy contract.
    let anvil = Anvil::new().spawn();
    let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

    let private_key = anvil.keys()[0].clone();
    let wallet = LocalWallet::from(private_key).with_chain_id(anvil.chain_id());
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    let factory = ContractFactory::new(abi, bytecode, client.clone());
    let contract = factory.deploy(()).unwrap().send().await.unwrap();
    let contract = UltraVerifier::new(contract.address(), client.clone());

    // Generate a proof for the circuit.
    project_dir.child("Prover.toml").write_str("x = 1\ny = 2").unwrap();
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("prove");
    cmd.assert().success();

    let encoded_proof: Vec<u8> =
        std::fs::read(project_dir.child("proofs").child("hello_world.proof")).unwrap();
    let proof = hex::decode(encoded_proof).unwrap();

    // Verify it against the contract.
    let y = FieldElement::from(2u128);
    let public_inputs: Vec<[u8; 32]> = vec![y.to_be_bytes().try_into().unwrap()];
    let valid_proof: bool = contract.verify(proof.into(), public_inputs).call().await.unwrap();

    assert!(valid_proof);

    drop(test_dir);
}
