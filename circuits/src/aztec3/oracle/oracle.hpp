#pragma once
#include <aztec3/circuits/abis/call_context.hpp>
#include <aztec3/circuits/apps/private_state_note_preimage.hpp>
#include <stdlib/types/native_types.hpp>

namespace aztec3::oracle {

using NT = plonk::stdlib::types::NativeTypes;
using aztec3::circuits::abis::CallContext;
using aztec3::circuits::apps::PrivateStateNotePreimage;
using plonk::stdlib::types::CircuitTypes;

/// Note: the server will always serve NATIVE types to the circuit, since eventually we'll be passing data to Noir (so
/// won't be handling circuit types at all from the Aztec3 end).
template <typename DB> class NativeOracleInterface {
  public:
    DB& db;

    NativeOracleInterface(DB& db, NT::fr const& contract_address, NT::address const& msg_sender)
        : db(db)
        , call_context({
              .msg_sender = msg_sender,
              .storage_contract_address = contract_address,
          }){};

    NativeOracleInterface(DB& db,
                          NT::fr const& contract_address,
                          NT::address const& msg_sender,
                          std::optional<NT::fr> msg_sender_private_key)
        : db(db)
        , call_context({
              .msg_sender = msg_sender,
              .storage_contract_address = contract_address,
          })
        , msg_sender_private_key(msg_sender_private_key){};

    NT::fr get_msg_sender_private_key()
    {
        if (!msg_sender_private_key) {
            throw_or_abort("no private key stored in memory");
        }
        if (msg_sender_private_key_already_got) {
            throw_or_abort(already_got_error);
        }
        msg_sender_private_key_already_got = true;
        return *msg_sender_private_key;
    };

    CallContext<NT> get_call_context()
    {
        if (call_context_already_got) {
            throw_or_abort(already_got_error);
        }
        call_context_already_got = true;
        return call_context;
    };

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
    std::optional<NT::fr> msg_sender_private_key;

    // Ensure functions called only once:
    bool call_context_already_got = false;
    bool msg_sender_private_key_already_got = false;
    std::string already_got_error = "Your circuit has already accessed this value. Don't ask the oracle twice, since "
                                    "it shouldn't be trusted, and could lead to circuit bugs";
};

// A temporary stub, whilst building other things first.
class FakeDB {
  public:
    FakeDB(){};

    std::pair<PrivateStateNotePreimage<NT>, PrivateStateNotePreimage<NT>>
    get_private_state_note_preimages_for_subtraction(NT::address const& contract_address,
                                                     NT::fr const& storage_slot,
                                                     NT::address const& owner,
                                                     NT::fr const& subtrahend)
    {
        if (contract_address.address_ != 0) {
            // do nothing - just making these variables in-use
        }

        NT::grumpkin_point slot_point;
        NT::fr x = storage_slot;
        NT::fr yy = x.sqr() * x + NT::grumpkin_group::curve_b;
        NT::fr y = yy.sqrt();
        NT::fr neg_y = -y;
        y = y < neg_y ? y : neg_y;
        slot_point = NT::grumpkin_group::affine_element(x, y);
        info("derived slot point:", slot_point);

        return std::make_pair(
            PrivateStateNotePreimage<NT>{
                .start_slot = 0,
                .slot_point = slot_point,
                .value = uint256_t(subtrahend) / 2 + 1,
                .owner_address = owner,
                .creator_address = 0,
                .salt = 1234,
                .input_nullifier = 2345,
                .memo = 3456,
                .is_real = true,
            },
            PrivateStateNotePreimage<NT>{
                .start_slot = 0,
                .slot_point = slot_point,
                .value = uint256_t(subtrahend) / 2 + 3,
                .owner_address = owner,
                .creator_address = 0,
                .salt = 4567,
                .input_nullifier = 5678,
                .memo = 6789,
                .is_real = true,
            });
    };
};

typedef NativeOracleInterface<FakeDB> NativeOracle;

} // namespace aztec3::oracle