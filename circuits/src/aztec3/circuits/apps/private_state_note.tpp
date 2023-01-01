// TODO: remove unused inclusions
// #include "private_state_note.hpp"
#include "oracle_wrapper.hpp"
#include "private_state_var.hpp"
#include "private_state_note_preimage.hpp"
#include "plonk/composer/turbo_composer.hpp"

#include <crypto/pedersen/generator_data.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/hash/blake2s/blake2s.hpp>
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::apps {

namespace {
using aztec3::GeneratorIndex;
using crypto::pedersen::generator_index_t;
using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;
} // namespace

template <typename Composer>
PrivateStateNote<Composer>::PrivateStateNote(PrivateStateVar<Composer>& private_state_var,
                                             PrivateStateNotePreimage<CT> preimage,
                                             bool commit_on_init)
    : private_state_var(private_state_var)
    , preimage(preimage)
    , is_partial(check_if_partial())
{
    if (commit_on_init) {
        if (is_partial) {
            partial_commitment = compute_partial_commitment();
        } else {
            commitment = compute_commitment();
        }
    }
}

template <typename Composer> bool PrivateStateNote<Composer>::check_if_partial() const
{
    const auto& [start_slot, storage_slot_point, value, owner, creator_address, salt, nonce, memo, _] = preimage;

    if (!value || !owner || !creator_address || !salt || !nonce || !memo) {
        return true;
    }
    if (private_state_var.is_partial_slot) {
        return true;
    }
    return false;
}

template <typename Composer> typename CircuitTypes<Composer>::fr PrivateStateNote<Composer>::compute_commitment() const
{
    if (commitment.has_value()) {
        return *commitment;
    }

    grumpkin_point storage_slot_point = private_state_var.storage_slot_point;

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

    const auto& [start_slot,
                 mapping_key_values_by_key_name,
                 value,
                 owner,
                 creator_address,
                 salt,
                 nonce,
                 memo,
                 is_real] = preimage;

    const auto commitment_point =
        storage_slot_point +
        CT::commit(
            { gen_pair_fr(value, PrivateStateNoteGeneratorIndex::VALUE),
              gen_pair_address(owner, PrivateStateNoteGeneratorIndex::OWNER),
              gen_pair_address(creator_address, PrivateStateNoteGeneratorIndex::CREATOR),
              gen_pair_fr(salt, PrivateStateNoteGeneratorIndex::SALT),
              gen_pair_fr(nonce, PrivateStateNoteGeneratorIndex::NONCE),
              gen_pair_fr(memo, PrivateStateNoteGeneratorIndex::MEMO),
              std::make_pair(is_real,
                             generator_index_t({ GeneratorIndex::COMMITMENT, PrivateStateNoteGeneratorIndex::IS_REAL }))

            });

    return commitment_point.x;
}

template <typename Composer>
typename CircuitTypes<Composer>::grumpkin_point PrivateStateNote<Composer>::compute_partial_commitment() const
{
    if (partial_commitment.has_value()) {
        info(
            "WARNING: you've already computed a partial commitment for this note. Now, you might have since changed "
            "the preimage and you want to update the partial commitment, and that's ok, so we won't throw an error "
            "here. But if that's not the case, you should call get_partial_commitment() instead, to save constraints.");
    }

    grumpkin_point storage_slot_point = private_state_var.storage_slot_point;

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

    const auto& [start_slot,
                 mapping_key_values_by_key_name,
                 value,
                 owner,
                 creator_address,
                 salt,
                 nonce,
                 memo,
                 is_real] = preimage;

    return storage_slot_point +
           CT::commit({ gen_pair_fr(value, PrivateStateNoteGeneratorIndex::VALUE),
                        gen_pair_address(owner, PrivateStateNoteGeneratorIndex::OWNER),
                        gen_pair_address(creator_address, PrivateStateNoteGeneratorIndex::CREATOR),
                        gen_pair_fr(salt, PrivateStateNoteGeneratorIndex::SALT),
                        gen_pair_fr(nonce, PrivateStateNoteGeneratorIndex::NONCE),
                        gen_pair_fr(memo, PrivateStateNoteGeneratorIndex::MEMO),
                        std::make_pair(
                            is_real,
                            generator_index_t({ GeneratorIndex::COMMITMENT, PrivateStateNoteGeneratorIndex::IS_REAL }))

           });
}

template <typename Composer>
std::pair<typename CircuitTypes<Composer>::fr, NullifierPreimage<CircuitTypes<Composer>>> PrivateStateNote<
    Composer>::compute_nullifier(fr const& owner_private_key)
{
    if (is_partial) {
        throw_or_abort("Can't nullify a partial note.");
    }
    if (!commitment) {
        throw_or_abort("Commitment not yet calculated. Call compute_commitment() or change how you initialise this "
                       "note to include the `commit_on_init` bool.");
    }
    if (nullifier && nullifier_preimage) {
        return std::make_pair(*nullifier, *nullifier_preimage);
    }
    nullifier = PrivateStateNote<Composer>::compute_nullifier(*commitment, owner_private_key, preimage.is_real);
    nullifier_preimage = {
        *commitment,
        owner_private_key,
        preimage.is_real,
    };
    return std::make_pair(*nullifier, *nullifier_preimage);
};

template <typename Composer>
typename CircuitTypes<Composer>::fr PrivateStateNote<Composer>::compute_nullifier(fr const& commitment,
                                                                                  fr const& owner_private_key,
                                                                                  boolean const& is_real_commitment)
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
        is_real_commitment,
    };

    // We compress the hash_inputs with Pedersen, because that's cheaper (constraint-wise) than compressing
    // the data directly with Blake2s in the next step.
    const fr compressed_inputs = CT::compress(hash_inputs, GeneratorIndex::NULLIFIER);

    // Blake2s hash the compressed result. Without this it's possible to leak info from the pedersen
    // compression.
    /** E.g. we can extract a representation of the hashed_pk:
     * Paraphrasing, if:
     *     nullifier = note_comm * G1 + hashed_pk * G2 + is_real_note * G3
     * Then an observer can derive hashed_pk * G2 = nullifier - note_comm * G1 - is_real_note * G3
     * They can derive this for every tx, to link which txs are being sent by the same user.
     * Notably, at the point someone withdraws, the observer would be able to connect `hashed_pk * G2` with a
     * specific eth address.
     */
    auto blake_input = typename CT::byte_array(compressed_inputs);
    auto blake_result = CT::blake2s(blake_input);
    return fr(blake_result);
};

template <typename Composer>
typename CircuitTypes<Composer>::fr PrivateStateNote<Composer>::compute_dummy_nullifier(fr const& dummy_commitment,
                                                                                        fr const& owner_private_key)
{
    return PrivateStateNote<Composer>::compute_nullifier(dummy_commitment, owner_private_key, false);
}

// template class PrivateStateNote<waffle::TurboComposer>;

} // namespace aztec3::circuits::apps