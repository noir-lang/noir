#include "acir_to_constraint_buf.hpp"
#include "barretenberg/common/container.hpp"
#include <cstddef>
#include <tuple>
#include <utility>
#ifndef __wasm__
#include "barretenberg/bb/get_bytecode.hpp"
#endif
#include "barretenberg/common/map.hpp"
namespace acir_format {

using namespace bb;

/**
 * @brief Construct a poly_tuple for a standard width-3 arithmetic gate from its acir representation
 *
 * @param arg acir representation of an 3-wire arithmetic operation
 * @return poly_triple
 * @note In principle Program::Expression can accommodate arbitrarily many quadratic and linear terms but in practice
 * the ones processed here have a max of 1 and 3 respectively, in accordance with the standard width-3 arithmetic gate.
 */
poly_triple serialize_arithmetic_gate(Program::Expression const& arg)
{
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/816): The initialization of the witness indices a,b,c
    // to 0 is implicitly assuming that (builder.zero_idx == 0) which is no longer the case. Now, witness idx 0 in
    // general will correspond to some non-zero value and some witnesses which are not explicitly set below will be
    // erroneously populated with this value. This does not cause failures however because the corresponding selector
    // will indeed be 0 so the gate will be satisfied. Still, its a bad idea to have erroneous wire values
    // even if they dont break the relation. They'll still add cost in commitments, for example.
    poly_triple pt{
        .a = 0,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = 0,
        .q_r = 0,
        .q_o = 0,
        .q_c = 0,
    };

    // Flags indicating whether each witness index for the present poly_tuple has been set
    bool a_set = false;
    bool b_set = false;
    bool c_set = false;

    // If necessary, set values for quadratic term (q_m * w_l * w_r)
    ASSERT(arg.mul_terms.size() <= 1); // We can only accommodate 1 quadratic term
    // Note: mul_terms are tuples of the form {selector_value, witness_idx_1, witness_idx_2}
    if (!arg.mul_terms.empty()) {
        const auto& mul_term = arg.mul_terms[0];
        pt.q_m = uint256_t(std::get<0>(mul_term));
        pt.a = std::get<1>(mul_term).value;
        pt.b = std::get<2>(mul_term).value;
        a_set = true;
        b_set = true;
    }

