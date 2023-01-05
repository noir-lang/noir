#pragma once
// #include <common/container.hpp>
// #include "oracle_wrapper.hpp"
// #include "private_state_note.hpp"
// #include "private_state_note_preimage.hpp"
// #include "private_state_operand.hpp"

#include "../function_execution_context.hpp"

#include "plonk/composer/turbo_composer.hpp"

#include <common/streams.hpp>
#include <common/map.hpp>
#include <crypto/pedersen/generator_data.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace {
using aztec3::circuits::apps::FunctionExecutionContext;
} // namespace

namespace aztec3::circuits::apps::state_vars {

using crypto::pedersen::generator_index_t;

using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

// Fr: start_slot
//
// mapping(fr => V):
//      level of nesting = 1
//      start_slot_point = start_slot * A
//      at(k1).slot = start_slot_point + k1 * B
//
// mapping(fr => mapping(fr => T)):
//      level_of_nesting = 2
//      start_slot_point becomes: prev_start_slot_point + k1 * B
//      at(k2).slot = new_start_slot_point + k2 * C

template <typename Composer, typename V>
std::tuple<NativeTypes::grumpkin_point, bool> MappingStateVar<Composer, V>::compute_slot_point_at_mapping_key(
    NT::fr const& start_slot, size_t level_of_container_nesting, std::optional<typename NT::fr> const& key)
{
    bool is_partial_slot = false;

    std::vector<std::pair<NativeTypes::fr, generator_index_t>> input_pairs;

    // TODO: compare (in a test) this little calc against calling `compute_start_slot_point`.
    input_pairs.push_back(std::make_pair(
        start_slot,
        generator_index_t({ StorageSlotGeneratorIndex::MAPPING_SLOT, 0 }))); // hash_sub_index 0 is reserved for the

    if (key) {
        input_pairs.push_back(std::make_pair(
            *key, generator_index_t({ StorageSlotGeneratorIndex::MAPPING_SLOT, level_of_container_nesting })));
    } else {
        // If this mapping key has no mapping_key_value (std::nullopt), then we must be partially committing and
        // omitting this mapping key from that partial commitment.
        // So use a placeholder generator for this mapping key, to signify "this mapping key is missing".
        // Note: we can't just commit to a value of `0` for this mapping key, since `0` is a valid value to
        // commit to, and so "missing" is distinguished as follows.
        input_pairs.push_back(std::make_pair(
            NativeTypes::fr(1),
            generator_index_t({ StorageSlotGeneratorIndex::MAPPING_SLOT_PLACEHOLDER, level_of_container_nesting })));
    }

    return std::make_tuple(NativeTypes::commit(input_pairs), is_partial_slot);
}

template <typename Composer, typename V>
std::tuple<typename CircuitTypes<Composer>::grumpkin_point, bool> MappingStateVar<Composer, V>::
    compute_slot_point_at_mapping_key(std::optional<fr> const& key)
{
    bool is_partial_slot = false;

    std::vector<std::pair<fr, generator_index_t>> input_pairs;

    input_pairs.push_back(std::make_pair(this->start_slot,
                                         generator_index_t({ StorageSlotGeneratorIndex::MAPPING_SLOT,
                                                             0 }))); // hash_sub_index 0 is reserved for the start_slot.

    if (key) {
        input_pairs.push_back(std::make_pair(
            *key,
            generator_index_t(
                { StorageSlotGeneratorIndex::MAPPING_SLOT,
                  this->level_of_container_nesting }))); // hash_sub_index 0 is reserved for the start_slot.
    } else {
        // If this mapping key has no mapping_key_value (std::nullopt), then we must be partially committing and
        // omitting this mapping key from that partial commitment.
        // So use a placeholder generator for this mapping key, to signify "this mapping key is missing".
        // Note: we can't just commit to a value of `0` for this mapping key, since `0` is a valid value to
        // commit to, and so "missing" is distinguished as follows.
        input_pairs.push_back(std::make_pair(fr(1),
                                             generator_index_t({ StorageSlotGeneratorIndex::MAPPING_SLOT_PLACEHOLDER,
                                                                 this->level_of_container_nesting })));

        is_partial_slot = true;
    }

    return std::make_tuple(CT::commit(input_pairs), is_partial_slot);
}

template <typename Composer, typename V> V& MappingStateVar<Composer, V>::at(std::optional<fr> const& key)
{
    // First calculate natively and check to see if we've already calculated this state's slot and stored it in the
    // cache, so we don't create unnecessary circuit gates:

    std::optional<NativeTypes::fr> native_key;
    if (!key) {
        native_key = std::nullopt;
    } else {
        native_key = NativeTypes::fr((*key).get_value());
    }

    bool is_partial_slot;
    NativeTypes::grumpkin_point native_new_slot_point;
    std::tie(native_new_slot_point, is_partial_slot) = MappingStateVar<Composer, V>::compute_slot_point_at_mapping_key(
        this->start_slot.get_value(), this->level_of_container_nesting, native_key);
    NativeTypes::fr native_lookup = native_new_slot_point.x;

    // Check cache
    if (this->value_cache.contains(native_lookup)) {
        return this->value_cache[native_lookup];
    }

    // Create gates:
    grumpkin_point new_slot_point;
    std::tie(new_slot_point, is_partial_slot) = compute_slot_point_at_mapping_key(key);
    NativeTypes::fr lookup = new_slot_point.x.get_value();

    if (lookup != native_lookup) {
        throw_or_abort("Expected lookup calcs to be equal!");
    }

    std::string value_name = this->state_var_name + (key ? format("[", *key, "]").c_str() : "[?]");

    V value = V(this->exec_ctx, value_name, new_slot_point, this->level_of_container_nesting + 1, is_partial_slot);

    this->value_cache[lookup] = value;

    return this->value_cache[lookup];
}

}; // namespace aztec3::circuits::apps::state_vars
