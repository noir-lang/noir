#pragma once
#include <aztec3/circuits/abis/call_context.hpp>
#include <aztec3/circuits/apps/private_state_note_preimage.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include <aztec3/oracle/oracle.hpp>

namespace aztec3::circuits::apps {

using NT = plonk::stdlib::types::NativeTypes;
using aztec3::circuits::abis::CallContext;
using aztec3::oracle::NativeOracle;
using plonk::stdlib::types::CircuitTypes;

/**
 * The main purpose of this wrapper is to cache values which have been already given by the oracle. Insecure circuits
 * could be built if the same value is queried twice from the oracle (since a malicious prover could provide two
 * different witnesses for a single thing).
 */
template <typename Composer> class OracleWrapperInterface {
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::address address;

  public:
    Composer& composer;
    NativeOracle& oracle;

    // Initialise from Native.
    // Used when initialising for a user's first call.
    OracleWrapperInterface(Composer& composer, NativeOracle& oracle)
        : composer(composer)
        , oracle(oracle){};

    fr& get_msg_sender_private_key()
    {
        if (msg_sender_private_key) {
            return *msg_sender_private_key;
        }
        msg_sender_private_key = plonk::stdlib::types::to_ct(composer, oracle.get_msg_sender_private_key());
        validate_msg_sender_private_key();
        return *msg_sender_private_key;
    };

    CallContext<CT>& get_call_context()
    {
        if (call_context) {
            return *call_context;
        }
        call_context = oracle.get_call_context().to_circuit_type(composer);
        return *call_context;
    };

    address& get_msg_sender() { return get_call_context().msg_sender; };

    address& get_this_contract_address() { return get_call_context().storage_contract_address; };

    address& get_tx_origin() { return get_call_context().tx_origin; };

    fr generate_salt() const { return plonk::stdlib::types::to_ct(composer, oracle.generate_salt()); }

    fr generate_random_element() const
    {
        return plonk::stdlib::types::to_ct(composer, oracle.generate_random_element());
    }

    std::pair<PrivateStateNotePreimage<CT>, PrivateStateNotePreimage<CT>>
    get_private_state_note_preimages_for_subtraction(fr const& storage_slot, address const& owner, fr const& subtrahend)
    {
        std::pair<PrivateStateNotePreimage<NT>, PrivateStateNotePreimage<NT>> native_preimages =
            oracle.get_private_state_note_preimages_for_subtraction(
                storage_slot.get_value(), NT::address(owner.to_field().get_value()), subtrahend.get_value());
        return std::make_pair(native_preimages.first.to_circuit_type(composer),
                              native_preimages.second.to_circuit_type(composer));
    }

  private:
    std::optional<CallContext<CT>> call_context;
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