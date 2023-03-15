#pragma once
#include "function_data.hpp"
#include "tx_context.hpp"
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct TxRequest {
    typedef typename NCT::address address;
    // typedef typename NCT::grumpkin_point grumpkin_point;
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    address from;
    address to;
    FunctionData<NCT> function_data;
    std::array<fr, ARGS_LENGTH> args;
    fr nonce;
    TxContext<NCT> tx_context;
    fr chain_id;

    template <typename Composer> TxRequest<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        TxRequest<CircuitTypes<Composer>> tx_request = {
            to_ct(from),     to_ct(to),    to_circuit_type(function_data),
            to_ct(args),     to_ct(nonce), to_circuit_type(tx_context),
            to_ct(chain_id),
        };

        return tx_request;
    };

    fr hash() const
    {
        std::vector<fr> inputs;
        inputs.push_back(fr(from));
        inputs.push_back(fr(to));
        inputs.push_back(function_data.hash());
        spread_arr_into_vec(args, inputs);
        inputs.push_back(nonce);
        inputs.push_back(tx_context.hash());
        inputs.push_back(chain_id);

        return NCT::compress(inputs, GeneratorIndex::TX_REQUEST);
    }
    template <size_t SIZE> void spread_arr_into_vec(std::array<fr, SIZE> const& arr, std::vector<fr>& vec) const
    {
        const auto arr_size = sizeof(arr) / sizeof(fr);
        vec.insert(vec.end(), &arr[0], &arr[0] + arr_size);
    }
};

template <typename NCT> void read(uint8_t const*& it, TxRequest<NCT>& tx_request)
{
    using serialize::read;

    read(it, tx_request.from);
    read(it, tx_request.to);
    read(it, tx_request.function_data);
    read(it, tx_request.args);
    read(it, tx_request.nonce);
    read(it, tx_request.tx_context);
    read(it, tx_request.chain_id);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, TxRequest<NCT> const& tx_request)
{
    using serialize::write;

    write(buf, tx_request.from);
    write(buf, tx_request.to);
    write(buf, tx_request.function_data);
    write(buf, tx_request.args);
    write(buf, tx_request.nonce);
    write(buf, tx_request.tx_context);
    write(buf, tx_request.chain_id);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, TxRequest<NCT> const& tx_request)
{
    return os << "from: " << tx_request.from << "\n"
              << "to: " << tx_request.to << "\n"
              << "function_data: " << tx_request.function_data << "\n"
              << "args: " << tx_request.args << "\n"
              << "nonce: " << tx_request.nonce << "\n"
              << "tx_context: " << tx_request.tx_context << "\n"
              << "chain_id: " << tx_request.chain_id << "\n";
}

} // namespace aztec3::circuits::abis