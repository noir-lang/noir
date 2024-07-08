#pragma once

#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/vm/avm_trace/constants.hpp"

#include "barretenberg/vm/generated/avm_flavor_settings.hpp"
#include "barretenberg/vm/generated/avm_full_row.hpp"

#include <array>
#include <cstdint>
#include <map>
#include <unordered_map>

namespace bb::avm_trace {

using FF = AvmFlavorSettings::FF;

// To toggle all relevant unit tests with proving, set the env variable "AVM_ENABLE_FULL_PROVING".
static const bool ENABLE_PROVING = std::getenv("AVM_ENABLE_FULL_PROVING") != nullptr;

// There are 4 public input columns, 1 for context inputs, and 3 for emitting side effects
using VmPublicInputs = std::tuple<std::array<FF, KERNEL_INPUTS_LENGTH>,   // Input: Kernel context inputs
                                  std::array<FF, KERNEL_OUTPUTS_LENGTH>,  // Output: Kernel outputs data
                                  std::array<FF, KERNEL_OUTPUTS_LENGTH>,  // Output: Kernel outputs side effects
                                  std::array<FF, KERNEL_OUTPUTS_LENGTH>>; // Output: Kernel outputs metadata
// Constants for indexing into the tuple above
static const size_t KERNEL_INPUTS = 0;
static const size_t KERNEL_OUTPUTS_VALUE = 1;
static const size_t KERNEL_OUTPUTS_SIDE_EFFECT_COUNTER = 2;
static const size_t KERNEL_OUTPUTS_METADATA = 3;

// Number of rows
static const size_t AVM_TRACE_SIZE = 1 << 18;
enum class IntermRegister : uint32_t { IA = 0, IB = 1, IC = 2, ID = 3 };
enum class IndirectRegister : uint32_t { IND_A = 0, IND_B = 1, IND_C = 2, IND_D = 3 };

// Keep following enum in sync with MAX_MEM_TAG below
enum class AvmMemoryTag : uint32_t { U0 = 0, U8 = 1, U16 = 2, U32 = 3, U64 = 4, U128 = 5, FF = 6 };
static const uint32_t MAX_MEM_TAG = 6;

static const size_t NUM_MEM_SPACES = 256;
static const uint8_t INTERNAL_CALL_SPACE_ID = 255;
static const uint32_t MAX_SIZE_INTERNAL_STACK = 1 << 16;

struct ExternalCallHint {
    FF success;
    std::vector<FF> return_data;
    uint32_t l2_gas_used;
    uint32_t da_gas_used;
    FF end_side_effect_counter;
};

// Add support for deserialization of ExternalCallHint. This is implicitly used by serialize::read
// when trying to read std::vector<ExternalCallHint>.
inline void read(uint8_t const*& it, ExternalCallHint& hint)
{
    using serialize::read;
    read(it, hint.success);
    read(it, hint.return_data);
    read(it, hint.l2_gas_used);
    read(it, hint.da_gas_used);
    read(it, hint.end_side_effect_counter);
}

struct ContractInstanceHint {
    FF address;
    FF instance_found_in_address;
    FF salt;
    FF deployer_addr;
    FF contract_class_id;
    FF initialisation_hash;
    FF public_key_hash;
};

// Add support for deserialization of ContractInstanceHint.
inline void read(uint8_t const*& it, ContractInstanceHint& hint)
{
    using serialize::read;
    read(it, hint.address);
    read(it, hint.instance_found_in_address);
    read(it, hint.salt);
    read(it, hint.deployer_addr);
    read(it, hint.contract_class_id);
    read(it, hint.initialisation_hash);
    read(it, hint.public_key_hash);
}

struct ExecutionHints {
    std::vector<std::pair<FF, FF>> storage_value_hints;
    std::vector<std::pair<FF, FF>> note_hash_exists_hints;
    std::vector<std::pair<FF, FF>> nullifier_exists_hints;
    std::vector<std::pair<FF, FF>> l1_to_l2_message_exists_hints;
    std::vector<ExternalCallHint> externalcall_hints;
    std::map<FF, ContractInstanceHint> contract_instance_hints;

    ExecutionHints() = default;

