use barretenberg_rs::composer::{Assignments, Constraint, ConstraintSystem, StandardComposer};

fn main() {
    println!("Creating constraint system\n");
    let constraint = Constraint {
        a: 1,
        b: 2,
        c: 3,
        qm: 0,
        ql: 1,
        qr: 1,
        qo: -1,
        qc: 0,
    };

    let constraint_system = ConstraintSystem {
        var_num: 3,
        pub_var_num: 0,
        constraints: vec![constraint],
    };

    println!("Constraint system created\n");

    println!("Initialising CRS, FFT, Pippenger and compiling WASM\n");
    let mut composer = StandardComposer::new(constraint_system.size());
    println!("WASM compiled and the standard composer is ready\n");

    let mut witness = Assignments::new();
    let num = 5;
    witness.push(num);
    witness.push(0);
    witness.push(num);

    println!("Creating proof");
    let proof = composer.create_proof(&constraint_system, witness);
    println!("Proof created\n");

    let public_inputs = None;

    println!("Verifying proof");
    let verified = composer.verify(&constraint_system, &proof, public_inputs);
    println!("Proof verified : {}\n", verified);
}
