#pragma once

#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;
using std::is_same;

template <typename NCT> struct NewContractData {
    typedef typename NCT::address address;
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    address contract_address = 0;
    address portal_contract_address = 0;
    fr function_tree_root = 0;

    boolean operator==(NewContractData<NCT> const& other) const
    {
        return contract_address == other.contract_address && portal_contract_address == other.portal_contract_address &&
               function_tree_root == other.function_tree_root;
    };

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

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };

        NewContractData<NativeTypes> new_contract_data = { to_nt(contract_address),
                                                           to_nt(portal_contract_address),
                                                           to_nt(function_tree_root) };

        return new_contract_data;
    };

    boolean is_empty() const
    {
        return ((contract_address.to_field() == fr(0)) && (portal_contract_address.to_field() == fr(0)) &&
                (function_tree_root == fr(0)));
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        contract_address.to_field().set_public();
        portal_contract_address.to_field().set_public();
        function_tree_root.set_public();
    }

    fr hash() const
    {
        std::vector<fr> inputs = {
            fr(contract_address),
            fr(portal_contract_address),
            fr(function_tree_root),
        };

        return NCT::compress(inputs, GeneratorIndex::CONTRACT_LEAF);
    }

    void conditional_select(const boolean& condition, const NewContractData<NCT>& other)
    {
        contract_address = address::conditional_assign(condition, other.contract_address, contract_address);
        portal_contract_address =
            address::conditional_assign(condition, other.portal_contract_address, portal_contract_address);
        function_tree_root = fr::conditional_assign(condition, other.function_tree_root, function_tree_root);
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

template <typename NCT> using ContractLeafPreimage = NewContractData<NCT>;

} // namespace aztec3::circuits::abis