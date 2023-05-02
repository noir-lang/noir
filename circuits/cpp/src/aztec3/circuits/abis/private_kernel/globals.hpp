#pragma once
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>

#include <barretenberg/stdlib/primitives/witness/witness.hpp>

namespace aztec3::circuits::abis::private_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct Globals {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr min_timestamp = 0;

    boolean operator==(Globals<NCT> const& other) const { return min_timestamp == other.min_timestamp; };

    template <typename Composer> Globals<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        Globals<CircuitTypes<Composer>> global_data = { to_ct(min_timestamp) };

        return global_data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        min_timestamp.set_public();
    }
};

}  // namespace aztec3::circuits::abis::private_kernel