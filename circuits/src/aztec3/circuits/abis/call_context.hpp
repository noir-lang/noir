#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include <crypto/pedersen/generator_data.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct CallContext {
    typedef typename NCT::address address;
    typedef typename NCT::grumpkin_point grumpkin_point;
    typedef typename NCT::fr fr;

    address msg_sender;
    address storage_contract_address;

    bool operator==(CallContext<NCT> const&) const = default;

    static CallContext<NCT> empty() { return { 0, 0 }; };

    template <typename Composer> CallContext<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        CallContext<CircuitTypes<Composer>> call_context = {
            to_ct(msg_sender),
            to_ct(storage_contract_address),
        };

        return call_context;
    };

    template <typename Composer> CallContext<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return plonk::stdlib::types::to_nt<Composer>(e); };

        CallContext<NativeTypes> call_context = {
            to_nt(msg_sender),
            to_nt(storage_contract_address),
        };

        return call_context;
    };

    fr hash()
    {
        std::vector<fr> inputs = {
            msg_sender.to_field(),
            storage_contract_address.to_field(),
        };

        return NCT::compress(inputs, GeneratorIndex::CALL_CONTEXT);
    }

    template <typename Composer> void assert_is_zero()
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        msg_sender.to_field().assert_is_zero();
        storage_contract_address.to_field().assert_is_zero();
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        msg_sender.to_field().set_public();
        storage_contract_address.to_field().set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, CallContext<NCT>& call_context)
{
    using serialize::read;

    read(it, call_context.msg_sender);
    read(it, call_context.storage_contract_address);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, CallContext<NCT> const& call_context)
{
    using serialize::write;

    write(buf, call_context.msg_sender);
    write(buf, call_context.storage_contract_address);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, CallContext<NCT> const& call_context)
{
    return os << "msg_sender: " << call_context.msg_sender << "\n"
              << "storage_contract_address: " << call_context.storage_contract_address << "\n";
}

} // namespace aztec3::circuits::abis