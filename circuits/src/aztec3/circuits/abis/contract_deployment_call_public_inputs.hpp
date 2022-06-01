#pragma once
// #include <stdlib/hash/pedersen/pedersen.hpp>
#include <common/map.hpp>
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include "../../constants.hpp"
#include "executed_callback.hpp"
#include "callback_stack_item.hpp"

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> class ContractDeploymentCallPublicInputs {
    typedef typename NCT::fr fr;

  public:
    fr private_constructor_public_inputs_hash;
    fr public_constructor_public_inputs_hash;

    fr private_constructor_vk_hash;
    fr public_constructor_vk_hash;

    fr contract_address;
    fr salt;
    fr vk_root;

    fr circuit_data_hash; // TODO: no uint256 circuit type?

    fr portal_contract_address; // TODO: no uint160 circuit type?

    bool operator==(ContractDeploymentCallPublicInputs const&) const = default;

    static ContractDeploymentCallPublicInputs<NCT> empty() { return { 0, 0, 0, 0, 0, 0, 0, 0, 0 }; };

    template <typename Composer>
    ContractDeploymentCallPublicInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        ContractDeploymentCallPublicInputs<CircuitTypes<Composer>> pis = {
            to_ct(private_constructor_public_inputs_hash),
            to_ct(public_constructor_public_inputs_hash),
            to_ct(private_constructor_vk_hash),
            to_ct(public_constructor_vk_hash),
            to_ct(contract_address),
            to_ct(salt),
            to_ct(vk_root),
            to_ct(circuit_data_hash),
            to_ct(portal_contract_address),
        };

        return pis;
    };
};

template <typename NCT>
void read(uint8_t const*& it, ContractDeploymentCallPublicInputs<NCT>& contract_deployment_call_public_inputs)
{
    using serialize::read;

    ContractDeploymentCallPublicInputs<NCT>& pis = contract_deployment_call_public_inputs;
    read(it, pis.private_constructor_public_inputs_hash);
    read(it, pis.public_constructor_public_inputs_hash);
    read(it, pis.private_constructor_vk_hash);
    read(it, pis.public_constructor_vk_hash);
    read(it, pis.contract_address);
    read(it, pis.salt);
    read(it, pis.vk_root);
    read(it, pis.circuit_data_hash);
    read(it, pis.portal_contract_address);
};

template <typename NCT>
void write(std::vector<uint8_t>& buf,
           ContractDeploymentCallPublicInputs<NCT> const& contract_deployment_call_public_inputs)
{
    using serialize::write;

    ContractDeploymentCallPublicInputs<NCT> const& pis = contract_deployment_call_public_inputs;

    write(buf, pis.private_constructor_public_inputs_hash);
    write(buf, pis.public_constructor_public_inputs_hash);
    write(buf, pis.private_constructor_vk_hash);
    write(buf, pis.public_constructor_vk_hash);
    write(buf, pis.contract_address);
    write(buf, pis.salt);
    write(buf, pis.vk_root);
    write(buf, pis.circuit_data_hash);
    write(buf, pis.portal_contract_address);
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os,
                         ContractDeploymentCallPublicInputs<NCT> const& contract_deployment_call_public_inputs)

{
    ContractDeploymentCallPublicInputs<NCT> const& pis = contract_deployment_call_public_inputs;
    return os << "private_constructor_public_inputs_hash: " << pis.private_constructor_public_inputs_hash << "\n"
              << "public_constructor_public_inputs_hash: " << pis.public_constructor_public_inputs_hash << "\n"
              << "private_constructor_vk_hash: " << pis.private_constructor_vk_hash << "\n"
              << "public_constructor_vk_hash: " << pis.public_constructor_vk_hash << "\n"
              << "contract_address: " << pis.contract_address << "\n"
              << "salt: " << pis.salt << "\n"
              << "vk_root: " << pis.vk_root << "\n"
              << "circuit_data_hash: " << pis.circuit_data_hash << "\n"
              << "portal_contract_address: " << pis.portal_contract_address << "\n";
}

} // namespace aztec3::circuits::abis