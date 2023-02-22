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

template <typename NCT> struct TxRequest {
    typedef typename NCT::address address;
    // typedef typename NCT::grumpkin_point grumpkin_point;
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    address from;
    address to;
    FunctionSignature<NCT> function_signature;
    std::array<fr, ARGS_LENGTH> args;
    fr nonce;
    TxContext<NCT> tx_context;
    fr chain_id;

    template <typename Composer> TxRequest<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        TxRequest<CircuitTypes<Composer>> tx_request = {
            to_ct(from),     to_ct(to),    to_circuit_type(function_signature),
            to_ct(args),     to_ct(nonce), to_circuit_type(tx_context),
            to_ct(chain_id),
        };

        return tx_request;
    };
};

template <typename NCT> void read(uint8_t const*& it, TxRequest<NCT>& tx_request)
{
    using serialize::read;

    read(it, tx_request.from);
    read(it, tx_request.to);
    read(it, tx_request.function_signature);
    read(it, tx_request.args);
    read(it, tx_request.nonce);
    read(it, tx_request.tx_context);
    read(it, tx_request.chain_id);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, TxRequest<NCT> const& tx_request)
{
    using serialize::write;

    write(buf, tx_request.from);
    write(buf, tx_request.to);
    write(buf, tx_request.function_signature);
    write(buf, tx_request.args);
    write(buf, tx_request.nonce);
    write(buf, tx_request.tx_context);
    write(buf, tx_request.chain_id);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, TxRequest<NCT> const& tx_request)
{
    return os << "from: " << tx_request.from << "\n"
              << "to: " << tx_request.to << "\n"
              << "function_signature: " << tx_request.function_signature << "\n"
              << "args: " << tx_request.args << "\n"
              << "nonce: " << tx_request.nonce << "\n"
              << "tx_context: " << tx_request.tx_context << "\n"
              << "chain_id: " << tx_request.chain_id << "\n";
}

} // namespace aztec3::circuits::abis