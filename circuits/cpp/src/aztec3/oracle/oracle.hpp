#pragma once

#include "fake_db.hpp"

#include <aztec3/circuits/abis/call_context.hpp>
#include <aztec3/circuits/abis/function_data.hpp>
#include <aztec3/circuits/abis/contract_deployment_data.hpp>

#include <aztec3/circuits/apps/utxo_datum.hpp>

#include <aztec3/circuits/apps/notes/default_private_note/note_preimage.hpp>
#include <aztec3/circuits/apps/notes/default_singleton_private_note/note_preimage.hpp>

#include <aztec3/utils/types/native_types.hpp>

namespace aztec3::oracle {

using aztec3::circuits::abis::CallContext;
using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::circuits::abis::FunctionData;

using aztec3::circuits::apps::UTXOSLoadDatum;

using aztec3::utils::types::CircuitTypes;
using NT = aztec3::utils::types::NativeTypes;

/// Note: the server will always serve NATIVE types to the circuit, since eventually we'll be passing data to Noir (so
/// won't be handling circuit types at all from the Aztec3 end).
template <typename DB> class NativeOracleInterface {
  public:
    DB& db;

    NativeOracleInterface(DB& db,
                          NT::address const& actual_contract_address,
                          FunctionData<NT> const& function_data,
                          CallContext<NT> const& call_context,
                          std::optional<NT::fr> const& msg_sender_private_key = std::nullopt)
        : db(db)
        , actual_contract_address(actual_contract_address)
        , function_data(function_data)
        , call_context(call_context)
        // , portal_contract_address(portal_contract_address)
        , msg_sender_private_key(msg_sender_private_key){};

    NativeOracleInterface(DB& db,
                          NT::address const& actual_contract_address,
                          FunctionData<NT> const& function_data,
                          CallContext<NT> const& call_context,
                          ContractDeploymentData<NT> const& contract_deployment_data,
                          std::optional<NT::fr> const& msg_sender_private_key = std::nullopt)
        : db(db)
        , actual_contract_address(actual_contract_address)
        , function_data(function_data)
        , call_context(call_context)
        , contract_deployment_data(contract_deployment_data)
        // , portal_contract_address(portal_contract_address)
        , msg_sender_private_key(msg_sender_private_key){};

    NT::fr get_msg_sender_private_key()
    {
        if (!msg_sender_private_key) {
            throw_or_abort("no private key stored in memory");
        }
        if (msg_sender_private_key_already_got) {
            throw_or_abort("msg_sender_private_key: " + already_got_error);
        }
        msg_sender_private_key_already_got = true;
        return *msg_sender_private_key;
    };

    // NT::fr get_portal_contract_address()
    // {
    //     if (portal_contract_address_already_got) {
    //         throw_or_abort(already_got_error);
    //     }
    //     portal_contract_address_already_got = true;
    //     return portal_contract_address;
    // };

    NT::address get_actual_contract_address() { return actual_contract_address; };

    FunctionData<NT> get_function_data() { return function_data; };

    CallContext<NT> get_call_context()
    {
        if (call_context_already_got) {
            throw_or_abort("call_context: " + already_got_error);
        }
        call_context_already_got = true;
        return call_context;
    };

    ContractDeploymentData<NT> get_contract_deployment_data()
    {
        if (contract_deployment_data_already_got) {
            throw_or_abort("contract_deployment_data: " + already_got_error);
        }
        contract_deployment_data_already_got = true;
        return contract_deployment_data;
    };

    template <typename NotePreimage>
    UTXOSLoadDatum<NT, NotePreimage> get_utxo_sload_datum(NT::grumpkin_point const storage_slot_point,
                                                          NotePreimage const advice)
    {
        // TODO: consider whether it's actually safe to bypass get_call_context() here...
        const auto& contract_address = call_context.storage_contract_address;
        return db.get_utxo_sload_datum(contract_address, storage_slot_point, advice);
    }

    template <typename NotePreimage>
    std::vector<UTXOSLoadDatum<NT, NotePreimage>> get_utxo_sload_data(NT::grumpkin_point const storage_slot_point,
                                                                      size_t const& num_notes,
                                                                      NotePreimage const advice)
    {
        // TODO: consider whether it's actually safe to bypass get_call_context() here...
        const auto& contract_address = call_context.storage_contract_address;
        return db.get_utxo_sload_data(contract_address, storage_slot_point, num_notes, advice);
    }

    NT::fr generate_salt() const { return NT::fr::random_element(); }

    NT::fr generate_random_element() const { return NT::fr::random_element(); }

  private:
    // We MUST make these values private, so the circuit isn't able to `get` these values more than once (the getter
    // functions can check this). This will help us write secure circuits. If we were to query the same thing twice, an
    // untrustworthy oracle could give two different pieces of information. As long as this (trusted) oracle catches
    // double-queries, we can ensure the circuit we build doesn't query twice.

    // Note: actual_contract_address and function_data are NOT to be provided to the circuit, so don't include
    // getter methods for these in the OracleWrapper.
    NT::address actual_contract_address; // not to be confused with call_context.storage_contract_address;
    FunctionData<NT> function_data;

    CallContext<NT> call_context;
    ContractDeploymentData<NT> contract_deployment_data;
    // NT::fr portal_contract_address;
    std::optional<NT::fr> msg_sender_private_key;

    // Ensure functions called only once:
    bool actual_contract_address_already_got = false;
    bool function_data_already_got = false;
    bool call_context_already_got = false;
    bool contract_deployment_data_already_got = false;
    // bool portal_contract_address_already_got = false;
    bool msg_sender_private_key_already_got = false;
    std::string already_got_error = "Your circuit has already accessed this value. Don't ask the oracle twice, since "
                                    "it shouldn't be trusted, and could lead to circuit bugs";
};

typedef NativeOracleInterface<FakeDB> NativeOracle;

} // namespace aztec3::oracle