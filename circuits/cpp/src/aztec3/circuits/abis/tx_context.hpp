#pragma once
#include "contract_deployment_data.hpp"
#include "function_data.hpp"
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include <crypto/generators/generator_data.hpp>
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

    boolean is_fee_payment_tx = false;
    boolean is_rebate_payment_tx = false;
    boolean is_contract_deployment_tx = false;

    ContractDeploymentData<NCT> contract_deployment_data;

    fr reference_block_num;

    template <typename Composer> TxContext<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };
        // auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        TxContext<CircuitTypes<Composer>> tx_context = {
            to_ct(is_fee_payment_tx),         to_ct(is_rebate_payment_tx),
            to_ct(is_contract_deployment_tx), contract_deployment_data.to_circuit_type(composer),
            to_ct(reference_block_num),
        };

        return tx_context;
    };

    template <typename Composer> TxContext<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return plonk::stdlib::types::to_nt<Composer>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Composer>(); };

        TxContext<NativeTypes> tx_context = {
            to_nt(is_fee_payment_tx),         to_nt(is_rebate_payment_tx),
            to_nt(is_contract_deployment_tx), to_native_type(contract_deployment_data),
            to_nt(reference_block_num),
        };

        return tx_context;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        fr(is_fee_payment_tx).set_public();
        fr(is_rebate_payment_tx).set_public();
        fr(is_contract_deployment_tx).set_public();
        contract_deployment_data.set_public();
        reference_block_num.set_public();
    }

    fr hash() const
    {
        std::vector<fr> inputs = {
            fr(is_fee_payment_tx),           fr(is_rebate_payment_tx), fr(is_contract_deployment_tx),
            contract_deployment_data.hash(), reference_block_num,
        };

        return NCT::compress(inputs, GeneratorIndex::TX_CONTEXT);
    }
};

template <typename NCT> void read(uint8_t const*& it, TxContext<NCT>& tx_context)
{
    using serialize::read;

    read(it, tx_context.is_fee_payment_tx);
    read(it, tx_context.is_rebate_payment_tx);
    read(it, tx_context.is_contract_deployment_tx);
    read(it, tx_context.contract_deployment_data);
    read(it, tx_context.reference_block_num);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, TxContext<NCT> const& tx_context)
{
    using serialize::write;

    write(buf, tx_context.is_fee_payment_tx);
    write(buf, tx_context.is_rebate_payment_tx);
    write(buf, tx_context.is_contract_deployment_tx);
    write(buf, tx_context.contract_deployment_data);
    write(buf, tx_context.reference_block_num);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, TxContext<NCT> const& tx_context)
{
    return os << "is_fee_payment_tx: " << tx_context.is_fee_payment_tx << "\n"
              << "is_rebate_payment_tx: " << tx_context.is_rebate_payment_tx << "\n"
              << "is_contract_deployment_tx: " << tx_context.is_contract_deployment_tx << "\n"
              << "contract_deployment_data: " << tx_context.contract_deployment_data << "\n"
              << "reference_block_num: " << tx_context.reference_block_num << "\n";
}

} // namespace aztec3::circuits::abis