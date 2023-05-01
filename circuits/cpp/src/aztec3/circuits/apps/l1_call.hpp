#pragma once

#include "l1_function_interface.hpp"

#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>

#include <barretenberg/stdlib/primitives/witness/witness.hpp>

namespace aztec3::circuits::apps {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename Composer> class L1Call {
    typedef typename CircuitTypes<Composer>::fr fr;

  public:
    L1FunctionInterface& l1_function;
    std::vector<fr> args;
    fr hash_of_argument_encodings = 0;
    fr partial_l1_call_stack_item = 0;  // keccak(function_selector, hash_of_argument_encodings)

    L1Call(L1FunctionInterface const& l1_function, std::vector<fr> const& args) : l1_function(l1_function), args(args)
    {
        /// TODO: in reality, we'll need to use keccak hash here, as this will need to be replecated on-chain.
        if (args.size() == 0) {
            hash_of_argument_encodings = 0;
        } else {
            hash_of_argument_encodings = args[0];  // lazy stub for a hash!
        }
        partial_l1_call_stack_item = function_selector;  // lazy stub for a hash!
    }

    bool operator==(L1Call<NCT> const&) const = default;

    template <typename Composer> L1Call<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        L1Call<CircuitTypes<Composer>> l1_call = { l1_function.to_circuit_type(composer),
                                                   to_ct(args),
                                                   to_ct(hash_of_argument_encodings),
                                                   to_ct(partial_l1_call_stack_item) };

        return l1_call;
    };
};

template <typename NCT> void read(uint8_t const*& it, L1Call<NCT>& l1_call)
{
    using serialize::read;

    read(it, l1_call.l1_function);
    read(it, l1_call.args);
    read(it, l1_call.hash_of_argument_encodings);
    read(it, l1_call.partial_l1_call_stack_item);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, L1Call<NCT> const& l1_call)
{
    using serialize::write;

    write(buf, l1_call.l1_function);
    write(buf, l1_call.args);
    write(buf, l1_call.hash_of_argument_encodings);
    write(buf, l1_call.partial_l1_call_stack_item);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, L1Call<NCT> const& l1_call)
{
    return os << "l1_function: " << l1_call.l1_function << "\n"
              << "args: " << l1_call.args << "\n"
              << "hash_of_argument_encodings: " << l1_call.hash_of_argument_encodings << "\n"
              << "partial_l1_call_stack_item: " << l1_call.partial_l1_call_stack_item << "\n";
}

}  // namespace aztec3::circuits::apps