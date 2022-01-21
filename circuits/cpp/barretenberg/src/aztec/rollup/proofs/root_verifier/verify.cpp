#include <common/container.hpp>
#include "verify.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

bool pairing_check(stdlib::recursion::recursion_output<outer_curve> recursion_output,
                   std::shared_ptr<waffle::VerifierReferenceString> const& srs)
{
    g1::affine_element P[2];
    P[0].x = barretenberg::fq(recursion_output.P0.x.get_value().lo);
    P[0].y = barretenberg::fq(recursion_output.P0.y.get_value().lo);
    P[1].x = barretenberg::fq(recursion_output.P1.x.get_value().lo);
    P[1].y = barretenberg::fq(recursion_output.P1.y.get_value().lo);
    barretenberg::fq12 inner_proof_result =
        barretenberg::pairing::reduced_ate_pairing_batch_precomputed(P, srs->get_precomputed_g2_lines(), 2);
    return inner_proof_result == barretenberg::fq12::one();
}

verify_result verify_internal(OuterComposer& composer,
                              root_verifier_tx& tx,
                              circuit_data const& cd,
                              root_rollup::circuit_data const& root_rollup_cd,
                              bool skip_pairing)
{
    verify_result result = { false, false, {}, {} };

    if (!root_rollup_cd.verification_key) {
        info("Inner verification key not provided.");
        return result;
    }

    if (root_rollup_cd.padding_proof.size() == 0) {
        info("Inner padding proof not provided.");
        return result;
    }

    if (!cd.verifier_crs) {
        info("Verifier crs not provided.");
        return result;
    }

    auto circuit_result = root_verifier_circuit(composer, tx, root_rollup_cd.verification_key, cd.valid_vks);

    result.public_inputs = composer.get_public_inputs();

    if (composer.failed) {
        info("Circuit logic failed: " + composer.err);
        return result;
    }

    if (!skip_pairing && !pairing_check(circuit_result, cd.verifier_crs)) {
        info("Native pairing check failed.");
        return result;
    }

    result.logic_verified = true;
    return result;
}

verify_result verify_logic(root_verifier_tx& tx,
                           circuit_data const& cd,
                           root_rollup::circuit_data const& root_rollup_cd)
{
    OuterComposer composer = OuterComposer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_internal(composer, tx, cd, root_rollup_cd, false);
}

verify_result verify_proverless(root_verifier_tx& tx,
                                circuit_data const& cd,
                                root_rollup::circuit_data const& root_rollup_cd)
{
    OuterComposer composer = OuterComposer(cd.proving_key, cd.verification_key, cd.num_gates);
    auto result = verify_internal(composer, tx, cd, root_rollup_cd, true);

    if (!result.logic_verified) {
        return result;
    }

    auto pub_input_buf = to_buffer(result.public_inputs);
    result.proof_data = join({ pub_input_buf, slice(cd.padding_proof, pub_input_buf.size()) });
    result.verified = true;
    return result;
}

verify_result verify(root_verifier_tx& tx, circuit_data const& cd, root_rollup::circuit_data const& root_rollup_cd)
{
    OuterComposer composer = OuterComposer(cd.proving_key, cd.verification_key, cd.num_gates);

    auto result = verify_internal(composer, tx, cd, root_rollup_cd, false);

    if (!result.logic_verified) {
        return result;
    }

    cd.proving_key->reset();

    auto prover = composer.create_prover();
    auto proof = prover.construct_proof();
    result.proof_data = proof.proof_data;

    auto verifier = composer.create_verifier();
    result.verified = verifier.verify_proof(proof);

    if (!result.verified) {
        info("Proof validation failed.");
        return result;
    }

    return result;
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup