#pragma once

#include "tx_request.hpp"

#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct SignedTxRequest {
    typedef typename NCT::boolean boolean;
    typedef typename NCT::ecdsa_signature Signature;

    TxRequest<NCT> tx_request = TxRequest<NCT>();
    Signature signature = Signature();

    boolean operator==(SignedTxRequest<NCT> const& other) const
    {
        return tx_request == other.tx_request && signature == other.signature;
    };

    template <typename Composer> SignedTxRequest<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        // auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        SignedTxRequest<CircuitTypes<Composer>> signed_tx_request;
        signed_tx_request.tx_request = to_circuit_type(tx_request);
        // TODO: to_ct(signature) is yielding an error.
        // = {
        //     to_circuit_type(tx_request),
        //     to_ct(signature)
        // };

        return signed_tx_request;
    };

    template <typename Composer> SignedTxRequest<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Composer>(); };

        SignedTxRequest<NativeTypes> signed_tx_request = {
            to_native_type(tx_request),
        };

        return signed_tx_request;
    };
};

template <typename NCT> void read(uint8_t const*& it, SignedTxRequest<NCT>& signed_tx_request)
{
    using serialize::read;

    read(it, signed_tx_request.tx_request);
    read(it, signed_tx_request.signature);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, SignedTxRequest<NCT> const& signed_tx_request)
{
    using serialize::write;

    write(buf, signed_tx_request.tx_request);
    write(buf, signed_tx_request.signature);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, SignedTxRequest<NCT> const& signed_tx_request)
{
    return os << "tx_request:\n"
              << signed_tx_request.tx_request << "\n"
              << "signature: " << signed_tx_request.signature << "\n";
}

} // namespace aztec3::circuits::abis