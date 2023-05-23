#pragma once
#include <aztec3/utils/msgpack_derived_output.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>

#include <barretenberg/serialize/msgpack.hpp>
#include <barretenberg/stdlib/primitives/witness/witness.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename NCT> struct ContractStorageUpdateRequest {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr storage_slot = 0;
    fr old_value = 0;
    fr new_value = 0;

    // for serialization, update with new fields
    MSGPACK_FIELDS(storage_slot, old_value, new_value);
    bool operator==(ContractStorageUpdateRequest<NCT> const&) const = default;
    template <typename Composer>
    ContractStorageUpdateRequest<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        ContractStorageUpdateRequest<CircuitTypes<Composer>> update_request = {
            to_ct(storage_slot),
            to_ct(old_value),
            to_ct(new_value),
        };

        return update_request;
    };

    template <typename Composer> ContractStorageUpdateRequest<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };

        ContractStorageUpdateRequest<NativeTypes> update_request = {
            to_nt(storage_slot),
            to_nt(old_value),
            to_nt(new_value),
        };

        return update_request;
    };

    fr hash() const
    {
        std::vector<fr> const inputs = {
            storage_slot,
            old_value,
            new_value,
        };

        return NCT::compress(inputs, GeneratorIndex::PUBLIC_DATA_UPDATE_REQUEST);
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        storage_slot.set_public();
        old_value.set_public();
        new_value.set_public();
    }

    boolean is_empty() const { return storage_slot == 0; }
};

template <typename NCT> void read(uint8_t const*& it, ContractStorageUpdateRequest<NCT>& update_request)
{
    using serialize::read;

    read(it, update_request.storage_slot);
    read(it, update_request.old_value);
    read(it, update_request.new_value);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, ContractStorageUpdateRequest<NCT> const& update_request)
{
    using serialize::write;

    write(buf, update_request.storage_slot);
    write(buf, update_request.old_value);
    write(buf, update_request.new_value);
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, ContractStorageUpdateRequest<NCT> const& update_request)
{
    utils::msgpack_derived_output(os, update_request);
    return os;
}

}  // namespace aztec3::circuits::abis