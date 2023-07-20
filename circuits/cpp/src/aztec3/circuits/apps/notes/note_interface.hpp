#pragma once

#include "aztec3/utils/types/circuit_types.hpp"

namespace aztec3::circuits::apps::notes {

using aztec3::utils::types::CircuitTypes;

/**
 * Note: The methods in this interface must be implemented by the derived Note types, even if such note types don't
 * require such functions.
 *
 * It's essentially a visitor pattern. The Opcodes and UTXOStateVar types can call all of these
 * methods on any Note, and the Note must choose whether to compute something, or whether to throw, for each method.
 */
template <typename Builder> class NoteInterface {
  public:
    using CT = CircuitTypes<Builder>;
    using fr = typename CT::fr;
    using grumpkin_point = typename CT::grumpkin_point;
    using address = typename CT::address;
    using boolean = typename CT::boolean;

    // Destructor explicitly made virtual, to ensure that the destructor of the derived class is called if the derived
    // object is deleted through a pointer to this base class. (In many places in the code, files handle
    // `NoteInterface*` pointers instead of the derived class).
    virtual ~NoteInterface() = default;

    // TODO: maybe rather than have this be a pure interface, we should have a constructor and the `state_var*` and
    // `note_preimage` members here (although that would require a NotePreimage template param).
    // This is all because the Opcodes actually _assume_ a particular constructor layout for each Note, as well as
    // _assume_ those two data members are always present. Having said that, there's still no way to actually enforce a
    // constructor function data of a derived class.

    virtual void remove() = 0;

    virtual fr get_commitment() = 0;

    virtual fr get_nullifier() = 0;

    virtual fr get_initialisation_nullifier() = 0;

    virtual fr get_initialisation_commitment() = 0;

    virtual void constrain_against_advice(NoteInterface<Builder> const& advice_note) = 0;

    virtual bool needs_nonce() = 0;

    virtual void set_nonce(fr const& nonce) = 0;

    virtual fr generate_nonce() = 0;
};

}  // namespace aztec3::circuits::apps::notes