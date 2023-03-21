#pragma once

#include "old_tree_roots.hpp"
#include "../tx_context.hpp"

#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis::private_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;
using std::is_same;

template <typename NCT> struct ConstantData {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    OldTreeRoots<NCT> old_tree_roots;
    TxContext<NCT> tx_context;

    template <typename Composer> ConstantData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        ConstantData<CircuitTypes<Composer>> constant_data = {
            old_tree_roots.to_circuit_type(composer),
            tx_context.to_circuit_type(composer),
        };

        return constant_data;
    };

    template <typename Composer> ConstantData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);

        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Composer>(); };

        ConstantData<NativeTypes> constant_data = {
            to_native_type(old_tree_roots),
            to_native_type(tx_context),
        };

        return constant_data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        old_tree_roots.set_public();
        tx_context.set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, ConstantData<NCT>& constant_data)
{
    using serialize::read;

    read(it, constant_data.old_tree_roots);
    read(it, constant_data.tx_context);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, ConstantData<NCT> const& constant_data)
{
    using serialize::write;

    write(buf, constant_data.old_tree_roots);
    write(buf, constant_data.tx_context);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, ConstantData<NCT> const& constant_data)
{
    return os << "old_tree_roots: " << constant_data.old_tree_roots << "\n"
              << "tx_context: " << constant_data.tx_context << "\n";
}

} // namespace aztec3::circuits::abis::private_kernel