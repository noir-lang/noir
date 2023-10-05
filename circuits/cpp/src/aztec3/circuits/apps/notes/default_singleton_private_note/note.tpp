#include "note_preimage.hpp"
#include "nullifier_preimage.hpp"
#include "../../opcodes/opcodes.hpp"
#include "../../oracle_wrapper.hpp"
#include "../../state_vars/state_var_base.hpp"
#include "../note_interface.hpp"

#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {
using aztec3::circuits::apps::opcodes::Opcodes;
using aztec3::circuits::apps::state_vars::StateVar;
}  // namespace

namespace aztec3::circuits::apps::notes {

using aztec3::GeneratorIndex;

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename Builder, typename V> void DefaultSingletonPrivateNote<Builder, V>::remove()
{
    Opcodes<Builder>::UTXO_NULL(state_var, *this);
}

template <typename Builder, typename V> auto& DefaultSingletonPrivateNote<Builder, V>::get_oracle()
{
    return state_var->exec_ctx->oracle;
}

template <typename Builder, typename V> bool DefaultSingletonPrivateNote<Builder, V>::is_partial_preimage() const
{
    const auto& [value, owner, salt, nonce] = note_preimage;

    return (!value || !owner || !salt || !nonce);
}

template <typename Builder, typename V> bool DefaultSingletonPrivateNote<Builder, V>::is_partial_storage_slot() const
{
    return state_var->is_partial_slot;
}

template <typename Builder, typename V> bool DefaultSingletonPrivateNote<Builder, V>::is_partial() const
{
    return is_partial_preimage() || is_partial_storage_slot();
}

template <typename Builder, typename V>
typename CircuitTypes<Builder>::fr DefaultSingletonPrivateNote<Builder, V>::compute_commitment()
{
    if (commitment.has_value()) {
        return *commitment;
    }

    grumpkin_point const storage_slot_point = state_var->storage_slot_point;

    if (!note_preimage.salt) {
        note_preimage.salt = get_oracle().generate_random_element();
    }

    const auto& [value, owner, salt, nonce] = note_preimage;

    const grumpkin_point commitment_point =
        storage_slot_point + CT::commit(
                                 {
                                     *value,              /*PrivateStateNoteGeneratorIndex::VALUE*/
                                     (*owner).to_field(), /*PrivateStateNoteGeneratorIndex::OWNER*/
                                     *salt,               /*PrivateStateNoteGeneratorIndex::SALT*/
                                     *nonce,              /*PrivateStateNoteGeneratorIndex::NONCE*/
                                 },
                                 GeneratorIndex::COMMITMENT);

    commitment = commitment_point.x;

    return *commitment;
}

template <typename Builder, typename V>
typename CircuitTypes<Builder>::fr DefaultSingletonPrivateNote<Builder, V>::compute_nullifier()
{
    if (is_partial()) {
        throw_or_abort("Can't nullify a partial note.");
    }
    if (nullifier && nullifier_preimage) {
        return *nullifier;
    }
    if (!commitment) {
        compute_commitment();
    }

    fr const& owner_private_key = get_oracle().get_msg_sender_private_key();

    nullifier = DefaultSingletonPrivateNote<Builder, V>::compute_nullifier(*commitment, owner_private_key);
    nullifier_preimage = {
        *commitment,
        owner_private_key,
    };
    return *nullifier;
};

template <typename Builder, typename V>
typename CircuitTypes<Builder>::fr DefaultSingletonPrivateNote<Builder, V>::compute_dummy_nullifier()
{
    auto& oracle = get_oracle();
    fr const dummy_commitment = oracle.generate_random_element();
    fr const& owner_private_key = oracle.get_msg_sender_private_key();
    const boolean is_dummy_commitment = true;

    return DefaultSingletonPrivateNote<Builder, V>::compute_nullifier(
        dummy_commitment, owner_private_key, is_dummy_commitment);
};

template <typename Builder, typename V>
typename CircuitTypes<Builder>::fr DefaultSingletonPrivateNote<Builder, V>::compute_nullifier(
    fr const& commitment, fr const& owner_private_key, boolean const& is_dummy_commitment)
{
    const std::vector<fr> hash_inputs{
        commitment,
        owner_private_key,
        is_dummy_commitment,
    };

    // We compress the hash_inputs with Pedersen, because that's cheaper (constraint-wise) than compressing
    // the data directly with Blake2s in the next step.
    const fr compressed_inputs = CT::compress(hash_inputs, GeneratorIndex::NULLIFIER);

    // Blake2s hash the compressed result. Without this it's possible to leak info from the pedersen
    // compression.
    /** E.g. we can extract a representation of the hashed_pk:
     * Paraphrasing, if:
     *     nullifier = note_comm * G1 + hashed_pk * G2 + is_dummy_note * G3
     * Then an observer can derive hashed_pk * G2 = nullifier - note_comm * G1 - is_dummy_note * G3
     * They can derive this for every tx, to link which txs are being sent by the same user.
     * Notably, at the point someone withdraws, the observer would be able to connect `hashed_pk * G2` with a
     * specific eth address.
     */
    auto blake_input = typename CT::byte_array(compressed_inputs);
    auto blake_result = CT::blake2s(blake_input);
    return fr(blake_result);
};

template <typename Builder, typename V>
void DefaultSingletonPrivateNote<Builder, V>::constrain_against_advice(NoteInterface<Builder> const& advice_note)
{
    // Cast from a ref to the base (interface) type to a ref to this derived type:
    const auto& advice_note_ref = dynamic_cast<const DefaultSingletonPrivateNote<Builder, V>&>(advice_note);

    auto assert_equal = []<typename T>(std::optional<T>& this_member, std::optional<T> const& advice_member) {
        if (advice_member) {
            (*this_member).assert_equal(*advice_member);
        }
    };

    const auto& advice_preimage = advice_note_ref.note_preimage;
    auto& this_preimage = note_preimage;

    assert_equal(this_preimage.value, advice_preimage.value);
    assert_equal(this_preimage.owner, advice_preimage.owner);
    assert_equal(this_preimage.salt, advice_preimage.salt);
    assert_equal(this_preimage.nonce, advice_preimage.nonce);
}

template <typename Builder, typename V> bool DefaultSingletonPrivateNote<Builder, V>::needs_nonce()
{
    return !note_preimage.nonce;
}

template <typename Builder, typename V>
void DefaultSingletonPrivateNote<Builder, V>::set_nonce(typename CircuitTypes<Builder>::fr const& nonce)
{
    ASSERT(!note_preimage.nonce);
    note_preimage.nonce = nonce;
};

template <typename Builder, typename V>
typename CircuitTypes<Builder>::fr DefaultSingletonPrivateNote<Builder, V>::generate_nonce()
{
    ASSERT(!note_preimage.nonce);
    note_preimage.nonce = compute_dummy_nullifier();
    return *(note_preimage.nonce);
};

template <typename Builder, typename V>
typename CircuitTypes<Builder>::fr DefaultSingletonPrivateNote<Builder, V>::get_initialisation_nullifier()
{
    auto& oracle = get_oracle();

    const fr& owner_private_key = oracle.get_msg_sender_private_key();

    // We prevent this storage slot from even being initialised again:
    auto& storage_slot_point = state_var->storage_slot_point;

    const std::vector<fr> hash_inputs{
        storage_slot_point.x,
        storage_slot_point.y,
    };

    const bool is_dummy = false;

    // We compress the hash_inputs with Pedersen, because that's cheap.
    const fr compressed_storage_slot_point = CT::compress(hash_inputs, GeneratorIndex::INITIALISATION_NULLIFIER);

    // For now, we piggy-back on the regular nullifier function.
    return DefaultSingletonPrivateNote<Builder, V>::compute_nullifier(
        compressed_storage_slot_point, owner_private_key, is_dummy);
};

template <typename Builder, typename V>
typename CircuitTypes<Builder>::fr DefaultSingletonPrivateNote<Builder, V>::get_initialisation_commitment()
{
    /**
     * TODO: Get rid of this temporary fix of including owner_private_key while computing the initialisation commitment.
     * Details: We need to add the initialisation commitment value to the `nullified_commitments`.
     * In this case, since the actual note data is not yet available, we compute the initialisation nullifier as:
     * null = hash(compressed_storage_slot, owner_private_key, false)
     *
     * Thus, the initialisation commitment here is `compressed_storage_slot`. But since the storage slot is not a real
     * circuit variable, `compressed_storage_slot` would be a circuit constant. The compiler doesn't allow us
     * to make a circuit constant as a public input of the circuit, it just crashes at runtime.
     * To avoid this, we compute the initial commitment as:
     * comm = hash(storage_slot_point.x, storage_slot_point.y, owner_private_key)
     *
     * This makes the initial commitment a "real" circuit variable.
     */
    auto& oracle = get_oracle();

    const fr& owner_private_key = oracle.get_msg_sender_private_key();

    // We prevent this storage slot from even being initialised again:
    auto& storage_slot_point = state_var->storage_slot_point;

    const std::vector<fr> hash_inputs{
        storage_slot_point.x,
        storage_slot_point.y,
        owner_private_key,
    };

    // We compress the hash_inputs with Pedersen, because that's cheap.
    fr compressed_storage_slot_point = CT::compress(hash_inputs, GeneratorIndex::INITIALISATION_NULLIFIER);

    return compressed_storage_slot_point;
};

}  // namespace aztec3::circuits::apps::notes