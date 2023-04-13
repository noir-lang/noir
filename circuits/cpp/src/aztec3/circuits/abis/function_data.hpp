#pragma once
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/constants.hpp>

namespace aztec3::circuits::abis {

// using plonk::stdlib::witness_t;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct FunctionData {
    typedef typename NCT::uint32 uint32;
    typedef typename NCT::boolean boolean;
    typedef typename NCT::fr fr;

    uint32 function_selector; // e.g. 1st 4-bytes of abi-encoding of function.
    boolean is_private = false;
    boolean is_constructor = false;

    boolean operator==(FunctionData<NCT> const& other) const
    {
        return function_selector == other.function_selector && is_private == other.is_private &&
               is_constructor == other.is_constructor;
    };

    template <typename Composer> FunctionData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        FunctionData<CircuitTypes<Composer>> function_data = {
            to_ct(function_selector),
            to_ct(is_private),
            to_ct(is_constructor),
        };

        return function_data;
    };

    template <typename Composer> FunctionData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };

        FunctionData<NativeTypes> function_data = {
            to_nt(function_selector),
            to_nt(is_private),
            to_nt(is_constructor),
        };

        return function_data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        fr(function_selector).set_public();
        fr(is_private).set_public();
        fr(is_constructor).set_public();
    }

    // TODO: this can all be packed into 1 field element, so this `hash` function should just return that field element.
    fr hash() const
    {
        std::vector<fr> inputs = {
            fr(function_selector),
            fr(is_private),
            fr(is_constructor),
        };

        return NCT::compress(inputs, GeneratorIndex::FUNCTION_DATA);
    }
};

template <typename NCT> void read(uint8_t const*& it, FunctionData<NCT>& function_data)
{
    using serialize::read;

    read(it, function_data.function_selector);
    read(it, function_data.is_private);
    read(it, function_data.is_constructor);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, FunctionData<NCT> const& function_data)
{
    using serialize::write;

    write(buf, function_data.function_selector);
    write(buf, function_data.is_private);
    write(buf, function_data.is_constructor);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, FunctionData<NCT> const& function_data)
{
    return os << "function_selector: " << function_data.function_selector << "\n"
              << "is_private: " << function_data.is_private << "\n"
              << "is_constructor: " << function_data.is_constructor << "\n";
}

} // namespace aztec3::circuits::abis