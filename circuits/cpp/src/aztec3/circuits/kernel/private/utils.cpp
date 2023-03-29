#include "index.hpp"
#include "init.hpp"

#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>

namespace {
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::private_kernel::PreviousKernelData;
using aztec3::circuits::abis::private_kernel::PublicInputs;
using aztec3::circuits::mock::mock_kernel_circuit;

using plonk::TurboComposer;
using namespace plonk::stdlib::types;

} // namespace

namespace aztec3::circuits::kernel::private_kernel::utils {

// TODO rename dummy
PreviousKernelData<NT> default_previous_kernel()
{
    // TODO confirm this is the right way to initialize struct of 0s
    auto mock_kernel_public_inputs = PublicInputs<NT>();
    mock_kernel_public_inputs.is_private = true;

    auto crs_factory = std::make_shared<EnvReferenceStringFactory>();
    Composer mock_kernel_composer = Composer(crs_factory);
    mock_kernel_circuit(mock_kernel_composer, mock_kernel_public_inputs);

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