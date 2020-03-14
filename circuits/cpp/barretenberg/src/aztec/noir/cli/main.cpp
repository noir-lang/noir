#include <fstream>
#include <iostream>
#include <noir/compiler/code_gen/compiler.hpp>
#include <noir/compiler/parser/parse.hpp>
#include <vector>

using namespace barretenberg;
using namespace noir::parser;
using namespace noir::code_gen;

namespace boost {
void throw_exception(std::exception const&)
{
    std::abort();
}
} // namespace boost

template <typename T> std::ostream& operator<<(std::ostream& os, std::vector<T> const& v)
{
    for (auto const& e : v) {
        os << std::hex << (int)e << ',';
    }
    return os;
}

template <typename T> void test_sha256(T const& input, std::vector<uint8_t> const& expected)
{
    std::ifstream file("../src/noir/compiler/code_gen/fixtures/sha256.noir");
    std::string code((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());
    auto ast = parse(code);

    auto composer = Composer();

    auto compiler = Compiler(composer);
    auto r = compiler.start(ast, { var_t(input, composer) });
    printf("composer gates = %zu\n", composer.get_num_gates());

    auto output_vars = boost::get<std::vector<var_t>>(r.first.value());
    std::vector<uint8_t> output;
    std::transform(output_vars.begin(), output_vars.end(), std::back_inserter(output), [](var_t const& v) {
        return static_cast<uint8_t>(boost::get<uint_nt>(v.value()).get_value());
    });

    if (output != expected) {
        std::cout << "Expected result: " << expected << std::endl;
        std::cout << "Received result: " << output << std::endl;
    }

    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = r.second.construct_proof();

    bool result = verifier.verify_proof(proof);
    std::cout << "Verified: " << result << std::endl;
}

int main(void)
{
    std::string input = "abc";
    std::vector<uint8_t> expected = {
        0xBA, 0x78, 0x16, 0xBF, 0x8F, 0x01, 0xCF, 0xEA, 0x41, 0x41, 0x40, 0xDE, 0x5D, 0xAE, 0x22, 0x23,
        0xB0, 0x03, 0x61, 0xA3, 0x96, 0x17, 0x7A, 0x9C, 0xB4, 0x10, 0xFF, 0x61, 0xF2, 0x00, 0x15, 0xAD,
    };
    test_sha256(input, expected);
    return 0;
}