#pragma once
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

// @todo Naming should not be previous. Annoying.
template <typename NCT> struct PreviousKernelData {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;
    using VK = typename NCT::VK;
    using uint32 = typename NCT::uint32;

    KernelCircuitPublicInputs<NCT> public_inputs{};  // TODO: not needed as already contained in proof?
    NativeTypes::Proof proof{};  // TODO: how to express proof as native/circuit type when it gets used as a buffer?
    std::shared_ptr<VK> vk;

    // TODO: this index and path are meant to be those of a leaf within the tree of _kernel circuit_ vks; not the tree
    // of functions within the contract tree.
    uint32 vk_index = 0;
    std::array<fr, VK_TREE_HEIGHT> vk_path = zero_array<fr, VK_TREE_HEIGHT>();

    // for serialization, update with new fields
    MSGPACK_FIELDS(public_inputs, proof, vk, vk_index, vk_path);
    boolean operator==(PreviousKernelData<NCT> const& other) const
    {
        // WARNING: proof not checked!
        return public_inputs == other.public_inputs &&
               // proof == other.proof &&
               vk == other.vk && vk_index == other.vk_index && vk_path == other.vk_path;
    };

    // WARNING: the `proof` does NOT get converted!
    template <typename Builder> PreviousKernelData<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        typedef CircuitTypes<Builder> CT;
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        PreviousKernelData<CircuitTypes<Builder>> data = {
            public_inputs.to_circuit_type(builder),
            proof,  // Notice: not converted! Stays as native.
            CT::VK::from_witness(&builder, vk),
            to_ct(vk_index),
            to_ct(vk_path),
        };

        return data;
    };

};  // namespace aztec3::circuits::abis::private_kernel

template <typename B> inline void read(B& buf, verification_key& key)
{
    using serialize::read;
    // Note this matches write() below
    verification_key_data data;
    read(buf, data);
    key = verification_key{ std::move(data), barretenberg::srs::get_crs_factory()->get_verifier_crs() };
}

template <typename NCT> void read(uint8_t const*& it, PreviousKernelData<NCT>& kernel_data)
{
    using aztec3::circuits::abis::read;
    using serialize::read;

    read(it, kernel_data.public_inputs);
    read(it, kernel_data.proof);
    read(it, kernel_data.vk);
    read(it, kernel_data.vk_index);
    read(it, kernel_data.vk_path);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, PreviousKernelData<NCT> const& kernel_data)
{
    using aztec3::circuits::abis::write;
    using serialize::write;

    write(buf, kernel_data.public_inputs);
    write(buf, kernel_data.proof);
    write(buf, *kernel_data.vk);
    write(buf, kernel_data.vk_index);
    write(buf, kernel_data.vk_path);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, PreviousKernelData<NCT> const& kernel_data)
{
    return os << "public_inputs: " << kernel_data.public_inputs << "\n"
              << "proof: " << kernel_data.proof << "\n"
              << "vk:\n"
              << *(kernel_data.vk) << "\n"
              << "vk_index: " << kernel_data.vk_index << "\n"
              << "vk_path: " << kernel_data.vk_path << "\n";
}

}  // namespace aztec3::circuits::abis