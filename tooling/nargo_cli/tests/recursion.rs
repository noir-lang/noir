//! This integration test checks whether Nargo can generate and verify recursive proofs.

use std::{collections::BTreeMap, process::Command, sync::Arc};

use assert_cmd::prelude::*;
use assert_fs::prelude::{FileWriteStr, PathAssert, PathChild};
use noirc_abi::{input_parser::Format, Abi, AbiParameter, AbiType};
use predicates::prelude::*;

use acvm::{acir::native_types::Witness, FieldElement};
use ethers::{
    prelude::{abigen, ContractFactory, SignerMiddleware},
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    solc::{CompilerInput, CompilerOutput, Solc},
    utils::Anvil,
};

const RECURSION_SOURCE: &str = "use dep::std;

fn main(
    verification_key : [Field; 114],
    proof : [Field; 94],
    public_inputs : [Field; 1],
    key_hash : Field,
) -> pub [Field;16] {
    let input_aggregation_object = [0; 16];
    std::verify_proof(
        verification_key.as_slice(),
        proof.as_slice(),
        public_inputs.as_slice(),
        key_hash,
        input_aggregation_object
    )
}
";

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
async fn recursive_verification() {
    let test_dir = assert_fs::TempDir::new().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    // We're going to set up two Noir projects in this test.

    // "main" will be the inner proof which we'll later verify inside another Noir program

    let main_project_name = "main";
    let main_project_dir = test_dir.child(main_project_name);

    // `nargo new main`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("new").arg(main_project_name);
    cmd.assert().success().stdout(predicate::str::contains("Project successfully created!"));

    // "recursion" is a Noir program which does a single thing, verifying a proof from another Noir program.

    let recursion_project_name = "recursion";
    let recursion_project_dir = test_dir.child(recursion_project_name);

    // `nargo new recursion`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("new").arg(recursion_project_name);
    cmd.assert().success().stdout(predicate::str::contains("Project successfully created!"));

    // First create the `main` project which will form the "intermediate" proof which we'll
    // later verify inside a later Noir program.
    {
        std::env::set_current_dir(&main_project_dir).unwrap();

        // `nargo check`
        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.arg("check");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Constraint system successfully built!"));

        // `nargo prove`
        main_project_dir.child("Prover.toml").write_str("x = 1\ny = 2").unwrap();

        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.arg("prove").arg("--recursive");
        cmd.assert().success();

        // `nargo verify p`
        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.arg("verify").arg("--recursive");
        cmd.assert().success();
    }

    // At this point we can create and verify intermediate proofs. Need to check that this can be done inside
    // another proof however.
    {
        std::env::set_current_dir(&recursion_project_dir).unwrap();

        // replace source with recursion program
        let entrypoint = recursion_project_dir.child("src").child("main.nr");
        std::fs::remove_file(&entrypoint).unwrap();
        entrypoint.write_str(RECURSION_SOURCE).unwrap();

        // Pull in recursion inputs as outputted by `main`.
        std::fs::copy(
            main_project_dir.child("recursive_prover.toml"),
            recursion_project_dir.child("Prover.toml"),
        )
        .unwrap();

        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.arg("prove");
        cmd.assert().success();

        // `nargo verify p`
        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.arg("verify");
        cmd.assert().success();

        // Now we need to test the proof against the solidity verifier.

        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.arg("codegen-verifier");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Contract successfully created and located at"));

        let contract_file =
            recursion_project_dir.child("contract").child("recursion").child("plonk_vk.sol");
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

        let encoded_proof: Vec<u8> =
            std::fs::read(recursion_project_dir.child("proofs").child("recursion.proof")).unwrap();
        let proof = hex::decode(encoded_proof).unwrap();

        let abi = Abi {
            parameters: Vec::new(),
            param_witnesses: BTreeMap::new(),
            return_type: Some(noirc_abi::AbiType::Array {
                length: 16,
                typ: Box::new(AbiType::Field),
            }),
            return_witnesses: (0..16).map(Witness).collect(),
        };

        let inputs = std::fs::read_to_string(recursion_project_dir.join("Verifier.toml")).unwrap();
        let mut inputs_map = Format::Toml.parse(&inputs, &abi).unwrap();
        let output_aggregation_object = inputs_map.remove("return").unwrap();

        let public_inputs = abi.encode(&BTreeMap::new(), Some(output_aggregation_object)).unwrap();

        let public_inputs = public_inputs
            .into_iter()
            .map(|(_, field)| field.to_be_bytes().try_into().unwrap())
            .collect();

        // Verify it against the contract.
        let valid_proof: bool = contract.verify(proof.into(), public_inputs).call().await.unwrap();

        assert!(valid_proof);
    }
}
