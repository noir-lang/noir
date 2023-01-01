#pragma once

#include "oracle_wrapper.hpp"
#include "private_state_note.hpp"
#include "private_state_note_preimage.hpp"
#include "private_state_operand.hpp"

#include <common/streams.hpp>
#include <common/map.hpp>

#include <crypto/pedersen/generator_data.hpp>

#include <plonk/composer/turbo_composer.hpp>

#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::apps {

using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

// Fr: |> start_slot * A
// M1: start_slot * A + mk1 * B |>
// M2: start_slot * A + mk1 * B + mk2 * C |>
template <typename Composer>
typename CircuitTypes<Composer>::grumpkin_point PrivateStateVar<Composer>::compute_start_slot_point()
{
    return CT::commit({ start_slot }, { StorageSlotGeneratorIndex::MAPPING_SLOT });
}

template <typename Composer>
std::tuple<NativeTypes::grumpkin_point, bool> PrivateStateVar<Composer>::compute_slot_point_at_mapping_keys(
    NativeTypes::fr const& start_slot, std::vector<std::optional<NativeTypes::fr>> const& keys)
{
    bool is_partial_slot = false;

    std::vector<std::pair<NativeTypes::fr, generator_index_t>> input_pairs;

    input_pairs.push_back(std::make_pair(start_slot,
                                         generator_index_t({ StorageSlotGeneratorIndex::MAPPING_SLOT,
                                                             0 }))); // hash_sub_index 0 is reserved for the start_slot.

    for (size_t i = 0; i < keys.size(); ++i) {
        if (keys[i]) {
            input_pairs.push_back(
                std::make_pair(*keys[i],
                               generator_index_t({ StorageSlotGeneratorIndex::MAPPING_SLOT,
                                                   i + 1 }))); // hash_sub_index 0 is reserved for the start_slot.
        } else {
            // If this mapping key has no mapping_key_value (std::nullopt), then we must be partially committing and
            // omitting this mapping key from that partial commitment.
            // So use a placeholder generator for this mapping key, to signify "this mapping key is missing".
            // Note: we can't just commit to a value of `0` for this mapping key, since `0` is a valid value to
            // commit to, and so "missing" is distinguished as follows.
            input_pairs.push_back(std::make_pair(
                NativeTypes::fr(1), generator_index_t({ StorageSlotGeneratorIndex::MAPPING_SLOT_PLACEHOLDER, i + 1 })));
        }
    }

    return std::make_tuple(NativeTypes::commit(input_pairs), is_partial_slot);
}

template <typename Composer>
std::tuple<typename CircuitTypes<Composer>::grumpkin_point, bool> PrivateStateVar<
    Composer>::compute_slot_point_at_mapping_keys(std::vector<std::optional<fr>> const& keys)
{
    bool is_partial_slot = false;

    std::vector<std::pair<fr, generator_index_t>> input_pairs;

    input_pairs.push_back(std::make_pair(start_slot,
                                         generator_index_t({ StorageSlotGeneratorIndex::MAPPING_SLOT,
                                                             0 }))); // hash_sub_index 0 is reserved for the start_slot.

    for (size_t i = 0; i < keys.size(); ++i) {
        if (keys[i]) {
            input_pairs.push_back(
                std::make_pair(*keys[i],
                               generator_index_t({ StorageSlotGeneratorIndex::MAPPING_SLOT,
                                                   i + 1 }))); // hash_sub_index 0 is reserved for the start_slot.
        } else {
            // If this mapping key has no mapping_key_value (std::nullopt), then we must be partially committing and
            // omitting this mapping key from that partial commitment.
            // So use a placeholder generator for this mapping key, to signify "this mapping key is missing".
            // Note: we can't just commit to a value of `0` for this mapping key, since `0` is a valid value to
            // commit to, and so "missing" is distinguished as follows.
            input_pairs.push_back(std::make_pair(
                fr(1), generator_index_t({ StorageSlotGeneratorIndex::MAPPING_SLOT_PLACEHOLDER, i + 1 })));
        }
    }

    return std::make_tuple(CT::commit(input_pairs), is_partial_slot);
}

