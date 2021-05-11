#include "defi_interaction_note.hpp"
#include "../../constants.hpp"
#include <crypto/pedersen/pedersen.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace defi_interaction {

grumpkin::g1::affine_element encrypt(defi_interaction_note const& note)
{
    grumpkin::g1::element p_1 = crypto::pedersen::fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(
        note.total_input_value, GeneratorIndex::DEFI_INTERACTION_NOTE_TOTAL_INPUT_VALUE);
    grumpkin::g1::element p_2 = crypto::pedersen::fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(
        note.total_output_a_value, GeneratorIndex::DEFI_INTERACTION_NOTE_TOTAL_OUTPUT_A_VALUE);
    grumpkin::g1::element p_3 = crypto::pedersen::fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(
        note.total_output_b_value, GeneratorIndex::DEFI_INTERACTION_NOTE_TOTAL_OUTPUT_B_VALUE);
    grumpkin::g1::element p_4 =
        crypto::pedersen::fixed_base_scalar_mul<254>(note.bridge_id, GeneratorIndex::DEFI_INTERACTION_NOTE_BRIDGE_ID);

    // input and output values could be zero so we conditionally include its term in the 'sum'
    // bridge_id is always non-zero as it would always contain 'bridge_contract_address'
    // similarly, interaction_nonce can be 0 so we add its term conditionally
    grumpkin::g1::element sum;
    uint32_t include_values = 0;
    if (note.total_input_value > 0) {
        sum = p_1 + p_4;
        include_values++;
    }
    if (note.total_output_a_value > 0) {
        sum += p_2;
        include_values++;
    }
    if (note.total_output_b_value > 0) {
        sum += p_3;
        include_values++;
    }
    if (include_values == 0) {
        sum = p_4;
    }

    grumpkin::g1::element p_5 = crypto::pedersen::fixed_base_scalar_mul<32>(
        (uint64_t)note.interaction_nonce, GeneratorIndex::DEFI_INTERACTION_NOTE_INTERACTION_NONCE);
    if (note.interaction_nonce > 0) {
        sum += p_5;
    }
    if (note.interaction_result) {
        grumpkin::g1::element p_6 = crypto::pedersen::fixed_base_scalar_mul<32>(
            barretenberg::fr(1), GeneratorIndex::DEFI_INTERACTION_NOTE_INTERACTION_RESULT);
        sum += p_6;
    }

    sum = sum.normalize();

    return { sum.x, sum.y };
}

} // namespace defi_interaction
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup