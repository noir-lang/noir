#pragma once

#include "tx_object.hpp"

#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct SignedTxObject {
    TxObject<NCT> tx_object;
    // Signature<NCT> signature; // TODO: import some kind of signature

    template <typename Composer> SignedTxObject<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        // auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        SignedTxObject<CircuitTypes<Composer>> signed_tx_object = {
            to_circuit_type(tx_object),
        };

        return signed_tx_object;
    };
};

template <typename NCT> void read(uint8_t const*& it, SignedTxObject<NCT>& signed_tx_object)
{
    using serialize::read;

    read(it, signed_tx_object.tx_object);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, SignedTxObject<NCT> const& signed_tx_object)
{
    using serialize::write;

    write(buf, signed_tx_object.tx_object);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, SignedTxObject<NCT> const& signed_tx_object)
{
    return os << "tx_object: " << signed_tx_object.tx_object << "\n";
}

} // namespace aztec3::circuits::abis