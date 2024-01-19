#include "compute_nullifier.hpp"
#include "../../constants.hpp"
#include "barretenberg/join_split_example/types.hpp"

namespace join_split_example::proofs::notes::circuit {

using namespace bb;
using namespace bb::stdlib;

field_ct compute_nullifier(field_ct const& note_commitment,
                           field_ct const& account_private_key,
                           bool_ct const& is_note_in_use)
{
    // Hashing the private key in this way enables the following use case:
    // - A user can demonstrate to a 3rd party that they have spent a note, by providing the hashed_private_key and the
    // note_commitment. The 3rd party can then recalculate the nullifier. This does not reveal the underlying
    // account_private_key to the 3rd party.
    auto hashed_private_key =
        pedersen_commitment::commit({ account_private_key }, GeneratorIndex::JOIN_SPLIT_NULLIFIER_ACCOUNT_PRIVATE_KEY);

    std::vector<field_ct> hash_inputs{
        note_commitment,
        hashed_private_key.x,
        hashed_private_key.y,
        is_note_in_use,
    };

    // We hash the `hash_inputs` with Pedersen, because that's cheaper (constraint-wise) than hashing
    // the data directly with Blake2s in the next step.
    const auto hashed_inputs = pedersen_hash::hash(hash_inputs, GeneratorIndex::JOIN_SPLIT_NULLIFIER);

    // Blake2s hash the pedersen hash's result. Without this it's possible to leak info from the pedersen hash because
    // it is not a random oracle.
    /** E.g. we can extract a representation of the hashed_pk:
     * Paraphrasing, if:
     *     nullifier = note_comm * G1 + hashed_pk * G2 + is_note_in_use * G3
     * Then an observer can derive hashed_pk * G2 = nullifier - note_comm * G1 - is_note_in_use * G3
     * They can derive this for every tx, to link which txs are being sent by the same user.
     * Notably, at the point someone withdraws, the observer would be able to connect `hashed_pk * G2` with a specific
     * eth address.
     */
    auto blake_input = byte_array_ct(hashed_inputs);
    auto blake_result = bb::stdlib::blake2s(blake_input);
    return field_ct(blake_result);
}

} // namespace join_split_example::proofs::notes::circuit
