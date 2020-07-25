#pragma once
#include "../client_proofs/join_split/sign_notes.hpp"
#include "compute_rollup_circuit_data.hpp"
#include "rollup_circuit.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace barretenberg;
using namespace rollup::rollup_proofs;
using namespace plonk::stdlib::types::turbo;

bool pairing_checks(std::vector<recursion_output<bn254>> recursion_outputs,
                    std::shared_ptr<waffle::verification_key> const& inner_verification_key)
{
    auto verified = true;
    for (auto recursion_output : recursion_outputs) {
        g1::affine_element P[2];
        P[0].x = barretenberg::fq(recursion_output.P0.x.get_value().lo);
        P[0].y = barretenberg::fq(recursion_output.P0.y.get_value().lo);
        P[1].x = barretenberg::fq(recursion_output.P1.x.get_value().lo);
        P[1].y = barretenberg::fq(recursion_output.P1.y.get_value().lo);
        barretenberg::fq12 inner_proof_result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
            P, inner_verification_key->reference_string->get_precomputed_g2_lines(), 2);
        verified &= inner_proof_result == barretenberg::fq12::one();
    }
    return verified;
}

bool verify_rollup_logic(rollup_tx const& rollup, rollup_circuit_data const& circuit_data)
{
    try {
        Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

        auto recursion_outputs =
            rollup_circuit(composer, rollup, circuit_data.inner_verification_key, circuit_data.rollup_size);

        if (composer.failed) {
            throw std::runtime_error("Rollup circuit logic failure.");
        }

        if (!pairing_checks(recursion_outputs, circuit_data.inner_verification_key)) {
            throw std::runtime_error("Pairing checks failed.");
        }

        return true;
    } catch (std::runtime_error const& err) {
        std::cerr << err.what() << std::endl;
        return false;
    }
}

struct verify_rollup_result {
    bool verified;
    std::vector<uint8_t> proof_data;
};

verify_rollup_result verify_rollup(rollup_tx const& rollup, rollup_circuit_data const& circuit_data)
{
    try {
        Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

        auto recursion_outputs =
            rollup_circuit(composer, rollup, circuit_data.inner_verification_key, circuit_data.rollup_size);

        if (composer.failed) {
            throw std::runtime_error("Circuit logic failed.");
        }

        auto prover = composer.create_prover();
        auto proof = prover.construct_proof();

        auto verifier = composer.create_verifier();
        if (!verifier.verify_proof(proof)) {
            throw std::runtime_error("Proof validation failed.");
        }

        if (!pairing_checks(recursion_outputs, circuit_data.inner_verification_key)) {
            throw std::runtime_error("Pairing checks failed.");
        }

        return { true, proof.proof_data };
    } catch (std::runtime_error const& err) {
        std::cerr << err.what() << std::endl;
        return { false, {} };
    }
}

} // namespace rollup_proofs
} // namespace rollup
