#include "index.hpp"
#include "init.hpp"

#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>
#include "aztec3/circuits/abis/private_kernel/new_contract_data.hpp"

namespace {
using NT = aztec3::utils::types::NativeTypes;
using AggregationObject = aztec3::utils::types::NativeTypes::AggregationObject;
using aztec3::circuits::abis::private_kernel::PreviousKernelData;
using aztec3::circuits::mock::mock_kernel_circuit;

} // namespace

namespace aztec3::circuits::kernel::private_kernel::utils {

PreviousKernelData<NT> dummy_previous_kernel_with_vk_proof()
{
    PreviousKernelData<NT> init_previous_kernel{};

    auto crs_factory = std::make_shared<EnvReferenceStringFactory>();
    Composer mock_kernel_composer = Composer(crs_factory);
    auto mock_kernel_public_inputs = mock_kernel_circuit(mock_kernel_composer, init_previous_kernel.public_inputs);

    plonk::stdlib::types::Prover mock_kernel_prover = mock_kernel_composer.create_prover();
    NT::Proof mock_kernel_proof = mock_kernel_prover.construct_proof();

    std::shared_ptr<NT::VK> mock_kernel_vk = mock_kernel_composer.compute_verification_key();

    PreviousKernelData<NT> previous_kernel = {
        .public_inputs = mock_kernel_public_inputs,
        .proof = mock_kernel_proof,
        .vk = mock_kernel_vk,
    };
    return previous_kernel;
}

} // namespace aztec3::circuits::kernel::private_kernel::utils