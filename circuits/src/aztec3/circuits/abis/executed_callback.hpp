#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/convert.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct ExecutedCallback {
    typedef typename NCT::fr fr;
    typedef typename NCT::uint32 uint32;

    fr l1_result_hash;
    uint32 l1_results_tree_leaf_index;

    bool operator==(ExecutedCallback<NCT> const&) const = default;

    static ExecutedCallback<NCT> empty() { return { 0, 0 }; };

    template <typename Composer> ExecutedCallback<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        ExecutedCallback<CircuitTypes<Composer>> executed_callback = {
            to_ct(l1_result_hash),
            to_ct(l1_results_tree_leaf_index),
        };

        return executed_callback;
    };

    template <typename Composer> ExecutedCallback<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return plonk::stdlib::types::to_nt<Composer>(e); };

        ExecutedCallback<NativeTypes> executed_callback = {
            to_nt(l1_result_hash),
            to_nt(l1_results_tree_leaf_index),
        };

        return executed_callback;
    };

    template <typename Composer> void assert_is_zero()
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        l1_result_hash.assert_is_zero();
        fr(l1_results_tree_leaf_index).assert_is_zero();
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        l1_result_hash.set_public();
        fr(l1_results_tree_leaf_index).set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, ExecutedCallback<NCT>& executed_callback)
{
    using serialize::read;

    read(it, executed_callback.l1_result_hash);
    read(it, executed_callback.l1_results_tree_leaf_index);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, ExecutedCallback<NCT> const& executed_callback)
{
    using serialize::write;

    write(buf, executed_callback.l1_result_hash);
    write(buf, executed_callback.l1_results_tree_leaf_index);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, ExecutedCallback<NCT> const& executed_callback)
{
    return os << "l1_result_hash: " << executed_callback.l1_result_hash << "\n"
              << "l1_results_tree_leaf_index: " << executed_callback.l1_results_tree_leaf_index << "\n";
}

} // namespace aztec3::circuits::abis