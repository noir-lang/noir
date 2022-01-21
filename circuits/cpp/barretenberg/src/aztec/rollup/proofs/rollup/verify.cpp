#include <common/container.hpp>
#include "verify.hpp"
#include "compute_circuit_data.hpp"
#include "rollup_circuit.hpp"
#include "rollup_proof_data.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

bool pairing_check(recursion_output<bn254> recursion_output,
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

verify_result verify_internal(Composer& composer,
                              rollup_tx& rollup,
                              circuit_data const& circuit_data,
                              bool skip_pairing)
{
    verify_result result = { false, false, "", {}, {} };

    if (!circuit_data.join_split_circuit_data.verification_key) {
        info("Join split verification key not provided.");
        return result;
    }

    if (circuit_data.join_split_circuit_data.padding_proof.size() == 0) {
        info("Join split padding proof not provided.");
        return result;
    }

    if (!circuit_data.verifier_crs) {
        info("Verifier crs not provided.");
        return result;
    }

    pad_rollup_tx(rollup, circuit_data.num_txs, circuit_data.join_split_circuit_data.padding_proof);

    Timer circuit_timer;
    info("Building tx rollup circuit...");
    auto recursion_output = rollup_circuit(composer, rollup, circuit_data.verification_keys, circuit_data.num_txs);
    info("Built tx rollup circuit in: ", circuit_timer.toString());

    if (composer.failed) {
        info("Tx rollup circuit logic failed: " + composer.err);
        result.err = composer.err;
        return result;
    }

    if (!skip_pairing && !pairing_check(recursion_output, circuit_data.verifier_crs)) {
        info("Native pairing check failed.");
        return result;
    }

    result.public_inputs = composer.get_public_inputs();

    result.logic_verified = true;
    return result;
}

verify_result verify_logic(rollup_tx& tx, circuit_data const& circuit_data)
{
    Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);
    return verify_internal(composer, tx, circuit_data, false);
}

verify_result verify_proverless(rollup_tx& tx, circuit_data const& circuit_data)
{
    Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);
    auto result = verify_internal(composer, tx, circuit_data, true);

    if (!result.logic_verified) {
        return result;
    }

    auto pub_input_buf = to_buffer(result.public_inputs);
    result.proof_data = join({ pub_input_buf, slice(circuit_data.padding_proof, pub_input_buf.size()) });
    result.verified = true;
    return result;
}

verify_result verify(rollup_tx& tx, circuit_data const& circuit_data)
{
    Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

    Timer timer;
    auto result = verify_internal(composer, tx, circuit_data, false);

    if (!result.logic_verified) {
        return result;
    }

    Timer proof_timer;
    info("Creating tx rollup proof...");
    circuit_data.proving_key->reset();
    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();
    result.proof_data = proof.proof_data;
    info("Created tx rollup proof in: ", proof_timer.toString());
    info("Time taken: ", timer.toString());

    auto verifier = composer.create_unrolled_verifier();
    result.verified = verifier.verify_proof(proof);

    if (!result.verified) {
        info("Proof validation failed.");
        return result;
    }

    return result;
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
