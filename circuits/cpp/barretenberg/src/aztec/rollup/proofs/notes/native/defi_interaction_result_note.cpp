#include "defi_interaction_result_note.hpp"
#include "../constants.hpp"
#include <crypto/pedersen/pedersen.hpp>

// using namespace barretenberg;

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

grumpkin::g1::affine_element encrypt_note(defi_interaction_result_note const& note)
{
    grumpkin::g1::element p_1 =
        crypto::pedersen::fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(note.total_input_value, 0);
    grumpkin::g1::element p_2 =
        crypto::pedersen::fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(note.total_output_a_value, 1);
    grumpkin::g1::element p_3 =
        crypto::pedersen::fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(note.total_output_b_value, 2);
    grumpkin::g1::element p_4 = crypto::pedersen::fixed_base_scalar_mul<254>(note.bridge_id.to_field(), 4);

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

    grumpkin::g1::element p_5 = crypto::pedersen::fixed_base_scalar_mul<32>((uint64_t)note.interaction_nonce, 5);
    if (note.interaction_nonce > 0) {
        sum += p_5;
    }
    sum = sum.normalize();

    return { sum.x, sum.y };
}

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup