#pragma once

#include <aztec3/circuits/abis/call_context.hpp>

#include <aztec3/circuits/apps/private_state_note_preimage.hpp>
#include <aztec3/circuits/apps/utxo_datum.hpp>

#include <aztec3/circuits/apps/notes/default_private_note/note_preimage.hpp>

#include <stdlib/types/native_types.hpp>

namespace aztec3::oracle {

using aztec3::circuits::abis::CallContext;

using aztec3::circuits::apps::PrivateStateNotePreimage;
using aztec3::circuits::apps::UTXOSLoadDatum;

using aztec3::circuits::apps::notes::DefaultPrivateNotePreimage;

using plonk::stdlib::types::CircuitTypes;
using NT = plonk::stdlib::types::NativeTypes;

/// Note: the server will always serve NATIVE types to the circuit, since eventually we'll be passing data to Noir (so
/// won't be handling circuit types at all from the Aztec3 end).
template <typename DB> class NativeOracleInterface {
  public:
    DB& db;

    NativeOracleInterface(DB& db,
                          NT::fr const& contract_address,
                          //   NT::fr const& portal_contract_address,
                          NT::address const& msg_sender,
                          NT::address const& tx_origin,
                          NT::boolean const& is_delegate_call = false,
                          NT::boolean const& is_static_call = false,
                          NT::fr const& reference_block_num = 0)
        : db(db)
        , call_context({
              .msg_sender = msg_sender,
              .storage_contract_address = contract_address,
              .tx_origin = tx_origin,
              .is_delegate_call = is_delegate_call,
              .is_static_call = is_static_call,
              .reference_block_num = reference_block_num,
          })
        // , portal_contract_address(portal_contract_address)
        {};

    // Include the msg_sender_private_key
    NativeOracleInterface(DB& db,
                          NT::fr const& contract_address,
                          //   NT::fr const& portal_contract_address,
                          NT::address const& msg_sender,
                          NT::address const& tx_origin,
                          std::optional<NT::fr> msg_sender_private_key,
                          NT::boolean const& is_delegate_call = false,
                          NT::boolean const& is_static_call = false,
                          NT::fr const& reference_block_num = 0)
        : db(db)
        , call_context({
              .msg_sender = msg_sender,
              .storage_contract_address = contract_address,
              .tx_origin = tx_origin,
              .is_delegate_call = is_delegate_call,
              .is_static_call = is_static_call,
              .reference_block_num = reference_block_num,
          })
        // , portal_contract_address(portal_contract_address)
        , msg_sender_private_key(msg_sender_private_key){};

    // CallContext as struct
    NativeOracleInterface(DB& db, CallContext<NT> call_context, std::optional<NT::fr> msg_sender_private_key)
        : db(db)
        , call_context(call_context)
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

    CallContext<NT> get_call_context()
    {
        if (call_context_already_got) {
            throw_or_abort("call_context: " + already_got_error);
        }
        call_context_already_got = true;
        return call_context;
    };

    template <typename NotePreimage>
    UTXOSLoadDatum<NT, NotePreimage> get_utxo_sload_datum(NT::grumpkin_point const storage_slot_point,
                                                          NotePreimage const advice)
    {
        // TODO: consider whether it's actually safe to bypass get_call_context() here...
        const auto& contract_address = call_context.storage_contract_address;
        return db.get_utxo_sload_datum(contract_address, storage_slot_point, advice);
    }

    std::pair<PrivateStateNotePreimage<NT>, PrivateStateNotePreimage<NT>>
    get_private_state_note_preimages_for_subtraction(NT::fr const& storage_slot,
                                                     NT::address const& owner,
                                                     NT::fr const& subtrahend)
    {
        const auto& contract_address = call_context.storage_contract_address;
        return db.get_private_state_note_preimages_for_subtraction(contract_address, storage_slot, owner, subtrahend);
    }

    NT::fr generate_salt() const { return NT::fr::random_element(); }

    NT::fr generate_random_element() const { return NT::fr::random_element(); }

  private:
    // We MUST make these values private, so the circuit isn't able to `get` these values more than once (the getter
    // functions can check this). This will help us write secure circuits. If we were to query the same thing twice, an
    // untrustworthy oracle could give two different pieces of information. As long as this (trusted) oracle catches
    // double-queries, we can ensure the circuit we build doesn't query twice.

    // A circuit doesn't know its own address, so we need to track the address from 'outside'.
    CallContext<NT> call_context;
    // NT::fr portal_contract_address;
    std::optional<NT::fr> msg_sender_private_key;

    // Ensure functions called only once:
    bool call_context_already_got = false;
    // bool portal_contract_address_already_got = false;
    bool msg_sender_private_key_already_got = false;
    std::string already_got_error = "Your circuit has already accessed this value. Don't ask the oracle twice, since "
                                    "it shouldn't be trusted, and could lead to circuit bugs";
};

// A temporary stub, whilst building other things first.
class FakeDB {
  public:
    FakeDB(){};

