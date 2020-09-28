use barretenberg_rs::composer::{
    Constraint, ConstraintSystem, LogicConstraint, RangeConstraint, Sha256Constraint,
};
use noir_evaluator::{polynomial::Arithmetic, Circuit, Gate, LowLevelStandardLibrary, HashLibrary};
use noir_field::FieldElement;

/// Converts a `Circuit` into the `StandardFormat` constraint system
/// XXX: This is only required for Barretenberg-rs
pub fn serialise_circuit(
    circuit: &Circuit,
    num_vars: usize,
    num_pub_inputs: usize,
) -> ConstraintSystem {
    // Create constraint system
    let mut constraints: Vec<Constraint> = Vec::new();
    let mut range_constraints: Vec<RangeConstraint> = Vec::new();
    let mut logic_constraints: Vec<LogicConstraint> = Vec::new();
    let mut sha256_constraints: Vec<Sha256Constraint> = Vec::new();

    for gate in circuit.0.iter() {
        match gate {
            Gate::Arithmetic(arithmetic) => {
                let constraint = serialise_arithmetic_gates(arithmetic);
                constraints.push(constraint);
            }
            Gate::Range(witness, num_bits) => {
                let range_constraint = RangeConstraint {
                    a: witness.witness_index() as i32,
                    num_bits: *num_bits as i32,
                };
                range_constraints.push(range_constraint);
            }
            Gate::And(and_gate) => {
                let and = LogicConstraint::and(
                    and_gate.a.witness_index() as i32,
                    and_gate.b.witness_index() as i32,
                    and_gate.result.witness_index() as i32,
                    and_gate.num_bits as i32,
                );
                logic_constraints.push(and);
            }
            Gate::Xor(xor_gate) => {
                let xor = LogicConstraint::xor(
                    xor_gate.a.witness_index() as i32,
                    xor_gate.b.witness_index() as i32,
                    xor_gate.result.witness_index() as i32,
                    xor_gate.num_bits as i32,
                );
                logic_constraints.push(xor);
            }
            Gate::GadgetCall(gadget_call) => {
                match gadget_call.name {
                    LowLevelStandardLibrary::Hash(HashLibrary::SHA256) => {
                        let mut sha256_inputs: Vec<(i32, i32)> = Vec::new();
                        for input in gadget_call.inputs.iter() {
                            let witness_index = input.witness.witness_index() as i32;
                            let num_bits = input.num_bits as i32;
                            sha256_inputs.push((witness_index, num_bits));
                        }

                        assert_eq!(gadget_call.outputs.len(), 2);
                        let result_low_128_witness_index =
                            gadget_call.outputs[0].witness_index() as i32;
                        let result_high_128_witness_index =
                            gadget_call.outputs[1].witness_index() as i32;

                        let sha256_constraint = Sha256Constraint {
                            inputs: sha256_inputs,
                            result_low_128: result_low_128_witness_index,
                            result_high_128: result_high_128_witness_index,
                        };

                        sha256_constraints.push(sha256_constraint);
                    },
                    LowLevelStandardLibrary::Hash(HashLibrary::AES) => panic!("AES has not yet been implemented")
                };
            }
            gate => panic!("Serialiser does not know how to serialise this gate"),
        }
    }

    // Create constraint system
    let constraint_system = ConstraintSystem {
        var_num: num_vars as u32,
        pub_var_num: num_pub_inputs as u32,
        logic_constraints: logic_constraints,
        range_constraints: range_constraints,
        sha256_constraints: vec![],
        constraints: constraints,
    };

    constraint_system
}

fn serialise_arithmetic_gates(gate: &Arithmetic) -> Constraint {
    let mut a: i32 = 0;
    let mut b: i32 = 0;
    let mut c: i32 = 0;
    let mut qm: FieldElement = 0.into();
    let mut ql: FieldElement = 0.into();
    let mut qr: FieldElement = 0.into();
    let mut qo: FieldElement = 0.into();
    let mut qc: FieldElement = 0.into();

    // check mul gate
    if gate.mul_terms.len() != 0 {
        let mul_term = &gate.mul_terms[0];
        qm = mul_term.0;

        // Get wL term
        let wL = &mul_term.1;
        a = wL.witness_index() as i32;

        // Get wR term
        let wR = &mul_term.2;
        b = wR.witness_index() as i32;
    }

    // If there is only one simplified fan term,
    // then put it in qO * wO
    // This is incase, the qM term is non-zero
    if gate.simplified_fan.len() == 1 {
        let qO_wO_term = &gate.simplified_fan[0];
        qo = qO_wO_term.0;

        let wO = &qO_wO_term.1;
        c = wO.witness_index() as i32;
    }

    // XXX: THis is a code smell. Refactor to be better. Maybe change barretenberg to take vectors
    // If there is more than one term,
    // Then add normally
    if gate.simplified_fan.len() == 2 {
        let qL_wL_term = &gate.simplified_fan[0];
        ql = qL_wL_term.0;

        let wL = &qL_wL_term.1;
        a = wL.witness_index() as i32;

        let qR_wR_term = &gate.simplified_fan[1];
        qr = qR_wR_term.0;

        let wR = &qR_wR_term.1;
        b = wR.witness_index() as i32;
    }

    if gate.simplified_fan.len() == 3 {
        let qL_wL_term = &gate.simplified_fan[0];
        ql = qL_wL_term.0;

        let wL = &qL_wL_term.1;
        a = wL.witness_index() as i32;

        let qR_wR_term = &gate.simplified_fan[1];
        qr = qR_wR_term.0;

        let wR = &qR_wR_term.1;
        b = wR.witness_index() as i32;

        let qO_wO_term = &gate.simplified_fan[2];
        qo = qO_wO_term.0;

        let wO = &qO_wO_term.1;
        c = wO.witness_index() as i32;
    }

    // Add the qc term
    qc = gate.q_C;

    Constraint {
        a,
        b,
        c,
        qm,
        ql,
        qr,
        qo,
        qc,
    }
}
