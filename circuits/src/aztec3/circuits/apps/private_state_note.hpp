#pragma once
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::apps {

using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename Composer> class PrivateStateVar;
template <typename NCT> struct PrivateStateNotePreimage;

template <typename Composer> class PrivateStateNote {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;
    typedef typename CT::address address;
    typedef typename CT::boolean boolean;

    PrivateStateVar<Composer>& private_state_var;
    PrivateStateNotePreimage<CT> preimage;

    bool is_partial = false;

    PrivateStateNote(PrivateStateVar<Composer>& private_state_var,
                     PrivateStateNotePreimage<CT> preimage,
                     bool commit_on_init = false);

    bool operator==(PrivateStateNote<Composer> const&) const = default;

    fr get_commitment() const
    {
        if (!commitment) {
            throw_or_abort("No commitment exists for this note. Are you sure you haven't accidentally created a "
                           "partial commitment? Make sure to populate all preimage values.");
        }
        return *commitment;
    };

    grumpkin_point get_partial_commitment() const
    {
        if (!partial_commitment) {
            throw_or_abort(
                "No partial_commitment exists for this note. Are you sure you haven't accidentally created a "
                "complete commitment?");
        }
        return *partial_commitment;
    };

    fr get_nullifier() const
    {
        if (!nullifier) {
            throw_or_abort("No nullifier exists for this note yet. Call compute_nullifier() first.");
        }
        return *nullifier;
    };

    fr compute_commitment() const;
    grumpkin_point compute_partial_commitment() const;

    fr compute_nullifier(fr const& owner_private_key);
    static fr compute_nullifier(fr const& commitment,
                                fr const& owner_private_key,
                                boolean const& is_real_commitment = true);
    static fr compute_dummy_nullifier(fr const& dummy_commitment, fr const& owner_private_key);

  private:
    bool check_if_partial() const;

    std::optional<grumpkin_point> partial_commitment;
    std::optional<fr> commitment;
    std::optional<fr> nullifier;
};

// // template <typename NCT, typename B> PrivateStateNote<NCT> from_buffer(B const& buffer, size_t offset = 0)
// // {
// //     using serialize::read;
// //     auto ptr = (uint8_t const*)&buffer[offset];

// //     PrivateStateVar<NCT> private_state_var;
// //     PrivateStateNotePreimage<NCT> preimage;

// //     read(ptr, private_state_var);
// //     read(ptr, preimage);

// //     PrivateStateNote<NCT> private_state_note = PrivateStateNote<NCT>(private_state_var, preimage);

// //     return private_state_note;
// // };

// template <typename NCT> void read(uint8_t const*& it, PrivateStateNote<NCT>& note)
// {
//     using serialize::read;

//     read(it, note.private_state_var);
//     read(it, note.preimage);
//     read(it, note.is_partial);
//     read(it, note.partial_commitment);
//     read(it, note.commitment);
//     read(it, note.nullifier);
// };

// template <typename NCT>
// void write(std::vector<uint8_t>& buf, PrivateStateNote<NCT> const& note)
// {
//     using serialize::write;

//     write(buf, note.private_state_var);
//     write(buf, note.preimage);
//     write(buf, note.is_partial);
//     write(buf, note.partial_commitment);
//     write(buf, note.commitment);
//     write(buf, note.nullifier);
// };

// template <typename NCT>
// std::ostream& operator<<(std::ostream& os, PrivateStateNote<NCT> const& note)
// {
//     return os << "private_state_var: " << note.private_state_var << "\n"
//               << "preimage: " << note.preimage << "\n"
//               << "is_partial: " << note.is_partial << "\n"
//               << "partial_commitment: " << note.partial_commitment << "\n"
//               << "commitment: " << note.commitment << "\n"
//               << "nullifier: " << note.nullifier << "\n";
// }

} // namespace aztec3::circuits::apps

// Importing in this way (rather than explicit instantiation of a template class at the bottom of a .cpp file) preserves
// the following:
// - We retain implicit instantiation of templates, meaning we can pick and choose (with static_assert) which class
// methods support native,
//   circuit or both types.
// - We don't implement method definitions in this file, to avoid a circular dependency with state_factory.hpp.
#include "private_state_note.tpp"