template <typename Composer>
PrivateStateVar<Composer>& PrivateStateVar<Composer>::at(std::vector<std::optional<fr>> const& keys)
{
    if (!is_mapping) {
        throw_or_abort("var is not a mapping - cannot call `at()`");
    }

    if (keys.size() != (*mapping_key_names).size()) {
        throw_or_abort("Need to provide a vector of length equal to this mapping's # keys, even if some are keys are "
                       "intentionally blank (std::optional, for partially-committing");
    }

    // First calculate natively and check to see if we've already calculated this state's slot and stored it in the
    // cache, so we don't create unnecessary circuit gates:
    std::vector<std::optional<NativeTypes::fr>> native_keys;
    for (const auto& key : keys) {
        std::optional<NativeTypes::fr> native_key;
        if (!key) {
            native_key = std::nullopt;
        } else {
            native_key = NativeTypes::fr((*key).get_value());
        }
        native_keys.push_back(native_key);
    }

    bool is_partial_slot;
    NativeTypes::grumpkin_point native_new_slot_point;
    std::tie(native_new_slot_point, is_partial_slot) =
        PrivateStateVar<Composer>::compute_slot_point_at_mapping_keys(start_slot.get_value(), native_keys);
    NativeTypes::fr native_lookup = native_new_slot_point.x;

    // Check cache
    if (private_states.contains(native_lookup)) {
        return private_states[native_lookup];
    }

    // Create gates:
    grumpkin_point new_slot_point;
    std::tie(new_slot_point, is_partial_slot) = compute_slot_point_at_mapping_keys(keys);
    NativeTypes::fr lookup = new_slot_point.x.get_value();

    if (lookup != native_lookup) {
        throw_or_abort("Expected lookup calcs to be equal!");
    }

    PrivateStateVar<Composer> new_state =
        PrivateStateVar<Composer>(exec_ctx, private_state_type, name, start_slot, new_slot_point, is_partial_slot);

    private_states[lookup] = new_state;

    return private_states[lookup];
}

template <typename Composer> std::vector<std::string> PrivateStateVar<Composer>::get_mapping_key_names() const
{
    if (!mapping_key_names) {
        throw_or_abort("Not a mapping.");
    }
    return *mapping_key_names;
}

template <typename Composer>
size_t PrivateStateVar<Composer>::get_index_of_mapping_key_name(std::string const& mapping_key_name) const
{
    if (mapping_key_names) {
        auto begin = (*mapping_key_names).begin();
        auto end = (*mapping_key_names).end();
        auto itr = std::find(begin, end, mapping_key_name);
        if (itr != end) {
            return size_t(itr - begin);
        } else {
            throw_or_abort("mapping key name not found");
        }
    } else {
        throw_or_abort("This private state is not a mapping.");
    }
}

template <typename Composer> void PrivateStateVar<Composer>::arithmetic_checks()
{
    if (private_state_type == WHOLE) {
        throw_or_abort("Code not yet written to support 'whole' private states");
    }

    if (is_partial_slot) {
        throw_or_abort("Arithmetic on a partial state is not supported");
    }

    if (op_count > 0) {
        throw_or_abort("Cannot perform more than one operation on this state.");
    }
    ++op_count;
}

template <typename Composer>
void PrivateStateVar<Composer>::validate_operand(PrivateStateOperand<CT> const& operand) const
{
    if (operand.creator_address) {
        auto& oracle = exec_ctx->oracle;
        const auto& msg_sender = oracle.get_msg_sender();
        (*operand.creator_address).assert_is_in_set({ msg_sender, address(0) });
    }
}

