#include "aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "aztec3/utils/dummy_composer.hpp"
#include "index.hpp"
#include "init.hpp"
#include "utils.hpp"
#include "c_bind.h"

#include <aztec3/constants.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include "aztec3/circuits/abis/signed_tx_request.hpp"
#include "aztec3/circuits/abis/private_kernel/private_call_data.hpp"
#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>

#include "barretenberg/srs/reference_string/env_reference_string.hpp"

#include "barretenberg/common/serialize.hpp"
#include "barretenberg/plonk/composer/turbo_composer.hpp"

namespace {
using NT = aztec3::utils::types::NativeTypes;
using DummyComposer = aztec3::utils::DummyComposer;
using aztec3::circuits::abis::BaseOrMergeRollupPublicInputs;
using aztec3::circuits::abis::BaseRollupInputs;
using aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit;

using plonk::TurboComposer;
using namespace plonk::stdlib::types;
} // namespace

#define WASM_EXPORT __attribute__((visibility("default")))
// WASM Cbinds
extern "C" {

WASM_EXPORT size_t base_rollup__init_proving_key(uint8_t const** pk_buf)
{
    std::vector<uint8_t> pk_vec(42, 0);

    auto raw_buf = (uint8_t*)malloc(pk_vec.size());
    memcpy(raw_buf, (void*)pk_vec.data(), pk_vec.size());
    *pk_buf = raw_buf;

    return pk_vec.size();
}

WASM_EXPORT size_t base_rollup__init_verification_key(uint8_t const* pk_buf, uint8_t const** vk_buf)
{
    std::vector<uint8_t> vk_vec(42, 0);
    // TODO remove when proving key is used
    (void)pk_buf; // unused

    auto raw_buf = (uint8_t*)malloc(vk_vec.size());
    memcpy(raw_buf, (void*)vk_vec.data(), vk_vec.size());
    *vk_buf = raw_buf;

    return vk_vec.size();
}

WASM_EXPORT size_t base_rollup__sim(uint8_t const* base_rollup_inputs_buf,
                                    uint8_t const** base_or_merge_rollup_public_inputs_buf)
{
    DummyComposer composer = DummyComposer();
    // TODO accept proving key and use that to initialize composers
    // this info is just to prevent error for unused pk_buf
    // TODO do we want to accept it or just get it from our factory?
    // auto crs_factory = std::make_shared<EnvReferenceStringFactory>();

    BaseRollupInputs<NT> base_rollup_inputs;
    read(base_rollup_inputs_buf, base_rollup_inputs);

    BaseOrMergeRollupPublicInputs<NT> public_inputs = base_rollup_circuit(composer, base_rollup_inputs);

    // TODO for circuit proof version of this function
    // NT::Proof base_rollup_proof;
    //    Composer composer = Composer(crs_factory);
    //    plonk::stdlib::types::Prover prover = composer.create_prover();
    //    public_inputs = base_rollup_circuit(composer, base_rollup_inputs);
    //    base_rollup_proof = prover.construct_proof();

    // serialize public inputs to bytes vec
    std::vector<uint8_t> public_inputs_vec;
    write(public_inputs_vec, public_inputs);
    // copy public inputs to output buffer
    auto raw_public_inputs_buf = (uint8_t*)malloc(public_inputs_vec.size());
    memcpy(raw_public_inputs_buf, (void*)public_inputs_vec.data(), public_inputs_vec.size());
    *base_or_merge_rollup_public_inputs_buf = raw_public_inputs_buf;

    return public_inputs_vec.size();
}

// WASM_EXPORT size_t base_rollup__sim(uint8_t const* base_rollup_inputs_buf,
//                                    bool second_present,
//                                    uint8_t const** base_or_merge_rollup_public_inputs_buf)
//{
//    // TODO accept proving key and use that to initialize composers
//    // this info is just to prevent error for unused pk_buf
//    // TODO do we want to accept it or just get it from our factory?
//    auto crs_factory = std::make_shared<EnvReferenceStringFactory>();
//
//    BaseRollupInputs<NT> base_rollup_inputs;
//    read(base_rollup_inputs_buf, base_rollup_inputs);
//
//    NT::Proof base_rollup_proof;
//    BaseOrMergeRollupPublicInputs<NT> public_inputs;
//    if (proverless) {
//        public_inputs = base_rollup_circuit(base_rollup_inputs);
//        // mocked proof - zeros
//        base_rollup_proof = NT::Proof{ std::vector<uint8_t>(42, 0) };
//    }// else {
//    //    Composer composer = Composer(crs_factory);
//    //    plonk::stdlib::types::Prover prover = composer.create_prover();
//    //    public_inputs = base_rollup_circuit(composer, base_rollup_inputs);
//    //    base_rollup_proof = prover.construct_proof();
//    //}
//
//    // copy proof data to output buffer
//    auto raw_proof_buf = (uint8_t*)malloc(base_rollup_proof.proof_data.size());
//    memcpy(raw_proof_buf, (void*)base_rollup_proof.proof_data.data(), base_rollup_proof.proof_data.size());
//    *proof_data_buf = raw_proof_buf;
//
//    // copy proof data size to output
//    *proof_data_size = base_rollup_proof.proof_data.size();
//
//    // serialize public inputs to bytes vec
//    std::vector<uint8_t> public_inputs_vec;
//    write(public_inputs_vec, public_inputs);
//    // copy public inputs to output buffer
//    auto raw_public_inputs_buf = (uint8_t*)malloc(public_inputs_vec.size());
//    memcpy(raw_public_inputs_buf, (void*)public_inputs_vec.data(), public_inputs_vec.size());
//    *base_or_merge_rollup_public_inputs_buf = raw_public_inputs_buf;
//
//    return base_rollup_proof.proof_data.size();
//}

// WASM_EXPORT size_t private_kernel__verify_proof(uint8_t const* vk_buf, uint8_t const* proof, uint32_t length)
// {
//     (void)vk_buf; // unused
//     (void)proof;  // unused
//     (void)length; // unused
//     return true;
// }

} // extern "C"