#pragma once

#include "../note_interface.hpp"

#include "note_preimage.hpp"
#include "nullifier_preimage.hpp"

#include "../../state_vars/utxo_state_var.hpp"

#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::apps::notes {

using aztec3::circuits::apps::state_vars::UTXOStateVar;

using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

// template <typename Composer, typename V> class UTXOStateVar;
// template <typename NCT, typename V> struct DefaultPrivateNotePreimage;
// template <typename NCT, typename V> struct DefaultPrivateNoteNullifierPreimage;

template <typename Composer, typename ValueType>
class DefaultPrivateNote : public NoteInterface<Composer,
                                                DefaultPrivateNotePreimage<CircuitTypes<Composer>, ValueType>,
                                                DefaultPrivateNoteNullifierPreimage<CircuitTypes<Composer>>> {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;
    typedef typename CT::address address;
    typedef typename CT::boolean boolean;

    // no known conversion
    //     from 'UTXOStateVar<waffle::TurboComposer,
    //     aztec3::circuits::apps::notes::DefaultPrivateNote<waffle::TurboComposer,
    //     plonk::stdlib::field_t<waffle::TurboComposer> > > *' to 'UTXOStateVar<waffle::TurboComposer,
    //     aztec3::circuits::apps::notes::DefaultPrivateNote<waffle::TurboComposer,
    //     plonk::stdlib::field_t<waffle::TurboComposer> >::Note> *'(
    //         aka 'UTXOStateVar<waffle::TurboComposer, DefaultPrivateNote<waffle::TurboComposer,
    //         plonk::stdlib::field_t<waffle::TurboComposer> > > *')

    // using Note = aztec3::circuits::apps::notes::DefaultPrivateNote<Composer, ValueType>;

    using NotePreimage = DefaultPrivateNotePreimage<CircuitTypes<Composer>, ValueType>;
    using NullifierPreimage = DefaultPrivateNoteNullifierPreimage<CircuitTypes<Composer>>;

    UTXOStateVar<Composer, DefaultPrivateNote>* utxo_state_var;

    DefaultPrivateNote(UTXOStateVar<Composer, DefaultPrivateNote>* utxo_state_var, NotePreimage note_preimage)
        : utxo_state_var(utxo_state_var)
        , note_preimage(note_preimage){};

    // bool operator==(PrivateStateNote<Composer> const&) const = default;

    // METHODS

    void remove() override;

    // HOOKS

    fr get_commitment() override
    {
        if (commitment) {
            return *commitment;
        }
        return compute_commitment().first;
    };

    fr get_nullifier() override
    {
        if (!nullifier) {
            throw_or_abort("No nullifier exists for this note yet. Call compute_nullifier() first.");
        }
        return *nullifier;
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

    std::pair<fr, NotePreimage> compute_commitment() override;

    std::pair<fr, NullifierPreimage> compute_nullifier() override;

    // std::pair<grumpkin_point, DefaultPrivateNotePreimage<CT>> compute_partial_commitment() override;

    static fr compute_nullifier(fr const& commitment,
                                fr const& owner_private_key,
                                boolean const& is_real_commitment = true);

    // static fr compute_dummy_nullifier(fr const& dummy_commitment, fr const& owner_private_key);

    NotePreimage& get_preimage() { return note_preimage; };

  private:
    bool check_if_partial() const;

    // bool check_if_partial() const = 0;

    std::optional<fr> commitment;
    std::optional<fr> nullifier;
    std::optional<grumpkin_point> partial_commitment;

    bool is_partial = false;

    NotePreimage note_preimage;
    std::optional<NullifierPreimage> nullifier_preimage;
};

} // namespace aztec3::circuits::apps::notes

// Importing in this way (rather than explicit instantiation of a template class at the bottom of a .cpp file) preserves
// the following:
// - We retain implicit instantiation of templates, meaning we can pick and choose (with static_assert) which class
// methods support native,
//   circuit or both types.
// - We don't implement method definitions in this file, to avoid a circular dependency with state_factory.hpp.
#include "note.tpp"