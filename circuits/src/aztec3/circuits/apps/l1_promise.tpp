#pragma once
#include <crypto/pedersen/generator_data.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include <aztec3/circuits/abis/function_signature.hpp>
#include "contract_factory.hpp"
#include "l1_result.hpp"

namespace aztec3::circuits::apps {

using aztec3::circuits::abis::FunctionSignature;
using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename Composer>
void L1Promise<Composer>::on_success(std::string const& function_name,
                                     std::vector<std::variant<fr, size_t>> const& args)
{
    // construct success_callback_call_hash and success_result_arg_map_acc

    std::vector<std::pair<fr, generator_index_t>> arg_input_pairs;

    for (size_t i = 0; i < args.size(); ++i) {
        if (std::holds_alternative<fr>(args[i])) {
            const fr arg = std::get<fr>(args[i]);
            arg_input_pairs.push_back(std::make_pair(arg, generator_index_t({ GeneratorIndex::CALL_ARGS, i })));
        } else {
            // For L1ResultArgIndex types, we won't know the result's value until after the L1 tx, so we hash using a
            // placeholder generator which represents a particular index of the result array:
            const size_t& result_arg_index = std::get<size_t>(args[i]);
            arg_input_pairs.push_back(
                std::make_pair(fr(1), generator_index_t({ GeneratorIndex::L1_RESULT_PLACEHOLDER, result_arg_index })));
        }
    }

    // TODO: do this calc in the CallStackItem struct instead, once we know whether we can simply hash args instead of
    // the whole set of public inputs.

    auto function_signature = contract.get_function_signature_by_name(function_name);

    fr function_signature_hash = function_signature.hash();
    fr arg_hash = CT::compress(arg_input_pairs);

    std::vector<fr> success_callback_call_hash_inputs = {
        function_signature_hash,
        arg_hash,
    };

    callback_stack_item.success_callback_call_hash =
        CT::compress(success_callback_call_hash_inputs, GeneratorIndex::CALLBACK_STACK_ITEM);
};

template <typename Composer>
void L1Promise<Composer>::on_failure(std::string const& function_name, std::vector<fr> const& args)
{
    // construct failure_callback_call_hash

    std::vector<std::pair<fr, generator_index_t>> arg_input_pairs;

    for (size_t i = 0; i < args.size(); ++i) {
        arg_input_pairs.push_back(std::make_pair(args[i], generator_index_t({ GeneratorIndex::CALL_ARGS, i })));
    }

    // TODO: do this calc in the CallStackItem struct instead, once we know whether we can simply hash args instead of
    // the whole set of public inputs.
    auto function_signature = contract.get_function_signature_by_name(function_name);

    fr function_signature_hash = function_signature.hash();
    fr arg_hash = CT::compress(arg_input_pairs);

    std::vector<fr> failure_callback_call_hash_inputs = {
        function_signature_hash,
        arg_hash,
    };

    callback_stack_item.failure_callback_call_hash =
        CT::compress(failure_callback_call_hash_inputs, GeneratorIndex::CALLBACK_STACK_ITEM);
};

} // namespace aztec3::circuits::apps