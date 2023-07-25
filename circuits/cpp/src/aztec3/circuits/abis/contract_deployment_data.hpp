#pragma once
#include "aztec3/circuits/abis/point.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

#include <array>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct ContractDeploymentData {
    using address = typename NCT::address;
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    Point<NCT> deployer_public_key;
    fr constructor_vk_hash = 0;
    fr function_tree_root = 0;
    fr contract_address_salt = 0;
    address portal_contract_address = 0;

    // for serialization: update up with new fields
    MSGPACK_FIELDS(
        deployer_public_key, constructor_vk_hash, function_tree_root, contract_address_salt, portal_contract_address);

    boolean operator==(ContractDeploymentData<NCT> const& other) const
    {
        return deployer_public_key == other.deployer_public_key && constructor_vk_hash == other.constructor_vk_hash &&
               function_tree_root == other.function_tree_root && contract_address_salt == other.contract_address_salt &&
               portal_contract_address == other.portal_contract_address;
    };

    template <typename Builder> ContractDeploymentData<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        ContractDeploymentData<CircuitTypes<Builder>> data = {
            deployer_public_key.to_circuit_type(builder),
            to_ct(constructor_vk_hash),
            to_ct(function_tree_root),
            to_ct(contract_address_salt),
            to_ct(portal_contract_address),
        };

        return data;
    };

    template <typename Builder> ContractDeploymentData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Builder>(); };

        ContractDeploymentData<NativeTypes> call_context = {
            to_native_type(deployer_public_key), to_nt(constructor_vk_hash),     to_nt(function_tree_root),
            to_nt(contract_address_salt),        to_nt(portal_contract_address),
        };

        return call_context;
    };

    template <typename Builder> void assert_is_zero()
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        deployer_public_key.assert_is_zero();
        constructor_vk_hash.assert_is_zero();
        function_tree_root.assert_is_zero();
        contract_address_salt.assert_is_zero();
        portal_contract_address.to_field().assert_is_zero();
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        deployer_public_key.set_public();
        constructor_vk_hash.set_public();
        function_tree_root.set_public();
        contract_address_salt.set_public();
        portal_contract_address.to_field().set_public();
    }

    fr hash() const
    {
        std::vector<fr> const inputs = {
            deployer_public_key.x, deployer_public_key.y, constructor_vk_hash,
            function_tree_root,    contract_address_salt, portal_contract_address.to_field(),
        };

        return NCT::hash(inputs, GeneratorIndex::CONTRACT_DEPLOYMENT_DATA);
    }
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, ContractDeploymentData<NCT> const& data)
{
    return os << "deployer_public_key: " << data.deployer_public_key << "\n"
              << "constructor_vk_hash: " << data.constructor_vk_hash << "\n"
              << "function_tree_root: " << data.function_tree_root << "\n"
              << "contract_address_salt: " << data.contract_address_salt << "\n"
              << "portal_contract_address: " << data.portal_contract_address << "\n";
}

}  // namespace aztec3::circuits::abis
