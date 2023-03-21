#pragma once

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
namespace aztec3::circuits::abis::private_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;
using std::is_same;

template <typename NCT> struct NewContractData {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    fr contract_address;        // TODO: Change to aztec address type
    fr portal_contract_address; // TODO: Change to eth address type
    fr function_tree_root;

    template <typename Composer> NewContractData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        NewContractData<CircuitTypes<Composer>> new_contract_data = { to_ct(contract_address),
                                                                      to_ct(portal_contract_address),
                                                                      to_ct(function_tree_root) };

        return new_contract_data;
    };

    template <typename Composer> NewContractData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);

        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Composer>(); };

        NewContractData<NativeTypes> new_contract_data = { to_native_type(contract_address),
                                                           to_native_type(portal_contract_address),
                                                           to_native_type(function_tree_root) };

        return new_contract_data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        contract_address.set_public();
        portal_contract_address.set_public();
        function_tree_root.set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, NewContractData<NCT>& new_contract_data)
{
    using serialize::read;

    read(it, new_contract_data.contract_address);
    read(it, new_contract_data.portal_contract_address);
    read(it, new_contract_data.function_tree_root);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, NewContractData<NCT> const& new_contract_data)
{
    using serialize::write;

    write(buf, new_contract_data.contract_address);
    write(buf, new_contract_data.portal_contract_address);
    write(buf, new_contract_data.function_tree_root);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, NewContractData<NCT> const& new_contract_data)
{
    return os << "contract_address: " << new_contract_data.contract_address << "\n"
              << "portal_contract_address: " << new_contract_data.portal_contract_address << "\n"
              << "function_tree_root: " << new_contract_data.function_tree_root << "\n";
}

} // namespace aztec3::circuits::abis::private_kernel