// TODO: move this to a new PrivateState class.
template <typename Composer>
void PrivateStateVar<Composer>::add(PrivateStateOperand<CircuitTypes<Composer>> const& operand)
{
    arithmetic_checks();
    validate_operand(operand);

    auto& oracle = exec_ctx->oracle;
    auto& composer = oracle.composer;

    PrivateStateNotePreimage<CT> new_note_preimage = PrivateStateNotePreimage<CT>{
        .start_slot = start_slot,
        .storage_slot_point = storage_slot_point,
        .value = operand.value,
        .owner = operand.owner,
        .creator_address = operand.creator_address,
        .memo = operand.memo,
        .salt = oracle.generate_salt(),
        // .nonce = // this will be injected by the exec_ctx at 'finalise'
        .is_real = plonk::stdlib::types::to_ct(composer, true),
    };

    auto new_note = PrivateStateNote<Composer>(*this, new_note_preimage);

    exec_ctx->push_new_note(new_note);
}

template <typename Composer>
void PrivateStateVar<Composer>::subtract(PrivateStateOperand<CircuitTypes<Composer>> const& operand)
{
    arithmetic_checks();
    validate_operand(operand);

    // Terminology: difference = minuend - subtrahend

    auto& oracle = exec_ctx->oracle;
    auto& composer = oracle.composer;

    const fr& subtrahend = operand.value;

    auto [minuend_preimage_1, minuend_preimage_2] =
        oracle.get_private_state_note_preimages_for_subtraction(storage_slot_point.x, operand.owner, subtrahend);

    (*minuend_preimage_1.start_slot).assert_equal(start_slot);
    (*minuend_preimage_1.storage_slot_point).assert_equal(storage_slot_point);
    // value enforced through the below subtraction.
    (*minuend_preimage_1.owner).assert_equal(operand.owner);
    // other info about notes being spent is irrelevant.

    (*minuend_preimage_2.start_slot).assert_equal(start_slot);
    (*minuend_preimage_2.storage_slot_point).assert_equal(storage_slot_point);
    // value enforced through the below subtraction.
    (*minuend_preimage_2.owner).assert_equal(operand.owner);
    // other info about notes being spent is irrelevant.

    const fr& minuend = *minuend_preimage_1.value + *minuend_preimage_2.value;

    const fr& difference = minuend - subtrahend; /// TODO: prevent underflow

    auto difference_preimage = PrivateStateNotePreimage<CT>{
        .start_slot = start_slot,
        .storage_slot_point = storage_slot_point,
        .value = difference,
        .owner = operand.owner,
        .creator_address = operand.creator_address,
        .memo = operand.memo,
        .salt = oracle.generate_salt(),
        // .nonce = // this will be injected by the exec_ctx upon `finalise()`
        .is_real = plonk::stdlib::types::to_ct(composer, true),
    };

    auto minuend_note_1 = PrivateStateNote<Composer>(*this, minuend_preimage_1, true);
    auto minuend_note_2 = PrivateStateNote<Composer>(*this, minuend_preimage_2, true);

    fr msg_sender_private_key = oracle.get_msg_sender_private_key();
    auto [minuend_nullifier_1, minuend_nullifier_preimage_1] = minuend_note_1.compute_nullifier(msg_sender_private_key);
    auto [minuend_nullifier_2, minuend_nullifier_preimage_2] = minuend_note_2.compute_nullifier(msg_sender_private_key);

    /// TODO: merkle membership proofs for the two minuend notes.

    auto difference_note = PrivateStateNote<Composer>(*this, difference_preimage);

    exec_ctx->push_new_note(difference_note);
    exec_ctx->push_new_nullifier_data(minuend_nullifier_1, minuend_nullifier_preimage_1);
    exec_ctx->push_new_nullifier_data(minuend_nullifier_2, minuend_nullifier_preimage_2);
}

// template class PrivateStateVar<waffle::TurboComposer>;

}; // namespace aztec3::circuits::apps
