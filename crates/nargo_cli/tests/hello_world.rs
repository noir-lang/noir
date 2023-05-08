//! This integration test aims to mirror the steps taken by a new user using Nargo for the first time.
//! It then follows the steps published at https://noir-lang.org/getting_started/hello_world.html
//! Any modifications to the commands run here MUST be documented in the noir-lang book.

mod prove_and_verify;

use acvm::FieldElement;
use assert_cmd::prelude::*;
use noirc_abi::input_parser::{Format, InputValue};
use predicates::prelude::*;
use std::{process::Command, collections::BTreeMap, fs::File, io::Write};
use tempdir::TempDir;

use prove_and_verify::copy_recursively;

use assert_fs::prelude::{FileWriteStr, PathAssert, PathChild};

#[test]
fn hello_world_example() {
    let test_dir = assert_fs::TempDir::new().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    let project_name = "hello_world";
    let project_dir = test_dir.child(project_name);

    // `nargo new hello_world`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("new").arg(project_name);
    cmd.assert().success().stdout(predicate::str::contains("Project successfully created!"));

    project_dir.child("src").assert(predicate::path::is_dir());
    project_dir.child("Nargo.toml").assert(predicate::path::is_file());

    std::env::set_current_dir(&project_dir).unwrap();

    // `nargo check`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("check");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Constraint system successfully built!"));

    project_dir.child("Prover.toml").assert(predicate::path::is_file());
    project_dir.child("Verifier.toml").assert(predicate::path::is_file());

    // `nargo prove p`
    let proof_name = "p";
    project_dir.child("Prover.toml").write_str("x = 1\ny = 2").unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("prove").arg(proof_name);
    cmd.assert().success();

    project_dir
        .child("proofs")
        .child(format!("{proof_name}.proof"))
        .assert(predicate::path::is_file());

    // `nargo verify p`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("verify").arg(proof_name);
    cmd.assert().success();
}

#[test]
fn recursive_proof_composition() {
    let current_dir = std::env::current_dir().unwrap();

    let xor_dir = current_dir.join("tests/test_data/xor");
    let recursion_dir = current_dir.join("tests/test_data/recursion");

    let tmp_dir = TempDir::new("recursion_tests").unwrap();
    let tmp_xor_dir = tmp_dir.as_ref().join("xor");
    let tmp_recursion_dir = tmp_dir.as_ref().join("recursion");

    copy_recursively(xor_dir, &tmp_xor_dir)
        .expect("failed to copy test cases to temp directory");

    copy_recursively(recursion_dir, &tmp_recursion_dir)
        .expect("failed to copy test cases to temp directory");

    std::env::set_current_dir(&tmp_xor_dir).unwrap();

    // `nargo check`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("check");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Constraint system successfully built!"));

    let circuit_name = "c";
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("compile").arg(circuit_name);
    cmd.assert().success();

    let proof_name = "p";
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("prove").arg(proof_name).arg(circuit_name);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("recursion").arg(proof_name).arg(circuit_name);
    cmd.assert().success();

    let recursion_artifacts_path = tmp_xor_dir.join("target/recursion").join(format!("{circuit_name}.toml"));

    let recursion_artifacts_string = std::fs::read_to_string(recursion_artifacts_path).unwrap();

    let mut recursion_circuit_inputs_map = BTreeMap::new();
    recursion_circuit_inputs_map.insert("public_input".to_owned(), InputValue::Field(FieldElement::from_hex("0x0a").unwrap()));

    let input_aggregation_object = [FieldElement::zero(); 16].to_vec();
    recursion_circuit_inputs_map.insert("input_aggregation_object".to_owned(), InputValue::Vec(input_aggregation_object));

    let additional_inputs = Format::Toml.serialize(&recursion_circuit_inputs_map).unwrap();
    let full_recursion_prover_inputs = recursion_artifacts_string + "\n" + &additional_inputs;

    let recursion_prover_file_path = tmp_recursion_dir.join("Prover").with_extension("toml");
    let mut recursion_prover_file = match File::create(recursion_prover_file_path) {
        Err(why) => panic!("couldn't create recursion Prover.toml: {why}"),
        Ok(file) => file,
    };
    match recursion_prover_file.write_all(full_recursion_prover_inputs.as_bytes()) {
        Err(why) => panic!("couldn't write to recursion Prover.toml: {why}"),
        Ok(_) => (),
    }

    std::env::set_current_dir(&tmp_recursion_dir).unwrap();

    // `nargo check`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("check");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Constraint system successfully built!"));

    // `nargo prove p`
    let proof_name = "p";
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("prove").arg(proof_name);
    cmd.assert().success();

    // `nargo verify p`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("verify").arg(proof_name);
    cmd.assert().success();

}
