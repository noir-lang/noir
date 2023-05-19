#pragma once
#include "aztec3/constants.hpp"
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>

#include <barretenberg/stdlib/primitives/witness/witness.hpp>

namespace aztec3::circuits::abis {

using aztec3::GeneratorIndex;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct PublicDataRead {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr leaf_index = 0;
    fr value = 0;

    // for serialization, update with new fields
    MSGPACK_FIELDS(leaf_index, value);
    bool operator==(PublicDataRead<NCT> const&) const = default;

    template <typename Composer> PublicDataRead<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        PublicDataRead<CircuitTypes<Composer>> read = {
            to_ct(leaf_index),
            to_ct(value),
        };

        return read;
    };

    template <typename Composer> PublicDataRead<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };

        PublicDataRead<NativeTypes> read = {
            to_nt(leaf_index),
            to_nt(value),
        };

        return read;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            leaf_index,
            value,
        };

        return NCT::compress(inputs, GeneratorIndex::PUBLIC_DATA_READ);
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        leaf_index.set_public();
        value.set_public();
    }

    boolean is_empty() const { return leaf_index == 0; }
};

template <typename NCT> void read(uint8_t const*& it, PublicDataRead<NCT>& publicDataRead)
{
    using serialize::read;

    read(it, publicDataRead.leaf_index);
    read(it, publicDataRead.value);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, PublicDataRead<NCT> const& publicDataRead)
{
    using serialize::write;

    write(buf, publicDataRead.leaf_index);
    write(buf, publicDataRead.value);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, PublicDataRead<NCT> const& publicDataRead)
{
    return os << "leaf_index: " << publicDataRead.leaf_index << "\n"
              << "value: " << publicDataRead.value << "\n";
}

}  // namespace aztec3::circuits::abis