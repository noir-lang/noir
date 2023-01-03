#pragma once

// #include "../state_vars/utxo_state_var.hpp"

#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::apps::notes {

// TODO: remove this `using` declaration?
// using aztec3::circuits::apps::state_vars::UTXOStateVar;

using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

// template <typename Composer, typename Note> class UTXOStateVar;
// template <typename Composer> class FunctionExecutionContext;

// template <typename NCT> struct PrivateStateNotePreimage;

template <typename Composer> class NoteInterface {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;
    typedef typename CT::address address;
    typedef typename CT::boolean boolean;

    // bool operator==(PrivateStateNote<Composer> const&) const = default;

    // UTXOStateVar<Composer, NoteInterface>* utxo_state_var;

    // NoteInterface(UTXOStateVar<Composer, NoteInterface>* utxo_state_var, NotePreimage note_preimage)
    //     : utxo_state_var(utxo_state_var)
    //     , note_preimage(note_preimage){};

    virtual ~NoteInterface() {
    } // Destructor explicitly made virtual, to ensure that the destructor of the derived class is called if the derived
      // object is deleted through a pointer to this base class. (In many places in the code, files handle
      // `NoteInterface*` pointers instead of the derived class).

    // METHODS

    virtual void remove() = 0;

    // HOOKS

    virtual fr get_commitment() = 0;

    virtual fr get_nullifier() = 0;

    // virtual grumpkin_point get_partial_commitment() const = 0;

    // virtual NotePreimage get_note_preimage() const = 0;

    // virtual NullifierPreimageTmp get_nullifier_preimage() const = 0;

    virtual fr compute_commitment() = 0;

    virtual fr compute_nullifier() = 0;

    // virtual void finalise(std::optional<fr> nonce) = 0;

    // virtual std::pair<grumpkin_point, NotePreimage> compute_partial_commitment() = 0;

    // STATIC HOOKS:

    // static fr compute_dummy_commitment() = 0;

    // static fr compute_nullifier(fr const& commitment,
    //                             fr const& owner_private_key,
    //                             boolean const& is_dummy_commitment = false) = 0;

    // static fr compute_dummy_nullifier(fr const& dummy_commitment, fr const& owner_private_key) = 0;
    // private:
    //   std::optional<NotePreimage> note_preimage;
};

} // namespace aztec3::circuits::apps::notes