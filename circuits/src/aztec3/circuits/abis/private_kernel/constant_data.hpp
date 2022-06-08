#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include "../executed_callback.hpp"
#include "old_tree_roots.hpp"
#include "globals.hpp"

namespace aztec3::circuits::abis::private_kernel {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;
using std::is_same;

template <typename NCT> struct ConstantData {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    OldTreeRoots<NCT> old_tree_roots;
    boolean is_constructor_recursion = false;
    boolean is_callback_recursion = false;
    ExecutedCallback<NCT> executed_callback;
    Globals<NCT> globals;

    template <typename Composer> ConstantData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        ConstantData<CircuitTypes<Composer>> constant_data = {
            old_tree_roots.to_circuit_type(composer),
            to_ct(is_constructor_recursion),
            to_ct(is_callback_recursion),
            executed_callback.to_circuit_type(composer),
            globals.to_circuit_type(composer),
        };

        return constant_data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        old_tree_roots.set_public();
        fr(is_constructor_recursion).set_public();
        fr(is_callback_recursion).set_public();
        executed_callback.set_public();
        globals.set_public();
    }

    // template <typename Composer> void set_private_data_tree_root(typename CircuitTypes<Composer>::fr const& value)
    // {
    //     old_tree_roots.private_data_tree_root.assert_equal(0, "Cannot edit a nonzero constant.");
    //     old_tree_roots.private_data_tree_root = value;
    // }
};

} // namespace aztec3::circuits::abis::private_kernel