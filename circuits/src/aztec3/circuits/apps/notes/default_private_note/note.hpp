#pragma once

#include "../note_interface.hpp"

#include "note_preimage.hpp"
#include "nullifier_preimage.hpp"

#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

#include <variant>

// Forward-declare from this namespace in particular:
namespace aztec3::circuits::apps::state_vars {
template <typename Composer> class StateVar;
template <typename Composer, typename V> class UTXOStateVar;
template <typename Composer, typename V> class UTXOSetStateVar;
} // namespace aztec3::circuits::apps::state_vars

namespace aztec3::circuits::apps::notes {

using aztec3::circuits::apps::state_vars::StateVar;        // Don't #include it!
using aztec3::circuits::apps::state_vars::UTXOSetStateVar; // Don't #include it!
using aztec3::circuits::apps::state_vars::UTXOStateVar;    // Don't #include it!

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

    // using VariantUTXOStateVar =
    //     std::variant<UTXOStateVar<Composer, DefaultPrivateNote>, UTXOSetStateVar<Composer, DefaultPrivateNote>>;
    using NotePreimage = DefaultPrivateNotePreimage<CircuitTypes<Composer>, ValueType>;
    using NullifierPreimage = DefaultPrivateNoteNullifierPreimage<CircuitTypes<Composer>>;

  public:
    StateVar<Composer>* state_var;

  private:
    std::optional<fr> commitment;
    std::optional<fr> nullifier;
    std::optional<grumpkin_point> partial_commitment;

    // bool is_partial = false;

    NotePreimage note_preimage;
    std::optional<NullifierPreimage> nullifier_preimage;

  public:
    // CUSTOM CONSTRUCTORS:

    DefaultPrivateNote(StateVar<Composer>* state_var, NotePreimage note_preimage)
        : state_var(state_var)
        , note_preimage(note_preimage){};

    ~DefaultPrivateNote() {}

    // OVERRIDE METHODS:

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

    void constrain_against_advice(NoteInterface<Composer> const& advice_note) override;

    bool needs_nonce() override;

    void set_nonce(fr nonce) override;

    fr generate_nonce() override;

    // CUSTOM METHODS

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