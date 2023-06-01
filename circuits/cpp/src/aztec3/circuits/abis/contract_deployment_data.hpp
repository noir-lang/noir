#pragma once
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct ContractDeploymentData {
    using address = typename NCT::address;
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr constructor_vk_hash = 0;
    fr function_tree_root = 0;
    fr contract_address_salt = 0;
    address portal_contract_address = 0;

    // for serialization: update up with new fields
    MSGPACK_FIELDS(constructor_vk_hash, function_tree_root, contract_address_salt, portal_contract_address);

    boolean operator==(ContractDeploymentData<NCT> const& other) const
    {
        return constructor_vk_hash == other.constructor_vk_hash && function_tree_root == other.function_tree_root &&
               contract_address_salt == other.contract_address_salt &&
               portal_contract_address == other.portal_contract_address;
    };

    template <typename Composer>
    ContractDeploymentData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        ContractDeploymentData<CircuitTypes<Composer>> data = {
            to_ct(constructor_vk_hash),
            to_ct(function_tree_root),
            to_ct(contract_address_salt),
            to_ct(portal_contract_address),
        };

        return data;
    };

    template <typename Composer> ContractDeploymentData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };

        ContractDeploymentData<NativeTypes> call_context = {
            to_nt(constructor_vk_hash),
            to_nt(function_tree_root),
            to_nt(contract_address_salt),
            to_nt(portal_contract_address),
        };

        return call_context;
    };

    template <typename Composer> void assert_is_zero()
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        constructor_vk_hash.assert_is_zero();
        function_tree_root.assert_is_zero();
        contract_address_salt.assert_is_zero();
        portal_contract_address.to_field().assert_is_zero();
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        constructor_vk_hash.set_public();
        function_tree_root.set_public();
        contract_address_salt.set_public();
        portal_contract_address.to_field().set_public();
    }

    fr hash() const
    {
        std::vector<fr> const inputs = {
            constructor_vk_hash,
            function_tree_root,
            contract_address_salt,
            portal_contract_address.to_field(),
        };

        return NCT::compress(inputs, GeneratorIndex::CONTRACT_DEPLOYMENT_DATA);
    }
};

template <typename NCT> void read(uint8_t const*& it, ContractDeploymentData<NCT>& data)
{
    using serialize::read;

    read(it, data.constructor_vk_hash);
    read(it, data.function_tree_root);
    read(it, data.contract_address_salt);
    read(it, data.portal_contract_address);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, ContractDeploymentData<NCT> const& data)
{
    using serialize::write;

    write(buf, data.constructor_vk_hash);
    write(buf, data.function_tree_root);
    write(buf, data.contract_address_salt);
    write(buf, data.portal_contract_address);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, ContractDeploymentData<NCT> const& data)
{
    return os << "constructor_vk_hash: " << data.constructor_vk_hash << "\n"
              << "function_tree_root: " << data.function_tree_root << "\n"
              << "contract_address_salt: " << data.contract_address_salt << "\n"
              << "portal_contract_address: " << data.portal_contract_address << "\n";
}

}  // namespace aztec3::circuits::abis