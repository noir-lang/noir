#pragma once
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/circuit_types.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename NCT> struct PublicDataWrite {
    typedef typename NCT::fr fr;

    fr leaf_index = 0;
    fr new_value = 0;

    bool operator==(PublicDataWrite<NCT> const&) const = default;

    template <typename Composer> PublicDataWrite<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        PublicDataWrite<CircuitTypes<Composer>> state_transition = {
            to_ct(leaf_index),
            to_ct(new_value),
        };

        return state_transition;
    };

    template <typename Composer> PublicDataWrite<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };

        PublicDataWrite<NativeTypes> state_transition = {
            to_nt(leaf_index),
            to_nt(new_value),
        };

        return state_transition;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            leaf_index,
            new_value,
        };

        return NCT::compress(inputs, GeneratorIndex::STATE_TRANSITION);
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        leaf_index.set_public();
        new_value.set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, PublicDataWrite<NCT>& state_transition)
{
    using serialize::read;

    read(it, state_transition.leaf_index);
    read(it, state_transition.new_value);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, PublicDataWrite<NCT> const& state_transition)
{
    using serialize::write;

    write(buf, state_transition.leaf_index);
    write(buf, state_transition.new_value);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, PublicDataWrite<NCT> const& state_transition)
{
    return os << "leaf_index: " << state_transition.leaf_index << "\n"
              << "new_value: " << state_transition.new_value << "\n";
}

} // namespace aztec3::circuits::abis