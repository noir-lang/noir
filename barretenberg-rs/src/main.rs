use barretenberg_rs::composer::{Assignments, Constraint,  RangeConstraint, ConstraintSystem, StandardComposer};

fn main() {
    println!("Creating constraint system\n");
    let constraint = Constraint {
        a: 1,
        b: 2,
        c: 3,
        qm: 0.into(),
        ql: 1.into(),
        qr: 1.into(),
        qo: (-1).into(),
        qc: 0.into(),
    };

    let range = RangeConstraint{a: 1, num_bits: 32};

    let constraint_system = ConstraintSystem {
        var_num: 3,
        pub_var_num: 0,
        range_constraints: vec![range],
        constraints: vec![constraint],
    };

    println!("Constraint system created\n");

    println!("Initialising CRS, FFT, Pippenger and compiling WASM\n");
    let mut composer = StandardComposer::new(constraint_system.size());
    println!("WASM compiled and the standard composer is ready\n");

    let mut witness = Assignments::new();
    let num = 5;
    witness.push_i32(num);
    witness.push_i32(0);
    witness.push_i32(num);

    println!("Creating proof");
    let proof = composer.create_proof(&constraint_system, witness);
    println!("Proof created\n");

    let public_inputs = None;

    println!("Verifying proof");
    let verified = composer.verify(&constraint_system, &proof, public_inputs);
    println!("Proof verified : {}\n", verified);
}
