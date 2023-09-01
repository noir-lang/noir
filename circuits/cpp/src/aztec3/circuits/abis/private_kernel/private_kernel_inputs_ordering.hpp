#pragma once

#include "private_call_data.hpp"
#include "../previous_kernel_data.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis::private_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PrivateKernelInputsOrdering {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    PreviousKernelData<NCT> previous_kernel{};

    std::array<fr, MAX_READ_REQUESTS_PER_TX> hint_to_commitments{};

    // For serialization, update with new fields
    MSGPACK_FIELDS(previous_kernel, hint_to_commitments);
    boolean operator==(PrivateKernelInputsOrdering<NCT> const& other) const
    {
        return msgpack_derived_equals<boolean>(*this, other);
    };

    template <typename Builder>
    PrivateKernelInputsOrdering<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        PrivateKernelInputsOrdering<CircuitTypes<Builder>> private_inputs = {
            previous_kernel.to_circuit_type(builder),
            hint_to_commitments.to_circuit_type(builder),
        };

        return private_inputs;
    };
};

}  // namespace aztec3::circuits::abis::private_kernel
