#pragma once

#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::apps::notes {

using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename Composer> class NoteInterface {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;
    typedef typename CT::address address;
    typedef typename CT::boolean boolean;

    // Destructor explicitly made virtual, to ensure that the destructor of the derived class is called if the derived
    // object is deleted through a pointer to this base class. (In many places in the code, files handle
    // `NoteInterface*` pointers instead of the derived class).
    virtual ~NoteInterface() {}

    virtual void remove() = 0;

    virtual fr get_commitment() = 0;

    virtual fr get_nullifier() = 0;

    virtual fr compute_commitment() = 0;

    virtual fr compute_nullifier() = 0;

    virtual void constrain_against_advice(NoteInterface<Composer> const& advice_note) = 0;

    virtual bool needs_nonce() = 0;

    virtual void set_nonce(fr const& nonce) = 0;

    virtual fr generate_nonce() = 0;
};

} // namespace aztec3::circuits::apps::notes