#pragma once
#include <common/map.hpp>
#include <common/streams.hpp>
#include <crypto/pedersen/generator_data.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::apps::notes {

using crypto::pedersen::generator_index_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct DefaultPrivateNoteNullifierPreimage {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    fr commitment;
    fr owner_private_key;
    boolean is_real;

    bool operator==(DefaultPrivateNoteNullifierPreimage<NCT> const&) const = default;

    template <typename Composer>
    DefaultPrivateNoteNullifierPreimage<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        DefaultPrivateNoteNullifierPreimage<CircuitTypes<Composer>> preimage = {
            to_ct(commitment),
            to_ct(owner_private_key),
            to_ct(is_real),
        };

        return preimage;
    };
};

template <typename NCT> void read(uint8_t const*& it, DefaultPrivateNoteNullifierPreimage<NCT>& preimage)
{
    using serialize::read;

    read(it, preimage.commitment);
    read(it, preimage.owner_private_key);
    read(it, preimage.is_real);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, DefaultPrivateNoteNullifierPreimage<NCT> const& preimage)
{
    using serialize::write;

    write(buf, preimage.commitment);
    write(buf, preimage.owner_private_key);
    write(buf, preimage.is_real);
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, DefaultPrivateNoteNullifierPreimage<NCT> const& preimage)
{
    return os << "commitment: " << preimage.commitment << "\n"
              << "owner_private_key: " << preimage.owner_private_key << "\n"
              << "is_real: " << preimage.is_real << "\n";
}

} // namespace aztec3::circuits::apps::notes