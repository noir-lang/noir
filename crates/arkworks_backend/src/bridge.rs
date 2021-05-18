use acir::circuit::PublicInputs;
use acir::native_types::Witness;
use ark_ff::Field;
use ark_relations::{
    lc,
    r1cs::{
        ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, SynthesisError, Variable,
    },
};
// AcirCircuit and AcirArithGate are structs that arkworks can synthesise.
//
// The difference between these structures and the ACIR structure that the compiler uses is the following:
// - The compilers ACIR struct is currently fixed to bn254
// - These structures only support arithmetic gates, while the compiler has other
// gate types. These can be added later once the backend knows how to deal with things like XOR
// or once ACIR is taught how to do convert these black box functions to Arithmetic gates.
//
// XXX: Ideally we want to implement `ConstraintSynthesizer` on ACIR however
// this does not seem possible since ACIR is juts a description of the constraint system and the API Asks for prover values also.
//
// Perfect API would look like:
// - index(srs, circ)
// - prove(index_pk, prover_values, rng)
// - verify(index_vk, verifier, rng)
#[derive(Clone)]
pub struct AcirCircuit<F: Field> {
    pub(crate) gates: Vec<AcirArithGate<F>>,
    pub(crate) public_inputs: PublicInputs,
    pub(crate) values: Vec<F>,
    pub(crate) num_variables: usize,
}

#[derive(Clone, Debug)]
pub struct AcirArithGate<F: Field> {
    pub(crate) mul_terms: Vec<(F, Witness, Witness)>,
    pub(crate) add_terms: Vec<(F, Witness)>,
    pub(crate) constant_term: F,
}

impl<ConstraintF: Field> ConstraintSynthesizer<ConstraintF> for AcirCircuit<ConstraintF> {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<ConstraintF>,
    ) -> Result<(), SynthesisError> {
        let mut variables = Vec::with_capacity(self.values.len());

        // First create all of the witness indices by adding the values into the constraint system
        for (i, val) in self.values.iter().enumerate() {
            let var = if self.public_inputs.contains(i) {
                cs.new_input_variable(|| Ok(*val))?
            } else {
                cs.new_witness_variable(|| Ok(*val))?
            };

            variables.push(var);
        }

        // Now iterate each gate and add it to the constraint system
        for gate in self.gates {
            let mut arith_gate = LinearCombination::<ConstraintF>::new();

            // Process mul terms
            for mul_term in gate.mul_terms {
                let coeff = mul_term.0;
                let left_val = self.values[mul_term.1.as_usize()];
                let right_val = self.values[mul_term.2.as_usize()];

                let out_val = left_val * right_val;

                let out_var = cs.new_witness_variable(|| Ok(out_val))?;
                arith_gate += (coeff, out_var);
            }

            // Process Add terms
            for add_term in gate.add_terms {
                let coeff = add_term.0;
                let add_var = variables[add_term.1.as_usize()];
                arith_gate += (coeff, add_var)
            }

            // Process constant term
            arith_gate += (gate.constant_term, Variable::One);

            cs.enforce_constraint(lc!() + Variable::One, arith_gate, lc!())?;
        }

        Ok(())
    }
}
