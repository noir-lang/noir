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

template <typename NCT> struct PrivateStateOperand {
    typedef typename NCT::fr fr;
    typedef typename NCT::address address;

    fr value;
    address owner;
    std::optional<address> creator_address;
    std::optional<fr> memo; // numerical representation of a string

    bool operator==(PrivateStateOperand<NCT> const&) const = default;

    template <typename Composer> PrivateStateOperand<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        PrivateStateOperand<CircuitTypes<Composer>> preimage = {
            to_ct(value),
            to_ct(owner),
            to_ct(creator_address),
            to_ct(memo),
        };

        return preimage;
    };
};

} // namespace aztec3::circuits::apps