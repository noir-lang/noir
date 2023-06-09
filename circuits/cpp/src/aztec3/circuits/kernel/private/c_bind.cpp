#include "c_bind.h"

#include "index.hpp"
#include "utils.hpp"

#include "aztec3/circuits/abis/combined_constant_data.hpp"
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {
using Composer = plonk::UltraPlonkComposer;
using NT = aztec3::utils::types::NativeTypes;
using DummyComposer = aztec3::utils::DummyComposer;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::circuits::abis::SignedTxRequest;
using aztec3::circuits::abis::private_kernel::PrivateCallData;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInit;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner;
using aztec3::circuits::kernel::private_kernel::native_private_kernel_circuit_initial;
using aztec3::circuits::kernel::private_kernel::native_private_kernel_circuit_inner;
using aztec3::circuits::kernel::private_kernel::private_kernel_circuit;
using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel;

}  // namespace

// WASM Cbinds

// TODO(dbanks12): might be able to get rid of proving key buffer
WASM_EXPORT size_t private_kernel__init_proving_key(uint8_t const** pk_buf)
{
    std::vector<uint8_t> pk_vec(42, 0);

    auto* raw_buf = (uint8_t*)malloc(pk_vec.size());
    memcpy(raw_buf, (void*)pk_vec.data(), pk_vec.size());
    *pk_buf = raw_buf;

    return pk_vec.size();
}

WASM_EXPORT size_t private_kernel__init_verification_key(uint8_t const* pk_buf, uint8_t const** vk_buf)
{
    (void)pk_buf;

    // TODO(dbanks12) actual verification key?
    // NT:VKData vk_data = { 0 };

    std::vector<uint8_t> vk_vec(42, 0);
    // write(vk_vec, vk_data);

    auto* raw_buf = (uint8_t*)malloc(vk_vec.size());
    memcpy(raw_buf, (void*)vk_vec.data(), vk_vec.size());
    *vk_buf = raw_buf;

    return vk_vec.size();
}

CBIND(private_kernel__dummy_previous_kernel, []() { return dummy_previous_kernel(); });

// TODO(dbanks12): comment about how public_inputs is a confusing name
// returns size of public inputs
WASM_EXPORT uint8_t* private_kernel__sim_init(uint8_t const* signed_tx_request_buf,
                                              uint8_t const* private_call_buf,
                                              size_t* private_kernel_public_inputs_size_out,
                                              uint8_t const** private_kernel_public_inputs_buf)
{
    DummyComposer composer = DummyComposer("private_kernel__sim_init");

    PrivateCallData<NT> private_call_data;
    read(private_call_buf, private_call_data);

    SignedTxRequest<NT> signed_tx_request;
    read(signed_tx_request_buf, signed_tx_request);

    PrivateKernelInputsInit<NT> const private_inputs = PrivateKernelInputsInit<NT>{
        .signed_tx_request = signed_tx_request,
        .private_call = private_call_data,
    };

    auto public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    // serialize public inputs to bytes vec
    std::vector<uint8_t> public_inputs_vec;
    write(public_inputs_vec, public_inputs);
    // copy public inputs to output buffer
    auto* raw_public_inputs_buf = (uint8_t*)malloc(public_inputs_vec.size());
    memcpy(raw_public_inputs_buf, (void*)public_inputs_vec.data(), public_inputs_vec.size());
    *private_kernel_public_inputs_buf = raw_public_inputs_buf;
    *private_kernel_public_inputs_size_out = public_inputs_vec.size();
    return composer.alloc_and_serialize_first_failure();
}

