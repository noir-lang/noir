#include "../bool/bool.hpp"
#include "../byte_array/byte_array.hpp"
#include "field.hpp"
#include <gtest/gtest.h>
#include <plonk/composer/standard_composer.hpp>

namespace test_stdlib_field {
using namespace barretenberg;
using namespace plonk;

typedef stdlib::bool_t<waffle::StandardComposer> bool_t;
typedef stdlib::field_t<waffle::StandardComposer> field_t;
typedef stdlib::witness_t<waffle::StandardComposer> witness_t;
typedef stdlib::public_witness_t<waffle::StandardComposer> public_witness_t;
typedef stdlib::byte_array<waffle::StandardComposer> byte_array;

void fibbonaci(waffle::StandardComposer& composer)
{
    field_t a(stdlib::witness_t(&composer, fr::one()));
    field_t b(stdlib::witness_t(&composer, fr::one()));

    field_t c = a + b;

    for (size_t i = 0; i < 17; ++i) {
        b = a;
        a = c;
        c = a + b;
    }
}
uint64_t fidget(waffle::StandardComposer& composer)
{
    field_t a(public_witness_t(&composer, fr::one())); // a is a legit wire value in our circuit
    field_t b(&composer,
              (fr::one())); // b is just a constant, and should not turn up as a wire value in our circuit

    // this shouldn't create a constraint - we just need to scale the addition/multiplication gates that `a` is involved
    // in c should point to the same wire value as a
    field_t c = a + b;
    field_t d(&composer, fr::coset_generator(0)); // like b, d is just a constant and not a wire value

    // by this point, we shouldn't have added any constraints in our circuit
    for (size_t i = 0; i < 17; ++i) {
        c = c * d; // shouldn't create a constraint - just scales up c (which points to same wire value as a)
        c = c - d; // shouldn't create a constraint - just adds a constant term into c's gates
        c = c * a; // will create a constraint - both c and a are wires in our circuit (the same wire actually, so this
                   // is a square-ish gate)
    }

    // run the same computation using normal types so we can compare the output
    uint64_t aa = 1;
    uint64_t bb = 1;
    uint64_t cc = aa + bb;
    uint64_t dd = 5;
    for (size_t i = 0; i < 17; ++i) {
        cc = cc * dd;
        cc = cc - dd;
        cc = cc * aa;
    }
    return cc;
}

void generate_test_plonk_circuit(waffle::StandardComposer& composer, size_t num_gates)
{
    field_t a(public_witness_t(&composer, barretenberg::fr::random_element()));
    field_t b(public_witness_t(&composer, barretenberg::fr::random_element()));

    field_t c(&composer);
    for (size_t i = 0; i < (num_gates / 4) - 4; ++i) {
        c = a + b;
        c = a * c;
        a = b * b;
        b = c * c;
    }
}

TEST(stdlib_field, test_add_mul_with_constants)
{
    waffle::StandardComposer composer = waffle::StandardComposer();

    uint64_t expected = fidget(composer);
    waffle::Prover prover = composer.preprocess();
    EXPECT_EQ(prover.witness->wires.at("w_3")[18], fr(expected));

    EXPECT_EQ(prover.n, 32UL);
    waffle::Verifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_field, test_field_fibbonaci)
{
    waffle::StandardComposer composer = waffle::StandardComposer();

    fibbonaci(composer);

    waffle::Prover prover = composer.preprocess();

    EXPECT_EQ(prover.witness->wires.at("w_3")[17], fr(4181));
    EXPECT_EQ(prover.n, 32UL);
    waffle::Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_field, test_equality)
{
    waffle::StandardComposer composer = waffle::StandardComposer();

    field_t a(stdlib::witness_t(&composer, 4));
    field_t b(stdlib::witness_t(&composer, 4));
    bool_t r = a == b;

    EXPECT_EQ(r.get_value(), true);

    waffle::Prover prover = composer.preprocess();

    fr x = composer.get_variable(r.witness_index);
    EXPECT_EQ(x, fr(1));

    EXPECT_EQ(prover.n, 8UL);
    waffle::Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_field, test_equality_false)
{
    waffle::StandardComposer composer = waffle::StandardComposer();

    field_t a(stdlib::witness_t(&composer, 4));
    field_t b(stdlib::witness_t(&composer, 3));
    bool_t r = a == b;

    EXPECT_EQ(r.get_value(), false);

    waffle::Prover prover = composer.preprocess();

    fr x = composer.get_variable(r.witness_index);
    EXPECT_EQ(x, fr(0));

    EXPECT_EQ(prover.n, 8UL);
    waffle::Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_field, test_equality_with_constants)
{
    waffle::StandardComposer composer = waffle::StandardComposer();

    field_t a(stdlib::witness_t(&composer, 4));
    field_t b = 3;
    field_t c = 7;
    bool_t r = a * c == b * c + c && b + 1 == a;

    EXPECT_EQ(r.get_value(), true);

    waffle::Prover prover = composer.preprocess();

    fr x = composer.get_variable(r.witness_index);
    EXPECT_EQ(x, fr(1));

    EXPECT_EQ(prover.n, 16UL);
    waffle::Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_field, test_larger_circuit)
{
    size_t n = 16384;
    waffle::StandardComposer composer = waffle::StandardComposer(n);

    generate_test_plonk_circuit(composer, n);

    waffle::Prover prover = composer.preprocess();

    waffle::Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_field, is_zero)
{
    waffle::StandardComposer composer = waffle::StandardComposer();

    // yuck
    field_t a = (public_witness_t(&composer, fr::random_element()));
    field_t b = (public_witness_t(&composer, fr::neg_one()));
    field_t c_1(&composer, uint256_t(0x1122334455667788, 0x8877665544332211, 0xaabbccddeeff9933, 0x1122112211221122));
    field_t c_2(&composer, uint256_t(0xaabbccddeeff9933, 0x8877665544332211, 0x1122334455667788, 0x1122112211221122));
    field_t c_3(&composer, barretenberg::fr::one());

    field_t c_4 = c_1 + c_2;
    a = a * c_4 + c_4; // add some constant terms in to validate our normalization check works
    b = b * c_4 + c_4;
    b = (b - c_1 - c_2) / c_4;
    b = b + c_3;

    field_t d(&composer, fr::zero());
    field_t e(&composer, fr::one());

    const size_t old_n = composer.get_num_gates();
    bool_t d_zero = d.is_zero();
    bool_t e_zero = e.is_zero();
    const size_t new_n = composer.get_num_gates();
    EXPECT_EQ(old_n, new_n);

    bool_t a_zero = a.is_zero();
    bool_t b_zero = b.is_zero();

    EXPECT_EQ(a_zero.get_value(), false);
    EXPECT_EQ(b_zero.get_value(), true);
    EXPECT_EQ(d_zero.get_value(), true);
    EXPECT_EQ(e_zero.get_value(), false);

    waffle::Prover prover = composer.preprocess();

    waffle::Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_field, test_byte_array_input_output_consistency)
{
    waffle::StandardComposer composer = waffle::StandardComposer();

    fr a_expected = fr::random_element();
    fr b_expected = fr::random_element();

    field_t a = witness_t(&composer, a_expected);
    field_t b = witness_t(&composer, b_expected);

    byte_array arr(&composer);

    arr.write(a);
    arr.write(b);

    EXPECT_EQ(arr.size(), 64UL);

    field_t a_result(arr.slice(0, 32));
    field_t b_result(arr.slice(32));

    EXPECT_EQ(a_result.get_value(), a_expected);
    EXPECT_EQ(b_result.get_value(), b_expected);
}

} // namespace test_stdlib_field