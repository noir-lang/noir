#include <barretenberg/noir/ast.hpp>
#include <barretenberg/noir/compiler/compiler.hpp>
#include <barretenberg/noir/parse.hpp>
#include <barretenberg/waffle/proof_system/preprocess.hpp>
#include <barretenberg/waffle/proof_system/verifier/verifier.hpp>
#include <barretenberg/waffle/stdlib/uint/noir_uint.hpp>
#include <fstream>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace noir::parser;
using namespace noir::code_gen;

namespace boost {
void throw_exception(std::exception const&)
{
    std::abort();
}
} // namespace boost

TEST(noir, format_string)
{
    EXPECT_EQ(format("hello %s %d", "world", 123), "hello world 123");
}

// TEST(noir, parse_fails)
// {
//     EXPECT_THROW(parse("1 + 2; blah"), std::runtime_error);
// }

TEST(noir, uint_sizes)
{
    auto ast = parse("          \n\
        uint2 my_int2 = 0;      \n\
        uint3 my_int3 = 0;      \n\
        uint32 my_int32 = 0;    \n\
        uint64 my_int64 = 0;    \n\
    ");

    auto type_id = boost::get<noir::ast::variable_declaration>(ast[0]).type;
    auto int_type = boost::get<noir::ast::int_type>(type_id.type);
    EXPECT_EQ(int_type.size, 2UL);
}

// TEST(noir, uint1_fail)
// {
//     EXPECT_THROW(parse("uint1 my_int1 = 0;"), std::runtime_error);
// }

// TEST(noir, uint65_fail)
// {
//     EXPECT_THROW(parse("uint65 my_int65 = 0;"), std::runtime_error);
// }

TEST(noir, uint_indexing)
{
    std::string code = "            \n\
        uint32 main(uint32 a) {     \n\
            a[3 + 4] = true;        \n\
            a[30] = false;          \n\
            return a;               \n\
        }                           \n\
    ";
    auto ast = parse(code);

    auto composer = Composer();
    auto compiler = Compiler(composer);
    std::vector<var_t> inputs = { noir::code_gen::uint(32, witness_t(&composer, 7ULL)) };
    auto r = compiler.start(ast, inputs);
    EXPECT_EQ(boost::get<noir::code_gen::uint>(r.first.value()).get_value(), 16777221ULL);
}

TEST(noir, uint_vector_bit_indexing)
{
    std::string code = "            \n\
        uint32 main() {             \n\
            uint32[1] a = [0];      \n\
            a[0][31] = true;        \n\
            return a[0];            \n\
        }                           \n\
    ";
    auto ast = parse(code);

    auto composer = Composer();
    auto compiler = Compiler(composer);
    auto r = compiler.start(ast, {});
    EXPECT_EQ(boost::get<noir::code_gen::uint>(r.first.value()).get_value(), 1ULL);
}

TEST(noir, symbol_constant)
{
    std::string code = "            \n\
        uint32 main() {             \n\
            uint32 a = 3;           \n\
            a + 4;                  \n\
            return a;               \n\
        }                           \n\
    ";
    auto ast = parse(code);

    auto composer = Composer();
    auto compiler = Compiler(composer);
    auto r = compiler.start(ast, {});
    EXPECT_EQ(boost::get<noir::code_gen::uint>(r.first.value()).get_value(), 3ULL);
}

TEST(noir, function_definition)
{
    parse("uint32 my_function(uint32 arg1, bool arg2) {}");
}

TEST(noir, function_call)
{
    parse("bool x = my_function(arg1, 3+5+(x));");
}

TEST(noir, array_variable_definition)
{
    parse("uint32[4] my_var = [0x1, 0x12, 0x123, 0x1234];");
}

TEST(noir, array_expressions)
{
    parse_function_statements("uint32[4] my_var = [func_call(), 13, true];");
}

TEST(noir, array_index)
{
    parse_function_statements("my_var = some_array[5*3][1+2];");
}

TEST(noir, unary)
{
    parse_function_statements("my_var = !x;");
}

