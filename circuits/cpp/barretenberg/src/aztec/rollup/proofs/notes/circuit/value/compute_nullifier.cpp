#include "compute_nullifier.hpp"
#include "../../constants.hpp"
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
    // Hashing the private key in this way enables the following use case:
    // - A user can demonstrate to a 3rd party that they have spent a note, by providing the hashed_private_key and the
    // note_commitment. The 3rd party can then recalculate the nullifier. This does not reveal the underlying
    // account_private_key to the 3rd party.
    auto hashed_private_key = group_ct::fixed_base_scalar_mul<254>(
        account_private_key, GeneratorIndex::JOIN_SPLIT_NULLIFIER_ACCOUNT_PRIVATE_KEY);

    std::vector<field_ct> hash_inputs{
        note_commitment,
        hashed_private_key.x,
        hashed_private_key.y,
        is_real_note,
    };

    // We compress the hash_inputs with Pedersen, because that's cheaper (constraint-wise) than compressing
    // the data directly with Blake2s in the next step.
    const auto compressed_inputs = pedersen::compress(hash_inputs, true, GeneratorIndex::JOIN_SPLIT_NULLIFIER);

    // Blake2s hash the compressed result. Without this it's possible to leak info from the pedersen compression.
    /** E.g. we can extract a representation of the hashed_pk:
     * Paraphrasing, if:
     *     nullifier = note_comm * G1 + hashed_pk * G2 + is_real_note * G3
     * Then an observer can derive hashed_pk * G2 = nullifier - note_comm * G1 - is_real_note * G3
     * They can derive this for every tx, to link which txs are being sent by the same user.
     * Notably, at the point someone withdraws, the observer would be able to connect `hashed_pk * G2` with a specific
     * eth address.
     */
    auto blake_input = byte_array_ct(compressed_inputs);
    auto blake_result = plonk::stdlib::blake2s(blake_input);
    return field_ct(blake_result);
}

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup