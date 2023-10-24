#pragma once
#include "contract_deployment_data.hpp"
#include "function_data.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct TxContext {
    using address = typename NCT::address;
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    boolean is_fee_payment_tx = false;
    boolean is_rebate_payment_tx = false;
    boolean is_contract_deployment_tx = false;

    ContractDeploymentData<NCT> contract_deployment_data{};

    fr chain_id = 0;
    fr version = 0;

    // for serialization: update up with new fields
    MSGPACK_FIELDS(is_fee_payment_tx,
                   is_rebate_payment_tx,
                   is_contract_deployment_tx,
                   contract_deployment_data,
                   chain_id,
                   version);
    boolean operator==(TxContext<NCT> const& other) const
    {
        return is_fee_payment_tx == other.is_fee_payment_tx && is_rebate_payment_tx == other.is_rebate_payment_tx &&
               is_contract_deployment_tx == other.is_contract_deployment_tx &&
               contract_deployment_data == other.contract_deployment_data && chain_id == other.chain_id &&
               version == other.version;
    };

    template <typename Builder> TxContext<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };
        // auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(builder); };

        TxContext<CircuitTypes<Builder>> tx_context = {
            to_ct(is_fee_payment_tx),
            to_ct(is_rebate_payment_tx),
            to_ct(is_contract_deployment_tx),
            contract_deployment_data.to_circuit_type(builder),
            to_ct(chain_id),
            to_ct(version),
        };

        return tx_context;
    };

    template <typename Builder> TxContext<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Builder>(); };

        TxContext<NativeTypes> tx_context = { to_nt(is_fee_payment_tx),
                                              to_nt(is_rebate_payment_tx),
                                              to_nt(is_contract_deployment_tx),
                                              to_native_type(contract_deployment_data),
                                              to_nt(chain_id),
                                              to_nt(version) };

        return tx_context;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        fr(is_fee_payment_tx).set_public();
        fr(is_rebate_payment_tx).set_public();
        fr(is_contract_deployment_tx).set_public();
        contract_deployment_data.set_public();
        chain_id.set_public();
        version.set_public();
    }

    fr hash() const
    {
        std::vector<fr> const inputs = {
            fr(is_fee_payment_tx),
            fr(is_rebate_payment_tx),
            fr(is_contract_deployment_tx),
            contract_deployment_data.hash(),
            chain_id,
            version,
        };

        return NCT::hash(inputs, GeneratorIndex::TX_CONTEXT);
    }
};

}  // namespace aztec3::circuits::abis