    // If necessary, set values for linears terms q_l * w_l, q_r * w_r and q_o * w_o
    ASSERT(arg.linear_combinations.size() <= 3); // We can only accommodate 3 linear terms
    for (const auto& linear_term : arg.linear_combinations) {
        fr selector_value(uint256_t(std::get<0>(linear_term)));
        uint32_t witness_idx = std::get<1>(linear_term).value;

        // If the witness index has not yet been set or if the corresponding linear term is active, set the witness
        // index and the corresponding selector value.
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/816): May need to adjust the pt.a == witness_idx
        // check (and the others like it) since we initialize a,b,c with 0 but 0 is a valid witness index once the
        // +1 offset is removed from noir.
        if (!a_set || pt.a == witness_idx) { // q_l * w_l
            pt.a = witness_idx;
            pt.q_l = selector_value;
            a_set = true;
        } else if (!b_set || pt.b == witness_idx) { // q_r * w_r
            pt.b = witness_idx;
            pt.q_r = selector_value;
            b_set = true;
        } else if (!c_set || pt.c == witness_idx) { // q_o * w_o
            pt.c = witness_idx;
            pt.q_o = selector_value;
            c_set = true;
        } else {
            return poly_triple{
                .a = 0,
                .b = 0,
                .c = 0,
                .q_m = 0,
                .q_l = 0,
                .q_r = 0,
                .q_o = 0,
                .q_c = 0,
            };
        }
    }

    // Set constant value q_c
    pt.q_c = uint256_t(arg.q_c);
    return pt;
}
mul_quad_<fr> serialize_mul_quad_gate(Program::Expression const& arg)
{
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/816): The initialization of the witness indices a,b,c
    // to 0 is implicitly assuming that (builder.zero_idx == 0) which is no longer the case. Now, witness idx 0 in
    // general will correspond to some non-zero value and some witnesses which are not explicitly set below will be
    // erroneously populated with this value. This does not cause failures however because the corresponding selector
    // will indeed be 0 so the gate will be satisfied. Still, its a bad idea to have erroneous wire values
    // even if they dont break the relation. They'll still add cost in commitments, for example.
    mul_quad_<fr> quad{ .a = 0,
                        .b = 0,
                        .c = 0,
                        .d = 0,
                        .mul_scaling = 0,
                        .a_scaling = 0,
                        .b_scaling = 0,
                        .c_scaling = 0,
                        .d_scaling = 0,
                        .const_scaling = 0 };

    // Flags indicating whether each witness index for the present mul_quad has been set
    bool a_set = false;
    bool b_set = false;
    bool c_set = false;
    bool d_set = false;
    ASSERT(arg.mul_terms.size() <= 1); // We can only accommodate 1 quadratic term
    // Note: mul_terms are tuples of the form {selector_value, witness_idx_1, witness_idx_2}
    if (!arg.mul_terms.empty()) {
        const auto& mul_term = arg.mul_terms[0];
        quad.mul_scaling = uint256_t(std::get<0>(mul_term));
        quad.a = std::get<1>(mul_term).value;
        quad.b = std::get<2>(mul_term).value;
        a_set = true;
        b_set = true;
    }
    // If necessary, set values for linears terms q_l * w_l, q_r * w_r and q_o * w_o
    ASSERT(arg.linear_combinations.size() <= 4); // We can only accommodate 4 linear terms
    for (const auto& linear_term : arg.linear_combinations) {
        fr selector_value(uint256_t(std::get<0>(linear_term)));
        uint32_t witness_idx = std::get<1>(linear_term).value;

        // If the witness index has not yet been set or if the corresponding linear term is active, set the witness
        // index and the corresponding selector value.
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/816): May need to adjust the quad.a == witness_idx
        // check (and the others like it) since we initialize a,b,c with 0 but 0 is a valid witness index once the
        // +1 offset is removed from noir.
        if (!a_set || quad.a == witness_idx) {
            quad.a = witness_idx;
            quad.a_scaling = selector_value;
            a_set = true;
        } else if (!b_set || quad.b == witness_idx) {
            quad.b = witness_idx;
            quad.b_scaling = selector_value;
            b_set = true;
        } else if (!c_set || quad.c == witness_idx) {
            quad.c = witness_idx;
            quad.c_scaling = selector_value;
            c_set = true;
        } else if (!d_set || quad.d == witness_idx) {
            quad.d = witness_idx;
            quad.d_scaling = selector_value;
            d_set = true;
        } else {
            throw_or_abort("Cannot assign linear term to a constraint of width 4");
        }
    }

    // Set constant value q_c
    quad.const_scaling = uint256_t(arg.q_c);
    return quad;
}

void handle_arithmetic(Program::Opcode::AssertZero const& arg, AcirFormat& af, size_t opcode_index)
{
    if (arg.value.linear_combinations.size() <= 3) {
        poly_triple pt = serialize_arithmetic_gate(arg.value);
        // Even if the number of linear terms is less than 3, we might not be able to fit it into a width-3 arithmetic
        // gate. This is the case if the linear terms are all disctinct witness from the multiplication term. In that
        // case, the serialize_arithmetic_gate() function will return a poly_triple with all 0's, and we use a width-4
        // gate instead. We could probably always use a width-4 gate in fact.
        if (pt == poly_triple{ 0, 0, 0, 0, 0, 0, 0, 0 }) {
            af.quad_constraints.push_back(serialize_mul_quad_gate(arg.value));
            af.original_opcode_indices.quad_constraints.push_back(opcode_index);

        } else {
            af.poly_triple_constraints.push_back(pt);
            af.original_opcode_indices.poly_triple_constraints.push_back(opcode_index);
        }
    } else {
        af.quad_constraints.push_back(serialize_mul_quad_gate(arg.value));
        af.original_opcode_indices.quad_constraints.push_back(opcode_index);
    }
}