/*
TEST(noir, bool_circuit)
{
    std::string code = "                      \n\
    bool main(bool a, bool b) {               \n\
      a = a ^ b;         // a = 1             \n\
      b = !b;            // b = 1 (witness 0) \n\
      bool c = (a == b); // c = 1             \n\
      bool d;            // d = ?             \n\
      d = false;         // d = 0             \n\
      bool e = a | d;    // e = 1 = a         \n\
      bool f = e ^ b;    // f = 0             \n\
      d = (!f) & a;      // d = 1             \n\
    }                                         \n\
    ";
    auto ast = parse(code);

    auto composer = Composer();
    auto compiler = Compiler(composer);
    std::vector<var_t> inputs = { bool_t(witness_t(&composer, true)), bool_t(witness_t(&composer, false)) };
    auto r = compiler.start(ast, inputs);
    auto prover = std::move(r.second);

    EXPECT_EQ(prover.w_l[0], fr(1));
    EXPECT_EQ(prover.w_r[0], fr(1));
    EXPECT_EQ(prover.w_o[0], fr(1));
    EXPECT_EQ(prover.w_l[1], fr(0));
    EXPECT_EQ(prover.w_r[1], fr(0));
    EXPECT_EQ(prover.w_o[1], fr(0));
    EXPECT_EQ(prover.w_l[2], fr(1));
    EXPECT_EQ(prover.w_r[2], fr(0));
    EXPECT_EQ(prover.w_o[2], fr(1));
    EXPECT_EQ(prover.w_l[3], fr(1));
    EXPECT_EQ(prover.w_r[3], fr(0));
    EXPECT_EQ(prover.w_o[3], fr(1));
    EXPECT_EQ(prover.w_l[4], fr(1));
    EXPECT_EQ(prover.w_r[4], fr(0));
    EXPECT_EQ(prover.w_o[4], fr(0));
    EXPECT_EQ(prover.w_l[5], fr(0));
    EXPECT_EQ(prover.w_r[5], fr(1));
    EXPECT_EQ(prover.w_o[5], fr(1));
    EXPECT_EQ(prover.n, 8UL);
}
*/

// template <typename T> void test_sha256(T const& input, std::vector<uint8_t> const& expected)
// {
//     std::ifstream file("../test/noir/sha256.noir");
//     std::string code((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());
//     auto ast = parse(code);

//     auto composer = Composer();

//     auto compiler = Compiler(composer);
//     auto r = compiler.start(ast, { var_t(input, composer) });
//     printf("composer gates = %zu\n", composer.get_num_gates());

//     auto output_vars = boost::get<std::vector<var_t>>(r.first.value());
//     std::vector<uint8_t> output;
//     std::transform(output_vars.begin(), output_vars.end(), std::back_inserter(output), [](var_t const& v) {
//         return static_cast<uint8_t>(boost::get<noir::code_gen::uint>(v.value()).get_value());
//     });

//     EXPECT_EQ(output, expected);

//     waffle::TurboVerifier verifier = composer.create_verifier();
//     waffle::plonk_proof proof = r.second.construct_proof();

//     bool result = verifier.verify_proof(proof);
//     EXPECT_EQ(result, true);
// }

// TEST(noir, sha256_NIST_one)
// {
//     std::string input = "abc";
//     std::vector<uint8_t> expected = {
//         0xBA, 0x78, 0x16, 0xBF, 0x8F, 0x01, 0xCF, 0xEA, 0x41, 0x41, 0x40, 0xDE, 0x5D, 0xAE, 0x22, 0x23,
//         0xB0, 0x03, 0x61, 0xA3, 0x96, 0x17, 0x7A, 0x9C, 0xB4, 0x10, 0xFF, 0x61, 0xF2, 0x00, 0x15, 0xAD,
//     };
//     test_sha256(input, expected);
// }

// TEST(noir, sha256_NIST_two)
// {
//     std::string input = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
//     std::vector<uint8_t> expected = {
//         0x24, 0x8D, 0x6A, 0x61, 0xD2, 0x06, 0x38, 0xB8, 0xE5, 0xC0, 0x26, 0x93, 0x0C, 0x3E, 0x60, 0x39,
//         0xA3, 0x3C, 0xE4, 0x59, 0x64, 0xFF, 0x21, 0x67, 0xF6, 0xEC, 0xED, 0xD4, 0x19, 0xDB, 0x06, 0xC1,
//     };
//     test_sha256(input, expected);
// }

// TEST(noir, sha256_NIST_three)
// {
//     std::vector<uint8_t> input = { 0xbd };
//     std::vector<uint8_t> expected = {
//         0x68, 0x32, 0x57, 0x20, 0xaa, 0xbd, 0x7c, 0x82, 0xf3, 0x0f, 0x55, 0x4b, 0x31, 0x3d, 0x05, 0x70,
//         0xc9, 0x5a, 0xcc, 0xbb, 0x7d, 0xc4, 0xb5, 0xaa, 0xe1, 0x12, 0x04, 0xc0, 0x8f, 0xfe, 0x73, 0x2b,
//     };
//     test_sha256(input, expected);
// }

// TEST(noir, sha256_NIST_four)
// {
//     std::vector<uint8_t> input = { 0xc9, 0x8c, 0x8e, 0x55 };
//     std::vector<uint8_t> expected = {
//         0x7a, 0xbc, 0x22, 0xc0, 0xae, 0x5a, 0xf2, 0x6c, 0xe9, 0x3d, 0xbb, 0x94, 0x43, 0x3a, 0x0e, 0x0b,
//         0x2e, 0x11, 0x9d, 0x01, 0x4f, 0x8e, 0x7f, 0x65, 0xbd, 0x56, 0xc6, 0x1c, 0xcc, 0xcd, 0x95, 0x04,
//     };
//     test_sha256(input, expected);
// }