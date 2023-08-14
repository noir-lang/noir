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

template <typename NCT> struct PrivateKernelInputsInner {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    PreviousKernelData<NCT> previous_kernel{};
    PrivateCallData<NCT> private_call{};

    // For serialization, update with new fields
    MSGPACK_FIELDS(previous_kernel, private_call);
    boolean operator==(PrivateKernelInputsInner<NCT> const& other) const
    {
        return previous_kernel == other.previous_kernel && private_call == other.private_call;
    };

    template <typename Builder> PrivateKernelInputsInner<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        PrivateKernelInputsInner<CircuitTypes<Builder>> private_inputs = {
            previous_kernel.to_circuit_type(builder),
            private_call.to_circuit_type(builder),
        };

        return private_inputs;
    };
};

}  // namespace aztec3::circuits::abis::private_kernel