uint32_t get_witness_from_function_input(Program::FunctionInput input)
{
    auto input_witness = std::get<Program::ConstantOrWitnessEnum::Witness>(input.input.value);
    return input_witness.value.value;
}

WitnessConstant<bb::fr> parse_input(Program::FunctionInput input)
{
    WitnessConstant result = std::visit(
        [&](auto&& e) {
            using T = std::decay_t<decltype(e)>;
            if constexpr (std::is_same_v<T, Program::ConstantOrWitnessEnum::Witness>) {
                return WitnessConstant<bb::fr>{
                    .index = e.value.value,
                    .value = bb::fr::zero(),
                    .is_constant = false,
                };
            } else if constexpr (std::is_same_v<T, Program::ConstantOrWitnessEnum::Constant>) {
                return WitnessConstant<bb::fr>{
                    .index = 0,
                    .value = uint256_t(e.value),
                    .is_constant = true,
                };
            } else {
                ASSERT(false);
            }
            return WitnessConstant<bb::fr>{
                .index = 0,
                .value = bb::fr::zero(),
                .is_constant = true,
            };
        },
        input.input.value);
    return result;

    // WitnessConstant result = std::visit(
    //     [&](auto&& e) {
    //         using T = std::decay_t<decltype(e)>;
    //         if constexpr (std::is_same_v<T, Program::FunctionInput::Witness>) {
    //             return WitnessConstant<bb::fr>{
    //                 .index = e.value.witness.value,
    //                 .value = bb::fr::zero(),
    //                 .is_constant = false,
    //             };
    //         } else if constexpr (std::is_same_v<T, Program::FunctionInput::Constant>) {
    //             return WitnessConstant<bb::fr>{
    //                 .index = 0,
    //                 .value = uint256_t(e.value.constant),
    //                 .is_constant = true,
    //             };
    //         } else {
    //             ASSERT(false);
    //         }
    //         return WitnessConstant<bb::fr>{
    //             .index = 0,
    //             .value = bb::fr::zero(),
    //             .is_constant = true,
    //         };
    //     },
    //     input.value);
    // return result;
}

void handle_blackbox_func_call(Program::Opcode::BlackBoxFuncCall const& arg,
                               AcirFormat& af,
                               bool honk_recursion,
                               size_t opcode_index)
{
    std::visit(
        [&](auto&& arg) {
            using T = std::decay_t<decltype(arg)>;
            if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::AND>) {
                auto lhs_input = get_witness_from_function_input(arg.lhs);
                auto rhs_input = get_witness_from_function_input(arg.rhs);
                af.logic_constraints.push_back(LogicConstraint{
                    .a = lhs_input,
                    .b = rhs_input,
                    .result = arg.output.value,
                    .num_bits = arg.lhs.num_bits,
                    .is_xor_gate = false,
                });
                af.original_opcode_indices.logic_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::XOR>) {
                auto lhs_input = get_witness_from_function_input(arg.lhs);
                auto rhs_input = get_witness_from_function_input(arg.rhs);
                af.logic_constraints.push_back(LogicConstraint{
                    .a = lhs_input,
                    .b = rhs_input,
                    .result = arg.output.value,
                    .num_bits = arg.lhs.num_bits,
                    .is_xor_gate = true,
                });
                af.original_opcode_indices.logic_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::RANGE>) {
                auto witness_input = get_witness_from_function_input(arg.input);
                af.range_constraints.push_back(RangeConstraint{
                    .witness = witness_input,
                    .num_bits = arg.input.num_bits,
                });
                af.original_opcode_indices.range_constraints.push_back(opcode_index);

            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::AES128Encrypt>) {
                af.aes128_constraints.push_back(AES128Constraint{
                    .inputs = map(arg.inputs,
                                  [](auto& e) {
                                      return AES128Input{
                                          .witness = get_witness_from_function_input(e),
                                          .num_bits = e.num_bits,
                                      };
                                  }),
                    .iv = map(arg.iv,
                              [](auto& e) {
                                  auto witness = get_witness_from_function_input(e);
                                  return AES128Input{
                                      .witness = witness,
                                      .num_bits = e.num_bits,
                                  };
                              }),
                    .key = map(arg.key,
                               [](auto& e) {
                                   auto input_witness = get_witness_from_function_input(e);
                                   return AES128Input{
                                       .witness = input_witness,
                                       .num_bits = e.num_bits,
                                   };
                               }),
                    .outputs = map(arg.outputs, [](auto& e) { return e.value; }),
                });
                af.original_opcode_indices.aes128_constraints.push_back(opcode_index);

            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::SHA256>) {
                af.sha256_constraints.push_back(Sha256Constraint{
                    .inputs = map(arg.inputs,
                                  [](auto& e) {
                                      auto input_witness = get_witness_from_function_input(e);
                                      return Sha256Input{
                                          .witness = input_witness,
                                          .num_bits = e.num_bits,
                                      };
                                  }),
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                });
                af.original_opcode_indices.sha256_constraints.push_back(opcode_index);

            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::Sha256Compression>) {
                af.sha256_compression.push_back(Sha256Compression{
                    .inputs = map(arg.inputs,
                                  [](auto& e) {
                                      auto input_witness = get_witness_from_function_input(e);
                                      return Sha256Input{
                                          .witness = input_witness,
                                          .num_bits = e.num_bits,
                                      };
                                  }),
                    .hash_values = map(arg.hash_values,
                                       [](auto& e) {
                                           auto input_witness = get_witness_from_function_input(e);
                                           return Sha256Input{
                                               .witness = input_witness,
                                               .num_bits = e.num_bits,
                                           };
                                       }),
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                });
                af.original_opcode_indices.sha256_compression.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::Blake2s>) {
                af.blake2s_constraints.push_back(Blake2sConstraint{
                    .inputs = map(arg.inputs,
                                  [](auto& e) {
                                      auto input_witness = get_witness_from_function_input(e);
                                      return Blake2sInput{
                                          .witness = input_witness,
                                          .num_bits = e.num_bits,
                                      };
                                  }),
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                });
                af.original_opcode_indices.blake2s_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::Blake3>) {
                af.blake3_constraints.push_back(Blake3Constraint{
                    .inputs = map(arg.inputs,
                                  [](auto& e) {
                                      auto input_witness = get_witness_from_function_input(e);
                                      return Blake3Input{
                                          .witness = input_witness,
                                          .num_bits = e.num_bits,
                                      };
                                  }),
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                });
                af.original_opcode_indices.blake3_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::SchnorrVerify>) {
                auto input_pkey_x = get_witness_from_function_input(arg.public_key_x);
                auto input_pkey_y = get_witness_from_function_input(arg.public_key_y);
                af.schnorr_constraints.push_back(SchnorrConstraint{
                    .message = map(arg.message, [](auto& e) { return get_witness_from_function_input(e); }),
                    .public_key_x = input_pkey_x,
                    .public_key_y = input_pkey_y,
                    .result = arg.output.value,
                    .signature = map(arg.signature, [](auto& e) { return get_witness_from_function_input(e); }),
                });
                af.original_opcode_indices.schnorr_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::PedersenCommitment>) {

                af.pedersen_constraints.push_back(PedersenConstraint{
                    .scalars = map(arg.inputs, [](auto& e) { return get_witness_from_function_input(e); }),
                    .hash_index = arg.domain_separator,
                    .result_x = arg.outputs[0].value,
                    .result_y = arg.outputs[1].value,
                });
                af.original_opcode_indices.pedersen_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::PedersenHash>) {
                af.pedersen_hash_constraints.push_back(PedersenHashConstraint{
                    .scalars = map(arg.inputs, [](auto& e) { return get_witness_from_function_input(e); }),
                    .hash_index = arg.domain_separator,
                    .result = arg.output.value,
                });
                af.original_opcode_indices.pedersen_hash_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::EcdsaSecp256k1>) {
                af.ecdsa_k1_constraints.push_back(EcdsaSecp256k1Constraint{
                    .hashed_message =
                        map(arg.hashed_message, [](auto& e) { return get_witness_from_function_input(e); }),
                    .signature = map(arg.signature, [](auto& e) { return get_witness_from_function_input(e); }),
                    .pub_x_indices = map(arg.public_key_x, [](auto& e) { return get_witness_from_function_input(e); }),
                    .pub_y_indices = map(arg.public_key_y, [](auto& e) { return get_witness_from_function_input(e); }),
                    .result = arg.output.value,
                });
                af.original_opcode_indices.ecdsa_k1_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::EcdsaSecp256r1>) {
                af.ecdsa_r1_constraints.push_back(EcdsaSecp256r1Constraint{
                    .hashed_message =
                        map(arg.hashed_message, [](auto& e) { return get_witness_from_function_input(e); }),
                    .pub_x_indices = map(arg.public_key_x, [](auto& e) { return get_witness_from_function_input(e); }),
                    .pub_y_indices = map(arg.public_key_y, [](auto& e) { return get_witness_from_function_input(e); }),
                    .result = arg.output.value,
                    .signature = map(arg.signature, [](auto& e) { return get_witness_from_function_input(e); }),
                });
                af.original_opcode_indices.ecdsa_r1_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::MultiScalarMul>) {
                af.multi_scalar_mul_constraints.push_back(MultiScalarMul{
                    .points = map(arg.points, [](auto& e) { return parse_input(e); }),
                    .scalars = map(arg.scalars, [](auto& e) { return parse_input(e); }),
                    .out_point_x = arg.outputs[0].value,
                    .out_point_y = arg.outputs[1].value,
                    .out_point_is_infinite = arg.outputs[2].value,
                });
                af.original_opcode_indices.multi_scalar_mul_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::EmbeddedCurveAdd>) {
                auto input_1_x = get_witness_from_function_input(arg.input1[0]);
                auto input_1_y = get_witness_from_function_input(arg.input1[1]);
                auto input_1_infinite = get_witness_from_function_input(arg.input1[2]);
                auto input_2_x = get_witness_from_function_input(arg.input2[0]);
                auto input_2_y = get_witness_from_function_input(arg.input2[1]);
                auto input_2_infinite = get_witness_from_function_input(arg.input2[2]);

                af.ec_add_constraints.push_back(EcAdd{
                    .input1_x = input_1_x,
                    .input1_y = input_1_y,
                    .input1_infinite = input_1_infinite,
                    .input2_x = input_2_x,
                    .input2_y = input_2_y,
                    .input2_infinite = input_2_infinite,
                    .result_x = arg.outputs[0].value,
                    .result_y = arg.outputs[1].value,
                    .result_infinite = arg.outputs[2].value,
                });
                af.original_opcode_indices.ec_add_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::Keccak256>) {
                auto input_var_message_size = get_witness_from_function_input(arg.var_message_size);
                af.keccak_constraints.push_back(KeccakConstraint{
                    .inputs = map(arg.inputs,
                                  [](auto& e) {
                                      auto input_witness = get_witness_from_function_input(e);
                                      return HashInput{
                                          .witness = input_witness,
                                          .num_bits = e.num_bits,
                                      };
                                  }),
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                    .var_message_size = input_var_message_size,
                });
                af.original_opcode_indices.keccak_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::Keccakf1600>) {
                af.keccak_permutations.push_back(Keccakf1600{
                    .state = map(arg.inputs,
                                 [](auto& e) {
                                     auto input_witness = get_witness_from_function_input(e);
                                     return input_witness;
                                 }),
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                });
                af.original_opcode_indices.keccak_permutations.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::RecursiveAggregation>) {
                if (honk_recursion) { // if we're using the honk recursive verifier
                    auto c = HonkRecursionConstraint{
                        .key = map(arg.verification_key, [](auto& e) { return get_witness_from_function_input(e); }),
                        .proof = map(arg.proof, [](auto& e) { return get_witness_from_function_input(e); }),
                        .public_inputs =
                            map(arg.public_inputs, [](auto& e) { return get_witness_from_function_input(e); }),
                    };
                    af.honk_recursion_constraints.push_back(c);
                    af.original_opcode_indices.honk_recursion_constraints.push_back(opcode_index);
                } else {
                    auto input_key = get_witness_from_function_input(arg.key_hash);

                    auto c = RecursionConstraint{
                        .key = map(arg.verification_key, [](auto& e) { return get_witness_from_function_input(e); }),
                        .proof = map(arg.proof, [](auto& e) { return get_witness_from_function_input(e); }),
                        .public_inputs =
                            map(arg.public_inputs, [](auto& e) { return get_witness_from_function_input(e); }),
                        .key_hash = input_key,
                    };
                    af.recursion_constraints.push_back(c);
                    af.original_opcode_indices.recursion_constraints.push_back(opcode_index);
                }
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::BigIntFromLeBytes>) {
                af.bigint_from_le_bytes_constraints.push_back(BigIntFromLeBytes{
                    .inputs = map(arg.inputs, [](auto& e) { return get_witness_from_function_input(e); }),
                    .modulus = map(arg.modulus, [](auto& e) -> uint32_t { return e; }),
                    .result = arg.output,
                });
                af.original_opcode_indices.bigint_from_le_bytes_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::BigIntToLeBytes>) {
                af.bigint_to_le_bytes_constraints.push_back(BigIntToLeBytes{
                    .input = arg.input,
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                });
                af.original_opcode_indices.bigint_to_le_bytes_constraints.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::BigIntAdd>) {
                af.bigint_operations.push_back(BigIntOperation{
                    .lhs = arg.lhs,
                    .rhs = arg.rhs,
                    .result = arg.output,
                    .opcode = BigIntOperationType::Add,
                });
                af.original_opcode_indices.bigint_operations.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::BigIntSub>) {
                af.bigint_operations.push_back(BigIntOperation{
                    .lhs = arg.lhs,
                    .rhs = arg.rhs,
                    .result = arg.output,
                    .opcode = BigIntOperationType::Sub,
                });
                af.original_opcode_indices.bigint_operations.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::BigIntMul>) {
                af.bigint_operations.push_back(BigIntOperation{
                    .lhs = arg.lhs,
                    .rhs = arg.rhs,
                    .result = arg.output,
                    .opcode = BigIntOperationType::Mul,
                });
                af.original_opcode_indices.bigint_operations.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::BigIntDiv>) {
                af.bigint_operations.push_back(BigIntOperation{
                    .lhs = arg.lhs,
                    .rhs = arg.rhs,
                    .result = arg.output,
                    .opcode = BigIntOperationType::Div,
                });
                af.original_opcode_indices.bigint_operations.push_back(opcode_index);
            } else if constexpr (std::is_same_v<T, Program::BlackBoxFuncCall::Poseidon2Permutation>) {
                af.poseidon2_constraints.push_back(Poseidon2Constraint{
                    .state = map(arg.inputs,
                                 [](auto& e) {
                                     auto input_witness = get_witness_from_function_input(e);
                                     return input_witness;
                                 }),
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                    .len = arg.len,
                });
                af.original_opcode_indices.poseidon2_constraints.push_back(opcode_index);
            }
        },
        arg.value.value);
}

BlockConstraint handle_memory_init(Program::Opcode::MemoryInit const& mem_init)
{
    BlockConstraint block{ .init = {}, .trace = {}, .type = BlockType::ROM };
    std::vector<poly_triple> init;
    std::vector<MemOp> trace;

    auto len = mem_init.init.size();
    for (size_t i = 0; i < len; ++i) {
        block.init.push_back(poly_triple{
            .a = mem_init.init[i].value,
            .b = 0,
            .c = 0,
            .q_m = 0,
            .q_l = 1,
            .q_r = 0,
            .q_o = 0,
            .q_c = 0,
        });
    }

    // Databus is only supported for Goblin, non Goblin builders will treat call_data and return_data as normal
    // array.
    if (std::holds_alternative<Program::BlockType::CallData>(mem_init.block_type.value)) {
        block.type = BlockType::CallData;
    } else if (std::holds_alternative<Program::BlockType::ReturnData>(mem_init.block_type.value)) {
        block.type = BlockType::ReturnData;
    }

    return block;
}

bool is_rom(Program::MemOp const& mem_op)
{
    return mem_op.operation.mul_terms.size() == 0 && mem_op.operation.linear_combinations.size() == 0 &&
           uint256_t(mem_op.operation.q_c) == 0;
}

void handle_memory_op(Program::Opcode::MemoryOp const& mem_op, BlockConstraint& block)
{
    uint8_t access_type = 1;
    if (is_rom(mem_op.op)) {
        access_type = 0;
    }
    if (access_type == 1) {
        // We are not allowed to write on the databus
        ASSERT((block.type != BlockType::CallData) && (block.type != BlockType::ReturnData));
        block.type = BlockType::RAM;
    }

    MemOp acir_mem_op = MemOp{ .access_type = access_type,
                               .index = serialize_arithmetic_gate(mem_op.op.index),
                               .value = serialize_arithmetic_gate(mem_op.op.value) };
    block.trace.push_back(acir_mem_op);
}

