#pragma once
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename NCT> struct StateRead {
    typedef typename NCT::fr fr;

    fr storage_slot = 0;
    fr current_value = 0;

    bool operator==(StateRead<NCT> const&) const = default;

    template <typename Composer> StateRead<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        StateRead<CircuitTypes<Composer>> state_read = {
            to_ct(storage_slot),
            to_ct(current_value),
        };

        return state_read;
    };

    template <typename Composer> StateRead<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };

        StateRead<NativeTypes> state_read = {
            to_nt(storage_slot),
            to_nt(current_value),
        };

        return state_read;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            storage_slot,
            current_value,
        };

        return NCT::compress(inputs, GeneratorIndex::STATE_READ);
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        storage_slot.set_public();
        current_value.set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, StateRead<NCT>& state_read)
{
    using serialize::read;

    read(it, state_read.storage_slot);
    read(it, state_read.current_value);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, StateRead<NCT> const& state_read)
{
    using serialize::write;

    write(buf, state_read.storage_slot);
    write(buf, state_read.current_value);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, StateRead<NCT> const& state_read)
{
    return os << "storage_slot: " << state_read.storage_slot << "\n"
              << "current_value: " << state_read.current_value << "\n";
}

} // namespace aztec3::circuits::abis
