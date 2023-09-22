#pragma once
#include "aztec3/circuits/abis/coordinate.hpp"
#include "aztec3/circuits/abis/point.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::circuits::compute_partial_address;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct CompleteAddress {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    typename NCT::address address;
    Point<NCT> public_key;
    fr partial_address;

    // for serialization, update with new fields
    MSGPACK_FIELDS(address, public_key, partial_address);
    bool operator==(CompleteAddress<NCT> const&) const = default;

    template <typename Builder> CompleteAddress<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        CompleteAddress<CircuitTypes<Builder>> complete_address = { to_ct(address),
                                                                    to_ct(public_key),
                                                                    to_ct(partial_address) };

        return complete_address;
    };

    template <typename Builder> CompleteAddress<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        CompleteAddress<NativeTypes> complete_address = { to_nt(address), to_nt(public_key), to_nt(partial_address) };

        return complete_address;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        address.set_public();
        public_key.set_public();
        partial_address.set_public();
    }

    void assert_is_zero()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        address.assert_is_zero();
        public_key.assert_is_zero();
        partial_address.assert_is_zero();
    }

    static CompleteAddress<NCT> compute(Point<NCT> const& point,
                                        typename NCT::fr const& contract_address_salt,
                                        typename NCT::fr const& function_tree_root,
                                        typename NCT::fr const& constructor_hash)
    {
        using fr = typename NCT::fr;

        const fr partial_address =
            compute_partial_address<NCT>(contract_address_salt, function_tree_root, constructor_hash);

        CompleteAddress<NCT> complete_address;
        complete_address.address = compute_contract_address_from_partial(point, partial_address);
        complete_address.public_key = point;
        complete_address.partial_address = partial_address;

        return complete_address;
    }
};

}  // namespace aztec3::circuits::abis