WASM_EXPORT uint8_t* private_kernel__sim_inner(uint8_t const* previous_kernel_buf,
                                               uint8_t const* private_call_buf,
                                               size_t* private_kernel_public_inputs_size_out,
                                               uint8_t const** private_kernel_public_inputs_buf)
{
    DummyComposer composer = DummyComposer("private_kernel__sim_inner");
    PrivateCallData<NT> private_call_data;
    read(private_call_buf, private_call_data);

    PreviousKernelData<NT> previous_kernel;
    read(previous_kernel_buf, previous_kernel);

    PrivateKernelInputsInner<NT> const private_inputs = PrivateKernelInputsInner<NT>{
        .previous_kernel = previous_kernel,
        .private_call = private_call_data,
    };

    auto public_inputs = native_private_kernel_circuit_inner(composer, private_inputs);

    // serialize public inputs to bytes vec
    std::vector<uint8_t> public_inputs_vec;
    write(public_inputs_vec, public_inputs);
    // copy public inputs to output buffer
    auto* raw_public_inputs_buf = (uint8_t*)malloc(public_inputs_vec.size());
    memcpy(raw_public_inputs_buf, (void*)public_inputs_vec.data(), public_inputs_vec.size());
    *private_kernel_public_inputs_buf = raw_public_inputs_buf;
    *private_kernel_public_inputs_size_out = public_inputs_vec.size();
    return composer.alloc_and_serialize_first_failure();
}

// TODO(jeanmon): We currently only support inner variant because the circuit version
// was not splitted into inner/init counterparts. Once this is done, we have to modify
// the below method to dispatch over the two variants based on first_iteration boolean.
// returns size of proof data
WASM_EXPORT size_t private_kernel__prove(uint8_t const* signed_tx_request_buf,
                                         uint8_t const* previous_kernel_buf,
                                         uint8_t const* private_call_buf,
                                         uint8_t const* pk_buf,
                                         bool first_iteration,
                                         uint8_t const** proof_data_buf)
{
    // TODO(dbanks12) might be able to get rid of proving key buffer
    // TODO(dbanks12) do we want to accept it or just get it from our factory?
    (void)pk_buf;  // unused
    auto crs_factory = barretenberg::srs::get_crs_factory();

    SignedTxRequest<NT> signed_tx_request;
    read(signed_tx_request_buf, signed_tx_request);

    PrivateCallData<NT> private_call_data;
    read(private_call_buf, private_call_data);

    PreviousKernelData<NT> previous_kernel;
    if (first_iteration) {
        previous_kernel = dummy_previous_kernel(true);

        previous_kernel.public_inputs.end.private_call_stack[0] = private_call_data.call_stack_item.hash();
        previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots.private_data_tree_root =
            private_call_data.call_stack_item.public_inputs.historic_private_data_tree_root;
        previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots.contract_tree_root =
            private_call_data.call_stack_item.public_inputs.historic_contract_tree_root;
        previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots
            .l1_to_l2_messages_tree_root =
            private_call_data.call_stack_item.public_inputs.historic_l1_to_l2_messages_tree_root;
        previous_kernel.public_inputs.constants.tx_context = signed_tx_request.tx_request.tx_context;
        previous_kernel.public_inputs.is_private = true;
    } else {
        read(previous_kernel_buf, previous_kernel);
    }
    PrivateKernelInputsInner<NT> const private_inputs = PrivateKernelInputsInner<NT>{
        .previous_kernel = previous_kernel,
        .private_call = private_call_data,
    };

    Composer private_kernel_composer = Composer(crs_factory);
    auto private_kernel_prover = private_kernel_composer.create_prover();

    KernelCircuitPublicInputs<NT> public_inputs;
    public_inputs = private_kernel_circuit(private_kernel_composer, private_inputs, first_iteration);
    NT::Proof private_kernel_proof;
    private_kernel_proof = private_kernel_prover.construct_proof();

    // copy proof data to output buffer
    auto* raw_proof_buf = (uint8_t*)malloc(private_kernel_proof.proof_data.size());
    memcpy(raw_proof_buf, (void*)private_kernel_proof.proof_data.data(), private_kernel_proof.proof_data.size());
    *proof_data_buf = raw_proof_buf;

    // TODO(rahul) - for whenever we end up using this method is TS, we need to figure a way for bberg's composer to
    // serialise errors.
    return private_kernel_proof.proof_data.size();
}

WASM_EXPORT size_t private_kernel__verify_proof(uint8_t const* vk_buf, uint8_t const* proof, uint32_t length)
{
    (void)vk_buf;  // unused
    (void)proof;   // unused
    (void)length;  // unused
    return 1U;
}
