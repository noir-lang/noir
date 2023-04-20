#pragma once

#include "../previous_kernel_data.hpp"
#include "public_call_data.hpp"
#include "../signed_tx_request.hpp"

#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis::public_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PublicKernelInputs {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    PreviousKernelData<NCT> previous_kernel{};
    PublicCallData<NCT> public_call{};

    boolean operator==(PublicKernelInputs<NCT> const& other) const
    {
        return previous_kernel == other.previous_kernel && public_call == other.public_call;
    };

    template <typename Composer> PublicKernelInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        PublicKernelInputs<CircuitTypes<Composer>> public_kernel_inputs = {
            previous_kernel.to_circuit_type(composer),
            public_call.to_circuit_type(composer),
        };

        return public_kernel_inputs;
    };
};

template <typename NCT> void read(uint8_t const*& it, PublicKernelInputs<NCT>& public_kernel_inputs)
{
    using serialize::read;

    read(it, public_kernel_inputs.previous_kernel);
    read(it, public_kernel_inputs.public_call);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, PublicKernelInputs<NCT> const& public_kernel_inputs)
{
    using serialize::write;

    write(buf, public_kernel_inputs.previous_kernel);
    write(buf, public_kernel_inputs.public_call);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, PublicKernelInputs<NCT> const& public_kernel_inputs)
{
    return os << "previous_kernel:\n"
              << public_kernel_inputs.previous_kernel << "\n"
              << "public_call:\n"
              << public_kernel_inputs.public_call << "\n";
}

} // namespace aztec3::circuits::abis::public_kernel