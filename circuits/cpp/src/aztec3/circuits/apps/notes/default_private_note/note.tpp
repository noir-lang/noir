#include "note_preimage.hpp"
#include "nullifier_preimage.hpp"

#include "../note_interface.hpp"

#include "../../oracle_wrapper.hpp"
#include "../../opcodes/opcodes.hpp"

#include "../../state_vars/state_var_base.hpp"

#include <barretenberg/crypto/generators/generator_data.hpp>

#include <barretenberg/plonk/composer/turbo_composer.hpp>

#include <barretenberg/stdlib/hash/pedersen/pedersen.hpp>
#include <barretenberg/stdlib/hash/blake2s/blake2s.hpp>
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace {
using aztec3::circuits::apps::opcodes::Opcodes;
using aztec3::circuits::apps::state_vars::StateVar;
} // namespace

namespace aztec3::circuits::apps::notes {

using aztec3::GeneratorIndex;

using crypto::generators::generator_index_t;

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename Composer, typename V> void DefaultPrivateNote<Composer, V>::remove()
{
    Opcodes<Composer>::UTXO_NULL(state_var, *this);
}

template <typename Composer, typename V> auto& DefaultPrivateNote<Composer, V>::get_oracle()
{
    return state_var->exec_ctx->oracle;
}

template <typename Composer, typename V> bool DefaultPrivateNote<Composer, V>::is_partial_preimage() const
{
    const auto& [value, owner, creator_address, memo, salt, nonce, _] = note_preimage;

    return (!value || !owner || !creator_address || !memo || !salt || !nonce);
}

template <typename Composer, typename V> bool DefaultPrivateNote<Composer, V>::is_partial_storage_slot() const
{
    return state_var->is_partial_slot;
}

template <typename Composer, typename V> bool DefaultPrivateNote<Composer, V>::is_partial() const
{
    return is_partial_preimage() || is_partial_storage_slot();
}

template <typename Composer, typename V>
typename CircuitTypes<Composer>::fr DefaultPrivateNote<Composer, V>::compute_commitment()
{
    if (commitment.has_value()) {
        return *commitment;
    }

    grumpkin_point storage_slot_point = state_var->storage_slot_point;

    std::vector<fr> inputs;
    std::vector<generator_index_t> generators;

    auto gen_pair_address = [&](std::optional<address> const& input, size_t const hash_sub_index) {
        if (!input) {
            throw_or_abort("Cannot commit to a partial preimage. Call compute_partial_commitment instead, or complete "
                           "the preimage.");
        }
        return std::make_pair((*input).to_field(), generator_index_t({ GeneratorIndex::COMMITMENT, hash_sub_index }));
    };

    auto gen_pair_fr = [&](std::optional<fr> const& input, size_t const hash_sub_index) {
        if (!input) {
            throw_or_abort("Cannot commit to a partial preimage. Call compute_partial_commitment instead, or complete "
                           "the preimage.");
        }
        return std::make_pair(*input, generator_index_t({ GeneratorIndex::COMMITMENT, hash_sub_index }));
    };

    if (!note_preimage.salt) {
        note_preimage.salt = get_oracle().generate_random_element();
    }

    const auto& [value, owner, creator_address, memo, salt, nonce, is_dummy] = note_preimage;

    const grumpkin_point commitment_point =
        storage_slot_point +
        CT::commit(
            { gen_pair_fr(value, PrivateStateNoteGeneratorIndex::VALUE),
              gen_pair_address(owner, PrivateStateNoteGeneratorIndex::OWNER),
              gen_pair_address(creator_address, PrivateStateNoteGeneratorIndex::CREATOR),
              gen_pair_fr(memo, PrivateStateNoteGeneratorIndex::MEMO),
              gen_pair_fr(salt, PrivateStateNoteGeneratorIndex::SALT),
              gen_pair_fr(nonce, PrivateStateNoteGeneratorIndex::NONCE),
              std::make_pair(
                  is_dummy, generator_index_t({ GeneratorIndex::COMMITMENT, PrivateStateNoteGeneratorIndex::IS_DUMMY }))

            });

    commitment = commitment_point.x;

    return *commitment;
}

template <typename Composer, typename V>
typename CircuitTypes<Composer>::grumpkin_point DefaultPrivateNote<Composer, V>::compute_partial_commitment()
{
    if (partial_commitment.has_value()) {
        info(
            "WARNING: you've already computed a partial commitment for this note. Now, you might have since changed "
            "the preimage and you want to update the partial commitment, and that's ok, so we won't throw an error "
            "here. But if that's not the case, you should call get_partial_commitment() instead, to save constraints.");
    }

    grumpkin_point storage_slot_point = state_var->storage_slot_point;

    std::vector<fr> inputs;
    std::vector<generator_index_t> generators;

    auto gen_pair_address = [&](std::optional<address> const& input, size_t const hash_sub_index) {
        return input ? std::make_pair((*input).to_field(),
                                      generator_index_t({ GeneratorIndex::COMMITMENT, hash_sub_index }))
                     : std::make_pair(fr(1),
                                      generator_index_t({ GeneratorIndex::COMMITMENT_PLACEHOLDER, hash_sub_index }));
    };

    auto gen_pair_fr = [&](std::optional<fr> const& input, size_t const hash_sub_index) {
        return input ? std::make_pair(*input, generator_index_t({ GeneratorIndex::COMMITMENT, hash_sub_index }))
                     : std::make_pair(fr(1),
                                      generator_index_t({ GeneratorIndex::COMMITMENT_PLACEHOLDER, hash_sub_index }));
    };

    if (!note_preimage.salt) {
        note_preimage.salt = get_oracle().generate_random_element();
    }

    const auto& [value, owner, creator_address, memo, salt, nonce, is_dummy] = note_preimage;

    const grumpkin_point partial_commitment_point =
        storage_slot_point +
        CT::commit(
            { gen_pair_fr(value, PrivateStateNoteGeneratorIndex::VALUE),
              gen_pair_address(owner, PrivateStateNoteGeneratorIndex::OWNER),
              gen_pair_address(creator_address, PrivateStateNoteGeneratorIndex::CREATOR),
              gen_pair_fr(salt, PrivateStateNoteGeneratorIndex::SALT),
              gen_pair_fr(nonce, PrivateStateNoteGeneratorIndex::NONCE),
              gen_pair_fr(memo, PrivateStateNoteGeneratorIndex::MEMO),
              std::make_pair(
                  is_dummy, generator_index_t({ GeneratorIndex::COMMITMENT, PrivateStateNoteGeneratorIndex::IS_DUMMY }))

            });

    partial_commitment = partial_commitment_point;

    return *partial_commitment;
}

template <typename Composer, typename V>
typename CircuitTypes<Composer>::fr DefaultPrivateNote<Composer, V>::compute_nullifier()
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

