use acvm::acir::{circuit::Circuit, native_types::Witness};
use acvm::acir::{circuit::PublicInputs, native_types::Arithmetic};
use ark_ff::Field;
use ark_relations::{
    lc,
    r1cs::{
        ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, SynthesisError, Variable,
    },
};

use ark_bn254::Fr;
pub type Bn254Acir = ACIRCircuit<Fr>;
pub type Bn254AcirGate = ACIRGate<Fr>;

#[derive(Clone)]
pub struct ACIRGate<F: Field> {
    mul_terms: Vec<(F, Witness, Witness)>,
    add_terms: Vec<(F, Witness)>,
    constant_term: F,
}

impl From<(Circuit, Vec<Fr>)> for Bn254Acir {
    fn from(circ_val: (Circuit, Vec<Fr>)) -> Bn254Acir {
        // Currently non-arithmetic gates are not supported
        // so we extract all of the arithmetic gates only
        let circ = circ_val.0;
        let arith_gates: Vec<_> = circ
            .gates
            .into_iter()
            .filter(|gate| gate.is_arithmetic())
            .map(|gate| Bn254AcirGate::from(gate.arithmetic()))
            .collect();

        let values = circ_val.1;

        let num_vars = (circ.current_witness_index + 1) as usize;
        Bn254Acir {
            gates: arith_gates,
            values,
            num_variables: num_vars,
            public_inputs: circ.public_inputs,
        }
    }
}

impl From<Arithmetic> for Bn254AcirGate {
    fn from(arith_gate: Arithmetic) -> Bn254AcirGate {
        let converted_mul_terms: Vec<_> = arith_gate
            .mul_terms
            .into_iter()
            .map(|(coeff, l_var, r_var)| (coeff.into_repr(), l_var, r_var))
            .collect();

        let converted_linear_combinations: Vec<_> = arith_gate
            .linear_combinations
            .into_iter()
            .map(|(coeff, var)| (coeff.into_repr(), var))
            .collect();

        Bn254AcirGate {
            mul_terms: converted_mul_terms,
            add_terms: converted_linear_combinations,
            constant_term: arith_gate.q_c.into_repr(),
        }
    }
}

#[derive(Clone)]
pub struct ACIRCircuit<F: Field> {
    gates: Vec<ACIRGate<F>>,
    public_inputs: PublicInputs,
    values: Vec<F>,
    num_variables: usize,
}

impl<ConstraintF: Field> ConstraintSynthesizer<ConstraintF> for ACIRCircuit<ConstraintF> {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<ConstraintF>,
    ) -> Result<(), SynthesisError> {
        // The first variable is zero in Noir.
        // In PLONK there is no Variable::zero
        // so we offset the witnesses in Noir by one
        let zero = cs.new_witness_variable(|| Ok(ConstraintF::zero()))?;

        let mut variables = vec![zero];

        // First create all of the witness indices by adding the values into the constraint system
        for i in 0..self.num_variables {
            let val = self.values[i];
            let var = if self.public_inputs.contains(i) {
                cs.new_input_variable(|| Ok(val))?
            } else {
                cs.new_witness_variable(|| Ok(val))?
            };

            variables.push(var)
        }

        // Now iterate each gate and add it to the constraint system
        for gate in self.gates {
            let mut arith_gate = LinearCombination::<ConstraintF>::new();

            // Process Mul terms
            for mul_term in gate.mul_terms {
                let coeff = mul_term.0;
                let left_var = variables[mul_term.1.as_usize()];
                let right_var = variables[mul_term.2.as_usize()];

                let left_val = cs.assigned_value(left_var).unwrap();
                let right_val = cs.assigned_value(right_var).unwrap();
                let out_val = left_val * &right_val;

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

            let result = cs.new_lc(arith_gate)?;
            cs.enforce_constraint(lc!() + Variable::One, lc!() + result, lc!() + result)?;
        }

        Ok(())
    }
}
