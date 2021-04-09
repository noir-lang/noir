use acir::circuit::Circuit;
use ark_bn254::{Bn254, Fr};
use ark_ff::Zero;
use ark_marlin::{Marlin, Proof};
use ark_poly::univariate::DensePolynomial;
use ark_poly_commit::marlin_pc::MarlinKZG10;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use blake2::Blake2s;
use noir_field::FieldElement;
use serialiser::serialise;

pub mod bridge;
pub mod serialiser;

type MultiPC = MarlinKZG10<Bn254, DensePolynomial<Fr>>;
type MarlinInst = Marlin<Fr, MultiPC, Blake2s>;
type MarlinBn254Proof = Proof<Fr, MultiPC>;

// Creates a proof using the Marlin proving system
pub fn prove(acir: Circuit, values: Vec<&FieldElement>) -> Vec<u8> {
    let num_vars = acir.num_vars() as usize;
    let num_constraints = compute_num_constraints(&acir);

    // The first variable is zero in Noir.
    // In PLONK there is no Variable::zero
    let values: Vec<_> = std::iter::once(&FieldElement::zero())
        .chain(values.into_iter())
        .copied()
        .map(|x| x.into_repr())
        .collect();

    let bn254_circ = serialise(acir, values);

    // XXX: This should not be used in production
    let rng = &mut ark_std::test_rng();

    let universal_srs = MarlinInst::universal_setup(num_constraints, num_vars, 100, rng).unwrap();

    let (index_pk, _) = MarlinInst::index(&universal_srs, bn254_circ.clone()).unwrap();

    let proof = MarlinInst::prove(&index_pk, bn254_circ, rng).unwrap();

    // Serialise proof
    let mut bytes = Vec::new();
    proof.serialize(&mut bytes).unwrap();
    bytes
}

pub fn verify(acir: Circuit, proof: &[u8], public_inputs: Vec<FieldElement>) -> bool {
    let num_vars = acir.num_vars() as usize;
    let num_constraints = compute_num_constraints(&acir);
    let bn254_circ = serialise(acir, vec![Fr::zero(); num_vars]);

    let rng = &mut ark_std::test_rng();

    let universal_srs = MarlinInst::universal_setup(num_constraints, num_vars, 100, rng).unwrap();

    let (_, index_vk) = MarlinInst::index(&universal_srs, bn254_circ.clone()).unwrap();

    let public_inputs: Vec<_> = public_inputs.into_iter().map(|x| x.into_repr()).collect();
    let proof = MarlinBn254Proof::deserialize(proof).unwrap();
    MarlinInst::verify(&index_vk, &public_inputs, &proof, rng).unwrap()
}

fn compute_num_constraints(acir: &Circuit) -> usize {
    // each multiplication term adds an extra constraint
    let mut num_gates = acir.gates.len();

    for gate in acir.gates.iter() {
        if let acir::circuit::Gate::Arithmetic(arith) = gate {
            num_gates += arith.num_mul_terms() + 1; // plus one for the linear combination gate
        } else {
            unreachable!("currently we do not support non-arithmetic gates")
        }
    }

    num_gates
}

#[test]
fn simple_equal() {
    let a = Witness(1);
    let b = Witness(2);

    // assert a == b
    let arith = Arithmetic {
        mul_terms: vec![],
        linear_combinations: vec![(FieldElement::one(), a), (-FieldElement::one(), b)],
        q_c: FieldElement::zero(),
    };
    let gate = Gate::Arithmetic(arith);
    let circ = Circuit {
        current_witness_index: 2,
        gates: vec![gate],
        public_inputs: PublicInputs(vec![Witness(1)]),
    };
    let a_val = FieldElement::from(6);
    let b_val = FieldElement::from(6);
    let values = vec![&a_val, &b_val];

    let proof = prove(circ.clone(), values);
    let ok = verify(circ.clone(), &proof, vec![a_val]);

    assert!(ok)
}
