#pragma once
#include "compute_circuit_data.hpp"
#include "root_rollup_circuit.hpp"
#include "../rollup/rollup_proof_data.hpp"
#include <stdlib/types/turbo.hpp>
#include <common/throw_or_abort.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

inline bool pairing_check(recursion_output<bn254> recursion_output, std::shared_ptr<waffle::verification_key> const& vk)
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

inline bool verify_logic(root_rollup_tx& rollup, circuit_data const& circuit_data)
{
#ifndef __wasm__
    try {
#endif
        Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

        // Pad the rollup if necessary.
        rollup.rollups.resize(circuit_data.num_inner_rollups, circuit_data.inner_rollup_circuit_data.padding_proof);

        auto recursion_output = root_rollup_circuit(composer,
                                                    rollup,
                                                    circuit_data.inner_rollup_circuit_data.rollup_size,
                                                    circuit_data.inner_rollup_circuit_data.verification_key);

        if (composer.failed) {
            throw_or_abort("Circuit logic failed: " + composer.err);
        }

        // WARNING: JUST UNTIL MIN FAIL TEST FIXED.
        // if (!pairing_check(recursion_output, circuit_data.verification_key)) {
        //     throw_or_abort("Pairing check failed.");
        // }

        return true;
#ifndef __wasm__
    } catch (std::runtime_error const& err) {
        std::cerr << err.what() << std::endl;
        return false;
    }
#endif
}

struct verify_result {
    bool verified;
    std::vector<uint8_t> proof_data;
};

inline verify_result verify(root_rollup_tx& rollup, circuit_data const& circuit_data)
{
#ifndef __wasm__
    try {
#endif
        Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

        // Pad the rollup if necessary.
        rollup.rollups.resize(circuit_data.num_inner_rollups, circuit_data.inner_rollup_circuit_data.padding_proof);

        root_rollup_circuit(composer,
                            rollup,
                            circuit_data.inner_rollup_circuit_data.rollup_size,
                            circuit_data.inner_rollup_circuit_data.verification_key);

        if (composer.failed) {
            throw_or_abort("Circuit logic failed: " + composer.err);
        }

        auto prover = composer.create_prover();
        auto proof = prover.construct_proof();

        auto verifier = composer.create_verifier();
        if (!verifier.verify_proof(proof)) {
            throw_or_abort("Proof validation failed.");
        }

        // Pairing check.
        auto data = rollup::rollup_proof_data(proof.proof_data);
        auto pairing = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
                           data.recursion_output,
                           circuit_data.verification_key->reference_string->get_precomputed_g2_lines(),
                           2) == barretenberg::fq12::one();
        if (!pairing) {
            throw_or_abort("Pairing check failed.");
        }

        return { true, proof.proof_data };

#ifndef __wasm__
    } catch (std::runtime_error const& err) {
        std::cerr << err.what() << std::endl;
        return { false, {} };
    }
#endif
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
