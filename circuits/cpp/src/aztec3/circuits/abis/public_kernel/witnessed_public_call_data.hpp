#pragma once

#include "../previous_kernel_data.hpp"
#include "public_call_data.hpp"
#include "../signed_tx_request.hpp"
#include "../membership_witness.hpp"

#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis::public_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct WitnessedPublicCallData {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    PublicCallData public_call_data;
    std::array<MembershipWitness<NCT, PUBLIC_DATA_TREE_HEIGHT>, STATE_TRANSITIONS_LENGTH>
        state_transitions_sibling_paths;
    std::array<MembershipWitness<NCT, PUBLIC_DATA_TREE_HEIGHT>, STATE_READS_LENGTH> state_reads_sibling_paths;
    fr public_data_tree_root;

    boolean operator==(WitnessedPublicCallData<NCT> const& other) const
    {
        return public_call_data == other.public_call_data &&
               state_transitions_sibling_paths == other.state_transitions_sibling_paths &&
               state_reads_sibling_paths == other.state_reads_sibling_paths &&
               public_data_tree_root == other.public_data_tree_root;
    };

    template <typename Composer>
    WitnessedPublicCallData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        WitnessedPublicCallData<CircuitTypes<Composer>> witnessed_call_data = {
            // TODO to_ct(signature),
            public_call_data.to_circuit_type(composer),
            state_transitions_sibling_paths.to_circuit_type(composer),
            state_reads_sibling_paths.to_circuit_type(composer),
            public_data_tree_root.to_circuit_type(composer),
        };

        return witnessed_call_data;
    };
};

template <typename NCT> void read(uint8_t const*& it, WitnessedPublicCallData<NCT>& witnessed_call_data)
{
    using serialize::read;

    read(it, witnessed_call_data.public_call_data);
    read(it, witnessed_call_data.state_transitions_sibling_paths);
    read(it, witnessed_call_data.state_reads_sibling_paths);
    read(it, witnessed_call_data.public_data_tree_root);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, WitnessedPublicCallData<NCT>& witnessed_call_data)
{
    using serialize::write;

    write(buf, witnessed_call_data.public_call_data);
    write(buf, witnessed_call_data.state_transitions_sibling_paths);
    write(buf, witnessed_call_data.state_reads_sibling_paths);
    write(buf, witnessed_call_data.public_data_tree_root);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, WitnessedPublicCallData<NCT>& witnessed_call_data)
{
    return os << "public_call_data:\n"
              << witnessed_call_data.public_call_data << "\n"
              << "state_transitions_sibling_paths:\n"
              << witnessed_call_data.state_transitions_sibling_paths << "\n"
              << "state_reads_sibling_paths:\n"
              << witnessed_call_data.state_reads_sibling_paths << "\n"
              << "public_data_tree_root:\n"
              << public_kernel_inputs.public_data_tree_root << "\n";
}

} // namespace aztec3::circuits::abis::public_kernel