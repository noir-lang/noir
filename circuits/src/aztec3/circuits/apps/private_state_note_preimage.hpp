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

    std::optional<fr> start_slot;                     // TODO: remove optionality
    std::optional<grumpkin_point> storage_slot_point; // TODO: remove optionality?

    // TODO: MAKE THIS A PAYLOAD LIKE ZEXE?????
    std::optional<fr> value;
    std::optional<address> owner;
    std::optional<address> creator_address;
    std::optional<fr> memo; // numerical representation of a string

    std::optional<fr> salt;
    std::optional<fr> nonce;

    boolean is_real;

    bool operator==(PrivateStateNotePreimage<NCT> const&) const = default;

    template <typename Composer>
    PrivateStateNotePreimage<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        PrivateStateNotePreimage<CircuitTypes<Composer>> preimage = {
            to_ct(start_slot),
            to_ct(storage_slot_point),
            to_ct(value),
            to_ct(owner),
            to_ct(creator_address),
            to_ct(memo),
            to_ct(salt),
            to_ct(nonce),
            to_ct(is_real),
        };

        return preimage;
    };

    fr hash() const {}

    PrivateStateNotePreimage<NCT> utxo_sload_hook(){};

    PrivateStateNotePreimage<NCT> utxo_sstore_hook(){
        // maybe call the oracle to generate a salt? But that would require us to pass in the oracle, which feels messy
        // / wrong...
        // We'd also need to generate a nonce...
    };
};

template <typename NCT> void read(uint8_t const*& it, PrivateStateNotePreimage<NCT>& preimage)
{
    using serialize::read;

    read(it, preimage.start_slot);
    read(it, preimage.storage_slot_point);
    read(it, preimage.value);
    read(it, preimage.owner);
    read(it, preimage.creator_address);
    read(it, preimage.memo);
    read(it, preimage.salt);
    read(it, preimage.nonce);
    read(it, preimage.is_real);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, PrivateStateNotePreimage<NCT> const& preimage)
{
    using serialize::write;

    write(buf, preimage.start_slot);
    write(buf, preimage.storage_slot_point);
    write(buf, preimage.value);
    write(buf, preimage.owner);
    write(buf, preimage.creator_address);
    write(buf, preimage.memo);
    write(buf, preimage.salt);
    write(buf, preimage.nonce);
    write(buf, preimage.is_real);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, PrivateStateNotePreimage<NCT> const& preimage)
{
    return os << "start_slot: " << preimage.start_slot << "\n"
              << "storage_slot_point: " << preimage.storage_slot_point << "\n"
              << "value: " << preimage.value << "\n"
              << "owner: " << preimage.owner << "\n"
              << "creator_address: " << preimage.creator_address << "\n"
              << "memo: " << preimage.memo << "\n"
              << "salt: " << preimage.salt << "\n"
              << "nonce: " << preimage.nonce << "\n"
              << "is_real: " << preimage.is_real << "\n";
}

// template <typename NCT> using MappingKeyValues = std::map<std::string, std::optional<typename NCT::fr>>;

} // namespace aztec3::circuits::apps