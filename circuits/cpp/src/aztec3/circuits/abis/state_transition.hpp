#pragma once
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/circuit_types.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename NCT> struct StateTransition {
    typedef typename NCT::fr fr;

    fr storage_slot = 0;
    fr old_value = 0;
    fr new_value = 0;

    bool operator==(StateTransition<NCT> const&) const = default;

    template <typename Composer> StateTransition<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        StateTransition<CircuitTypes<Composer>> state_transition = {
            to_ct(storage_slot),
            to_ct(old_value),
            to_ct(new_value),
        };

        return state_transition;
    };

    template <typename Composer> StateTransition<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };

        StateTransition<NativeTypes> state_transition = {
            to_nt(storage_slot),
            to_nt(old_value),
            to_nt(new_value),
        };

        return state_transition;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            storage_slot,
            old_value,
            new_value,
        };

        return NCT::compress(inputs, GeneratorIndex::STATE_TRANSITION);
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        storage_slot.set_public();
        old_value.set_public();
        new_value.set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, StateTransition<NCT>& state_transition)
{
    using serialize::read;

    read(it, state_transition.storage_slot);
    read(it, state_transition.old_value);
    read(it, state_transition.new_value);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, StateTransition<NCT> const& state_transition)
{
    using serialize::write;

    write(buf, state_transition.storage_slot);
    write(buf, state_transition.old_value);
    write(buf, state_transition.new_value);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, StateTransition<NCT> const& state_transition)
{
    return os << "storage_slot: " << state_transition.storage_slot << "\n"
              << "old_value: " << state_transition.old_value << "\n"
              << "new_value: " << state_transition.new_value << "\n";
}

} // namespace aztec3::circuits::abis