    fr& owner_private_key = get_oracle().get_msg_sender_private_key();

    nullifier =
        DefaultPrivateNote<Composer, V>::compute_nullifier(*commitment, owner_private_key, note_preimage.is_dummy);
    nullifier_preimage = {
        *commitment,
        owner_private_key,
        note_preimage.is_dummy,
    };
    return *nullifier;
};

template <typename Composer, typename V>
typename CircuitTypes<Composer>::fr DefaultPrivateNote<Composer, V>::compute_dummy_nullifier()
{
    auto& oracle = get_oracle();
    fr dummy_commitment = oracle.generate_random_element();
    fr& owner_private_key = oracle.get_msg_sender_private_key();
    const boolean is_dummy_commitment = true;

    return DefaultPrivateNote<Composer, V>::compute_nullifier(dummy_commitment, owner_private_key, is_dummy_commitment);
};

template <typename Composer, typename V>
typename CircuitTypes<Composer>::fr DefaultPrivateNote<Composer, V>::compute_nullifier(
    fr const& commitment, fr const& owner_private_key, boolean const& is_dummy_commitment)
{
    /**
     * Hashing the private key in this way enables the following use case:
     * - A user can demonstrate to a 3rd party that they have spent a note, by providing the
     hashed_private_key
     * and the note_commitment. The 3rd party can then recalculate the nullifier. This does not reveal the
     * underlying private_key to the 3rd party. */
    const grumpkin_point hashed_private_key = CT::grumpkin_group::template fixed_base_scalar_mul<254>(
        owner_private_key, GeneratorIndex::NULLIFIER_HASHED_PRIVATE_KEY);

    const std::vector<fr> hash_inputs{
        commitment,
        hashed_private_key.x,
        hashed_private_key.y,
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

template <typename Composer, typename V>
void DefaultPrivateNote<Composer, V>::constrain_against_advice(NoteInterface<Composer> const& advice_note)
{
    // Cast from a ref to the base (interface) type to a ref to this derived type:
    const DefaultPrivateNote<Composer, V>& advice_note_ref =
        dynamic_cast<const DefaultPrivateNote<Composer, V>&>(advice_note);

    auto assert_equal = []<typename T>(std::optional<T>& this_member, std::optional<T> const& advice_member) {
        if (advice_member) {
            (*this_member).assert_equal(*advice_member);
        }
    };

    const auto& advice_preimage = advice_note_ref.note_preimage;
    auto& this_preimage = note_preimage;

    assert_equal(this_preimage.value, advice_preimage.value);
    assert_equal(this_preimage.owner, advice_preimage.owner);
    assert_equal(this_preimage.creator_address, advice_preimage.creator_address);
    assert_equal(this_preimage.memo, advice_preimage.memo);
    assert_equal(this_preimage.salt, advice_preimage.salt);
    assert_equal(this_preimage.nonce, advice_preimage.nonce);
}

template <typename Composer, typename V> bool DefaultPrivateNote<Composer, V>::needs_nonce()
{
    return !note_preimage.nonce;
}

template <typename Composer, typename V>
void DefaultPrivateNote<Composer, V>::set_nonce(typename CircuitTypes<Composer>::fr const& nonce)
{
    ASSERT(!note_preimage.nonce);
    note_preimage.nonce = nonce;
};

template <typename Composer, typename V>
typename CircuitTypes<Composer>::fr DefaultPrivateNote<Composer, V>::generate_nonce()
{
    ASSERT(!note_preimage.nonce);
    note_preimage.nonce = compute_dummy_nullifier();
    return *(note_preimage.nonce);
};

} // namespace aztec3::circuits::apps::notes