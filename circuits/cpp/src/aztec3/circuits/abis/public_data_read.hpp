#pragma once
#include "aztec3/constants.hpp"
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/circuit_types.hpp>

namespace aztec3::circuits::abis {

using aztec3::GeneratorIndex;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct PublicDataRead {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    fr leaf_index = 0;
    fr value = 0;

    bool operator==(PublicDataRead<NCT> const&) const = default;

    template <typename Composer> PublicDataRead<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        PublicDataRead<CircuitTypes<Composer>> state_transition = {
            to_ct(leaf_index),
            to_ct(value),
        };

        return state_transition;
    };

    template <typename Composer> PublicDataRead<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };

        PublicDataRead<NativeTypes> state_transition = {
            to_nt(leaf_index),
            to_nt(value),
        };

        return state_transition;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            leaf_index,
            value,
        };

        return NCT::compress(inputs, GeneratorIndex::STATE_READ);
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        leaf_index.set_public();
        value.set_public();
    }

    boolean is_empty() const { return leaf_index == 0; }
};

template <typename NCT> void read(uint8_t const*& it, PublicDataRead<NCT>& state_transition)
{
    using serialize::read;

    read(it, state_transition.leaf_index);
    read(it, state_transition.value);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, PublicDataRead<NCT> const& state_transition)
{
    using serialize::write;

    write(buf, state_transition.leaf_index);
    write(buf, state_transition.value);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, PublicDataRead<NCT> const& state_transition)
{
    return os << "leaf_index: " << state_transition.leaf_index << "\n"
              << "value: " << state_transition.value << "\n";
}

} // namespace aztec3::circuits::abis