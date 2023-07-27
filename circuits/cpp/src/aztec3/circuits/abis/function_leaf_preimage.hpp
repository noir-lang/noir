#pragma once

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
 * - `function_selector` keccak hash of function signature truncated to NUM_FUNCTION_SELECTOR_BYTES
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

    uint32 function_selector = 0;
    boolean is_internal = false;
    boolean is_private = false;
    fr vk_hash = 0;
    fr acir_hash = 0;

    boolean operator==(FunctionLeafPreimage<NCT> const& other) const
    {
        return function_selector == other.function_selector && is_internal == other.is_internal &&
               is_private == other.is_private && vk_hash == other.vk_hash && acir_hash == other.acir_hash;
    };

    template <typename Builder> FunctionLeafPreimage<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        FunctionLeafPreimage<CircuitTypes<Builder>> preimage = {
            to_ct(function_selector), to_ct(is_internal), to_ct(is_private), to_ct(vk_hash), to_ct(acir_hash),
        };

        return preimage;
    };

    template <typename Builder> FunctionLeafPreimage<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        FunctionLeafPreimage<NativeTypes> preimage = {
            to_nt(function_selector), to_nt(is_internal), to_nt(is_private), to_nt(vk_hash), to_nt(acir_hash),
        };

        return preimage;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        function_selector.set_public();
        fr(is_internal).set_public();
        fr(is_private).set_public();
        vk_hash.set_public();
        acir_hash.set_public();
    }

    fr hash() const
    {
        std::vector<fr> const inputs = {
            function_selector, fr(is_internal), fr(is_private), vk_hash, acir_hash,
        };
        return NCT::compress(inputs, GeneratorIndex::FUNCTION_LEAF);
    }
};

template <typename NCT> void read(uint8_t const*& it, FunctionLeafPreimage<NCT>& preimage)
{
    using serialize::read;

    read(it, preimage.function_selector);
    read(it, preimage.is_internal);
    read(it, preimage.is_private);
    read(it, preimage.vk_hash);
    read(it, preimage.acir_hash);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, FunctionLeafPreimage<NCT> const& preimage)
{
    using serialize::write;

    write(buf, preimage.function_selector);
    write(buf, preimage.is_internal);
    write(buf, preimage.is_private);
    write(buf, preimage.vk_hash);
    write(buf, preimage.acir_hash);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, FunctionLeafPreimage<NCT> const& preimage)
{
    return os << "function_selector: " << preimage.function_selector << "\n"
              << "is_internal: " << preimage.is_internal << "\n"
              << "is_private: " << preimage.is_private << "\n"
              << "vk_hash: " << preimage.vk_hash << "\n"
              << "acir_hash: " << preimage.acir_hash << "\n";
}

}  // namespace aztec3::circuits::abis