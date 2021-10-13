#include "verify.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

bool pairing_check(stdlib::recursion::recursion_output<outer_curve> recursion_output,
                   std::shared_ptr<waffle::verification_key> const& vk)
{
    g1::affine_element P[2];
    P[0].x = barretenberg::fq(recursion_output.P0.x.get_value().lo);
    P[0].y = barretenberg::fq(recursion_output.P0.y.get_value().lo);
    P[1].x = barretenberg::fq(recursion_output.P1.x.get_value().lo);
    P[1].y = barretenberg::fq(recursion_output.P1.y.get_value().lo);
    barretenberg::fq12 inner_proof_result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
        P, vk->reference_string->get_precomputed_g2_lines(), 2);
    return inner_proof_result == barretenberg::fq12::one();
}

verify_result verify(root_verifier_tx& tx, circuit_data const& circuit_data)
{
    OuterComposer composer =
        OuterComposer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

    root_verifier_circuit(composer, tx, circuit_data.root_rollup_circuit_data.verification_key);

    if (composer.failed) {
        throw_or_abort("Circuit logic failed: " + composer.err);
    }

    auto prover = composer.create_prover();
    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier();

    verify_result result({ .verified = true, .proof_data = proof.proof_data });

    if (!verifier.verify_proof(proof)) {
        info("Proof validation failed.");
        result.verified = false;
        return result;
    }

    // Pairing check.
    auto data = root_verifier_proof_data(proof.proof_data);
    auto pairing =
        barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
            data.recursion_output, circuit_data.verification_key->reference_string->get_precomputed_g2_lines(), 2) ==
        barretenberg::fq12::one();
    if (!pairing) {
        info("Pairing check failed.");
        result.verified = false;
    }

    return result;
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup