#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/convert.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct StateTransition {
    typedef typename NCT::fr fr;

    fr storage_slot;
    fr old_value;
    fr new_value;

    bool operator==(StateTransition<NCT> const&) const = default;

    static StateTransition<NCT> empty() { return { 0, 0, 0 }; };

    template <typename Composer> StateTransition<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        StateTransition<CircuitTypes<Composer>> state_transition = {
            to_ct(storage_slot),
            to_ct(old_value),
            to_ct(new_value),
        };

        return state_transition;
    };
};

template <typename NCT> void read(uint8_t const*& it, StateTransition<NCT>& state_transition)
{
    using serialize::read;

    read(it, state_transition.l1_result_hash);
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