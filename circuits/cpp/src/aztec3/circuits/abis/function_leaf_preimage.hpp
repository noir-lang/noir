#pragma once

#include "aztec3/circuits/abis/function_selector.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

/**
 * @brief A struct representing the "preimage" of a function tree leaf.
 * Templated on NativeTypes/CircuitTypes.
 *
 * @details A FunctionLeafPreimage contains:
 * - `selector` keccak hash of function signature truncated to NUM_FUNCTION_SELECTOR_BYTES
 * - `is_private` boolean flag
 * - `vk_hash` pedersen hash of the function verification key
 * - `acir_hash` hash of the function's acir bytecode
 * This struct includes a `hash()` function for computing its pedersen compression.
 * There are also static functions for:
 * - converting preimages between native/circuit types
 * - serialising and deserialising preimages
 * - writing a preimage to an ostream
 */
template <typename NCT> struct FunctionLeafPreimage {
    using boolean = typename NCT::boolean;
    using fr = typename NCT::fr;
    using uint32 = typename NCT::uint32;

    FunctionSelector<NCT> selector = {};
    boolean is_internal = false;
    boolean is_private = false;
    fr vk_hash = 0;
    fr acir_hash = 0;

    // For serialization, update with new fields
    MSGPACK_FIELDS(selector, is_internal, is_private, vk_hash, acir_hash);

    boolean operator==(FunctionLeafPreimage<NCT> const& other) const
    {
        return selector == other.selector && is_internal == other.is_internal && is_private == other.is_private &&
               vk_hash == other.vk_hash && acir_hash == other.acir_hash;
    };

    template <typename Builder> FunctionLeafPreimage<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        FunctionLeafPreimage<CircuitTypes<Builder>> preimage = {
            selector.to_circuit_type(builder), to_ct(is_internal), to_ct(is_private), to_ct(vk_hash), to_ct(acir_hash),
        };

        return preimage;
    };

    template <typename Builder> FunctionLeafPreimage<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Builder>(); };
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        FunctionLeafPreimage<NativeTypes> preimage = {
            to_native_type(selector), to_nt(is_internal), to_nt(is_private), to_nt(vk_hash), to_nt(acir_hash),
        };

        return preimage;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        selector.set_public();
        fr(is_internal).set_public();
        fr(is_private).set_public();
        vk_hash.set_public();
        acir_hash.set_public();
    }

    fr hash() const
    {
        std::vector<fr> const inputs = {
            selector.value, fr(is_internal), fr(is_private), vk_hash, acir_hash,
        };
        return NCT::compress(inputs, GeneratorIndex::FUNCTION_LEAF);
    }
};

}  // namespace aztec3::circuits::abis
