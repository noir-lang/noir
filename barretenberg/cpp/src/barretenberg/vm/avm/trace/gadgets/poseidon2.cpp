#include "barretenberg/vm/avm/trace/gadgets/poseidon2.hpp"
#include "barretenberg/crypto/poseidon2/poseidon2_permutation.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"

namespace bb::avm_trace {

std::vector<AvmPoseidon2TraceBuilder::Poseidon2TraceEntry> AvmPoseidon2TraceBuilder::finalize()
{
    return std::move(poseidon2_trace);
}

void AvmPoseidon2TraceBuilder::reset()
{
    poseidon2_trace.clear();
}

std::array<FF, 4> AvmPoseidon2TraceBuilder::poseidon2_permutation(std::array<FF, 4> const& input,
                                                                  uint32_t clk,
                                                                  uint32_t input_addr,
                                                                  uint32_t output_addr)
{
    // Currently we commit to intermediate round values, changes to codegen might reduce the number of committed polys

    // This is lifted from bb::poeidon2, we need to extract the intermediate round values here.
    using State = std::array<FF, 4>;
    using Poseidon2 = crypto::Poseidon2Permutation<crypto::Poseidon2Bn254ScalarFieldParams>;
    std::array<State, Poseidon2::NUM_ROUNDS> interm_round_vals;
    State current_state(input);

    // Apply 1st linear layer
    Poseidon2::matrix_multiplication_external(current_state);
    std::array<FF, 4> first_ext = current_state;
    // First set of external rounds
    constexpr size_t rounds_f_beginning = Poseidon2::rounds_f / 2;
    for (size_t i = 0; i < rounds_f_beginning; ++i) {
        Poseidon2::add_round_constants(current_state, Poseidon2::round_constants[i]);
        Poseidon2::apply_sbox(current_state);
        Poseidon2::matrix_multiplication_external(current_state);
        // Store end of round state
        interm_round_vals[i] = current_state;
    }

    // Internal rounds
    const size_t p_end = rounds_f_beginning + Poseidon2::rounds_p;
    for (size_t i = rounds_f_beginning; i < p_end; ++i) {
        current_state[0] += Poseidon2::round_constants[i][0];
        Poseidon2::apply_single_sbox(current_state[0]);
        Poseidon2::matrix_multiplication_internal(current_state);
        // Store end of round state
        interm_round_vals[i] = current_state;
    }

    // Remaining external rounds
    for (size_t i = p_end; i < Poseidon2::NUM_ROUNDS; ++i) {
        Poseidon2::add_round_constants(current_state, Poseidon2::round_constants[i]);
        Poseidon2::apply_sbox(current_state);
        Poseidon2::matrix_multiplication_external(current_state);
        // Store end of round state
        interm_round_vals[i] = current_state;
    }

    // Current state is the output
    poseidon2_trace.push_back(
        Poseidon2TraceEntry{ clk, input, current_state, first_ext, interm_round_vals, input_addr, output_addr });

    return current_state;
}

} // namespace bb::avm_trace
