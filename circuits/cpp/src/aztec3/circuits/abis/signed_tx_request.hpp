#pragma once

#include "tx_request.hpp"

#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct SignedTxRequest {
    TxRequest<NCT> tx_request;
    // Signature<NCT> signature; // TODO: import some kind of signature

    template <typename Composer> SignedTxRequest<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        // auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        SignedTxRequest<CircuitTypes<Composer>> signed_tx_request = {
            to_circuit_type(tx_request),
        };

        return signed_tx_request;
    };
};

template <typename NCT> void read(uint8_t const*& it, SignedTxRequest<NCT>& signed_tx_request)
{
    using serialize::read;

    read(it, signed_tx_request.tx_request);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, SignedTxRequest<NCT> const& signed_tx_request)
{
    using serialize::write;

    write(buf, signed_tx_request.tx_request);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, SignedTxRequest<NCT> const& signed_tx_request)
{
    return os << "tx_request: " << signed_tx_request.tx_request << "\n";
}

} // namespace aztec3::circuits::abis