#include "compute_nullifier.hpp"
#include "../constants.hpp"
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

field_ct compute_nullifier(field_ct const& note_commitment,
                           field_ct const& account_private_key,
                           bool_ct const& is_real_note)
{
    // We hash the account_private_key to ensure that the result is a field (254 bits).
    auto hashed_pk = group_ct::fixed_base_scalar_mul<254>(account_private_key,
                                                          GeneratorIndex::JOIN_SPLIT_NULLIFIER_ACCOUNT_PRIVATE_KEY);

    std::vector<field_ct> hash_inputs{
        note_commitment,
        hashed_pk.x,
        hashed_pk.y,
        is_real_note,
    };

    const auto result = pedersen::commit(hash_inputs, GeneratorIndex::JOIN_SPLIT_NULLIFIER, true);

    // Blake2s hash the compressed result. Without this it's possible to leak info from the pedersen compression.
    auto blake_input = byte_array_ct(result.x).write(byte_array_ct(result.y));
    auto blake_result = plonk::stdlib::blake2s(blake_input);
    return field_ct(blake_result);
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup