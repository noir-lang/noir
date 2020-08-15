use super::crs::CRS;
use super::fft::FFT;
use super::pippenger::Pippenger;
use super::prover::Prover;
use super::Barretenberg;
use wasmer_runtime::Value;

// XXX: Notice when we convert i32 to field elements, we use the composer.to_field method
// This is not very nice. Best method would be to call a library which can create the bytes slice for bn128 from numbers
// We most likely will need it anyways, as currently it takes i32, but you could have Field elements and witnesses be more than range(i32)

pub struct StandardComposer {
    barretenberg: Barretenberg,
    pippenger: Pippenger,
    fft: FFT,
    crs: CRS,
    prover: Prover,
}

impl StandardComposer {
    pub fn new(circuit_size: usize) -> StandardComposer {
        let mut barretenberg = Barretenberg::new();

        let crs = CRS::new(circuit_size);

        let pippenger = Pippenger::new(&crs.g1_data, &mut barretenberg);

        let fft = FFT::new(&mut barretenberg, circuit_size);

        let prover = Prover {};

        StandardComposer {
            fft,
            crs,
            barretenberg,
            pippenger,
            prover,
        }
    }
}

#[derive(Clone)]
pub struct Assignments(Vec<i32>);
pub type WitnessAssignments = Assignments;

impl Assignments {
    pub fn to_bytes(&self, composer: &mut StandardComposer) -> Vec<u8> {
        let mut buffer = Vec::new();

        let witness_len = self.0.len() as u32;
        buffer.extend_from_slice(&witness_len.to_be_bytes());

        for assignment in self.0.iter() {
            buffer.extend(composer.to_field(*assignment));
        }

        buffer
    }

    pub fn push(&mut self, value: i32) {
        self.0.push(value);
    }
    pub fn new() -> Assignments {
        Assignments(vec![])
    }
}
#[derive(Clone)]
pub struct Constraint {
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub qm: i32,
    pub ql: i32,
    pub qr: i32,
    pub qo: i32,
    pub qc: i32,
}

impl Constraint {
    fn to_bytes(&self, composer: &mut StandardComposer) -> Vec<u8> {
        let mut buffer = Vec::new();
        // Serialiasing Wires
        buffer.extend_from_slice(&self.a.to_be_bytes());
        buffer.extend_from_slice(&self.b.to_be_bytes());
        buffer.extend_from_slice(&self.c.to_be_bytes());

        // serialise selectors
        buffer.extend(composer.to_field(self.qm));
        buffer.extend(composer.to_field(self.ql));
        buffer.extend(composer.to_field(self.qr));
        buffer.extend(composer.to_field(self.qo));
        buffer.extend(composer.to_field(self.qc));

        buffer
    }
}

#[derive(Clone)]
pub struct ConstraintSystem {
    pub var_num: u32,
    pub pub_var_num: u32,

    pub constraints: Vec<Constraint>,
}

impl ConstraintSystem {
    fn to_bytes(&self, composer: &mut StandardComposer) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        // Push lengths onto the buffer
        buffer.extend_from_slice(&self.var_num.to_be_bytes());
        buffer.extend_from_slice(&self.pub_var_num.to_be_bytes());

        let constraints_len = self.constraints.len() as u32;
        buffer.extend_from_slice(&constraints_len.to_be_bytes());
        // Serialise each arithmetic constraint
        for constraint in self.constraints.iter() {
            buffer.extend(&constraint.to_bytes(composer));
        }

        buffer
    }

    pub fn size(&self) -> usize {

        // XXX: We do not want the user to need to enter the circuit size for a circuit
        // as this is prone to error. For now, we will create a dummy standard composer, which will 
        // call get_circuit_size and then we drop it
        let mut dummy_composer = StandardComposer::new(2);
        dummy_composer.get_circuit_size(&self) as usize
    }
}

impl StandardComposer {
    // XXX: See top of file : This should ideally be done natively
    fn to_field(&mut self, num: i32) -> Vec<u8> {
        if num >= 0 {
            let mut buffer = vec![0; 28];
            buffer.extend_from_slice(&(num as u32).to_be_bytes());
            return buffer;
        }

        self.barretenberg.negate_field(num)
    }

    pub fn get_circuit_size(&mut self, constraint_system: &ConstraintSystem) -> u128 {
        let cs_buf = constraint_system.to_bytes(self);
        let cs_ptr = self.barretenberg.allocate(&cs_buf);

        let circuit_size = self
            .barretenberg
            .call("composer__get_circuit_size", &cs_ptr)
            .value();

        self.barretenberg.free(cs_ptr);

        circuit_size.to_u128()
    }

    pub fn create_proof(
        &mut self,
        constraint_system: &ConstraintSystem,
        witness: WitnessAssignments,
    ) -> Vec<u8> {
        let cs_buf = constraint_system.to_bytes(self);
        let cs_ptr = self.barretenberg.allocate(&cs_buf);

        let witness_buf = witness.to_bytes(self);
        let witness_ptr = self.barretenberg.allocate(&witness_buf);

        let g2_ptr = self.barretenberg.allocate(&self.crs.g2_data);

        let prover_ptr = self
            .barretenberg
            .call_multiple(
                "composer__new_prover",
                vec![&self.pippenger.pointer(), &g2_ptr, &cs_ptr, &witness_ptr],
            )
            .value();

        self.barretenberg.free(cs_ptr);
        self.barretenberg.free(witness_ptr);
        self.barretenberg.free(g2_ptr);

        let proof = self.prover.create_proof(
            &mut self.barretenberg,
            &mut self.pippenger,
            &mut self.fft,
            &prover_ptr,
        );

        self.barretenberg
            .call("composer__delete_prover", &prover_ptr);

        return proof;
    }

    pub fn verify(
        &mut self,
        constraint_system: &ConstraintSystem,
        proof: &[u8],
        public_inputs: Option<Assignments>,
    ) -> bool {
        let cs_buf = constraint_system.to_bytes(self);
        let cs_ptr = self.barretenberg.allocate(&cs_buf);

        let proof_ptr = self.barretenberg.allocate(proof);

        let g2_ptr = self.barretenberg.allocate(&self.crs.g2_data);

        let verified = match public_inputs {
            None => {
                let verified = self
                    .barretenberg
                    .call_multiple(
                        "composer__verify_proof",
                        vec![
                            &self.pippenger.pointer(),
                            &g2_ptr,
                            &cs_ptr,
                            &proof_ptr,
                            &Value::I32(proof.len() as i32),
                        ],
                    )
                    .value();
                verified.clone()
            }
            Some(pub_inputs) => {
                let pub_inputs_buf = pub_inputs.to_bytes(self);
                let pub_inputs_ptr = self.barretenberg.allocate(&pub_inputs_buf);

                let verified = self
                    .barretenberg
                    .call_multiple(
                        "composer__verify_proof_with_public_inputs",
                        vec![
                            &self.pippenger.pointer(),
                            &g2_ptr,
                            &cs_ptr,
                            &pub_inputs_ptr,
                            &proof_ptr,
                            &Value::I32(proof.len() as i32),
                        ],
                    )
                    .value();

                self.barretenberg.free(pub_inputs_ptr);

                verified.clone()
            }
        };
        self.barretenberg.free(cs_ptr);
        self.barretenberg.free(proof_ptr);
        self.barretenberg.free(g2_ptr);
    

        match verified.to_u128() {
            0 => false,
            1 => true,
            _ => panic!("Expected a 1 or a zero for the verification result"),
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test_a_single_constraint() {
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
            constraints: vec![constraint.clone()],
        };

        let case_1 = WitnessResult {
            witness: Assignments(vec![-1, 2, 1]),
            public_inputs: None,
            result: true,
        };
        let case_2 = WitnessResult {
            witness: Assignments(vec![0, 0, 0]),
            public_inputs: None,
            result: true,
        };
        let case_3 = WitnessResult {
            witness: Assignments(vec![10, -3, 7]),
            public_inputs: None,
            result: true,
        };
        let case_4 = WitnessResult {
            witness: Assignments(vec![0, 0, 1]),
            public_inputs: None,
            result: false,
        };
        let case_5 = WitnessResult {
            witness: Assignments(vec![1, 2, 6]),
            public_inputs: None,
            result: false,
        };
        // This should fail as we specified that we do not have any public inputs
        let case_6a = WitnessResult {
            witness: Assignments(vec![1, 2, 6]),
            public_inputs: Some(Assignments(vec![1])),
            result: false,
        };
        // Even if the public input is zero
        let case_6b = WitnessResult {
            witness: Assignments(vec![1, 2, 6]),
            public_inputs: Some(Assignments(vec![0])),
            result: false,
        };

        test_circuit(
            &constraint_system,
            vec![
                case_1.clone(),
                case_2.clone(),
                case_3.clone(),
                case_4.clone(),
                case_5.clone(),
                case_6a,
                case_6b,
            ],
        );

        // Now lets create the same constraint system
        // However, we specify that we want to reserve space for 2 public inputs.
        // Test cases should work the same, even though we supply no public inputs
        // It should also work, if we supply the correct public inputs
        let constraint_system = ConstraintSystem {
            var_num: 3,
            pub_var_num: 2,
            constraints: vec![constraint],
        };

        let case_7 = WitnessResult {
            witness: Assignments(vec![1, 2, 3]),
            public_inputs: Some(Assignments(vec![1, 2])),
            result: true,
        };
        let case_8 = WitnessResult {
            witness: Assignments(vec![1, 2, 3]),
            public_inputs: Some(Assignments(vec![1, 3])),
            result: false,
        };

        test_circuit(
            &constraint_system,
            vec![case_1, case_2, case_3, case_4, case_5, case_7, case_8],
        );
    }

    #[test]
    fn test_multiple_constraints() {
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
        let constraint2 = Constraint {
            a: 2,
            b: 3,
            c: 4,
            qm: 1,
            ql: 0,
            qr: 0,
            qo: -1,
            qc: 1,
        };


        let constraint_system = ConstraintSystem {
            var_num: 4,
            pub_var_num: 1,
            constraints: vec![constraint, constraint2],
        };


        let case_1 = WitnessResult {
            witness: Assignments(vec![1, 1, 2, 3]),
            public_inputs: None,
            result: true,
        };
        let case_2 = WitnessResult {
            witness: Assignments(vec![1, 1, 2, 13]),
            public_inputs: None,
            result: false,
        };

        test_circuit(&constraint_system, vec![case_1, case_2]);
    }

    #[derive(Clone)]
    struct WitnessResult {
        witness: WitnessAssignments,
        public_inputs: Option<Assignments>,
        result: bool,
    }

    fn test_circuit(constraint_system: &ConstraintSystem, test_cases: Vec<WitnessResult>) {
        let mut sc = StandardComposer::new(constraint_system.size());

        for test_case in test_cases.into_iter() {
            let proof = sc.create_proof(&constraint_system, test_case.witness);
            let verified = sc.verify(&constraint_system, &proof, test_case.public_inputs);
            assert_eq!(verified, test_case.result);
        }
    }
}
