#pragma once

#include "../note_interface.hpp"

#include "note_preimage.hpp"
#include "nullifier_preimage.hpp"

// #include "../../state_vars/utxo_state_var.hpp"

#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

// Forward-declare from this namespace in particular:
namespace aztec3::circuits::apps::state_vars {
template <typename Composer, typename V> class UTXOStateVar;
} // namespace aztec3::circuits::apps::state_vars

namespace aztec3::circuits::apps::notes {

using aztec3::circuits::apps::state_vars::UTXOStateVar; // Don't #include it!

using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

// template <typename NCT, typename V> struct DefaultPrivateNotePreimage;
// template <typename NCT, typename V> struct DefaultPrivateNoteNullifierPreimage;

template <typename Composer, typename ValueType> class DefaultPrivateNote : public NoteInterface<Composer> {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;
    typedef typename CT::address address;
    typedef typename CT::boolean boolean;

    using NotePreimage = DefaultPrivateNotePreimage<CircuitTypes<Composer>, ValueType>;
    using NullifierPreimage = DefaultPrivateNoteNullifierPreimage<CircuitTypes<Composer>>;

  public:
    UTXOStateVar<Composer, DefaultPrivateNote>* utxo_state_var;

  private:
    std::optional<fr> commitment;
    std::optional<fr> nullifier;
    std::optional<grumpkin_point> partial_commitment;

    // bool is_partial = false;

    NotePreimage note_preimage;
    std::optional<NullifierPreimage> nullifier_preimage;

  public:
    DefaultPrivateNote(UTXOStateVar<Composer, DefaultPrivateNote>* utxo_state_var, NotePreimage note_preimage)
        : utxo_state_var(utxo_state_var)
        , note_preimage(note_preimage){};

    ~DefaultPrivateNote() {}

    // bool operator==(PrivateStateNote<Composer> const&) const = default;

    // METHODS

    void remove() override;

    // HOOKS

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

    fr compute_commitment() override;

    fr compute_nullifier() override;

    // void finalise(std::optional<fr> nonce) override;

    auto& get_oracle();

    grumpkin_point compute_partial_commitment();

    fr compute_dummy_nullifier();

    static fr compute_nullifier(fr const& commitment,
                                fr const& owner_private_key,
                                boolean const& is_dummy_commitment = false);

    NotePreimage& get_preimage() { return note_preimage; };

  private:
    bool is_partial_preimage() const;
    bool is_partial_storage_slot() const;
    bool is_partial() const;

    // bool check_if_partial() const = 0;
};

} // namespace aztec3::circuits::apps::notes

// Importing in this way (rather than explicit instantiation of a template class at the bottom of a .cpp file) preserves
// the following:
// - We retain implicit instantiation of templates, meaning we can pick and choose (with static_assert) which class
// methods support native,
//   circuit or both types.
// - We don't implement method definitions in this file, to avoid a circular dependency with state_factory.hpp.
#include "note.tpp"