    // Builder.
    ExecutionHints& with_storage_value_hints(std::vector<std::pair<FF, FF>> storage_value_hints)
    {
        this->storage_value_hints = std::move(storage_value_hints);
        return *this;
    }
    ExecutionHints& with_note_hash_exists_hints(std::vector<std::pair<FF, FF>> note_hash_exists_hints)
    {
        this->note_hash_exists_hints = std::move(note_hash_exists_hints);
        return *this;
    }
    ExecutionHints& with_nullifier_exists_hints(std::vector<std::pair<FF, FF>> nullifier_exists_hints)
    {
        this->nullifier_exists_hints = std::move(nullifier_exists_hints);
        return *this;
    }
    ExecutionHints& with_l1_to_l2_message_exists_hints(std::vector<std::pair<FF, FF>> l1_to_l2_message_exists_hints)
    {
        this->l1_to_l2_message_exists_hints = std::move(l1_to_l2_message_exists_hints);
        return *this;
    }
    ExecutionHints& with_externalcall_hints(std::vector<ExternalCallHint> externalcall_hints)
    {
        this->externalcall_hints = std::move(externalcall_hints);
        return *this;
    }
    ExecutionHints& with_contract_instance_hints(std::map<FF, ContractInstanceHint> contract_instance_hints)
    {
        this->contract_instance_hints = std::move(contract_instance_hints);
        return *this;
    }

    static void push_vec_into_map(std::unordered_map<uint32_t, FF>& into_map,
                                  const std::vector<std::pair<FF, FF>>& from_pair_vec)
    {
        for (const auto& pair : from_pair_vec) {
            into_map[static_cast<uint32_t>(pair.first)] = pair.second;
        }
    }

    // TODO: Cache.
    // Side effect counter -> value
    std::unordered_map<uint32_t, FF> get_side_effect_hints() const
    {
        std::unordered_map<uint32_t, FF> hints_map;
        push_vec_into_map(hints_map, storage_value_hints);
        push_vec_into_map(hints_map, note_hash_exists_hints);
        push_vec_into_map(hints_map, nullifier_exists_hints);
        push_vec_into_map(hints_map, l1_to_l2_message_exists_hints);
        return hints_map;
    }

    static ExecutionHints from(const std::vector<uint8_t>& data)
    {
        std::vector<std::pair<FF, FF>> storage_value_hints;
        std::vector<std::pair<FF, FF>> note_hash_exists_hints;
        std::vector<std::pair<FF, FF>> nullifier_exists_hints;
        std::vector<std::pair<FF, FF>> l1_to_l2_message_exists_hints;

        using serialize::read;
        const auto* it = data.data();
        read(it, storage_value_hints);
        read(it, note_hash_exists_hints);
        read(it, nullifier_exists_hints);
        read(it, l1_to_l2_message_exists_hints);

        std::vector<ExternalCallHint> externalcall_hints;
        read(it, externalcall_hints);

        std::vector<ContractInstanceHint> contract_instance_hints_vec;
        read(it, contract_instance_hints_vec);
        std::map<FF, ContractInstanceHint> contract_instance_hints;
        for (const auto& instance : contract_instance_hints_vec) {
            contract_instance_hints[instance.address] = instance;
        }

        if (it != data.data() + data.size()) {
            throw_or_abort("Failed to deserialize ExecutionHints: only read" + std::to_string(it - data.data()) +
                           " bytes out of " + std::to_string(data.size()) + " bytes");
        }

        return { std::move(storage_value_hints),    std::move(note_hash_exists_hints),
                 std::move(nullifier_exists_hints), std::move(l1_to_l2_message_exists_hints),
                 std::move(externalcall_hints),     std::move(contract_instance_hints) };
    }

  private:
    ExecutionHints(std::vector<std::pair<FF, FF>> storage_value_hints,
                   std::vector<std::pair<FF, FF>> note_hash_exists_hints,
                   std::vector<std::pair<FF, FF>> nullifier_exists_hints,
                   std::vector<std::pair<FF, FF>> l1_to_l2_message_exists_hints,
                   std::vector<ExternalCallHint> externalcall_hints,
                   std::map<FF, ContractInstanceHint> contract_instance_hints)
        : storage_value_hints(std::move(storage_value_hints))
        , note_hash_exists_hints(std::move(note_hash_exists_hints))
        , nullifier_exists_hints(std::move(nullifier_exists_hints))
        , l1_to_l2_message_exists_hints(std::move(l1_to_l2_message_exists_hints))
        , externalcall_hints(std::move(externalcall_hints))
        , contract_instance_hints(std::move(contract_instance_hints))
    {}
};

} // namespace bb::avm_trace
