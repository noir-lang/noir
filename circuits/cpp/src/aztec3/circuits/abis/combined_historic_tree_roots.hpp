#pragma once
#include "private_historic_tree_roots.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::circuits::abis::PrivateHistoricTreeRoots;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct CombinedHistoricTreeRoots {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    PrivateHistoricTreeRoots<NCT> private_historic_tree_roots{};


    // for serialization, update with new fields
    MSGPACK_FIELDS(private_historic_tree_roots);

    boolean operator==(CombinedHistoricTreeRoots<NCT> const& other) const
    {
        return private_historic_tree_roots == other.private_historic_tree_roots;
    };

    template <typename Builder> CombinedHistoricTreeRoots<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(builder); };

        CombinedHistoricTreeRoots<CircuitTypes<Builder>> data = {
            to_circuit_type(private_historic_tree_roots),
        };

        return data;
    };

    template <typename Builder> CombinedHistoricTreeRoots<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_native_type = [&]<typename T>(T& e) { return e.template to_native_type<Builder>(); };

        CombinedHistoricTreeRoots<NativeTypes> data = {
            to_native_type(private_historic_tree_roots),
        };

        return data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        private_historic_tree_roots.set_public();
    }
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, CombinedHistoricTreeRoots<NCT> const& historic_tree_roots)
{
    return os << "private_historic_tree_roots: " << historic_tree_roots.private_historic_tree_roots << "\n";
}

}  // namespace aztec3::circuits::abis
