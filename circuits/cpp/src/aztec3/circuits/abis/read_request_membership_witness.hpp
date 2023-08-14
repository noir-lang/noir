#pragma once

#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

/**
 * A ReadRequestMembershipWitness is similar to a MembershipWitness but includes
 * some additional fields used to direct the kernel regarding whether a read is transient
 * and if so which commitment it corresponds to.
 */
template <typename NCT, unsigned int N> struct ReadRequestMembershipWitness {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr leaf_index = 0;
    std::array<fr, N> sibling_path{};
    boolean is_transient = false;  // whether or not the read request corresponds to a pending commitment
                                   // In case we change the default to true, we have to adapt is_empty() method
    fr hint_to_commitment = 0;     // hint to point kernel to the commitment this rr corresponds to

    // For serialization, update with new fields
    MSGPACK_FIELDS(leaf_index, sibling_path, is_transient, hint_to_commitment);

    boolean operator==(ReadRequestMembershipWitness<NCT, N> const& other) const
    {
        return leaf_index == other.leaf_index && sibling_path == other.sibling_path &&
               is_transient == other.is_transient && hint_to_commitment == other.hint_to_commitment;
    };

    template <typename Builder>
    ReadRequestMembershipWitness<CircuitTypes<Builder>, N> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        ReadRequestMembershipWitness<CircuitTypes<Builder>, N> witness = {
            to_ct(leaf_index), to_ct(sibling_path), to_ct(is_transient), to_ct(hint_to_commitment)
        };

        return witness;
    }

    template <typename Builder> ReadRequestMembershipWitness<NativeTypes, N> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        ReadRequestMembershipWitness<NativeTypes, N> witness = {
            to_nt(leaf_index), map(sibling_path, to_nt), to_nt(is_transient), to_nt(hint_to_commitment)
        };

        return witness;
    }


    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        leaf_index.set_public();
        for (fr const& e : sibling_path) {
            e.set_public();
        }

        fr(is_transient).set_public();
        hint_to_commitment.set_public();
    }

    // Deliberately consider a transient read request membership witness as non-empty.
    boolean is_empty() const
    {
        return aztec3::utils::is_empty(leaf_index) && is_array_empty(sibling_path) && !is_transient &&
               aztec3::utils::is_empty(hint_to_commitment);
    }
};

}  // namespace aztec3::circuits::abis
