use barretenberg_rs::composer::{
    Assignments, Constraint, ConstraintSystem, LogicConstraint, RangeConstraint, StandardComposer,
};

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
    let range = RangeConstraint { a: 1, num_bits: 32 };

    let lhs_operand: i32 = 230;
    let rhs_operand: i32 = 315;
    let and_result = lhs_operand & rhs_operand;

    let and = {
        let a = 4;
        let b = 5;
        let result = 6;
        let num_bits = 32;

        LogicConstraint::and(a, b, result, num_bits)
    };
    // This constrains the result of the AND
    let constraint2 = Constraint {
        a: 0,
        b: 0,
        c: 6,
        qm: 0.into(),
        ql: 0.into(),
        qr: 0.into(),
        qo: (-1).into(),
        qc: (and_result as i128).into(),
    };

    let constraint_system = ConstraintSystem {
        var_num: 6,
        pub_var_num: 0,
        logic_constraints: vec![and],
        range_constraints: vec![range],
        constraints: vec![constraint, constraint2],
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
    witness.push_i32(lhs_operand);
    witness.push_i32(rhs_operand);
    witness.push_i32(0); // Note: This will be populated by barretenberg with the copy_from_to method

    println!("Creating proof");
    let proof = composer.create_proof(&constraint_system, witness);
    println!("Proof created\n");

    let public_inputs = None;

    println!("Verifying proof");
    let verified = composer.verify(&constraint_system, &proof, public_inputs);
    println!("Proof verified : {}\n", verified);
}

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn write_to_file(bytes: &[u8], path: &Path) -> String {
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(bytes) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => display.to_string(),
    }
}