AcirFormat circuit_serde_to_acir_format(Program::Circuit const& circuit, bool honk_recursion)
{
    AcirFormat af;
    // `varnum` is the true number of variables, thus we add one to the index which starts at zero
    af.varnum = circuit.current_witness_index + 1;
    af.recursive = circuit.recursive;
    af.num_acir_opcodes = static_cast<uint32_t>(circuit.opcodes.size());
    af.public_inputs = join({ map(circuit.public_parameters.value, [](auto e) { return e.value; }),
                              map(circuit.return_values.value, [](auto e) { return e.value; }) });
    // Map to a pair of: BlockConstraint, and list of opcodes associated with that BlockConstraint
    std::unordered_map<uint32_t, std::pair<BlockConstraint, std::vector<size_t>>> block_id_to_block_constraint;
    for (size_t i = 0; i < circuit.opcodes.size(); ++i) {
        auto gate = circuit.opcodes[i];
        std::visit(
            [&](auto&& arg) {
                using T = std::decay_t<decltype(arg)>;
                if constexpr (std::is_same_v<T, Program::Opcode::AssertZero>) {
                    handle_arithmetic(arg, af, i);
                } else if constexpr (std::is_same_v<T, Program::Opcode::BlackBoxFuncCall>) {
                    handle_blackbox_func_call(arg, af, honk_recursion, i);
                } else if constexpr (std::is_same_v<T, Program::Opcode::MemoryInit>) {
                    auto block = handle_memory_init(arg);
                    uint32_t block_id = arg.block_id.value;
                    std::vector<size_t> opcode_indices = { i };
                    block_id_to_block_constraint[block_id] = std::make_pair(block, opcode_indices);
                } else if constexpr (std::is_same_v<T, Program::Opcode::MemoryOp>) {
                    auto block = block_id_to_block_constraint.find(arg.block_id.value);
                    if (block == block_id_to_block_constraint.end()) {
                        throw_or_abort("unitialized MemoryOp");
                    }
                    handle_memory_op(arg, block->second.first);
                    block->second.second.push_back(i);
                }
            },
            gate.value);
    }
    for (const auto& [block_id, block] : block_id_to_block_constraint) {
        // Note: the trace will always be empty for ReturnData since it cannot be explicitly read from in noir
        if (!block.first.trace.empty() || block.first.type == BlockType::ReturnData) {
            af.block_constraints.push_back(block.first);
            af.original_opcode_indices.block_constraints.push_back(block.second);
        }
    }
    return af;
}

AcirFormat circuit_buf_to_acir_format(std::vector<uint8_t> const& buf, bool honk_recursion)
{
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/927): Move to using just
    // `program_buf_to_acir_format` once Honk fully supports all ACIR test flows For now the backend still expects
    // to work with a single ACIR function
    auto circuit = Program::Program::bincodeDeserialize(buf).functions[0];

    return circuit_serde_to_acir_format(circuit, honk_recursion);
}

/**
 * @brief Converts from the ACIR-native `WitnessMap` format to Barretenberg's internal `WitnessVector` format.
 *
 * @param witness_map ACIR-native `WitnessMap` deserialized from a buffer
 * @return A `WitnessVector` equivalent to the passed `WitnessMap`.
 * @note This transformation results in all unassigned witnesses within the `WitnessMap` being assigned the value 0.
 *       Converting the `WitnessVector` back to a `WitnessMap` is unlikely to return the exact same `WitnessMap`.
 */
WitnessVector witness_map_to_witness_vector(WitnessStack::WitnessMap const& witness_map)
{
    WitnessVector wv;
    size_t index = 0;
    for (auto& e : witness_map.value) {
        // ACIR uses a sparse format for WitnessMap where unused witness indices may be left unassigned.
        // To ensure that witnesses sit at the correct indices in the `WitnessVector`, we fill any indices
        // which do not exist within the `WitnessMap` with the dummy value of zero.
        while (index < e.first.value) {
            wv.push_back(fr(0));
            index++;
        }
        wv.push_back(fr(uint256_t(e.second)));
        index++;
    }
    return wv;
}

/**
 * @brief Converts from the ACIR-native `WitnessMap` format to Barretenberg's internal `WitnessVector` format.
 *
 * @param buf Serialized representation of a `WitnessMap`.
 * @return A `WitnessVector` equivalent to the passed `WitnessMap`.
 * @note This transformation results in all unassigned witnesses within the `WitnessMap` being assigned the value 0.
 *       Converting the `WitnessVector` back to a `WitnessMap` is unlikely to return the exact same `WitnessMap`.
 */
WitnessVector witness_buf_to_witness_data(std::vector<uint8_t> const& buf)
{
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/927): Move to using just
    // `witness_buf_to_witness_stack` once Honk fully supports all ACIR test flows. For now the backend still
    // expects to work with the stop of the `WitnessStack`.
    auto witness_stack = WitnessStack::WitnessStack::bincodeDeserialize(buf);
    auto w = witness_stack.stack[witness_stack.stack.size() - 1].witness;

    return witness_map_to_witness_vector(w);
}

std::vector<AcirFormat> program_buf_to_acir_format(std::vector<uint8_t> const& buf, bool honk_recursion)
{
    auto program = Program::Program::bincodeDeserialize(buf);

    std::vector<AcirFormat> constraint_systems;
    constraint_systems.reserve(program.functions.size());
    for (auto const& function : program.functions) {
        constraint_systems.emplace_back(circuit_serde_to_acir_format(function, honk_recursion));
    }

    return constraint_systems;
}

WitnessVectorStack witness_buf_to_witness_stack(std::vector<uint8_t> const& buf)
{
    auto witness_stack = WitnessStack::WitnessStack::bincodeDeserialize(buf);
    WitnessVectorStack witness_vector_stack;
    witness_vector_stack.reserve(witness_stack.stack.size());
    for (auto const& stack_item : witness_stack.stack) {
        witness_vector_stack.emplace_back(
            std::make_pair(stack_item.index, witness_map_to_witness_vector(stack_item.witness)));
    }
    return witness_vector_stack;
}

#ifndef __wasm__
AcirProgramStack get_acir_program_stack(std::string const& bytecode_path,
                                        std::string const& witness_path,
                                        bool honk_recursion)
{
    std::vector<uint8_t> bytecode = get_bytecode(bytecode_path);
    std::vector<AcirFormat> constraint_systems =
        program_buf_to_acir_format(bytecode,
                                   honk_recursion); // TODO(https://github.com/AztecProtocol/barretenberg/issues/1013):
                                                    // Remove honk recursion flag

    std::vector<uint8_t> witness_data = get_bytecode(witness_path);
    WitnessVectorStack witness_stack = witness_buf_to_witness_stack(witness_data);

    return { constraint_systems, witness_stack };
}
#endif
} // namespace acir_format