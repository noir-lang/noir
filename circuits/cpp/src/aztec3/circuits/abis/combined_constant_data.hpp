#pragma once

#include "tx_context.hpp"

#include "aztec3/circuits/abis/combined_historic_tree_roots.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::circuits::abis::CombinedHistoricTreeRoots;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct CombinedConstantData {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    CombinedHistoricTreeRoots<NCT> historic_tree_roots{};
    TxContext<NCT> tx_context{};

    // for serialization: update up with new fields
    MSGPACK_FIELDS(historic_tree_roots, tx_context);
    boolean operator==(CombinedConstantData<NCT> const& other) const
    {
        return historic_tree_roots == other.historic_tree_roots && tx_context == other.tx_context;
    }

    template <typename Builder> CombinedConstantData<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        CombinedConstantData<CircuitTypes<Builder>> constant_data = {
            historic_tree_roots.to_circuit_type(builder),
            tx_context.to_circuit_type(builder),
        };

        return constant_data;
    };

    template <typename Builder> CombinedConstantData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);

        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Builder>(); };

        CombinedConstantData<NativeTypes> constant_data = {
            to_native_type(historic_tree_roots),
            to_native_type(tx_context),
        };

        return constant_data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        historic_tree_roots.set_public();
        tx_context.set_public();
    }
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, CombinedConstantData<NCT> const& constant_data)
{
    return os << "historic_tree_roots: " << constant_data.historic_tree_roots << "\n"
              << "tx_context: " << constant_data.tx_context << "\n";
}

}  // namespace aztec3::circuits::abis
