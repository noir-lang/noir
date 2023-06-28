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

template <typename NCT> void read(uint8_t const*& it, PrivateKernelInputsInner<NCT>& private_inputs)
{
    using serialize::read;

    read(it, private_inputs.previous_kernel);
    read(it, private_inputs.private_call);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, PrivateKernelInputsInner<NCT> const& private_inputs)
{
    using serialize::write;

    write(buf, private_inputs.previous_kernel);
    write(buf, private_inputs.private_call);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, PrivateKernelInputsInner<NCT> const& private_inputs)
{
    return os << "previous_kernel:\n"
              << private_inputs.previous_kernel << "\n"
              << "private_call:\n"
              << private_inputs.private_call << "\n";
}

}  // namespace aztec3::circuits::abis::private_kernel