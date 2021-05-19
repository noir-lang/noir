#pragma once
#include "compute_circuit_data.hpp"
#include "rollup_circuit.hpp"
#include "rollup_proof_data.hpp"
#include "create_rollup.hpp"
#include <stdlib/types/turbo.hpp>
#include <common/throw_or_abort.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

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

inline bool verify_rollup_logic(rollup_tx& rollup, circuit_data const& circuit_data)
{
#ifndef __wasm__
    try {
#endif
        Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

        pad_rollup_tx(rollup, circuit_data.num_txs, circuit_data.join_split_circuit_data.padding_proof);

        auto recursion_output = rollup_circuit(composer, rollup, circuit_data.verification_keys, circuit_data.num_txs);

        if (composer.failed) {
            throw_or_abort("Circuit logic failed: " + composer.err);
        }

        if (!pairing_check(recursion_output, circuit_data.verification_keys[0])) {
            throw_or_abort("Pairing check failed.");
        }

        return true;
#ifndef __wasm__
    } catch (std::runtime_error const& err) {
        std::cerr << err.what() << std::endl;
        return false;
    }
#endif
}

struct verify_rollup_result {
    bool verified;
    std::vector<uint8_t> proof_data;
};

inline verify_rollup_result verify_rollup(rollup_tx& rollup, circuit_data const& circuit_data)
{
#ifndef __wasm__
    try {
#endif
        if (!circuit_data.proving_key) {
            error("Proving key not provided.");
            return { false, {} };
        }

        if (!circuit_data.verification_key) {
            error("Verification key not provided.");
            return { false, {} };
        }

        if (!circuit_data.join_split_circuit_data.padding_proof.size()) {
            error("Tx padding proof not provided.");
            return { false, {} };
        }

        Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

        pad_rollup_tx(rollup, circuit_data.num_txs, circuit_data.join_split_circuit_data.padding_proof);

        rollup_circuit(composer, rollup, circuit_data.verification_keys, circuit_data.num_txs);

        if (composer.failed) {
            throw_or_abort("Circuit logic failed: " + composer.err);
        }

        auto prover = composer.create_unrolled_prover();
        auto proof = prover.construct_proof();

        auto verifier = composer.create_unrolled_verifier();
        if (!verifier.verify_proof(proof)) {
            throw_or_abort("Proof validation failed.");
        }

        // Pairing check.
        auto data = rollup_proof_data(proof.proof_data);
        auto pairing = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
                           data.recursion_output,
                           circuit_data.verification_keys[0]->reference_string->get_precomputed_g2_lines(),
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

} // namespace rollup
} // namespace proofs
} // namespace rollup
