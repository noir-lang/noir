#include "public_inputs.hpp"
#include <gtest/gtest.h>
#include <polynomials/evaluation_domain.hpp>

using namespace barretenberg;

/*
```
elliptic curve point addition on a short weierstrass curve.

circuit has 9 gates, I've added 7 dummy gates so that the polynomial degrees are a power of 2

input points: (x_1, y_1), (x_2, y_2)
output point: (x_3, y_3)
intermediate variables: (t_1, t_2, t_3, t_4, t_5, t_6, t_7)

Variable assignments:
t_1 = (y_2 - y_1)
t_2 = (x_2 - x_1)
t_3 = (y_2 - y_1) / (x_2 - x_1)
x_3 = t_3*t_3 - x_2 - x_1
y_3 = t_3*(x_1 - x_3) - y_1
t_4 = (x_3 + x_1)
t_5 = (t_4 + x_2)
t_6 = (y_3 + y_1)
t_7 = (x_1 - x_3)

Constraints:
(y_2 - y_1) - t_1 = 0
(x_2 - x_1) - t_2 = 0
(x_1 + x_2) - t_4 = 0
(t_4 + x_3) - t_5 = 0
(y_3 + y_1) - t_6 = 0
(x_1 - x_3) - t_7 = 0
 (t_3 * t_2) - t_1 = 0
-(t_3 * t_3) + t_5 = 0
-(t_3 * t_7) + t_6 = 0

Wire polynomials:
w_l = [y_2, x_2, x_1, t_4, y_3, x_1, t_3, t_3, t_3, 0, 0, 0, 0, 0, 0, 0]
w_r = [y_1, x_1, x_2, x_3, y_1, x_3, t_2, t_3, t_7, 0, 0, 0, 0, 0, 0, 0]
w_o = [t_1, t_2, t_4, t_5, t_6, t_7, t_1, t_5, t_6, 0, 0, 0, 0, 0, 0, 0]

Gate polynomials:
q_m = [ 0,  0,  0,  0,  0,  0,  1, -1, -1, 0, 0, 0, 0, 0, 0, 0]
q_l = [ 1,  1,  1,  1,  1,  1,  0,  0,  0, 0, 0, 0, 0, 0, 0, 0]
q_r = [-1, -1,  1,  1,  1, -1,  0,  0,  0, 0, 0, 0, 0, 0, 0, 0]
q_o = [-1, -1, -1, -1, -1, -1, -1,  1,  1, 0, 0, 0, 0, 0, 0, 0]
q_c = [ 0,  0,  0,  0,  0,  0,  0,  0,  0, 0, 0, 0, 0, 0, 0, 0]

Permutation polynomials:
s_id = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
sigma_1 = [1, 3+n, 6, 3+2n, 5, 2+n, 8, 9, 8+n, 10, 11, 12, 13, 14, 15, 16]
sigma_2 = [5+n, 3, 2, 6+n, 1+n, 4+n, 2+2n, 7, 6+2n, 10+n, 11+n, 12+n, 13+n, 14+n, 15+n, 16+n]
sigma_3 = [7+2n, 7+n, 4, 8+2n, 9+2n, 9+n, 1+2n, 4+2n, 5+2n, 10+2n, 11+2n, 12+2n, 13+2n, 14+2n, 15+2n]

(for n = 16, permutation polynomials are)
sigma_1 = [1, 19, 6, 35, 5, 18, 8, 9, 24, 10, 11, 12, 13, 14, 15, 16]
sigma_2 = [21, 3, 2, 22, 17, 20, 34, 7, 38, 26, 27, 28, 29, 30, 31, 32]
sigma_3 = [39, 23, 4, 40, 41, 25, 33, 36, 37, 42, 43, 44, 45, 46, 47, 48]
```
*/
using namespace barretenberg;
using namespace waffle;

namespace {

TEST(test_public_inputs, compute_delta)
{
    constexpr uint32_t circuit_size = 256;
    constexpr size_t num_public_inputs = 7;

    evaluation_domain domain(circuit_size);

    std::vector<fr> left;
    std::vector<fr> right;
    std::vector<fr> sigma_1;
    std::vector<fr> sigma_2;

    fr work_root = fr::one();
    for (size_t i = 0; i < circuit_size; ++i) {
        fr temp = fr::random_element();
        left.push_back(temp);
        right.push_back(temp);
        sigma_1.push_back((fr::coset_generator(0) * work_root));
        sigma_2.push_back(work_root);
        work_root = work_root * domain.root;
    }

    fr beta = fr::random_element();
    fr gamma = fr::random_element();
    fr root = domain.root;
    const auto compute_grand_product = [root, beta, gamma](std::vector<fr>& left,
                                                           std::vector<fr>& right,
                                                           std::vector<fr>& sigma_1,
                                                           std::vector<fr>& sigma_2) {
        fr numerator = fr::one();
        fr denominator = fr::one();
        fr work_root = fr::one();
        for (size_t i = 0; i < circuit_size; ++i) {
            fr T0 = left[i] + gamma;
            fr T1 = right[i] + gamma;

            fr T2 = work_root * beta;
            fr T3 = fr::coset_generator(0) * T2;

            fr T4 = T0 + T2;
            fr T5 = T1 + T3;
            fr T6 = T4 * T5;

            numerator = numerator * T6;

            fr T7 = (T0 + sigma_1[i] * beta);
            fr T8 = (T1 + sigma_2[i] * beta);
            fr T9 = T7 * T8;
            denominator = denominator * T9;
            work_root = work_root * root;
        }

        denominator = denominator.invert();

        fr product = numerator * denominator;
        return product;
    };

    fr init_result = compute_grand_product(left, right, sigma_1, sigma_2);

    EXPECT_EQ((init_result == fr::one()), true);

    work_root = fr::one();
    for (size_t i = 0; i < num_public_inputs; ++i) {
        sigma_1[i] = work_root;
        work_root = work_root * domain.root;
    }

    fr modified_result = compute_grand_product(left, right, sigma_1, sigma_2);

    std::vector<fr> public_inputs;
    for (size_t i = 0; i < num_public_inputs; ++i) {
        public_inputs.push_back(left[i]);
    }
    fr target_delta = waffle::compute_public_input_delta<fr>(public_inputs, beta, gamma, domain.root);

    EXPECT_EQ((modified_result == target_delta), true);
}
} // namespace