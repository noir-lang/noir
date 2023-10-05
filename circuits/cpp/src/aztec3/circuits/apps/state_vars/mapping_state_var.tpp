#pragma once

// #include "oracle_wrapper.hpp"
// #include "private_state_note.hpp"
// #include "private_state_note_preimage.hpp"
// #include "private_state_operand.hpp"

#include "../function_execution_context.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {
using aztec3::circuits::apps::FunctionExecutionContext;
}  // namespace

namespace aztec3::circuits::apps::state_vars {

using crypto::generators::generator_index_t;

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

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

template <typename Builder, typename V>
std::tuple<NativeTypes::grumpkin_point, bool> MappingStateVar<Builder, V>::compute_slot_point_at_mapping_key(
    NT::fr const& start_slot, std::optional<typename NT::fr> const& key)
{
    bool const is_partial_slot = false;

    std::vector<NativeTypes::fr> inputs;

    // TODO: compare (in a test) this little calc against calling `compute_start_slot_point`.
    inputs.emplace_back(start_slot);

    if (key) {
        inputs.emplace_back(*key);
    } else {
        // If this mapping key has no mapping_key_value (std::nullopt), then we must be partially committing and
        // omitting this mapping key from that partial commitment.
        // So use a placeholder generator for this mapping key, to signify "this mapping key is missing".
        // Note: we can't just commit to a value of `0` for this mapping key, since `0` is a valid value to
        // commit to, and so "missing" is distinguished as follows.
        inputs.emplace_back(NativeTypes::fr(1));
    }

    return std::make_tuple(NativeTypes::commit(inputs, StorageSlotGeneratorIndex::MAPPING_SLOT), is_partial_slot);
}

template <typename Builder, typename V>
std::tuple<typename CircuitTypes<Builder>::grumpkin_point, bool> MappingStateVar<Builder, V>::
    compute_slot_point_at_mapping_key(std::optional<fr> const& key)
{
    bool is_partial_slot = false;

    std::vector<fr> inputs;

    inputs.push_back(this->start_slot);

    if (key) {
        inputs.push_back(*key);
    } else {
        // If this mapping key has no mapping_key_value (std::nullopt), then we must be partially committing and
        // omitting this mapping key from that partial commitment.
        // So use a placeholder generator for this mapping key, to signify "this mapping key is missing".
        // Note: we can't just commit to a value of `0` for this mapping key, since `0` is a valid value to
        // commit to, and so "missing" is distinguished as follows.
        inputs.push_back(fr(1));
        is_partial_slot = true;
    }

    return std::make_tuple(CT::commit(inputs, StorageSlotGeneratorIndex::MAPPING_SLOT), is_partial_slot);
}

template <typename Builder, typename V> V& MappingStateVar<Builder, V>::at(std::optional<fr> const& key)
{
    // First calculate natively and check to see if we've already calculated this state's slot and stored it in the
    // cache, so we don't create unnecessary circuit gates:

    std::optional<NativeTypes::fr> native_key;
    if (!key) {
        native_key = std::nullopt;
    } else {
        native_key = static_cast<NativeTypes::fr>((*key).get_value());
    }

    bool is_partial_slot = false;
    NativeTypes::grumpkin_point native_new_slot_point;
    std::tie(native_new_slot_point, is_partial_slot) =
        MappingStateVar<Builder, V>::compute_slot_point_at_mapping_key(this->start_slot.get_value(), native_key);
    NativeTypes::fr const native_lookup = native_new_slot_point.x;

    // Check cache
    if (this->value_cache.contains(native_lookup)) {
        return this->value_cache[native_lookup];
    }

    // Create gates:
    grumpkin_point new_slot_point;
    std::tie(new_slot_point, is_partial_slot) = compute_slot_point_at_mapping_key(key);
    NativeTypes::fr const lookup = new_slot_point.x.get_value();

    if (lookup != native_lookup) {
        throw_or_abort("Expected lookup calcs to be equal!");
    }

    std::string const value_name = this->state_var_name + (key ? format("[", *key, "]").c_str() : "[?]");

    V value = V(this->exec_ctx, value_name, new_slot_point, this->level_of_container_nesting + 1, is_partial_slot);

    this->value_cache[lookup] = value;

    return this->value_cache[lookup];
}

};  // namespace aztec3::circuits::apps::state_vars
