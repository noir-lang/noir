#pragma once
#include "function_signature.hpp"
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include <crypto/pedersen/generator_data.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct TxContext {
    typedef typename NCT::address address;
    // typedef typename NCT::grumpkin_point grumpkin_point;
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    boolean called_from_l1;
    boolean called_from_public_l2;
    boolean is_fee_payment_tx;
    // FeeData<NCT> fee_data;
    fr reference_block_num;

    template <typename Composer> TxContext<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };
        // auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        TxContext<CircuitTypes<Composer>> tx_context = {
            to_ct(called_from_l1),
            to_ct(called_from_public_l2),
            to_ct(is_fee_payment_tx),
            to_ct(reference_block_num),
        };

        return tx_context;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        fr(called_from_l1).set_public();
        fr(called_from_public_l2).set_public();
        fr(is_fee_payment_tx).set_public();
        reference_block_num.set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, TxContext<NCT>& tx_context)
{
    using serialize::read;

    read(it, tx_context.called_from_l1);
    read(it, tx_context.called_from_public_l2);
    read(it, tx_context.is_fee_payment_tx);
    read(it, tx_context.reference_block_num);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, TxContext<NCT> const& tx_context)
{
    using serialize::write;

    write(buf, tx_context.called_from_l1);
    write(buf, tx_context.called_from_public_l2);
    write(buf, tx_context.is_fee_payment_tx);
    write(buf, tx_context.reference_block_num);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, TxContext<NCT> const& tx_context)
{
    return os << "called_from_l1: " << tx_context.called_from_l1 << "\n"
              << "called_from_public_l2: " << tx_context.called_from_public_l2 << "\n"
              << "is_fee_payment_tx: " << tx_context.is_fee_payment_tx << "\n"
              << "reference_block_num: " << tx_context.reference_block_num << "\n";
}

} // namespace aztec3::circuits::abis