#pragma once

#include "tx_request.hpp"

#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename NCT> struct SignedTxRequest {
    TxRequest<NCT> tx_request;
    // Signature<NCT> signature; // TODO: import some kind of signature

    template <typename Composer> SignedTxRequest<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        // auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };
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