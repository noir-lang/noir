#include "index.hpp"
#include "init.hpp"
#include "c_bind.h"

#include <aztec3/constants.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include "aztec3/circuits/abis/signed_tx_request.hpp"
#include "aztec3/circuits/abis/private_kernel/private_call_data.hpp"
#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/public_inputs.hpp>
#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>

#include "barretenberg/srs/reference_string/env_reference_string.hpp"

#include "barretenberg/common/serialize.hpp"
#include "barretenberg/plonk/composer/turbo_composer.hpp"

namespace {
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::SignedTxRequest;
using aztec3::circuits::abis::private_kernel::PreviousKernelData;
using aztec3::circuits::abis::private_kernel::PrivateCallData;
using aztec3::circuits::abis::private_kernel::PrivateInputs;
using aztec3::circuits::abis::private_kernel::PublicInputs;
using aztec3::circuits::kernel::private_kernel::private_kernel_circuit;
using aztec3::circuits::kernel::private_kernel::private_kernel_native;
using aztec3::circuits::mock::mock_kernel_circuit;

using plonk::TurboComposer;
using namespace plonk::stdlib::types;
} // namespace

#define WASM_EXPORT __attribute__((visibility("default")))
// WASM Cbinds
extern "C" {

WASM_EXPORT size_t private_kernel__init_proving_key(uint8_t const** pk_buf)
{
    std::vector<uint8_t> pk_vec(42, 0);

    auto raw_buf = (uint8_t*)malloc(pk_vec.size());
    memcpy(raw_buf, (void*)pk_vec.data(), pk_vec.size());
    *pk_buf = raw_buf;

    return pk_vec.size();
}

WASM_EXPORT size_t private_kernel__init_verification_key(uint8_t const* pk_buf, uint8_t const** vk_buf)
{
    // TODO actual verification key
    // NT:VKData vk_data = { 0 };

    std::vector<uint8_t> vk_vec(42, 0);
    // write(vk_vec, vk_data);
    info(pk_buf);

    auto raw_buf = (uint8_t*)malloc(vk_vec.size());
    memcpy(raw_buf, (void*)vk_vec.data(), vk_vec.size());
    *vk_buf = raw_buf;

    return vk_vec.size();
}

// TODO comment about how public_inputs is a confusing name
// returns size of public inputs
WASM_EXPORT size_t private_kernel__create_proof(uint8_t const* signed_tx_request_buf,
                                                uint8_t const* previous_kernel_buf,
                                                uint8_t const* private_call_buf,
                                                uint8_t const* pk_buf,
                                                bool proverless,
                                                uint8_t const** proof_data_buf,
                                                size_t* proof_data_size,
                                                uint8_t const** private_kernel_public_inputs_buf)
{
    info(previous_kernel_buf);
    // TODO accept proving key and use that to initialize composers
    // this info is just to prevent error for unused pk_buf
    // TODO do we want to accept it or just get it from our factory?
    (void)pk_buf; // unused
    auto crs_factory = std::make_shared<EnvReferenceStringFactory>();

    SignedTxRequest<NT> signed_tx_request;
    read(signed_tx_request_buf, signed_tx_request);

    PrivateCallData<NT> private_call_data;
    read(private_call_buf, private_call_data);

    std::array<NT::fr, aztec3::KERNEL_PRIVATE_CALL_STACK_LENGTH> initial_kernel_private_call_stack{};
    initial_kernel_private_call_stack[0] = private_call_data.call_stack_item.hash();

    auto mock_kernel_public_inputs = PublicInputs<NT>();
    mock_kernel_public_inputs.end.private_call_stack = initial_kernel_private_call_stack,
    // TODO who should inject this? C++ or cbind?
        mock_kernel_public_inputs.constants.old_tree_roots.private_data_tree_root =
            private_call_data.call_stack_item.public_inputs.historic_private_data_tree_root;
    mock_kernel_public_inputs.constants.tx_context = signed_tx_request.tx_request.tx_context;
    mock_kernel_public_inputs.is_private = true;

    // FIXME composer doesn't work in wasm
    Composer mock_kernel_composer = Composer(crs_factory);
    mock_kernel_circuit(mock_kernel_composer, mock_kernel_public_inputs);

    plonk::stdlib::types::Prover mock_kernel_prover = mock_kernel_composer.create_prover();
    NT::Proof mock_kernel_proof = mock_kernel_prover.construct_proof();

    std::shared_ptr<NT::VK> mock_kernel_vk = mock_kernel_composer.compute_verification_key();

    PrivateInputs<NT> private_inputs = PrivateInputs<NT>{
        .signed_tx_request = signed_tx_request,
        .previous_kernel =
            PreviousKernelData<NT>{
                .public_inputs = mock_kernel_public_inputs,
                .proof = mock_kernel_proof,
                .vk = mock_kernel_vk,
            },
        .private_call = private_call_data,

    };

    NT::Proof private_kernel_proof;
    PublicInputs<NT> public_inputs;
    if (proverless) {
        public_inputs = private_kernel_native(private_inputs);
        // mocked proof - zeros
        private_kernel_proof = NT::Proof{ std::vector<uint8_t>(42, 0) };
    } else {
        Composer private_kernel_composer = Composer(crs_factory);
        plonk::stdlib::types::Prover private_kernel_prover = private_kernel_composer.create_prover();
        public_inputs = private_kernel_circuit(private_kernel_composer, private_inputs);
        private_kernel_proof = private_kernel_prover.construct_proof();
    }

    // copy proof data to output buffer
    auto raw_proof_buf = (uint8_t*)malloc(private_kernel_proof.proof_data.size());
    memcpy(raw_proof_buf, (void*)private_kernel_proof.proof_data.data(), private_kernel_proof.proof_data.size());
    *proof_data_buf = raw_proof_buf;

    // copy proof data size to output
    *proof_data_size = private_kernel_proof.proof_data.size();

    // serialize public inputs to bytes vec
    std::vector<uint8_t> public_inputs_vec;
    write(public_inputs_vec, public_inputs);
    // copy public inputs to output buffer
    auto raw_public_inputs_buf = (uint8_t*)malloc(public_inputs_vec.size());
    memcpy(raw_public_inputs_buf, (void*)public_inputs_vec.data(), public_inputs_vec.size());
    *private_kernel_public_inputs_buf = raw_public_inputs_buf;

    return private_kernel_proof.proof_data.size();
}

WASM_EXPORT size_t private_kernel__verify_proof(uint8_t const* vk_buf, uint8_t const* proof, uint32_t length)
{
    info(vk_buf);
    info(proof);
    info(length);
    return true;
}

} // extern "C"