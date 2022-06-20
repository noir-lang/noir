#pragma once
#include <aztec3/constants.hpp>
#include <stdlib/types/convert.hpp>
#include "nullifier_preimage.hpp"
#include "private_state_note.hpp"
#include "private_state_var.hpp"
#include "oracle_wrapper.hpp"

namespace aztec3::circuits::apps {

// using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using NT = plonk::stdlib::types::NativeTypes;

template <typename Composer> class PrivateStateFactory {
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;

  public:
    Composer& composer; // TODO: can we remove this?
    OracleWrapperInterface<Composer>& oracle;
    const std::string contract_name;
    fr private_state_counter = 0;

    std::map<std::string, PrivateStateVar<Composer>> private_state_vars;
    std::vector<PrivateStateNote<Composer>> new_private_state_notes;
    std::vector<fr> new_commitments;
    std::vector<NullifierPreimage<CT>> new_nullifier_preimages;
    std::vector<fr> new_nullifiers;

    PrivateStateFactory<Composer>(Composer& composer,
                                  OracleWrapperInterface<Composer>& oracle,
                                  std::string contract_name)
        : composer(composer)
        , oracle(oracle)
        , contract_name(contract_name){};

    PrivateStateVar<Composer>& new_private_state(std::string const& name,
                                                 PrivateStateType const& private_state_type = PARTITIONED)
    {
        if (private_state_vars.contains(name)) {
            throw_or_abort("name already exists");
        }
        PrivateStateVar<Composer> private_state_var =
            PrivateStateVar<Composer>(this, private_state_type, name, private_state_counter++);
        private_state_vars.insert(std::make_pair(name, private_state_var));
        return private_state_vars[name];
    };

    // For initialising a private state which is a mapping.
    PrivateStateVar<Composer>& new_private_state(std::string const& name,
                                                 std::vector<std::string> const& mapping_key_names,
                                                 PrivateStateType const& private_state_type = PARTITIONED)
    {
        if (private_state_vars.contains(name)) {
            throw_or_abort("name already exists");
        }
        PrivateStateVar<Composer> private_state_var =
            PrivateStateVar<Composer>(this, private_state_type, name, private_state_counter++, mapping_key_names);
        private_state_vars.insert(std::make_pair(name, private_state_var));
        return private_state_vars[name];
    };

    void finalise()
    {

        if (new_private_state_notes.size() > new_nullifiers.size()) {
            // We've created more commitments than nullifiers so far. But we want to inject an input_nullifier into each
            // new commitment. So, let's create more dummy nullifiers.
            const auto& msg_sender_private_key = oracle.get_msg_sender_private_key();
            for (size_t i = new_nullifiers.size(); i < new_private_state_notes.size(); ++i) {
                auto dummy_commitment = oracle.generate_random_element();
                new_nullifiers.push_back(
                    PrivateStateNote<Composer>::compute_dummy_nullifier(dummy_commitment, msg_sender_private_key));
                new_nullifier_preimages.push_back(NullifierPreimage<CT>{
                    dummy_commitment,
                    msg_sender_private_key,
                    false,
                });
            }
        }
        for (size_t i = 0; i < new_private_state_notes.size(); ++i) {
            new_private_state_notes[i].preimage.input_nullifier = new_nullifiers[i];
            new_commitments.push_back(new_private_state_notes[i].compute_commitment());
        }
    }

    PrivateStateVar<Composer>& get(std::string const& name)
    {
        if (!private_state_vars.contains(name)) {
            throw_or_abort("name not found");
        }
        return private_state_vars[name];
    };

    void push_new_note(PrivateStateNote<Composer> const private_state_note)
    {
        new_private_state_notes.push_back(private_state_note);
    }

    void push_new_nullifier_data(fr nullifier, NullifierPreimage<CT> nullifier_preimage)
    {
        new_nullifiers.push_back(nullifier);
        new_nullifier_preimages.push_back(nullifier_preimage);
    }

    std::vector<PrivateStateNote<Composer>> get_notes() { return new_private_state_notes; }
};

} // namespace aztec3::circuits::apps