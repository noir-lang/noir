#include "commit.hpp"
#include "../../constants.hpp"
#include <crypto/pedersen/pedersen.hpp>

using namespace barretenberg;

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace value {

grumpkin::g1::affine_element commit(value_note const& note)
{
    grumpkin::g1::element p_1 = crypto::pedersen::fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(
        note.value, GeneratorIndex::JOIN_SPLIT_NOTE_VALUE);
    grumpkin::g1::element p_2 =
        crypto::pedersen::fixed_base_scalar_mul<254>(note.secret, GeneratorIndex::JOIN_SPLIT_NOTE_SECRET);
    grumpkin::g1::element p_4 =
        crypto::pedersen::fixed_base_scalar_mul<32>((uint64_t)note.asset_id, GeneratorIndex::JOIN_SPLIT_NOTE_ASSET_ID);
    grumpkin::g1::element sum;
    if (note.value > 0) {
        sum = p_1 + p_2;
    } else {
        sum = p_2;
    }
    if (note.asset_id > 0) {
        sum += p_4;
    }
    grumpkin::g1::affine_element p_3 =
        crypto::pedersen::compress_to_point_native(note.owner.x, note.owner.y, GeneratorIndex::JOIN_SPLIT_NOTE_OWNER);

    sum += p_3;
    grumpkin::g1::element p_5 =
        crypto::pedersen::fixed_base_scalar_mul<32>((uint64_t)note.nonce, GeneratorIndex::JOIN_SPLIT_NOTE_NONCE);
    if (note.nonce > 0) {
        sum += p_5;
    }
    sum = sum.normalize();

    return { sum.x, sum.y };
}

} // namespace value
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup