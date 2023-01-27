#include "../bool/bool.hpp"
#include "field.hpp"
#include "plonk/proof_system/constants.hpp"
#include <gtest/gtest.h>
#include <honk/composer/standard_honk_composer.hpp>
#include <plonk/composer/standard_composer.hpp>
#include <plonk/composer/ultra_composer.hpp>
#include <plonk/composer/turbo_composer.hpp>
#include <numeric/random/engine.hpp>

using namespace bonk;

namespace test_stdlib_field {

namespace {
auto& engine = numeric::random::get_debug_engine();
}

template <class T> void ignore_unused(T&) {} // use to ignore unused variables in lambdas

using namespace barretenberg;
using namespace plonk;

template <typename Composer> class stdlib_field : public testing::Test {
    typedef stdlib::bool_t<Composer> bool_ct;
    typedef stdlib::field_t<Composer> field_ct;
    typedef stdlib::witness_t<Composer> witness_ct;
    typedef stdlib::public_witness_t<Composer> public_witness_ct;

    static void fibbonaci(Composer& composer)
    {
        field_ct a(witness_ct(&composer, fr::one()));
        field_ct b(witness_ct(&composer, fr::one()));

        field_ct c = a + b;

        for (size_t i = 0; i < 17; ++i) {
            b = a;
            a = c;
            c = a + b;
        }
    }
    static uint64_t fidget(Composer& composer)
    {
        field_ct a(public_witness_ct(&composer, fr::one())); // a is a legit wire value in our circuit
        field_ct b(&composer,
                   (fr::one())); // b is just a constant, and should not turn up as a wire value in our circuit

        // this shouldn't create a constraint - we just need to scale the addition/multiplication gates that `a` is
        // involved in c should point to the same wire value as a
        field_ct c = a + b;
        field_ct d(&composer, fr::coset_generator(0)); // like b, d is just a constant and not a wire value

        // by this point, we shouldn't have added any constraints in our circuit
        for (size_t i = 0; i < 17; ++i) {
            c = c * d; // shouldn't create a constraint - just scales up c (which points to same wire value as a)
            c = c - d; // shouldn't create a constraint - just adds a constant term into c's gates
            c = c * a; // will create a constraint - both c and a are wires in our circuit (the same wire actually, so
                       // this is a square-ish gate)
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

    static void generate_test_plonk_circuit(Composer& composer, size_t num_gates)
    {
        field_ct a(public_witness_ct(&composer, barretenberg::fr::random_element()));
        field_ct b(public_witness_ct(&composer, barretenberg::fr::random_element()));

        field_ct c(&composer);
        for (size_t i = 0; i < (num_gates / 4) - 4; ++i) {
            c = a + b;
            c = a * c;
            a = b * b;
            b = c * c;
        }
    }

  public:
    static void create_range_constraint()
    {
        auto run_test = [&](fr elt, size_t num_bits, bool expect_verified) {
            Composer composer = Composer();
            field_ct a(witness_ct(&composer, elt));
            a.create_range_constraint(num_bits, "field_tests: range_constraint on a fails");

            auto prover = composer.create_prover();
            auto verifier = composer.create_verifier();
            waffle::plonk_proof proof = prover.construct_proof();
            bool verified = verifier.verify_proof(proof);
            EXPECT_EQ(verified, expect_verified);
            if (verified != expect_verified) {
                info("Range constraint malfunction on ", elt, " with num_bits ", num_bits);
            }
        };

        run_test(2, 1, false);
        run_test(2, 2, true);
        run_test(3, 2, true);
        // 130 = 0b10000010, 8 bits
        for (size_t num_bits = 1; num_bits < 17; num_bits++) {
            run_test(130, num_bits, num_bits < 8 ? false : true);
        }

        // -1 has maximum bit length
        run_test(-1, fr::modulus.get_msb(), false);
        run_test(-1, 128, false);
        run_test(-1, fr::modulus.get_msb() + 1, true);
    }

    /**
     * @brief Demonstrate current behavior of assert_equal.
     */
    static void test_assert_equal()
    {
        auto run_test = [](bool constrain, bool true_when_y_val_zero = true) {
            Composer composer = Composer();
            field_ct x = witness_ct(&composer, 1);
            field_ct y = witness_ct(&composer, 0);

            // With no constraints, the proof verification will pass even though
            // we assert x and y are equal.
            bool expected_result = true;

            if (constrain) {
                /* The fact that we have a passing test in both cases that follow tells us
                 * that the failure in the first case comes from the additive constraint,
                 * not from a copy constraint. That failure is because the assert_equal
                 * below says that 'the value of y was always x'--the value 1 is substituted
                 * for x when evaluating the gate identity.
                 */
                if (true_when_y_val_zero) {
                    // constraint: 0*x + 1*y + 0*0 + 0 == 0
                    add_triple t{ .a = x.witness_index,
                                  .b = y.witness_index,
                                  .c = composer.zero_idx,
                                  .a_scaling = 0,
                                  .b_scaling = 1,
                                  .c_scaling = 0,
                                  .const_scaling = 0 };

                    composer.create_add_gate(t);
                    expected_result = false;
                } else {
                    // constraint: 0*x + 1*y + 0*0 - 1 == 0
                    add_triple t{ .a = x.witness_index,
                                  .b = y.witness_index,
                                  .c = composer.zero_idx,
                                  .a_scaling = 0,
                                  .b_scaling = 1,
                                  .c_scaling = 0,
                                  .const_scaling = -1 };

                    composer.create_add_gate(t);
                    expected_result = true;
                }
            }

            x.assert_equal(y);

            // both field elements have real value 1 now
            EXPECT_EQ(x.get_value(), 1);
            EXPECT_EQ(y.get_value(), 1);

            auto prover = composer.create_prover();
            waffle::plonk_proof proof = prover.construct_proof();
            auto verifier = composer.create_verifier();
            bool result = verifier.verify_proof(proof);

            EXPECT_EQ(result, expected_result);
        };

        run_test(false);
        run_test(true, true);
        run_test(true, false);
    }

    static void test_add_mul_with_constants()
    {
        Composer composer = Composer();

        uint64_t expected = fidget(composer);
        auto prover = composer.create_prover();

        if constexpr (Composer::type == waffle::ComposerType::STANDARD_HONK) {
            EXPECT_EQ(prover.key->polynomial_cache.get("w_3_lagrange")[20], fr(expected));
        } else {
            EXPECT_EQ(prover.key->polynomial_cache.get("w_3_lagrange")[18], fr(expected));
        }

        EXPECT_EQ(prover.n, 32UL);
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_div()
    {
        Composer composer = Composer();

        field_ct a = witness_ct(&composer, barretenberg::fr::random_element());
        a *= barretenberg::fr::random_element();
        a += barretenberg::fr::random_element();

        field_ct b = witness_ct(&composer, barretenberg::fr::random_element());
        b *= barretenberg::fr::random_element();
        b += barretenberg::fr::random_element();

        // numerator constant
        field_ct out = field_ct(&composer, b.get_value()) / a;
        EXPECT_EQ(out.get_value(), b.get_value() / a.get_value());

        out = b / a;
        EXPECT_EQ(out.get_value(), b.get_value() / a.get_value());

        // denominator constant
        out = a / b.get_value();
        EXPECT_EQ(out.get_value(), a.get_value() / b.get_value());

        // numerator 0
        out = field_ct(0) / b;
        EXPECT_EQ(out.get_value(), 0);
        EXPECT_EQ(out.is_constant(), true);

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_field_fibbonaci()
    {
        Composer composer = Composer();

        fibbonaci(composer);

        auto prover = composer.create_prover();

        if constexpr (Composer::type == waffle::ComposerType::STANDARD_HONK) {
            EXPECT_EQ(prover.key->polynomial_cache.get("w_3_lagrange")[19], fr(4181));
        } else {
            EXPECT_EQ(prover.key->polynomial_cache.get("w_3_lagrange")[17], fr(4181));
        }

        EXPECT_EQ(prover.n, 32UL);
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_field_pythagorean()
    {
        Composer composer = Composer();
        field_ct a(witness_ct(&composer, 3));
        field_ct b(witness_ct(&composer, 4));
        field_ct c(witness_ct(&composer, 5));

        field_ct a_sqr = a * a;
        field_ct b_sqr = b * b;
        field_ct c_sqr = c * c;
        c_sqr.set_public();
        field_ct sum_sqrs = a_sqr + b_sqr;

        // composer.assert_equal(sum_sqrs.witness_index, c_sqr.witness_index, "triple is not pythagorean");
        c_sqr.assert_equal(sum_sqrs);

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool verified = verifier.verify_proof(proof);

        for (size_t i = 0; i < composer.variables.size(); i++) {
            info(i, composer.variables[i]);
        }
        ASSERT_TRUE(verified);
    }

    static void test_equality()
    {
        Composer composer = Composer();

        field_ct a(witness_ct(&composer, 4));
        field_ct b(witness_ct(&composer, 4));
        bool_ct r = a == b;

        EXPECT_EQ(r.get_value(), true);

        auto prover = composer.create_prover();

        fr x = composer.get_variable(r.witness_index);
        EXPECT_EQ(x, fr(1));

        EXPECT_EQ(prover.n, 16UL);
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_equality_false()
    {
        Composer composer = Composer();

        field_ct a(witness_ct(&composer, 4));
        field_ct b(witness_ct(&composer, 3));
        bool_ct r = a == b;

        EXPECT_EQ(r.get_value(), false);

        auto prover = composer.create_prover();

        fr x = composer.get_variable(r.witness_index);
        EXPECT_EQ(x, fr(0));

        EXPECT_EQ(prover.n, 16UL);
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_equality_with_constants()
    {
        Composer composer = Composer();

        field_ct a(witness_ct(&composer, 4));
        field_ct b = 3;
        field_ct c = 7;
        bool_ct r = a * c == b * c + c && b + 1 == a;

        EXPECT_EQ(r.get_value(), true);

        auto prover = composer.create_prover();

        fr x = composer.get_variable(r.witness_index);
        EXPECT_EQ(x, fr(1));

        EXPECT_EQ(prover.n, 16UL);
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_larger_circuit()
    {
        size_t n = 16384;
        Composer composer = Composer("../srs_db/ignition", n);

        generate_test_plonk_circuit(composer, n);

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void is_zero()
    {
        Composer composer = Composer();

        // yuck
        field_ct a = (public_witness_ct(&composer, fr::random_element()));
        field_ct b = (public_witness_ct(&composer, fr::neg_one()));
        field_ct c_1(&composer,
                     uint256_t(0x1122334455667788, 0x8877665544332211, 0xaabbccddeeff9933, 0x1122112211221122));
        field_ct c_2(&composer,
                     uint256_t(0xaabbccddeeff9933, 0x8877665544332211, 0x1122334455667788, 0x1122112211221122));
        field_ct c_3(&composer, barretenberg::fr::one());

        field_ct c_4 = c_1 + c_2;
        a = a * c_4 + c_4; // add some constant terms in to validate our normalization check works
        b = b * c_4 + c_4;
        b = (b - c_1 - c_2) / c_4;
        b = b + c_3;

        field_ct d(&composer, fr::zero());
        field_ct e(&composer, fr::one());

        const size_t old_n = composer.get_num_gates();
        bool_ct d_zero = d.is_zero();
        bool_ct e_zero = e.is_zero();
        const size_t new_n = composer.get_num_gates();
        EXPECT_EQ(old_n, new_n);

        bool_ct a_zero = a.is_zero();
        bool_ct b_zero = b.is_zero();

        EXPECT_EQ(a_zero.get_value(), false);
        EXPECT_EQ(b_zero.get_value(), true);
        EXPECT_EQ(d_zero.get_value(), true);
        EXPECT_EQ(e_zero.get_value(), false);

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void madd()
    {
        Composer composer = Composer();

        field_ct a(witness_ct(&composer, fr::random_element()));
        field_ct b(witness_ct(&composer, fr::random_element()));
        field_ct c(witness_ct(&composer, fr::random_element()));
        field_ct ma(&composer, fr::random_element());
        field_ct ca(&composer, fr::random_element());
        field_ct mb(&composer, fr::random_element());
        field_ct cb(&composer, fr::random_element());
        field_ct mc(&composer, fr::random_element());
        field_ct cc(&composer, fr::random_element());

        // test madd when all operands are witnesses
        field_ct d = a * ma + ca;
        field_ct e = b * mb + cb;
        field_ct f = c * mc + cc;
        field_ct g = d.madd(e, f);
        field_ct h = d * e + f;
        h = h.normalize();
        g = g.normalize();
        EXPECT_EQ(g.get_value(), h.get_value());

        // test madd when to_add = constant
        field_ct i = a.madd(b, ma);
        field_ct j = a * b + ma;
        i = i.normalize();
        j = j.normalize();
        EXPECT_EQ(i.get_value(), j.get_value());

        // test madd when to_mul = constant
        field_ct k = a.madd(mb, c);
        field_ct l = a * mb + c;
        k = k.normalize();
        l = l.normalize();
        EXPECT_EQ(k.get_value(), l.get_value());

        // test madd when lhs is constant
        field_ct m = ma.madd(b, c);
        field_ct n = ma * b + c;
        m = m.normalize();
        n = n.normalize();
        EXPECT_EQ(m.get_value(), n.get_value());

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void two_bit_table()
    {
        Composer composer = Composer();
        field_ct a(witness_ct(&composer, fr::random_element()));
        field_ct b(witness_ct(&composer, fr::random_element()));
        field_ct c(witness_ct(&composer, fr::random_element()));
        field_ct d(witness_ct(&composer, fr::random_element()));

        std::array<field_ct, 4> table = field_ct::preprocess_two_bit_table(a, b, c, d);

        bool_ct zero(witness_ct(&composer, false));
        bool_ct one(witness_ct(&composer, true));

        field_ct result_a = field_ct::select_from_two_bit_table(table, zero, zero).normalize();
        field_ct result_b = field_ct::select_from_two_bit_table(table, zero, one).normalize();
        field_ct result_c = field_ct::select_from_two_bit_table(table, one, zero).normalize();
        field_ct result_d = field_ct::select_from_two_bit_table(table, one, one).normalize();

        EXPECT_EQ(result_a.get_value(), a.get_value());
        EXPECT_EQ(result_b.get_value(), b.get_value());
        EXPECT_EQ(result_c.get_value(), c.get_value());
        EXPECT_EQ(result_d.get_value(), d.get_value());

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_slice()
    {
        Composer composer = Composer();
        // 0b11110110101001011
        //         ^      ^
        //        msb    lsb
        //        10      3
        // hi=0x111101, lo=0x011, slice=0x10101001
        //
        field_ct a(witness_ct(&composer, fr(126283)));
        auto slice_data = a.slice(10, 3);

        EXPECT_EQ(slice_data[0].get_value(), fr(3));
        EXPECT_EQ(slice_data[1].get_value(), fr(169));
        EXPECT_EQ(slice_data[2].get_value(), fr(61));

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_slice_equal_msb_lsb()
    {
        Composer composer = Composer();
        // 0b11110110101001011
        //             ^
        //         msb = lsb
        //             6
        // hi=0b1111011010, lo=0b001011, slice=0b1
        //
        field_ct a(witness_ct(&composer, fr(126283)));
        auto slice_data = a.slice(6, 6);

        EXPECT_EQ(slice_data[0].get_value(), fr(11));
        EXPECT_EQ(slice_data[1].get_value(), fr(1));
        EXPECT_EQ(slice_data[2].get_value(), fr(986));

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_slice_random()
    {
        Composer composer = Composer();

        uint8_t lsb = 106;
        uint8_t msb = 189;
        fr a_ = fr(uint256_t(fr::random_element()) && ((uint256_t(1) << 252) - 1));
        field_ct a(witness_ct(&composer, a_));
        auto slice = a.slice(msb, lsb);

        const uint256_t expected0 = uint256_t(a_) & ((uint256_t(1) << uint64_t(lsb)) - 1);
        const uint256_t expected1 = (uint256_t(a_) >> lsb) & ((uint256_t(1) << (uint64_t(msb - lsb) + 1)) - 1);
        const uint256_t expected2 = (uint256_t(a_) >> (msb + 1)) & ((uint256_t(1) << (uint64_t(252 - msb) - 1)) - 1);

        EXPECT_EQ(slice[0].get_value(), fr(expected0));
        EXPECT_EQ(slice[1].get_value(), fr(expected1));
        EXPECT_EQ(slice[2].get_value(), fr(expected2));

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void three_bit_table()
    {
        Composer composer = Composer();
        field_ct a(witness_ct(&composer, fr::random_element()));
        field_ct b(witness_ct(&composer, fr::random_element()));
        field_ct c(witness_ct(&composer, fr::random_element()));
        field_ct d(witness_ct(&composer, fr::random_element()));
        field_ct e(witness_ct(&composer, fr::random_element()));
        field_ct f(witness_ct(&composer, fr::random_element()));
        field_ct g(witness_ct(&composer, fr::random_element()));
        field_ct h(witness_ct(&composer, fr::random_element()));

        std::array<field_ct, 8> table = field_ct::preprocess_three_bit_table(a, b, c, d, e, f, g, h);

        bool_ct zero(witness_ct(&composer, false));
        bool_ct one(witness_ct(&composer, true));

        field_ct result_a = field_ct::select_from_three_bit_table(table, zero, zero, zero).normalize();
        field_ct result_b = field_ct::select_from_three_bit_table(table, zero, zero, one).normalize();
        field_ct result_c = field_ct::select_from_three_bit_table(table, zero, one, zero).normalize();
        field_ct result_d = field_ct::select_from_three_bit_table(table, zero, one, one).normalize();
        field_ct result_e = field_ct::select_from_three_bit_table(table, one, zero, zero).normalize();
        field_ct result_f = field_ct::select_from_three_bit_table(table, one, zero, one).normalize();
        field_ct result_g = field_ct::select_from_three_bit_table(table, one, one, zero).normalize();
        field_ct result_h = field_ct::select_from_three_bit_table(table, one, one, one).normalize();

        EXPECT_EQ(result_a.get_value(), a.get_value());
        EXPECT_EQ(result_b.get_value(), b.get_value());
        EXPECT_EQ(result_c.get_value(), c.get_value());
        EXPECT_EQ(result_d.get_value(), d.get_value());
        EXPECT_EQ(result_e.get_value(), e.get_value());
        EXPECT_EQ(result_f.get_value(), f.get_value());
        EXPECT_EQ(result_g.get_value(), g.get_value());
        EXPECT_EQ(result_h.get_value(), h.get_value());

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    /**
     * @brief Test success and failure cases for decompose_into_bits.
     *
     * @details The target function constructs `sum` from a supplied collection of bits and compares it with a value
     * `val_256`. We supply bit vectors to test some failure cases.
     */

    static void decompose_into_bits()
    {
        using witness_supplier_type = std::function<witness_ct(Composer * ctx, uint64_t, uint256_t)>;

        // check that constraints are satisfied for a variety of inputs
        auto run_success_test = [&]() {
            Composer composer = Composer();

            constexpr uint256_t modulus_minus_one = fr::modulus - 1;
            const fr p_lo = modulus_minus_one.slice(0, 130);

            std::vector<barretenberg::fr> test_elements = {
                barretenberg::fr::random_element(),
                0,
                -1,
                barretenberg::fr(static_cast<uint256_t>(engine.get_random_uint8())),
                barretenberg::fr((static_cast<uint256_t>(1) << 130) + 1 + p_lo)
            };

            for (auto a_expected : test_elements) {
                field_ct a = witness_ct(&composer, a_expected);
                std::vector<bool_ct> c = a.decompose_into_bits(256);
                fr bit_sum = 0;
                for (size_t i = 0; i < c.size(); i++) {
                    fr scaling_factor_value = fr(2).pow(static_cast<uint64_t>(i));
                    bit_sum += (fr(c[i].get_value()) * scaling_factor_value);
                }
                EXPECT_EQ(bit_sum, a_expected);
            };

            auto prover = composer.create_prover();
            auto verifier = composer.create_verifier();
            waffle::plonk_proof proof = prover.construct_proof();
            bool verified = verifier.verify_proof(proof);
            ASSERT_TRUE(verified);
        };

        // Now try to supply unintended witness values and test for failure.
        // Fr::modulus is equivalent to zero in Fr, but this should be forbidden by a range constraint.
        witness_supplier_type supply_modulus_bits = [](Composer* ctx, uint64_t j, uint256_t val_256) {
            ignore_unused(val_256);
            // use this to get `sum` to be fr::modulus.
            return witness_ct(ctx, fr::modulus.get_bit(j));
        };

        // design a bit vector that will pass all range constraints, but it fails the copy constraint.
        witness_supplier_type supply_half_modulus_bits = [](Composer* ctx, uint64_t j, uint256_t val_256) {
            // use this to fit y_hi into 128 bits
            if (j > 127) {
                return witness_ct(ctx, val_256.get_bit(j));
            };
            return witness_ct(ctx, (fr::modulus).get_bit(j));
        };

        auto run_failure_test = [&](witness_supplier_type witness_supplier) {
            Composer composer = Composer();

            fr a_expected = 0;
            field_ct a = witness_ct(&composer, a_expected);
            std::vector<bool_ct> c = a.decompose_into_bits(256, witness_supplier);

            auto prover = composer.create_prover();
            auto verifier = composer.create_verifier();
            waffle::plonk_proof proof = prover.construct_proof();
            bool verified = verifier.verify_proof(proof);
            ASSERT_FALSE(verified);
        };

        run_success_test();
        run_failure_test(supply_modulus_bits);
        run_failure_test(supply_half_modulus_bits);
    }

    static void test_assert_is_in_set()
    {
        Composer composer = Composer();

        field_ct a(witness_ct(&composer, fr(1)));
        field_ct b(witness_ct(&composer, fr(2)));
        field_ct c(witness_ct(&composer, fr(3)));
        field_ct d(witness_ct(&composer, fr(4)));
        field_ct e(witness_ct(&composer, fr(5)));
        std::vector<field_ct> set = { a, b, c, d, e };

        a.assert_is_in_set(set);

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        info("composer gates = ", composer.get_num_gates());
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_assert_is_in_set_fails()
    {
        Composer composer = Composer();

        field_ct a(witness_ct(&composer, fr(1)));
        field_ct b(witness_ct(&composer, fr(2)));
        field_ct c(witness_ct(&composer, fr(3)));
        field_ct d(witness_ct(&composer, fr(4)));
        field_ct e(witness_ct(&composer, fr(5)));
        std::vector<field_ct> set = { a, b, c, d, e };

        field_ct f(witness_ct(&composer, fr(6)));
        f.assert_is_in_set(set);

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        info("composer gates = ", composer.get_num_gates());
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }

    static void test_pow()
    {
        Composer composer = Composer();

        barretenberg::fr base_val(engine.get_random_uint256());
        uint32_t exponent_val = engine.get_random_uint32();

        field_ct base = witness_ct(&composer, base_val);
        field_ct exponent = witness_ct(&composer, exponent_val);
        field_ct result = base.pow(exponent);
        barretenberg::fr expected = base_val.pow(exponent_val);

        EXPECT_EQ(result.get_value(), expected);

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        auto proof = prover.construct_proof();
        info("composer gates = ", composer.get_num_gates());
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_pow_zero()
    {
        Composer composer = Composer();

        barretenberg::fr base_val(engine.get_random_uint256());
        uint32_t exponent_val = 0;

        field_ct base = witness_ct(&composer, base_val);
        field_ct exponent = witness_ct(&composer, exponent_val);
        field_ct result = base.pow(exponent);

        EXPECT_EQ(result.get_value(), barretenberg::fr(1));

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        auto proof = prover.construct_proof();
        info("composer gates = ", composer.get_num_gates());
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_pow_one()
    {
        Composer composer = Composer();

        barretenberg::fr base_val(engine.get_random_uint256());
        uint32_t exponent_val = 1;

        field_ct base = witness_ct(&composer, base_val);
        field_ct exponent = witness_ct(&composer, exponent_val);
        field_ct result = base.pow(exponent);

        EXPECT_EQ(result.get_value(), base_val);

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        auto proof = prover.construct_proof();
        info("composer gates = ", composer.get_num_gates());
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_pow_both_constant()
    {
        Composer composer = Composer();

        const size_t num_gates_start = composer.n;

        barretenberg::fr base_val(engine.get_random_uint256());
        uint32_t exponent_val = engine.get_random_uint32();

        field_ct base(&composer, base_val);
        field_ct exponent(&composer, exponent_val);
        field_ct result = base.pow(exponent);
        barretenberg::fr expected = base_val.pow(exponent_val);

        EXPECT_EQ(result.get_value(), expected);

        const size_t num_gates_end = composer.n;
        EXPECT_EQ(num_gates_start, num_gates_end);
    }

    static void test_pow_base_constant()
    {
        Composer composer = Composer();

        barretenberg::fr base_val(engine.get_random_uint256());
        uint32_t exponent_val = engine.get_random_uint32();

        field_ct base(&composer, base_val);
        field_ct exponent = witness_ct(&composer, exponent_val);
        field_ct result = base.pow(exponent);
        barretenberg::fr expected = base_val.pow(exponent_val);

        EXPECT_EQ(result.get_value(), expected);

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        auto proof = prover.construct_proof();
        info("composer gates = ", composer.get_num_gates());
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_pow_exponent_constant()
    {
        Composer composer = Composer();

        barretenberg::fr base_val(engine.get_random_uint256());
        uint32_t exponent_val = engine.get_random_uint32();

        field_ct base = witness_ct(&composer, base_val);
        field_ct exponent(&composer, exponent_val);
        field_ct result = base.pow(exponent);
        barretenberg::fr expected = base_val.pow(exponent_val);

        EXPECT_EQ(result.get_value(), expected);

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        auto proof = prover.construct_proof();
        info("composer gates = ", composer.get_num_gates());
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    };

    static void test_pow_exponent_out_of_range()
    {
        Composer composer = Composer();

        barretenberg::fr base_val(engine.get_random_uint256());
        uint64_t exponent_val = engine.get_random_uint32();
        exponent_val += (uint64_t(1) << 32);

        field_ct base = witness_ct(&composer, base_val);
        field_ct exponent = witness_ct(&composer, exponent_val);
        field_ct result = base.pow(exponent);
        barretenberg::fr expected = base_val.pow(exponent_val);

        EXPECT_NE(result.get_value(), expected);
        EXPECT_EQ(composer.failed(), true);
        EXPECT_EQ(composer.err(), "field_t::pow exponent accumulator incorrect");
    };
};

typedef testing::
    Types<waffle::UltraComposer, waffle::TurboComposer, waffle::StandardComposer, honk::StandardHonkComposer>
        ComposerTypes;

TYPED_TEST_SUITE(stdlib_field, ComposerTypes);

TYPED_TEST(stdlib_field, test_create_range_constraint)
{
    TestFixture::create_range_constraint();
}
TYPED_TEST(stdlib_field, test_assert_equal)
{
    TestFixture::test_assert_equal();
}
TYPED_TEST(stdlib_field, test_add_mul_with_constants)
{
    TestFixture::test_add_mul_with_constants();
}
TYPED_TEST(stdlib_field, test_div)
{
    TestFixture::test_div();
}
TYPED_TEST(stdlib_field, test_field_fibbonaci)
{
    TestFixture::test_field_fibbonaci();
}
TYPED_TEST(stdlib_field, test_field_pythagorean)
{
    TestFixture::test_field_pythagorean();
}
TYPED_TEST(stdlib_field, test_equality)
{
    TestFixture::test_equality();
}
TYPED_TEST(stdlib_field, test_equality_false)
{
    TestFixture::test_equality_false();
}
TYPED_TEST(stdlib_field, test_equality_with_constants)
{
    TestFixture::test_equality_with_constants();
}
TYPED_TEST(stdlib_field, test_larger_circuit)
{
    TestFixture::test_larger_circuit();
}
TYPED_TEST(stdlib_field, is_zero)
{
    TestFixture::is_zero();
}
TYPED_TEST(stdlib_field, madd)
{
    TestFixture::madd();
}
TYPED_TEST(stdlib_field, two_bit_table)
{
    TestFixture::two_bit_table();
}
TYPED_TEST(stdlib_field, test_slice)
{
    TestFixture::test_slice();
}
TYPED_TEST(stdlib_field, test_slice_equal_msb_lsb)
{
    TestFixture::test_slice_equal_msb_lsb();
}
TYPED_TEST(stdlib_field, test_slice_random)
{
    TestFixture::test_slice_random();
}
TYPED_TEST(stdlib_field, three_bit_table)
{
    TestFixture::three_bit_table();
}
TYPED_TEST(stdlib_field, decompose_into_bits)
{
    TestFixture::decompose_into_bits();
}
TYPED_TEST(stdlib_field, test_assert_is_in_set)
{
    TestFixture::test_assert_is_in_set();
}
TYPED_TEST(stdlib_field, test_assert_is_in_set_fails)
{
    TestFixture::test_assert_is_in_set_fails();
}
TYPED_TEST(stdlib_field, test_pow)
{
    TestFixture::test_pow();
}
TYPED_TEST(stdlib_field, test_pow_zero)
{
    TestFixture::test_pow_zero();
}
TYPED_TEST(stdlib_field, test_pow_one)
{
    TestFixture::test_pow_one();
}
TYPED_TEST(stdlib_field, test_pow_both_constant)
{
    TestFixture::test_pow_both_constant();
}
TYPED_TEST(stdlib_field, test_pow_base_constant)
{
    TestFixture::test_pow_base_constant();
}
TYPED_TEST(stdlib_field, test_pow_exponent_constant)
{
    TestFixture::test_pow_exponent_constant();
}
TYPED_TEST(stdlib_field, test_pow_exponent_out_of_range)
{
    TestFixture::test_pow_exponent_out_of_range();
}
} // namespace test_stdlib_field