#pragma once

#include "../previous_kernel_data.hpp"
#include "public_call_data.hpp"
#include "witnessed_public_call_data.hpp"

#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis::public_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PublicKernelInputsNoKernelInput {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    SignedTxRequest<NCT> signed_tx_request{};
    WitnessedPublicCallData<NCT> public_call{};

    boolean operator==(PublicKernelInputsNoKernelInput<NCT> const& other) const
    {
        return signed_tx_request == other.signed_tx_request && public_call == other.public_call;
    };

    template <typename Composer>
    PublicKernelInputsNoKernelInput<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        PublicKernelInputsNoKernelInput<CircuitTypes<Composer>> public_kernel_inputs = {
            // TODO to_ct(signature),
            signed_tx_request.to_circuit_type(composer),
            public_call.to_circuit_type(composer),
        };

        return public_kernel_inputs;
    };
};

template <typename NCT> void read(uint8_t const*& it, PublicKernelInputsNoKernelInput<NCT>& public_kernel_inputs)
{
    using serialize::read;

    read(it, public_kernel_inputs.signed_tx_request);
    read(it, public_kernel_inputs.public_call);
};

template <typename NCT>
void write(std::vector<uint8_t>& buf, PublicKernelInputsNoKernelInput<NCT> const& public_kernel_inputs)
{
    using serialize::write;

    write(buf, public_kernel_inputs.signed_tx_request);
    write(buf, public_kernel_inputs.public_call);
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, PublicKernelInputsNoKernelInput<NCT> const& public_kernel_inputs)
{
    return os << "signed_tx_request:\n"
              << public_kernel_inputs.signed_tx_request << "\n"
              << "public_call:\n"
              << public_kernel_inputs.public_call << "\n";
}

} // namespace aztec3::circuits::abis::public_kernel