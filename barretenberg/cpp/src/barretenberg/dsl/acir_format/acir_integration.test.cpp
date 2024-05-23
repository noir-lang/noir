#ifndef __wasm__
#include "barretenberg/bb/exec_pipe.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/dsl/acir_format/acir_to_constraint_buf.hpp"

#include <filesystem>
#include <gtest/gtest.h>

class AcirIntegrationTest : public ::testing::Test {
  public:
    static std::vector<uint8_t> get_bytecode(const std::string& bytecodePath)
    {
        std::filesystem::path filePath = bytecodePath;
        if (filePath.extension() == ".json") {
            // Try reading json files as if they are a Nargo build artifact
            std::string command = "jq -r '.bytecode' \"" + bytecodePath + "\" | base64 -d | gunzip -c";
            return exec_pipe(command);
        }

        // For other extensions, assume file is a raw ACIR program
        std::string command = "gunzip -c \"" + bytecodePath + "\"";
        return exec_pipe(command);
    }

    // Function to check if a file exists
    bool file_exists(const std::string& path)
    {
        std::ifstream file(path);
        return file.good();
    }

    acir_format::AcirProgramStack get_program_stack_data_from_test_file(const std::string& test_program_name)
    {
        std::string base_path = "../../acir_tests/acir_tests/" + test_program_name + "/target";
        std::string bytecode_path = base_path + "/program.json";
        std::string witness_path = base_path + "/witness.gz";

        return acir_format::get_acir_program_stack(bytecode_path, witness_path);
    }

    acir_format::AcirProgram get_program_data_from_test_file(const std::string& test_program_name)
    {
        auto program_stack = get_program_stack_data_from_test_file(test_program_name);
        ASSERT(program_stack.size() == 1); // Otherwise this method will not return full stack data

        return program_stack.back();
    }

    template <class Flavor> bool prove_and_verify_honk(Flavor::CircuitBuilder& builder)
    {
        using Prover = UltraProver_<Flavor>;
        using Verifier = UltraVerifier_<Flavor>;
        using VerificationKey = Flavor::VerificationKey;

        Prover prover{ builder };
        // builder.blocks.summarize();
        // info("num gates          = ", builder.get_num_gates());
        // info("total circuit size = ", builder.get_total_circuit_size());
        // info("circuit size       = ", prover.instance->proving_key.circuit_size);
        // info("log circuit size   = ", prover.instance->proving_key.log_circuit_size);
        auto proof = prover.construct_proof();

        // Verify Honk proof
        auto verification_key = std::make_shared<VerificationKey>(prover.instance->proving_key);
        Verifier verifier{ verification_key };

        return verifier.verify_proof(proof);
    }
};

class AcirIntegrationSingleTest : public AcirIntegrationTest, public testing::WithParamInterface<std::string> {
  protected:
    static void SetUpTestSuite() { srs::init_crs_factory("../srs_db/ignition"); }
};

