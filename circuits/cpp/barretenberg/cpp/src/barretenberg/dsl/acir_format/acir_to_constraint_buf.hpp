#pragma once
#include "acir_format.hpp"
#include "barretenberg/common/container.hpp"
#include "barretenberg/dsl/acir_format/blake2s_constraint.hpp"
#include "barretenberg/dsl/acir_format/block_constraint.hpp"
#include "barretenberg/dsl/acir_format/ecdsa_secp256k1.hpp"
#include "barretenberg/dsl/acir_format/hash_to_field.hpp"
#include "barretenberg/dsl/acir_format/keccak_constraint.hpp"
#include "barretenberg/dsl/acir_format/logic_constraint.hpp"
#include "barretenberg/dsl/acir_format/pedersen.hpp"
#include "barretenberg/dsl/acir_format/range_constraint.hpp"
#include "barretenberg/dsl/acir_format/recursion_constraint.hpp"
#include "barretenberg/dsl/acir_format/schnorr_verify.hpp"
#include "barretenberg/dsl/acir_format/sha256_constraint.hpp"
#include "barretenberg/proof_system/arithmetization/gate_data.hpp"
#include "serde/index.hpp"
#include <iterator>

namespace acir_format {

poly_triple serialize_arithmetic_gate(Circuit::Expression const& arg)
{
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
    // Think this never longer than 1?
    for (const auto& e : arg.mul_terms) {
        uint256_t qm(std::get<0>(e));
        uint32_t a = std::get<1>(e).value;
        uint32_t b = std::get<2>(e).value;
        pt.q_m = qm;
        pt.a = a;
        pt.b = b;
    }
    for (const auto& e : arg.linear_combinations) {
        barretenberg::fr x(uint256_t(std::get<0>(e)));
        uint32_t witness = std::get<1>(e).value;

        if (pt.a == 0 || pt.a == witness) {
            pt.a = witness;
            pt.q_l = x;
        } else if (pt.b == 0 || pt.b == witness) {
            pt.b = witness;
            pt.q_r = x;
        } else if (pt.c == 0 || pt.c == witness) {
            pt.c = witness;
            pt.q_o = x;
        } else {
            throw_or_abort("Cannot assign linear term to a constrain of width 3");
        }
    }
    pt.q_c = uint256_t(arg.q_c);
    return pt;
}

void handle_arithmetic(Circuit::Opcode::Arithmetic const& arg, acir_format& af)
{
    af.constraints.push_back(serialize_arithmetic_gate(arg.value));
}

void handle_blackbox_func_call(Circuit::Opcode::BlackBoxFuncCall const& arg, acir_format& af)
{
    std::visit(
        [&](auto&& arg) {
            using T = std::decay_t<decltype(arg)>;
            if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::AND>) {
                af.logic_constraints.push_back(LogicConstraint{
                    .a = arg.lhs.witness.value,
                    .b = arg.rhs.witness.value,
                    .result = arg.output.value,
                    .num_bits = arg.lhs.num_bits,
                    .is_xor_gate = false,
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::XOR>) {
                af.logic_constraints.push_back(LogicConstraint{
                    .a = arg.lhs.witness.value,
                    .b = arg.rhs.witness.value,
                    .result = arg.output.value,
                    .num_bits = arg.lhs.num_bits,
                    .is_xor_gate = true,
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::RANGE>) {
                af.range_constraints.push_back(RangeConstraint{
                    .witness = arg.input.witness.value,
                    .num_bits = arg.input.num_bits,
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::SHA256>) {
                af.sha256_constraints.push_back(Sha256Constraint{
                    .inputs = map(arg.inputs,
                                  [](auto& e) {
                                      return Sha256Input{
                                          .witness = e.witness.value,
                                          .num_bits = e.num_bits,
                                      };
                                  }),
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::Blake2s>) {
                af.blake2s_constraints.push_back(Blake2sConstraint{
                    .inputs = map(arg.inputs,
                                  [](auto& e) {
                                      return Blake2sInput{
                                          .witness = e.witness.value,
                                          .num_bits = e.num_bits,
                                      };
                                  }),
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::SchnorrVerify>) {
                af.schnorr_constraints.push_back(SchnorrConstraint{
                    .message = map(arg.message, [](auto& e) { return e.witness.value; }),
                    .public_key_x = arg.public_key_x.witness.value,
                    .public_key_y = arg.public_key_y.witness.value,
                    .result = arg.output.value,
                    .signature = map(arg.signature, [](auto& e) { return e.witness.value; }),
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::Pedersen>) {
                af.pedersen_constraints.push_back(PedersenConstraint{
                    .scalars = map(arg.inputs, [](auto& e) { return e.witness.value; }),
                    .hash_index = arg.domain_separator,
                    .result_x = arg.outputs[0].value,
                    .result_y = arg.outputs[1].value,
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::HashToField128Security>) {
                af.hash_to_field_constraints.push_back(HashToFieldConstraint{
                    .inputs = map(arg.inputs,
                                  [](auto& e) {
                                      return HashToFieldInput{
                                          .witness = e.witness.value,
                                          .num_bits = e.num_bits,
                                      };
                                  }),
                    .result = arg.output.value,
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::EcdsaSecp256k1>) {
                af.ecdsa_k1_constraints.push_back(EcdsaSecp256k1Constraint{
                    .hashed_message = map(arg.hashed_message, [](auto& e) { return e.witness.value; }),
                    .signature = map(arg.signature, [](auto& e) { return e.witness.value; }),
                    .pub_x_indices = map(arg.public_key_x, [](auto& e) { return e.witness.value; }),
                    .pub_y_indices = map(arg.public_key_y, [](auto& e) { return e.witness.value; }),
                    .result = arg.output.value,
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::EcdsaSecp256r1>) {
                af.ecdsa_r1_constraints.push_back(EcdsaSecp256r1Constraint{
                    .hashed_message = map(arg.hashed_message, [](auto& e) { return e.witness.value; }),
                    .pub_x_indices = map(arg.public_key_x, [](auto& e) { return e.witness.value; }),
                    .pub_y_indices = map(arg.public_key_y, [](auto& e) { return e.witness.value; }),
                    .result = arg.output.value,
                    .signature = map(arg.signature, [](auto& e) { return e.witness.value; }),
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::FixedBaseScalarMul>) {
                af.fixed_base_scalar_mul_constraints.push_back(FixedBaseScalarMul{
                    .scalar = arg.input.witness.value,
                    .pub_key_x = arg.outputs[0].value,
                    .pub_key_y = arg.outputs[1].value,
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::Keccak256>) {
                af.keccak_constraints.push_back(KeccakConstraint{
                    .inputs = map(arg.inputs,
                                  [](auto& e) {
                                      return HashInput{
                                          .witness = e.witness.value,
                                          .num_bits = e.num_bits,
                                      };
                                  }),
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::Keccak256VariableLength>) {
                af.keccak_var_constraints.push_back(KeccakVarConstraint{
                    .inputs = map(arg.inputs,
                                  [](auto& e) {
                                      return HashInput{
                                          .witness = e.witness.value,
                                          .num_bits = e.num_bits,
                                      };
                                  }),
                    .result = map(arg.outputs, [](auto& e) { return e.value; }),
                    .var_message_size = arg.var_message_size.witness.value,
                });
            } else if constexpr (std::is_same_v<T, Circuit::BlackBoxFuncCall::RecursiveAggregation>) {
                auto c = RecursionConstraint{
                    .key = map(arg.verification_key, [](auto& e) { return e.witness.value; }),
                    .proof = map(arg.proof, [](auto& e) { return e.witness.value; }),
                    .public_inputs = map(arg.public_inputs, [](auto& e) { return e.witness.value; }),
                    .key_hash = arg.key_hash.witness.value,
                    .input_aggregation_object = {},
                    .output_aggregation_object = {},
                    .nested_aggregation_object = {},
                };
                if (arg.input_aggregation_object.has_value()) {
                    for (size_t i = 0; i < RecursionConstraint::AGGREGATION_OBJECT_SIZE; ++i) {
                        c.input_aggregation_object[i] = (*arg.input_aggregation_object)[i].witness.value;
                    }
                }
                for (size_t i = 0; i < RecursionConstraint::AGGREGATION_OBJECT_SIZE; ++i) {
                    c.output_aggregation_object[i] = arg.output_aggregation_object[i].value;
                }
                af.recursion_constraints.push_back(c);
            }
        },
        arg.value.value);
}

void handle_memory(Circuit::MemoryBlock const& mem_block, bool is_ram, acir_format& af)
{
    std::vector<poly_triple> init;
    std::vector<MemOp> trace;
    auto len = mem_block.len;
    for (size_t i = 0; i < len; ++i) {
        init.push_back(serialize_arithmetic_gate(mem_block.trace[i].value));
    }
    for (size_t i = len; i < mem_block.trace.size(); ++i) {
        auto index = serialize_arithmetic_gate(mem_block.trace[i].index);
        auto value = serialize_arithmetic_gate(mem_block.trace[i].value);
        auto op = mem_block.trace[i].operation;
        if (!(op.mul_terms.empty() && op.linear_combinations.empty())) {
            throw_or_abort("Expected constant.");
        }
        bool access_type(uint256_t(op.q_c));
        trace.push_back(MemOp{
            .access_type = static_cast<uint8_t>(access_type),
            .index = index,
            .value = value,
        });
    }
    af.block_constraints.push_back(BlockConstraint{ .init = init, .trace = trace, .type = (BlockType)is_ram });
}

acir_format circuit_buf_to_acir_format(std::vector<uint8_t> const& buf)
{
    auto circuit = Circuit::Circuit::bincodeDeserialize(buf);

    acir_format af;
    af.varnum = circuit.current_witness_index + 1;
    af.public_inputs = join({ map(circuit.public_parameters.value, [](auto e) { return e.value; }),
                              map(circuit.return_values.value, [](auto e) { return e.value; }) });

    for (auto gate : circuit.opcodes) {
        std::visit(
            [&](auto&& arg) {
                using T = std::decay_t<decltype(arg)>;
                if constexpr (std::is_same_v<T, Circuit::Opcode::Arithmetic>) {
                    handle_arithmetic(arg, af);
                } else if constexpr (std::is_same_v<T, Circuit::Opcode::BlackBoxFuncCall>) {
                    handle_blackbox_func_call(arg, af);
                } else if constexpr (std::is_same_v<T, Circuit::Opcode::RAM>) {
                    handle_memory(arg.value, true, af);
                } else if constexpr (std::is_same_v<T, Circuit::Opcode::ROM>) {
                    handle_memory(arg.value, false, af);
                }
            },
            gate.value);
    }
    return af;
}

WitnessVector witness_buf_to_witness_data(std::vector<uint8_t> const& buf)
{
    auto w = WitnessMap::WitnessMap::bincodeDeserialize(buf);
    WitnessVector wv;
    size_t index = 1;
    for (auto& e : w.value) {
        while (index < e.first.value) {
            wv.push_back(barretenberg::fr(0));
            index++;
        }
        wv.push_back(barretenberg::fr(uint256_t(e.second)));
        index++;
    }
    return wv;
}

} // namespace acir_format
