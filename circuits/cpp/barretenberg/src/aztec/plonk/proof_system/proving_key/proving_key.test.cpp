#include <common/test.hpp>
#include <common/streams.hpp>
#include "proving_key.hpp"
#include "serialize.hpp"

using namespace barretenberg;
using namespace waffle;

polynomial create_polynomial(size_t size)
{
    polynomial p;
    for (size_t i = 0; i < size; ++i) {
        p.add_coefficient(fr::random_element());
    }
    return p;
}

TEST(proving_key, buffer_serialization)
{
    proving_key_data key;
    key.n = 1234;
    key.num_public_inputs = 10;
    key.constraint_selectors["test1"] = create_polynomial(5);
    key.constraint_selectors["test2"] = create_polynomial(3);
    key.constraint_selector_ffts["foo1"] = create_polynomial(2);
    key.constraint_selector_ffts["foo2"] = create_polynomial(7);
    key.contains_recursive_proof = false;
    auto buf = to_buffer(key);
    auto result = from_buffer<proving_key_data>(buf);

    EXPECT_EQ(key, result);
}

TEST(proving_key, stream_serialization)
{
    proving_key_data key;
    key.n = 1234;
    key.num_public_inputs = 10;
    key.constraint_selectors["test1"] = create_polynomial(5);
    key.constraint_selectors["test2"] = create_polynomial(3);
    key.constraint_selector_ffts["foo1"] = create_polynomial(2);
    key.constraint_selector_ffts["foo2"] = create_polynomial(7);
    key.contains_recursive_proof = true;

    std::stringstream s;
    write(s, key);

    proving_key_data result;
    read(static_cast<std::istream&>(s), result);

    EXPECT_EQ(key, result);
}