class AcirIntegrationFoldingTest : public AcirIntegrationTest, public testing::WithParamInterface<std::string> {
  protected:
    static void SetUpTestSuite() { srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_P(AcirIntegrationSingleTest, ProveAndVerifyProgram)
{
    using Flavor = GoblinUltraFlavor;
    using Builder = Flavor::CircuitBuilder;

    std::string test_name = GetParam();
    info("Test: ", test_name);
    acir_format::AcirProgram acir_program = get_program_data_from_test_file(test_name);

    // Construct a bberg circuit from the acir representation
    Builder builder = acir_format::create_circuit<Builder>(acir_program.constraints, 0, acir_program.witness);

    // Construct and verify Honk proof
    EXPECT_TRUE(prove_and_verify_honk<Flavor>(builder));
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/994): Run all tests
INSTANTIATE_TEST_SUITE_P(AcirTests,
                         AcirIntegrationSingleTest,
                         testing::Values("1327_concrete_in_generic",
                                         "1_mul",
                                         "2_div",
                                         "3_add",
                                         "4_sub",
                                         "5_over",
                                         "6",
                                         "6_array",
                                         "7",
                                         "7_function",
                                         "aes128_encrypt",
                                         "arithmetic_binary_operations",
                                         "array_dynamic",
                                         "array_dynamic_blackbox_input",
                                         "array_dynamic_main_output",
                                         "array_dynamic_nested_blackbox_input",
                                         "array_eq",
                                         "array_if_cond_simple",
                                         "array_len",
                                         "array_neq",
                                         "array_sort",
                                         "array_to_slice",
                                         "array_to_slice_constant_length",
                                         "assert",
                                         "assert_statement",
                                         "assert_statement_recursive",
                                         "assign_ex",
                                         "bigint",
                                         "bit_and",
                                         "bit_not",
                                         "bit_shifts_comptime",
                                         "bit_shifts_runtime",
                                         "blake3",
                                         "bool_not",
                                         "bool_or",
                                         "break_and_continue",
                                         "brillig_acir_as_brillig",
                                         "brillig_array_eq",
                                         "brillig_array_to_slice",
                                         "brillig_arrays",
                                         "brillig_assert",
                                         "brillig_bit_shifts_runtime",
                                         "brillig_blake2s",
                                         "brillig_blake3",
                                         "brillig_calls",
                                         "brillig_calls_array",
                                         "brillig_calls_conditionals",
                                         "brillig_conditional",
                                         "brillig_cow",
                                         "brillig_cow_assign",
                                         "brillig_cow_regression",
                                         "brillig_ecdsa_secp256k1",
                                         "brillig_ecdsa_secp256r1",
                                         "brillig_embedded_curve",
                                         "brillig_fns_as_values",
                                         "brillig_hash_to_field",
                                         "brillig_identity_function",
                                         "brillig_keccak",
                                         "brillig_loop",
                                         "brillig_nested_arrays",
                                         "brillig_not",
                                         "brillig_oracle",
                                         "brillig_pedersen",
                                         "brillig_recursion",
                                         "brillig_references",
                                         //  "brillig_scalar_mul",
                                         "brillig_schnorr",
                                         "brillig_sha256",
                                         "brillig_signed_cmp",
                                         "brillig_signed_div",
                                         "brillig_slice_input",
                                         "brillig_slices",
                                         "brillig_to_be_bytes",
                                         "brillig_to_bits",
                                         "brillig_to_bytes_integration",
                                         "brillig_to_le_bytes",
                                         "brillig_top_level",
                                         "brillig_unitialised_arrays",
                                         "brillig_wrapping",
                                         "cast_bool",
                                         "closures_mut_ref",
                                         "conditional_1",
                                         "conditional_2",
                                         "conditional_regression_421",
                                         "conditional_regression_547",
                                         "conditional_regression_661",
                                         "conditional_regression_short_circuit",
                                         "conditional_regression_underflow",
                                         "custom_entry",
                                         "databus",
                                         "debug_logs",
                                         "diamond_deps_0",
                                         //  "distinct_keyword",
                                         "double_verify_nested_proof",
                                         "double_verify_proof",
                                         "double_verify_proof_recursive",
                                         "ecdsa_secp256k1",
                                         "ecdsa_secp256r1",
                                         "eddsa",
                                         "embedded_curve_ops",
                                         "field_attribute",
                                         "generics",
                                         "global_consts",
                                         "hash_to_field",
                                         "hashmap",
                                         "higher_order_functions",
                                         "if_else_chain",
                                         "import",
                                         "inline_never_basic",
                                         "integer_array_indexing",
                                         "keccak256",
                                         "main_bool_arg",
                                         "main_return",
                                         "merkle_insert",
                                         "missing_closure_env",
                                         "modules",
                                         "modules_more",
                                         "modulus",
                                         "nested_array_dynamic",
                                         "nested_array_dynamic_simple",
                                         "nested_array_in_slice",
                                         "nested_arrays_from_brillig",
                                         "no_predicates_basic",
                                         "no_predicates_brillig",
                                         "no_predicates_numeric_generic_poseidon",
                                         "operator_overloading",
                                         "pedersen_check",
                                         "pedersen_commitment",
                                         "pedersen_hash",
                                         "poseidon_bn254_hash",
                                         "poseidonsponge_x5_254",
                                         "pred_eq",
                                         "prelude",
                                         "references",
                                         "regression",
                                         "regression_2660",
                                         "regression_3051",
                                         "regression_3394",
                                         "regression_3607",
                                         "regression_3889",
                                         "regression_4088",
                                         "regression_4124",
                                         "regression_4202",
                                         "regression_4383",
                                         "regression_4436",
                                         "regression_4449",
                                         "regression_4709",
                                         "regression_capacity_tracker",
                                         "regression_mem_op_predicate",
                                         "regression_method_cannot_be_found",
                                         //  "regression_sha256_slice",
                                         "regression_struct_array_conditional",
                                         //  "scalar_mul",
                                         "schnorr",
                                         "sha256",
                                         "sha2_byte",
                                         "side_effects_constrain_array",
                                         "signed_arithmetic",
                                         "signed_comparison",
                                         "signed_division",
                                         "simple_2d_array",
                                         "simple_add_and_ret_arr",
                                         "simple_array_param",
                                         "simple_bitwise",
                                         "simple_comparison",
                                         "simple_mut",
                                         "simple_not",
                                         "simple_print",
                                         "simple_program_addition",
                                         "simple_radix",
                                         "simple_shield",
                                         "simple_shift_left_right",
                                         "slice_coercion",
                                         "slice_dynamic_index",
                                         "slice_init_with_complex_type",
                                         "slice_loop",
                                         "slices",
                                         "strings",
                                         "struct",
                                         "struct_array_inputs",
                                         "struct_fields_ordering",
                                         "struct_inputs",
                                         "submodules",
                                         "to_be_bytes",
                                         "to_bytes_consistent",
                                         "to_bytes_integration",
                                         "to_le_bytes",
                                         "trait_as_return_type",
                                         "trait_impl_base_type",
                                         "traits_in_crates_1",
                                         "traits_in_crates_2",
                                         "tuple_inputs",
                                         "tuples",
                                         "type_aliases",
                                         "u128",
                                         "u16_support",
                                         "unconstrained_empty",
                                         "unit_value",
                                         "unsafe_range_constraint",
                                         "witness_compression",
                                         "xor"));

TEST_P(AcirIntegrationFoldingTest, ProveAndVerifyProgramStack)
{
    using Flavor = GoblinUltraFlavor;
    using Builder = Flavor::CircuitBuilder;

    std::string test_name = GetParam();
    info("Test: ", test_name);

    auto program_stack = get_program_stack_data_from_test_file(test_name);

    while (!program_stack.empty()) {
        auto program = program_stack.back();

        // Construct a bberg circuit from the acir representation
        auto builder = acir_format::create_circuit<Builder>(program.constraints, 0, program.witness);

        // Construct and verify Honk proof for the individidual circuit
        EXPECT_TRUE(prove_and_verify_honk<Flavor>(builder));

        program_stack.pop_back();
    }
}

INSTANTIATE_TEST_SUITE_P(AcirTests,
                         AcirIntegrationFoldingTest,
                         testing::Values("fold_after_inlined_calls",
                                         "fold_basic",
                                         "fold_basic_nested_call",
                                         "fold_call_witness_condition",
                                         "fold_complex_outputs",
                                         "fold_distinct_return",
                                         "fold_fibonacci",
                                         "fold_numeric_generic_poseidon"));
#endif