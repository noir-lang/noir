#pragma once
#include <common/map.hpp>
#include <common/streams.hpp>
#include <crypto/pedersen/generator_data.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::apps {

using crypto::pedersen::generator_index_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct PrivateStateNotePreimage {
    typedef typename NCT::fr fr;
    typedef typename NCT::grumpkin_point grumpkin_point;
    typedef typename NCT::address address;
    typedef typename NCT::boolean boolean;

    // No custom constructors so that designated initializers can be used (for readability of test circuits).

    std::optional<fr> start_slot;             // TODO: remove optionality
    std::optional<grumpkin_point> slot_point; // TODO: remove optionality?
    std::optional<fr> value;
    std::optional<address> owner_address;
    std::optional<address> creator_address;
    std::optional<fr> salt;
    std::optional<fr> input_nullifier;
    std::optional<fr> memo; // numerical representation of a string

    boolean is_real;

    bool operator==(PrivateStateNotePreimage<NCT> const&) const = default;

    template <typename Composer>
    PrivateStateNotePreimage<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        PrivateStateNotePreimage<CircuitTypes<Composer>> preimage = {
            to_ct(start_slot), to_ct(slot_point),      to_ct(value), to_ct(owner_address), to_ct(creator_address),
            to_ct(salt),       to_ct(input_nullifier), to_ct(memo),  to_ct(is_real),
        };

        return preimage;
    };
};

template <typename NCT> void read(uint8_t const*& it, PrivateStateNotePreimage<NCT>& preimage)
{
    using serialize::read;

    read(it, preimage.start_slot);
    read(it, preimage.slot_point);
    read(it, preimage.value);
    read(it, preimage.owner_address);
    read(it, preimage.creator_address);
    read(it, preimage.salt);
    read(it, preimage.input_nullifier);
    read(it, preimage.memo);
    read(it, preimage.is_real);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, PrivateStateNotePreimage<NCT> const& preimage)
{
    using serialize::write;

    write(buf, preimage.start_slot);
    write(buf, preimage.slot_point);
    write(buf, preimage.value);
    write(buf, preimage.owner_address);
    write(buf, preimage.creator_address);
    write(buf, preimage.salt);
    write(buf, preimage.input_nullifier);
    write(buf, preimage.memo);
    write(buf, preimage.is_real);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, PrivateStateNotePreimage<NCT> const& preimage)
{
    return os << "start_slot: " << preimage.start_slot << "\n"
              << "slot_point: " << preimage.slot_point << "\n"
              << "value: " << preimage.value << "\n"
              << "owner_address: " << preimage.owner_address << "\n"
              << "creator_address: " << preimage.creator_address << "\n"
              << "salt: " << preimage.salt << "\n"
              << "input_nullifier: " << preimage.input_nullifier << "\n"
              << "memo: " << preimage.memo << "\n"
              << "is_real: " << preimage.is_real << "\n";
}

// template <typename NCT> using MappingKeyValues = std::map<std::string, std::optional<typename NCT::fr>>;

} // namespace aztec3::circuits::apps