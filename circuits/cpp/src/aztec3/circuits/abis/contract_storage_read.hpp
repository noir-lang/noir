#pragma once
#include "aztec3/utils/msgpack_derived_output.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename NCT> struct ContractStorageRead {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr storage_slot = 0;
    fr current_value = 0;

    // for serialization, update with new fields
    MSGPACK_FIELDS(storage_slot, current_value);
    bool operator==(ContractStorageRead<NCT> const&) const = default;

    template <typename Builder> ContractStorageRead<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        ContractStorageRead<CircuitTypes<Builder>> contract_storage_read = {
            to_ct(storage_slot),
            to_ct(current_value),
        };

        return contract_storage_read;
    };

    template <typename Builder> ContractStorageRead<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        ContractStorageRead<NativeTypes> contract_storage_read = {
            to_nt(storage_slot),
            to_nt(current_value),
        };

        return contract_storage_read;
    };

    fr hash() const
    {
        std::vector<fr> const inputs = {
            storage_slot,
            current_value,
        };

        return NCT::hash(inputs, GeneratorIndex::PUBLIC_DATA_READ);
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        storage_slot.set_public();
        current_value.set_public();
    }

    boolean is_empty() const { return storage_slot == 0; }
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, ContractStorageRead<NCT> const& contract_storage_read)
{
    utils::msgpack_derived_output(os, contract_storage_read);
    return os;
}

}  // namespace aztec3::circuits::abis
