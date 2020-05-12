#pragma once
#include "../tx/user_context.hpp"
#include "../client_proofs/join_split/sign_notes.hpp"
#include "compute_rollup_circuit_data.hpp"
#include "create_noop_join_split_proof.hpp"
#include "rollup_circuit.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace barretenberg;
using namespace rollup::rollup_proofs;
using namespace plonk::stdlib::types::turbo;

bool verify_rollup(rollup_tx const& rollup, rollup_circuit_data const& circuit_data)
{
    Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);

    auto recursion_outputs =
        rollup_circuit(composer, rollup, circuit_data.inner_verification_key, circuit_data.rollup_size);

    auto prover = composer.create_prover();
    auto proof = prover.construct_proof();

    // std::cout << proof.proof_data << std::endl;

    auto verifier = composer.create_verifier();
    auto verified = verifier.verify_proof(proof);

    for (auto recursion_output : recursion_outputs) {
        g1::affine_element P[2];
        P[0].x = barretenberg::fq(recursion_output.P0.x.get_value().lo);
        P[0].y = barretenberg::fq(recursion_output.P0.y.get_value().lo);
        P[1].x = barretenberg::fq(recursion_output.P1.x.get_value().lo);
        P[1].y = barretenberg::fq(recursion_output.P1.y.get_value().lo);
        barretenberg::fq12 inner_proof_result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
            P, circuit_data.inner_verification_key->reference_string->get_precomputed_g2_lines(), 2);
        verified &= inner_proof_result == barretenberg::fq12::one();
    }

    // EXPECT_EQ(circuit_output.recursion_output.public_inputs[0].get_value(), inner_inputs[0]);
    // EXPECT_EQ(circuit_output.recursion_output.public_inputs[1].get_value(), inner_inputs[1]);

    return verified;
}

} // namespace rollup_proofs
} // namespace rollup
