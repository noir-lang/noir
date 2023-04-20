#pragma once

#include "combined_historic_tree_roots.hpp"
#include "tx_context.hpp"

#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;
using std::is_same;

template <typename NCT> struct CombinedConstantData {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    CombinedHistoricTreeRoots<NCT> historic_tree_roots{};
    TxContext<NCT> tx_context{};

    boolean operator==(CombinedConstantData<NCT> const& other) const
    {
        return historic_tree_roots == other.historic_tree_roots && tx_context == other.tx_context;
    };

    template <typename Composer> CombinedConstantData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        CombinedConstantData<CircuitTypes<Composer>> constant_data = {
            historic_tree_roots.to_circuit_type(composer),
            tx_context.to_circuit_type(composer),
        };

        return constant_data;
    };

    template <typename Composer> CombinedConstantData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);

        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Composer>(); };

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

template <typename NCT> void read(uint8_t const*& it, CombinedConstantData<NCT>& constant_data)
{
    using serialize::read;

    read(it, constant_data.historic_tree_roots);
    read(it, constant_data.tx_context);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, CombinedConstantData<NCT> const& constant_data)
{
    using serialize::write;

    write(buf, constant_data.historic_tree_roots);
    write(buf, constant_data.tx_context);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, CombinedConstantData<NCT> const& constant_data)
{
    return os << "historic_tree_roots: " << constant_data.historic_tree_roots << "\n"
              << "tx_context: " << constant_data.tx_context << "\n";
}

} // namespace aztec3::circuits::abis