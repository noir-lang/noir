#pragma once
#include "./mock/mock_circuit.hpp"
#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include <stdlib/recursion/verifier/verifier.hpp>

namespace rollup {
namespace proofs {

template <typename Composer> struct verify_result {
    verify_result()
        : logic_verified(false)
        , verified(false)
    {}

    bool logic_verified;
    std::string err;
    std::vector<fr> public_inputs;
    plonk::stdlib::recursion::recursion_output<plonk::stdlib::bn254<Composer>> recursion_output;

    std::vector<uint8_t> proof_data;
    bool verified;
};

template <typename Composer>
inline bool pairing_check(plonk::stdlib::recursion::recursion_output<plonk::stdlib::bn254<Composer>> recursion_output,
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

template <typename Composer, typename Tx, typename CircuitData, typename F>
auto verify_logic_internal(Composer& composer, Tx& tx, CircuitData const& cd, char const* name, F const& build_circuit)
{
    info(name, ": Building circuit...");
    Timer timer;
    auto result = build_circuit(composer, tx, cd);
    info(name, ": Circuit built in ", timer.toString(), "s");

    if (composer.failed) {
        info(name, ": Circuit logic failed: " + composer.err);
        result.err = composer.err;
        return result;
    }

    if (!cd.srs) {
        info(name, ": Srs not provided.");
        return result;
    }

    if (!pairing_check(result.recursion_output, cd.srs->get_verifier_crs())) {
        info(name, ": Native pairing check failed.");
        return result;
    }

    result.public_inputs = composer.get_public_inputs();
    result.logic_verified = true;

    return result;
}

template <typename Composer, typename Tx, typename CircuitData, typename F>
auto verify_internal(
    Composer& composer, Tx& tx, CircuitData const& cd, char const* name, bool unrolled, F const& build_circuit)
{
    Timer timer;
    auto result = verify_logic_internal(composer, tx, cd, name, build_circuit);

    if (!result.logic_verified) {
        return result;
    }

    cd.proving_key->reset();

    Timer proof_timer;
    info(name, ": Creating proof...");

    if (!cd.mock) {
        if (unrolled) {
            auto prover = composer.create_unrolled_prover();
            auto proof = prover.construct_proof();
            result.proof_data = proof.proof_data;
        } else {
            auto prover = composer.create_prover();
            auto proof = prover.construct_proof();
            result.proof_data = proof.proof_data;
        }
    } else {
        Composer mock_proof_composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
        ::rollup::proofs::mock::mock_circuit(mock_proof_composer, composer.get_public_inputs());
        if (unrolled) {
            auto prover = mock_proof_composer.create_unrolled_prover();
            auto proof = prover.construct_proof();
            result.proof_data = proof.proof_data;
        } else {
            auto prover = mock_proof_composer.create_prover();
            auto proof = prover.construct_proof();
            result.proof_data = proof.proof_data;
        }
    }

    info(name, ": Proof created in ", proof_timer.toString(), "s");
    info(name, ": Total time taken: ", timer.toString(), "s");

    if (unrolled) {
        auto verifier = composer.create_unrolled_verifier();
        result.verified = verifier.verify_proof({ result.proof_data });
    } else {
        auto verifier = composer.create_verifier();
        result.verified = verifier.verify_proof({ result.proof_data });
    }

    if (!result.verified) {
        info(name, ": Proof validation failed.");
        return result;
    } else {
        info(name, ": Verified successfully.");
    }

    return result;
}

} // namespace proofs
} // namespace rollup
