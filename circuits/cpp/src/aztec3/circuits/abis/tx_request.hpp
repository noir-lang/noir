#pragma once
#include "function_data.hpp"
#include "tx_context.hpp"

#include "aztec3/utils/array.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::zero_array;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct TxRequest {
    using address = typename NCT::address;
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    address from = 0;
    address to = 0;
    FunctionData<NCT> function_data{};
    std::array<fr, ARGS_LENGTH> args = zero_array<fr, ARGS_LENGTH>();
    fr nonce = 0;
    TxContext<NCT> tx_context{};
    fr chain_id = 0;

    boolean operator==(TxContext<NCT> const& other) const
    {
        return from == other.from && to == other.to && function_data == other.function_data && args == other.args &&
               nonce == other.nonce && tx_context == other.tx_context && chain_id == other.chain_id;
    };

    template <typename Composer> TxRequest<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };
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
        vec.insert(vec.end(), arr.data(), arr.data() + arr_size);
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

}  // namespace aztec3::circuits::abis