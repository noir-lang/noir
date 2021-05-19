#include "verify.hpp"
#include "root_rollup_circuit.hpp"
#include "create_root_rollup_tx.hpp"
#include "root_rollup_proof_data.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

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

bool verify_internal(Composer& composer, root_rollup_tx& tx, circuit_data const& circuit_data)
{
    if (!circuit_data.inner_rollup_circuit_data.verification_key) {
        error("Inner verification key not provided.");
        return false;
    }

    if (circuit_data.inner_rollup_circuit_data.padding_proof.size() == 0) {
        error("Inner padding proof not provided.");
        return false;
    }

    if (!circuit_data.verifier_crs) {
        error("Verifier crs not provided.");
        return false;
    }

    // Pad the rollup if necessary.
    pad_rollup_tx(tx, circuit_data);

    auto recursion_output = root_rollup_circuit(composer,
                                                tx,
                                                circuit_data.inner_rollup_circuit_data.rollup_size,
                                                circuit_data.rollup_size,
                                                circuit_data.inner_rollup_circuit_data.verification_key);

    if (composer.failed) {
        error("Circuit logic failed: " + composer.err);
        return false;
    }

    if (!pairing_check(recursion_output, circuit_data.verifier_crs)) {
        error("Native pairing check failed.");
        return false;
    }

    return true;
}

bool verify_logic(root_rollup_tx& tx, circuit_data const& circuit_data)
{
    Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);
    return verify_internal(composer, tx, circuit_data);
}

verify_result verify(root_rollup_tx& tx, circuit_data const& circuit_data)
{
    Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

    if (!verify_internal(composer, tx, circuit_data)) {
        return { false, {} };
    }

    auto prover = composer.create_prover();
    auto proof = prover.construct_proof();

    // Pairing check.
    auto data = rollup::root_rollup_proof_data(proof.proof_data);
    auto pairing = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
                       data.recursion_output, circuit_data.verifier_crs->get_precomputed_g2_lines(), 2) ==
                   barretenberg::fq12::one();
    if (!pairing) {
        error("Proof data pairing check failed.");
        return { false, {} };
    }

    auto verifier = composer.create_verifier();
    if (!verifier.verify_proof(proof)) {
        error("Proof validation failed.");
        return { false, {} };
    }

    return { true, proof.proof_data };
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
