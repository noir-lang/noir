#pragma once
#include "function_signature.hpp"
#include "tx_context.hpp"
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct TxObject {
    typedef typename NCT::address address;
    // typedef typename NCT::grumpkin_point grumpkin_point;
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    address from;
    address to;
    FunctionSignature<NCT> function_signature;
    std::array<fr, CUSTOM_INPUTS_LENGTH> custom_inputs;
    fr nonce;
    TxContext<NCT> tx_context;
    fr chain_id;

    template <typename Composer> TxObject<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        TxObject<CircuitTypes<Composer>> tx_object = {
            to_ct(from),          to_ct(to),    to_circuit_type(function_signature),
            to_ct(custom_inputs), to_ct(nonce), to_circuit_type(tx_context),
            to_ct(chain_id),
        };

        return tx_object;
    };
};

template <typename NCT> void read(uint8_t const*& it, TxObject<NCT>& tx_object)
{
    using serialize::read;

    read(it, tx_object.from);
    read(it, tx_object.to);
    read(it, tx_object.function_signature);
    read(it, tx_object.custom_inputs);
    read(it, tx_object.nonce);
    read(it, tx_object.tx_context);
    read(it, tx_object.chain_id);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, TxObject<NCT> const& tx_object)
{
    using serialize::write;

    write(buf, tx_object.from);
    write(buf, tx_object.to);
    write(buf, tx_object.function_signature);
    write(buf, tx_object.custom_inputs);
    write(buf, tx_object.nonce);
    write(buf, tx_object.tx_context);
    write(buf, tx_object.chain_id);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, TxObject<NCT> const& tx_object)
{
    return os << "from: " << tx_object.from << "\n"
              << "to: " << tx_object.to << "\n"
              << "function_signature: " << tx_object.function_signature << "\n"
              << "custom_inputs: " << tx_object.custom_inputs << "\n"
              << "nonce: " << tx_object.nonce << "\n"
              << "tx_context: " << tx_object.tx_context << "\n"
              << "chain_id: " << tx_object.chain_id << "\n";
}

} // namespace aztec3::circuits::abis