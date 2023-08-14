#pragma once

#include "public_call_data.hpp"
#include "../previous_kernel_data.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis::public_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PublicKernelInputs {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    PreviousKernelData<NCT> previous_kernel{};
    PublicCallData<NCT> public_call{};

    // for serialization, update with new fields
    MSGPACK_FIELDS(previous_kernel, public_call);
    boolean operator==(PublicKernelInputs<NCT> const& other) const
    {
        return previous_kernel == other.previous_kernel && public_call == other.public_call;
    };

    template <typename Builder> PublicKernelInputs<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        PublicKernelInputs<CircuitTypes<Builder>> public_kernel_inputs = {
            previous_kernel.to_circuit_type(builder),
            public_call.to_circuit_type(builder),
        };

        return public_kernel_inputs;
    };
};

}  // namespace aztec3::circuits::abis::public_kernel
