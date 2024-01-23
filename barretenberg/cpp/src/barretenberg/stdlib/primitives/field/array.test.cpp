#include "array.hpp"
#include "../bool/bool.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders.hpp"
#include "field.hpp"
#include <gtest/gtest.h>
#include <utility>

using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();
}

template <class T> void ignore_unused(T&) {} // use to ignore unused variables in lambdas

template <typename Builder> class stdlib_array : public testing::Test {
    typedef stdlib::bool_t<Builder> bool_ct;
    typedef stdlib::field_t<Builder> field_ct;
    typedef stdlib::witness_t<Builder> witness_ct;
    typedef stdlib::public_witness_t<Builder> public_witness_ct;

  public:
    static void test_array_length()
    {
        Builder builder = Builder();

        constexpr size_t ARRAY_LEN = 10;
        std::array<fr, ARRAY_LEN> values;
        std::array<field_ct, ARRAY_LEN> values_ct;

        constexpr size_t filled = 6;
        for (size_t i = 0; i < filled; i++) {
            values[i] = fr::random_element();
            values_ct[i] = witness_ct(&builder, values[i]);
        }
        auto filled_len = array_length<Builder>(values_ct);
        EXPECT_EQ(filled_len.get_value(), filled);

        info("num gates = ", builder.get_num_gates());
        bool proof_result = builder.check_circuit();
        EXPECT_EQ(proof_result, true);
    }

    static void test_array_length_null()
    {
        Builder builder = Builder();

        std::array<field_ct, 0> values_ct;
        auto filled_len = array_length<Builder>(values_ct);
        EXPECT_EQ(filled_len.get_value(), 0);
        EXPECT_TRUE(filled_len.is_constant());

        info("num gates = ", builder.get_num_gates());
        bool proof_result = builder.check_circuit();
        EXPECT_EQ(proof_result, true);
    }

    static void test_array_length_fails()
    {
        Builder builder = Builder();

        constexpr size_t ARRAY_LEN = 10;
        std::array<fr, ARRAY_LEN> values;
        std::array<field_ct, ARRAY_LEN> values_ct;

        constexpr size_t filled = 6;
        for (size_t i = 0; i < filled; i++) {
            values[i] = fr::random_element();
            values_ct[i] = witness_ct(&builder, values[i]);
        }

        // Put a zero in the middle of the array, so that the array_length function complains that all zeros thereafter
        // should be zero.
        values_ct[4] = 0;

        array_length<Builder>(values_ct);

        EXPECT_EQ(builder.failed(), true);
        EXPECT_EQ(builder.err(), "Once we've hit the first zero, there must only be zeros thereafter!");
    }

    static void test_array_pop()
    {
        Builder builder = Builder();

        constexpr size_t ARRAY_LEN = 10;
        std::array<fr, ARRAY_LEN> values;
        std::array<field_ct, ARRAY_LEN> values_ct;

        constexpr size_t filled = 6;
        for (size_t i = 0; i < filled; i++) {
            values[i] = fr::random_element();
            values_ct[i] = witness_ct(&builder, values[i]);
        }
        auto popped = array_pop<Builder>(values_ct);
        EXPECT_EQ(popped.get_value(), values[filled - 1]);

        info("num gates = ", builder.get_num_gates());
        bool proof_result = builder.check_circuit();
        EXPECT_EQ(proof_result, true);
    };

    static void test_array_pop_from_empty()
    {
        Builder builder = Builder();

        constexpr size_t ARRAY_LEN = 10;
        std::array<fr, ARRAY_LEN> values;
        std::array<field_ct, ARRAY_LEN> values_ct;

        constexpr size_t filled = 0;
        for (size_t i = 0; i < filled; i++) {
            values[i] = fr::random_element();
            values_ct[i] = witness_ct(&builder, values[i]);
        }
        for (size_t i = filled; i < ARRAY_LEN; i++) {
            values[i] = 0;
            values_ct[i] = witness_ct(&builder, values[i]);
        }

        auto popped = array_pop<Builder>(values_ct);
        EXPECT_EQ(popped.get_value(), 0);

        EXPECT_EQ(builder.failed(), true);
        EXPECT_EQ(builder.err(), "array_pop cannot pop from an empty array");
    };

    static void test_array_push()
    {
        Builder builder = Builder();

        constexpr size_t ARRAY_LEN = 10;
        std::array<fr, ARRAY_LEN> values;
        std::array<field_ct, ARRAY_LEN> values_ct;

        constexpr size_t filled = 6;
        for (size_t i = 0; i < filled; i++) {
            values[i] = fr::random_element();
            values_ct[i] = witness_ct(&builder, values[i]);
        }
        for (size_t i = filled; i < ARRAY_LEN; i++) {
            values[i] = 0;
            values_ct[i] = witness_ct(&builder, values[i]);
        }

        auto value_ct = field_ct(&builder, fr::random_element());
        array_push<Builder>(values_ct, value_ct);
        EXPECT_EQ(value_ct.get_value(), values_ct[filled].get_value());

        info("num gates = ", builder.get_num_gates());
        bool proof_result = builder.check_circuit();
        EXPECT_EQ(proof_result, true);
    }

    static void test_array_push_optional()
    {
        Builder builder = Builder();

        constexpr size_t ARRAY_LEN = 10;
        std::array<std::optional<fr>, ARRAY_LEN> values;
        std::array<std::optional<field_ct>, ARRAY_LEN> values_ct;

        // Fill the array with some values
        for (size_t i = 0; i < ARRAY_LEN; i++) {
            values[i] = std::nullopt;
            values_ct[i] = std::nullopt;
        }

        // Push some values into the array
        size_t num_pushes = 0;
        for (size_t i = 0; i < ARRAY_LEN; i++) {
            auto value = field_ct(&builder, fr::random_element());
            size_t idx = array_push<Builder>(values_ct, value);
            EXPECT_TRUE(values_ct[idx].has_value());
            EXPECT_EQ(values_ct[idx].value().get_value(), value.get_value());
            num_pushes++;
        }

        // Make sure the array is full now
        try {
            auto value = field_ct(&builder, fr::random_element());
            array_push<Builder>(values_ct, value);
            FAIL() << "array_push should have thrown an exception when trying to push to a full array";
        } catch (std::runtime_error& e) {
            EXPECT_EQ(e.what(), std::string("array_push cannot push to a full array"));
        }

        EXPECT_EQ(num_pushes, ARRAY_LEN);

        info("num gates = ", builder.get_num_gates());
        bool proof_result = builder.check_circuit();
        EXPECT_EQ(proof_result, true);
    }

    static void test_array_push_shared_ptr()
    {
        constexpr size_t ARRAY_LEN = 5;
        std::array<std::shared_ptr<int>, ARRAY_LEN> arr;
        for (size_t i = 0; i < arr.size(); ++i) {
            arr[i] = nullptr;
        }

        // Fill the array up to capacity
        for (size_t i = 0; i < arr.size(); ++i) {
            arr[i] = std::make_shared<int>(i);
        }

        // Attempt to push a value to the array
        std::shared_ptr<int> new_value = std::make_shared<int>(42);
        EXPECT_THROW(array_push<Builder>(arr, new_value), std::runtime_error);

        // Ensure that the array was not modified
        for (size_t i = 0; i < arr.size(); ++i) {
            EXPECT_NE(arr[i], new_value);
        }
    }

    static void test_is_array_empty()
    {
        Builder builder = Builder();

        constexpr size_t ARRAY_LEN = 10;
        std::array<fr, ARRAY_LEN> values;
        std::array<field_ct, ARRAY_LEN> values_ct;

        // Test non-empty array
        constexpr size_t filled = 3;
        for (size_t i = 0; i < filled; i++) {
            values[i] = fr::random_element();
            values_ct[i] = witness_ct(&builder, values[i]);
        }
        for (size_t i = filled; i < ARRAY_LEN; i++) {
            values[i] = 0;
            values_ct[i] = witness_ct(&builder, values[i]);
        }
        auto is_empty = is_array_empty<Builder>(values_ct);
        EXPECT_EQ(is_empty.get_value(), false);

        // Test empty array
        for (size_t i = 0; i < ARRAY_LEN; i++) {
            values[i] = 0;
            values_ct[i] = witness_ct(&builder, values[i]);
        }
        is_empty = is_array_empty<Builder>(values_ct);
        EXPECT_EQ(is_empty.get_value(), true);

        info("num gates = ", builder.get_num_gates());
        bool proof_result = builder.check_circuit();
        EXPECT_EQ(proof_result, true);
    };

    template <size_t size_1, size_t size_2>
    static auto test_push_array_to_array_helper(Builder& builder,
                                                std::array<fr, size_1> const& source,
                                                std::array<fr, size_2> const& target,
                                                std::array<fr, size_2> const& expected_target,
                                                bool const expect_fail = false)
    {
        std::array<field_ct, size_1> source_ct;
        std::array<field_ct, size_2> target_ct;
        for (size_t i = 0; i < source.size(); i++) {
            source_ct[i] = witness_ct(&builder, source[i]);
        }
        for (size_t i = 0; i < target.size(); i++) {
            target_ct[i] = witness_ct(&builder, target[i]);
        }

        push_array_to_array<Builder>(source_ct, target_ct);

        // Check that the source array has been inserted into the first available index of the target array.
        if (!expect_fail) {
            for (size_t i = 0; i < target.size(); i++) {
                EXPECT_EQ(target_ct[i].get_value(), expected_target[i]);
            }
        }

        bool proof_result = false;
        if (!builder.failed()) {
            info("num gates = ", builder.get_num_gates());
            proof_result = builder.check_circuit();
        }

        return std::make_pair(proof_result, builder.err());
    }

    static void test_pata_large_bench()
    {
        // Benchmark
        Builder builder = Builder();

        std::array<fr, 64> source;
        std::array<fr, 128> target = { 0 };
        std::array<fr, 128> expected_target;
        for (size_t i = 0; i < source.max_size(); ++i) {
            source[i] = i + 1;
            target[i] = i + 1;
            expected_target[i] = i + 1;
            expected_target[i + source.max_size()] = i + 1;
        };
        bool proof_result;
        std::string error;
        std::tie(proof_result, error) = test_push_array_to_array_helper(builder, source, target, expected_target);

        EXPECT_TRUE(proof_result);
    }

    static void test_pata_same_size_not_full_to_not_full()
    {
        Builder builder = Builder();

        std::array<fr, 4> source = { 1, 0, 0, 0 };
        std::array<fr, 4> target = { 3, 0, 0, 0 };
        std::array<fr, 4> expected_target = { 3, 1, 0, 0 };
        bool proof_result;
        std::string error;
        std::tie(proof_result, error) = test_push_array_to_array_helper(builder, source, target, expected_target);

        EXPECT_TRUE(proof_result);
    }

    static void test_pata_same_size_not_full_to_not_full_2()
    {
        Builder builder = Builder();

        std::array<fr, 4> source = { 3, 4, 0, 0 };
        std::array<fr, 4> target = { 1, 2, 0, 0 };
        std::array<fr, 4> expected_target = { 1, 2, 3, 4 };
        bool proof_result;
        std::string error;
        std::tie(proof_result, error) = test_push_array_to_array_helper(builder, source, target, expected_target);

        EXPECT_TRUE(proof_result);
    }

    static void test_pata_same_size_not_full_to_empty()
    {
        Builder builder = Builder();

        std::array<fr, 4> source = { 1, 2, 3, 0 };
        std::array<fr, 4> target = { 0, 0, 0, 0 };
        std::array<fr, 4> expected_target = { 1, 2, 3, 0 };
        bool proof_result;
        std::string error;
        std::tie(proof_result, error) = test_push_array_to_array_helper(builder, source, target, expected_target);

        EXPECT_TRUE(proof_result);
    }

    static void test_pata_smaller_source_full_to_not_full()
    {
        Builder builder = Builder();

        std::array<fr, 3> source = { 1, 2, 3 };
        std::array<fr, 6> target = { 4, 5, 6, 0, 0, 0 };
        std::array<fr, 6> expected_target = { 4, 5, 6, 1, 2, 3 };
        bool proof_result;
        std::string error;
        std::tie(proof_result, error) = test_push_array_to_array_helper(builder, source, target, expected_target);

        EXPECT_TRUE(proof_result);
    }

    static void test_pata_null_source()
    {
        // null means array size is 0
        Builder builder = Builder();

        std::array<fr, 0> source;
        std::array<fr, 4> target = { 1, 2, 0, 0 };
        std::array<fr, 4> expected_target = { 1, 2, 0, 0 };
        bool proof_result;
        std::string error;
        std::tie(proof_result, error) = test_push_array_to_array_helper(builder, source, target, expected_target);

        EXPECT_TRUE(proof_result);
    }

    static void test_pata_null_target_fails()
    {
        Builder builder = Builder();

        std::array<fr, 4> source = { 1, 2, 0, 0 };
        std::array<fr, 0> target;
        std::array<fr, 0> expected_target;
        bool proof_result;
        std::string error;
        std::tie(proof_result, error) = test_push_array_to_array_helper(builder, source, target, expected_target);

        EXPECT_FALSE(proof_result);
        EXPECT_EQ(error, "push_array_to_array target array capacity exceeded");
    }

    static void test_pata_singletons_full_to_not_full()
    {
        Builder builder = Builder();

        std::array<fr, 1> source = { 1 };
        std::array<fr, 1> target = { 0 };
        std::array<fr, 1> expected_target = { 1 };
        bool proof_result;
        std::string error;
        std::tie(proof_result, error) = test_push_array_to_array_helper(builder, source, target, expected_target);

        EXPECT_TRUE(proof_result);
    }

    static void test_pata_singletons_not_full_to_full()
    {
        Builder builder = Builder();

        std::array<fr, 1> source = { 0 };
        std::array<fr, 1> target = { 1 };
        std::array<fr, 1> expected_target = { 1 };
        bool proof_result;
        std::string error;
        std::tie(proof_result, error) = test_push_array_to_array_helper(builder, source, target, expected_target);

        EXPECT_TRUE(proof_result);
    }

    static void test_pata_singletons_full_to_full()
    {
        Builder builder = Builder();

        std::array<fr, 1> source = { 2 };
        std::array<fr, 1> target = { 1 };
        std::array<fr, 1> expected_target = { 1 };
        bool proof_result;
        std::string error;
        std::tie(proof_result, error) = test_push_array_to_array_helper(builder, source, target, expected_target);

        EXPECT_FALSE(proof_result);
        EXPECT_EQ(error, "push_array_to_array target array capacity exceeded");
    }

    static void test_pata_same_size_full_to_full_fails()
    {
        Builder builder = Builder();

        std::array<fr, 5> source = { 1, 2, 3, 4, 5 };
        std::array<fr, 5> target = { 5, 6, 7, 8, 9 };
        std::array<fr, 5> expected_target = { 5, 6, 7, 8, 9 };
        bool proof_result;
        std::string error;
        bool expect_fail = true;
        std::tie(proof_result, error) =
            test_push_array_to_array_helper(builder, source, target, expected_target, expect_fail);

        EXPECT_FALSE(proof_result);
        EXPECT_EQ(error, "push_array_to_array target array capacity exceeded");
    }

    static void test_pata_nonzero_after_zero_source_fails()
    {
        Builder builder = Builder();

        std::array<fr, 4> source = { 1, 0, 2, 3 };
        std::array<fr, 6> target = { 4, 5, 6, 7, 8, 0 };
        std::array<fr, 6> expected_target = { 4, 5, 6, 7, 8, 0 };
        bool proof_result;
        std::string error;
        bool expect_fail = true;
        std::tie(proof_result, error) =
            test_push_array_to_array_helper(builder, source, target, expected_target, expect_fail);

        EXPECT_FALSE(proof_result);
        EXPECT_EQ(error, "Once we've hit the first source zero, there must only be zeros thereafter!");
    }

    static void test_pata_nonzero_after_zero_source_fails_2()
    {
        Builder builder = Builder();

        std::array<fr, 3> source = { 1, 0, 3 };
        std::array<fr, 6> target = { 4, 5, 2, 0, 0, 0 };
        std::array<fr, 6> expected_target = { 4, 5, 2, 1, 0, 3 };
        bool proof_result;
        std::string error;
        bool expect_fail = true;
        std::tie(proof_result, error) =
            test_push_array_to_array_helper(builder, source, target, expected_target, expect_fail);

        EXPECT_FALSE(proof_result);
        EXPECT_EQ(error, "Once we've hit the first source zero, there must only be zeros thereafter!");
    }

    static void test_pata_nonzero_after_zero_target_fails()
    {
        Builder builder = Builder();

        std::array<fr, 3> source = { 1, 2, 3 };
        std::array<fr, 6> target = { 4, 5, 0, 6, 7, 8 };
        std::array<fr, 6> expected_target = { 4, 5, 0, 6, 7, 8 };
        bool proof_result;
        std::string error;
        bool expect_fail = true;
        std::tie(proof_result, error) =
            test_push_array_to_array_helper(builder, source, target, expected_target, expect_fail);

        EXPECT_FALSE(proof_result);
        EXPECT_EQ(error, "Once we've hit the first zero, there must only be zeros thereafter!");
    }

    static void test_pata_nonzero_after_zero_target_fails_2()
    {
        Builder builder = Builder();

        std::array<fr, 3> source = { 1, 0, 3 };
        std::array<fr, 6> target = { 4, 5, 0, 6, 7, 8 };
        std::array<fr, 6> expected_target = { 4, 5, 0, 6, 7, 8 };
        bool proof_result;
        std::string error;
        bool expect_fail = true;
        std::tie(proof_result, error) =
            test_push_array_to_array_helper(builder, source, target, expected_target, expect_fail);

        EXPECT_FALSE(proof_result);
        EXPECT_EQ(error, "Once we've hit the first zero, there must only be zeros thereafter!");
    }

    class MockClass {
      public:
        MockClass()
            : m_a(field_ct(0))
            , m_b(field_ct(0))
        {}
        MockClass(field_ct a, field_ct b)
            : m_a(a)
            , m_b(b)
        {}

        bool_ct is_empty() const { return m_a == 0 && m_b == 0; }

        std::pair<field_ct, field_ct> get_values() { return std::make_pair(m_a, m_b); }

        void conditional_select(bool_ct const& condition, MockClass const& other)
        {
            m_a = field_ct::conditional_assign(condition, other.m_a, m_a);
            m_b = field_ct::conditional_assign(condition, other.m_b, m_b);
        }

      private:
        field_ct m_a;
        field_ct m_b;
    };

    void test_array_push_generic()
    {
        Builder builder = Builder();

        constexpr size_t SIZE = 5;
        std::array<MockClass, SIZE> arr{};

        // Push values into the array
        stdlib::array_push<Builder>(arr, MockClass(witness_ct(&builder, 1), witness_ct(&builder, 10)));
        stdlib::array_push<Builder>(arr, MockClass(witness_ct(&builder, 2), witness_ct(&builder, 20)));
        stdlib::array_push<Builder>(arr, MockClass(witness_ct(&builder, 3), witness_ct(&builder, 30)));

        // Check the values in the array
        EXPECT_EQ(arr[0].get_values().first.get_value(), 1);
        EXPECT_EQ(arr[0].get_values().second.get_value(), 10);
        EXPECT_EQ(arr[1].get_values().first.get_value(), 2);
        EXPECT_EQ(arr[1].get_values().second.get_value(), 20);
        EXPECT_EQ(arr[2].get_values().first.get_value(), 3);
        EXPECT_EQ(arr[2].get_values().second.get_value(), 30);

        info("num gates = ", builder.get_num_gates());
        bool proof_result = builder.check_circuit();
        EXPECT_EQ(proof_result, true);
    }

    void test_array_push_generic_full()
    {
        Builder builder = Builder();

        constexpr size_t SIZE = 5;
        std::array<MockClass, SIZE> arr{};

        // Push values into the array
        stdlib::array_push<Builder>(arr, MockClass(witness_ct(&builder, 1), witness_ct(&builder, 10)));
        stdlib::array_push<Builder>(arr, MockClass(witness_ct(&builder, 2), witness_ct(&builder, 20)));
        stdlib::array_push<Builder>(arr, MockClass(witness_ct(&builder, 3), witness_ct(&builder, 30)));
        stdlib::array_push<Builder>(arr, MockClass(witness_ct(&builder, 4), witness_ct(&builder, 40)));
        stdlib::array_push<Builder>(arr, MockClass(witness_ct(&builder, 5), witness_ct(&builder, 50)));

        // Try to push into a full array
        stdlib::array_push<Builder>(arr, MockClass(witness_ct(&builder, 6), witness_ct(&builder, 60)));

        EXPECT_EQ(builder.failed(), true);
        EXPECT_EQ(builder.err(), "array_push cannot push to a full array");
    }
};

typedef testing::Types<bb::StandardCircuitBuilder, bb::UltraCircuitBuilder> CircuitTypes;

TYPED_TEST_SUITE(stdlib_array, CircuitTypes);

TYPED_TEST(stdlib_array, test_array_length)
{
    TestFixture::test_array_length();
}
TYPED_TEST(stdlib_array, test_array_length_null)
{
    TestFixture::test_array_length_null();
}
TYPED_TEST(stdlib_array, test_array_length_fails)
{
    TestFixture::test_array_length_fails();
}
TYPED_TEST(stdlib_array, test_array_pop)
{
    TestFixture::test_array_pop();
}
TYPED_TEST(stdlib_array, test_array_pop_from_empty)
{
    TestFixture::test_array_pop_from_empty();
}
TYPED_TEST(stdlib_array, test_array_push)
{
    TestFixture::test_array_push();
}
TYPED_TEST(stdlib_array, test_array_push_optional)
{
    TestFixture::test_array_push_optional();
}
TYPED_TEST(stdlib_array, test_array_push_generic)
{
    TestFixture::test_array_push_generic();
}
TYPED_TEST(stdlib_array, test_array_push_generic_full)
{
    TestFixture::test_array_push_generic_full();
}
// push array to array (pata) tests
TYPED_TEST(stdlib_array, test_pata_large_bench)
{
    TestFixture::test_pata_large_bench();
}
TYPED_TEST(stdlib_array, test_pata_same_size_not_full_to_not_full)
{
    TestFixture::test_pata_same_size_not_full_to_not_full();
}
TYPED_TEST(stdlib_array, test_pata_same_size_not_full_to_not_full_2)
{
    TestFixture::test_pata_same_size_not_full_to_not_full_2();
}
TYPED_TEST(stdlib_array, test_pata_same_size_not_full_to_empty)
{
    TestFixture::test_pata_same_size_not_full_to_empty();
}
TYPED_TEST(stdlib_array, test_pata_smaller_source_full_to_not_full)
{
    TestFixture::test_pata_smaller_source_full_to_not_full();
}
TYPED_TEST(stdlib_array, test_pata_null_source)
{
    TestFixture::test_pata_null_source();
}
TYPED_TEST(stdlib_array, test_pata_null_target_fails)
{
    TestFixture::test_pata_null_target_fails();
}
TYPED_TEST(stdlib_array, test_pata_singletons_full_to_not_full)
{
    TestFixture::test_pata_singletons_full_to_not_full();
}
TYPED_TEST(stdlib_array, test_pata_singletons_not_full_to_full)
{
    TestFixture::test_pata_singletons_not_full_to_full();
}
TYPED_TEST(stdlib_array, test_pata_singletons_full_to_full)
{
    TestFixture::test_pata_singletons_full_to_full();
}
TYPED_TEST(stdlib_array, test_pata_same_size_full_to_full_fails)
{
    TestFixture::test_pata_same_size_full_to_full_fails();
}
TYPED_TEST(stdlib_array, test_pata_nonzero_after_zero_source_fails)
{
    TestFixture::test_pata_nonzero_after_zero_source_fails();
}
TYPED_TEST(stdlib_array, test_pata_nonzero_after_zero_source_fails_2)
{
    TestFixture::test_pata_nonzero_after_zero_source_fails_2();
}
TYPED_TEST(stdlib_array, test_pata_nonzero_after_zero_target_fails)
{
    TestFixture::test_pata_nonzero_after_zero_target_fails();
}
TYPED_TEST(stdlib_array, test_pata_nonzero_after_zero_target_fails_2)
{
    TestFixture::test_pata_nonzero_after_zero_target_fails_2();
}
