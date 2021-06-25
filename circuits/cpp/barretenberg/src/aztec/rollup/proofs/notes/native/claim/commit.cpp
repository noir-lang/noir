#include "claim_note.hpp"
#include "../../constants.hpp"
#include <crypto/pedersen/pedersen.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

grumpkin::g1::affine_element commit(claim_note const& note)
{
    grumpkin::g1::element p_1 = crypto::pedersen::fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(
        note.deposit_value, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_VALUE);
    grumpkin::g1::element p_2 =
        crypto::pedersen::fixed_base_scalar_mul<254>(note.bridge_id, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_BRIDGE_ID);

    // deposit value could be zero so we conditionally include its term in the 'sum'
    // bridge_id is always non-zero as it would always contain 'bridge_contract_address'
    // similarly, defi_interaction_nonce can be 0 so we add its term conditionally
    grumpkin::g1::element sum;
    if (note.deposit_value > 0) {
        sum = p_1 + p_2;
    } else {
        sum = p_2;
    }

    grumpkin::g1::affine_element p_3 = crypto::pedersen::compress_to_point_native(
        note.partial_state.x, note.partial_state.y, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_PARTIAL_STATE);
    sum += p_3;

    grumpkin::g1::element p_4 = crypto::pedersen::fixed_base_scalar_mul<32>(
        (uint64_t)note.defi_interaction_nonce, GeneratorIndex::JOIN_SPLIT_CLAIM_NOTE_DEFI_INTERACTION_NONCE);
    if (note.defi_interaction_nonce > 0) {
        sum += p_4;
    }
    sum = sum.normalize();

    return { sum.x, sum.y };
}

grumpkin::g1::affine_element create_partial_value_note(barretenberg::fr const& secret,
                                                       grumpkin::g1::affine_element const& owner,
                                                       uint32_t nonce)
{
    grumpkin::g1::element sum =
        crypto::pedersen::fixed_base_scalar_mul<254>(secret, GeneratorIndex::JOIN_SPLIT_NOTE_SECRET);
    sum += crypto::pedersen::compress_to_point_native(owner.x, owner.y, GeneratorIndex::JOIN_SPLIT_NOTE_OWNER);

    if (nonce > 0) {
        sum += crypto::pedersen::fixed_base_scalar_mul<32>((uint64_t)nonce, GeneratorIndex::JOIN_SPLIT_NOTE_NONCE);
    }
    sum = sum.normalize();

    return { sum.x, sum.y };
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup