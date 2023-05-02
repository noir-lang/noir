#pragma once
#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "aztec3/constants.hpp"
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>

#include <type_traits>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PreviousRollupData {
    BaseOrMergeRollupPublicInputs<NCT> base_or_merge_rollup_public_inputs;

    NativeTypes::Proof proof;
    std::shared_ptr<NativeTypes::VK> vk;
    NativeTypes::uint32 vk_index;
    MembershipWitness<NCT, ROLLUP_VK_TREE_HEIGHT> vk_sibling_path;

    bool operator==(PreviousRollupData<NCT> const&) const = default;
};

template <typename NCT> void read(uint8_t const*& it, PreviousRollupData<NCT>& obj)
{
    using serialize::read;

    read(it, obj.base_or_merge_rollup_public_inputs);
    read(it, obj.proof);
    read(it, obj.vk);
    read(it, obj.vk_index);
    read(it, obj.vk_sibling_path);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, PreviousRollupData<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.base_or_merge_rollup_public_inputs);
    write(buf, obj.proof);
    write(buf, *obj.vk);
    write(buf, obj.vk_index);
    write(buf, obj.vk_sibling_path);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, PreviousRollupData<NCT> const& obj)
{
    return os << "base_or_merge_rollup_public_inputs: " << obj.base_or_merge_rollup_public_inputs << "\n"
              << "proof: " << obj.proof << "\n"
              << "vk: " << *(obj.vk) << "\n"
              << "vk_index: " << obj.vk_index << "\n"
              << "vk_sibling_path: " << obj.vk_sibling_path << "\n";
};

}  // namespace aztec3::circuits::abis