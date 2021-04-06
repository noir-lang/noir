use acir::circuit::Circuit;
use ark_bn254::{Bn254, Fr};
use ark_marlin::Marlin;
use ark_poly::univariate::DensePolynomial;
use ark_poly_commit::marlin_pc::MarlinKZG10;
use ark_serialize::CanonicalSerialize;
use blake2::Blake2s;
use noir_field::FieldElement;
use serialiser::serialise;

pub mod bridge;
pub mod serialiser;

type MultiPC = MarlinKZG10<Bn254, DensePolynomial<Fr>>;
type MarlinInst = Marlin<Fr, MultiPC, Blake2s>;

pub fn prove(acir: Circuit, values: Vec<&FieldElement>) -> Vec<u8> {
    let values: Vec<_> = values.into_iter().copied().map(|x| x.into_repr()).collect();
    let bn254_circ = serialise(acir, values);

    let rng = &mut ark_std::test_rng();

    let universal_srs = MarlinInst::universal_setup(100, 25, 100, rng).unwrap();

    let (index_pk, _) = MarlinInst::index(&universal_srs, bn254_circ.clone()).unwrap();

    let proof = MarlinInst::prove(&index_pk, bn254_circ, rng).unwrap();

    // Serialise proof
    let mut bytes = Vec::new();
    proof.serialize(&mut bytes).unwrap();
    bytes
}

pub fn verify(acir: Circuit, proof: &[u8], public_inputs: Vec<FieldElement>) -> bool {
    todo!()
}
