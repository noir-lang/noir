#pragma once

#include "aztec3/constants.hpp"
#include "aztec3/utils/msgpack_derived_equals.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct NewContractData {
    using address = typename NCT::address;
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    address contract_address = 0;
    address portal_contract_address = 0;
    fr function_tree_root = 0;
    // for serialization, update with new fields
    MSGPACK_FIELDS(contract_address, portal_contract_address, function_tree_root);

    boolean operator==(NewContractData<NCT> const& other) const
    {
        // we can't use =default with a custom boolean, but we can use a msgpack-derived utility
        return utils::msgpack_derived_equals<boolean>(*this, other);
    };

    template <typename Builder> NewContractData<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        NewContractData<CircuitTypes<Builder>> new_contract_data = { to_ct(contract_address),
                                                                     to_ct(portal_contract_address),
                                                                     to_ct(function_tree_root) };

        return new_contract_data;
    };

    template <typename Builder> NewContractData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        NewContractData<NativeTypes> new_contract_data = { to_nt(contract_address),
                                                           to_nt(portal_contract_address),
                                                           to_nt(function_tree_root) };

        return new_contract_data;
    };

    boolean is_empty() const
    {
        return ((contract_address.to_field().is_zero()) && (portal_contract_address.to_field().is_zero()) &&
                (function_tree_root.is_zero()));
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
        // as per the circuit implementation, if contract address == zero then return a zero leaf
        if (is_empty()) {
            return fr::zero();
        }
        std::vector<fr> const inputs = {
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

template <typename NCT> std::ostream& operator<<(std::ostream& os, NewContractData<NCT> const& new_contract_data)
{
    return os << "contract_address: " << new_contract_data.contract_address << "\n"
              << "portal_contract_address: " << new_contract_data.portal_contract_address << "\n"
              << "function_tree_root: " << new_contract_data.function_tree_root << "\n";
}

template <typename NCT> using ContractLeafPreimage = NewContractData<NCT>;

}  // namespace aztec3::circuits::abis
