#include "compute_nullifier.hpp"
#include "../constants.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

field_ct compute_nullifier(point_ct const& encrypted_note,
                           field_ct const& tree_index,
                           field_ct const& account_private_key,
                           bool_ct const& is_real_note)
{
    // modified_index = tree_index plus a modifier to indicate whether the note is a real note or a virtual note (i.e.
    // value 0 and not a member of the tree) For virtual notes, we set the 65'th bit of modified_index to be true (this
    // cannot overlap with tree index, which we range constrain to be 32 bits)
    barretenberg::fr shift = uint256_t(1) << 64;
    field_ct modified_index = (tree_index + (static_cast<field_ct>(is_real_note) * shift)).normalize();

    // We hash the account_private_key to ensure that the result is a field (254 bits).
    auto hashed_pk = group_ct::fixed_base_scalar_mul<254>(account_private_key, TX_NOTE_ACCOUNT_PRIVATE_KEY_INDEX);

    std::vector<field_ct> hash_inputs{
        encrypted_note.x,
        hashed_pk.x,
        hashed_pk.y,
        modified_index,
    };

    const auto result = pedersen::encrypt(hash_inputs, TX_NOTE_NULLIFIER_INDEX, true);

    // Blake2s hash the compressed result. Without this it's possible to leak info from the pedersen compression.
    auto blake_input = byte_array_ct(result.x).write(byte_array_ct(result.y));
    auto blake_result = plonk::stdlib::blake2s(blake_input);
    return field_ct(blake_result);
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup