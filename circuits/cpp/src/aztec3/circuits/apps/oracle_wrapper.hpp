#pragma once
#include <aztec3/circuits/abis/call_context.hpp>
#include <aztec3/circuits/abis/contract_deployment_data.hpp>
#include <aztec3/oracle/oracle.hpp>

#include <barretenberg/common/map.hpp>

#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::apps {

using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::CallContext;
using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::oracle::NativeOracle;
using aztec3::utils::types::CircuitTypes;

/**
 * The main purpose of this wrapper is to:
 * - cache values which have been already given by the oracle previously during this execution;
 * - convert Native types (returned by the oracle) into circuit types, using the composer instance.
 * Note: Insecure circuits could be built if the same value is queried twice from the oracle (since a malicious prover
 * could provide two different witnesses for a single thing). The Native oracle will throw if you try a double-query of
 * certain information.
 */
template <typename Composer> class OracleWrapperInterface {
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;
    typedef typename CT::address address;

  public:
    Composer& composer;
    NativeOracle& native_oracle;

    // Initialise from Native.
    // Used when initialising for a user's first call.
    OracleWrapperInterface(Composer& composer, NativeOracle& native_oracle)
        : composer(composer)
        , native_oracle(native_oracle){};

    fr& get_msg_sender_private_key()
    {
        if (msg_sender_private_key) {
            return *msg_sender_private_key;
        }
        msg_sender_private_key = aztec3::utils::types::to_ct(composer, native_oracle.get_msg_sender_private_key());
        validate_msg_sender_private_key();
        return *msg_sender_private_key;
    };

    address get_contract_address() { return get_call_context().storage_contract_address; };

    CallContext<CT>& get_call_context()
    {
        if (call_context) {
            return *call_context;
        }
        call_context = native_oracle.get_call_context().to_circuit_type(composer);
        return *call_context;
    };

    ContractDeploymentData<CT>& get_contract_deployment_data()
    {
        if (contract_deployment_data) {
            return *contract_deployment_data;
        }
        contract_deployment_data = native_oracle.get_contract_deployment_data().to_circuit_type(composer);
        return *contract_deployment_data;
    };

    address& get_msg_sender() { return get_call_context().msg_sender; };

    address& get_this_contract_address() { return get_call_context().storage_contract_address; };

    address& get_tx_origin() { return get_call_context().tx_origin; };

    fr generate_salt() const { return aztec3::utils::types::to_ct(composer, native_oracle.generate_salt()); }

    fr generate_random_element() const
    {
        return aztec3::utils::types::to_ct(composer, native_oracle.generate_random_element());
    }

    template <typename NotePreimage>
    auto get_utxo_sload_datum(grumpkin_point const& storage_slot_point, NotePreimage const& advice)
    {
        auto native_storage_slot_point = aztec3::utils::types::to_nt<Composer>(storage_slot_point);

        auto native_advice = advice.template to_native_type<Composer>();

        auto native_utxo_sload_datum = native_oracle.get_utxo_sload_datum(native_storage_slot_point, native_advice);

        return native_utxo_sload_datum.to_circuit_type(composer);
    }

    template <typename NotePreimage>
    auto get_utxo_sload_data(grumpkin_point const& storage_slot_point,
                             size_t const& num_notes,
                             NotePreimage const& advice)
    {
        auto native_storage_slot_point = aztec3::utils::types::to_nt<Composer>(storage_slot_point);

        auto native_advice = advice.template to_native_type<Composer>();

        auto native_utxo_sload_data =
            native_oracle.get_utxo_sload_data(native_storage_slot_point, num_notes, native_advice);

        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        return map(native_utxo_sload_data, to_circuit_type);
    }

  private:
    std::optional<CallContext<CT>> call_context;
    std::optional<ContractDeploymentData<CT>> contract_deployment_data;
    std::optional<fr> msg_sender_private_key;

    void validate_msg_sender_private_key()
    {
        address msg_sender = get_msg_sender();
        address derived_msg_sender = address::derive_from_private_key(*msg_sender_private_key);
        msg_sender.assert_equal(derived_msg_sender,
                                format("msg_sender validation failed.\nmsg_sender_private_key: ",
                                       msg_sender_private_key,
                                       "\nPurported msg_sender: ",
                                       msg_sender,
                                       "\nDerived msg_sender: ",
                                       derived_msg_sender));
    }
};

} // namespace aztec3::circuits::apps