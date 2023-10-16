//! This integration test checks whether Nargo can generate and verify recursive proofs.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

use assert_fs::prelude::{FileWriteStr, PathChild};

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

#[test]
fn recursive_verification() {
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
    }
}