    /**
     * For getting a singleton UTXO (not a set).
     */
    UTXOSLoadDatum<NT, DefaultPrivateNotePreimage<NT, typename NT::fr>> get_utxo_sload_datum(
        NT::address const& contract_address,
        NT::grumpkin_point const& storage_slot_point,
        DefaultPrivateNotePreimage<NT, typename NT::fr> const& advice)
    // NT::address const& owner,
    // NT::fr required_utxo_tree_root,
    // size_t utxo_tree_depth)
    {
        (void)storage_slot_point; // Not used in this 'fake' implementation.

        DefaultPrivateNotePreimage<NT, NT::fr> preimage{
            .value = 100,
            .owner = advice.owner,
            .creator_address = 0,
            .memo = 3456,
            .salt = 1234,
            .nonce = 2345,
            .is_dummy = false,
        };

        const size_t utxo_tree_depth = 32;
        const NT::fr required_utxo_tree_root = 2468;

        std::vector<NT::fr> sibling_path(utxo_tree_depth);
        std::fill(sibling_path.begin(), sibling_path.end(), 1); // Fill with 1's to be lazy. TODO: return a valid path.

        return {
            .commitment = 1, // TODO: implement commit() method in `preimage`?
            .contract_address = contract_address,
            .preimage = preimage,

            .sibling_path = sibling_path,
            .leaf_index = 2,
            .old_private_data_tree_root = required_utxo_tree_root,
        };
    };

    std::pair<PrivateStateNotePreimage<NT>, PrivateStateNotePreimage<NT>>
    get_private_state_note_preimages_for_subtraction(NT::address const& contract_address,
                                                     NT::fr const& storage_slot,
                                                     NT::address const& owner,
                                                     NT::fr const& subtrahend)
    {
        (void)contract_address; // currently unused

        NT::grumpkin_point storage_slot_point;
        NT::fr x = storage_slot;
        NT::fr yy = x.sqr() * x + NT::grumpkin_group::curve_b;
        NT::fr y = yy.sqrt();
        NT::fr neg_y = -y;
        y = y < neg_y ? y : neg_y;
        storage_slot_point = NT::grumpkin_group::affine_element(x, y);
        info("derived slot point:", storage_slot_point);

        return std::make_pair(
            PrivateStateNotePreimage<NT>{
                .start_slot = 0,
                .storage_slot_point = storage_slot_point,
                .value = uint256_t(subtrahend) / 2 + 1,
                .owner = owner,
                .creator_address = 0,
                .memo = 3456,
                .salt = 1234,
                .nonce = 2345,
                .is_real = true,
            },
            PrivateStateNotePreimage<NT>{
                .start_slot = 0,
                .storage_slot_point = storage_slot_point,
                .value = uint256_t(subtrahend) / 2 + 3,
                .owner = owner,
                .creator_address = 0,
                .memo = 6789,
                .salt = 4567,
                .nonce = 5678,
                .is_real = true,
            });
    };
};

typedef NativeOracleInterface<FakeDB> NativeOracle;

} // namespace aztec3::oracle