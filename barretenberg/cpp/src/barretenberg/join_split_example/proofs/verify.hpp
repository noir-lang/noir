#pragma once
#include "./mock/mock_circuit.hpp"
#include "barretenberg/ecc/curves/bn254/fq12.hpp"
#include "barretenberg/ecc/curves/bn254/pairing.hpp"
#include "barretenberg/stdlib/recursion/aggregation_state/aggregation_state.hpp"
#include "barretenberg/stdlib/recursion/verifier/verifier.hpp"

namespace join_split_example {
namespace proofs {

template <typename Composer> struct verify_result {
    verify_result()
        : logic_verified(false)
        , verified(false)
    {}

    bool logic_verified;
    std::string err;
    std::vector<fr> public_inputs;
    stdlib::recursion::aggregation_state<stdlib::bn254<Composer>> aggregation_state;

    std::vector<uint8_t> proof_data;
    bool verified;
    std::shared_ptr<plonk::verification_key> verification_key;
    size_t number_of_gates;
};

template <typename Composer>
inline bool pairing_check(stdlib::recursion::aggregation_state<stdlib::bn254<Composer>> aggregation_state,
                          std::shared_ptr<bb::srs::factories::VerifierCrs> const& srs)
{
    g1::affine_element P[2];
    P[0].x = bb::fq(aggregation_state.P0.x.get_value().lo);
    P[0].y = bb::fq(aggregation_state.P0.y.get_value().lo);
    P[1].x = bb::fq(aggregation_state.P1.x.get_value().lo);
    P[1].y = bb::fq(aggregation_state.P1.y.get_value().lo);
    bb::fq12 inner_proof_result =
        bb::pairing::reduced_ate_pairing_batch_precomputed(P, srs->get_precomputed_g2_lines(), 2);
    return inner_proof_result == bb::fq12::one();
}

template <typename Builder, typename Tx, typename CircuitData, typename F>
auto verify_logic_internal(Builder& builder, Tx& tx, CircuitData const& cd, char const* name, F const& build_circuit)
{
    info(name, ": Building circuit...");
    Timer timer;
    auto result = build_circuit(builder, tx, cd);
    info(name, ": Circuit built in ", timer.toString(), "s");

    if (builder.failed()) {
        info(name, ": Circuit logic failed: " + builder.err());
        result.err = builder.err();
        return result;
    }

    if (!cd.srs) {
        info(name, ": Srs not provided.");
        return result;
    }

    if (!pairing_check(result.aggregation_state, cd.srs->get_verifier_crs())) {
        info(name, ": Native pairing check failed.");
        return result;
    }

    result.public_inputs = builder.get_public_inputs();
    result.logic_verified = true;
    result.number_of_gates = builder.get_num_gates();

    return result;
}

} // namespace proofs
} // namespace join_split_example
