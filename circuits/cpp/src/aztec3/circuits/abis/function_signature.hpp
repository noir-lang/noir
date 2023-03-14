#pragma once
//#include <crypto/generators/generator_data.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
//#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include <stdlib/types/native_types.hpp>
#include <aztec3/constants.hpp>

namespace aztec3::circuits::abis {

// using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;
using std::is_same;

template <typename NCT> struct FunctionSignature {
    // typedef typename NCT::address address;
    typedef typename NCT::uint32 uint32;
    typedef typename NCT::boolean boolean;
    // typedef typename NCT::grumpkin_point grumpkin_point;
    typedef typename NCT::fr fr;

    uint32 function_encoding; // e.g. 1st 4-bytes of abi-encoding of function.
    boolean is_private = false;
    boolean is_constructor = false;

    bool operator==(FunctionSignature<NCT> const&) const = default;

    static FunctionSignature<NCT> empty() { return { 0, 0, 0 }; };

    template <typename Composer> FunctionSignature<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        FunctionSignature<CircuitTypes<Composer>> function_signature = {
            to_ct(function_encoding),
            to_ct(is_private),
            to_ct(is_constructor),
        };

        return function_signature;
    };

    template <typename Composer> FunctionSignature<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return plonk::stdlib::types::to_nt<Composer>(e); };

        FunctionSignature<NativeTypes> fs = {
            to_nt(function_encoding),
            to_nt(is_private),
            to_nt(is_constructor),
        };

        return fs;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        fr(function_encoding).set_public();
        fr(is_private).set_public();
        fr(is_constructor).set_public();
    }

    // TODO: this can all be packed into 1 field element, so this `hash` function should just return that field element.
    fr hash() const
    {
        std::vector<fr> inputs = {
            fr(function_encoding),
            fr(is_private),
            fr(is_constructor),
        };

        return NCT::compress(inputs, GeneratorIndex::FUNCTION_SIGNATURE);
    }
};

template <typename NCT> void read(uint8_t const*& it, FunctionSignature<NCT>& function_signature)
{
    using serialize::read;

    read(it, function_signature.function_encoding);
    read(it, function_signature.is_private);
    read(it, function_signature.is_constructor);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, FunctionSignature<NCT> const& function_signature)
{
    using serialize::write;

    write(buf, function_signature.function_encoding);
    write(buf, function_signature.is_private);
    write(buf, function_signature.is_constructor);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, FunctionSignature<NCT> const& function_signature)
{
    return os << "function_encoding: " << function_signature.function_encoding << "\n"
              << "is_private: " << function_signature.is_private << "\n"
              << "is_constructor: " << function_signature.is_constructor << "\n";
}

} // namespace aztec3::circuits::abis