#pragma once

#include "note_preimage.hpp"
#include "nullifier_preimage.hpp"
#include "../note_interface.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

// Forward-declare from this namespace in particular:
namespace aztec3::circuits::apps::state_vars {
template <typename Builder> class StateVar;
}  // namespace aztec3::circuits::apps::state_vars

namespace aztec3::circuits::apps::notes {

using aztec3::circuits::apps::state_vars::StateVar;  // Don't #include it!

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename Builder, typename ValueType> class DefaultPrivateNote : public NoteInterface<Builder> {
  public:
    using CT = CircuitTypes<Builder>;
    using fr = typename CT::fr;
    using grumpkin_point = typename CT::grumpkin_point;
    using address = typename CT::address;
    using boolean = typename CT::boolean;

    using NotePreimage = DefaultPrivateNotePreimage<CircuitTypes<Builder>, ValueType>;
    using NullifierPreimage = DefaultPrivateNoteNullifierPreimage<CircuitTypes<Builder>>;


    StateVar<Builder>* state_var;

  private:
    std::optional<fr> commitment;
    std::optional<fr> nullifier;
    std::optional<grumpkin_point> partial_commitment;

    NotePreimage note_preimage;
    std::optional<NullifierPreimage> nullifier_preimage;

  public:
    DefaultPrivateNote(StateVar<Builder>* state_var, NotePreimage note_preimage)
        : state_var(state_var), note_preimage(note_preimage){};

    ~DefaultPrivateNote() override = default;

    // OVERRIDE METHODS:

    void remove() override;

    fr get_commitment() override
    {
        if (commitment) {
            return *commitment;
        }
        return compute_commitment();
    };

    fr get_nullifier() override
    {
        if (nullifier) {
            return *nullifier;
        }
        return compute_nullifier();
    };

    // grumpkin_point get_partial_commitment() override const
    // {
    //     if (!partial_commitment) {
    //         throw_or_abort(
    //             "No partial_commitment exists for this note. Are you sure you haven't accidentally created a "
    //             "complete commitment?");
    //     }
    //     return *partial_commitment;
    // };

    void constrain_against_advice(NoteInterface<Builder> const& advice_note) override;

    bool needs_nonce() override;

    void set_nonce(fr const& nonce) override;

    fr generate_nonce() override;

    fr get_initialisation_nullifier() override
    {
        throw_or_abort(
            "DefaultPrivateNote does not support initialisation. Maybe use DefaultSingletonPrivateNote instead?");
    };

    fr get_initialisation_commitment() override
    {
        throw_or_abort(
            "DefaultPrivateNote does not support initialisation. Maybe use DefaultSingletonPrivateNote instead?");
    };

    // CUSTOM METHODS

    auto& get_oracle();

    grumpkin_point compute_partial_commitment();

    fr compute_dummy_nullifier();

    static fr compute_nullifier(fr const& commitment,
                                fr const& owner_private_key,
                                boolean const& is_dummy_commitment = false);

    NotePreimage& get_preimage() { return note_preimage; };

  private:
    fr compute_commitment();
    fr compute_nullifier();

    bool is_partial_preimage() const;
    bool is_partial_storage_slot() const;
    bool is_partial() const;
};

}  // namespace aztec3::circuits::apps::notes

// Importing in this way (rather than explicit instantiation of a template class at the bottom of a .cpp file) preserves
// the following:
// - We retain implicit instantiation of templates.
// - We don't implement method definitions in this file, to avoid a circular dependency with the state_var files (which
//   are forward-declared in this file).
#include "note.tpp"